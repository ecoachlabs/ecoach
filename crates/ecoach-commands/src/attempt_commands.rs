use chrono::Utc;
use ecoach_coach_brain::{
    CoachBrainTrigger, EvidenceInterpretationEngine, PedagogicalAttemptSignal,
    PedagogicalRuntimeService, evaluate_coach_brain, resolve_next_coach_action,
};
use ecoach_sessions::{SessionAnswerInput, SessionService};
use ecoach_student_model::{AnswerSubmission, StudentModelService};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Instant;

use crate::{error::CommandError, state::AppState};

/// Input for the submit_attempt hot path.
/// This is the primary command the frontend calls when a student answers a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAttemptInput {
    pub student_id: i64,
    pub session_id: i64,
    pub session_item_id: i64,
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
    pub hint_count: i64,
    pub changed_answer_count: i64,
    #[serde(default)]
    pub skipped: bool,
    #[serde(default)]
    pub timed_out: bool,
    pub was_timed: bool,
    #[serde(default)]
    pub defer_coach_brain: bool,
    /// Multi-correct MCQ: all option ids the student selected. When
    /// present, `submit_attempt` grades by set equality against the
    /// question's `is_correct` options and overrides the legacy
    /// single-option correctness check.
    #[serde(default)]
    pub selected_option_ids: Option<Vec<i64>>,
    /// Fill-in-the-blank: student's entered text per blank, ordered by
    /// blank index (1-based). Each blank is graded against the
    /// question_options rows whose `option_label` matches its index —
    /// case- and whitespace-insensitive match against any row is
    /// correct. A blank entry of "" (empty string) grades as incorrect.
    #[serde(default)]
    pub blank_answers: Option<Vec<String>>,
}

/// Comprehensive result DTO returned after the full hot path completes.
/// Contains everything the frontend needs to render feedback and next steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResultDto {
    pub attempt_id: i64,

    // Answer feedback
    pub is_correct: bool,
    pub explanation: Option<String>,
    pub correct_option_text: Option<String>,
    pub selected_option_text: Option<String>,
    pub misconception_info: Option<String>,

    // Error diagnosis (if wrong)
    pub error_type: Option<String>,
    pub diagnosis_summary: Option<String>,
    pub recommended_action: Option<String>,

    // Updated learner state
    pub updated_mastery: i64,
    pub updated_gap: i64,

    // Session progress
    pub session_answered: i64,
    pub session_remaining: i64,
    pub session_complete: bool,

    // Coach next action (step 8)
    pub next_action_type: String,
    pub next_action_title: String,
    pub next_action_route: String,
}

