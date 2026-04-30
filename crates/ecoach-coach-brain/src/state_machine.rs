use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Instant;

use crate::constitution::CoachConstitutionService;
use crate::plan_engine::{
    CoachBlocker, CoachMissionBrief, CoachRoadmapSnapshot, PlanEngine, StudyBudgetSnapshot,
};

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

#[derive(Debug, Clone)]
struct ActiveJourneySignal {
    route_id: i64,
    route_type: String,
    target_exam: Option<String>,
    station_id: i64,
    station_code: String,
    station_type: String,
    topic_id: Option<i64>,
    retry_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachBrainTrigger {
    ManualRefresh,
    AttemptSubmitted,
    SessionCompleted,
    DiagnosticCompleted,
    MissionGenerated,
    EngagementRecorded,
    ContentChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachRecoveryStateSummary {
    pub state_type: String,
    pub recovery_action: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachBrainOutput {
    pub trigger: String,
    pub state: CoachStateResolution,
    pub next_action: CoachNextAction,
    pub content_readiness: ContentReadinessResolution,
    pub roadmap: Option<CoachRoadmapSnapshot>,
    pub today_mission: Option<CoachMissionBrief>,
    pub study_budget: Option<StudyBudgetSnapshot>,
    pub blockers: Vec<CoachBlocker>,
    pub recovery_states: Vec<CoachRecoveryStateSummary>,
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
        LearnerJourneyState::ReviewDay => {
            let journey = load_active_journey_signal(conn, student_id)?;
            CoachNextAction {
                state: resolution.state,
                action_type: CoachActionType::StartTodayMission,
                title: "Run today's review mission".to_string(),
                subtitle: "Journey is waiting on review proof before it can move forward."
                    .to_string(),
                estimated_minutes: Some(15),
                route: "/coach/mission/today".to_string(),
                context: serde_json::json!({
                    "reason": resolution.reason,
                    "journey_route_id": journey.as_ref().map(|item| item.route_id),
                    "journey_station_id": journey.as_ref().map(|item| item.station_id),
                    "journey_station_code": journey.as_ref().map(|item| item.station_code.clone()),
                    "journey_station_type": journey.as_ref().map(|item| item.station_type.clone()),
                    "journey_topic_id": journey.as_ref().and_then(|item| item.topic_id),
                }),
            }
        }
        LearnerJourneyState::ExamMode => {
            let journey = load_active_journey_signal(conn, student_id)?;
            CoachNextAction {
                state: resolution.state,
                action_type: CoachActionType::StartTodayMission,
                title: "Run today's performance mission".to_string(),
                subtitle: "Journey is in performance mode and expects exam-style proof."
                    .to_string(),
                estimated_minutes: Some(20),
                route: "/coach/mission/today".to_string(),
                context: serde_json::json!({
                    "reason": resolution.reason,
                    "journey_route_id": journey.as_ref().map(|item| item.route_id),
                    "journey_station_id": journey.as_ref().map(|item| item.station_id),
                    "journey_station_code": journey.as_ref().map(|item| item.station_code.clone()),
                    "journey_station_type": journey.as_ref().map(|item| item.station_type.clone()),
                    "journey_topic_id": journey.as_ref().and_then(|item| item.topic_id),
                    "target_exam": journey.as_ref().and_then(|item| item.target_exam.clone()),
                }),
            }
        }
        LearnerJourneyState::ReadyForTodayMission => CoachNextAction {
            state: resolution.state,
            action_type: CoachActionType::StartTodayMission,
            title: "Start today’s mission".to_string(),
            subtitle: "Your coach has enough evidence and content to assign focused work."
                .to_string(),
            estimated_minutes: Some(20),
            route: "/coach/mission/today".to_string(),
            context: serde_json::json!({
                "subject_codes": readiness.subject_codes,
                "journey": load_active_journey_signal(conn, student_id)?.map(|item| serde_json::json!({
                    "route_id": item.route_id,
                    "route_type": item.route_type,
                    "station_id": item.station_id,
                    "station_code": item.station_code,
                    "station_type": item.station_type,
                    "topic_id": item.topic_id,
                })),
            }),
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

pub fn evaluate_coach_brain(
    conn: &Connection,
    student_id: i64,
    trigger: CoachBrainTrigger,
    horizon_days: usize,
) -> EcoachResult<CoachBrainOutput> {
    let total_start = Instant::now();
    eprintln!(
        "[perf][coach.evaluate_coach_brain] enter student_id={} trigger={:?} horizon_days={}",
        student_id, trigger, horizon_days
    );
    let content_readiness_start = Instant::now();
    let content_readiness = assess_content_readiness(conn, student_id)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] assess_content_readiness {:.1}ms",
        content_readiness_start.elapsed().as_secs_f64() * 1000.0
    );
    let state_start = Instant::now();
    let state = resolve_coach_state(conn, student_id)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] resolve_coach_state {:.1}ms",
        state_start.elapsed().as_secs_f64() * 1000.0
    );
    let orchestration_start = Instant::now();
    let orchestration =
        CoachConstitutionService::new(conn).build_orchestration_snapshot(student_id, None, None)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] build_orchestration_snapshot {:.1}ms",
        orchestration_start.elapsed().as_secs_f64() * 1000.0
    );
    let next_action = orchestration.next_action.clone();
    let plan_engine = PlanEngine::new(conn);
    let roadmap_start = Instant::now();
    let roadmap = plan_engine.get_coach_roadmap(student_id, horizon_days.max(1))?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] get_coach_roadmap {:.1}ms",
        roadmap_start.elapsed().as_secs_f64() * 1000.0
    );
    let mission_start = Instant::now();
    let today_mission = plan_engine.get_today_mission_brief(student_id)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] get_today_mission_brief {:.1}ms",
        mission_start.elapsed().as_secs_f64() * 1000.0
    );
    let budget_start = Instant::now();
    let study_budget = plan_engine.build_study_budget_snapshot(student_id, None)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] build_study_budget_snapshot {:.1}ms",
        budget_start.elapsed().as_secs_f64() * 1000.0
    );
    let blockers_start = Instant::now();
    let blockers = plan_engine.list_active_blockers(student_id, 5)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] list_active_blockers {:.1}ms",
        blockers_start.elapsed().as_secs_f64() * 1000.0
    );
    let content_projection_start = Instant::now();
    sync_content_readiness_projection(conn, student_id, &content_readiness)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] sync_content_readiness_projection {:.1}ms",
        content_projection_start.elapsed().as_secs_f64() * 1000.0
    );
    let lifecycle_projection_start = Instant::now();
    sync_lifecycle_state_projection(conn, student_id, &state)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] sync_lifecycle_state_projection {:.1}ms",
        lifecycle_projection_start.elapsed().as_secs_f64() * 1000.0
    );
    let next_action_projection_start = Instant::now();
    sync_next_action_projection(conn, student_id, &next_action)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] sync_next_action_projection {:.1}ms",
        next_action_projection_start.elapsed().as_secs_f64() * 1000.0
    );
    let recovery_projection_start = Instant::now();
    sync_recovery_state_projection(
        conn,
        student_id,
        &content_readiness,
        &state,
        &study_budget,
        !blockers.is_empty(),
    )?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] sync_recovery_state_projection {:.1}ms",
        recovery_projection_start.elapsed().as_secs_f64() * 1000.0
    );
    let recovery_states_start = Instant::now();
    let recovery_states = list_recovery_state_projection(conn, student_id)?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] list_recovery_state_projection {:.1}ms",
        recovery_states_start.elapsed().as_secs_f64() * 1000.0
    );
    let decision_trace_start = Instant::now();
    write_decision_trace(
        conn,
        student_id,
        &trigger,
        &content_readiness,
        &state,
        &next_action,
        roadmap.as_ref(),
        study_budget.as_ref(),
        blockers.len() as i64,
        Some((
            &orchestration.guardrail_status,
            orchestration.governance_checks.len() as i64,
        )),
    )?;
    eprintln!(
        "[perf][coach.evaluate_coach_brain] write_decision_trace {:.1}ms",
        decision_trace_start.elapsed().as_secs_f64() * 1000.0
    );
    eprintln!(
        "[perf][coach.evaluate_coach_brain] total {:.1}ms",
        total_start.elapsed().as_secs_f64() * 1000.0
    );

    Ok(CoachBrainOutput {
        trigger: format!("{:?}", trigger),
        state,
        next_action,
        content_readiness,
        roadmap,
        today_mission,
        study_budget,
        blockers,
        recovery_states,
    })
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

    if let Some(journey) = load_active_journey_signal(conn, student_id)? {
        if journey.retry_count >= 2 {
            return Ok(CoachStateResolution::new(
                LearnerJourneyState::PlanAdjustmentRequired,
                Some(format!(
                    "journey station {} has stalled after repeated retries",
                    journey.station_code
                )),
            ));
        }
        if journey.station_type == "review" {
            return Ok(CoachStateResolution::new(
                LearnerJourneyState::ReviewDay,
                Some(format!(
                    "journey station {} is waiting on review proof",
                    journey.station_code
                )),
            ));
        }
        if journey.station_type == "performance" && journey.target_exam.is_some() {
            return Ok(CoachStateResolution::new(
                LearnerJourneyState::ExamMode,
                Some(format!(
                    "journey station {} is in performance mode",
                    journey.station_code
                )),
            ));
        }
    }

    Ok(CoachStateResolution::new(
        LearnerJourneyState::ReadyForTodayMission,
        Some("ready for mission generation".to_string()),
    ))
}

