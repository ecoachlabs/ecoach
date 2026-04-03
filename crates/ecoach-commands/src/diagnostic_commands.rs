use std::collections::HashMap;

use chrono::{Duration, Utc};
use ecoach_coach_brain::{
    InterventionLibraryService, JourneyRouteSnapshot, JourneyService, PlanEngine,
};
use ecoach_diagnostics::{
    DiagnosticBattery, DiagnosticEngine, DiagnosticMode, DiagnosticPhaseItem, DiagnosticPhasePlan,
    DiagnosticRootCauseHypothesis, DiagnosticTopicAnalytics,
};
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    dtos::{
        DiagnosticAudienceReportDto, DiagnosticCauseEvolutionDto,
        DiagnosticInterventionPrescriptionDto, DiagnosticItemRoutingProfileDto,
        DiagnosticLongitudinalSummaryDto, DiagnosticProblemCauseFixCardDto,
        DiagnosticRecommendationDto, DiagnosticResultDto, DiagnosticSkillResultDto,
        DiagnosticSubjectBlueprintDto,
    },
    error::CommandError,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRunDto {
    pub diagnostic_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicAnalyticsDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub classification: String,
    pub mastery_score: i64,
    pub confidence_score: i64,
    pub recommended_action: String,
    pub endurance_score: i64,
    pub weakness_type: Option<String>,
    pub failure_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDiagnosticAttemptInput {
    pub attempt_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
    pub changed_answer_count: i64,
    pub skipped: bool,
    pub timed_out: bool,
    pub first_focus_at: Option<String>,
    pub first_input_at: Option<String>,
    pub concept_guess: Option<String>,
    pub final_answer: Option<serde_json::Value>,
    pub interaction_log: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRootCauseHypothesisDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub hypothesis_code: String,
    pub confidence_score: i64,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCompletionSyncDto {
    pub diagnostic_id: i64,
    pub overall_readiness: i64,
    pub readiness_band: String,
    pub analytics: Vec<TopicAnalyticsDto>,
    pub top_hypotheses: Vec<DiagnosticRootCauseHypothesisDto>,
    pub diagnostic_result: DiagnosticResultDto,
    pub longitudinal_summary: Option<DiagnosticLongitudinalSummaryDto>,
    pub cause_evolution: Vec<DiagnosticCauseEvolutionDto>,
    pub problem_cause_fix_cards: Vec<DiagnosticProblemCauseFixCardDto>,
    pub intervention_prescriptions: Vec<DiagnosticInterventionPrescriptionDto>,
    pub synced_topic_count: usize,
    pub blocker_count: usize,
    pub rewritten_plan_id: Option<i64>,
    pub refreshed_route: Option<JourneyRouteSnapshot>,
    pub generated_mission_id: Option<i64>,
}

pub type DiagnosticBatteryDto = DiagnosticBattery;
pub type DiagnosticPhasePlanDto = DiagnosticPhasePlan;
pub type DiagnosticPhaseItemDto = DiagnosticPhaseItem;

pub fn launch_diagnostic(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    mode: String,
) -> Result<DiagnosticRunDto, CommandError> {
    state.with_connection(|conn| {
        let engine = DiagnosticEngine::new(conn);
        let diagnostic_id = engine.start_diagnostic(student_id, subject_id, parse_mode(&mode))?;
        Ok(DiagnosticRunDto { diagnostic_id })
    })
}

pub fn get_diagnostic_battery(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<DiagnosticBatteryDto, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn).get_diagnostic_battery(diagnostic_id)?)
    })
}

pub fn get_diagnostic_subject_blueprint(
    state: &AppState,
    subject_id: i64,
) -> Result<Option<DiagnosticSubjectBlueprintDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .get_subject_blueprint(subject_id)?
            .map(DiagnosticSubjectBlueprintDto::from))
    })
}

pub fn list_diagnostic_item_routing_profiles(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticItemRoutingProfileDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_item_routing_profiles(diagnostic_id)?
            .into_iter()
            .map(DiagnosticItemRoutingProfileDto::from)
            .collect())
    })
}

pub fn list_diagnostic_problem_cause_fix_cards(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticProblemCauseFixCardDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_problem_cause_fix_cards(diagnostic_id)?
            .into_iter()
            .map(DiagnosticProblemCauseFixCardDto::from)
            .collect())
    })
}

pub fn list_diagnostic_intervention_prescriptions(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticInterventionPrescriptionDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_intervention_prescriptions(diagnostic_id)?
            .into_iter()
            .map(DiagnosticInterventionPrescriptionDto::from)
            .collect())
    })
}

pub fn list_diagnostic_phase_items(
    state: &AppState,
    diagnostic_id: i64,
    phase_number: i64,
) -> Result<Vec<DiagnosticPhaseItemDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn).list_phase_items(diagnostic_id, phase_number)?)
    })
}

