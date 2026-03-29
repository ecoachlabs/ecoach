use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnerJourneyState {
    OnboardingRequired,
    SubjectSelectionRequired,
    ContentReadinessRequired,
    DiagnosticRequired,
    PlanGenerationRequired,
    ReadyForTodayMission,
    MissionInProgress,
    MissionReviewRequired,
    RepairRequired,
    BlockedOnTopic,
    PlanAdjustmentRequired,
    ReviewDay,
    ExamMode,
    StalledNoContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachStateResolution {
    pub state: LearnerJourneyState,
    pub reason: Option<String>,
}

impl CoachStateResolution {
    pub fn new(state: LearnerJourneyState, reason: impl Into<Option<String>>) -> Self {
        Self {
            state,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentReadinessStatus {
    Ready,
    NoSubjectsSelected,
    NoPacksInstalled,
    NoTopicsAvailable,
    TopicsExistButNoQuestions,
    InsufficientQuestionCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReadinessResolution {
    pub status: ContentReadinessStatus,
    pub subject_codes: Vec<String>,
    pub active_pack_count: i64,
    pub topic_count: i64,
    pub question_count: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachActionType {
    ContinueOnboarding,
    SelectSubjects,
    ResolveContent,
    StartDiagnostic,
    GeneratePlan,
    StartTodayMission,
    ResumeMission,
    ReviewResults,
    StartRepair,
    AdjustPlan,
    ViewOverview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNextAction {
    pub state: LearnerJourneyState,
    pub action_type: CoachActionType,
    pub title: String,
    pub subtitle: String,
    pub estimated_minutes: Option<i64>,
    pub route: String,
    pub context: Value,
}

pub fn assess_content_readiness(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<ContentReadinessResolution> {
    let subject_codes = load_selected_subject_codes(conn, student_id)?;
    if subject_codes.is_empty() {
        return Ok(ContentReadinessResolution {
            status: ContentReadinessStatus::NoSubjectsSelected,
            subject_codes,
            active_pack_count: 0,
            topic_count: 0,
            question_count: 0,
            reason: Some("no preferred subjects selected".to_string()),
        });
    }

    let active_pack_count = count_matching_subject_codes(
        conn,
        "SELECT COUNT(*) FROM content_packs WHERE status = 'active' AND subject_code IN ({})",
        &subject_codes,
    )?;
    if active_pack_count == 0 {
        return Ok(ContentReadinessResolution {
            status: ContentReadinessStatus::NoPacksInstalled,
            subject_codes,
            active_pack_count,
            topic_count: 0,
            question_count: 0,
            reason: Some("no active content packs for selected subjects".to_string()),
        });
    }

    let topic_count = count_matching_subject_codes(
        conn,
        "SELECT COUNT(*) FROM topics WHERE subject_id IN (SELECT id FROM subjects WHERE code IN ({}))",
        &subject_codes,
    )?;
    if topic_count == 0 {
        return Ok(ContentReadinessResolution {
            status: ContentReadinessStatus::NoTopicsAvailable,
            subject_codes,
            active_pack_count,
            topic_count,
            question_count: 0,
            reason: Some("content packs exist but no topics are available".to_string()),
        });
    }

    let question_count = count_matching_subject_codes(
        conn,
        "SELECT COUNT(*) FROM questions WHERE is_active = 1 AND subject_id IN (SELECT id FROM subjects WHERE code IN ({}))",
        &subject_codes,
    )?;
    if question_count == 0 {
        return Ok(ContentReadinessResolution {
            status: ContentReadinessStatus::TopicsExistButNoQuestions,
            subject_codes,
            active_pack_count,
            topic_count,
            question_count,
            reason: Some("topics exist but no active questions are available".to_string()),
        });
    }

    if question_count < topic_count {
        return Ok(ContentReadinessResolution {
            status: ContentReadinessStatus::InsufficientQuestionCoverage,
            subject_codes,
            active_pack_count,
            topic_count,
            question_count,
            reason: Some("question coverage is too thin for stable coaching".to_string()),
        });
    }

    Ok(ContentReadinessResolution {
        status: ContentReadinessStatus::Ready,
        subject_codes,
        active_pack_count,
        topic_count,
        question_count,
        reason: Some("content readiness checks passed".to_string()),
    })
}

pub fn resolve_next_coach_action(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<CoachNextAction> {
    let resolution = resolve_coach_state(conn, student_id)?;
    let readiness = assess_content_readiness(conn, student_id)?;

    let action = match resolution.state {
        LearnerJourneyState::OnboardingRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::ContinueOnboarding,
            title: "Finish onboarding".to_string(),
            subtitle: "We need your baseline setup before coaching can begin.".to_string(),
            estimated_minutes: Some(5),
            route: "/coach/onboarding".to_string(),
            context: serde_json::json!({ "reason": resolution.reason }),
        },
        LearnerJourneyState::SubjectSelectionRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::SelectSubjects,
            title: "Select your study subjects".to_string(),
            subtitle: "Your coach needs to know which subjects to guide.".to_string(),
            estimated_minutes: Some(3),
            route: "/coach/onboarding/subjects".to_string(),
            context: serde_json::json!({ "reason": resolution.reason }),
        },
        LearnerJourneyState::ContentReadinessRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::ResolveContent,
            title: "Fix content readiness".to_string(),
            subtitle: readiness.reason.clone().unwrap_or_else(|| {
                "Your selected subjects need content before coaching can continue.".to_string()
            }),
            estimated_minutes: Some(5),
            route: "/coach/content".to_string(),
            context: serde_json::json!({
                "readiness_status": readiness.status,
                "subject_codes": readiness.subject_codes,
                "topic_count": readiness.topic_count,
                "question_count": readiness.question_count,
            }),
        },
        LearnerJourneyState::DiagnosticRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::StartDiagnostic,
            title: "Start your diagnostic".to_string(),
            subtitle: "We need baseline evidence before building your plan.".to_string(),
            estimated_minutes: Some(20),
            route: "/coach/diagnostic".to_string(),
            context: serde_json::json!({ "subject_codes": readiness.subject_codes }),
        },
        LearnerJourneyState::PlanGenerationRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::GeneratePlan,
            title: "Generate your study plan".to_string(),
            subtitle: "Your evidence is ready. Let the coach map your next days.".to_string(),
            estimated_minutes: Some(2),
            route: "/coach/plan".to_string(),
            context: serde_json::json!({ "subject_codes": readiness.subject_codes }),
        },
        LearnerJourneyState::MissionInProgress => {
            let (mission_id, target_minutes): (i64, Option<i64>) = conn
                .query_row(
                    "SELECT id, target_minutes FROM coach_missions
                     WHERE student_id = ?1 AND status = 'active'
                     ORDER BY id DESC LIMIT 1",
                    [student_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            CoachNextAction {
                state: resolution.state,
                action_type: CoachActionType::ResumeMission,
                title: "Resume your mission".to_string(),
                subtitle: "Your current coaching session is still in progress.".to_string(),
                estimated_minutes: target_minutes,
                route: format!("/coach/mission/{}", mission_id),
                context: serde_json::json!({ "mission_id": mission_id }),
            }
        }
        LearnerJourneyState::MissionReviewRequired => {
            let (memory_id, mission_id, review_due_at): (i64, i64, Option<String>) = conn
                .query_row(
                    "SELECT id, mission_id, review_due_at
                     FROM coach_mission_memories
                     WHERE student_id = ?1 AND review_status = 'pending'
                     ORDER BY created_at DESC, id DESC
                     LIMIT 1",
                    [student_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            CoachNextAction {
                state: resolution.state,
                action_type: CoachActionType::ReviewResults,
                title: "Review your last mission".to_string(),
                subtitle: "Your coach has feedback and the next step ready.".to_string(),
                estimated_minutes: Some(5),
                route: format!("/coach/review/{}", memory_id),
                context: serde_json::json!({
                    "memory_id": memory_id,
                    "mission_id": mission_id,
                    "review_due_at": review_due_at,
                }),
            }
        }
        LearnerJourneyState::BlockedOnTopic | LearnerJourneyState::RepairRequired => {
            CoachNextAction {
                state: resolution.state,
                action_type: CoachActionType::StartRepair,
                title: "Repair the blocked topic".to_string(),
                subtitle:
                    "A topic is blocking forward progress, so the coach is routing you to repair."
                        .to_string(),
                estimated_minutes: Some(20),
                route: "/coach/repair".to_string(),
                context: serde_json::json!({ "reason": resolution.reason }),
            }
        }
        LearnerJourneyState::PlanAdjustmentRequired => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::AdjustPlan,
            title: "Refresh your study plan".to_string(),
            subtitle: "Your plan is stale and needs recalibration.".to_string(),
            estimated_minutes: Some(3),
            route: "/coach/plan/refresh".to_string(),
            context: serde_json::json!({ "reason": resolution.reason }),
        },
        LearnerJourneyState::ReadyForTodayMission => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::StartTodayMission,
            title: "Start today’s mission".to_string(),
            subtitle: "Your coach has enough evidence and content to assign focused work."
                .to_string(),
            estimated_minutes: Some(20),
            route: "/coach/mission/today".to_string(),
            context: serde_json::json!({ "subject_codes": readiness.subject_codes }),
        },
        _ => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::ViewOverview,
            title: "Open coach overview".to_string(),
            subtitle: resolution
                .reason
                .clone()
                .unwrap_or_else(|| "Review your current coach state.".to_string()),
            estimated_minutes: None,
            route: "/coach".to_string(),
            context: serde_json::json!({ "reason": resolution.reason }),
        },
    };

    Ok(action)
}

pub fn resolve_coach_state(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<CoachStateResolution> {
    let first_run: i64 = conn
        .query_row(
            "SELECT first_run FROM accounts WHERE id = ?1",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if first_run == 1 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::OnboardingRequired,
            Some("student needs onboarding".to_string()),
        ));
    }

    let preferred_subjects: Option<String> = conn
        .query_row(
            "SELECT preferred_subjects FROM student_profiles WHERE account_id = ?1",
            [student_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if preferred_subjects.as_deref().unwrap_or("[]") == "[]" {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::SubjectSelectionRequired,
            Some("no preferred subjects selected".to_string()),
        ));
    }

    let has_active_mission: i64 = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM coach_missions WHERE student_id = ?1 AND status = 'active')",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if has_active_mission == 1 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::MissionInProgress,
            Some("active mission exists".to_string()),
        ));
    }

    let pending_review: i64 = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM coach_mission_memories
                WHERE student_id = ?1 AND review_status = 'pending'
            )",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if pending_review == 1 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::MissionReviewRequired,
            Some("a completed mission is waiting for review".to_string()),
        ));
    }

    let blocked_topics: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM coach_blockers WHERE student_id = ?1 AND resolved_at IS NULL",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if blocked_topics > 0 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::BlockedOnTopic,
            Some("active blockers exist".to_string()),
        ));
    }

    let readiness = assess_content_readiness(conn, student_id)?;
    if readiness.status != ContentReadinessStatus::Ready {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::ContentReadinessRequired,
            readiness.reason,
        ));
    }

    let has_diagnostic: i64 = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM diagnostic_instances WHERE student_id = ?1 AND status = 'completed')",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if has_diagnostic == 0 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::DiagnosticRequired,
            Some("no completed diagnostic".to_string()),
        ));
    }

    let has_plan: i64 = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM coach_plans WHERE student_id = ?1 AND status IN ('active', 'stale'))",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if has_plan == 0 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::PlanGenerationRequired,
            Some("no active plan".to_string()),
        ));
    }

    let stale_plan: i64 = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM coach_plans WHERE student_id = ?1 AND status = 'stale')",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    if stale_plan == 1 {
        return Ok(CoachStateResolution::new(
            LearnerJourneyState::PlanAdjustmentRequired,
            Some("plan is stale".to_string()),
        ));
    }

    Ok(CoachStateResolution::new(
        LearnerJourneyState::ReadyForTodayMission,
        Some("ready for mission generation".to_string()),
    ))
}