/// The canonical submit_attempt hot path (Section 7.4).
///
/// Orchestrates the full 10-step pipeline:
/// 1. Persist raw attempt event (via student model)
/// 2. Write attempt row (via student model)
/// 3. Classify outcome and misconception signal (via student model)
/// 4. Update evidence aggregates for affected SkillAtoms (via student model)
/// 5. Recompute skill state (via student model)
/// 6. Recompute curriculum-node rollups (via student model topic_state)
/// 7. Recompute pressure and memory effects (via student model)
/// 8. Recompute coach next action (via coach-brain)
/// 9. Update read models (on-read pattern — dashboards compute fresh)
/// 10. Enqueue secondary jobs (decay batch checked inline)
pub fn submit_attempt(
    state: &AppState,
    input: SubmitAttemptInput,
) -> Result<AttemptResultDto, CommandError> {
    let total_start = Instant::now();
    eprintln!(
        "[perf][rust.submit_attempt] enter session_id={} session_item_id={} question_id={} selected_option_id={:?}",
        input.session_id, input.session_item_id, input.question_id, input.selected_option_id
    );
    if input.selected_option_id.is_none() && !input.skipped && !input.timed_out {
        return Err(CommandError {
            code: "validation_error".to_string(),
            message: "attempt requires a selected option, skip, or timeout".to_string(),
        });
    }
    if input.skipped && input.timed_out {
        return Err(CommandError {
            code: "validation_error".to_string(),
            message: "attempt cannot be both skipped and timed out".to_string(),
        });
    }
    if input.selected_option_id.is_some() && (input.skipped || input.timed_out) {
        return Err(CommandError {
            code: "validation_error".to_string(),
            message: "attempt cannot include both an answer and a non-answer outcome".to_string(),
        });
    }

    state.with_connection(|conn| {
        let command_start = Instant::now();
        let now = Utc::now();
        let confidence_level = input.confidence_level.clone();
        let response_time_ms = input.response_time_ms.map(|value| value.max(0));

        // Phase 4 — server-side grading for multi-correct MCQ + fill-blank.
        //
        // When the caller supplied `selected_option_ids` or
        // `blank_answers`, we grade here (single DB round-trip for the
        // option list) and hand the result to the student model as
        // `precomputed_is_correct`. This keeps the downstream pipeline
        // intact while making backend stats accurate for the two newer
        // question types.
        let precomputed_is_correct: Option<bool> =
            if input.skipped || input.timed_out {
                None
            } else if input.selected_option_ids.is_some() || input.blank_answers.is_some() {
                Some(grade_attempt_server_side(conn, &input)?)
            } else {
                None
            };

        // Step 1-2: Record the answer in session_items
        let session_service = SessionService::new(conn);
        let session_lookup_start = Instant::now();
        let session_type = session_service
            .get_session(input.session_id)?
            .map(|session| session.session_type);
        eprintln!(
            "[perf][rust.submit_attempt] session.lookup {:.1}ms",
            session_lookup_start.elapsed().as_secs_f64() * 1000.0
        );
        let session_write_start = Instant::now();
        if input.defer_coach_brain {
            begin_immediate_transaction(conn)?;
            let write_result = (|| -> Result<(), CommandError> {
                if let Some(selected_option_id) = input.selected_option_id {
                    let _session_item = session_service.record_answer_without_attempt_row(
                        input.session_id,
                        &SessionAnswerInput {
                            item_id: input.session_item_id,
                            selected_option_id,
                            response_time_ms,
                        },
                    )?;
                } else {
                    let _session_item = session_service.record_nonanswer_outcome(
                        input.session_id,
                        input.session_item_id,
                        response_time_ms,
                        input.timed_out,
                    )?;
                }
                session_service.mark_deferred_completion_pending(
                    input.session_id,
                    "submit_attempt.defer_coach_brain",
                )?;
                Ok(())
            })();
            match write_result {
                Ok(()) => commit_transaction(conn)?,
                Err(err) => {
                    rollback_transaction(conn);
                    return Err(err);
                }
            }
        } else if let Some(selected_option_id) = input.selected_option_id {
            let _session_item = session_service.record_answer_without_attempt_row(
                input.session_id,
                &SessionAnswerInput {
                    item_id: input.session_item_id,
                    selected_option_id,
                    response_time_ms,
                },
            )?;
        } else {
            let _session_item = session_service.record_nonanswer_outcome(
                input.session_id,
                input.session_item_id,
                response_time_ms,
                input.timed_out,
            )?;
        }
        eprintln!(
            "[perf][rust.submit_attempt] session.record_outcome {:.1}ms",
            session_write_start.elapsed().as_secs_f64() * 1000.0
        );

        // Steps 1-7: Full learner truth pipeline
        let student_model = StudentModelService::new(conn);
        let student_model_start = Instant::now();
        let result = student_model.process_answer(
            input.student_id,
            &AnswerSubmission {
                question_id: input.question_id,
                selected_option_id: input.selected_option_id,
                answer_text: input
                    .blank_answers
                    .as_ref()
                    .map(|vals| vals.join(" | ")),
                session_id: Some(input.session_id),
                session_type,
                started_at: now,
                submitted_at: now,
                response_time_ms,
                confidence_level,
                hint_count: input.hint_count,
                changed_answer_count: input.changed_answer_count,
                skipped: input.skipped,
                timed_out: input.timed_out,
                support_level: None,
                was_timed: input.was_timed,
                was_transfer_variant: false,
                was_retention_check: false,
                was_mixed_context: false,
                precomputed_is_correct,
            },
        )?;
        eprintln!(
            "[perf][rust.submit_attempt] student_model.process_answer {:.1}ms",
            student_model_start.elapsed().as_secs_f64() * 1000.0
        );
        let evidence_start = Instant::now();
        EvidenceInterpretationEngine::new(conn).interpret_attempt(result.attempt_id)?;
        eprintln!(
            "[perf][rust.submit_attempt] evidence.interpret_attempt {:.1}ms",
            evidence_start.elapsed().as_secs_f64() * 1000.0
        );
        let error_type = result
            .error_type
            .as_ref()
            .map(|error| error.as_str().to_string());
        let recommended_action = result.recommended_action.clone();
        let runtime_feedback_start = Instant::now();
        PedagogicalRuntimeService::new(conn).record_attempt_feedback(PedagogicalAttemptSignal {
            student_id: input.student_id,
            session_id: input.session_id,
            question_id: input.question_id,
            response_time_ms,
            confidence_level: input.confidence_level.clone(),
            hint_count: input.hint_count,
            was_timed: input.was_timed,
            was_transfer_variant: false,
            was_retention_check: false,
            was_mixed_context: false,
            is_correct: result.is_correct,
            error_type: error_type.clone(),
            recommended_action: recommended_action.clone(),
        })?;
        eprintln!(
            "[perf][rust.submit_attempt] pedagogical_runtime.record_attempt_feedback {:.1}ms",
            runtime_feedback_start.elapsed().as_secs_f64() * 1000.0
        );

        // Session progress
        let session_snapshot_start = Instant::now();
        let snapshot = session_service.get_session_snapshot(input.session_id)?;
        eprintln!(
            "[perf][rust.submit_attempt] session.get_session_snapshot {:.1}ms",
            session_snapshot_start.elapsed().as_secs_f64() * 1000.0
        );
        let (session_answered, session_total, session_status) = match &snapshot {
            Some(snap) => {
                let engaged = snap
                    .items
                    .iter()
                    .filter(|item| matches!(item.status.as_str(), "answered" | "skipped" | "timed_out"))
                    .count() as i64;
                (engaged, snap.items.len() as i64, snap.session.status.as_str())
            }
            None => (0, 0, "unknown"),
        };
        let session_complete = session_status == "completed"
            || (session_total > 0 && session_answered >= session_total);

        // Step 8: Recompute coach next action
        let skip_coach_brain_via_env = env::var("ECOACH_PERF_SKIP_COACH_BRAIN")
            .map(|value| value == "1")
            .unwrap_or(false);
        let skip_coach_brain = input.defer_coach_brain || skip_coach_brain_via_env;
        let next_action = if skip_coach_brain {
            let skip_reason = if input.defer_coach_brain {
                "defer_coach_brain"
            } else {
                "ECOACH_PERF_SKIP_COACH_BRAIN=1"
            };
            eprintln!(
                "[perf][rust.submit_attempt] coach.evaluate_coach_brain skipped by {}",
                skip_reason
            );
            ecoach_coach_brain::CoachNextAction {
                state: ecoach_coach_brain::LearnerJourneyState::ReadyForTodayMission,
                action_type: ecoach_coach_brain::CoachActionType::ViewOverview,
                title: "Open coach overview".to_string(),
                subtitle: "Skipped during perf isolation".to_string(),
                estimated_minutes: None,
                route: "/coach".to_string(),
                context: serde_json::json!({ "skipped": true }),
            }
        } else {
            let coach_start = Instant::now();
            let brain = evaluate_coach_brain(
                conn,
                input.student_id,
                CoachBrainTrigger::AttemptSubmitted,
                14,
            )?;
            eprintln!(
                "[perf][rust.submit_attempt] coach.evaluate_coach_brain {:.1}ms",
                coach_start.elapsed().as_secs_f64() * 1000.0
            );
            brain.next_action
        };
        let selected_option_text = if input.selected_option_id.is_some() {
            Some(result.selected_option_text.clone())
        } else {
            None
        };

        // Steps 9-10: Read models are computed on-demand; decay is batched separately
        eprintln!(
            "[perf][rust.submit_attempt] total {:.1}ms",
            command_start.elapsed().as_secs_f64() * 1000.0
        );

        Ok(AttemptResultDto {
            attempt_id: result.attempt_id,
            is_correct: result.is_correct,
            explanation: result.explanation,
            correct_option_text: result.correct_option_text,
            selected_option_text,
            misconception_info: result.misconception_info,
            error_type,
            diagnosis_summary: result.diagnosis_summary,
            recommended_action,
            updated_mastery: result.updated_mastery as i64,
            updated_gap: result.updated_gap as i64,
            session_answered,
            session_remaining: session_total - session_answered,
            session_complete,
            next_action_type: format!("{:?}", next_action.action_type),
            next_action_title: next_action.title,
            next_action_route: next_action.route,
        })
    }).inspect(|_| {
        eprintln!(
            "[perf][rust.submit_attempt] outer_total {:.1}ms",
            total_start.elapsed().as_secs_f64() * 1000.0
        );
    })
}