pub fn submit_diagnostic_attempt(
    state: &AppState,
    diagnostic_id: i64,
    input: SubmitDiagnosticAttemptInput,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        DiagnosticEngine::new(conn).submit_phase_attempt_details(
            diagnostic_id,
            input.attempt_id,
            input.selected_option_id,
            input.response_time_ms,
            input.confidence_level.as_deref(),
            input.changed_answer_count,
            input.skipped,
            input.timed_out,
            input.first_focus_at.as_deref(),
            input.first_input_at.as_deref(),
            input.concept_guess.as_deref(),
            input.final_answer.as_ref(),
            input.interaction_log.as_ref(),
        )?;
        Ok(())
    })
}

pub fn advance_diagnostic_phase(
    state: &AppState,
    diagnostic_id: i64,
    completed_phase_number: i64,
) -> Result<Option<DiagnosticPhasePlanDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn).advance_phase(diagnostic_id, completed_phase_number)?)
    })
}

pub fn complete_diagnostic_and_sync(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<DiagnosticCompletionSyncDto, CommandError> {
    state.with_connection(|conn| {
        let engine = DiagnosticEngine::new(conn);
        let battery = engine.get_diagnostic_battery(diagnostic_id)?;
        let result = engine.complete_diagnostic(diagnostic_id)?;
        let analytics = engine.list_topic_analytics(diagnostic_id)?;
        let hypotheses = engine.list_root_cause_hypotheses(diagnostic_id, None)?;

        let top_hypotheses = top_hypotheses_by_topic(&hypotheses);
        let now = Utc::now().to_rfc3339();
        let mut blocker_count = 0usize;

        for analytics_row in &analytics {
            let top_hypothesis = top_hypotheses.get(&analytics_row.topic_id);
            sync_student_topic_state(
                conn,
                battery.student_id,
                analytics_row,
                top_hypothesis,
                &now,
            )?;
            sync_student_error_profile(
                conn,
                battery.student_id,
                analytics_row,
                top_hypothesis,
                &now,
            )?;
            let blocked = sync_coach_topic_profile(
                conn,
                battery.student_id,
                analytics_row,
                top_hypothesis,
                &now,
            )?;
            sync_coach_blockers(
                conn,
                battery.student_id,
                analytics_row,
                top_hypothesis,
                blocked,
                &now,
            )?;
            if blocked {
                blocker_count += 1;
            }
        }

        let rewritten_plan_id = ensure_plan_from_diagnostic(conn, battery.student_id)?;
        let exam_target = load_exam_target(conn, battery.student_id)?;
        let refreshed_route = if analytics.is_empty() {
            None
        } else {
            Some(JourneyService::new(conn).build_or_refresh_route(
                battery.student_id,
                battery.subject_id,
                exam_target.as_deref(),
            )?)
        };
        let generated_mission_id = rewritten_plan_id
            .map(|_| PlanEngine::new(conn).generate_today_mission(battery.student_id))
            .transpose()?;
        let topic_ids = analytics
            .iter()
            .map(|item| item.topic_id)
            .collect::<Vec<_>>();
        if !topic_ids.is_empty() {
            InterventionLibraryService::new(conn).sync_diagnostic_prescriptions(
                diagnostic_id,
                battery.student_id,
                &topic_ids,
            )?;
        }

        append_runtime_event(
            conn,
            DomainEvent::new(
                "diagnostic.synced_to_coach",
                diagnostic_id.to_string(),
                json!({
                    "student_id": battery.student_id,
                    "subject_id": battery.subject_id,
                    "topic_count": analytics.len(),
                    "blocker_count": blocker_count,
                    "rewritten_plan_id": rewritten_plan_id,
                    "generated_mission_id": generated_mission_id,
                }),
            ),
        )?;

        let synced_topic_count = result.topic_results.len();
        let overall_readiness = result.overall_readiness as i64;
        let readiness_band = result.readiness_band.clone();
        let refreshed_result = engine
            .get_diagnostic_result(diagnostic_id)?
            .ok_or_else(|| {
                CommandError::from(EcoachError::NotFound(format!(
                    "diagnostic {} result was not persisted",
                    diagnostic_id
                )))
            })?;
        let problem_cause_fix_cards = refreshed_result
            .problem_cause_fix_cards
            .clone()
            .into_iter()
            .map(DiagnosticProblemCauseFixCardDto::from)
            .collect();
        let intervention_prescriptions = refreshed_result
            .intervention_prescriptions
            .clone()
            .into_iter()
            .map(DiagnosticInterventionPrescriptionDto::from)
            .collect();
        let diagnostic_result = DiagnosticResultDto::from(refreshed_result.clone());
        let longitudinal_summary = refreshed_result
            .longitudinal_summary
            .clone()
            .map(DiagnosticLongitudinalSummaryDto::from);
        let cause_evolution = longitudinal_summary
            .as_ref()
            .map(|summary| summary.cause_evolution.clone())
            .unwrap_or_default();

        Ok(DiagnosticCompletionSyncDto {
            diagnostic_id,
            overall_readiness,
            readiness_band,
            analytics: analytics.into_iter().map(TopicAnalyticsDto::from).collect(),
            top_hypotheses: top_hypotheses
                .into_values()
                .map(DiagnosticRootCauseHypothesisDto::from)
                .collect(),
            diagnostic_result,
            longitudinal_summary,
            cause_evolution,
            problem_cause_fix_cards,
            intervention_prescriptions,
            synced_topic_count,
            blocker_count,
            rewritten_plan_id,
            refreshed_route,
            generated_mission_id,
        })
    })
}

pub fn get_diagnostic_report(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<TopicAnalyticsDto>, CommandError> {
    state.with_connection(|conn| {
        let engine = DiagnosticEngine::new(conn);
        let analytics = engine.list_topic_analytics(diagnostic_id)?;
        Ok(analytics.into_iter().map(TopicAnalyticsDto::from).collect())
    })
}

pub fn get_diagnostic_result(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Option<DiagnosticResultDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .get_diagnostic_result(diagnostic_id)?
            .map(DiagnosticResultDto::from))
    })
}

