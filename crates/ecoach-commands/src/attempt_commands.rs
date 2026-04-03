use chrono::Utc;
use ecoach_coach_brain::{
    CoachBrainTrigger, PedagogicalAttemptSignal, PedagogicalRuntimeService,
    evaluate_coach_brain,
};
use ecoach_sessions::{SessionAnswerInput, SessionService};
use ecoach_student_model::{AnswerSubmission, StudentModelService};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

/// Input for the submit_attempt hot path.
/// This is the primary command the frontend calls when a student answers a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAttemptInput {
    pub student_id: i64,
    pub session_id: i64,
    pub session_item_id: i64,
    pub question_id: i64,
    pub selected_option_id: i64,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
    pub hint_count: i64,
    pub changed_answer_count: i64,
    pub was_timed: bool,
}

/// Comprehensive result DTO returned after the full hot path completes.
/// Contains everything the frontend needs to render feedback and next steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResultDto {
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
    state.with_connection(|conn| {
        let now = Utc::now();
        let confidence_level = input.confidence_level.clone();

        // Step 1-2: Record the answer in session_items
        let session_service = SessionService::new(conn);
        let _session_item = session_service.record_answer(
            input.session_id,
            &SessionAnswerInput {
                item_id: input.session_item_id,
                selected_option_id: input.selected_option_id,
                response_time_ms: input.response_time_ms,
            },
        )?;

        // Steps 1-7: Full learner truth pipeline
        let student_model = StudentModelService::new(conn);
        let result = student_model.process_answer(
            input.student_id,
            &AnswerSubmission {
                question_id: input.question_id,
                selected_option_id: Some(input.selected_option_id),
                answer_text: None,
                session_id: Some(input.session_id),
                session_type: None,
                started_at: now,
                submitted_at: now,
                response_time_ms: input.response_time_ms,
                confidence_level,
                hint_count: input.hint_count,
                changed_answer_count: input.changed_answer_count,
                skipped: false,
                timed_out: false,
                support_level: None,
                was_timed: input.was_timed,
                was_transfer_variant: false,
                was_retention_check: false,
                was_mixed_context: false,
            },
        )?;
        let error_type = result.error_type.as_ref().map(|error| error.as_str().to_string());
        let recommended_action = result.recommended_action.clone();
        PedagogicalRuntimeService::new(conn).record_attempt_feedback(PedagogicalAttemptSignal {
            student_id: input.student_id,
            session_id: input.session_id,
            question_id: input.question_id,
            response_time_ms: input.response_time_ms,
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

        // Session progress
        let snapshot = session_service.get_session_snapshot(input.session_id)?;
        let (session_answered, session_total, session_status) = match &snapshot {
            Some(snap) => {
                let answered = snap.items.iter().filter(|i| i.status == "answered").count() as i64;
                let total = snap.items.len() as i64;
                (answered, total, snap.session.status.as_str())
            }
            None => (0, 0, "unknown"),
        };
        let session_complete = session_status == "completed"
            || (session_total > 0 && session_answered >= session_total);

        // Step 8: Recompute coach next action
        let brain = evaluate_coach_brain(
            conn,
            input.student_id,
            CoachBrainTrigger::AttemptSubmitted,
            14,
        )?;
        let next_action = brain.next_action;

        // Steps 9-10: Read models are computed on-demand; decay is batched separately

        Ok(AttemptResultDto {
            is_correct: result.is_correct,
            explanation: result.explanation,
            correct_option_text: result.correct_option_text,
            selected_option_text: Some(result.selected_option_text),
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

pub fn complete_session_with_pipeline(
    state: &AppState,
    student_id: i64,
    session_id: i64,
) -> Result<SessionCompletionResultDto, CommandError> {
    state.with_connection(|conn| {
        // Steps 1-4: Finalize session
        let session_service = SessionService::new(conn);
        let summary = session_service.complete_session(session_id)?;

        // Steps 5-6: Recompute coach next action (which reads readiness internally)
        let brain =
            evaluate_coach_brain(conn, student_id, CoachBrainTrigger::SessionCompleted, 14)?;
        let next_action = brain.next_action;

        // Step 7: Parent digest is computed on-demand when parent requests it

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
    })
}