fn load_selected_subject_codes(conn: &Connection, student_id: i64) -> EcoachResult<Vec<String>> {
    let preferred_subjects: Option<String> = conn
        .query_row(
            "SELECT preferred_subjects FROM student_profiles WHERE account_id = ?1",
            [student_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let raw = preferred_subjects.unwrap_or_else(|| "[]".to_string());
    serde_json::from_str::<Vec<String>>(&raw)
        .map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn count_matching_subject_codes(
    conn: &Connection,
    sql_template: &str,
    subject_codes: &[String],
) -> EcoachResult<i64> {
    if subject_codes.is_empty() {
        return Ok(0);
    }

    let placeholders = subject_codes
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = sql_template.replace("{}", &placeholders);
    let mut statement = conn
        .prepare(&sql)
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let params = subject_codes
        .iter()
        .map(|code| rusqlite::types::Value::from(code.clone()))
        .collect::<Vec<_>>();

    statement
        .query_row(rusqlite::params_from_iter(params.iter()), |row| row.get(0))
        .map_err(|err| EcoachError::Storage(err.to_string()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn coach_resolver_requires_content_before_diagnostic() {
        let conn = open_test_database();
        let student_id = insert_student(&conn, true);

        let state = resolve_coach_state(&conn, student_id).expect("state should resolve");
        let next_action =
            resolve_next_coach_action(&conn, student_id).expect("next action should resolve");

        assert_eq!(state.state, LearnerJourneyState::ContentReadinessRequired);
        assert_eq!(next_action.action_type, CoachActionType::ResolveContent);
    }

    #[test]
    fn coach_resolver_moves_from_content_ready_to_diagnostic_then_plan() {
        let conn = open_test_database();
        let student_id = insert_student(&conn, true);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let state_after_pack =
            resolve_coach_state(&conn, student_id).expect("state should resolve after pack");
        assert_eq!(
            state_after_pack.state,
            LearnerJourneyState::DiagnosticRequired
        );

        conn.execute(
            "INSERT INTO diagnostic_instances (student_id, subject_id, session_mode, status, started_at, completed_at, result_json)
             VALUES (?1, (SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1), 'standard', 'completed', datetime('now'), datetime('now'), '{}')",
            [student_id],
        ).expect("diagnostic instance should insert");

        let state_after_diagnostic =
            resolve_coach_state(&conn, student_id).expect("state should resolve after diagnostic");
        let next_action = resolve_next_coach_action(&conn, student_id)
            .expect("next action should resolve after diagnostic");

        assert_eq!(
            state_after_diagnostic.state,
            LearnerJourneyState::PlanGenerationRequired
        );
        assert_eq!(next_action.action_type, CoachActionType::GeneratePlan);
    }

    #[test]
    fn coach_resolver_surfaces_pending_mission_review() {
        let conn = open_test_database();
        let student_id = insert_student(&conn, true);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO diagnostic_instances (student_id, subject_id, session_mode, status, started_at, completed_at, result_json)
             VALUES (?1, (SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1), 'standard', 'completed', datetime('now'), datetime('now'), '{}')",
            [student_id],
        )
        .expect("diagnostic should insert");
        conn.execute(
            "INSERT INTO coach_plans (student_id, exam_target, exam_date, start_date, total_days, daily_budget_minutes, current_phase, status, plan_data_json)
             VALUES (?1, 'BECE', '2030-01-01', '2029-01-01', 30, 60, 'performance', 'active', '{}')",
            [student_id],
        )
        .expect("plan should insert");
        let plan_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO coach_plan_days (plan_id, date, phase, target_minutes, status)
             VALUES (?1, date('now'), 'performance', 60, 'completed')",
            [plan_id],
        )
        .expect("plan day should insert");
        let plan_day_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO coach_missions (plan_day_id, student_id, title, reason, activity_type, target_minutes, status)
             VALUES (?1, ?2, 'Repair Mission', 'recent weakness', 'repair', 20, 'completed')",
            params![plan_day_id, student_id],
        )
        .expect("mission should insert");
        let mission_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO coach_mission_memories (mission_id, plan_day_id, student_id, mission_status, attempt_count, correct_count, next_action_type, review_status)
             VALUES (?1, ?2, ?3, 'repair_required', 4, 1, 'review_results', 'pending')",
            params![mission_id, plan_day_id, student_id],
        )
        .expect("mission memory should insert");

        let state = resolve_coach_state(&conn, student_id).expect("state should resolve");
        let next_action =
            resolve_next_coach_action(&conn, student_id).expect("next action should resolve");

        assert_eq!(state.state, LearnerJourneyState::MissionReviewRequired);
        assert_eq!(next_action.action_type, CoachActionType::ReviewResults);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn insert_student(conn: &Connection, with_subjects: bool) -> i64 {
        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        ).expect("student should be insertable");
        let student_id = conn.last_insert_rowid();
        let preferred_subjects = if with_subjects { "[\"MATH\"]" } else { "[]" };
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, ?2, 60)",
            params![student_id, preferred_subjects],
        ).expect("student profile should be insertable");
        student_id
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
