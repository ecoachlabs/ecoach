use chrono::{Days, NaiveDate, Utc};
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::CanonicalIntelligenceStore;
use crate::journey::JourneyService;
use crate::readiness_engine::ReadinessEngine;
use crate::topic_case::{TopicCase, list_priority_topic_cases};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachMissionMemory {
    pub id: i64,
    pub mission_id: i64,
    pub plan_day_id: Option<i64>,
    pub student_id: i64,
    pub session_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub mission_status: String,
    pub attempt_count: i64,
    pub correct_count: i64,
    pub accuracy_score: Option<i64>,
    pub avg_latency_ms: Option<i64>,
    pub misconception_tags: Vec<String>,
    pub review_due_at: Option<String>,
    pub next_action_type: String,
    pub strategy_effect: Option<String>,
    pub summary_json: String,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRewriteResult {
    pub previous_plan_id: i64,
    pub new_plan_id: i64,
    pub reason: String,
    pub carryover_topic_ids: Vec<i64>,
    pub pending_review_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachPlanActivity {
    pub id: i64,
    pub plan_day_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub activity_type: String,
    pub target_minutes: i64,
    pub sequence_order: i64,
    pub target_outcome: Value,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachPlanDaySnapshot {
    pub id: i64,
    pub date: String,
    pub phase: String,
    pub target_minutes: i64,
    pub carryover_minutes: i64,
    pub status: String,
    pub activities: Vec<CoachPlanActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachRoadmapSnapshot {
    pub plan_id: i64,
    pub student_id: i64,
    pub exam_target: String,
    pub exam_date: String,
    pub start_date: String,
    pub total_days: i64,
    pub days_completed: i64,
    pub days_remaining: i64,
    pub current_phase: String,
    pub daily_budget_minutes: i64,
    pub weekly_completion_bp: BasisPoints,
    pub current_readiness_score: BasisPoints,
    pub target_readiness_score: BasisPoints,
    pub readiness_band: String,
    pub today: Option<CoachPlanDaySnapshot>,
    pub upcoming: Vec<CoachPlanDaySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyBudgetSnapshot {
    pub date: String,
    pub planned_minutes: i64,
    pub carryover_minutes: i64,
    pub actual_minutes: i64,
    pub focus_minutes: i64,
    pub idle_minutes: i64,
    pub remaining_minutes: i64,
    pub completed_session_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachBlocker {
    pub id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub reason: String,
    pub severity: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachMissionBrief {
    pub mission_id: i64,
    pub plan_day_id: Option<i64>,
    pub student_id: i64,
    pub session_id: Option<i64>,
    pub title: String,
    pub reason: String,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub activity_type: String,
    pub target_minutes: i64,
    pub status: String,
    pub plan_day_phase: Option<String>,
    pub steps: Value,
    pub success_criteria: Value,
    pub question_ids: Vec<i64>,
}

pub struct PlanEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug)]
struct SessionOutcome {
    attempt_count: i64,
    correct_count: i64,
    accuracy_score: Option<BasisPoints>,
    timed_accuracy: Option<BasisPoints>,
    avg_latency_ms: Option<i64>,
    misconception_tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct SubjectMomentumProfile {
    current_stage: String,
    current_mode: String,
    momentum_score: BasisPoints,
    strain_score: BasisPoints,
    recovery_need_score: BasisPoints,
    streak_days: i64,
}

#[derive(Debug)]
struct MissionBlueprint {
    subject_id: i64,
    topic_id: i64,
    activity_type: &'static str,
    title: &'static str,
    reason: String,
    target_minutes: i64,
    steps: Value,
    success_criteria: Value,
}

#[derive(Debug, Clone)]
struct ActiveJourneyStation {
    route_id: i64,
    route_type: String,
    target_exam: Option<String>,
    station_id: i64,
    station_code: String,
    station_type: String,
    topic_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct MissionJourneyBinding {
    route_id: i64,
    station_id: i64,
    station_code: String,
    station_type: String,
}

#[derive(Debug, Clone)]
struct JourneyMissionFeedback {
    route_id: i64,
    route_status: String,
    transition: &'static str,
    completed_station_code: String,
    completed_station_type: String,
    retry_count: i64,
    active_station_code: Option<String>,
    active_station_type: Option<String>,
    active_topic_id: Option<i64>,
}

impl<'a> PlanEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn generate_plan(
        &self,
        student_id: i64,
        exam_target: &str,
        exam_date: &str,
        daily_budget_minutes: i64,
    ) -> EcoachResult<i64> {
        let today = Utc::now().date_naive();
        let exam = NaiveDate::parse_from_str(exam_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let total_days = (exam - today).num_days().max(1);
        let phase = phase_for_remaining_days(total_days);
        let selected_subjects = self.load_selected_subjects(student_id)?;
        let selected_subject_ids = self.load_subject_ids_for_codes(&selected_subjects)?;
        let plan_data_json = serde_json::to_string(&serde_json::json!({
            "selected_subjects": selected_subjects,
            "generated_at": today.to_string(),
            "phase_model": "days_to_exam",
            "daily_budget_minutes": daily_budget_minutes,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO coach_plans (
                    student_id, exam_target, exam_date, start_date, total_days,
                    daily_budget_minutes, current_phase, status, plan_data_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8)",
                params![
                    student_id,
                    exam_target,
                    exam_date,
                    today.to_string(),
                    total_days,
                    daily_budget_minutes,
                    phase,
                    plan_data_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let plan_id = self.conn.last_insert_rowid();

        for offset in 0..total_days {
            let plan_date = today
                .checked_add_days(Days::new(offset as u64))
                .ok_or_else(|| {
                    EcoachError::Validation("failed to compute plan date".to_string())
                })?;
            let remaining_days = total_days - offset;
            let day_phase = phase_for_remaining_days(remaining_days);
            let day_target_minutes = if is_review_day(offset) {
                (daily_budget_minutes / 2).max(15)
            } else {
                daily_budget_minutes
            };
            let day_status = if offset == 0 { "active" } else { "pending" };

            self.conn
                .execute(
                    "INSERT INTO coach_plan_days (plan_id, date, phase, target_minutes, status)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        plan_id,
                        plan_date.to_string(),
                        if is_review_day(offset) {
                            "review_day"
                        } else {
                            day_phase
                        },
                        day_target_minutes,
                        day_status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let plan_day_id = self.conn.last_insert_rowid();
            self.seed_plan_day_activities(
                plan_day_id,
                student_id,
                offset,
                if is_review_day(offset) {
                    "review_day"
                } else {
                    day_phase
                },
                day_target_minutes,
                &selected_subject_ids,
            )?;
        }
        self.append_runtime_event(DomainEvent::new(
            "plan.generated",
            plan_id.to_string(),
            serde_json::json!({
                "student_id": student_id,
                "exam_target": exam_target,
                "exam_date": exam_date,
                "phase": phase,
                "daily_budget_minutes": daily_budget_minutes,
                "plan_day_count": total_days,
                "selected_subjects": selected_subjects,
            }),
        ))?;

        Ok(plan_id)
    }

    pub fn rewrite_active_plan(
        &self,
        student_id: i64,
        reason: &str,
    ) -> EcoachResult<PlanRewriteResult> {
        let context = self.load_latest_plan_context(student_id)?;
        let carryover_topic_ids = self.load_plan_carryover_topic_ids(student_id, 5)?;
        let pending_review_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM coach_mission_memories
                 WHERE student_id = ?1 AND review_status = 'pending'",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE coach_plans
                 SET status = 'stale', updated_at = datetime('now')
                 WHERE student_id = ?1 AND status = 'active'",
                [student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let new_plan_id = self.generate_plan(
            student_id,
            &context.exam_target,
            &context.exam_date,
            context.daily_budget_minutes,
        )?;
        let carryover_minutes = pending_review_count * 10 + (carryover_topic_ids.len() as i64 * 5);
        self.conn
            .execute(
                "UPDATE coach_plan_days
                 SET carryover_minutes = ?1,
                     target_minutes = target_minutes + ?1
                 WHERE id = (
                    SELECT cpd.id
                    FROM coach_plan_days cpd
                    INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                    WHERE cp.id = ?2
                    ORDER BY cpd.date ASC, cpd.id ASC
                    LIMIT 1
                 )",
                params![carryover_minutes, new_plan_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let plan_data_json = serde_json::to_string(&serde_json::json!({
            "rewrite_reason": reason,
            "previous_plan_id": context.plan_id,
            "carryover_topic_ids": carryover_topic_ids,
            "pending_review_count": pending_review_count,
            "daily_budget_minutes": context.daily_budget_minutes,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE coach_plans
                 SET plan_data_json = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![plan_data_json, new_plan_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_runtime_event(DomainEvent::new(
            "plan.rewritten",
            new_plan_id.to_string(),
            serde_json::json!({
                "student_id": student_id,
                "previous_plan_id": context.plan_id,
                "reason": reason,
                "carryover_topic_ids": carryover_topic_ids,
                "pending_review_count": pending_review_count,
            }),
        ))?;

        Ok(PlanRewriteResult {
            previous_plan_id: context.plan_id,
            new_plan_id,
            reason: reason.to_string(),
            carryover_topic_ids,
            pending_review_count,
        })
    }

    pub fn generate_today_mission(&self, student_id: i64) -> EcoachResult<i64> {
        let today = Utc::now().date_naive().to_string();
        if let Some(existing_mission_id) = self.load_existing_today_mission(student_id, &today)? {
            return Ok(existing_mission_id);
        }

        let (plan_day_id, plan_day_phase, target_minutes) =
            self.ensure_active_plan_day(student_id, &today)?;
        let topic_cases = CanonicalIntelligenceStore::new(self.conn)
            .list_priority_topic_cases(student_id, 5)?;
        let topic_cases = if topic_cases.is_empty() {
            list_priority_topic_cases(self.conn, student_id, 5)?
        } else {
            topic_cases
        };
        let blueprint = self.build_mission_blueprint(
            student_id,
            &plan_day_phase,
            target_minutes,
            &topic_cases,
        )?;

        self.conn
            .execute(
                "INSERT INTO coach_missions (
                    plan_day_id, student_id, title, reason, subject_id, primary_topic_id,
                    activity_type, target_minutes, status, steps_json, success_criteria_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9, ?10)",
                params![
                    plan_day_id,
                    student_id,
                    blueprint.title,
                    blueprint.reason,
                    blueprint.subject_id,
                    blueprint.topic_id,
                    blueprint.activity_type,
                    blueprint.target_minutes,
                    serde_json::to_string(&blueprint.steps)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&blueprint.success_criteria)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mission_id = self.conn.last_insert_rowid();
        self.activate_plan_activity_for_mission(
            plan_day_id,
            blueprint.subject_id,
            blueprint.topic_id,
            blueprint.activity_type,
            blueprint.target_minutes,
        )?;
        self.append_runtime_event(DomainEvent::new(
            "mission.generated",
            mission_id.to_string(),
            serde_json::json!({
                "student_id": student_id,
                "subject_id": blueprint.subject_id,
                "topic_id": blueprint.topic_id,
                "activity_type": blueprint.activity_type,
                "plan_day_id": plan_day_id,
                "plan_day_phase": plan_day_phase,
                "target_minutes": blueprint.target_minutes,
                "steps": blueprint.steps,
                "success_criteria": blueprint.success_criteria,
            }),
        ))?;

        Ok(mission_id)
    }

    pub fn start_mission(&self, mission_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE coach_missions
                 SET status = 'active'
                 WHERE id = ?1 AND status IN ('pending', 'deferred')",
                [mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(DomainEvent::new(
            "mission.started",
            mission_id.to_string(),
            serde_json::json!({ "mission_id": mission_id }),
        ))?;
        Ok(())
    }

    pub fn complete_mission_from_session(
        &self,
        mission_id: i64,
        session_id: Option<i64>,
    ) -> EcoachResult<CoachMissionMemory> {
        let mission = self.load_mission_context(mission_id)?;
        let outcome = self.load_session_outcome(session_id)?;
        let prior_evidence_count =
            self.count_prior_coach_evidence(mission.student_id, mission.topic_id)?;
        let mission_status =
            derive_mission_status(&mission.activity_type, &outcome, prior_evidence_count);
        let review_due_at = derive_review_due_at(
            &mission.activity_type,
            &mission_status,
            outcome.accuracy_score,
        );
        let next_action_type = derive_next_action_type(&mission.activity_type, &mission_status);
        let strategy_effect = derive_strategy_effect(
            &mission.activity_type,
            &mission_status,
            outcome.accuracy_score,
            prior_evidence_count,
        );
        let summary_json = serde_json::to_string(&serde_json::json!({
            "mission_title": mission.title,
            "reason": mission.reason,
            "activity_type": mission.activity_type,
            "attempt_count": outcome.attempt_count,
            "correct_count": outcome.correct_count,
            "accuracy_score": outcome.accuracy_score,
            "timed_accuracy": outcome.timed_accuracy,
            "avg_latency_ms": outcome.avg_latency_ms,
            "misconception_tags": outcome.misconception_tags,
            "prior_evidence_count": prior_evidence_count,
            "mission_status": mission_status,
            "next_action_type": next_action_type,
            "strategy_effect": strategy_effect,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let misconception_tags_json = serde_json::to_string(&outcome.misconception_tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE coach_missions
                 SET status = 'completed', completed_at = datetime('now')
                 WHERE id = ?1",
                [mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE coach_plan_days
                 SET status = CASE WHEN ?2 > 0 THEN 'completed' ELSE 'partial' END
                 WHERE id = ?1",
                params![mission.plan_day_id, outcome.attempt_count],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.complete_plan_activity_from_mission(
            mission.plan_day_id,
            mission.subject_id,
            mission.topic_id,
            &mission.activity_type,
            &mission_status,
        )?;

        self.conn
            .execute(
                "INSERT INTO coach_session_evidence (
                    mission_id, student_id, subject_id, topic_id, activity_type, attempt_count,
                    correct_count, accuracy, avg_latency_ms, misconception_tags, completed_at
                    , timed_accuracy
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'), ?11)",
                params![
                    mission_id,
                    mission.student_id,
                    mission.subject_id,
                    mission.topic_id,
                    mission.activity_type,
                    outcome.attempt_count,
                    outcome.correct_count,
                    outcome.accuracy_score,
                    outcome.avg_latency_ms,
                    misconception_tags_json,
                    outcome.timed_accuracy,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.sync_coach_topic_profile(
            mission.student_id,
            mission.topic_id,
            outcome.attempt_count,
            outcome.misconception_tags.len() as i64,
            &mission_status,
        )?;
        self.update_blockers(mission.student_id, mission.topic_id, &mission_status)?;

        self.conn
            .execute(
                "INSERT INTO coach_mission_memories (
                    mission_id, plan_day_id, student_id, session_id, subject_id, topic_id,
                    mission_status, attempt_count, correct_count, accuracy_score, avg_latency_ms,
                    misconception_tags, review_due_at, next_action_type, strategy_effect,
                    summary_json, review_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 'pending')
                 ON CONFLICT(mission_id) DO UPDATE SET
                    session_id = excluded.session_id,
                    mission_status = excluded.mission_status,
                    attempt_count = excluded.attempt_count,
                    correct_count = excluded.correct_count,
                    accuracy_score = excluded.accuracy_score,
                    avg_latency_ms = excluded.avg_latency_ms,
                    misconception_tags = excluded.misconception_tags,
                    review_due_at = excluded.review_due_at,
                    next_action_type = excluded.next_action_type,
                    strategy_effect = excluded.strategy_effect,
                    summary_json = excluded.summary_json,
                    review_status = 'pending',
                    updated_at = datetime('now')",
                params![
                    mission_id,
                    mission.plan_day_id,
                    mission.student_id,
                    session_id,
                    mission.subject_id,
                    mission.topic_id,
                    mission_status,
                    outcome.attempt_count,
                    outcome.correct_count,
                    outcome.accuracy_score,
                    outcome.avg_latency_ms,
                    misconception_tags_json,
                    review_due_at,
                    next_action_type,
                    strategy_effect,
                    summary_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let memory_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM coach_mission_memories WHERE mission_id = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(DomainEvent::new(
            "mission.completed",
            mission_id.to_string(),
            serde_json::json!({
                "mission_id": mission_id,
                "session_id": session_id,
                "mission_status": mission_status,
                "review_due_at": review_due_at,
                "next_action_type": next_action_type,
                "timed_accuracy": outcome.timed_accuracy,
                "activity_type": mission.activity_type,
            }),
        ))?;
        self.append_runtime_event(DomainEvent::new(
            "review.scheduled",
            mission_id.to_string(),
            serde_json::json!({
                "memory_id": memory_id,
                "review_due_at": review_due_at,
                "student_id": mission.student_id,
            }),
        ))?;
        let journey_feedback = self.sync_journey_progress_from_mission(
            mission_id,
            &mission,
            &outcome,
            &mission_status,
        )?;
        self.apply_mission_feedback_to_plan(
            mission_id,
            &mission,
            &outcome,
            &mission_status,
            prior_evidence_count,
            journey_feedback.as_ref(),
        )?;

        self.get_pending_mission_review(mission.student_id)?
            .ok_or_else(|| EcoachError::NotFound("mission memory was not persisted".to_string()))
    }

    pub fn get_pending_mission_review(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<CoachMissionMemory>> {
        self.conn
            .query_row(
                "SELECT id, mission_id, plan_day_id, student_id, session_id, subject_id, topic_id,
                        mission_status, attempt_count, correct_count, accuracy_score, avg_latency_ms,
                        misconception_tags, review_due_at, next_action_type, strategy_effect,
                        summary_json, review_status
                 FROM coach_mission_memories
                 WHERE student_id = ?1 AND review_status = 'pending'
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                [student_id],
                map_coach_mission_memory,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn acknowledge_mission_review(&self, memory_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE coach_mission_memories
                 SET review_status = 'acknowledged', updated_at = datetime('now')
                 WHERE id = ?1",
                [memory_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn get_mission_brief(&self, mission_id: i64) -> EcoachResult<Option<CoachMissionBrief>> {
        self.conn
            .query_row(
                "SELECT cm.id, cm.plan_day_id, cm.student_id, cm.session_id, cm.title, cm.reason,
                        cm.subject_id, cm.primary_topic_id, cm.activity_type, cm.target_minutes,
                        cm.status, cpd.phase, cm.steps_json, cm.success_criteria_json,
                        cm.question_ids_json
                 FROM coach_missions cm
                 LEFT JOIN coach_plan_days cpd ON cpd.id = cm.plan_day_id
                 WHERE cm.id = ?1",
                [mission_id],
                |row| {
                    let steps_json: String = row.get(12)?;
                    let success_criteria_json: String = row.get(13)?;
                    let question_ids_json: String = row.get(14)?;
                    Ok(CoachMissionBrief {
                        mission_id: row.get(0)?,
                        plan_day_id: row.get(1)?,
                        student_id: row.get(2)?,
                        session_id: row.get(3)?,
                        title: row.get(4)?,
                        reason: row.get(5)?,
                        subject_id: row.get(6)?,
                        topic_id: row.get(7)?,
                        activity_type: row.get(8)?,
                        target_minutes: row.get(9)?,
                        status: row.get(10)?,
                        plan_day_phase: row.get(11)?,
                        steps: serde_json::from_str(&steps_json).map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                12,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })?,
                        success_criteria: serde_json::from_str(&success_criteria_json).map_err(
                            |err| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    13,
                                    rusqlite::types::Type::Text,
                                    Box::new(err),
                                )
                            },
                        )?,
                        question_ids: serde_json::from_str(&question_ids_json).map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                14,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_today_mission_brief(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<CoachMissionBrief>> {
        let today = Utc::now().date_naive().to_string();
        let mission_id = self.load_existing_today_mission(student_id, &today)?;
        match mission_id {
            Some(mission_id) => self.get_mission_brief(mission_id),
            None => Ok(None),
        }
    }

    pub fn get_or_prepare_today_mission(&self, student_id: i64) -> EcoachResult<CoachMissionBrief> {
        let mission_id = self.generate_today_mission(student_id)?;
        self.get_mission_brief(mission_id)?
            .ok_or_else(|| EcoachError::NotFound("today mission could not be loaded".to_string()))
    }

    pub fn attach_session_to_mission(
        &self,
        mission_id: i64,
        session_id: i64,
        question_ids: &[i64],
    ) -> EcoachResult<()> {
        let question_ids_json = serde_json::to_string(question_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE coach_missions
                 SET session_id = ?1,
                     question_ids_json = ?2
                 WHERE id = ?3",
                params![session_id, question_ids_json, mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn mark_mission_no_question_recovery(
        &self,
        student_id: i64,
        mission_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE coach_missions
                 SET status = 'deferred'
                 WHERE id = ?1 AND status IN ('pending', 'active')",
                [mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO coach_recovery_states (student_id, state_type, recovery_action)
                 VALUES (?1, 'no_questions_for_topic', ?2)",
                params![
                    student_id,
                    topic_id.map(|value| format!("load_real_questions_for_topic:{}", value)),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(DomainEvent::new(
            "mission.blocked_no_questions",
            mission_id.to_string(),
            json!({
                "student_id": student_id,
                "topic_id": topic_id,
                "mission_id": mission_id,
            }),
        ))?;
        Ok(())
    }

    pub fn get_coach_roadmap(
        &self,
        student_id: i64,
        horizon_days: usize,
    ) -> EcoachResult<Option<CoachRoadmapSnapshot>> {
        let Some(plan) = self.load_active_plan_snapshot(student_id)? else {
            return Ok(None);
        };
        let today = Utc::now().date_naive();
        let exam_date = NaiveDate::parse_from_str(&plan.exam_date, "%Y-%m-%d").unwrap_or(today);
        let days_remaining = (exam_date - today).num_days().max(0);
        let days_completed: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_plan_days
                 WHERE plan_id = ?1 AND status = 'completed'",
                [plan.plan_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let weekly_completion_bp = self.compute_weekly_completion_bp(plan.plan_id)?;
        let current_readiness_score = self.compute_overall_readiness_score(student_id)?;
        let target_readiness_score =
            target_readiness_for_days_remaining(plan.total_days, days_remaining);

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, date, phase, target_minutes, carryover_minutes, status
                 FROM coach_plan_days
                 WHERE plan_id = ?1
                   AND date >= date('now')
                 ORDER BY date ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![plan.plan_id, horizon_days.max(1) as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut day_snapshots = Vec::new();
        for row in rows {
            let (plan_day_id, date, phase, target_minutes, carryover_minutes, status) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let activities = self.list_plan_day_activities(plan_day_id)?;
            day_snapshots.push(CoachPlanDaySnapshot {
                id: plan_day_id,
                date,
                phase,
                target_minutes,
                carryover_minutes,
                status,
                activities,
            });
        }
        let today_snapshot = day_snapshots
            .iter()
            .find(|day| day.date == today.to_string())
            .cloned()
            .or_else(|| day_snapshots.first().cloned());
        let upcoming = day_snapshots
            .into_iter()
            .filter(|day| Some(day.id) != today_snapshot.as_ref().map(|item| item.id))
            .collect::<Vec<_>>();

        Ok(Some(CoachRoadmapSnapshot {
            plan_id: plan.plan_id,
            student_id,
            exam_target: plan.exam_target,
            exam_date: plan.exam_date,
            start_date: plan.start_date,
            total_days: plan.total_days,
            days_completed,
            days_remaining,
            current_phase: plan.current_phase,
            daily_budget_minutes: plan.daily_budget_minutes,
            weekly_completion_bp,
            current_readiness_score,
            target_readiness_score,
            readiness_band: readiness_band_for_score(current_readiness_score).to_string(),
            today: today_snapshot,
            upcoming,
        }))
    }

    pub fn build_study_budget_snapshot(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<Option<StudyBudgetSnapshot>> {
        let date = anchor_date
            .map(str::to_string)
            .unwrap_or_else(|| Utc::now().date_naive().to_string());
        let plan_day = self
            .conn
            .query_row(
                "SELECT cpd.target_minutes, cpd.carryover_minutes
                 FROM coach_plan_days cpd
                 INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                 WHERE cp.student_id = ?1
                   AND cp.status IN ('active', 'stale')
                   AND cpd.date = ?2
                 ORDER BY cpd.id DESC
                 LIMIT 1",
                params![student_id, date],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((planned_minutes, carryover_minutes)) = plan_day else {
            return Ok(None);
        };

        let (stored_active_ms, stored_idle_ms, focus_active_ms, completed_session_minutes): (
            i64,
            i64,
            i64,
            i64,
        ) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE(SUM(COALESCE(active_study_time_ms, 0)), 0),
                    COALESCE(SUM(COALESCE(idle_time_ms, 0)), 0),
                    COALESCE(SUM(CASE
                        WHEN COALESCE(focus_mode, 0) = 1 OR session_type = 'coach_mission'
                            THEN COALESCE(active_study_time_ms, 0)
                        ELSE 0
                    END), 0),
                    COALESCE(SUM(CASE
                        WHEN status = 'completed'
                            THEN COALESCE(active_study_time_ms, 0) / 60000
                        ELSE 0
                    END), 0)
                 FROM sessions
                 WHERE student_id = ?1
                   AND COALESCE(date(completed_at), date(last_activity_at), date(started_at), date(created_at)) = ?2",
                params![student_id, date],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let fallback_active_ms: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(response_time_ms), 0)
                 FROM student_question_attempts
                 WHERE student_id = ?1
                   AND date(submitted_at) = ?2",
                params![student_id, date],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let active_ms = stored_active_ms.max(fallback_active_ms);
        let actual_minutes = ((active_ms as f64) / 60000.0).ceil() as i64;
        let focus_minutes = ((focus_active_ms as f64) / 60000.0).ceil() as i64;
        let idle_minutes = ((stored_idle_ms as f64) / 60000.0).ceil() as i64;
        let remaining_minutes = (planned_minutes + carryover_minutes - actual_minutes).max(0);

        Ok(Some(StudyBudgetSnapshot {
            date,
            planned_minutes,
            carryover_minutes,
            actual_minutes,
            focus_minutes,
            idle_minutes,
            remaining_minutes,
            completed_session_minutes,
        }))
    }

    pub fn list_active_blockers(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<CoachBlocker>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT cb.id, cb.topic_id, t.name, cb.reason, cb.severity, cb.created_at
                 FROM coach_blockers cb
                 INNER JOIN topics t ON t.id = cb.topic_id
                 WHERE cb.student_id = ?1 AND cb.resolved_at IS NULL
                 ORDER BY cb.created_at DESC, cb.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit.max(1) as i64], |row| {
                Ok(CoachBlocker {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    topic_name: row.get(2)?,
                    reason: row.get(3)?,
                    severity: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    fn load_selected_subjects(&self, student_id: i64) -> EcoachResult<Vec<String>> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT preferred_subjects FROM student_profiles WHERE account_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        serde_json::from_str::<Vec<String>>(raw.as_deref().unwrap_or("[]"))
            .map_err(|err| EcoachError::Serialization(err.to_string()))
    }

    fn load_subject_ids_for_codes(&self, subject_codes: &[String]) -> EcoachResult<Vec<i64>> {
        if subject_codes.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = subject_codes
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id
             FROM subjects
             WHERE code IN ({})
             ORDER BY display_order ASC, id ASC",
            placeholders
        );
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let params = subject_codes
            .iter()
            .map(|code| rusqlite::types::Value::from(code.clone()))
            .collect::<Vec<_>>();
        let rows = statement
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    fn seed_plan_day_activities(
        &self,
        plan_day_id: i64,
        student_id: i64,
        day_offset: i64,
        phase: &str,
        target_minutes: i64,
        selected_subject_ids: &[i64],
    ) -> EcoachResult<()> {
        let templates = activity_templates_for_phase(phase);
        for (index, (activity_type, share_bp, outcome_text)) in templates.iter().enumerate() {
            let subject_id = if selected_subject_ids.is_empty() {
                None
            } else {
                Some(
                    selected_subject_ids
                        [((day_offset as usize) + index) % selected_subject_ids.len()],
                )
            };
            let topic_id = match subject_id {
                Some(subject_id) => {
                    self.load_anchor_topic_for_subject(student_id, subject_id, activity_type)?
                }
                None => None,
            };
            let activity_minutes = (((target_minutes.max(10) as f64) * (*share_bp as f64 / 100.0))
                .round() as i64)
                .max(5);
            self.conn
                .execute(
                    "INSERT INTO coach_plan_activities (
                        plan_day_id, subject_id, topic_id, activity_type, target_minutes,
                        sequence_order, target_outcome_json, status, notes
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)",
                    params![
                        plan_day_id,
                        subject_id,
                        topic_id,
                        *activity_type,
                        activity_minutes,
                        (index + 1) as i64,
                        json!({
                            "phase": phase,
                            "headline": outcome_text,
                            "target_minutes": activity_minutes,
                        })
                        .to_string(),
                        format!("seeded_for_{}", phase),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn load_anchor_topic_for_subject(
        &self,
        student_id: i64,
        subject_id: i64,
        activity_type: &str,
    ) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT t.id
                 FROM topics t
                 LEFT JOIN student_topic_states sts
                    ON sts.topic_id = t.id
                   AND sts.student_id = ?1
                 WHERE t.subject_id = ?2
                 ORDER BY
                    CASE
                        WHEN ?3 IN ('repair', 'worked_example', 'review', 'memory_reactivation')
                            THEN COALESCE(sts.gap_score, 10000)
                        ELSE 10000 - COALESCE(sts.mastery_score, 0)
                    END DESC,
                    COALESCE(sts.priority_score, 0) DESC,
                    t.display_order ASC,
                    t.id ASC
                 LIMIT 1",
                params![student_id, subject_id, activity_type],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn activate_plan_activity_for_mission(
        &self,
        plan_day_id: i64,
        subject_id: i64,
        topic_id: i64,
        activity_type: &str,
        target_minutes: i64,
    ) -> EcoachResult<()> {
        let updated = self
            .conn
            .execute(
                "UPDATE coach_plan_activities
                 SET status = 'active',
                     topic_id = COALESCE(topic_id, ?1),
                     updated_at = datetime('now')
                 WHERE id = (
                    SELECT id
                    FROM coach_plan_activities
                    WHERE plan_day_id = ?2
                      AND COALESCE(subject_id, ?3) = ?3
                      AND activity_type = ?4
                      AND status IN ('pending', 'deferred')
                    ORDER BY sequence_order ASC, id ASC
                    LIMIT 1
                 )",
                params![topic_id, plan_day_id, subject_id, activity_type],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if updated == 0 {
            self.conn
                .execute(
                    "INSERT INTO coach_plan_activities (
                        plan_day_id, subject_id, topic_id, activity_type, target_minutes,
                        sequence_order, target_outcome_json, status, notes
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8)",
                    params![
                        plan_day_id,
                        subject_id,
                        topic_id,
                        activity_type,
                        target_minutes,
                        99i64,
                        json!({ "source": "mission_fallback" }).to_string(),
                        "inserted_from_mission_generation",
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn complete_plan_activity_from_mission(
        &self,
        plan_day_id: Option<i64>,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        activity_type: &str,
        mission_status: &str,
    ) -> EcoachResult<()> {
        let Some(plan_day_id) = plan_day_id else {
            return Ok(());
        };
        let Some(subject_id) = subject_id else {
            return Ok(());
        };
        let resolved_status = if mission_status == "repair_required" {
            "blocked"
        } else if mission_status == "partial" {
            "deferred"
        } else {
            "completed"
        };
        self.conn
            .execute(
                "UPDATE coach_plan_activities
                 SET status = ?1,
                     topic_id = COALESCE(topic_id, ?2),
                     updated_at = datetime('now')
                 WHERE id = (
                    SELECT id
                    FROM coach_plan_activities
                    WHERE plan_day_id = ?3
                      AND COALESCE(subject_id, ?4) = ?4
                      AND activity_type = ?5
                      AND status IN ('active', 'pending', 'deferred')
                    ORDER BY CASE status WHEN 'active' THEN 0 ELSE 1 END, sequence_order ASC, id ASC
                    LIMIT 1
                 )",
                params![
                    resolved_status,
                    topic_id,
                    plan_day_id,
                    subject_id,
                    activity_type
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_active_plan_snapshot(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<ActivePlanSnapshot>> {
        self.conn
            .query_row(
                "SELECT id, COALESCE(exam_target, 'BECE'), COALESCE(exam_date, date('now', '+30 day')),
                        start_date, COALESCE(total_days, 1), daily_budget_minutes, current_phase
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'stale')
                 ORDER BY id DESC
                 LIMIT 1",
                [student_id],
                |row| {
                    Ok(ActivePlanSnapshot {
                        plan_id: row.get(0)?,
                        exam_target: row.get(1)?,
                        exam_date: row.get(2)?,
                        start_date: row.get(3)?,
                        total_days: row.get(4)?,
                        daily_budget_minutes: row.get(5)?,
                        current_phase: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_plan_day_activities(&self, plan_day_id: i64) -> EcoachResult<Vec<CoachPlanActivity>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, plan_day_id, subject_id, topic_id, activity_type, target_minutes,
                        sequence_order, target_outcome_json, status, notes
                 FROM coach_plan_activities
                 WHERE plan_day_id = ?1
                 ORDER BY sequence_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([plan_day_id], |row| {
                let target_outcome_json: String = row.get(7)?;
                Ok(CoachPlanActivity {
                    id: row.get(0)?,
                    plan_day_id: row.get(1)?,
                    subject_id: row.get(2)?,
                    topic_id: row.get(3)?,
                    activity_type: row.get(4)?,
                    target_minutes: row.get(5)?,
                    sequence_order: row.get(6)?,
                    target_outcome: serde_json::from_str(&target_outcome_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?,
                    status: row.get(8)?,
                    notes: row.get(9)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    fn compute_weekly_completion_bp(&self, plan_id: i64) -> EcoachResult<BasisPoints> {
        let (completed_days, total_days): (i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0),
                    COUNT(*)
                 FROM coach_plan_days
                 WHERE plan_id = ?1
                   AND date >= date('now', '-6 day')
                   AND date <= date('now')",
                [plan_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if total_days == 0 {
            return Ok(0);
        }
        Ok(clamp_bp((completed_days * 10_000) / total_days))
    }

    fn compute_overall_readiness_score(&self, student_id: i64) -> EcoachResult<BasisPoints> {
        let subject_ids =
            self.load_subject_ids_for_codes(&self.load_selected_subjects(student_id)?)?;
        if subject_ids.is_empty() {
            return Ok(0);
        }
        let mut total = 0i64;
        let mut count = 0i64;
        for subject_id in subject_ids {
            let snapshot =
                ReadinessEngine::new(self.conn).build_subject_readiness(student_id, subject_id)?;
            total += snapshot.readiness_score as i64;
            count += 1;
        }
        if count == 0 {
            Ok(0)
        } else {
            Ok(clamp_bp(total / count))
        }
    }

    fn load_latest_plan_context(&self, student_id: i64) -> EcoachResult<PlanContext> {
        self.conn
            .query_row(
                "SELECT id,
                        COALESCE(exam_target, 'BECE'),
                        COALESCE(exam_date, date('now', '+30 day')),
                        daily_budget_minutes
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'stale')
                 ORDER BY id DESC
                 LIMIT 1",
                [student_id],
                |row| {
                    Ok(PlanContext {
                        plan_id: row.get(0)?,
                        exam_target: row.get(1)?,
                        exam_date: row.get(2)?,
                        daily_budget_minutes: row.get(3)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_existing_today_mission(
        &self,
        student_id: i64,
        today: &str,
    ) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT cm.id
                 FROM coach_missions cm
                 INNER JOIN coach_plan_days cpd ON cpd.id = cm.plan_day_id
                 WHERE cm.student_id = ?1
                   AND cpd.date = ?2
                   AND cm.status IN ('pending', 'active')
                 ORDER BY CASE cm.status WHEN 'active' THEN 0 ELSE 1 END, cm.id DESC
                 LIMIT 1",
                params![student_id, today],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_active_journey_station(
        &self,
        student_id: i64,
        topic_cases: &[TopicCase],
        plan_day_phase: &str,
    ) -> EcoachResult<Option<ActiveJourneyStation>> {
        if topic_cases.is_empty() {
            return Ok(None);
        }
        let topic_order = topic_cases
            .iter()
            .enumerate()
            .map(|(index, case)| (case.topic_id, index as i64))
            .collect::<std::collections::HashMap<_, _>>();
        let mut statement = self
            .conn
            .prepare(
                "SELECT jr.id, jr.route_type, jr.target_exam,
                        js.id, js.station_code, js.station_type, js.topic_id
                 FROM journey_routes jr
                 INNER JOIN journey_stations js ON js.route_id = jr.id
                 WHERE jr.student_id = ?1
                   AND jr.status = 'active'
                   AND js.status = 'active'
                 ORDER BY jr.id DESC, js.sequence_no ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok(ActiveJourneyStation {
                    route_id: row.get(0)?,
                    route_type: row.get(1)?,
                    target_exam: row.get(2)?,
                    station_id: row.get(3)?,
                    station_code: row.get(4)?,
                    station_type: row.get(5)?,
                    topic_id: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut best: Option<(i64, ActiveJourneyStation)> = None;
        for row in rows {
            let station = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let Some(topic_id) = station.topic_id else {
                continue;
            };
            let Some(topic_rank) = topic_order.get(&topic_id) else {
                continue;
            };
            let fit_score = station_fit_score(&station.station_type, plan_day_phase);
            let score = fit_score + ((topic_cases.len() as i64 - *topic_rank).max(1) * 10);
            match &best {
                Some((best_score, _)) if *best_score >= score => {}
                _ => best = Some((score, station)),
            }
        }

        Ok(best.map(|(_, station)| station))
    }

    fn build_mission_blueprint(
        &self,
        student_id: i64,
        plan_day_phase: &str,
        target_minutes: i64,
        topic_cases: &[TopicCase],
    ) -> EcoachResult<MissionBlueprint> {
        let journey_station =
            self.load_active_journey_station(student_id, topic_cases, plan_day_phase)?;
        let mission_case = journey_station
            .as_ref()
            .and_then(|station| {
                station
                    .topic_id
                    .and_then(|topic_id| topic_cases.iter().find(|case| case.topic_id == topic_id))
            })
            .or_else(|| {
                topic_cases
                    .iter()
                    .find(|case| is_priority_repair_case(case))
            })
            .or_else(|| {
                if plan_day_phase == "review_day" {
                    topic_cases
                        .iter()
                        .find(|case| is_review_priority_case(case))
                } else {
                    None
                }
            })
            .or_else(|| {
                topic_cases
                    .iter()
                    .find(|case| is_review_priority_case(case))
            })
            .or_else(|| {
                if plan_day_phase == "final_revision" || plan_day_phase == "performance" {
                    topic_cases
                        .iter()
                        .find(|case| supports_performance_push(case))
                } else {
                    None
                }
            })
            .or_else(|| topic_cases.first())
            .ok_or_else(|| EcoachError::NotFound("no prioritized topic found".to_string()))?;
        let subject_id = self.load_subject_id_for_topic(mission_case.topic_id)?;
        let readiness = ReadinessEngine::new(self.conn)
            .build_subject_readiness(student_id, subject_id)
            .ok();
        let momentum = self.load_subject_momentum_profile(student_id, subject_id)?;
        let exam_days_remaining = self.load_exam_days_remaining(student_id)?;
        let activity_type = self.resolve_activity_type(
            student_id,
            mission_case,
            plan_day_phase,
            readiness.as_ref(),
            &momentum,
            exam_days_remaining,
        )?;
        let activity_type = journey_station
            .as_ref()
            .map(|station| {
                align_activity_type_with_station(
                    activity_type,
                    &station.station_type,
                    mission_case,
                    exam_days_remaining,
                )
            })
            .unwrap_or(activity_type);
        let title = mission_title_for_activity(activity_type);
        let target_minutes =
            adaptive_target_minutes(target_minutes, mission_case, &momentum, activity_type);
        let reason = mission_reason(
            plan_day_phase,
            mission_case,
            readiness.as_ref(),
            &momentum,
            exam_days_remaining,
            activity_type,
            journey_station.as_ref(),
        );
        let steps = json!({
            "plan_day_phase": plan_day_phase,
            "topic_id": mission_case.topic_id,
            "topic_name": mission_case.topic_name,
            "primary_hypothesis_code": mission_case.primary_hypothesis_code,
            "diagnosis_certainty": mission_case.diagnosis_certainty,
            "recommended_intervention": mission_case.recommended_intervention,
            "proof_gaps": mission_case.proof_gaps,
            "open_questions": mission_case.open_questions,
            "coach_memory": {
                "memory_state": mission_case.memory_state,
                "memory_strength": mission_case.memory_strength,
                "decay_risk": mission_case.decay_risk,
                "recent_accuracy": mission_case.recent_accuracy,
            },
            "beat_yesterday": {
                "current_stage": momentum.current_stage,
                "current_mode": momentum.current_mode,
                "momentum_score": momentum.momentum_score,
                "strain_score": momentum.strain_score,
                "recovery_need_score": momentum.recovery_need_score,
                "streak_days": momentum.streak_days,
            },
            "journey": journey_station.as_ref().map(|station| json!({
                "route_id": station.route_id,
                "route_type": station.route_type,
                "target_exam": station.target_exam,
                "station_id": station.station_id,
                "station_code": station.station_code,
                "station_type": station.station_type,
                "topic_id": station.topic_id,
            })),
            "steps": mission_steps_for_activity(activity_type, mission_case),
        });
        let success_criteria = json!({
            "activity_type": activity_type,
            "target_accuracy_score": target_accuracy_for_activity(activity_type, mission_case),
            "min_attempts": min_attempts_for_activity(activity_type),
            "max_avg_latency_ms": target_latency_for_activity(activity_type),
            "proof_goal": proof_goal_for_activity(activity_type, mission_case),
            "review_due_expected": matches!(activity_type, "memory_reactivation" | "review"),
            "journey_station_type": journey_station.as_ref().map(|station| station.station_type.clone()),
            "journey_station_code": journey_station.as_ref().map(|station| station.station_code.clone()),
        });

        Ok(MissionBlueprint {
            subject_id,
            topic_id: mission_case.topic_id,
            activity_type,
            title,
            reason,
            target_minutes,
            steps,
            success_criteria,
        })
    }

    fn apply_mission_feedback_to_plan(
        &self,
        mission_id: i64,
        mission: &MissionContext,
        outcome: &SessionOutcome,
        mission_status: &str,
        prior_evidence_count: i64,
        journey_feedback: Option<&JourneyMissionFeedback>,
    ) -> EcoachResult<()> {
        let has_open_plan: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM coach_plans
                    WHERE student_id = ?1 AND status = 'active'
                 )",
                [mission.student_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            == 1;
        if !has_open_plan {
            return Ok(());
        }

        let timed_collapse = outcome.timed_accuracy.unwrap_or(10_000) < 5_500;
        let rewrite_reason = if mission_status == "repair_required" {
            Some("mission_repair_required")
        } else if journey_feedback
            .map(|feedback| feedback.retry_count >= 2)
            .unwrap_or(false)
        {
            Some("journey_station_stalled")
        } else if mission_status == "review_due" && (timed_collapse || prior_evidence_count >= 2) {
            Some("mission_regression_requires_replan")
        } else {
            None
        };
        if let Some(rewrite_reason) = rewrite_reason {
            let rewrite = self.rewrite_active_plan(mission.student_id, rewrite_reason)?;
            self.append_runtime_event(DomainEvent::new(
                "plan.adjusted_from_mission",
                rewrite.new_plan_id.to_string(),
                json!({
                    "mission_id": mission_id,
                    "student_id": mission.student_id,
                    "topic_id": mission.topic_id,
                    "mission_status": mission_status,
                    "mode": "rewrite",
                    "reason": rewrite_reason,
                    "previous_plan_id": rewrite.previous_plan_id,
                    "new_plan_id": rewrite.new_plan_id,
                }),
            ))?;
            return Ok(());
        }

        let (phase_override, carryover_minutes) = if let Some(feedback) = journey_feedback {
            let phase = feedback
                .active_station_type
                .as_deref()
                .map(plan_phase_for_station_type)
                .or_else(|| {
                    if mission_status == "review_due"
                        && mission.activity_type == "memory_reactivation"
                    {
                        Some("review_day")
                    } else {
                        None
                    }
                });
            let carryover = match feedback.transition {
                "retry_required" => carryover_minutes_for_station_retry(
                    feedback
                        .active_station_type
                        .as_deref()
                        .unwrap_or(&feedback.completed_station_type),
                    mission_status,
                ),
                "advanced" => carryover_minutes_for_next_station(
                    feedback.active_station_type.as_deref(),
                    mission_status,
                ),
                "route_completed" => 0,
                _ => 0,
            };
            (phase, carryover)
        } else {
            let phase = if mission.activity_type == "memory_reactivation" {
                Some("review_day")
            } else {
                None
            };
            let carryover = match mission_status {
                "review_due" => {
                    if mission.activity_type == "memory_reactivation" {
                        20
                    } else {
                        15
                    }
                }
                "partial" => 10,
                _ => 0,
            };
            (phase, carryover)
        };
        if carryover_minutes == 0 && phase_override.is_none() {
            return Ok(());
        }

        let updated_rows = self
            .conn
            .execute(
                "UPDATE coach_plan_days
                 SET carryover_minutes = carryover_minutes + ?1,
                     target_minutes = MAX(target_minutes, 15) + ?1,
                     phase = COALESCE(?2, phase)
                 WHERE id = (
                    SELECT cpd.id
                    FROM coach_plan_days cpd
                    INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                    WHERE cp.student_id = ?3
                      AND cp.status = 'active'
                      AND cpd.date > date('now')
                    ORDER BY cpd.date ASC, cpd.id ASC
                    LIMIT 1
                 )",
                params![carryover_minutes, phase_override, mission.student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if updated_rows > 0 {
            self.append_runtime_event(DomainEvent::new(
                "plan.adjusted_from_mission",
                mission_id.to_string(),
                json!({
                    "mission_id": mission_id,
                    "student_id": mission.student_id,
                    "topic_id": mission.topic_id,
                    "mission_status": mission_status,
                    "mode": if journey_feedback.is_some() { "journey_alignment" } else { "carryover" },
                    "carryover_minutes": carryover_minutes,
                    "phase_override": phase_override,
                    "journey_feedback": journey_feedback.as_ref().map(|feedback| json!({
                        "route_id": feedback.route_id,
                        "route_status": feedback.route_status,
                        "transition": feedback.transition,
                        "completed_station_code": feedback.completed_station_code,
                        "completed_station_type": feedback.completed_station_type,
                        "retry_count": feedback.retry_count,
                        "active_station_code": feedback.active_station_code,
                        "active_station_type": feedback.active_station_type,
                        "active_topic_id": feedback.active_topic_id,
                    })),
                }),
            ))?;
        }

        Ok(())
    }

    fn sync_journey_progress_from_mission(
        &self,
        mission_id: i64,
        mission: &MissionContext,
        outcome: &SessionOutcome,
        mission_status: &str,
    ) -> EcoachResult<Option<JourneyMissionFeedback>> {
        let Some(binding) = mission_journey_binding(&mission.steps) else {
            return Ok(None);
        };
        let evidence =
            self.build_journey_station_evidence(mission_id, mission, outcome, mission_status)?;
        let snapshot =
            JourneyService::new(self.conn).complete_station(binding.station_id, &evidence)?;
        let active_station = snapshot
            .stations
            .iter()
            .find(|station| station.status == "active");
        let retry_count = active_station
            .filter(|station| station.id == binding.station_id)
            .and_then(|station| station.evidence.get("retry_count"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let transition = if snapshot.route.status == "completed" {
            "route_completed"
        } else if active_station
            .map(|station| station.id == binding.station_id)
            .unwrap_or(false)
        {
            "retry_required"
        } else {
            "advanced"
        };

        Ok(Some(JourneyMissionFeedback {
            route_id: binding.route_id,
            route_status: snapshot.route.status.clone(),
            transition,
            completed_station_code: binding.station_code,
            completed_station_type: binding.station_type,
            retry_count,
            active_station_code: active_station.map(|station| station.station_code.clone()),
            active_station_type: active_station.map(|station| station.station_type.clone()),
            active_topic_id: active_station.and_then(|station| station.topic_id),
        }))
    }

    fn build_journey_station_evidence(
        &self,
        mission_id: i64,
        mission: &MissionContext,
        outcome: &SessionOutcome,
        mission_status: &str,
    ) -> EcoachResult<Value> {
        let mastery_score = match mission.topic_id {
            Some(topic_id) => self
                .conn
                .query_row(
                    "SELECT mastery_score
                     FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                    params![mission.student_id, topic_id],
                    |row| row.get::<_, Option<i64>>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .flatten(),
            None => None,
        };
        let readiness_score = match mission.subject_id {
            Some(subject_id) => Some(
                ReadinessEngine::new(self.conn)
                    .build_subject_readiness(mission.student_id, subject_id)?
                    .readiness_score as i64,
            ),
            None => None,
        };

        Ok(json!({
            "status": if mission_status == "completed" { "passed" } else { "needs_retry" },
            "mission_id": mission_id,
            "mission_status": mission_status,
            "activity_type": mission.activity_type,
            "answered_questions": outcome.attempt_count,
            "attempt_count": outcome.attempt_count,
            "correct_count": outcome.correct_count,
            "accuracy_score": outcome.accuracy_score.map(|score| score as i64),
            "timed_accuracy": outcome.timed_accuracy.map(|score| score as i64),
            "avg_latency_ms": outcome.avg_latency_ms,
            "timed_success": mission_status == "completed"
                && outcome.timed_accuracy.unwrap_or(0) >= 7_000,
            "delayed_recall_passed": mission_status == "completed"
                && matches!(mission.activity_type.as_str(), "memory_reactivation" | "review"),
            "mastery_score": mastery_score,
            "readiness_score": readiness_score,
            "misconception_tags": outcome.misconception_tags,
        }))
    }

    fn load_plan_carryover_topic_ids(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut topic_ids = Vec::new();

        let mut blocker_statement = self
            .conn
            .prepare(
                "SELECT DISTINCT topic_id
                 FROM coach_blockers
                 WHERE student_id = ?1 AND resolved_at IS NULL
                 ORDER BY id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let blocker_rows = blocker_statement
            .query_map(params![student_id, limit as i64], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in blocker_rows {
            let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !topic_ids.contains(&topic_id) {
                topic_ids.push(topic_id);
            }
        }

        if topic_ids.len() < limit {
            let remaining = (limit - topic_ids.len()) as i64;
            let mut priority_statement = self
                .conn
                .prepare(
                    "SELECT topic_id
                     FROM student_topic_states
                     WHERE student_id = ?1
                     ORDER BY priority_score DESC, gap_score DESC, id DESC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let priority_rows = priority_statement
                .query_map(params![student_id, remaining], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in priority_rows {
                let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                if !topic_ids.contains(&topic_id) {
                    topic_ids.push(topic_id);
                }
            }
        }

        Ok(topic_ids)
    }

    fn ensure_active_plan_day(
        &self,
        student_id: i64,
        today: &str,
    ) -> EcoachResult<(i64, String, i64)> {
        let active_day = self
            .conn
            .query_row(
                "SELECT cpd.id, cpd.phase, cpd.target_minutes
                 FROM coach_plan_days cpd
                 INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                 WHERE cp.student_id = ?1
                   AND cp.status IN ('active', 'stale')
                   AND cpd.date = ?2
                 ORDER BY cpd.id DESC
                 LIMIT 1",
                params![student_id, today],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (plan_day_id, phase, target_minutes) = active_day.ok_or_else(|| {
            EcoachError::NotFound("no active coach plan day for today".to_string())
        })?;
        self.conn
            .execute(
                "UPDATE coach_plan_days SET status = 'active' WHERE id = ?1 AND status = 'pending'",
                [plan_day_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok((plan_day_id, phase, target_minutes))
    }

    fn resolve_activity_type(
        &self,
        student_id: i64,
        topic_case: &TopicCase,
        plan_day_phase: &str,
        readiness: Option<&crate::readiness_engine::StudentReadinessSnapshot>,
        momentum: &SubjectMomentumProfile,
        exam_days_remaining: Option<i64>,
    ) -> EcoachResult<&'static str> {
        let (mastery_score, speed_score, fragility_score): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT mastery_score, speed_score, fragility_score
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_case.topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0, 0));
        let due_review_load = readiness
            .map(|snapshot| snapshot.due_review_count + snapshot.due_memory_count)
            .unwrap_or(0);
        let exam_push = exam_days_remaining.map(|days| days <= 7).unwrap_or(false)
            || plan_day_phase == "final_revision";

        let activity = if momentum.is_recovery_mode() {
            if is_review_priority_case(topic_case) {
                "memory_reactivation"
            } else {
                "review"
            }
        } else if plan_day_phase == "review_day" {
            if is_review_priority_case(topic_case) {
                "memory_reactivation"
            } else {
                "review"
            }
        } else if due_review_load > 0 && is_review_priority_case(topic_case) {
            "memory_reactivation"
        } else if topic_case.active_blocker.is_some() {
            "repair"
        } else {
            match topic_case.primary_hypothesis_code.as_str() {
                "blocked_topic" => "repair",
                "conceptual_confusion" => "worked_example",
                "memory_decay" => "memory_reactivation",
                "pressure_collapse" => {
                    if exam_push || plan_day_phase == "performance" {
                        "pressure_conditioning"
                    } else {
                        "checkpoint"
                    }
                }
                "knowledge_gap" => {
                    if mastery_score < 3500 || plan_day_phase == "foundation" {
                        "learn"
                    } else if fragility_score > 6000 {
                        "repair"
                    } else {
                        "guided_practice"
                    }
                }
                "execution_drift" => "checkpoint",
                _ => match plan_day_phase {
                    "foundation" => {
                        if mastery_score < 5000 {
                            "learn"
                        } else {
                            "guided_practice"
                        }
                    }
                    "strengthening" => {
                        if fragility_score > 6000 {
                            "guided_practice"
                        } else {
                            "checkpoint"
                        }
                    }
                    "performance" => {
                        if exam_push || topic_case.pressure_collapse_index >= 5500 {
                            "pressure_conditioning"
                        } else if speed_score < 4500 {
                            "speed_drill"
                        } else {
                            "mixed_test"
                        }
                    }
                    "consolidation" => "review",
                    "final_revision" => {
                        if topic_case.mastery_score >= 6500 {
                            "mixed_test"
                        } else {
                            "checkpoint"
                        }
                    }
                    _ => "guided_practice",
                },
            }
        };
        let subject_id = self.load_subject_id_for_topic(topic_case.topic_id)?;
        let activity = match CanonicalIntelligenceStore::new(self.conn)
            .suggest_activity_override(student_id, subject_id, topic_case, activity, plan_day_phase)?
            .as_deref()
        {
            Some("memory_reactivation") => "memory_reactivation",
            Some("review") => "review",
            Some("checkpoint") => "checkpoint",
            Some("pressure_conditioning") => "pressure_conditioning",
            Some("speed_drill") => "speed_drill",
            Some("repair") => "repair",
            _ => activity,
        };

        Ok(activity)
    }

    fn load_mission_context(&self, mission_id: i64) -> EcoachResult<MissionContext> {
        self.conn
            .query_row(
                "SELECT plan_day_id, student_id, title, reason, subject_id, primary_topic_id, activity_type, steps_json
                 FROM coach_missions
                 WHERE id = ?1",
                [mission_id],
                |row| {
                    let steps_json: String = row.get(7)?;
                    Ok(MissionContext {
                        plan_day_id: row.get(0)?,
                        student_id: row.get(1)?,
                        title: row.get(2)?,
                        reason: row.get(3)?,
                        subject_id: row.get(4)?,
                        topic_id: row.get(5)?,
                        activity_type: row.get(6)?,
                        steps: serde_json::from_str::<Value>(&steps_json).map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_session_outcome(&self, session_id: Option<i64>) -> EcoachResult<SessionOutcome> {
        let Some(session_id) = session_id else {
            return Ok(SessionOutcome {
                attempt_count: 0,
                correct_count: 0,
                accuracy_score: None,
                timed_accuracy: None,
                avg_latency_ms: None,
                misconception_tags: Vec::new(),
            });
        };

        let (attempt_count, correct_count, accuracy_score, avg_latency_ms, is_timed): (
            i64,
            i64,
            Option<BasisPoints>,
            Option<i64>,
            bool,
        ) = self
            .conn
            .query_row(
                "SELECT answered_questions, correct_questions, accuracy_score, avg_response_time_ms, is_timed
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get::<_, i64>(4)? == 1,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT COALESCE(mp.title, sqa.error_type)
                 FROM student_question_attempts sqa
                 LEFT JOIN misconception_patterns mp ON mp.id = sqa.misconception_triggered_id
                 WHERE sqa.session_id = ?1
                   AND COALESCE(mp.title, sqa.error_type) IS NOT NULL
                 ORDER BY COALESCE(mp.title, sqa.error_type) ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([session_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut misconception_tags = Vec::new();
        for row in rows {
            misconception_tags.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(SessionOutcome {
            attempt_count,
            correct_count,
            accuracy_score,
            timed_accuracy: if is_timed { accuracy_score } else { None },
            avg_latency_ms,
            misconception_tags,
        })
    }

    fn load_subject_id_for_topic(&self, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_subject_momentum_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<SubjectMomentumProfile> {
        self.conn
            .query_row(
                "SELECT current_stage, current_mode, momentum_score, strain_score,
                        recovery_need_score, streak_days
                 FROM beat_yesterday_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| {
                    Ok(SubjectMomentumProfile {
                        current_stage: row.get(0)?,
                        current_mode: row.get(1)?,
                        momentum_score: row.get(2)?,
                        strain_score: row.get(3)?,
                        recovery_need_score: row.get(4)?,
                        streak_days: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .map(|value| value.unwrap_or_default())
    }

    fn load_exam_days_remaining(&self, student_id: i64) -> EcoachResult<Option<i64>> {
        let exam_date = self
            .conn
            .query_row(
                "SELECT exam_date
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'stale')
                 ORDER BY id DESC
                 LIMIT 1",
                [student_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten();
        let Some(exam_date) = exam_date else {
            return Ok(None);
        };
        let exam_date = NaiveDate::parse_from_str(&exam_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        Ok(Some(
            (exam_date - Utc::now().date_naive()).num_days().max(0),
        ))
    }

    fn count_prior_coach_evidence(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<i64> {
        let Some(topic_id) = topic_id else {
            return Ok(0);
        };
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM coach_session_evidence WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn sync_coach_topic_profile(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        attempt_count: i64,
        misconception_count: i64,
        mission_status: &str,
    ) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };
        let (mastery_estimate, fragility_score, speed_score, updated_at): (
            i64,
            i64,
            i64,
            Option<String>,
        ) = self
            .conn
            .query_row(
                "SELECT mastery_score, fragility_score, speed_score, updated_at
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0, 0, None));

        self.conn
            .execute(
                "INSERT INTO coach_topic_profiles (
                    student_id, topic_id, mastery_estimate, fragility_score, speed_score,
                    misconception_recurrence, evidence_count, attempt_count, last_seen_at,
                    blocked_status, repair_priority, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, datetime('now'), ?8, ?9, COALESCE(?10, datetime('now')))
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    mastery_estimate = excluded.mastery_estimate,
                    fragility_score = excluded.fragility_score,
                    speed_score = excluded.speed_score,
                    misconception_recurrence = coach_topic_profiles.misconception_recurrence + excluded.misconception_recurrence,
                    evidence_count = coach_topic_profiles.evidence_count + 1,
                    attempt_count = coach_topic_profiles.attempt_count + excluded.attempt_count,
                    last_seen_at = datetime('now'),
                    blocked_status = excluded.blocked_status,
                    repair_priority = excluded.repair_priority,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    topic_id,
                    mastery_estimate,
                    fragility_score,
                    speed_score,
                    misconception_count,
                    attempt_count,
                    if mission_status == "repair_required" { 1 } else { 0 },
                    match mission_status {
                        "repair_required" => 9000,
                        "review_due" => 6500,
                        _ => 2500,
                    },
                    updated_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn update_blockers(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        mission_status: &str,
    ) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };

        if mission_status == "repair_required" {
            let updated = self
                .conn
                .execute(
                    "UPDATE coach_blockers
                     SET reason = 'repeated low mission accuracy',
                         severity = 'high'
                     WHERE student_id = ?1 AND topic_id = ?2 AND resolved_at IS NULL",
                    params![student_id, topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if updated == 0 {
                self.conn
                    .execute(
                        "INSERT INTO coach_blockers (student_id, topic_id, reason, severity)
                         VALUES (?1, ?2, 'repeated low mission accuracy', 'high')",
                        params![student_id, topic_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        } else {
            self.conn
                .execute(
                    "UPDATE coach_blockers
                     SET resolved_at = datetime('now')
                     WHERE student_id = ?1 AND topic_id = ?2 AND resolved_at IS NULL",
                    params![student_id, topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn append_runtime_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'coach', ?3, ?4, ?5, ?6)",
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
}

#[derive(Debug)]
struct MissionContext {
    plan_day_id: Option<i64>,
    student_id: i64,
    title: String,
    reason: String,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
    activity_type: String,
    steps: Value,
}

#[derive(Debug)]
struct PlanContext {
    plan_id: i64,
    exam_target: String,
    exam_date: String,
    daily_budget_minutes: i64,
}

#[derive(Debug)]
struct ActivePlanSnapshot {
    plan_id: i64,
    exam_target: String,
    exam_date: String,
    start_date: String,
    total_days: i64,
    daily_budget_minutes: i64,
    current_phase: String,
}

impl SubjectMomentumProfile {
    fn is_recovery_mode(&self) -> bool {
        self.current_mode == "recovery_mode"
            || self.strain_score >= 7000
            || self.recovery_need_score >= 7000
    }
}

fn activity_templates_for_phase(phase: &str) -> [(&'static str, i64, &'static str); 3] {
    match phase {
        "review_day" => [
            (
                "review",
                40,
                "Consolidate the most recent learning before it fades.",
            ),
            (
                "memory_reactivation",
                35,
                "Recover recall on the most fragile topic in scope.",
            ),
            (
                "checkpoint",
                25,
                "Verify that the review held under light pressure.",
            ),
        ],
        "foundation" => [
            (
                "learn",
                40,
                "Build first-pass understanding of the current topic.",
            ),
            (
                "guided_practice",
                40,
                "Turn the new explanation into correct worked execution.",
            ),
            ("review", 20, "Close with a short retrieval loop."),
        ],
        "strengthening" => [
            (
                "guided_practice",
                45,
                "Improve consistency on currently weak areas.",
            ),
            (
                "checkpoint",
                30,
                "Measure whether the topic is ready to advance.",
            ),
            ("review", 25, "Stabilize gains with short recall."),
        ],
        "performance" => [
            ("speed_drill", 35, "Convert accuracy into usable exam pace."),
            (
                "mixed_test",
                45,
                "Practice switching under exam-style pressure.",
            ),
            ("review", 20, "Repair slips before they compound."),
        ],
        "consolidation" => [
            ("review", 40, "Protect marks on fragile topics."),
            (
                "guided_practice",
                35,
                "Keep execution smooth on recently improved topics.",
            ),
            ("checkpoint", 25, "Confirm the topic remains stable."),
        ],
        "final_revision" => [
            ("mixed_test", 50, "Run an exam-like mixed practice block."),
            ("speed_drill", 25, "Sharpen pace on high-yield work."),
            ("review", 25, "Protect weak marks before the exam."),
        ],
        _ => [
            ("guided_practice", 40, "Keep the current topic moving."),
            (
                "checkpoint",
                35,
                "Check whether the topic is ready to advance.",
            ),
            ("review", 25, "Close with short retrieval."),
        ],
    }
}

fn target_readiness_for_days_remaining(total_days: i64, days_remaining: i64) -> BasisPoints {
    if total_days <= 1 {
        return 8_500;
    }
    let elapsed_days = (total_days - days_remaining).max(0);
    let progress_bp = clamp_bp((elapsed_days * 10_000) / total_days.max(1));
    clamp_bp(4_200 + ((progress_bp as i64 * 4_300) / 10_000))
}

fn readiness_band_for_score(score: BasisPoints) -> &'static str {
    match score {
        0..=2999 => "critical",
        3000..=4999 => "fragile",
        5000..=6799 => "developing",
        6800..=8199 => "progressing",
        _ => "ready",
    }
}

fn phase_for_remaining_days(total_days: i64) -> &'static str {
    if total_days > 90 {
        "foundation"
    } else if total_days > 45 {
        "strengthening"
    } else if total_days > 21 {
        "performance"
    } else if total_days > 7 {
        "consolidation"
    } else {
        "final_revision"
    }
}

fn derive_mission_status(
    activity_type: &str,
    outcome: &SessionOutcome,
    prior_evidence_count: i64,
) -> &'static str {
    let effective_accuracy = outcome
        .timed_accuracy
        .or(outcome.accuracy_score)
        .unwrap_or(0);
    let misconception_overload = outcome.misconception_tags.len() >= 2;
    if outcome.attempt_count == 0 {
        return "partial";
    }

    match activity_type {
        "memory_reactivation" => {
            if effective_accuracy < 4500 && prior_evidence_count >= 1 {
                "repair_required"
            } else if effective_accuracy < 7800 || misconception_overload {
                "review_due"
            } else {
                "completed"
            }
        }
        "pressure_conditioning" | "mixed_test" | "speed_drill" | "checkpoint" => {
            if effective_accuracy < 4500 && prior_evidence_count >= 1 {
                "repair_required"
            } else if effective_accuracy < 7000 || misconception_overload {
                "review_due"
            } else {
                "completed"
            }
        }
        _ => {
            if effective_accuracy < 4000 && prior_evidence_count >= 1 {
                "repair_required"
            } else if effective_accuracy < 6500 || misconception_overload {
                "review_due"
            } else {
                "completed"
            }
        }
    }
}

fn derive_review_due_at(
    activity_type: &str,
    mission_status: &str,
    accuracy_score: Option<BasisPoints>,
) -> Option<String> {
    let today = Utc::now().date_naive();
    let offset_days = match (activity_type, mission_status, accuracy_score.unwrap_or(0)) {
        (_, "partial", _) => 1,
        (_, "repair_required", _) => 1,
        ("memory_reactivation", "completed", _) => 2,
        ("memory_reactivation", _, _) => 1,
        ("pressure_conditioning" | "mixed_test" | "speed_drill" | "checkpoint", "completed", _) => {
            3
        }
        ("pressure_conditioning" | "mixed_test" | "speed_drill" | "checkpoint", _, _) => 1,
        (_, "review_due", _) => 2,
        (_, _, score) if score < 7000 => 3,
        _ => 5,
    };
    Some(
        today
            .checked_add_days(Days::new(offset_days as u64))?
            .to_string(),
    )
}

fn derive_next_action_type(activity_type: &str, mission_status: &str) -> &'static str {
    match (activity_type, mission_status) {
        (_, "repair_required") => "start_repair",
        (_, "review_due") => "review_results",
        (_, "partial") => "resume_mission",
        ("pressure_conditioning" | "mixed_test" | "checkpoint", _) => "start_today_mission",
        _ => "start_today_mission",
    }
}

fn derive_strategy_effect(
    activity_type: &str,
    mission_status: &str,
    accuracy_score: Option<BasisPoints>,
    prior_evidence_count: i64,
) -> Option<String> {
    let effect = match (activity_type, mission_status) {
        (_, "repair_required") if prior_evidence_count >= 1 => "escalate_to_repair",
        ("memory_reactivation", "completed") => "schedule_memory_recheck",
        ("memory_reactivation", _) => "schedule_short_review",
        ("worked_example", "completed") => "shift_to_guided_practice",
        ("learn", "completed") => "unlock_guided_practice",
        ("checkpoint", "completed") => "advance_to_mixed_test",
        ("pressure_conditioning", "completed") => "unlock_exam_push",
        (_, "review_due") => "schedule_short_review",
        (_, "partial") => "collect_more_evidence",
        (_, _) if accuracy_score.unwrap_or(0) >= 8000 => "unlock_next_planned_step",
        _ => "stabilize_and_probe",
    };
    Some(effect.to_string())
}

fn is_priority_repair_case(topic_case: &TopicCase) -> bool {
    topic_case.active_blocker.is_some()
        || matches!(
            topic_case.primary_hypothesis_code.as_str(),
            "blocked_topic" | "conceptual_confusion" | "knowledge_gap"
        )
        || topic_case.recommended_intervention.next_action_type == "start_repair"
}

fn is_review_priority_case(topic_case: &TopicCase) -> bool {
    matches!(topic_case.primary_hypothesis_code.as_str(), "memory_decay")
        || matches!(
            topic_case.memory_state.as_str(),
            "fragile" | "at_risk" | "fading" | "rebuilding" | "collapsed"
        )
        || topic_case
            .proof_gaps
            .iter()
            .any(|gap| gap.contains("Delayed retrieval"))
}

fn supports_performance_push(topic_case: &TopicCase) -> bool {
    topic_case.mastery_score >= 6000
        && topic_case.gap_score <= 6500
        && topic_case.active_blocker.is_none()
}

fn adaptive_target_minutes(
    plan_target_minutes: i64,
    topic_case: &TopicCase,
    momentum: &SubjectMomentumProfile,
    activity_type: &str,
) -> i64 {
    let mut minutes = topic_case
        .recommended_intervention
        .recommended_minutes
        .clamp(10, plan_target_minutes.max(10));
    if momentum.is_recovery_mode() {
        minutes = minutes.min(15).max(10);
    } else if matches!(activity_type, "pressure_conditioning" | "mixed_test") {
        minutes = minutes.max(20);
    } else if matches!(activity_type, "memory_reactivation" | "review") {
        minutes = minutes.min(15).max(10);
    }
    minutes.min(plan_target_minutes.max(10))
}

fn mission_reason(
    plan_day_phase: &str,
    topic_case: &TopicCase,
    readiness: Option<&crate::readiness_engine::StudentReadinessSnapshot>,
    momentum: &SubjectMomentumProfile,
    exam_days_remaining: Option<i64>,
    activity_type: &str,
    journey_station: Option<&ActiveJourneyStation>,
) -> String {
    let mut reasons = vec![topic_case.recommended_intervention.reason.clone()];
    if plan_day_phase == "review_day" {
        reasons.push("Today's coach day is reserved for review and proof recovery.".to_string());
    }
    if momentum.is_recovery_mode() {
        reasons.push(
            "Recent strain is elevated, so the coach is protecting load instead of forcing volume."
                .to_string(),
        );
    }
    if let Some(days_remaining) = exam_days_remaining.filter(|days| *days <= 7) {
        reasons.push(format!(
            "Exam mode is active with {} day(s) remaining.",
            days_remaining
        ));
    }
    if let Some(snapshot) = readiness {
        if snapshot.due_review_count > 0
            && matches!(activity_type, "review" | "memory_reactivation")
        {
            reasons.push(format!(
                "{} mission review(s) are still pending in this subject.",
                snapshot.due_review_count
            ));
        }
        if snapshot.due_memory_count > 0 && activity_type == "memory_reactivation" {
            reasons.push(format!(
                "{} topic(s) are overdue for memory proof.",
                snapshot.due_memory_count
            ));
        }
    }
    if let Some(station) = journey_station {
        reasons.push(format!(
            "Journey is actively routing this topic through a {} station ({}).",
            station.station_type.replace('_', " "),
            station.station_code
        ));
    }
    reasons.join(" ")
}

fn mission_steps_for_activity(activity_type: &str, topic_case: &TopicCase) -> Vec<String> {
    let first_probe = topic_case
        .active_hypotheses
        .first()
        .and_then(|item| item.recommended_probe.clone());
    match activity_type {
        "learn" => vec![
            format!("Rebuild the core idea behind {}.", topic_case.topic_name),
            "Work through one guided example slowly.".to_string(),
            "Close with a short independent check.".to_string(),
        ],
        "worked_example" => vec![
            format!(
                "Explain why the correct idea for {} differs from the tempting wrong one.",
                topic_case.topic_name
            ),
            "Walk one worked example end to end.".to_string(),
            "Finish with one contrast check in the learner's own words.".to_string(),
        ],
        "memory_reactivation" => vec![
            format!(
                "Retrieve {} without rereading first.",
                topic_case.topic_name
            ),
            "Use a short cue ladder only if recall stalls.".to_string(),
            "End with one delayed recall promise for the next review.".to_string(),
        ],
        "pressure_conditioning" => vec![
            "Start with one calm win.".to_string(),
            "Repeat under a stricter clock.".to_string(),
            "Debrief what changed under pressure.".to_string(),
        ],
        "checkpoint" => vec![
            "Run a short independent checkpoint.".to_string(),
            "Mark the exact leak, not just the final score.".to_string(),
            "Decide whether the next step is review, repair, or mixed performance.".to_string(),
        ],
        "mixed_test" => vec![
            "Run a mixed performance set.".to_string(),
            "Track accuracy and pace together.".to_string(),
            "Escalate only if both stay stable.".to_string(),
        ],
        "review" => vec![
            "Revisit the last weak proof point.".to_string(),
            "Confirm the topic still holds after a gap.".to_string(),
            "Log whether the coach can safely move on.".to_string(),
        ],
        "speed_drill" => vec![
            "Run a short pace-focused burst.".to_string(),
            "Keep method quality visible while the clock is running.".to_string(),
            "Stop if speed collapses correctness.".to_string(),
        ],
        _ => {
            let mut steps = vec![
                format!("Repair the main weakness in {}.", topic_case.topic_name),
                "Run one focused correction set.".to_string(),
                "Close with one proof item to confirm the leak actually moved.".to_string(),
            ];
            if let Some(probe) = first_probe {
                steps.push(probe);
            }
            steps
        }
    }
}

fn target_accuracy_for_activity(activity_type: &str, topic_case: &TopicCase) -> BasisPoints {
    let base_target = match activity_type {
        "memory_reactivation" => 7800,
        "pressure_conditioning" | "mixed_test" | "checkpoint" => 7000,
        "review" => 7200,
        _ => 6500,
    };
    clamp_bp(base_target.max(topic_case.mastery_score as i64 + 800))
}

fn min_attempts_for_activity(activity_type: &str) -> i64 {
    match activity_type {
        "memory_reactivation" | "review" => 4,
        "pressure_conditioning" | "mixed_test" => 6,
        _ => 5,
    }
}

fn target_latency_for_activity(activity_type: &str) -> Option<i64> {
    match activity_type {
        "speed_drill" => Some(20_000),
        "pressure_conditioning" => Some(18_000),
        "mixed_test" => Some(25_000),
        _ => None,
    }
}

fn proof_goal_for_activity(activity_type: &str, topic_case: &TopicCase) -> String {
    match activity_type {
        "memory_reactivation" => format!(
            "Show that {} can still be retrieved after a gap.",
            topic_case.topic_name
        ),
        "pressure_conditioning" => format!(
            "Hold method quality on {} while the clock is active.",
            topic_case.topic_name
        ),
        "worked_example" => format!(
            "Explain the concept boundary inside {} without confusion.",
            topic_case.topic_name
        ),
        "checkpoint" => format!(
            "Produce one clean independent checkpoint on {}.",
            topic_case.topic_name
        ),
        _ => format!(
            "Move the main weakness in {} with proof, not just exposure.",
            topic_case.topic_name
        ),
    }
}

fn align_activity_type_with_station(
    default_activity: &'static str,
    station_type: &str,
    topic_case: &TopicCase,
    exam_days_remaining: Option<i64>,
) -> &'static str {
    match station_type {
        "review" => {
            if is_review_priority_case(topic_case) {
                "memory_reactivation"
            } else {
                "review"
            }
        }
        "foundation" => {
            if topic_case.mastery_score < 3500
                || topic_case.primary_hypothesis_code == "knowledge_gap"
            {
                "learn"
            } else {
                "repair"
            }
        }
        "repair" => {
            if topic_case.primary_hypothesis_code == "conceptual_confusion" {
                "worked_example"
            } else {
                "repair"
            }
        }
        "checkpoint" => "checkpoint",
        "performance" => {
            if topic_case.pressure_collapse_index >= 5500
                || exam_days_remaining.map(|days| days <= 7).unwrap_or(false)
            {
                "pressure_conditioning"
            } else if default_activity == "speed_drill" {
                "speed_drill"
            } else {
                "mixed_test"
            }
        }
        _ => default_activity,
    }
}

fn station_fit_score(station_type: &str, plan_day_phase: &str) -> i64 {
    match (station_type, plan_day_phase) {
        ("review", "review_day") => 100,
        ("performance" | "checkpoint", "performance" | "final_revision") => 90,
        ("foundation" | "repair", "foundation") => 85,
        ("checkpoint", "strengthening") => 75,
        ("repair", "strengthening") => 70,
        ("review", _) => 60,
        ("performance", _) => 55,
        _ => 40,
    }
}

fn plan_phase_for_station_type(station_type: &str) -> &'static str {
    match station_type {
        "review" => "review_day",
        "foundation" => "foundation",
        "repair" | "checkpoint" => "strengthening",
        "performance" => "performance",
        _ => "strengthening",
    }
}

fn carryover_minutes_for_station_retry(station_type: &str, mission_status: &str) -> i64 {
    match (station_type, mission_status) {
        ("review", _) => 20,
        ("performance", "review_due") => 20,
        ("performance", _) => 15,
        ("foundation" | "repair", _) => 25,
        (_, "partial") => 10,
        _ => 15,
    }
}

fn carryover_minutes_for_next_station(
    next_station_type: Option<&str>,
    mission_status: &str,
) -> i64 {
    match (next_station_type, mission_status) {
        (Some("review"), _) => 5,
        (Some("performance"), "completed") => 10,
        (Some("foundation" | "repair"), _) => 10,
        (_, "review_due") => 10,
        (_, _) => 0,
    }
}

fn mission_journey_binding(steps: &Value) -> Option<MissionJourneyBinding> {
    let journey = steps.get("journey")?;
    Some(MissionJourneyBinding {
        route_id: journey.get("route_id")?.as_i64()?,
        station_id: journey.get("station_id")?.as_i64()?,
        station_code: journey.get("station_code")?.as_str()?.to_string(),
        station_type: journey.get("station_type")?.as_str()?.to_string(),
    })
}

fn map_coach_mission_memory(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoachMissionMemory> {
    let misconception_json: String = row.get(12)?;
    let misconception_tags =
        serde_json::from_str::<Vec<String>>(&misconception_json).unwrap_or_default();

    Ok(CoachMissionMemory {
        id: row.get(0)?,
        mission_id: row.get(1)?,
        plan_day_id: row.get(2)?,
        student_id: row.get(3)?,
        session_id: row.get(4)?,
        subject_id: row.get(5)?,
        topic_id: row.get(6)?,
        mission_status: row.get(7)?,
        attempt_count: row.get(8)?,
        correct_count: row.get(9)?,
        accuracy_score: row.get(10)?,
        avg_latency_ms: row.get(11)?,
        misconception_tags,
        review_due_at: row.get(13)?,
        next_action_type: row.get(14)?,
        strategy_effect: row.get(15)?,
        summary_json: row.get(16)?,
        review_status: row.get(17)?,
    })
}

fn is_review_day(offset: i64) -> bool {
    offset > 0 && (offset + 1) % 7 == 0
}

fn mission_title_for_activity(activity_type: &str) -> &'static str {
    match activity_type {
        "learn" => "Foundation Learning Mission",
        "guided_practice" => "Guided Practice Mission",
        "worked_example" => "Concept Repair Mission",
        "speed_drill" => "Speed Conversion Mission",
        "mixed_test" => "Mixed Performance Mission",
        "review" => "Review and Recovery Mission",
        "memory_reactivation" => "Memory Reactivation Mission",
        "checkpoint" => "Checkpoint Mission",
        "pressure_conditioning" => "Pressure Conditioning Mission",
        _ => "Priority Repair Mission",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;
    use crate::{TopicCaseHypothesis, TopicCaseIntervention, journey::JourneyService};

    #[test]
    fn generate_plan_creates_multiday_schedule_and_today_mission() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student_id],
        )
        .expect("student profile should insert");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (student_id, topic_id, mastery_score, gap_score, priority_score, fragility_score, speed_score)
             VALUES (?1, ?2, 3000, 8500, 9000, 6500, 4000)",
            params![student_id, topic_id],
        )
        .expect("topic state should insert");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(14))
            .expect("future date should exist")
            .to_string();
        let plan_id = engine
            .generate_plan(student_id, "BECE", &exam_date, 60)
            .expect("plan should generate");
        let mission_id = engine
            .generate_today_mission(student_id)
            .expect("mission should generate");
        engine
            .start_mission(mission_id)
            .expect("mission should start cleanly");

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        let misconception_id: i64 = conn
            .query_row(
                "SELECT id FROM misconception_patterns ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("misconception should exist");
        let session_topic_ids = format!("[{}]", topic_id);
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (?1, 'coach_mission', ?2, ?3, 2, 2, 0, 'completed', datetime('now'), datetime('now'), 2, 1, 5000, 15000)",
            params![student_id, subject_id, session_topic_ids],
        )
        .expect("session should insert");
        let session_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, session_id, session_type, attempt_number, started_at, submitted_at,
                response_time_ms, is_correct, error_type, misconception_triggered_id
             ) VALUES (?1, ?2, ?3, 'coach_mission', 1, datetime('now'), datetime('now'), 18000, 0, 'misconception_triggered', ?4)",
            params![student_id, question_id, session_id, misconception_id],
        )
        .expect("attempt should insert");
        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, session_id, session_type, attempt_number, started_at, submitted_at,
                response_time_ms, is_correct
             ) VALUES (?1, ?2, ?3, 'coach_mission', 2, datetime('now'), datetime('now'), 12000, 1)",
            params![student_id, question_id, session_id],
        )
        .expect("second attempt should insert");
        let mission_memory = engine
            .complete_mission_from_session(mission_id, Some(session_id))
            .expect("mission memory should persist");

        let plan_day_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_plan_days WHERE plan_id = ?1",
                [plan_id],
                |row| row.get(0),
            )
            .expect("plan day count should query");
        let mission_plan_day_id: i64 = conn
            .query_row(
                "SELECT plan_day_id FROM coach_missions WHERE id = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .expect("mission should belong to a plan day");
        let pending_review_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_mission_memories WHERE student_id = ?1 AND review_status = 'pending'",
                [student_id],
                |row| row.get(0),
            )
            .expect("mission review count should query");

        assert!(plan_day_count >= 14);
        assert!(mission_plan_day_id > 0);
        assert_eq!(mission_memory.review_status, "pending");
        assert_eq!(pending_review_count, 1);
    }

    #[test]
    fn complete_mission_rewrites_plan_after_repeated_failure() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Yaw', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student_id],
        )
        .expect("student profile should insert");
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, priority_score, fragility_score, speed_score
             ) VALUES (?1, ?2, 2800, 9100, 9600, 7200, 3800)",
            params![student_id, topic_id],
        )
        .expect("topic state should insert");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(18))
            .expect("future date should exist")
            .to_string();
        let original_plan_id = engine
            .generate_plan(student_id, "BECE", &exam_date, 60)
            .expect("plan should generate");
        let mission_id = engine
            .generate_today_mission(student_id)
            .expect("mission should generate");

        conn.execute(
            "INSERT INTO coach_session_evidence (
                mission_id, student_id, subject_id, topic_id, activity_type, attempt_count,
                correct_count, accuracy, avg_latency_ms, misconception_tags, timed_accuracy
             ) VALUES (NULL, ?1, ?2, ?3, 'repair', 6, 2, 3300, 18000, '[]', 3300)",
            params![student_id, subject_id, topic_id],
        )
        .expect("prior coach evidence should insert");
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (?1, 'coach_mission', ?2, ?3, 4, 4, 0, 'completed', datetime('now'), datetime('now'), 4, 1, 2500, 19000)",
            params![student_id, subject_id, format!("[{}]", topic_id)],
        )
        .expect("session should insert");
        let session_id = conn.last_insert_rowid();

        engine
            .complete_mission_from_session(mission_id, Some(session_id))
            .expect("mission should complete and trigger replanning");

        let stale_plan_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_plans WHERE student_id = ?1 AND status = 'stale'",
                [student_id],
                |row| row.get(0),
            )
            .expect("stale plan count should query");
        let new_active_plan_id: i64 = conn
            .query_row(
                "SELECT id FROM coach_plans WHERE student_id = ?1 AND status = 'active' ORDER BY id DESC LIMIT 1",
                [student_id],
                |row| row.get(0),
            )
            .expect("new active plan should exist");
        let adjustment_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events
                 WHERE event_type = 'plan.adjusted_from_mission'
                   AND json_extract(payload_json, '$.mission_id') = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .expect("adjustment event should query");

        assert_eq!(stale_plan_count, 1);
        assert_ne!(new_active_plan_id, original_plan_id);
        assert_eq!(adjustment_event_count, 1);
    }

    #[test]
    fn mission_generation_and_completion_follow_active_journey_station() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Abena', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student_id],
        )
        .expect("student profile should insert");

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
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority, speed_score
             ) VALUES (?1, ?2, 8200, 1800, 2200, 9800, 1800, 7600, 1500, 5, 2200, 7200)",
            params![student_id, performance_topic_id],
        )
        .expect("performance topic state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score,
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority, speed_score
             ) VALUES (?1, ?2, 6100, 5100, 4300, 6200, 2200, 2600, 8400, 4, 2500, 4800)",
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
        assert_eq!(route.stations[0].topic_id, Some(review_topic_id));
        assert_eq!(route.stations[0].station_type, "review");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(14))
            .expect("future date should exist")
            .to_string();
        engine
            .generate_plan(student_id, "BECE", &exam_date, 60)
            .expect("plan should generate");
        let mission_id = engine
            .generate_today_mission(student_id)
            .expect("mission should generate");

        let (mission_topic_id, activity_type, steps_json): (i64, String, String) = conn
            .query_row(
                "SELECT primary_topic_id, activity_type, steps_json
                 FROM coach_missions
                 WHERE id = ?1",
                [mission_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("mission should query");
        let steps: Value =
            serde_json::from_str(&steps_json).expect("mission steps should deserialize");

        assert_eq!(mission_topic_id, review_topic_id);
        assert_eq!(activity_type, "memory_reactivation");
        assert_eq!(steps["journey"]["station_type"].as_str(), Some("review"));

        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (?1, 'coach_mission', ?2, ?3, 5, 5, 0, 'completed', datetime('now'), datetime('now'), 5, 5, 8600, 9000)",
            params![student_id, subject_id, format!("[{}]", review_topic_id)],
        )
        .expect("session should insert");
        let session_id = conn.last_insert_rowid();

        engine
            .complete_mission_from_session(mission_id, Some(session_id))
            .expect("mission should complete");

        let refreshed_route = JourneyService::new(&conn)
            .get_active_route(student_id, subject_id)
            .expect("active route should load")
            .expect("active route should remain");
        let active_station = refreshed_route
            .stations
            .iter()
            .find(|station| station.status == "active")
            .expect("next station should activate");
        let (next_plan_phase, next_plan_carryover): (String, i64) = conn
            .query_row(
                "SELECT phase, carryover_minutes
                 FROM coach_plan_days cpd
                 INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                 WHERE cp.student_id = ?1
                   AND cp.status = 'active'
                   AND cpd.date > date('now')
                 ORDER BY cpd.date ASC, cpd.id ASC
                 LIMIT 1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("future plan day should query");

        assert_eq!(
            refreshed_route.route.current_station_code.as_deref(),
            Some(active_station.station_code.as_str())
        );
        assert_eq!(
            next_plan_phase,
            plan_phase_for_station_type(&active_station.station_type)
        );
        assert!(next_plan_carryover > 0);
    }

    #[test]
    fn rewrite_active_plan_marks_old_plan_stale_and_creates_new_plan() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Kojo', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student_id],
        )
        .expect("student profile should insert");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (student_id, topic_id, mastery_score, gap_score, priority_score)
             VALUES (?1, ?2, 3500, 8200, 9100)",
            params![student_id, topic_id],
        )
        .expect("topic state should insert");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(21))
            .expect("future date should exist")
            .to_string();
        let original_plan_id = engine
            .generate_plan(student_id, "BECE", &exam_date, 50)
            .expect("original plan should generate");
        conn.execute(
            "INSERT INTO coach_blockers (student_id, topic_id, reason, severity)
             VALUES (?1, ?2, 'blocked', 'high')",
            params![student_id, topic_id],
        )
        .expect("blocker should insert");

        let result = engine
            .rewrite_active_plan(student_id, "readiness_drop")
            .expect("plan rewrite should succeed");

        let original_status: String = conn
            .query_row(
                "SELECT status FROM coach_plans WHERE id = ?1",
                [original_plan_id],
                |row| row.get(0),
            )
            .expect("original plan status should query");
        let new_status: String = conn
            .query_row(
                "SELECT status FROM coach_plans WHERE id = ?1",
                [result.new_plan_id],
                |row| row.get(0),
            )
            .expect("new plan status should query");

        assert_eq!(result.previous_plan_id, original_plan_id);
        assert_eq!(original_status, "stale");
        assert_eq!(new_status, "active");
        assert!(!result.carryover_topic_ids.is_empty());
    }

    #[test]
    fn generate_today_mission_prefers_canonical_timing_bundle() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Efua', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (1, '[\"MATH\"]', 45)",
            [],
        )
        .expect("student profile should insert");
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 1",
                [subject_id],
                |row| row.get(0),
            )
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score,
                fragility_score, pressure_collapse_index, decay_risk, memory_strength, speed_score
             ) VALUES (1, ?1, 6200, 'stable', 2800, 9500, 3100, 2200, 8200, 2600, 5200)",
            [topic_id],
        )
        .expect("topic state should insert");
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (1, ?1, 'fading', 2600, 8200, datetime('now', '-1 day'))",
            [topic_id],
        )
        .expect("memory state should insert");

        let topic_case = TopicCase {
            student_id: 1,
            topic_id,
            topic_name: "Canonical Review Topic".to_string(),
            subject_code: "MATH".to_string(),
            priority_score: 9500,
            mastery_score: 6200,
            mastery_state: "stable".to_string(),
            gap_score: 2800,
            fragility_score: 3100,
            pressure_collapse_index: 2200,
            memory_state: "fading".to_string(),
            memory_strength: 2600,
            decay_risk: 8200,
            evidence_count: 5,
            recent_attempt_count: 3,
            recent_accuracy: Some(7200),
            active_blocker: None,
            recent_diagnoses: Vec::new(),
            active_hypotheses: vec![TopicCaseHypothesis {
                code: "memory_decay".to_string(),
                label: "Memory Decay".to_string(),
                confidence_score: 7800,
                evidence_summary: "Recall is fading after delay.".to_string(),
                recommended_probe: Some("Run a delayed recall check.".to_string()),
                recommended_response: "Prioritize review before new learning.".to_string(),
            }],
            primary_hypothesis_code: "memory_decay".to_string(),
            diagnosis_certainty: 7800,
            requires_probe: false,
            recommended_intervention: TopicCaseIntervention {
                mode: "review".to_string(),
                urgency: "high".to_string(),
                next_action_type: "review".to_string(),
                recommended_minutes: 25,
                reason: "Memory is fading and needs recall repair.".to_string(),
            },
            proof_gaps: vec!["delayed_recall".to_string()],
            open_questions: Vec::new(),
        };
        let strategy_file_json = serde_json::to_string(&json!({
            "topic_case": topic_case,
        }))
        .expect("strategy file json should serialize");
        conn.execute(
            "INSERT INTO ic_topic_teaching (
                learner_id, subject_id, topic_id, decision_id, dominant_hypothesis,
                co_causes_json, teaching_mode, entry_point, mastery_state, false_mastery_score,
                bottleneck_concept_id, evidence_spine_json, proof_contract_json,
                strategy_file_json, delayed_recall_required, confidence_bundle_json,
                owner_engine_key, updated_at
             ) VALUES (
                1, ?1, ?2, 'topic-decision-1', 'memory_decay',
                '[]', 'review', 'review', 'stable', 2400,
                NULL, '{}', '{}',
                ?3, 1, '{\"priority_score\":9500,\"diagnosis_certainty\":7800}',
                'topic', datetime('now')
             )",
            params![subject_id, topic_id, strategy_file_json],
        )
        .expect("canonical topic teaching should insert");
        conn.execute(
            "INSERT INTO ic_timing_decisions (
                decision_id, learner_id, subject_id, topic_id, action_type, action_scope,
                scheduled_for, current_phase, rationale_json, source_engine, consumed,
                owner_engine_key, updated_at
             ) VALUES (
                'timing-1', 1, ?1, ?2, 'delayed_recall', 'topic',
                datetime('now'), 'review', '{}', 'timing', 0,
                'timing', datetime('now')
             )",
            params![subject_id, topic_id],
        )
        .expect("timing decision should insert");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(14))
            .expect("future date should exist")
            .to_string();
        engine
            .generate_plan(1, "BECE", &exam_date, 45)
            .expect("plan should generate");
        let mission_id = engine
            .generate_today_mission(1)
            .expect("mission should generate");

        let activity_type: String = conn
            .query_row(
                "SELECT activity_type FROM coach_missions WHERE id = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .expect("mission activity should query");

        assert_eq!(activity_type, "memory_reactivation");
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
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
