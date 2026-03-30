use ecoach_coach_brain::{
    AdaptationResult, CoachNextAction, CoachStateResolution, ComposedSession,
    ConsistencySnapshot, ContentReadinessResolution, DeadlinePressure, JourneyAdaptationEngine,
    JourneyRouteSnapshot, JourneyService, KnowledgeMapNode, PlanEngine, RouteMode,
    SessionComposer, TopicCase, assess_content_readiness, list_priority_topic_cases,
    resolve_coach_state, resolve_next_coach_action,
};
use ecoach_goals_calendar::GoalsCalendarService;
use ecoach_reporting::{DashboardService, StudentDashboard};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    dtos::{DailyReplanDto, FreeNowRecommendationDto},
    error::CommandError,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachStateDto {
    pub state: String,
    pub reason: String,
}

impl From<CoachStateResolution> for CoachStateDto {
    fn from(v: CoachStateResolution) -> Self {
        Self {
            state: format!("{:?}", v.state),
            reason: v.reason.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNextActionDto {
    pub state: String,
    pub action_type: String,
    pub title: String,
    pub subtitle: String,
    pub estimated_minutes: Option<i64>,
    pub route: String,
}

impl From<CoachNextAction> for CoachNextActionDto {
    fn from(v: CoachNextAction) -> Self {
        Self {
            state: format!("{:?}", v.state),
            action_type: format!("{:?}", v.action_type),
            title: v.title,
            subtitle: v.subtitle,
            estimated_minutes: v.estimated_minutes,
            route: v.route,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReadinessDto {
    pub status: String,
    pub subject_codes: Vec<String>,
    pub active_pack_count: i64,
    pub topic_count: i64,
    pub question_count: i64,
    pub reason: Option<String>,
}

impl From<ContentReadinessResolution> for ContentReadinessDto {
    fn from(v: ContentReadinessResolution) -> Self {
        Self {
            status: format!("{:?}", v.status),
            subject_codes: v.subject_codes,
            active_pack_count: v.active_pack_count,
            topic_count: v.topic_count,
            question_count: v.question_count,
            reason: v.reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub subject_code: String,
    pub priority_score: i64,
    pub mastery_score: i64,
    pub mastery_state: String,
    pub gap_score: i64,
    pub fragility_score: i64,
    pub memory_strength: i64,
    pub decay_risk: i64,
    pub evidence_count: i64,
    pub requires_probe: bool,
    pub intervention_mode: String,
    pub intervention_urgency: String,
    pub intervention_reason: String,
}

impl From<TopicCase> for TopicCaseDto {
    fn from(v: TopicCase) -> Self {
        Self {
            topic_id: v.topic_id,
            topic_name: v.topic_name,
            subject_code: v.subject_code,
            priority_score: v.priority_score as i64,
            mastery_score: v.mastery_score as i64,
            mastery_state: v.mastery_state,
            gap_score: v.gap_score as i64,
            fragility_score: v.fragility_score as i64,
            memory_strength: v.memory_strength as i64,
            decay_risk: v.decay_risk as i64,
            evidence_count: v.evidence_count,
            requires_probe: v.requires_probe,
            intervention_mode: v.recommended_intervention.mode,
            intervention_urgency: v.recommended_intervention.urgency,
            intervention_reason: v.recommended_intervention.reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentDashboardDto {
    pub student_name: String,
    pub exam_target: Option<String>,
    pub overall_readiness_band: String,
    pub subjects: Vec<SubjectSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSummaryDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub readiness_band: String,
    pub mastered_topic_count: usize,
    pub weak_topic_count: usize,
    pub total_topic_count: usize,
}

impl From<StudentDashboard> for StudentDashboardDto {
    fn from(v: StudentDashboard) -> Self {
        Self {
            student_name: v.student_name,
            exam_target: v.exam_target,
            overall_readiness_band: v.overall_readiness_band,
            subjects: v
                .subject_summaries
                .into_iter()
                .map(|s| SubjectSummaryDto {
                    subject_id: s.subject_id,
                    subject_name: s.subject_name,
                    readiness_band: s.readiness_band,
                    mastered_topic_count: s.mastered_topic_count,
                    weak_topic_count: s.weak_topic_count,
                    total_topic_count: s.total_topic_count,
                })
                .collect(),
        }
    }
}

pub type JourneyRouteSnapshotDto = JourneyRouteSnapshot;

pub fn get_coach_state(state: &AppState, student_id: i64) -> Result<CoachStateDto, CommandError> {
    state.with_connection(|conn| {
        let resolution = resolve_coach_state(conn, student_id)?;
        Ok(CoachStateDto::from(resolution))
    })
}

pub fn get_coach_next_action(
    state: &AppState,
    student_id: i64,
) -> Result<CoachNextActionDto, CommandError> {
    state.with_connection(|conn| {
        let action = resolve_next_coach_action(conn, student_id)?;
        Ok(CoachNextActionDto::from(action))
    })
}

pub fn get_content_readiness(
    state: &AppState,
    student_id: i64,
) -> Result<ContentReadinessDto, CommandError> {
    state.with_connection(|conn| {
        let readiness = assess_content_readiness(conn, student_id)?;
        Ok(ContentReadinessDto::from(readiness))
    })
}

pub fn get_priority_topics(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TopicCaseDto>, CommandError> {
    state.with_connection(|conn| {
        let cases = list_priority_topic_cases(conn, student_id, limit)?;
        Ok(cases.into_iter().map(TopicCaseDto::from).collect())
    })
}

pub fn get_student_dashboard(
    state: &AppState,
    student_id: i64,
) -> Result<StudentDashboardDto, CommandError> {
    state.with_connection(|conn| {
        let service = DashboardService::new(conn);
        let dashboard = service.get_student_dashboard(student_id)?;
        Ok(StudentDashboardDto::from(dashboard))
    })
}

pub fn build_or_refresh_journey_route(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    target_exam: Option<String>,
) -> Result<JourneyRouteSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyService::new(conn).build_or_refresh_route(
            student_id,
            subject_id,
            target_exam.as_deref(),
        )?)
    })
}

pub fn get_active_journey_route(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<JourneyRouteSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyService::new(conn).get_active_route(student_id, subject_id)?)
    })
}

pub fn complete_journey_station(
    state: &AppState,
    station_id: i64,
    evidence: Value,
) -> Result<JourneyRouteSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyService::new(conn).complete_station(station_id, &evidence)?)
    })
}

pub fn generate_today_mission(state: &AppState, student_id: i64) -> Result<i64, CommandError> {
    state.with_connection(|conn| Ok(PlanEngine::new(conn).generate_today_mission(student_id)?))
}

pub fn recommend_free_now_session(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    date: &str,
    minute_of_day: i64,
    available_minutes: i64,
) -> Result<FreeNowRecommendationDto, CommandError> {
    state.with_connection(|conn| {
        let recommendation = GoalsCalendarService::new(conn).recommend_free_now_session(
            student_id,
            subject_id,
            date,
            minute_of_day,
            available_minutes,
        )?;
        Ok(FreeNowRecommendationDto::from(recommendation))
    })
}

pub fn replan_remaining_day(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    date: &str,
    minute_of_day: i64,
) -> Result<DailyReplanDto, CommandError> {
    state.with_connection(|conn| {
        let replan = GoalsCalendarService::new(conn).replan_remaining_day(
            student_id,
            subject_id,
            date,
            minute_of_day,
        )?;
        Ok(DailyReplanDto::from(replan))
    })
}

// ── Journey adaptation commands ──

pub fn get_deadline_pressure(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<DeadlinePressure, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyAdaptationEngine::new(conn).compute_deadline_pressure(student_id, subject_id)?)
    })
}