pub fn list_diagnostic_skill_results(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticSkillResultDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_skill_results(diagnostic_id)?
            .into_iter()
            .map(DiagnosticSkillResultDto::from)
            .collect())
    })
}

pub fn list_diagnostic_recommendations(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticRecommendationDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_recommendations(diagnostic_id)?
            .into_iter()
            .map(DiagnosticRecommendationDto::from)
            .collect())
    })
}

pub fn get_diagnostic_audience_report(
    state: &AppState,
    diagnostic_id: i64,
    audience: String,
) -> Result<Option<DiagnosticAudienceReportDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .get_audience_report(diagnostic_id, &audience)?
            .map(DiagnosticAudienceReportDto::from))
    })
}

pub fn get_diagnostic_longitudinal_summary(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Option<DiagnosticLongitudinalSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .get_longitudinal_summary(diagnostic_id)?
            .map(DiagnosticLongitudinalSummaryDto::from))
    })
}

pub fn list_diagnostic_cause_evolution(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticCauseEvolutionDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(DiagnosticEngine::new(conn)
            .list_cause_evolution(diagnostic_id)?
            .into_iter()
            .map(DiagnosticCauseEvolutionDto::from)
            .collect())
    })
}

fn parse_mode(mode: &str) -> DiagnosticMode {
    match mode {
        "quick" => DiagnosticMode::Quick,
        "deep" => DiagnosticMode::Deep,
        _ => DiagnosticMode::Standard,
    }
}

fn top_hypotheses_by_topic(
    hypotheses: &[DiagnosticRootCauseHypothesis],
) -> HashMap<i64, DiagnosticRootCauseHypothesis> {
    let mut top = HashMap::new();
    for hypothesis in hypotheses {
        let should_replace = top
            .get(&hypothesis.topic_id)
            .map(|existing: &DiagnosticRootCauseHypothesis| {
                existing.confidence_score < hypothesis.confidence_score
            })
            .unwrap_or(true);
        if should_replace {
            top.insert(hypothesis.topic_id, hypothesis.clone());
        }
    }
    top
}