fn load_active_journey_signal(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<Option<ActiveJourneySignal>> {
    conn.query_row(
        "SELECT jr.id, jr.route_type, jr.target_exam,
                js.id, js.station_code, js.station_type, js.topic_id,
                COALESCE(CAST(json_extract(js.evidence_json, '$.retry_count') AS INTEGER), 0)
         FROM journey_routes jr
         INNER JOIN journey_stations js
            ON js.route_id = jr.id
           AND js.status = 'active'
         WHERE jr.student_id = ?1
           AND jr.status = 'active'
         ORDER BY jr.id DESC, js.sequence_no ASC
         LIMIT 1",
        [student_id],
        |row| {
            Ok(ActiveJourneySignal {
                route_id: row.get(0)?,
                route_type: row.get(1)?,
                target_exam: row.get(2)?,
                station_id: row.get(3)?,
                station_code: row.get(4)?,
                station_type: row.get(5)?,
                topic_id: row.get(6)?,
                retry_count: row.get(7)?,
            })
        },
    )
    .optional()
    .map_err(|err| EcoachError::Storage(err.to_string()))
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

fn sync_content_readiness_projection(
    conn: &Connection,
    student_id: i64,
    readiness: &ContentReadinessResolution,
) -> EcoachResult<()> {
    conn.execute(
        "INSERT INTO coach_content_readiness (
            student_id, readiness_status, selected_subjects_json, installed_packs_json,
            topic_count, question_count, failure_reason, checked_at
         ) VALUES (?1, ?2, ?3, '[]', ?4, ?5, ?6, datetime('now'))
         ON CONFLICT(student_id) DO UPDATE SET
            readiness_status = excluded.readiness_status,
            selected_subjects_json = excluded.selected_subjects_json,
            topic_count = excluded.topic_count,
            question_count = excluded.question_count,
            failure_reason = excluded.failure_reason,
            checked_at = datetime('now')",
        rusqlite::params![
            student_id,
            content_readiness_status_code(readiness.status),
            json!(readiness.subject_codes).to_string(),
            readiness.topic_count,
            readiness.question_count,
            readiness.reason,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn sync_lifecycle_state_projection(
    conn: &Connection,
    student_id: i64,
    state: &CoachStateResolution,
) -> EcoachResult<()> {
    conn.execute(
        "INSERT INTO coach_lifecycle_states (
            student_id, current_state, blocking_reason, state_entered_at, updated_at
         ) VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
         ON CONFLICT(student_id) DO UPDATE SET
            current_state = excluded.current_state,
            blocking_reason = excluded.blocking_reason,
            updated_at = datetime('now')",
        rusqlite::params![
            student_id,
            learner_journey_state_code(state.state),
            state.reason,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn sync_next_action_projection(
    conn: &Connection,
    student_id: i64,
    action: &CoachNextAction,
) -> EcoachResult<()> {
    let context_json = serde_json::to_string(&action.context)
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
    conn.execute(
        "INSERT INTO coach_next_actions (
            student_id, action_type, title, subtitle, estimated_minutes, route, context_json,
            urgency_bp, computed_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
         ON CONFLICT(student_id) DO UPDATE SET
            action_type = excluded.action_type,
            title = excluded.title,
            subtitle = excluded.subtitle,
            estimated_minutes = excluded.estimated_minutes,
            route = excluded.route,
            context_json = excluded.context_json,
            urgency_bp = excluded.urgency_bp,
            computed_at = datetime('now')",
        rusqlite::params![
            student_id,
            coach_action_type_code(action.action_type),
            action.title,
            action.subtitle,
            action.estimated_minutes,
            action.route,
            context_json,
            action_urgency_bp(action),
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn sync_recovery_state_projection(
    conn: &Connection,
    student_id: i64,
    readiness: &ContentReadinessResolution,
    state: &CoachStateResolution,
    study_budget: &Option<StudyBudgetSnapshot>,
    has_blockers: bool,
) -> EcoachResult<()> {
    let mut active_types = Vec::new();
    match readiness.status {
        ContentReadinessStatus::NoSubjectsSelected
        | ContentReadinessStatus::NoPacksInstalled
        | ContentReadinessStatus::NoTopicsAvailable => {
            active_types.push((
                "no_content_installed",
                readiness
                    .reason
                    .clone()
                    .unwrap_or_else(|| "resolve content readiness".to_string()),
            ));
        }
        ContentReadinessStatus::TopicsExistButNoQuestions
        | ContentReadinessStatus::InsufficientQuestionCoverage => {
            active_types.push((
                "no_questions_for_topic",
                readiness
                    .reason
                    .clone()
                    .unwrap_or_else(|| "load real questions".to_string()),
            ));
        }
        ContentReadinessStatus::Ready => {}
    }
    if matches!(state.state, LearnerJourneyState::DiagnosticRequired) {
        active_types.push((
            "insufficient_evidence",
            "complete the diagnostic to generate stable coaching evidence".to_string(),
        ));
    }
    if matches!(
        state.state,
        LearnerJourneyState::BlockedOnTopic | LearnerJourneyState::RepairRequired
    ) || has_blockers
    {
        active_types.push((
            "topic_blocked_awaiting_repair",
            state
                .reason
                .clone()
                .unwrap_or_else(|| "repair the blocked topic".to_string()),
        ));
    }
    if matches!(state.state, LearnerJourneyState::PlanAdjustmentRequired) {
        active_types.push((
            "plan_generation_failed",
            state
                .reason
                .clone()
                .unwrap_or_else(|| "refresh the study plan".to_string()),
        ));
    }
    if study_budget
        .as_ref()
        .map(|budget| budget.remaining_minutes == 0 && budget.actual_minutes > 0)
        .unwrap_or(false)
    {
        active_types.push((
            "study_budget_exhausted",
            "today's planned study budget has been exhausted".to_string(),
        ));
    }

    conn.execute(
        "UPDATE coach_recovery_states
         SET resolved = 1, resolved_at = datetime('now')
         WHERE student_id = ?1 AND resolved = 0",
        [student_id],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    for (state_type, recovery_action) in active_types {
        conn.execute(
            "INSERT INTO coach_recovery_states (student_id, state_type, recovery_action, resolved)
             VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![student_id, state_type, recovery_action],
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    }
    Ok(())
}

fn list_recovery_state_projection(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<Vec<CoachRecoveryStateSummary>> {
    let mut statement = conn
        .prepare(
            "SELECT state_type, recovery_action, created_at
             FROM coach_recovery_states
             WHERE student_id = ?1 AND resolved = 0
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map([student_id], |row| {
            Ok(CoachRecoveryStateSummary {
                state_type: row.get(0)?,
                recovery_action: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(out)
}

fn write_decision_trace(
    conn: &Connection,
    student_id: i64,
    trigger: &CoachBrainTrigger,
    readiness: &ContentReadinessResolution,
    state: &CoachStateResolution,
    action: &CoachNextAction,
    roadmap: Option<&CoachRoadmapSnapshot>,
    study_budget: Option<&StudyBudgetSnapshot>,
    blocker_count: i64,
    governance: Option<(&str, i64)>,
) -> EcoachResult<()> {
    let input_summary_json = json!({
        "trigger": format!("{:?}", trigger),
        "student_id": student_id,
    })
    .to_string();
    let output_summary_json = json!({
      "state": learner_journey_state_code(state.state),
      "action_type": coach_action_type_code(action.action_type),
      "readiness_status": content_readiness_status_code(readiness.status),
    "has_roadmap": roadmap.is_some(),
    "remaining_minutes": study_budget.map(|item| item.remaining_minutes),
    "blocker_count": blocker_count,
    "guardrail_status": governance.map(|item| item.0),
    "governance_check_count": governance.map(|item| item.1),
    })
    .to_string();
    let reasoning_json = json!({
        "reason": state.reason,
        "next_action_title": action.title,
        "readiness_reason": readiness.reason,
    })
    .to_string();
    conn.execute(
        "INSERT INTO decision_traces (
            student_id, engine_name, decision_type, input_summary_json, output_summary_json,
            reasoning_json, confidence_bp
         ) VALUES (?1, 'coach_brain', 'hub_snapshot', ?2, ?3, ?4, ?5)",
        rusqlite::params![
            student_id,
            input_summary_json,
            output_summary_json,
            reasoning_json,
            action_urgency_bp(action),
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(())
}

fn content_readiness_status_code(status: ContentReadinessStatus) -> &'static str {
    match status {
        ContentReadinessStatus::Ready => "ready",
        ContentReadinessStatus::NoSubjectsSelected => "no_subjects_selected",
        ContentReadinessStatus::NoPacksInstalled => "no_packs_installed",
        ContentReadinessStatus::NoTopicsAvailable => "no_topics_available",
        ContentReadinessStatus::TopicsExistButNoQuestions => "topics_no_questions",
        ContentReadinessStatus::InsufficientQuestionCoverage => "insufficient_coverage",
    }
}

fn learner_journey_state_code(state: LearnerJourneyState) -> &'static str {
    match state {
        LearnerJourneyState::OnboardingRequired => "onboarding_required",
        LearnerJourneyState::SubjectSelectionRequired => "subject_selection_required",
        LearnerJourneyState::ContentReadinessRequired => "content_readiness_required",
        LearnerJourneyState::DiagnosticRequired => "diagnostic_required",
        LearnerJourneyState::PlanGenerationRequired => "plan_generation_required",
        LearnerJourneyState::ReadyForTodayMission => "ready_for_today_mission",
        LearnerJourneyState::MissionInProgress => "mission_in_progress",
        LearnerJourneyState::MissionReviewRequired => "mission_review_required",
        LearnerJourneyState::RepairRequired => "repair_required",
        LearnerJourneyState::BlockedOnTopic => "blocked_on_topic",
        LearnerJourneyState::PlanAdjustmentRequired => "plan_adjustment_required",
        LearnerJourneyState::ReviewDay => "review_day",
        LearnerJourneyState::ExamMode => "exam_mode",
        LearnerJourneyState::StalledNoContent => "stalled_no_content",
    }
}

fn coach_action_type_code(action: CoachActionType) -> &'static str {
    match action {
        CoachActionType::ContinueOnboarding => "continue_onboarding",
        CoachActionType::SelectSubjects => "select_subjects",
        CoachActionType::ResolveContent => "install_content",
        CoachActionType::StartDiagnostic => "start_diagnostic",
        CoachActionType::GeneratePlan => "generate_plan",
        CoachActionType::StartTodayMission => "start_today_mission",
        CoachActionType::ResumeMission => "resume_mission",
        CoachActionType::ReviewResults => "review_results",
        CoachActionType::StartRepair => "start_repair",
        CoachActionType::AdjustPlan => "view_today_plan",
        CoachActionType::ViewOverview => "view_today_plan",
    }
}

fn action_urgency_bp(action: &CoachNextAction) -> i64 {
    match action.action_type {
        CoachActionType::ResolveContent => 9_000,
        CoachActionType::StartRepair => 8_700,
        CoachActionType::ReviewResults => 8_200,
        CoachActionType::StartDiagnostic | CoachActionType::GeneratePlan => 7_800,
        CoachActionType::ResumeMission => 7_400,
        CoachActionType::StartTodayMission => 7_000,
        CoachActionType::AdjustPlan => 6_900,
        CoachActionType::ContinueOnboarding | CoachActionType::SelectSubjects => 6_200,
        CoachActionType::ViewOverview => 5_000,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;
    use crate::journey::JourneyService;

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

    #[test]
    fn coach_resolver_uses_active_journey_review_and_stall_signals() {
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
             VALUES (?1, 'BECE', '2030-01-01', date('now'), 30, 60, 'performance', 'active', '{}')",
            [student_id],
        )
        .expect("plan should insert");

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_ids = {
            let mut statement = conn
                .prepare("SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 2")
                .expect("statement should prepare");
            let rows = statement
                .query_map([subject_id], |row| row.get::<_, i64>(0))
                .expect("topic rows should query");
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.expect("topic id should map"));
            }
            ids
        };
        let performance_topic_id = topic_ids[0];
        let review_topic_id = topic_ids[1];

        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score,
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority
             ) VALUES (?1, ?2, 8200, 1800, 2200, 9800, 1800, 7600, 1500, 5, 2200)",
            params![student_id, performance_topic_id],
        )
        .expect("performance topic state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score,
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority
             ) VALUES (?1, ?2, 6100, 5100, 4300, 6200, 2200, 2600, 8400, 4, 2500)",
            params![student_id, review_topic_id],
        )
        .expect("review topic state should insert");
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, recall_fluency, decay_risk, review_due_at
             ) VALUES (?1, ?2, 'fading', 2400, 1800, 8600, datetime('now', '-1 day'))",
            params![student_id, review_topic_id],
        )
        .expect("memory state should insert");

        let route = JourneyService::new(&conn)
            .build_or_refresh_route(student_id, subject_id, Some("BECE"))
            .expect("route should build");
        let active_station = route
            .stations
            .iter()
            .find(|station| station.status == "active")
            .expect("active station should exist");
        assert_eq!(active_station.station_type, "review");

        let state = resolve_coach_state(&conn, student_id).expect("state should resolve");
        let next_action =
            resolve_next_coach_action(&conn, student_id).expect("next action should resolve");
        assert_eq!(state.state, LearnerJourneyState::ReviewDay);
        assert_eq!(next_action.action_type, CoachActionType::StartTodayMission);

        conn.execute(
            "UPDATE journey_stations
             SET evidence_json = json_set(evidence_json, '$.retry_count', 2)
             WHERE id = ?1",
            [active_station.id],
        )
        .expect("retry count should update");

        let stalled_state =
            resolve_coach_state(&conn, student_id).expect("stalled state should resolve");
        assert_eq!(
            stalled_state.state,
            LearnerJourneyState::PlanAdjustmentRequired
        );
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