pub fn adapt_journey_route(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<AdaptationResult, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyAdaptationEngine::new(conn).adapt_route(student_id, subject_id)?)
    })
}

pub fn get_consistency_snapshot(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<ConsistencySnapshot, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyAdaptationEngine::new(conn).get_consistency_snapshot(student_id, subject_id)?)
    })
}

pub fn record_study_day(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    minutes: i64,
    questions: i64,
    accuracy_bp: u16,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyAdaptationEngine::new(conn).record_study_day(
            student_id, subject_id, minutes, questions, accuracy_bp,
        )?)
    })
}

pub fn get_knowledge_map(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<KnowledgeMapNode>, CommandError> {
    state.with_connection(|conn| {
        Ok(JourneyAdaptationEngine::new(conn).refresh_knowledge_map(student_id, subject_id)?)
    })
}

pub fn compose_session(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    station_type: &str,
    route_mode: &str,
    daily_budget_minutes: i64,
) -> Result<ComposedSession, CommandError> {
    state.with_connection(|conn| {
        let mode = RouteMode::from_str(route_mode);
        Ok(SessionComposer::new(conn).compose_session(
            student_id, subject_id, station_type, mode, daily_budget_minutes,
        )?)
    })
}

pub fn set_exam_date(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    exam_date: &str,
    daily_budget_minutes: i64,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        conn.execute(
            "UPDATE journey_routes SET exam_date = ?1, daily_budget_minutes = ?2, updated_at = datetime('now')
             WHERE student_id = ?3 AND subject_id = ?4 AND status = 'active'",
            rusqlite::params![exam_date, daily_budget_minutes, student_id, subject_id],
        )
        .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_goals_calendar::{AvailabilityProfile, AvailabilityWindow, GoalsCalendarService};
    use ecoach_identity::CreateAccountInput;
    use ecoach_substrate::{AccountType, EntitlementTier};
    use rusqlite::params;

    use super::*;
    use crate::identity_commands;

    #[test]
    fn recommend_free_now_session_returns_outcome_aware_dto() {
        let state = seeded_state();
        let student_id = insert_student(&state);
        let (subject_id, topic_id) = load_math_subject_and_topic(&state);
        let date = "2026-03-30";

        seed_availability(&state, student_id);
        seed_topic_state(
            &state, student_id, topic_id, 4_200, 9_200, 7_300, 9_000, true,
        );
        seed_memory_state(
            &state,
            student_id,
            topic_id,
            "collapsed",
            2_200,
            8_800,
            Some(&format!("{date}T18:00:00Z")),
        );
        seed_baseline_session(&state, student_id, subject_id, topic_id, "2026-03-29");
        generate_daily_target(&state, student_id, subject_id, date);
        seed_solidification_outcome(
            &state,
            student_id,
            subject_id,
            topic_id,
            "failed",
            "reteach_before_retry",
            2_200,
            &format!("{date}T17:30:00Z"),
        );

        let recommendation =
            super::recommend_free_now_session(&state, student_id, subject_id, date, 19 * 60, 35)
                .expect("free-now dto should resolve");

        assert!(recommendation.available_now);
        assert_eq!(recommendation.session_type, "comeback_reteach");
        assert_eq!(recommendation.recommended_comeback_topic_id, Some(topic_id));
        assert_eq!(
            recommendation.recent_repair_outcome.as_deref(),
            Some("failed")
        );
        assert_eq!(
            recommendation.focus_topic_ids.first().copied(),
            Some(topic_id)
        );
        assert!(recommendation.pressure_score >= 7_000);
        assert!(recommendation.repair_buffer_minutes >= 12);
        assert!(recommendation.target_id.is_some());
    }

    #[test]
    fn replan_remaining_day_returns_retention_downshift_dto() {
        let state = seeded_state();
        let student_id = insert_student(&state);
        let (subject_id, topic_id) = load_math_subject_and_topic(&state);
        let date = "2026-03-30";

        seed_availability(&state, student_id);
        seed_topic_state(
            &state, student_id, topic_id, 7_600, 3_600, 1_800, 2_500, false,
        );
        seed_memory_state(
            &state,
            student_id,
            topic_id,
            "accessible",
            7_600,
            2_100,
            Some("2026-04-02T18:00:00Z"),
        );
        seed_solidification_outcome(
            &state,
            student_id,
            subject_id,
            topic_id,
            "success",
            "stabilize_memory",
            8_600,
            &format!("{date}T16:30:00Z"),
        );

        let replan = super::replan_remaining_day(&state, student_id, subject_id, date, 19 * 60)
            .expect("replan dto should resolve");

        assert_eq!(replan.next_session_type, "retention_check");
        assert_eq!(replan.recommended_comeback_topic_id, Some(topic_id));
        assert_eq!(replan.recent_repair_outcome.as_deref(), Some("success"));
        assert_eq!(replan.repair_buffer_minutes, 6);
        assert_eq!(replan.remaining_target_minutes, 6);
        assert!(replan.rationale.contains("retention"));
    }

    fn seeded_state() -> AppState {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                PackService::new(conn).install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");
        state
    }

    fn insert_student(state: &AppState) -> i64 {
        let account = identity_commands::create_account(
            state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kojo".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student account should create");
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
                     VALUES (?1, '[\"MATH\"]', 90)",
                    [account.id],
                )
                .map_err(storage_error)?;
                Ok(())
            })
            .expect("student profile should insert");
        account.id
    }

    fn load_math_subject_and_topic(state: &AppState) -> (i64, i64) {
        state
            .with_connection(|conn| {
                let subject_id = conn
                    .query_row(
                        "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;
                let topic_id = conn
                    .query_row(
                        "SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 1",
                        [subject_id],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;
                Ok((subject_id, topic_id))
            })
            .expect("math subject/topic should load")
    }

    fn seed_availability(state: &AppState, student_id: i64) {
        state
            .with_connection(|conn| {
                let service = GoalsCalendarService::new(conn);
                service.upsert_availability_profile(&AvailabilityProfile {
                    student_id,
                    timezone_name: "America/New_York".to_string(),
                    preferred_daily_minutes: 90,
                    min_session_minutes: 15,
                    max_session_minutes: 60,
                })?;
                service.replace_availability_windows(
                    student_id,
                    &[AvailabilityWindow {
                        weekday: 0,
                        start_minute: 18 * 60,
                        end_minute: 21 * 60,
                        is_preferred: true,
                    }],
                )?;
                Ok(())
            })
            .expect("availability should seed");
    }

    fn seed_topic_state(
        state: &AppState,
        student_id: i64,
        topic_id: i64,
        mastery_score: i64,
        priority_score: i64,
        fragility_score: i64,
        repair_priority: i64,
        is_urgent: bool,
    ) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO student_topic_states (
                        student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score,
                        fragility_score, pressure_collapse_index, decay_risk, memory_strength,
                        evidence_count, repair_priority, is_urgent
                     ) VALUES (?1, ?2, ?3, 'fragile', 8600, ?4, ?5, 5400, 6200, 3400, 3, ?6, ?7)",
                    params![
                        student_id,
                        topic_id,
                        mastery_score,
                        priority_score,
                        fragility_score,
                        repair_priority,
                        if is_urgent { 1 } else { 0 },
                    ],
                )
                .map_err(storage_error)?;
                Ok(())
            })
            .expect("topic state should seed");
    }

    fn seed_memory_state(
        state: &AppState,
        student_id: i64,
        topic_id: i64,
        memory_state: &str,
        memory_strength: i64,
        decay_risk: i64,
        review_due_at: Option<&str>,
    ) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO memory_states (
                        student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        student_id,
                        topic_id,
                        memory_state,
                        memory_strength,
                        decay_risk,
                        review_due_at,
                    ],
                )
                .map_err(storage_error)?;
                Ok(())
            })
            .expect("memory state should seed");
    }

    fn seed_baseline_session(
        state: &AppState,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        date: &str,
    ) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO sessions (
                        student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                        is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                        accuracy_score, avg_response_time_ms
                     ) VALUES (?1, 'practice', ?2, ?3, 8, 8, 0, 'completed', ?4, ?4, 8, 5, 6250, 24000)",
                    params![
                        student_id,
                        subject_id,
                        format!("[{}]", topic_id),
                        format!("{date} 18:00:00"),
                    ],
                )
                .map_err(storage_error)?;
                Ok(())
            })
            .expect("baseline session should seed");
    }

    fn generate_daily_target(state: &AppState, student_id: i64, subject_id: i64, date: &str) {
        state
            .with_connection(|conn| {
                GoalsCalendarService::new(conn)
                    .generate_daily_climb_target(student_id, subject_id, date)?;
                Ok(())
            })
            .expect("daily target should generate");
    }

    fn seed_solidification_outcome(
        state: &AppState,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        outcome: &str,
        next_action_hint: &str,
        accuracy_score: i64,
        occurred_at: &str,
    ) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO sessions (
                        student_id, session_type, subject_id, topic_ids, status, started_at, completed_at,
                        answered_questions, correct_questions, accuracy_score
                     ) VALUES (?1, 'gap_repair', ?2, ?3, 'completed', ?4, ?4, 4, 3, ?5)",
                    params![
                        student_id,
                        subject_id,
                        format!("[{}]", topic_id),
                        occurred_at,
                        accuracy_score,
                    ],
                )
                .map_err(storage_error)?;
                let session_id = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO solidification_sessions (
                        student_id, topic_id, session_id, status, completed_at
                     ) VALUES (?1, ?2, ?3, 'completed', ?4)",
                    params![student_id, topic_id, session_id, occurred_at],
                )
                .map_err(storage_error)?;
                conn.execute(
                    "INSERT INTO runtime_events (
                        event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                     ) VALUES (?1, 'session.interpreted', 'session', ?2, ?3, ?4, ?5)",
                    params![
                        format!("session-interpreted-{}", session_id),
                        session_id.to_string(),
                        format!("trace-session-{}", session_id),
                        Value::from(serde_json::json!({
                            "repair_outcome": outcome,
                            "next_action_hint": next_action_hint,
                            "topic_summaries": [
                                {
                                    "topic_id": topic_id,
                                    "accuracy_score": accuracy_score
                                }
                            ]
                        }))
                        .to_string(),
                        occurred_at,
                    ],
                )
                .map_err(storage_error)?;
                Ok(())
            })
            .expect("solidification outcome should seed");
    }

    fn storage_error(err: rusqlite::Error) -> CommandError {
        CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        }
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