fn sync_student_topic_state(
    conn: &Connection,
    student_id: i64,
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
    now: &str,
) -> Result<(), EcoachError> {
    let fragility_score = fragility_score_for_analytics(analytics);
    let repair_priority = repair_priority_for_analytics(analytics, top_hypothesis);
    let decay_risk = decay_risk_for_analytics(analytics, fragility_score);
    let pressure_collapse_index =
        clamp_bp((analytics.mastery_score as i64 - analytics.pressure_score as i64).max(0));
    let next_review_at = next_review_at_for_analytics(analytics, top_hypothesis);

    conn.execute(
        "INSERT INTO student_topic_states (
            student_id, topic_id, mastery_score, mastery_state, accuracy_score, speed_score,
            retention_score, transfer_score, consistency_score, gap_score, priority_score,
            trend_state, fragility_score, pressure_collapse_index, evidence_count, last_seen_at,
            decay_risk, next_review_at, memory_strength, repair_priority, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 1, ?15, ?16, ?17, ?18, ?19, ?15)
         ON CONFLICT(student_id, topic_id) DO UPDATE SET
            mastery_score = excluded.mastery_score,
            mastery_state = excluded.mastery_state,
            accuracy_score = excluded.accuracy_score,
            speed_score = excluded.speed_score,
            retention_score = excluded.retention_score,
            transfer_score = excluded.transfer_score,
            consistency_score = excluded.consistency_score,
            gap_score = excluded.gap_score,
            priority_score = excluded.priority_score,
            trend_state = excluded.trend_state,
            fragility_score = excluded.fragility_score,
            pressure_collapse_index = excluded.pressure_collapse_index,
            evidence_count = MAX(student_topic_states.evidence_count, excluded.evidence_count),
            last_seen_at = excluded.last_seen_at,
            decay_risk = excluded.decay_risk,
            next_review_at = excluded.next_review_at,
            memory_strength = excluded.memory_strength,
            repair_priority = excluded.repair_priority,
            updated_at = excluded.updated_at",
        params![
            student_id,
            analytics.topic_id,
            analytics.mastery_score,
            mastery_state_for_score(analytics.mastery_score),
            analytics.precision_score,
            analytics.fluency_score,
            analytics.stability_score,
            analytics.flexibility_score,
            analytics.confidence_score,
            clamp_bp(10_000 - analytics.mastery_score as i64),
            repair_priority,
            trend_state_for_analytics(analytics),
            fragility_score,
            pressure_collapse_index,
            now,
            decay_risk,
            next_review_at,
            analytics.stability_score,
            repair_priority,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn sync_student_error_profile(
    conn: &Connection,
    student_id: i64,
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
    now: &str,
) -> Result<(), EcoachError> {
    let scores = diagnostic_error_scores(analytics, top_hypothesis);
    conn.execute(
        "INSERT INTO student_error_profiles (
            student_id, topic_id, knowledge_gap_score, conceptual_confusion_score,
            recognition_failure_score, execution_error_score, carelessness_score,
            pressure_breakdown_score, expression_weakness_score, speed_error_score, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(student_id, topic_id) DO UPDATE SET
            knowledge_gap_score = excluded.knowledge_gap_score,
            conceptual_confusion_score = excluded.conceptual_confusion_score,
            recognition_failure_score = excluded.recognition_failure_score,
            execution_error_score = excluded.execution_error_score,
            carelessness_score = excluded.carelessness_score,
            pressure_breakdown_score = excluded.pressure_breakdown_score,
            expression_weakness_score = excluded.expression_weakness_score,
            speed_error_score = excluded.speed_error_score,
            updated_at = excluded.updated_at",
        params![
            student_id,
            analytics.topic_id,
            scores.knowledge_gap_score,
            scores.conceptual_confusion_score,
            scores.recognition_failure_score,
            scores.execution_error_score,
            scores.carelessness_score,
            scores.pressure_breakdown_score,
            scores.expression_weakness_score,
            scores.speed_error_score,
            now,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn sync_coach_topic_profile(
    conn: &Connection,
    student_id: i64,
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
    now: &str,
) -> Result<bool, EcoachError> {
    let blocked = topic_is_blocked(analytics, top_hypothesis);
    let misconception_recurrence = top_hypothesis
        .filter(|hypothesis| hypothesis.hypothesis_code == "misconception_root_cause")
        .map(|_| 1)
        .unwrap_or(0);
    let repair_priority = repair_priority_for_analytics(analytics, top_hypothesis);

    conn.execute(
        "INSERT INTO coach_topic_profiles (
            student_id, topic_id, mastery_estimate, fragility_score, speed_score,
            misconception_recurrence, evidence_count, attempt_count, last_seen_at,
            blocked_status, repair_priority, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 0, ?7, ?8, ?9, ?7)
         ON CONFLICT(student_id, topic_id) DO UPDATE SET
            mastery_estimate = excluded.mastery_estimate,
            fragility_score = excluded.fragility_score,
            speed_score = excluded.speed_score,
            misconception_recurrence = excluded.misconception_recurrence,
            evidence_count = MAX(coach_topic_profiles.evidence_count, excluded.evidence_count),
            last_seen_at = excluded.last_seen_at,
            blocked_status = excluded.blocked_status,
            repair_priority = excluded.repair_priority,
            updated_at = excluded.updated_at",
        params![
            student_id,
            analytics.topic_id,
            analytics.mastery_score,
            fragility_score_for_analytics(analytics),
            analytics.fluency_score,
            misconception_recurrence,
            now,
            if blocked { 1 } else { 0 },
            repair_priority,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;

    Ok(blocked)
}

fn sync_coach_blockers(
    conn: &Connection,
    student_id: i64,
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
    blocked: bool,
    now: &str,
) -> Result<(), EcoachError> {
    conn.execute(
        "UPDATE coach_blockers
         SET resolved_at = ?3
         WHERE student_id = ?1 AND topic_id = ?2 AND resolved_at IS NULL",
        params![student_id, analytics.topic_id, now],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;

    if blocked {
        let reason = top_hypothesis
            .map(|hypothesis| {
                format!(
                    "Diagnostic sync flagged {} via {}.",
                    analytics.topic_name,
                    hypothesis.hypothesis_code.replace('_', " ")
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "Diagnostic sync flagged {} as a coach blocker.",
                    analytics.topic_name
                )
            });
        let severity = if analytics.mastery_score < 4_000
            || top_hypothesis
                .map(|hypothesis| hypothesis.confidence_score >= 8_000)
                .unwrap_or(false)
        {
            "high"
        } else {
            "moderate"
        };

        conn.execute(
            "INSERT INTO coach_blockers (student_id, topic_id, reason, severity, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![student_id, analytics.topic_id, reason, severity, now],
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    }

    Ok(())
}

fn ensure_plan_from_diagnostic(
    conn: &Connection,
    student_id: i64,
) -> Result<Option<i64>, EcoachError> {
    let active_plan_id: Option<i64> = conn
        .query_row(
            "SELECT id
             FROM coach_plans
             WHERE student_id = ?1 AND status IN ('active', 'stale')
             ORDER BY id DESC
             LIMIT 1",
            [student_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let engine = PlanEngine::new(conn);
    if active_plan_id.is_some() {
        let rewrite = engine.rewrite_active_plan(student_id, "diagnostic_sync_replan")?;
        Ok(Some(rewrite.new_plan_id))
    } else {
        let (exam_target, exam_date, daily_budget_minutes) = load_plan_defaults(conn, student_id)?;
        Ok(Some(engine.generate_plan(
            student_id,
            &exam_target,
            &exam_date,
            daily_budget_minutes,
        )?))
    }
}

fn load_plan_defaults(
    conn: &Connection,
    student_id: i64,
) -> Result<(String, String, i64), EcoachError> {
    let today = Utc::now().date_naive();
    let defaults = conn
        .query_row(
            "SELECT
                COALESCE(exam_target, 'BECE'),
                COALESCE(exam_target_date, ?2),
                COALESCE(daily_study_budget_minutes, 60)
             FROM student_profiles
             WHERE account_id = ?1",
            params![student_id, (today + Duration::days(30)).to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    Ok(defaults.unwrap_or((
        "BECE".to_string(),
        (today + Duration::days(30)).to_string(),
        60,
    )))
}

fn load_exam_target(conn: &Connection, student_id: i64) -> Result<Option<String>, EcoachError> {
    let value = conn
        .query_row(
            "SELECT exam_target FROM student_profiles WHERE account_id = ?1",
            [student_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(|err| EcoachError::Storage(err.to_string()))?
        .flatten();
    Ok(value)
}

fn mastery_state_for_score(score: BasisPoints) -> &'static str {
    match score {
        9000..=10000 => "exam_ready",
        7800..=8999 => "robust",
        6500..=7799 => "stable",
        5000..=6499 => "partial",
        3200..=4999 => "fragile",
        1500..=3199 => "emerging",
        1..=1499 => "exposed",
        _ => "unseen",
    }
}

fn trend_state_for_analytics(analytics: &DiagnosticTopicAnalytics) -> &'static str {
    if analytics.mastery_score < 3500 {
        "critical"
    } else if fragility_score_for_analytics(analytics) >= 6500 {
        "fragile"
    } else if analytics.mastery_score >= 7000 {
        "improving"
    } else {
        "stable"
    }
}

fn fragility_score_for_analytics(analytics: &DiagnosticTopicAnalytics) -> BasisPoints {
    clamp_bp(
        (10_000 - analytics.stability_score as i64)
            .max(analytics.mastery_score as i64 - analytics.pressure_score as i64)
            .max(analytics.mastery_score as i64 - analytics.flexibility_score as i64),
    )
}

fn repair_priority_for_analytics(
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
) -> BasisPoints {
    let hypothesis_boost = top_hypothesis
        .map(|hypothesis| hypothesis.confidence_score as i64 / 3)
        .unwrap_or(0);
    clamp_bp(
        (10_000 - analytics.mastery_score as i64)
            + (fragility_score_for_analytics(analytics) as i64 / 2)
            + hypothesis_boost,
    )
}

fn decay_risk_for_analytics(
    analytics: &DiagnosticTopicAnalytics,
    fragility_score: BasisPoints,
) -> BasisPoints {
    clamp_bp((10_000 - analytics.stability_score as i64) + (fragility_score as i64 / 3))
}

fn next_review_at_for_analytics(
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
) -> Option<String> {
    let offset_days = if analytics.mastery_score < 3500 {
        1
    } else if top_hypothesis
        .map(|hypothesis| {
            matches!(
                hypothesis.hypothesis_code.as_str(),
                "timed_pressure_breakdown" | "transfer_fragility" | "misconception_root_cause"
            )
        })
        .unwrap_or(false)
    {
        2
    } else if analytics.mastery_score < 6500 || fragility_score_for_analytics(analytics) >= 5500 {
        3
    } else {
        7
    };

    Some((Utc::now().date_naive() + Duration::days(offset_days)).to_string())
}

struct DiagnosticErrorScores {
    knowledge_gap_score: BasisPoints,
    conceptual_confusion_score: BasisPoints,
    recognition_failure_score: BasisPoints,
    execution_error_score: BasisPoints,
    carelessness_score: BasisPoints,
    pressure_breakdown_score: BasisPoints,
    expression_weakness_score: BasisPoints,
    speed_error_score: BasisPoints,
}

fn diagnostic_error_scores(
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
) -> DiagnosticErrorScores {
    let hypothesis_code = top_hypothesis.map(|hypothesis| hypothesis.hypothesis_code.as_str());
    let hypothesis_confidence = top_hypothesis
        .map(|hypothesis| hypothesis.confidence_score)
        .unwrap_or(0);
    let pressure_gap =
        clamp_bp((analytics.mastery_score as i64 - analytics.pressure_score as i64).max(0));
    let flex_gap =
        clamp_bp((analytics.mastery_score as i64 - analytics.flexibility_score as i64).max(0));
    let speed_gap =
        clamp_bp((analytics.precision_score as i64 - analytics.fluency_score as i64).max(0));

    DiagnosticErrorScores {
        knowledge_gap_score: if matches!(hypothesis_code, Some("foundation_gap")) {
            hypothesis_confidence
        } else {
            clamp_bp(10_000 - analytics.mastery_score as i64)
        },
        conceptual_confusion_score: if matches!(hypothesis_code, Some("misconception_root_cause")) {
            hypothesis_confidence
        } else {
            clamp_bp(flex_gap as i64 / 2)
        },
        recognition_failure_score: if matches!(hypothesis_code, Some("retrieval_latency_gap")) {
            hypothesis_confidence
        } else {
            speed_gap
        },
        execution_error_score: clamp_bp(
            ((10_000 - analytics.precision_score as i64) + flex_gap as i64) / 2,
        ),
        carelessness_score: if matches!(hypothesis_code, Some("confidence_distortion")) {
            hypothesis_confidence
        } else {
            clamp_bp((analytics.confidence_score as i64 + pressure_gap as i64) / 3)
        },
        pressure_breakdown_score: if matches!(hypothesis_code, Some("timed_pressure_breakdown")) {
            hypothesis_confidence
        } else {
            pressure_gap
        },
        expression_weakness_score: clamp_bp((10_000 - analytics.precision_score as i64) / 2),
        speed_error_score: speed_gap,
    }
}

fn topic_is_blocked(
    analytics: &DiagnosticTopicAnalytics,
    top_hypothesis: Option<&DiagnosticRootCauseHypothesis>,
) -> bool {
    analytics.mastery_score < 4500
        || fragility_score_for_analytics(analytics) >= 6500
        || top_hypothesis
            .map(|hypothesis| {
                matches!(
                    hypothesis.hypothesis_code.as_str(),
                    "foundation_gap"
                        | "misconception_root_cause"
                        | "timed_pressure_breakdown"
                        | "transfer_fragility"
                )
            })
            .unwrap_or(false)
}

fn append_runtime_event(conn: &Connection, event: DomainEvent) -> Result<(), EcoachError> {
    let payload_json = serde_json::to_string(&event.payload)
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
    conn.execute(
        "INSERT INTO runtime_events (
            event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
         ) VALUES (?1, ?2, 'diagnostic', ?3, ?4, ?5, ?6)",
        params![
            event.event_id,
            event.event_type,
            event.aggregate_id,
            event.trace_id,
            payload_json,
            event.occurred_at.to_rfc3339(),
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

impl From<DiagnosticTopicAnalytics> for TopicAnalyticsDto {
    fn from(value: DiagnosticTopicAnalytics) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            classification: value.classification,
            mastery_score: value.mastery_score as i64,
            confidence_score: value.confidence_score as i64,
            recommended_action: value.recommended_action,
            endurance_score: value.endurance_score as i64,
            weakness_type: value.weakness_type,
            failure_stage: value.failure_stage,
        }
    }
}

impl From<DiagnosticRootCauseHypothesis> for DiagnosticRootCauseHypothesisDto {
    fn from(value: DiagnosticRootCauseHypothesis) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            hypothesis_code: value.hypothesis_code,
            confidence_score: value.confidence_score as i64,
            recommended_action: value.recommended_action,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_identity::CreateAccountInput;
    use ecoach_questions::QuestionService;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{error::CommandError, identity_commands, state::AppState};

    use super::{
        SubmitDiagnosticAttemptInput, complete_diagnostic_and_sync, get_diagnostic_audience_report,
        get_diagnostic_battery, get_diagnostic_longitudinal_summary, get_diagnostic_result,
        launch_diagnostic, list_diagnostic_cause_evolution, list_diagnostic_phase_items,
        list_diagnostic_recommendations, list_diagnostic_skill_results, submit_diagnostic_attempt,
    };

    #[test]
    fn diagnostic_longitudinal_reads_are_exposed_through_command_layer() {
        let state = setup_state();
        let student = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Esi".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student account should create");

        let first = launch_diagnostic(&state, student.id, 1, "quick".to_string())
            .expect("first diagnostic should launch");
        complete_diagnostic_and_sync(&state, first.diagnostic_id)
            .expect("first diagnostic should complete");

        let second = launch_diagnostic(&state, student.id, 1, "quick".to_string())
            .expect("second diagnostic should launch");
        let sync = complete_diagnostic_and_sync(&state, second.diagnostic_id)
            .expect("second diagnostic should complete");

        let result = get_diagnostic_result(&state, second.diagnostic_id)
            .expect("diagnostic result command should succeed")
            .expect("diagnostic result should exist");
        let summary = get_diagnostic_longitudinal_summary(&state, second.diagnostic_id)
            .expect("longitudinal summary command should succeed")
            .expect("longitudinal summary should exist");
        let cause_evolution = list_diagnostic_cause_evolution(&state, second.diagnostic_id)
            .expect("cause evolution command should succeed");

        assert_eq!(result.readiness_band, sync.readiness_band);
        assert_eq!(result.overall_readiness, sync.overall_readiness);
        assert_eq!(result.topic_results.len(), sync.analytics.len());
        assert_eq!(summary.previous_diagnostic_id, Some(first.diagnostic_id));
        assert_eq!(
            sync.longitudinal_summary
                .as_ref()
                .and_then(|item| item.previous_diagnostic_id),
            Some(first.diagnostic_id)
        );
        assert_eq!(cause_evolution, sync.cause_evolution);
        assert_eq!(result.longitudinal_summary, sync.longitudinal_summary);
    }

    #[test]
    fn diagnostic_deep_outputs_are_exposed_through_command_layer() {
        let state = setup_state();
        let student = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kojo".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student account should create");

        let battery = launch_diagnostic(&state, student.id, 1, "deep".to_string())
            .expect("deep diagnostic should launch");
        let battery_detail = get_diagnostic_battery(&state, battery.diagnostic_id)
            .expect("diagnostic battery should be retrievable");

        for phase in &battery_detail.phases {
            let items =
                list_diagnostic_phase_items(&state, battery.diagnostic_id, phase.phase_number)
                    .expect("phase items should be retrievable");
            let Some(first_item) = items.first() else {
                continue;
            };
            let (correct_option_id, wrong_option_id) = state
                .with_connection(|conn| {
                    let question_service = QuestionService::new(conn);
                    let options = question_service.list_options(first_item.question_id)?;
                    let correct_option_id = options
                        .iter()
                        .find(|option| option.is_correct)
                        .map(|option| option.id)
                        .expect("correct option should exist");
                    let wrong_option_id = options
                        .iter()
                        .find(|option| !option.is_correct)
                        .map(|option| option.id)
                        .expect("wrong option should exist");
                    Ok((correct_option_id, wrong_option_id))
                })
                .expect("question options should load");
            let selected_option_id = match phase.phase_code.as_str() {
                "pressure" | "flex" | "root_cause" => wrong_option_id,
                _ => correct_option_id,
            };
            submit_diagnostic_attempt(
                &state,
                battery.diagnostic_id,
                SubmitDiagnosticAttemptInput {
                    attempt_id: first_item.attempt_id,
                    selected_option_id: Some(selected_option_id),
                    response_time_ms: Some(phase.time_limit_seconds.unwrap_or(45) * 800),
                    confidence_level: Some("not_sure".to_string()),
                    changed_answer_count: 1,
                    skipped: false,
                    timed_out: false,
                    first_focus_at: None,
                    first_input_at: None,
                    concept_guess: None,
                    final_answer: None,
                    interaction_log: None,
                },
            )
            .expect("phase attempt should submit");
        }

        complete_diagnostic_and_sync(&state, battery.diagnostic_id)
            .expect("deep diagnostic should complete");

        let result = get_diagnostic_result(&state, battery.diagnostic_id)
            .expect("diagnostic result command should succeed")
            .expect("diagnostic result should exist");
        let skill_results = list_diagnostic_skill_results(&state, battery.diagnostic_id)
            .expect("skill results command should succeed");
        let recommendations = list_diagnostic_recommendations(&state, battery.diagnostic_id)
            .expect("recommendations command should succeed");
        let primary_report = result
            .audience_reports
            .first()
            .cloned()
            .expect("audience report should exist");
        let report = get_diagnostic_audience_report(
            &state,
            battery.diagnostic_id,
            primary_report.audience.clone(),
        )
        .expect("audience report command should succeed");

        assert!(
            result
                .session_scores
                .iter()
                .any(|score| score.phase_code == "endurance")
        );
        assert!(
            result
                .session_scores
                .iter()
                .any(|score| score.phase_code == "recovery")
        );
        assert!(!result.skill_results.is_empty());
        assert!(!result.recommendations.is_empty());
        assert!(!result.audience_reports.is_empty());
        assert_eq!(skill_results, result.skill_results);
        assert_eq!(recommendations, result.recommendations);
        assert_eq!(report, Some(primary_report));
    }

    #[test]
    fn submit_diagnostic_attempt_accepts_constructed_response_payloads() {
        use ecoach_diagnostics::{DiagnosticEngine, DiagnosticMode};

        let state = setup_state();
        let student = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Abena".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student account should create");

        let (diagnostic_id, attempt_id) = state
            .with_connection(|conn| {
                let subject_id: i64 = conn
                    .query_row("SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1", [], |row| {
                        row.get(0)
                    })
                    .map_err(|err| {
                        CommandError::from(ecoach_substrate::EcoachError::Storage(
                            err.to_string(),
                        ))
                    })?;
                conn.execute(
                    "INSERT INTO topics (subject_id, code, name)
                     VALUES (?1, 'CMD_DNA34_FR', 'Command Diagnostic Free Response')",
                    [subject_id],
                )
                .map_err(|err| {
                    CommandError::from(ecoach_substrate::EcoachError::Storage(err.to_string()))
                })?;
                let topic_id = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO questions (
                        subject_id, topic_id, stem, question_format, explanation_text,
                        difficulty_level, estimated_time_seconds, marks, is_active
                     ) VALUES (?1, ?2, 'Write 0.75 as a fraction in simplest form.', 'short_answer', 'Convert decimal to fraction.', 5100, 45, 1, 1)",
                    rusqlite::params![subject_id, topic_id],
                )
                .map_err(|err| {
                    CommandError::from(ecoach_substrate::EcoachError::Storage(err.to_string()))
                })?;
                let question_id = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO question_options (
                        question_id, option_label, option_text, is_correct, position
                     ) VALUES (?1, 'A', '3/4', 1, 1)",
                    [question_id],
                )
                .map_err(|err| {
                    CommandError::from(ecoach_substrate::EcoachError::Storage(err.to_string()))
                })?;

                let engine = DiagnosticEngine::new(conn);
                let battery = engine.start_diagnostic_battery(
                    student.id,
                    subject_id,
                    vec![topic_id],
                    DiagnosticMode::Quick,
                )?;
                let first_phase = battery
                    .phases
                    .first()
                    .expect("phase should exist")
                    .phase_number;
                let first_item = engine
                    .list_phase_items(battery.diagnostic_id, first_phase)?
                    .into_iter()
                    .next()
                    .expect("constructed response item should exist");
                Ok((battery.diagnostic_id, first_item.attempt_id))
            })
            .expect("battery should be created");

        submit_diagnostic_attempt(
            &state,
            diagnostic_id,
            SubmitDiagnosticAttemptInput {
                attempt_id,
                selected_option_id: None,
                response_time_ms: Some(11_000),
                confidence_level: Some("sure".to_string()),
                changed_answer_count: 0,
                skipped: false,
                timed_out: false,
                first_focus_at: None,
                first_input_at: None,
                concept_guess: Some("fraction_form".to_string()),
                final_answer: Some(serde_json::json!({ "text": "3/4" })),
                interaction_log: Some(serde_json::json!({ "typed": true })),
            },
        )
        .expect("constructed response command submission should succeed");

        let stored = state
            .with_connection(|conn| {
                Ok(conn
                    .query_row(
                        "SELECT selected_option_id, is_correct, final_answer_json
                         FROM diagnostic_item_attempts
                         WHERE id = ?1",
                        [attempt_id],
                        |row| {
                            Ok((
                                row.get::<_, Option<i64>>(0)?,
                                row.get::<_, Option<i64>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        },
                    )
                    .map_err(|err| {
                        CommandError::from(ecoach_substrate::EcoachError::Storage(err.to_string()))
                    })?)
            })
            .expect("attempt should persist");

        assert_eq!(stored.0, None);
        assert_eq!(stored.1, Some(1));
        assert!(stored.2.as_deref().unwrap_or_default().contains("3/4"));
    }

    fn setup_state() -> AppState {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                let service = PackService::new(conn);
                service.install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");
        state
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}