/// Simplified version for session completion that triggers the session-end pipeline (Section 7.5).
///
/// 1. Finalize counted-time truth
/// 2. Close open session segments
/// 3. Update plan adherence
/// 4. Compute session summary
/// 5. Recompute readiness deltas
/// 6. Recompute review queue and next coach action
/// 7. Emit parent/admin digest deltas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompletionResultDto {
    pub session_id: i64,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub accuracy_score: Option<i64>,
    pub status: String,
    pub next_action_type: String,
    pub next_action_title: String,
    pub next_action_route: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeferredCompletionRecoveryResultDto {
    pub attempted: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub skipped: i64,
    pub remaining: i64,
    pub recovered_session_ids: Vec<i64>,
    pub failed_session_ids: Vec<i64>,
    pub skipped_session_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
struct ProjectedNextAction {
    action_type: String,
    title: String,
    route: String,
}

fn begin_immediate_transaction(conn: &Connection) -> Result<(), CommandError> {
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION;")
        .map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: format!("failed to begin transaction: {err}"),
        })
}

fn commit_transaction(conn: &Connection) -> Result<(), CommandError> {
    conn.execute_batch("COMMIT;").map_err(|err| CommandError {
        code: "storage_error".to_string(),
        message: format!("failed to commit transaction: {err}"),
    })
}

fn rollback_transaction(conn: &Connection) {
    let _ = conn.execute_batch("ROLLBACK;");
}

fn load_projected_next_action(
    conn: &Connection,
    student_id: i64,
) -> Result<Option<ProjectedNextAction>, CommandError> {
    use rusqlite::OptionalExtension;

    conn.query_row(
        "SELECT action_type, title, route
         FROM coach_next_actions
         WHERE student_id = ?1",
        [student_id],
        |row| {
            Ok(ProjectedNextAction {
                action_type: row.get(0)?,
                title: row.get(1)?,
                route: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        },
    )
    .optional()
    .map_err(|err| CommandError {
        code: "storage_error".to_string(),
        message: format!("failed to load projected next action: {err}"),
    })
}

#[cfg(debug_assertions)]
fn maybe_inject_deferred_completion_failure(
    stage: &str,
    session_id: i64,
) -> Result<(), CommandError> {
    let configured_stage = env::var("ECOACH_DEFERRED_COMPLETION_FAIL_STAGE")
        .ok()
        .unwrap_or_default();
    if configured_stage != stage {
        return Ok(());
    }

    let configured_session_id = env::var("ECOACH_DEFERRED_COMPLETION_FAIL_SESSION_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok());
    if let Some(target_session_id) = configured_session_id {
        if target_session_id != session_id {
            return Ok(());
        }
    }

    eprintln!(
        "[chaos][deferred_completion] injected_failure stage={} session_id={}",
        stage, session_id
    );
    Err(CommandError {
        code: "chaos_injected".to_string(),
        message: format!(
            "injected deferred completion failure at stage={} for session {}",
            stage, session_id
        ),
    })
}

#[cfg(not(debug_assertions))]
fn maybe_inject_deferred_completion_failure(
    _stage: &str,
    _session_id: i64,
) -> Result<(), CommandError> {
    Ok(())
}

fn build_completion_result_from_projection(
    conn: &Connection,
    student_id: i64,
    session_id: i64,
) -> Result<SessionCompletionResultDto, CommandError> {
    let session_service = SessionService::new(conn);
    let summary = session_service.complete_session(session_id)?;
    let next_action = if let Some(projection) = load_projected_next_action(conn, student_id)? {
        (projection.action_type, projection.title, projection.route)
    } else {
        let resolved = resolve_next_coach_action(conn, student_id)?;
        (
            format!("{:?}", resolved.action_type),
            resolved.title,
            resolved.route,
        )
    };
    Ok(SessionCompletionResultDto {
        session_id: summary.session_id,
        answered_questions: summary.answered_questions,
        correct_questions: summary.correct_questions,
        accuracy_score: summary.accuracy_score,
        status: summary.status,
        next_action_type: next_action.0,
        next_action_title: next_action.1,
        next_action_route: next_action.2,
    })
}

fn execute_completion_pipeline(
    conn: &Connection,
    student_id: i64,
    session_id: i64,
    reason: &str,
) -> Result<SessionCompletionResultDto, CommandError> {
    let session_service = SessionService::new(conn);
    let session = session_service
        .get_session(session_id)?
        .ok_or_else(|| CommandError {
            code: "not_found".to_string(),
            message: format!("session {} not found", session_id),
        })?;
    if session.student_id != student_id {
        return Err(CommandError {
            code: "validation_error".to_string(),
            message: format!(
                "session {} does not belong to student {}",
                session_id, student_id
            ),
        });
    }
    if session.deferred_completion_state == "processed" {
        eprintln!(
            "[recovery][deferred_completion] skip_already_processed session_id={} reason={}",
            session_id, reason
        );
        return build_completion_result_from_projection(conn, student_id, session_id);
    }

    let is_deferred = session.deferred_completion_state != "idle";
    if is_deferred {
        eprintln!(
            "[recovery][deferred_completion] processing session_id={} status={} state={} reason={}",
            session_id, session.status, session.deferred_completion_state, reason
        );
    }

    begin_immediate_transaction(conn)?;
    let result = (|| -> Result<SessionCompletionResultDto, CommandError> {
        let session_service = SessionService::new(conn);
        let summary = if is_deferred {
            session_service.finalize_deferred_completion_session(session_id)?
        } else {
            session_service.complete_session(session_id)?
        };
        maybe_inject_deferred_completion_failure("before_coach", session_id)?;
        let brain =
            evaluate_coach_brain(conn, student_id, CoachBrainTrigger::SessionCompleted, 14)?;
        maybe_inject_deferred_completion_failure("after_coach", session_id)?;
        let next_action = brain.next_action;
        if is_deferred {
            session_service.mark_deferred_completion_processed(session_id, reason)?;
        }
        Ok(SessionCompletionResultDto {
            session_id: summary.session_id,
            answered_questions: summary.answered_questions,
            correct_questions: summary.correct_questions,
            accuracy_score: summary.accuracy_score,
            status: summary.status,
            next_action_type: format!("{:?}", next_action.action_type),
            next_action_title: next_action.title,
            next_action_route: next_action.route,
        })
    })();

    match result {
        Ok(dto) => {
            commit_transaction(conn)?;
            if is_deferred {
                eprintln!(
                    "[recovery][deferred_completion] success session_id={} status={} reason={}",
                    session_id, dto.status, reason
                );
            }
            Ok(dto)
        }
        Err(err) => {
            rollback_transaction(conn);
            if is_deferred {
                let error_message = err.message.clone();
                if let Err(mark_err) = SessionService::new(conn)
                    .mark_deferred_completion_failed(session_id, reason, &error_message)
                {
                    eprintln!(
                        "[recovery][deferred_completion] failure_marker_error session_id={} reason={} error={}",
                        session_id, reason, mark_err
                    );
                }
                eprintln!(
                    "[recovery][deferred_completion] failed session_id={} reason={} error={}",
                    session_id, reason, error_message
                );
            }
            Err(err)
        }
    }
}

fn recover_deferred_completion_sessions_with_conn(
    conn: &Connection,
    student_id: i64,
    reason: &str,
    max_sessions: Option<usize>,
) -> Result<DeferredCompletionRecoveryResultDto, CommandError> {
    let session_service = SessionService::new(conn);
    let mut sessions = session_service.list_sessions_needing_deferred_completion(student_id)?;
    let total_candidates = sessions.len() as i64;
    if let Some(limit) = max_sessions {
        sessions.truncate(limit);
    }
    let mut result = DeferredCompletionRecoveryResultDto::default();
    for session in sessions {
        result.attempted += 1;
        match execute_completion_pipeline(conn, student_id, session.id, reason) {
            Ok(_) => {
                result.succeeded += 1;
                result.recovered_session_ids.push(session.id);
            }
            Err(err) => {
                if session.deferred_completion_state == "processed" {
                    result.skipped += 1;
                    result.skipped_session_ids.push(session.id);
                } else {
                    result.failed += 1;
                    result.failed_session_ids.push(session.id);
                    eprintln!(
                        "[recovery][deferred_completion] retry_failed session_id={} reason={} error={}",
                        session.id, reason, err.message
                    );
                }
            }
        }
    }
    result.remaining = (total_candidates - result.attempted).max(0);
    Ok(result)
}

pub fn recover_deferred_completion_sessions(
    state: &AppState,
    student_id: i64,
    reason: &str,
    max_sessions: Option<usize>,
) -> Result<DeferredCompletionRecoveryResultDto, CommandError> {
    state.with_connection(|conn| {
        recover_deferred_completion_sessions_with_conn(
            conn,
            student_id,
            reason,
            max_sessions,
        )
    })
}

pub fn backfill_historical_deferred_completion_sessions(
    state: &AppState,
    student_id: i64,
    stale_after_seconds: i64,
    max_sessions: Option<usize>,
) -> Result<DeferredCompletionRecoveryResultDto, CommandError> {
    state.with_connection(|conn| {
        let session_service = SessionService::new(conn);
        let mut sessions = session_service.list_historical_deferred_completion_candidates(
            student_id,
            stale_after_seconds,
        )?;
        let total_candidates = sessions.len() as i64;
        if let Some(limit) = max_sessions {
            sessions.truncate(limit);
        }
        let mut result = DeferredCompletionRecoveryResultDto::default();
        for session in sessions {
            result.attempted += 1;
            session_service.mark_deferred_completion_pending(
                session.id,
                "historical_backfill",
            )?;
            result.succeeded += 1;
            result.recovered_session_ids.push(session.id);
        }
        result.remaining = (total_candidates - result.attempted).max(0);
        Ok(result)
    })
}

pub fn complete_session_with_pipeline(
    state: &AppState,
    student_id: i64,
    session_id: i64,
) -> Result<SessionCompletionResultDto, CommandError> {
    let total_start = Instant::now();
    eprintln!(
        "[perf][rust.complete_session_with_pipeline] enter student_id={} session_id={}",
        student_id, session_id
    );
    state.with_connection(|conn| {
        let result =
            execute_completion_pipeline(conn, student_id, session_id, "explicit_completion")?;
        eprintln!(
            "[perf][rust.complete_session_with_pipeline] total {:.1}ms",
            total_start.elapsed().as_secs_f64() * 1000.0
        );
        Ok(result)
    })
}

// ── Phase 4 server-side grading helpers ─────────────────────────────
//
// These do one SELECT against `question_options` and then compute the
// outcome purely in Rust. Called only when the frontend supplies
// `selected_option_ids` (multi-correct MCQ) or `blank_answers` (fill-
// in-the-blank). Legacy single-answer MCQ stays untouched.

/// Grade the attempt using whichever Phase-4 payload is present.
/// Precedence: `blank_answers` wins if both are supplied (a fill-blank
/// question can't also be multi-correct-MCQ).
fn grade_attempt_server_side(
    conn: &Connection,
    input: &SubmitAttemptInput,
) -> Result<bool, CommandError> {
    if let Some(blank_values) = input.blank_answers.as_ref() {
        return Ok(grade_fill_blank(conn, input.question_id, blank_values)?);
    }
    if let Some(ids) = input.selected_option_ids.as_ref() {
        return Ok(grade_multi_correct(conn, input.question_id, ids)?);
    }
    // Caller shouldn't reach here — submit_attempt guards this. Default
    // to false so any surprise lands on the safe (stricter) side.
    Ok(false)
}

/// Set equality: the student's selected ids must match exactly the set
/// of `is_correct=1` options on the question. No partial credit.
fn grade_multi_correct(
    conn: &Connection,
    question_id: i64,
    selected_ids: &[i64],
) -> Result<bool, CommandError> {
    use std::collections::BTreeSet;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM question_options
             WHERE question_id = ?1 AND is_correct = 1",
        )
        .map_err(storage_error)?;
    let correct: BTreeSet<i64> = stmt
        .query_map([question_id], |row| row.get::<_, i64>(0))
        .map_err(storage_error)?
        .filter_map(|r| r.ok())
        .collect();

    let selected: BTreeSet<i64> = selected_ids.iter().copied().collect();
    Ok(!correct.is_empty() && correct == selected)
}

/// Per-blank match: every blank must have at least one acceptable
/// answer (case- and whitespace-insensitive). Fill-blank uses
/// `option_label` as the blank index; multiple rows per label = the
/// accept-list. An empty input grades as incorrect.
fn grade_fill_blank(
    conn: &Connection,
    question_id: i64,
    blank_values: &[String],
) -> Result<bool, CommandError> {
    use std::collections::HashMap;
    let mut stmt = conn
        .prepare(
            "SELECT option_label, option_text FROM question_options
             WHERE question_id = ?1 AND is_correct = 1",
        )
        .map_err(storage_error)?;
    // blank_index (as i64) → list of accepted lowercase-trimmed strings.
    let mut accept: HashMap<i64, Vec<String>> = HashMap::new();
    for row in stmt
        .query_map([question_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?
    {
        let (label, text) = row.map_err(storage_error)?;
        if let Ok(idx) = label.trim().parse::<i64>() {
            let normalised = text.trim().to_lowercase();
            if !normalised.is_empty() {
                accept.entry(idx).or_default().push(normalised);
            }
        }
    }
    if accept.is_empty() {
        // Question has no authored answers — can't grade; treat as wrong.
        return Ok(false);
    }

    for (i, value) in blank_values.iter().enumerate() {
        let blank_idx = (i as i64) + 1;
        let normalised = value.trim().to_lowercase();
        if normalised.is_empty() {
            return Ok(false);
        }
        let Some(list) = accept.get(&blank_idx) else {
            return Ok(false);
        };
        if !list.contains(&normalised) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn storage_error(err: impl ToString) -> CommandError {
    CommandError {
        code: "storage_error".to_string(),
        message: err.to_string(),
    }
}
