use chrono::{Duration, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{OptionalExtension, Row, params};
use serde::Serialize;
use serde_json::{Value, json};

use super::MemoryService;
use crate::models::{
    CompleteInterventionStepInput, InterventionPlanRecord, InterventionStep,
    KnowledgeStateTransitionRecord, KnowledgeUnitEdgeRecord, KnowledgeUnitRecord,
    MemoryAnalyticsHotspot, MemoryCohortAnalytics, MemoryEngineEventRecord,
    MemoryExplainability, MemoryKnowledgeStateDetail, PressureProfileRecord, RecallProfile,
    RecordMemoryEvidenceInput, RetestPlan, RetrievalAttemptRecord, ReviewScheduleItemRecord,
    StudentInterferenceEdge, StudentKnowledgeStateRecord, TopicKnowledgeMap,
};

#[derive(Debug, Clone, Default)]
struct DerivedFeatureBundle {
    recall_profile: RecallProfile,
    rolling_accuracy: BasisPoints,
    recent_accuracy: BasisPoints,
    latency_score: BasisPoints,
    support_dependency_score: BasisPoints,
    confidence_calibration_score: BasisPoints,
    pressure_gap_score: BasisPoints,
    interference_rate: BasisPoints,
    consistency_score: BasisPoints,
    cue_recovery_rate: BasisPoints,
    resilience_score: BasisPoints,
    state_confidence: BasisPoints,
    decay_risk_score: BasisPoints,
    interference_risk_score: BasisPoints,
    downstream_risk_score: BasisPoints,
    review_urgency_score: BasisPoints,
    primary_failure_mode: Option<String>,
    secondary_failure_mode: Option<String>,
    flags: Vec<String>,
    dominant_intruder_node_id: Option<i64>,
    attempt_count: i64,
    success_count: i64,
    failure_count: i64,
    last_attempt_at: Option<String>,
    last_success_at: Option<String>,
    last_free_recall_success_at: Option<String>,
    last_application_success_at: Option<String>,
    last_pressure_success_at: Option<String>,
    recommended_review_mode: String,
    recommended_review_reason: String,
    recommended_action: Option<String>,
}

impl<'a> MemoryService<'a> {
    pub(super) fn record_idea32_attempt(
        &self,
        input: &RecordMemoryEvidenceInput,
    ) -> EcoachResult<Option<(i64, i64)>> {
        let Some(knowledge_unit) = self.ensure_knowledge_unit(input.topic_id, input.node_id)? else {
            return Ok(None);
        };

        let expected_unit_id = match input.expected_node_id {
            Some(node_id) => self
                .ensure_knowledge_unit(input.topic_id, Some(node_id))?
                .map(|unit| unit.id),
            None => None,
        };
        let intruding_unit_id = match input.intruding_node_id {
            Some(node_id) => self
                .ensure_knowledge_unit(input.topic_id, Some(node_id))?
                .map(|unit| unit.id),
            None => None,
        };

        let now = Utc::now().to_rfc3339();
        let raw_score_bp = input
            .raw_score_bp
            .unwrap_or(if input.was_correct { 10_000 } else { 0 });
        let correctness = if raw_score_bp >= 8_500 || input.was_correct {
            "correct"
        } else if raw_score_bp >= 4_000 {
            "partially_correct"
        } else {
            "incorrect"
        };
        let format = input
            .format
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown");

        self.conn
            .execute(
                "INSERT INTO retrieval_attempts (
                    student_id, knowledge_unit_id, session_id, question_id, mode, format,
                    timed, time_limit_ms, response_time_ms, first_commit_time_ms,
                    correctness, raw_score_bp, confidence_self_report_bp, hints_used,
                    hint_strength_bp, options_visible, formula_bank_visible, answer_text,
                    expected_node_id, intruding_node_id, switched_answer,
                    guess_likelihood_bp, freeze_marker, hesitation_score_bp,
                    derived_tags_json, attempt_key, created_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17, ?18,
                    ?19, ?20, ?21,
                    ?22, ?23, ?24,
                    ?25, ?26, ?27
                 )",
                params![
                    input.student_id,
                    knowledge_unit.id,
                    input.session_id,
                    input.question_id,
                    input.recall_mode.as_str(),
                    format,
                    bool_to_i64(input.timed),
                    input.time_limit_ms,
                    input.response_time_ms,
                    input.first_commit_time_ms,
                    correctness,
                    raw_score_bp,
                    input.confidence_self_report_bp,
                    input.hints_used,
                    input.hint_strength_bp,
                    bool_to_i64(input.options_visible),
                    bool_to_i64(input.formula_bank_visible),
                    input.answer_text,
                    input.expected_node_id,
                    input.intruding_node_id,
                    bool_to_i64(input.switched_answer),
                    input.guess_likelihood_bp,
                    bool_to_i64(input.freeze_marker),
                    input.hesitation_score_bp,
                    to_json(&input.derived_tags)?,
                    input.attempt_key.clone(),
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let attempt_id = self.conn.last_insert_rowid();

        self.append_memory_engine_event(
            "retrieval_attempt_recorded",
            Some(input.student_id),
            Some(knowledge_unit.id),
            json!({
                "attempt_id": attempt_id,
                "mode": input.recall_mode.as_str(),
                "correctness": correctness,
                "raw_score_bp": raw_score_bp,
            }),
        )?;
        self.enqueue_memory_job(
            "recompute_knowledge_state",
            Some(input.student_id),
            Some(knowledge_unit.id),
            json!({ "attempt_id": attempt_id }),
            input
                .attempt_key
                .clone()
                .unwrap_or_else(|| format!("attempt-{}-recompute", attempt_id)),
        )?;

        if input.interference_detected || input.intruding_node_id.is_some() {
            self.upsert_student_interference_edge(
                input.student_id,
                input.node_id,
                input.intruding_node_id.or(input.expected_node_id),
                knowledge_unit.id,
                intruding_unit_id.or(expected_unit_id),
                raw_score_bp,
                input.timed,
                &input.derived_tags,
            )?;
        }

        if input.timed || matches!(input.recall_mode, crate::models::RecallMode::Pressure) {
            self.append_pressure_attempt_summary(
                input.student_id,
                knowledge_unit.id,
                attempt_id,
                raw_score_bp,
                input,
            )?;
        }

        Ok(Some((attempt_id, knowledge_unit.id)))
    }

    pub(super) fn sync_idea32_for_unit(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        node_id: Option<i64>,
        reason: &str,
    ) -> EcoachResult<Option<StudentKnowledgeStateRecord>> {
        let Some(knowledge_unit) = self.ensure_knowledge_unit(topic_id, node_id)? else {
            return Ok(None);
        };

        self.sync_topic_knowledge_graph(knowledge_unit.topic_id)?;

        let previous_state = self
            .load_student_knowledge_state(student_id, knowledge_unit.id)?
            .map(|detail| detail.state);
        let legacy_state = self
            .load_legacy_memory_state(student_id, topic_id, node_id)?
            .or_else(|| {
                previous_state.as_ref().map(|state| crate::models::MemoryStateRecord {
                    id: 0,
                    student_id,
                    topic_id: state.topic_id,
                    node_id: state.node_id,
                    memory_state: state.memory_state.clone(),
                    memory_strength: state.recall_profile.free_recall,
                    recall_fluency: state.latency_score,
                    decay_risk: state.decay_risk_score,
                    review_due_at: state.next_review_at.clone(),
                    last_recalled_at: state.last_attempt_at.clone(),
                    created_at: state.created_at.clone(),
                    updated_at: state.updated_at.clone(),
                })
            });
        let recent_attempts = self.list_recent_attempts(student_id, knowledge_unit.id, 24)?;
        let features =
            self.derive_feature_bundle(student_id, &knowledge_unit, &recent_attempts, legacy_state.as_ref())?;
        let next_state = self.build_student_knowledge_state(
            student_id,
            &knowledge_unit,
            &features,
            previous_state.as_ref(),
            legacy_state.as_ref(),
        );

        self.upsert_student_knowledge_state(&next_state)?;
        self.complete_memory_jobs(student_id, knowledge_unit.id, "recompute_knowledge_state")?;

        if let Some(previous) = previous_state.as_ref() {
            if previous.memory_state != next_state.memory_state {
                self.insert_knowledge_state_transition(
                    student_id,
                    knowledge_unit.id,
                    &previous.memory_state,
                    &next_state.memory_state,
                    reason,
                    &json!({
                        "rolling_accuracy_bp": features.rolling_accuracy,
                        "decay_risk_score": features.decay_risk_score,
                        "primary_failure_mode": features.primary_failure_mode,
                    }),
                )?;
                self.append_memory_engine_event(
                    if state_rank(&next_state.memory_state) > state_rank(&previous.memory_state) {
                        "knowledge_state_promoted"
                    } else {
                        "knowledge_state_demoted"
                    },
                    Some(student_id),
                    Some(knowledge_unit.id),
                    json!({
                        "from_state": previous.memory_state,
                        "to_state": next_state.memory_state,
                        "reason": reason,
                    }),
                )?;
            }

            if previous.decay_status != next_state.decay_status {
                match next_state.decay_status.as_str() {
                    "watchlist" => self.append_memory_engine_event(
                        "decay_watchlist_entered",
                        Some(student_id),
                        Some(knowledge_unit.id),
                        json!({ "reason": reason }),
                    )?,
                    "fragile" => self.append_memory_engine_event(
                        "decay_fragile_entered",
                        Some(student_id),
                        Some(knowledge_unit.id),
                        json!({ "reason": reason }),
                    )?,
                    "collapsed" => self.append_memory_engine_event(
                        "decay_collapsed_entered",
                        Some(student_id),
                        Some(knowledge_unit.id),
                        json!({ "reason": reason }),
                    )?,
                    _ => {}
                }
            }
        }

        self.sync_decay_profiles(student_id, &knowledge_unit, &next_state, &features, &recent_attempts)?;
        self.sync_pressure_profile(student_id, knowledge_unit.id, &recent_attempts)?;
        self.sync_review_schedule(student_id, knowledge_unit.id, &next_state, &features)?;
        self.sync_intervention_plan(student_id, knowledge_unit.id, &next_state, &features)?;

        self.append_memory_engine_event(
            "knowledge_state_recomputed",
            Some(student_id),
            Some(knowledge_unit.id),
            json!({
                "memory_state": next_state.memory_state,
                "decay_status": next_state.decay_status,
                "review_urgency_score": next_state.review_urgency_score,
                "reason": reason,
            }),
        )?;

        self.load_student_knowledge_state(student_id, knowledge_unit.id)
            .map(|detail| detail.map(|detail| detail.state))
    }

    pub(super) fn sync_review_completion(
        &self,
        student_id: i64,
        node_id: Option<i64>,
    ) -> EcoachResult<()> {
        let Some(knowledge_unit) = self.ensure_knowledge_unit_from_existing(student_id, node_id)? else {
            return Ok(());
        };

        self.conn
            .execute(
                "UPDATE review_schedule_items
                 SET status = 'done', updated_at = ?3
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2 AND status = 'scheduled'",
                params![student_id, knowledge_unit.id, Utc::now().to_rfc3339()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.complete_memory_jobs(student_id, knowledge_unit.id, "schedule_review")?;
        self.append_memory_engine_event(
            "review_item_completed",
            Some(student_id),
            Some(knowledge_unit.id),
            json!({ "node_id": node_id }),
        )?;
        Ok(())
    }

    pub fn get_memory_knowledge_state(
        &self,
        student_id: i64,
        node_id: i64,
    ) -> EcoachResult<MemoryKnowledgeStateDetail> {
        self.sync_idea32_for_unit(student_id, None, Some(node_id), "manual_query")?;
        let knowledge_unit = self
            .ensure_knowledge_unit(None, Some(node_id))?
            .ok_or_else(|| EcoachError::NotFound("knowledge unit not found".to_string()))?;
        self.load_student_knowledge_state(student_id, knowledge_unit.id)?
            .ok_or_else(|| EcoachError::NotFound("knowledge state not found".to_string()))
    }

    pub fn list_memory_review_schedule(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ReviewScheduleItemRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, knowledge_unit_id, due_at, urgency_score,
                        recommended_mode, reason, status, created_at, updated_at
                 FROM review_schedule_items
                 WHERE student_id = ?1
                 ORDER BY
                    CASE status WHEN 'scheduled' THEN 0 WHEN 'missed' THEN 1 ELSE 2 END,
                    urgency_score DESC,
                    due_at ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], map_review_schedule_item)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    pub fn list_active_interventions(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<InterventionPlanRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, knowledge_unit_id, family, reason,
                        primary_failure_mode, target_state, steps_json, retest_plan_json,
                        estimated_difficulty_bp, estimated_duration_min, priority_score,
                        status, completed_step_count, total_step_count, created_at, updated_at
                 FROM intervention_plans
                 WHERE student_id = ?1 AND status IN ('pending', 'active')
                 ORDER BY priority_score DESC, updated_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], map_intervention_plan)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    pub fn complete_intervention_step(
        &self,
        input: &CompleteInterventionStepInput,
    ) -> EcoachResult<InterventionPlanRecord> {
        let plan = self
            .load_intervention_plan(input.plan_id)?
            .ok_or_else(|| EcoachError::NotFound("intervention plan not found".to_string()))?;
        let mut steps = plan.steps.clone();
        let mut found = false;
        for step in &mut steps {
            if step.step_code == input.step_code {
                step.status = if input.successful {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                };
                found = true;
            }
        }
        if !found {
            steps.push(InterventionStep {
                step_code: input.step_code.clone(),
                prompt: input.outcome.clone(),
                status: if input.successful {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
            });
        }

        self.conn
            .execute(
                "INSERT INTO intervention_step_events (plan_id, step_code, outcome, successful)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    input.plan_id,
                    input.step_code,
                    input.outcome,
                    bool_to_i64(input.successful),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let completed_steps = steps
            .iter()
            .filter(|step| step.status == "completed")
            .count() as i64;
        let failed_steps = steps.iter().filter(|step| step.status == "failed").count() as i64;
        let next_status = if completed_steps as usize >= steps.len() && input.successful {
            "completed"
        } else if failed_steps >= 2 {
            "abandoned"
        } else {
            "active"
        };

        self.conn
            .execute(
                "UPDATE intervention_plans
                 SET steps_json = ?1,
                     status = ?2,
                     completed_step_count = ?3,
                     total_step_count = ?4,
                     updated_at = ?5
                 WHERE id = ?6",
                params![
                    to_json(&steps)?,
                    next_status,
                    completed_steps,
                    steps.len() as i64,
                    Utc::now().to_rfc3339(),
                    input.plan_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO intervention_outcomes (
                    plan_id, student_id, knowledge_unit_id, outcome, success_flag
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    input.plan_id,
                    plan.student_id,
                    plan.knowledge_unit_id,
                    input.outcome,
                    bool_to_i64(input.successful),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO intervention_history_rollups (
                    student_id, knowledge_unit_id, total_plans, successful_plans,
                    failed_plans, last_outcome, updated_at
                 ) VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6)
                 ON CONFLICT(student_id, knowledge_unit_id) DO UPDATE SET
                    total_plans = intervention_history_rollups.total_plans + 1,
                    successful_plans = intervention_history_rollups.successful_plans + excluded.successful_plans,
                    failed_plans = intervention_history_rollups.failed_plans + excluded.failed_plans,
                    last_outcome = excluded.last_outcome,
                    updated_at = excluded.updated_at",
                params![
                    plan.student_id,
                    plan.knowledge_unit_id,
                    if input.successful { 1 } else { 0 },
                    if input.successful { 0 } else { 1 },
                    input.outcome,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if next_status == "completed" {
            let due_at = Utc::now() + Duration::hours(plan.retest_plan.review_after_hours.max(6));
            self.upsert_review_schedule_item(
                plan.student_id,
                plan.knowledge_unit_id,
                &due_at.to_rfc3339(),
                6_000,
                &plan.retest_plan.recommended_mode,
                "recent_recovery",
            )?;
            self.conn
                .execute(
                    "UPDATE student_knowledge_states
                     SET current_intervention_plan_id = NULL, updated_at = ?3
                     WHERE student_id = ?1 AND knowledge_unit_id = ?2",
                    params![plan.student_id, plan.knowledge_unit_id, Utc::now().to_rfc3339()],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.append_memory_engine_event(
                "intervention_completed",
                Some(plan.student_id),
                Some(plan.knowledge_unit_id),
                json!({ "plan_id": plan.id, "step_code": input.step_code }),
            )?;
        } else if next_status == "abandoned" {
            self.append_memory_engine_event(
                "intervention_escalation_required",
                Some(plan.student_id),
                Some(plan.knowledge_unit_id),
                json!({ "plan_id": plan.id, "step_code": input.step_code }),
            )?;
            self.enqueue_memory_job(
                "plan_intervention",
                Some(plan.student_id),
                Some(plan.knowledge_unit_id),
                json!({ "plan_id": plan.id, "reason": "step_failure_threshold" }),
                format!("plan-{}-escalate", plan.id),
            )?;
        }

        self.force_recompute_knowledge_state(plan.student_id, plan.knowledge_unit_id)
            .and_then(|detail| {
                detail.active_intervention.ok_or_else(|| {
                    if next_status == "completed" {
                        EcoachError::NotFound("intervention completed".to_string())
                    } else {
                        EcoachError::NotFound("active intervention missing".to_string())
                    }
                })
            })
            .or_else(|_| {
                self.load_intervention_plan(input.plan_id)?.ok_or_else(|| {
                    EcoachError::NotFound("intervention plan not found after update".to_string())
                })
            })
    }

    pub fn force_recompute_knowledge_state(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
    ) -> EcoachResult<MemoryKnowledgeStateDetail> {
        let knowledge_unit = self
            .load_knowledge_unit(knowledge_unit_id)?
            .ok_or_else(|| EcoachError::NotFound("knowledge unit not found".to_string()))?;
        self.enqueue_memory_job(
            "recompute_knowledge_state",
            Some(student_id),
            Some(knowledge_unit_id),
            json!({ "reason": "manual_force_recompute" }),
            format!("manual-{}-{}", student_id, knowledge_unit_id),
        )?;
        self.sync_idea32_for_unit(
            student_id,
            knowledge_unit.topic_id,
            knowledge_unit.node_id,
            "manual_force_recompute",
        )?;
        self.load_student_knowledge_state(student_id, knowledge_unit_id)?
            .ok_or_else(|| EcoachError::NotFound("knowledge state not found".to_string()))
    }

    pub fn get_memory_cohort_analytics(
        &self,
        topic_id: i64,
        hotspot_limit: usize,
    ) -> EcoachResult<MemoryCohortAnalytics> {
        self.sync_topic_knowledge_graph(Some(topic_id))?;

        let student_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT student_id)
                 FROM student_knowledge_states
                 WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let knowledge_unit_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_units WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let fragile_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM student_knowledge_states
                 WHERE topic_id = ?1 AND decay_status = 'fragile'",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let decaying_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM student_knowledge_states
                 WHERE topic_id = ?1 AND decay_status = 'decaying'",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let collapsed_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM student_knowledge_states
                 WHERE topic_id = ?1 AND decay_status = 'collapsed'",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let due_reviews: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM review_schedule_items rsi
                 INNER JOIN knowledge_units ku ON ku.id = rsi.knowledge_unit_id
                 WHERE ku.topic_id = ?1 AND rsi.status = 'scheduled'",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let active_interventions: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM intervention_plans ip
                 INNER JOIN knowledge_units ku ON ku.id = ip.knowledge_unit_id
                 WHERE ku.topic_id = ?1 AND ip.status IN ('pending', 'active')",
                [topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let average_decay_risk: BasisPoints = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(decay_risk_score), 0)
                 FROM student_knowledge_states
                 WHERE topic_id = ?1",
                [topic_id],
                |row| row.get::<_, i64>(0),
            )
            .map(clamp_bp)
            .unwrap_or(0);
        let average_pressure_gap: BasisPoints = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(pp.pressure_gap_score), 0)
                 FROM pressure_profiles pp
                 INNER JOIN knowledge_units ku ON ku.id = pp.knowledge_unit_id
                 WHERE ku.topic_id = ?1",
                [topic_id],
                |row| row.get::<_, i64>(0),
            )
            .map(clamp_bp)
            .unwrap_or(0);

        let mut failure_statement = self
            .conn
            .prepare(
                "SELECT primary_failure_mode, COUNT(*)
                 FROM student_knowledge_states
                 WHERE topic_id = ?1 AND primary_failure_mode IS NOT NULL
                 GROUP BY primary_failure_mode
                 ORDER BY COUNT(*) DESC, primary_failure_mode ASC
                 LIMIT 5",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let failure_rows = failure_statement
            .query_map([topic_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut top_failure_modes = Vec::new();
        for row in failure_rows {
            top_failure_modes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut hotspot_statement = self
            .conn
            .prepare(
                "SELECT ku.id, ku.node_id, ku.title,
                        SUM(CASE WHEN sks.decay_status = 'fragile' THEN 1 ELSE 0 END) AS fragile_count,
                        SUM(CASE WHEN sks.decay_status = 'collapsed' THEN 1 ELSE 0 END) AS collapsed_count,
                        (
                            SELECT COUNT(*)
                            FROM intervention_plans ip
                            WHERE ip.knowledge_unit_id = ku.id AND ip.status IN ('pending', 'active')
                        ) AS active_interventions,
                        (
                            SELECT COUNT(*)
                            FROM review_schedule_items rsi
                            WHERE rsi.knowledge_unit_id = ku.id AND rsi.status = 'scheduled'
                        ) AS due_reviews,
                        COALESCE(AVG(sks.decay_risk_score), 0) AS average_decay_risk
                 FROM knowledge_units ku
                 LEFT JOIN student_knowledge_states sks ON sks.knowledge_unit_id = ku.id
                 WHERE ku.topic_id = ?1
                 GROUP BY ku.id, ku.node_id, ku.title
                 ORDER BY average_decay_risk DESC, collapsed_count DESC, fragile_count DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let hotspot_rows = hotspot_statement
            .query_map(params![topic_id, hotspot_limit as i64], |row| {
                Ok(MemoryAnalyticsHotspot {
                    knowledge_unit_id: row.get(0)?,
                    node_id: row.get(1)?,
                    title: row.get(2)?,
                    fragile_count: row.get(3)?,
                    collapsed_count: row.get(4)?,
                    active_interventions: row.get(5)?,
                    due_reviews: row.get(6)?,
                    average_decay_risk: clamp_bp(row.get::<_, i64>(7)?),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut hotspots = Vec::new();
        for row in hotspot_rows {
            hotspots.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(MemoryCohortAnalytics {
            topic_id,
            student_count,
            knowledge_unit_count,
            fragile_count,
            decaying_count,
            collapsed_count,
            due_reviews,
            active_interventions,
            average_decay_risk,
            average_pressure_gap,
            top_failure_modes,
            hotspots,
        })
    }

    pub fn list_student_interference_edges(
        &self,
        student_id: i64,
        node_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<StudentInterferenceEdge>> {
        let Some(knowledge_unit) = self.ensure_knowledge_unit(None, Some(node_id))? else {
            return Ok(Vec::new());
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, from_node_id, to_node_id, source_knowledge_unit_id,
                        target_knowledge_unit_id, confusion_strength, directionality,
                        timed_confusion_strength, calm_confusion_strength, total_confusions,
                        context_tags_json, status, last_seen_at, updated_at
                 FROM interference_edges
                 WHERE student_id = ?1
                   AND (source_knowledge_unit_id = ?2 OR target_knowledge_unit_id = ?2)
                 ORDER BY confusion_strength DESC, updated_at DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, knowledge_unit.id, limit as i64], map_interference_edge)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        if edges.is_empty() {
            return self.get_interference_edges(node_id).map(|legacy| {
                legacy
                    .into_iter()
                    .map(|edge| StudentInterferenceEdge {
                        id: edge.id,
                        student_id: None,
                        from_node_id: edge.from_node_id,
                        to_node_id: edge.to_node_id,
                        source_knowledge_unit_id: None,
                        target_knowledge_unit_id: None,
                        confusion_strength: edge.strength_score,
                        directionality: "source_to_target".to_string(),
                        timed_confusion_strength: 0,
                        calm_confusion_strength: edge.strength_score,
                        total_confusions: 0,
                        context_tags: Vec::new(),
                        status: "watchlist".to_string(),
                        last_confusion_at: edge.last_seen_at.clone(),
                        updated_at: edge.last_seen_at,
                    })
                    .collect()
            });
        }
        Ok(edges)
    }

    pub fn get_topic_knowledge_map(&self, topic_id: i64) -> EcoachResult<TopicKnowledgeMap> {
        self.sync_topic_knowledge_graph(Some(topic_id))?;
        let units = self.list_knowledge_units_for_topic(topic_id)?;
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, source_unit_id, target_unit_id, edge_type, weight_bp, created_at
                 FROM knowledge_unit_edges
                 WHERE source_unit_id IN (
                    SELECT id FROM knowledge_units WHERE topic_id = ?1
                 )
                    OR target_unit_id IN (
                        SELECT id FROM knowledge_units WHERE topic_id = ?1
                 )
                 ORDER BY edge_type ASC, weight_bp DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], map_knowledge_unit_edge)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(TopicKnowledgeMap {
            topic_id,
            units,
            edges,
        })
    }

    fn derive_feature_bundle(
        &self,
        student_id: i64,
        knowledge_unit: &KnowledgeUnitRecord,
        attempts: &[RetrievalAttemptRecord],
        legacy_state: Option<&crate::models::MemoryStateRecord>,
    ) -> EcoachResult<DerivedFeatureBundle> {
        let mut bundle = DerivedFeatureBundle::default();
        if attempts.is_empty() {
            bundle.review_urgency_score = legacy_state.map(|state| state.decay_risk).unwrap_or(0);
            bundle.decay_risk_score = legacy_state.map(|state| state.decay_risk).unwrap_or(0);
            bundle.recommended_review_mode = "free_recall".to_string();
            bundle.recommended_review_reason = "decay_risk".to_string();
            return Ok(bundle);
        }

        let recent_slice = attempts.iter().take(5).collect::<Vec<_>>();
        let raw_average = average_bp(attempts.iter().map(|attempt| attempt.raw_score_bp as i64));
        let recent_average =
            average_bp(recent_slice.iter().map(|attempt| attempt.raw_score_bp as i64));
        let recognition = mode_average(attempts, "recognition");
        let cued = mode_average(attempts, "cued_recall");
        let free = mode_average(attempts, "free_recall");
        let application = mode_average(attempts, "application");
        let transfer = mode_average(attempts, "transfer");
        let pressure = mode_average(attempts, "pressure");
        let pressure_gap =
            clamp_bp((recognition.max(free.max(application)) as i64) - pressure as i64);

        let latency_score = average_bp(attempts.iter().filter_map(|attempt| {
            attempt
                .response_time_ms
                .map(|response_time| normalized_latency_score(response_time, attempt.time_limit_ms))
        }));
        let support_dependency = average_bp(attempts.iter().map(|attempt| {
            support_dependency_for_attempt(
                attempt.hints_used,
                attempt.hint_strength_bp,
                attempt.options_visible,
                attempt.formula_bank_visible,
            ) as i64
        }));
        let confidence_calibration = average_bp(attempts.iter().map(|attempt| {
            confidence_alignment(attempt.confidence_self_report_bp, attempt.raw_score_bp) as i64
        }));
        let interference_rate = average_bp(attempts.iter().map(|attempt| {
            if attempt.intruding_node_id.is_some() {
                10_000
            } else {
                0
            }
        }));
        let cue_recovery_rate = compute_cue_recovery_rate(attempts);
        let consistency_score = compute_consistency_score(attempts);
        let regression_count_14d =
            self.count_recent_regressions(student_id, knowledge_unit.id, 14_i64)?;
        let downstream_risk =
            self.compute_downstream_risk(knowledge_unit.id, transfer, application)?;
        let resilience_score = clamp_bp(
            ((free as i64 + application as i64 + transfer as i64 + (10_000 - pressure_gap) as i64)
                / 4)
                .max(0),
        );
        let state_confidence = clamp_bp(
            (attempts.len() as i64 * 1_100)
                + (consistency_score as i64 / 2)
                + (confidence_calibration as i64 / 3),
        );

        bundle.recall_profile = RecallProfile {
            recognition,
            cued_recall: cued,
            free_recall: free,
            application,
            transfer,
            pressure,
        };
        bundle.rolling_accuracy = raw_average;
        bundle.recent_accuracy = recent_average;
        bundle.latency_score = latency_score;
        bundle.support_dependency_score = support_dependency;
        bundle.confidence_calibration_score = confidence_calibration;
        bundle.pressure_gap_score = pressure_gap;
        bundle.interference_rate = interference_rate;
        bundle.consistency_score = consistency_score;
        bundle.cue_recovery_rate = cue_recovery_rate;
        bundle.resilience_score = resilience_score;
        bundle.state_confidence = state_confidence;
        bundle.attempt_count = attempts.len() as i64;
        bundle.success_count = attempts
            .iter()
            .filter(|attempt| attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000)
            .count() as i64;
        bundle.failure_count = attempts
            .iter()
            .filter(|attempt| attempt.correctness == "incorrect" || attempt.raw_score_bp < 4_000)
            .count() as i64;
        bundle.last_attempt_at = attempts.first().map(|attempt| attempt.created_at.clone());
        bundle.last_success_at = attempts
            .iter()
            .find(|attempt| attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000)
            .map(|attempt| attempt.created_at.clone());
        bundle.last_free_recall_success_at = attempts
            .iter()
            .find(|attempt| {
                attempt.mode == "free_recall"
                    && (attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000)
            })
            .map(|attempt| attempt.created_at.clone());
        bundle.last_application_success_at = attempts
            .iter()
            .find(|attempt| {
                attempt.mode == "application"
                    && (attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000)
            })
            .map(|attempt| attempt.created_at.clone());
        bundle.last_pressure_success_at = attempts
            .iter()
            .find(|attempt| {
                attempt.mode == "pressure"
                    && (attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000)
            })
            .map(|attempt| attempt.created_at.clone());
        bundle.dominant_intruder_node_id = dominant_intruder(attempts);
        bundle.interference_risk_score = clamp_bp(
            (interference_rate as i64 / 2)
                + (pressure_gap as i64 / 5)
                + (regression_count_14d * 800),
        );
        bundle.downstream_risk_score = downstream_risk;
        bundle.decay_risk_score = self.compute_knowledge_decay_risk(
            legacy_state,
            raw_average,
            recent_average,
            interference_rate,
            regression_count_14d,
        );

        let failure_modes = derive_failure_modes(
            &bundle.recall_profile,
            support_dependency,
            confidence_calibration,
            pressure_gap,
            interference_rate,
            raw_average,
            cue_recovery_rate,
        );
        bundle.primary_failure_mode = failure_modes.first().cloned();
        bundle.secondary_failure_mode = failure_modes.get(1).cloned();
        bundle.flags = derive_flags(
            bundle.decay_risk_score,
            pressure_gap,
            bundle.interference_risk_score,
            regression_count_14d,
        );
        bundle.review_urgency_score = clamp_bp(
            (bundle.decay_risk_score as i64 / 2)
                + (bundle.interference_risk_score as i64 / 5)
                + (downstream_risk as i64 / 4)
                + (pressure_gap as i64 / 4)
                + (knowledge_unit.exam_frequency_weight_bp as i64 / 5),
        );
        let (recommended_mode, recommended_reason) = recommended_review_action(&bundle);
        bundle.recommended_review_mode = recommended_mode.to_string();
        bundle.recommended_review_reason = recommended_reason.to_string();
        bundle.recommended_action = recommended_action_text(&bundle);
        Ok(bundle)
    }

    fn build_student_knowledge_state(
        &self,
        student_id: i64,
        knowledge_unit: &KnowledgeUnitRecord,
        features: &DerivedFeatureBundle,
        previous_state: Option<&StudentKnowledgeStateRecord>,
        legacy_state: Option<&crate::models::MemoryStateRecord>,
    ) -> StudentKnowledgeStateRecord {
        let now = Utc::now().to_rfc3339();
        let memory_state = resolve_knowledge_state(features, legacy_state);
        let decay_status = resolve_decay_status(features.decay_risk_score, &memory_state);
        let next_review_at = legacy_state
            .and_then(|state| state.review_due_at.clone())
            .or_else(|| {
                Some(
                    (Utc::now()
                        + Duration::hours(review_offset_hours(features.review_urgency_score)))
                    .to_rfc3339(),
                )
            });
        let explainability = MemoryExplainability {
            primary_driver: explainability_primary_driver(features),
            secondary_driver: features.secondary_failure_mode.clone(),
            feature_summary_json: json!({
                "rolling_accuracy_bp": features.rolling_accuracy,
                "recent_accuracy_bp": features.recent_accuracy,
                "pressure_gap_score": features.pressure_gap_score,
                "interference_rate": features.interference_rate,
                "support_dependency_score": features.support_dependency_score,
                "review_urgency_score": features.review_urgency_score,
            })
            .to_string(),
            recommended_action: features.recommended_action.clone(),
        };
        StudentKnowledgeStateRecord {
            student_id,
            knowledge_unit_id: knowledge_unit.id,
            node_id: knowledge_unit.node_id,
            topic_id: knowledge_unit.topic_id,
            memory_state,
            state_confidence_bp: features.state_confidence,
            state_updated_at: now.clone(),
            decay_status,
            decay_risk_score: features.decay_risk_score,
            recall_profile: features.recall_profile.clone(),
            support_dependency_score: features.support_dependency_score,
            confidence_calibration_score: features.confidence_calibration_score,
            latency_score: features.latency_score,
            resilience_score: features.resilience_score,
            primary_failure_mode: features.primary_failure_mode.clone(),
            secondary_failure_mode: features.secondary_failure_mode.clone(),
            interference_risk_score: features.interference_risk_score,
            downstream_risk_score: features.downstream_risk_score,
            exposure_count: features.attempt_count,
            attempt_count: features.attempt_count,
            success_count: features.success_count,
            failure_count: features.failure_count,
            last_seen_at: features.last_attempt_at.clone(),
            last_attempt_at: features.last_attempt_at.clone(),
            last_success_at: features.last_success_at.clone(),
            last_free_recall_success_at: features.last_free_recall_success_at.clone(),
            last_application_success_at: features.last_application_success_at.clone(),
            last_pressure_success_at: features.last_pressure_success_at.clone(),
            current_intervention_plan_id: previous_state
                .and_then(|state| state.current_intervention_plan_id),
            next_review_at,
            review_urgency_score: Some(features.review_urgency_score),
            flags: features.flags.clone(),
            explainability,
            created_at: previous_state
                .map(|state| state.created_at.clone())
                .unwrap_or(now.clone()),
            updated_at: now,
        }
    }

    fn upsert_student_knowledge_state(
        &self,
        state: &StudentKnowledgeStateRecord,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO student_knowledge_states (
                    student_id, knowledge_unit_id, node_id, topic_id, memory_state,
                    state_confidence_bp, state_updated_at, decay_status, decay_risk_score,
                    recall_profile_json, support_dependency_score, confidence_calibration_score,
                    latency_score, resilience_score, primary_failure_mode,
                    secondary_failure_mode, interference_risk_score, downstream_risk_score,
                    exposure_count, attempt_count, success_count, failure_count, last_seen_at,
                    last_attempt_at, last_success_at, last_free_recall_success_at,
                    last_application_success_at, last_pressure_success_at,
                    current_intervention_plan_id, next_review_at, review_urgency_score,
                    flags_json, explanation_json, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9,
                    ?10, ?11, ?12,
                    ?13, ?14, ?15,
                    ?16, ?17, ?18,
                    ?19, ?20, ?21, ?22, ?23,
                    ?24, ?25, ?26,
                    ?27, ?28,
                    ?29, ?30, ?31,
                    ?32, ?33, ?34, ?35
                 )
                 ON CONFLICT(student_id, knowledge_unit_id) DO UPDATE SET
                    node_id = excluded.node_id,
                    topic_id = excluded.topic_id,
                    memory_state = excluded.memory_state,
                    state_confidence_bp = excluded.state_confidence_bp,
                    state_updated_at = excluded.state_updated_at,
                    decay_status = excluded.decay_status,
                    decay_risk_score = excluded.decay_risk_score,
                    recall_profile_json = excluded.recall_profile_json,
                    support_dependency_score = excluded.support_dependency_score,
                    confidence_calibration_score = excluded.confidence_calibration_score,
                    latency_score = excluded.latency_score,
                    resilience_score = excluded.resilience_score,
                    primary_failure_mode = excluded.primary_failure_mode,
                    secondary_failure_mode = excluded.secondary_failure_mode,
                    interference_risk_score = excluded.interference_risk_score,
                    downstream_risk_score = excluded.downstream_risk_score,
                    exposure_count = excluded.exposure_count,
                    attempt_count = excluded.attempt_count,
                    success_count = excluded.success_count,
                    failure_count = excluded.failure_count,
                    last_seen_at = excluded.last_seen_at,
                    last_attempt_at = excluded.last_attempt_at,
                    last_success_at = excluded.last_success_at,
                    last_free_recall_success_at = excluded.last_free_recall_success_at,
                    last_application_success_at = excluded.last_application_success_at,
                    last_pressure_success_at = excluded.last_pressure_success_at,
                    current_intervention_plan_id = excluded.current_intervention_plan_id,
                    next_review_at = excluded.next_review_at,
                    review_urgency_score = excluded.review_urgency_score,
                    flags_json = excluded.flags_json,
                    explanation_json = excluded.explanation_json,
                    updated_at = excluded.updated_at",
                params![
                    state.student_id,
                    state.knowledge_unit_id,
                    state.node_id,
                    state.topic_id,
                    state.memory_state,
                    state.state_confidence_bp,
                    state.state_updated_at,
                    state.decay_status,
                    state.decay_risk_score,
                    to_json(&state.recall_profile)?,
                    state.support_dependency_score,
                    state.confidence_calibration_score,
                    state.latency_score,
                    state.resilience_score,
                    state.primary_failure_mode,
                    state.secondary_failure_mode,
                    state.interference_risk_score,
                    state.downstream_risk_score,
                    state.exposure_count,
                    state.attempt_count,
                    state.success_count,
                    state.failure_count,
                    state.last_seen_at,
                    state.last_attempt_at,
                    state.last_success_at,
                    state.last_free_recall_success_at,
                    state.last_application_success_at,
                    state.last_pressure_success_at,
                    state.current_intervention_plan_id,
                    state.next_review_at,
                    state.review_urgency_score,
                    to_json(&state.flags)?,
                    to_json(&state.explainability)?,
                    state.created_at,
                    state.updated_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn sync_decay_profiles(
        &self,
        student_id: i64,
        knowledge_unit: &KnowledgeUnitRecord,
        state: &StudentKnowledgeStateRecord,
        features: &DerivedFeatureBundle,
        attempts: &[RetrievalAttemptRecord],
    ) -> EcoachResult<()> {
        let Some(topic_id) = knowledge_unit.topic_id else {
            return Ok(());
        };
        let now = Utc::now().to_rfc3339();
        let strategy_code = intervention_strategy_code(features.primary_failure_mode.as_deref());
        let recommended_strategy_id = self
            .conn
            .query_row(
                "SELECT id FROM memory_recovery_strategies WHERE strategy_code = ?1",
                [strategy_code],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .unwrap_or(None);

        self.conn
            .execute(
                "INSERT INTO decay_detection_profiles (
                    student_id, topic_id, node_id, decay_severity, stability_flag,
                    watchlist_flag, fragile_flag, decaying_flag, collapsed_flag,
                    time_since_last_retrieval_hours, successful_recall_count,
                    failed_recall_count, recall_speed_trend, confidence_trend,
                    hint_dependency_bp, interference_weight_bp, relapse_frequency,
                    decay_severity_bp, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9,
                    ?10, ?11,
                    ?12, ?13, ?14,
                    ?15, ?16, ?17,
                    ?18, ?19
                 )
                 ON CONFLICT(student_id, topic_id, node_id) DO UPDATE SET
                    decay_severity = excluded.decay_severity,
                    stability_flag = excluded.stability_flag,
                    watchlist_flag = excluded.watchlist_flag,
                    fragile_flag = excluded.fragile_flag,
                    decaying_flag = excluded.decaying_flag,
                    collapsed_flag = excluded.collapsed_flag,
                    time_since_last_retrieval_hours = excluded.time_since_last_retrieval_hours,
                    successful_recall_count = excluded.successful_recall_count,
                    failed_recall_count = excluded.failed_recall_count,
                    recall_speed_trend = excluded.recall_speed_trend,
                    confidence_trend = excluded.confidence_trend,
                    hint_dependency_bp = excluded.hint_dependency_bp,
                    interference_weight_bp = excluded.interference_weight_bp,
                    relapse_frequency = excluded.relapse_frequency,
                    decay_severity_bp = excluded.decay_severity_bp,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    topic_id,
                    knowledge_unit.node_id,
                    decay_severity_label(state.decay_risk_score),
                    bool_to_i64(state.decay_status == "stable"),
                    bool_to_i64(state.decay_status == "watchlist"),
                    bool_to_i64(state.decay_status == "fragile"),
                    bool_to_i64(state.decay_status == "decaying"),
                    bool_to_i64(state.decay_status == "collapsed"),
                    hours_since(state.last_attempt_at.as_deref()),
                    state.success_count,
                    state.failure_count,
                    trend_label(features.latency_score, 5_500),
                    trend_label(features.confidence_calibration_score, 5_500),
                    state.support_dependency_score,
                    state.interference_risk_score,
                    self.count_recent_regressions(student_id, knowledge_unit.id, 30_i64)?,
                    state.decay_risk_score,
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO memory_strength_profiles (
                    student_id, topic_id, memory_strength_bp, recognition_only_bp,
                    cued_recall_bp, free_recall_bp, applied_recall_bp,
                    transferred_recall_bp, pressured_recall_bp,
                    pressure_resistance_bp, interference_risk_bp,
                    last_three_outcomes_json, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7,
                    ?8, ?9,
                    ?10, ?11,
                    ?12, ?13
                 )
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    memory_strength_bp = excluded.memory_strength_bp,
                    recognition_only_bp = excluded.recognition_only_bp,
                    cued_recall_bp = excluded.cued_recall_bp,
                    free_recall_bp = excluded.free_recall_bp,
                    applied_recall_bp = excluded.applied_recall_bp,
                    transferred_recall_bp = excluded.transferred_recall_bp,
                    pressured_recall_bp = excluded.pressured_recall_bp,
                    pressure_resistance_bp = excluded.pressure_resistance_bp,
                    interference_risk_bp = excluded.interference_risk_bp,
                    last_three_outcomes_json = excluded.last_three_outcomes_json,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    topic_id,
                    state.recall_profile.free_recall,
                    state.recall_profile.recognition,
                    state.recall_profile.cued_recall,
                    state.recall_profile.free_recall,
                    state.recall_profile.application,
                    state.recall_profile.transfer,
                    state.recall_profile.pressure,
                    10_000u16.saturating_sub(features.pressure_gap_score),
                    state.interference_risk_score,
                    to_json(
                        &attempts
                            .iter()
                            .take(3)
                            .map(|attempt| attempt.correctness.clone())
                            .collect::<Vec<_>>(),
                    )?,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO memory_decay_priorities (
                    student_id, topic_id, importance_weight_bp, exam_relevance_weight_bp,
                    decay_severity_weight_bp, recurrence_risk_weight_bp,
                    dependency_weight_bp, composite_priority_bp,
                    recommended_strategy_id, computed_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6,
                    ?7, ?8,
                    ?9, ?10
                 )
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    importance_weight_bp = excluded.importance_weight_bp,
                    exam_relevance_weight_bp = excluded.exam_relevance_weight_bp,
                    decay_severity_weight_bp = excluded.decay_severity_weight_bp,
                    recurrence_risk_weight_bp = excluded.recurrence_risk_weight_bp,
                    dependency_weight_bp = excluded.dependency_weight_bp,
                    composite_priority_bp = excluded.composite_priority_bp,
                    recommended_strategy_id = excluded.recommended_strategy_id,
                    computed_at = excluded.computed_at",
                params![
                    student_id,
                    topic_id,
                    knowledge_unit.importance_weight_bp,
                    knowledge_unit.exam_frequency_weight_bp,
                    state.decay_risk_score,
                    state.interference_risk_score,
                    knowledge_unit.dependency_weight_bp,
                    clamp_bp(
                        (knowledge_unit.importance_weight_bp as i64 / 4)
                            + (knowledge_unit.exam_frequency_weight_bp as i64 / 4)
                            + (state.decay_risk_score as i64 / 3)
                            + (state.interference_risk_score as i64 / 6),
                    ),
                    recommended_strategy_id,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.sync_study_decay_profile(student_id, attempts)?;
        self.sync_performance_time_profile(student_id)?;
        Ok(())
    }

    fn sync_pressure_profile(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        attempts: &[RetrievalAttemptRecord],
    ) -> EcoachResult<()> {
        let calm_attempts = attempts
            .iter()
            .filter(|attempt| !attempt.timed && attempt.mode != "pressure")
            .collect::<Vec<_>>();
        let pressure_attempts = attempts
            .iter()
            .filter(|attempt| attempt.timed || attempt.mode == "pressure")
            .collect::<Vec<_>>();
        let calm_accuracy = average_bp(calm_attempts.iter().map(|attempt| attempt.raw_score_bp as i64));
        let timed_accuracy =
            average_bp(pressure_attempts.iter().map(|attempt| attempt.raw_score_bp as i64));
        let pressure_gap = clamp_bp((calm_accuracy as i64 - timed_accuracy as i64).max(0));
        let switch_risk = average_bp(pressure_attempts.iter().map(|attempt| {
            if attempt.switched_answer { 10_000 } else { 0 }
        }));
        let freeze_risk = average_bp(pressure_attempts.iter().map(|attempt| {
            if attempt.freeze_marker { 10_000 } else { 0 }
        }));
        let pressure_state = if pressure_attempts.is_empty() {
            "neutral"
        } else if pressure_gap > 3_000 || freeze_risk > 3_000 {
            "vulnerable"
        } else if timed_accuracy >= 7_500 {
            "stable"
        } else {
            "watchlist"
        };

        let prior_state: Option<String> = self
            .conn
            .query_row(
                "SELECT pressure_state FROM pressure_profiles
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2",
                params![student_id, knowledge_unit_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO pressure_profiles (
                    student_id, knowledge_unit_id, calm_accuracy_bp, timed_accuracy_bp,
                    pressure_gap_score, switch_risk_score, freeze_risk_score,
                    pressure_state, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(student_id, knowledge_unit_id) DO UPDATE SET
                    calm_accuracy_bp = excluded.calm_accuracy_bp,
                    timed_accuracy_bp = excluded.timed_accuracy_bp,
                    pressure_gap_score = excluded.pressure_gap_score,
                    switch_risk_score = excluded.switch_risk_score,
                    freeze_risk_score = excluded.freeze_risk_score,
                    pressure_state = excluded.pressure_state,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    knowledge_unit_id,
                    calm_accuracy,
                    timed_accuracy,
                    pressure_gap,
                    switch_risk,
                    freeze_risk,
                    pressure_state,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if prior_state.as_deref() != Some(pressure_state) {
            self.conn
                .execute(
                    "INSERT INTO pressure_transition_events (
                        student_id, knowledge_unit_id, from_state, to_state, pressure_gap_score
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        student_id,
                        knowledge_unit_id,
                        prior_state.unwrap_or_else(|| "neutral".to_string()),
                        pressure_state,
                        pressure_gap,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        if pressure_gap > 2_500 {
            self.append_memory_engine_event(
                "pressure_gap_detected",
                Some(student_id),
                Some(knowledge_unit_id),
                json!({ "pressure_gap_score": pressure_gap }),
            )?;
        }
        Ok(())
    }

    fn sync_review_schedule(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        state: &StudentKnowledgeStateRecord,
        features: &DerivedFeatureBundle,
    ) -> EcoachResult<()> {
        if features.review_urgency_score < 2_500 {
            self.conn
                .execute(
                    "UPDATE review_schedule_items
                     SET status = 'rescheduled', updated_at = ?3
                     WHERE student_id = ?1 AND knowledge_unit_id = ?2 AND status = 'scheduled'",
                    params![student_id, knowledge_unit_id, Utc::now().to_rfc3339()],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            return Ok(());
        }

        let due_at = state.next_review_at.clone().unwrap_or_else(|| {
            (Utc::now() + Duration::hours(review_offset_hours(features.review_urgency_score)))
                .to_rfc3339()
        });
        self.upsert_review_schedule_item(
            student_id,
            knowledge_unit_id,
            &due_at,
            features.review_urgency_score,
            &features.recommended_review_mode,
            &features.recommended_review_reason,
        )?;
        self.complete_memory_jobs(student_id, knowledge_unit_id, "schedule_review")?;
        Ok(())
    }

    fn sync_intervention_plan(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        state: &StudentKnowledgeStateRecord,
        features: &DerivedFeatureBundle,
    ) -> EcoachResult<()> {
        let Some(primary_failure_mode) = features.primary_failure_mode.as_deref() else {
            return Ok(());
        };
        if state.decay_risk_score < 4_500 && features.pressure_gap_score < 2_500 {
            return Ok(());
        }

        let family = intervention_family(primary_failure_mode, features);
        let steps = intervention_steps_for_family(&family);
        let retest_plan = RetestPlan {
            recommended_mode: features.recommended_review_mode.clone(),
            target_success_bp: 7_500,
            review_after_hours: review_offset_hours(features.review_urgency_score).max(6),
        };
        let reason = features
            .recommended_action
            .clone()
            .unwrap_or_else(|| "memory recovery required".to_string());

        let existing_plan: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM intervention_plans
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2
                   AND status IN ('pending', 'active')",
                params![student_id, knowledge_unit_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(plan_id) = existing_plan {
            self.conn
                .execute(
                    "UPDATE intervention_plans
                     SET family = ?1,
                         reason = ?2,
                         primary_failure_mode = ?3,
                         target_state = ?4,
                         steps_json = ?5,
                         retest_plan_json = ?6,
                         estimated_difficulty_bp = ?7,
                         estimated_duration_min = ?8,
                         priority_score = ?9,
                         status = 'active',
                         total_step_count = ?10,
                         updated_at = ?11
                     WHERE id = ?12",
                    params![
                        family,
                        reason,
                        primary_failure_mode,
                        target_state_for_failure(primary_failure_mode),
                        to_json(&steps)?,
                        to_json(&retest_plan)?,
                        clamp_bp(
                            (state.decay_risk_score as i64 / 2)
                                + (state.support_dependency_score as i64 / 4)
                                + (features.pressure_gap_score as i64 / 4),
                        ),
                        (steps.len() as i64 * 4).max(8),
                        features.review_urgency_score,
                        steps.len() as i64,
                        Utc::now().to_rfc3339(),
                        plan_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.append_memory_engine_event(
                "intervention_plan_escalated",
                Some(student_id),
                Some(knowledge_unit_id),
                json!({ "plan_id": plan_id, "family": family }),
            )?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO intervention_plans (
                        student_id, knowledge_unit_id, family, reason,
                        primary_failure_mode, target_state, steps_json, retest_plan_json,
                        estimated_difficulty_bp, estimated_duration_min, priority_score,
                        status, completed_step_count, total_step_count, created_at, updated_at
                     ) VALUES (
                        ?1, ?2, ?3, ?4,
                        ?5, ?6, ?7, ?8,
                        ?9, ?10, ?11,
                        'pending', 0, ?12, ?13, ?13
                     )",
                    params![
                        student_id,
                        knowledge_unit_id,
                        family,
                        reason,
                        primary_failure_mode,
                        target_state_for_failure(primary_failure_mode),
                        to_json(&steps)?,
                        to_json(&retest_plan)?,
                        clamp_bp(
                            (state.decay_risk_score as i64 / 2)
                                + (state.support_dependency_score as i64 / 4)
                                + (features.pressure_gap_score as i64 / 4),
                        ),
                        (steps.len() as i64 * 4).max(8),
                        features.review_urgency_score,
                        steps.len() as i64,
                        Utc::now().to_rfc3339(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let plan_id = self.conn.last_insert_rowid();
            self.conn
                .execute(
                    "UPDATE student_knowledge_states
                     SET current_intervention_plan_id = ?3, updated_at = ?4
                     WHERE student_id = ?1 AND knowledge_unit_id = ?2",
                    params![
                        student_id,
                        knowledge_unit_id,
                        plan_id,
                        Utc::now().to_rfc3339(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.append_memory_engine_event(
                "intervention_plan_created",
                Some(student_id),
                Some(knowledge_unit_id),
                json!({ "plan_id": plan_id, "family": family }),
            )?;
        }

        if state.downstream_risk_score > 6_500 {
            self.append_memory_engine_event(
                "foundation_risk_raised",
                Some(student_id),
                Some(knowledge_unit_id),
                json!({ "downstream_risk_score": state.downstream_risk_score }),
            )?;
        }

        self.complete_memory_jobs(student_id, knowledge_unit_id, "plan_intervention")?;
        Ok(())
    }

    fn upsert_review_schedule_item(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        due_at: &str,
        urgency_score: BasisPoints,
        recommended_mode: &str,
        reason: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE review_schedule_items
                 SET status = 'rescheduled', updated_at = ?3
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2 AND status = 'scheduled'",
                params![student_id, knowledge_unit_id, Utc::now().to_rfc3339()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO review_schedule_items (
                    student_id, knowledge_unit_id, due_at, urgency_score,
                    recommended_mode, reason, status, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'scheduled', ?7, ?7)",
                params![
                    student_id,
                    knowledge_unit_id,
                    due_at,
                    urgency_score,
                    recommended_mode,
                    reason,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_memory_engine_event(
            "review_item_scheduled",
            Some(student_id),
            Some(knowledge_unit_id),
            json!({
                "due_at": due_at,
                "urgency_score": urgency_score,
                "recommended_mode": recommended_mode,
                "reason": reason,
            }),
        )?;
        self.enqueue_memory_job(
            "schedule_review",
            Some(student_id),
            Some(knowledge_unit_id),
            json!({
                "due_at": due_at,
                "urgency_score": urgency_score,
            }),
            format!("schedule-{}-{}", student_id, knowledge_unit_id),
        )?;
        Ok(())
    }

    fn append_pressure_attempt_summary(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        attempt_id: i64,
        raw_score_bp: BasisPoints,
        input: &RecordMemoryEvidenceInput,
    ) -> EcoachResult<()> {
        let pressure_gap = if input.timed
            || matches!(input.recall_mode, crate::models::RecallMode::Pressure)
        {
            clamp_bp((10_000 - raw_score_bp as i64).max(0))
        } else {
            0
        };
        self.conn
            .execute(
                "INSERT INTO pressure_attempt_summaries (
                    student_id, knowledge_unit_id, retrieval_attempt_id, timed,
                    pressure_gap_score, switch_risk_score, freeze_risk_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    student_id,
                    knowledge_unit_id,
                    attempt_id,
                    bool_to_i64(input.timed),
                    pressure_gap,
                    if input.switched_answer { 10_000 } else { 0 },
                    if input.freeze_marker { 10_000 } else { 0 },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn upsert_student_interference_edge(
        &self,
        student_id: i64,
        source_node_id: Option<i64>,
        target_node_id: Option<i64>,
        source_knowledge_unit_id: i64,
        target_knowledge_unit_id: Option<i64>,
        attempt_score: BasisPoints,
        timed: bool,
        context_tags: &[String],
    ) -> EcoachResult<()> {
        let (Some(from_node_id), Some(to_node_id), Some(target_knowledge_unit_id)) =
            (source_node_id, target_node_id, target_knowledge_unit_id)
        else {
            return Ok(());
        };
        let now = Utc::now().to_rfc3339();
        let additional_strength = clamp_bp(((10_000 - attempt_score as i64) / 2).max(1_500));
        let existing_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM interference_edges
                 WHERE student_id = ?1
                   AND source_knowledge_unit_id = ?2
                   AND target_knowledge_unit_id = ?3",
                params![student_id, source_knowledge_unit_id, target_knowledge_unit_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (event_type, edge_id) = if let Some(edge_id) = existing_id {
            self.conn
                .execute(
                    "UPDATE interference_edges
                     SET strength_score = MIN(COALESCE(strength_score, 0) + ?1, 10000),
                         confusion_strength = MIN(COALESCE(confusion_strength, 0) + ?1, 10000),
                         timed_confusion_strength = MIN(
                             COALESCE(timed_confusion_strength, 0) + ?2,
                             10000
                         ),
                         calm_confusion_strength = MIN(
                             COALESCE(calm_confusion_strength, 0) + ?3,
                             10000
                         ),
                         total_confusions = COALESCE(total_confusions, 0) + 1,
                         status = CASE
                            WHEN MIN(COALESCE(confusion_strength, 0) + ?1, 10000) >= 7500 THEN 'high_risk'
                            WHEN MIN(COALESCE(confusion_strength, 0) + ?1, 10000) >= 4500 THEN 'active'
                            ELSE 'watchlist'
                         END,
                         directionality = 'source_to_target',
                         context_tags_json = ?4,
                         last_seen_at = ?5,
                         updated_at = ?5
                     WHERE id = ?6",
                    params![
                        additional_strength,
                        if timed { additional_strength } else { 0 },
                        if timed { 0 } else { additional_strength },
                        to_json(&context_tags)?,
                        now,
                        edge_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            ("interference_edge_strengthened", edge_id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO interference_edges (
                        from_node_id, to_node_id, strength_score, last_seen_at, created_at,
                        student_id, source_knowledge_unit_id, target_knowledge_unit_id,
                        confusion_strength, directionality, timed_confusion_strength,
                        calm_confusion_strength, total_confusions, context_tags_json,
                        status, updated_at
                     ) VALUES (
                        ?1, ?2, ?3, ?4, ?4,
                        ?5, ?6, ?7,
                        ?3, 'source_to_target', ?8,
                        ?9, 1, ?10,
                        ?11, ?4
                     )",
                    params![
                        from_node_id,
                        to_node_id,
                        additional_strength,
                        now,
                        student_id,
                        source_knowledge_unit_id,
                        target_knowledge_unit_id,
                        if timed { additional_strength } else { 0 },
                        if timed { 0 } else { additional_strength },
                        to_json(&context_tags)?,
                        if additional_strength >= 7_500 {
                            "high_risk"
                        } else if additional_strength >= 4_500 {
                            "active"
                        } else {
                            "watchlist"
                        },
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            ("interference_edge_created", self.conn.last_insert_rowid())
        };

        self.conn
            .execute(
                "INSERT INTO interference_events (
                    student_id, source_knowledge_unit_id, target_knowledge_unit_id,
                    event_type, payload_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    student_id,
                    source_knowledge_unit_id,
                    target_knowledge_unit_id,
                    event_type,
                    json!({
                        "edge_id": edge_id,
                        "timed": timed,
                        "context_tags": context_tags,
                    })
                    .to_string(),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_memory_engine_event(
            event_type,
            Some(student_id),
            Some(source_knowledge_unit_id),
            json!({
                "edge_id": edge_id,
                "target_knowledge_unit_id": target_knowledge_unit_id,
            }),
        )?;
        Ok(())
    }

    fn append_memory_engine_event(
        &self,
        event_type: &str,
        student_id: Option<i64>,
        knowledge_unit_id: Option<i64>,
        payload: Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO memory_engine_events (
                    event_type, student_id, knowledge_unit_id, payload_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    event_type,
                    student_id,
                    knowledge_unit_id,
                    payload.to_string(),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn enqueue_memory_job(
        &self,
        job_type: &str,
        student_id: Option<i64>,
        knowledge_unit_id: Option<i64>,
        payload: Value,
        idempotency_key: String,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO memory_engine_jobs (
                    job_type, student_id, knowledge_unit_id, status,
                    payload_json, idempotency_key, scheduled_at
                 ) VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)",
                params![
                    job_type,
                    student_id,
                    knowledge_unit_id,
                    payload.to_string(),
                    idempotency_key,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn complete_memory_jobs(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        job_type: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE memory_engine_jobs
                 SET status = 'completed',
                     finished_at = ?4
                 WHERE job_type = ?1
                   AND student_id = ?2
                   AND knowledge_unit_id = ?3
                   AND status IN ('pending', 'running')",
                params![job_type, student_id, knowledge_unit_id, Utc::now().to_rfc3339()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_knowledge_state_transition(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        from_state: &str,
        to_state: &str,
        reason: &str,
        evidence_snapshot: &Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO knowledge_state_transitions (
                    student_id, knowledge_unit_id, from_state, to_state,
                    reason, evidence_snapshot_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    student_id,
                    knowledge_unit_id,
                    from_state,
                    to_state,
                    reason,
                    evidence_snapshot.to_string(),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_student_knowledge_state(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
    ) -> EcoachResult<Option<MemoryKnowledgeStateDetail>> {
        let state = self
            .conn
            .query_row(
                "SELECT student_id, knowledge_unit_id, node_id, topic_id, memory_state,
                        state_confidence_bp, state_updated_at, decay_status,
                        decay_risk_score, recall_profile_json, support_dependency_score,
                        confidence_calibration_score, latency_score, resilience_score,
                        primary_failure_mode, secondary_failure_mode,
                        interference_risk_score, downstream_risk_score, exposure_count,
                        attempt_count, success_count, failure_count, last_seen_at,
                        last_attempt_at, last_success_at, last_free_recall_success_at,
                        last_application_success_at, last_pressure_success_at,
                        current_intervention_plan_id, next_review_at, review_urgency_score,
                        flags_json, explanation_json, created_at, updated_at
                 FROM student_knowledge_states
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2",
                params![student_id, knowledge_unit_id],
                map_student_knowledge_state,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(state) = state else {
            return Ok(None);
        };
        let knowledge_unit = self
            .load_knowledge_unit(knowledge_unit_id)?
            .ok_or_else(|| EcoachError::NotFound("knowledge unit not found".to_string()))?;
        let active_intervention = match state.current_intervention_plan_id {
            Some(plan_id) => self.load_intervention_plan(plan_id)?,
            None => None,
        };
        let review_items = self.list_review_items_for_unit(student_id, knowledge_unit_id, 5)?;
        let recent_attempts = self.list_recent_attempts(student_id, knowledge_unit_id, 8)?;
        let interference_edges = if let Some(node_id) = knowledge_unit.node_id {
            self.list_student_interference_edges(student_id, node_id, 6)?
        } else {
            Vec::new()
        };
        let recent_transitions =
            self.list_recent_transitions(student_id, knowledge_unit_id, 6)?;
        let recent_engine_events =
            self.list_recent_memory_engine_events(student_id, knowledge_unit_id, 8)?;
        let pressure_profile = self.load_pressure_profile(student_id, knowledge_unit_id)?;

        Ok(Some(MemoryKnowledgeStateDetail {
            knowledge_unit,
            state,
            active_intervention,
            review_items,
            recent_attempts,
            interference_edges,
            recent_transitions,
            recent_engine_events,
            pressure_profile,
        }))
    }

    fn load_legacy_memory_state(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        node_id: Option<i64>,
    ) -> EcoachResult<Option<crate::models::MemoryStateRecord>> {
        self.conn
            .query_row(
                "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                        recall_fluency, decay_risk, review_due_at, last_recalled_at,
                        created_at, updated_at
                 FROM memory_states
                 WHERE student_id = ?1 AND topic_id IS ?2 AND node_id IS ?3",
                params![student_id, topic_id, node_id],
                |row| {
                    Ok(crate::models::MemoryStateRecord {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        node_id: row.get(3)?,
                        memory_state: row.get(4)?,
                        memory_strength: row.get(5)?,
                        recall_fluency: row.get(6)?,
                        decay_risk: row.get(7)?,
                        review_due_at: row.get(8)?,
                        last_recalled_at: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_knowledge_unit(
        &self,
        knowledge_unit_id: i64,
    ) -> EcoachResult<Option<KnowledgeUnitRecord>> {
        self.conn
            .query_row(
                "SELECT id, node_id, subject_id, topic_id, subtopic_id, title, canonical_label,
                        description, unit_type, difficulty_bp, importance_weight_bp,
                        dependency_weight_bp, confusion_proneness_bp,
                        exam_frequency_weight_bp, canonical_representations_json,
                        tags_json, created_at, updated_at
                 FROM knowledge_units
                 WHERE id = ?1",
                [knowledge_unit_id],
                map_knowledge_unit,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn ensure_knowledge_unit_from_existing(
        &self,
        student_id: i64,
        node_id: Option<i64>,
    ) -> EcoachResult<Option<KnowledgeUnitRecord>> {
        if let Some(node_id) = node_id {
            return self.ensure_knowledge_unit(None, Some(node_id));
        }
        let existing_unit_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT knowledge_unit_id
                 FROM student_knowledge_states
                 WHERE student_id = ?1
                 ORDER BY updated_at DESC
                 LIMIT 1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        match existing_unit_id {
            Some(unit_id) => self.load_knowledge_unit(unit_id),
            None => Ok(None),
        }
    }

    fn ensure_knowledge_unit(
        &self,
        topic_id: Option<i64>,
        node_id: Option<i64>,
    ) -> EcoachResult<Option<KnowledgeUnitRecord>> {
        if node_id.is_none() && topic_id.is_none() {
            return Ok(None);
        }

        let existing = if let Some(node_id) = node_id {
            self.conn
                .query_row(
                    "SELECT id, node_id, subject_id, topic_id, subtopic_id, title, canonical_label,
                            description, unit_type, difficulty_bp, importance_weight_bp,
                            dependency_weight_bp, confusion_proneness_bp,
                            exam_frequency_weight_bp, canonical_representations_json,
                            tags_json, created_at, updated_at
                     FROM knowledge_units
                     WHERE node_id = ?1",
                    [node_id],
                    map_knowledge_unit,
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else if let Some(topic_id) = topic_id {
            self.conn
                .query_row(
                    "SELECT id, node_id, subject_id, topic_id, subtopic_id, title, canonical_label,
                            description, unit_type, difficulty_bp, importance_weight_bp,
                            dependency_weight_bp, confusion_proneness_bp,
                            exam_frequency_weight_bp, canonical_representations_json,
                            tags_json, created_at, updated_at
                     FROM knowledge_units
                     WHERE topic_id = ?1 AND node_id IS NULL
                     LIMIT 1",
                    [topic_id],
                    map_knowledge_unit,
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else {
            None
        };
        if existing.is_some() {
            return Ok(existing);
        }

        let title = self.resolve_unit_title(topic_id, node_id)?;
        let canonical_label = slugify_label(&title);
        let subject_id = self.resolve_subject_id(topic_id)?;
        let unit_type = self.resolve_unit_type(node_id)?;
        let canonical_representations_json = json!({ "textual": [title.clone()] }).to_string();
        let tags = if let Some(topic_id) = topic_id {
            vec![format!("topic:{topic_id}")]
        } else {
            Vec::new()
        };
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO knowledge_units (
                    node_id, subject_id, topic_id, subtopic_id, title, canonical_label,
                    description, unit_type, difficulty_bp, importance_weight_bp,
                    dependency_weight_bp, confusion_proneness_bp,
                    exam_frequency_weight_bp, canonical_representations_json,
                    tags_json, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, NULL, ?4, ?5,
                    NULL, ?6, 5000, 6500,
                    5000, 1500,
                    5000, ?7,
                    ?8, ?9, ?9
                 )",
                params![
                    node_id,
                    subject_id,
                    topic_id,
                    title,
                    canonical_label,
                    unit_type,
                    canonical_representations_json,
                    to_json(&tags)?,
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.load_knowledge_unit(self.conn.last_insert_rowid())
    }

    fn sync_topic_knowledge_graph(&self, topic_id: Option<i64>) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };
        if self.column_exists("academic_nodes", "topic_id")? {
            let mut node_statement = self
                .conn
                .prepare("SELECT id FROM academic_nodes WHERE topic_id = ?1")
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let node_rows = node_statement
                .query_map([topic_id], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in node_rows {
                self.ensure_knowledge_unit(
                    Some(topic_id),
                    Some(row.map_err(|err| EcoachError::Storage(err.to_string()))?),
                )?;
            }
        }

        if self.table_exists("curriculum_relationships")? {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO knowledge_unit_edges (
                        source_unit_id, target_unit_id, edge_type, weight_bp, created_at
                     )
                     SELECT source_unit.id,
                            target_unit.id,
                            CASE relationship.relationship_type
                                WHEN 'prerequisite' THEN 'prerequisite'
                                WHEN 'builds_on' THEN 'prerequisite'
                                WHEN 'contrast_pair' THEN 'contrast_pair'
                                WHEN 'contrasts_with' THEN 'contrast_pair'
                                WHEN 'related_to' THEN 'related'
                                WHEN 'supports_application_of' THEN 'supports_application_of'
                                ELSE 'related'
                            END,
                            COALESCE(relationship.strength_score, 5000),
                            datetime('now')
                     FROM curriculum_relationships relationship
                     INNER JOIN knowledge_units source_unit
                        ON source_unit.node_id = relationship.from_entity_id
                     INNER JOIN knowledge_units target_unit
                        ON target_unit.node_id = relationship.to_entity_id
                     WHERE relationship.from_entity_type = 'node'
                       AND relationship.to_entity_type = 'node'
                       AND (source_unit.topic_id = ?1 OR target_unit.topic_id = ?1)",
                    [topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn list_knowledge_units_for_topic(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Vec<KnowledgeUnitRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, node_id, subject_id, topic_id, subtopic_id, title, canonical_label,
                        description, unit_type, difficulty_bp, importance_weight_bp,
                        dependency_weight_bp, confusion_proneness_bp,
                        exam_frequency_weight_bp, canonical_representations_json,
                        tags_json, created_at, updated_at
                 FROM knowledge_units
                 WHERE topic_id = ?1
                 ORDER BY title ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], map_knowledge_unit)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    fn list_recent_attempts(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<RetrievalAttemptRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, knowledge_unit_id, session_id, question_id,
                        mode, format, timed, time_limit_ms, response_time_ms,
                        first_commit_time_ms, correctness, raw_score_bp,
                        confidence_self_report_bp, hints_used, hint_strength_bp,
                        options_visible, formula_bank_visible, answer_text,
                        expected_node_id, intruding_node_id, switched_answer,
                        guess_likelihood_bp, freeze_marker, hesitation_score_bp,
                        derived_tags_json, created_at
                 FROM retrieval_attempts
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, knowledge_unit_id, limit as i64],
                map_retrieval_attempt,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    fn list_review_items_for_unit(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ReviewScheduleItemRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, knowledge_unit_id, due_at, urgency_score,
                        recommended_mode, reason, status, created_at, updated_at
                 FROM review_schedule_items
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2
                 ORDER BY due_at ASC, urgency_score DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, knowledge_unit_id, limit as i64],
                map_review_schedule_item,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    fn list_recent_transitions(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<KnowledgeStateTransitionRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, knowledge_unit_id, from_state, to_state, reason,
                        evidence_snapshot_json, created_at
                 FROM knowledge_state_transitions
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, knowledge_unit_id, limit as i64], map_transition)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    fn list_recent_memory_engine_events(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<MemoryEngineEventRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, event_type, student_id, knowledge_unit_id, payload_json, created_at
                 FROM memory_engine_events
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, knowledge_unit_id, limit as i64],
                map_memory_engine_event,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        rows.into_iter()
            .map(|row| row.map_err(|err| EcoachError::Storage(err.to_string())))
            .collect()
    }

    fn load_intervention_plan(&self, plan_id: i64) -> EcoachResult<Option<InterventionPlanRecord>> {
        self.conn
            .query_row(
                "SELECT id, student_id, knowledge_unit_id, family, reason,
                        primary_failure_mode, target_state, steps_json, retest_plan_json,
                        estimated_difficulty_bp, estimated_duration_min, priority_score,
                        status, completed_step_count, total_step_count, created_at, updated_at
                 FROM intervention_plans
                 WHERE id = ?1",
                [plan_id],
                map_intervention_plan,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_pressure_profile(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
    ) -> EcoachResult<Option<PressureProfileRecord>> {
        self.conn
            .query_row(
                "SELECT student_id, knowledge_unit_id, calm_accuracy_bp, timed_accuracy_bp,
                        pressure_gap_score, switch_risk_score, freeze_risk_score,
                        pressure_state, updated_at
                 FROM pressure_profiles
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2",
                params![student_id, knowledge_unit_id],
                map_pressure_profile,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn resolve_unit_title(
        &self,
        topic_id: Option<i64>,
        node_id: Option<i64>,
    ) -> EcoachResult<String> {
        if let Some(node_id) = node_id {
            if self.column_exists("academic_nodes", "canonical_title")? {
                if let Some(title) = self
                    .conn
                    .query_row(
                        "SELECT canonical_title FROM academic_nodes WHERE id = ?1",
                        [node_id],
                        |row| row.get::<_, Option<String>>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .flatten()
                {
                    if !title.trim().is_empty() {
                        return Ok(title);
                    }
                }
            }
            return Ok(format!("Node {}", node_id));
        }
        if let Some(topic_id) = topic_id {
            if let Some(name) = self
                .conn
                .query_row(
                    "SELECT name FROM topics WHERE id = ?1",
                    [topic_id],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .flatten()
            {
                if !name.trim().is_empty() {
                    return Ok(name);
                }
            }
            return Ok(format!("Topic {}", topic_id));
        }
        Ok("Knowledge Unit".to_string())
    }

    fn resolve_subject_id(&self, topic_id: Option<i64>) -> EcoachResult<Option<i64>> {
        let Some(topic_id) = topic_id else {
            return Ok(None);
        };
        if !self.column_exists("topics", "subject_id")? {
            return Ok(None);
        }
        self.conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn resolve_unit_type(&self, node_id: Option<i64>) -> EcoachResult<String> {
        let Some(node_id) = node_id else {
            return Ok("concept".to_string());
        };
        if !self.column_exists("academic_nodes", "primary_content_type")? {
            return Ok("concept".to_string());
        }
        let content_type: Option<String> = self
            .conn
            .query_row(
                "SELECT primary_content_type FROM academic_nodes WHERE id = ?1",
                [node_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten();
        Ok(content_type.unwrap_or_else(|| "concept".to_string()))
    }

    fn compute_knowledge_decay_risk(
        &self,
        legacy_state: Option<&crate::models::MemoryStateRecord>,
        rolling_accuracy: BasisPoints,
        recent_accuracy: BasisPoints,
        interference_rate: BasisPoints,
        regression_count_14d: i64,
    ) -> BasisPoints {
        let legacy_risk = legacy_state.map(|state| state.decay_risk as i64).unwrap_or(0);
        clamp_bp(
            (legacy_risk / 2)
                + ((10_000 - rolling_accuracy as i64) / 4)
                + ((10_000 - recent_accuracy as i64) / 5)
                + (interference_rate as i64 / 4)
                + (regression_count_14d * 800),
        )
    }

    fn compute_downstream_risk(
        &self,
        knowledge_unit_id: i64,
        transfer_score: BasisPoints,
        application_score: BasisPoints,
    ) -> EcoachResult<BasisPoints> {
        let dependency_weight: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(weight_bp), 0)
                 FROM knowledge_unit_edges
                 WHERE source_unit_id = ?1 AND edge_type IN ('prerequisite', 'supports_application_of')",
                [knowledge_unit_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(clamp_bp(
            (dependency_weight / 3)
                + ((10_000 - transfer_score as i64) / 3)
                + ((10_000 - application_score as i64) / 3),
        ))
    }

    fn count_recent_regressions(
        &self,
        student_id: i64,
        knowledge_unit_id: i64,
        days: i64,
    ) -> EcoachResult<i64> {
        let threshold = (Utc::now() - Duration::days(days)).to_rfc3339();
        let mut statement = self
            .conn
            .prepare(
                "SELECT from_state, to_state
                 FROM knowledge_state_transitions
                 WHERE student_id = ?1 AND knowledge_unit_id = ?2 AND created_at >= ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, knowledge_unit_id, threshold], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut count = 0;
        for row in rows {
            let (from_state, to_state) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if state_rank(&to_state) < state_rank(&from_state) {
                count += 1;
            }
        }
        Ok(count)
    }

    fn sync_study_decay_profile(
        &self,
        student_id: i64,
        attempts: &[RetrievalAttemptRecord],
    ) -> EcoachResult<()> {
        let observed = attempts.len() as i64;
        let half_life_hours = if observed >= 6 { 72 } else { 36 };
        let rapid_forgetting = if observed >= 6 { 18 } else { 12 };
        self.conn
            .execute(
                "INSERT INTO study_decay_profiles (
                    student_id, decay_curve_type, half_life_hours,
                    rapid_forgetting_threshold_hours, retention_curve_params_json,
                    observed_data_points, updated_at
                 ) VALUES (
                    ?1, 'exponential', ?2, ?3, ?4, ?5, ?6
                 )
                 ON CONFLICT(student_id) DO UPDATE SET
                    decay_curve_type = excluded.decay_curve_type,
                    half_life_hours = excluded.half_life_hours,
                    rapid_forgetting_threshold_hours = excluded.rapid_forgetting_threshold_hours,
                    retention_curve_params_json = excluded.retention_curve_params_json,
                    observed_data_points = excluded.observed_data_points,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    half_life_hours,
                    rapid_forgetting,
                    json!({ "sample_size": observed }).to_string(),
                    observed,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn sync_performance_time_profile(&self, student_id: i64) -> EcoachResult<()> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT substr(created_at, 12, 2) AS hour_bucket,
                        COALESCE(AVG(raw_score_bp), 0)
                 FROM retrieval_attempts
                 WHERE student_id = ?1
                 GROUP BY substr(created_at, 12, 2)",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok((row.get::<_, String>(0)?, clamp_bp(row.get::<_, i64>(1)?)))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut time_map = serde_json::Map::new();
        let mut best: Option<(String, BasisPoints)> = None;
        let mut worst: Option<(String, BasisPoints)> = None;
        for row in rows {
            let (hour, score) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            time_map.insert(hour.clone(), json!(score));
            if best.as_ref().map(|(_, value)| score > *value).unwrap_or(true) {
                best = Some((hour.clone(), score));
            }
            if worst.as_ref().map(|(_, value)| score < *value).unwrap_or(true) {
                worst = Some((hour, score));
            }
        }
        self.conn
            .execute(
                "INSERT INTO performance_time_profiles (
                    student_id, time_performance_json, best_performance_window,
                    worst_performance_window, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(student_id) DO UPDATE SET
                    time_performance_json = excluded.time_performance_json,
                    best_performance_window = excluded.best_performance_window,
                    worst_performance_window = excluded.worst_performance_window,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    Value::Object(time_map).to_string(),
                    best.map(|(hour, _)| hour),
                    worst.map(|(hour, _)| hour),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn table_exists(&self, table_name: &str) -> EcoachResult<bool> {
        let exists: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(exists.is_some())
    }

    fn column_exists(&self, table_name: &str, column_name: &str) -> EcoachResult<bool> {
        let pragma = format!("PRAGMA table_info({table_name})");
        let mut statement = self
            .conn
            .prepare(&pragma)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            if row.map_err(|err| EcoachError::Storage(err.to_string()))? == column_name {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn map_knowledge_unit(row: &Row<'_>) -> rusqlite::Result<KnowledgeUnitRecord> {
    Ok(KnowledgeUnitRecord {
        id: row.get(0)?,
        node_id: row.get(1)?,
        subject_id: row.get(2)?,
        topic_id: row.get(3)?,
        subtopic_id: row.get(4)?,
        title: row.get(5)?,
        canonical_label: row.get(6)?,
        description: row.get(7)?,
        unit_type: row.get(8)?,
        difficulty_bp: row.get(9)?,
        importance_weight_bp: row.get(10)?,
        dependency_weight_bp: row.get(11)?,
        confusion_proneness_bp: row.get(12)?,
        exam_frequency_weight_bp: row.get(13)?,
        canonical_representations_json: row.get(14)?,
        tags: parse_vec_from_json(row.get::<_, String>(15)?),
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn map_knowledge_unit_edge(row: &Row<'_>) -> rusqlite::Result<KnowledgeUnitEdgeRecord> {
    Ok(KnowledgeUnitEdgeRecord {
        id: row.get(0)?,
        source_unit_id: row.get(1)?,
        target_unit_id: row.get(2)?,
        edge_type: row.get(3)?,
        weight_bp: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn map_student_knowledge_state(row: &Row<'_>) -> rusqlite::Result<StudentKnowledgeStateRecord> {
    Ok(StudentKnowledgeStateRecord {
        student_id: row.get(0)?,
        knowledge_unit_id: row.get(1)?,
        node_id: row.get(2)?,
        topic_id: row.get(3)?,
        memory_state: row.get(4)?,
        state_confidence_bp: row.get(5)?,
        state_updated_at: row.get(6)?,
        decay_status: row.get(7)?,
        decay_risk_score: row.get(8)?,
        recall_profile: parse_recall_profile(row.get::<_, String>(9)?),
        support_dependency_score: row.get(10)?,
        confidence_calibration_score: row.get(11)?,
        latency_score: row.get(12)?,
        resilience_score: row.get(13)?,
        primary_failure_mode: row.get(14)?,
        secondary_failure_mode: row.get(15)?,
        interference_risk_score: row.get(16)?,
        downstream_risk_score: row.get(17)?,
        exposure_count: row.get(18)?,
        attempt_count: row.get(19)?,
        success_count: row.get(20)?,
        failure_count: row.get(21)?,
        last_seen_at: row.get(22)?,
        last_attempt_at: row.get(23)?,
        last_success_at: row.get(24)?,
        last_free_recall_success_at: row.get(25)?,
        last_application_success_at: row.get(26)?,
        last_pressure_success_at: row.get(27)?,
        current_intervention_plan_id: row.get(28)?,
        next_review_at: row.get(29)?,
        review_urgency_score: row.get(30)?,
        flags: parse_vec_from_json(row.get::<_, String>(31)?),
        explainability: parse_explainability(row.get::<_, String>(32)?),
        created_at: row.get(33)?,
        updated_at: row.get(34)?,
    })
}

fn map_retrieval_attempt(row: &Row<'_>) -> rusqlite::Result<RetrievalAttemptRecord> {
    Ok(RetrievalAttemptRecord {
        id: row.get(0)?,
        student_id: row.get(1)?,
        knowledge_unit_id: row.get(2)?,
        session_id: row.get(3)?,
        question_id: row.get(4)?,
        mode: row.get(5)?,
        format: row.get(6)?,
        timed: row.get::<_, i64>(7)? != 0,
        time_limit_ms: row.get(8)?,
        response_time_ms: row.get(9)?,
        first_commit_time_ms: row.get(10)?,
        correctness: row.get(11)?,
        raw_score_bp: row.get(12)?,
        confidence_self_report_bp: row.get(13)?,
        hints_used: row.get(14)?,
        hint_strength_bp: row.get(15)?,
        options_visible: row.get::<_, i64>(16)? != 0,
        formula_bank_visible: row.get::<_, i64>(17)? != 0,
        answer_text: row.get(18)?,
        expected_node_id: row.get(19)?,
        intruding_node_id: row.get(20)?,
        switched_answer: row.get::<_, i64>(21)? != 0,
        guess_likelihood_bp: row.get(22)?,
        freeze_marker: row.get::<_, i64>(23)? != 0,
        hesitation_score_bp: row.get(24)?,
        derived_tags: parse_vec_from_json(row.get::<_, String>(25)?),
        created_at: row.get(26)?,
    })
}

fn map_interference_edge(row: &Row<'_>) -> rusqlite::Result<StudentInterferenceEdge> {
    Ok(StudentInterferenceEdge {
        id: row.get(0)?,
        student_id: row.get(1)?,
        from_node_id: row.get(2)?,
        to_node_id: row.get(3)?,
        source_knowledge_unit_id: row.get(4)?,
        target_knowledge_unit_id: row.get(5)?,
        confusion_strength: row.get(6)?,
        directionality: row.get(7)?,
        timed_confusion_strength: row.get(8)?,
        calm_confusion_strength: row.get(9)?,
        total_confusions: row.get(10)?,
        context_tags: parse_vec_from_json(row.get::<_, String>(11)?),
        status: row.get(12)?,
        last_confusion_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn map_intervention_plan(row: &Row<'_>) -> rusqlite::Result<InterventionPlanRecord> {
    Ok(InterventionPlanRecord {
        id: row.get(0)?,
        student_id: row.get(1)?,
        knowledge_unit_id: row.get(2)?,
        family: row.get(3)?,
        reason: row.get(4)?,
        primary_failure_mode: row.get(5)?,
        target_state: row.get(6)?,
        steps: parse_steps(row.get::<_, String>(7)?),
        retest_plan: parse_retest_plan(row.get::<_, String>(8)?),
        estimated_difficulty_bp: row.get(9)?,
        estimated_duration_min: row.get(10)?,
        priority_score: row.get(11)?,
        status: row.get(12)?,
        completed_step_count: row.get(13)?,
        total_step_count: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn map_review_schedule_item(row: &Row<'_>) -> rusqlite::Result<ReviewScheduleItemRecord> {
    Ok(ReviewScheduleItemRecord {
        id: row.get(0)?,
        student_id: row.get(1)?,
        knowledge_unit_id: row.get(2)?,
        due_at: row.get(3)?,
        urgency_score: row.get(4)?,
        recommended_mode: row.get(5)?,
        reason: row.get(6)?,
        status: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn map_transition(row: &Row<'_>) -> rusqlite::Result<KnowledgeStateTransitionRecord> {
    Ok(KnowledgeStateTransitionRecord {
        id: row.get(0)?,
        student_id: row.get(1)?,
        knowledge_unit_id: row.get(2)?,
        from_state: row.get(3)?,
        to_state: row.get(4)?,
        reason: row.get(5)?,
        evidence_snapshot_json: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn map_memory_engine_event(row: &Row<'_>) -> rusqlite::Result<MemoryEngineEventRecord> {
    Ok(MemoryEngineEventRecord {
        id: row.get(0)?,
        event_type: row.get(1)?,
        student_id: row.get(2)?,
        knowledge_unit_id: row.get(3)?,
        payload_json: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn map_pressure_profile(row: &Row<'_>) -> rusqlite::Result<PressureProfileRecord> {
    Ok(PressureProfileRecord {
        student_id: row.get(0)?,
        knowledge_unit_id: row.get(1)?,
        calm_accuracy_bp: row.get(2)?,
        timed_accuracy_bp: row.get(3)?,
        pressure_gap_score: row.get(4)?,
        switch_risk_score: row.get(5)?,
        freeze_risk_score: row.get(6)?,
        pressure_state: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn parse_vec_from_json(raw: String) -> Vec<String> {
    serde_json::from_str(&raw).unwrap_or_default()
}

fn parse_recall_profile(raw: String) -> RecallProfile {
    serde_json::from_str(&raw).unwrap_or_default()
}

fn parse_explainability(raw: String) -> MemoryExplainability {
    serde_json::from_str(&raw).unwrap_or_else(|_| MemoryExplainability {
        primary_driver: "memory_signal".to_string(),
        secondary_driver: None,
        feature_summary_json: "{}".to_string(),
        recommended_action: None,
    })
}

fn parse_steps(raw: String) -> Vec<InterventionStep> {
    serde_json::from_str(&raw).unwrap_or_default()
}

fn parse_retest_plan(raw: String) -> RetestPlan {
    serde_json::from_str(&raw).unwrap_or(RetestPlan {
        recommended_mode: "free_recall".to_string(),
        target_success_bp: 7_500,
        review_after_hours: 24,
    })
}

fn to_json<T: Serialize>(value: &T) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn average_bp<I>(values: I) -> BasisPoints
where
    I: Iterator<Item = i64>,
{
    let mut sum = 0i64;
    let mut count = 0i64;
    for value in values {
        sum += value;
        count += 1;
    }
    if count == 0 { 0 } else { clamp_bp(sum / count) }
}

fn mode_average(attempts: &[RetrievalAttemptRecord], mode: &str) -> BasisPoints {
    average_bp(
        attempts
            .iter()
            .filter(|attempt| attempt.mode == mode)
            .map(|attempt| attempt.raw_score_bp as i64),
    )
}

fn normalized_latency_score(response_time_ms: i64, time_limit_ms: Option<i64>) -> i64 {
    let target = time_limit_ms.unwrap_or(30_000).max(1) as f64;
    let ratio = (target / response_time_ms.max(1) as f64).clamp(0.0, 1.0);
    to_bp(ratio) as i64
}

fn support_dependency_for_attempt(
    hints_used: i64,
    hint_strength_bp: Option<BasisPoints>,
    options_visible: bool,
    formula_bank_visible: bool,
) -> BasisPoints {
    clamp_bp(
        (hints_used * 1_500)
            + hint_strength_bp.unwrap_or(0) as i64
            + if options_visible { 1_500 } else { 0 }
            + if formula_bank_visible { 1_500 } else { 0 },
    )
}

fn confidence_alignment(
    confidence_self_report_bp: Option<BasisPoints>,
    raw_score_bp: BasisPoints,
) -> BasisPoints {
    let confidence = confidence_self_report_bp.unwrap_or(raw_score_bp);
    clamp_bp(10_000 - (confidence as i64 - raw_score_bp as i64).abs())
}

fn compute_cue_recovery_rate(attempts: &[RetrievalAttemptRecord]) -> BasisPoints {
    let hinted = attempts
        .iter()
        .filter(|attempt| attempt.hints_used > 0 || attempt.options_visible || attempt.formula_bank_visible)
        .collect::<Vec<_>>();
    if hinted.is_empty() {
        return 0;
    }
    average_bp(hinted.into_iter().map(|attempt| {
        if attempt.correctness == "correct" || attempt.raw_score_bp >= 7_000 {
            10_000
        } else {
            0
        }
    }))
}

fn compute_consistency_score(attempts: &[RetrievalAttemptRecord]) -> BasisPoints {
    if attempts.is_empty() {
        return 0;
    }
    let average = average_bp(attempts.iter().map(|attempt| attempt.raw_score_bp as i64)) as i64;
    let mean_abs_dev = attempts
        .iter()
        .map(|attempt| (attempt.raw_score_bp as i64 - average).abs())
        .sum::<i64>()
        / attempts.len() as i64;
    clamp_bp(10_000 - mean_abs_dev)
}

fn dominant_intruder(attempts: &[RetrievalAttemptRecord]) -> Option<i64> {
    let mut counts = std::collections::BTreeMap::<i64, i64>::new();
    for attempt in attempts {
        if let Some(intruder) = attempt.intruding_node_id {
            *counts.entry(intruder).or_insert(0) += 1;
        }
    }
    counts.into_iter().max_by_key(|(_, count)| *count).map(|(node_id, _)| node_id)
}

fn derive_failure_modes(
    recall_profile: &RecallProfile,
    support_dependency_score: BasisPoints,
    confidence_calibration_score: BasisPoints,
    pressure_gap_score: BasisPoints,
    interference_rate: BasisPoints,
    rolling_accuracy: BasisPoints,
    cue_recovery_rate: BasisPoints,
) -> Vec<String> {
    let mut failures = Vec::new();
    if interference_rate >= 2_500 {
        failures.push("interference".to_string());
    }
    if pressure_gap_score >= 2_500 {
        failures.push("pressure_collapse".to_string());
    }
    if support_dependency_score >= 5_500 && recall_profile.free_recall < 4_500 {
        failures.push("retrieval_blockage".to_string());
    }
    if confidence_calibration_score <= 4_000 {
        failures.push("confidence_disruption".to_string());
    }
    if recall_profile.recognition >= 6_500 && recall_profile.application <= 4_500 {
        failures.push("incomplete_understanding".to_string());
    }
    if recall_profile.recognition >= 6_000
        && recall_profile.free_recall <= 4_000
        && cue_recovery_rate >= 5_000
    {
        failures.push("context_lock".to_string());
    }
    if rolling_accuracy <= 4_000 {
        failures.push("true_forgetting".to_string());
    }
    failures.dedup();
    failures
}

fn derive_flags(
    decay_risk_score: BasisPoints,
    pressure_gap_score: BasisPoints,
    interference_risk_score: BasisPoints,
    regression_count_14d: i64,
) -> Vec<String> {
    let mut flags = Vec::new();
    if decay_risk_score >= 3_500 {
        flags.push("watchlist".to_string());
    }
    if decay_risk_score >= 5_500 {
        flags.push("fragile".to_string());
    }
    if decay_risk_score >= 8_500 {
        flags.push("collapsed".to_string());
    }
    if pressure_gap_score >= 2_500 {
        flags.push("pressure_vulnerable".to_string());
    }
    if interference_risk_score >= 3_000 {
        flags.push("interference_prone".to_string());
    }
    if regression_count_14d > 0 {
        flags.push("relapsing".to_string());
    }
    flags
}

fn recommended_review_action(features: &DerivedFeatureBundle) -> (&'static str, &'static str) {
    if features.primary_failure_mode.as_deref() == Some("interference") {
        ("contrast", "interference")
    } else if features.primary_failure_mode.as_deref() == Some("pressure_collapse") {
        ("pressure", "pressure_gap")
    } else if features.recall_profile.transfer < 5_500 {
        ("transfer", "high_dependency")
    } else if features.recall_profile.application < 5_500 {
        ("application", "high_dependency")
    } else {
        ("free_recall", "decay_risk")
    }
}

fn recommended_action_text(features: &DerivedFeatureBundle) -> Option<String> {
    match features.primary_failure_mode.as_deref() {
        Some("interference") => Some("insert contrast micro-drill now".to_string()),
        Some("pressure_collapse") => {
            Some("schedule timed stabilization before next exam block".to_string())
        }
        Some("retrieval_blockage") => Some("fade cues and require cleaner free recall".to_string()),
        Some("true_forgetting") => Some("reactivate with short spaced recovery".to_string()),
        Some("incomplete_understanding") => Some("rebuild concept before more questions".to_string()),
        _ => None,
    }
}

fn resolve_knowledge_state(
    features: &DerivedFeatureBundle,
    legacy_state: Option<&crate::models::MemoryStateRecord>,
) -> String {
    if features.attempt_count == 0 {
        return match legacy_state.map(|state| state.memory_state.as_str()) {
            Some("locked_in") => "durable".to_string(),
            Some("confirmed") => "applicable".to_string(),
            Some("anchoring") => "free_recallable".to_string(),
            Some("accessible") => "cued_recallable".to_string(),
            Some("encoded") => "familiar".to_string(),
            Some("seen") => "exposed".to_string(),
            _ => "unexposed".to_string(),
        };
    }
    if features.recall_profile.pressure >= 7_500
        && features.recall_profile.transfer >= 7_000
        && features.decay_risk_score < 3_500
    {
        return "pressure_stable".to_string();
    }
    if features.resilience_score >= 7_800 && features.decay_risk_score < 2_800 {
        return "durable".to_string();
    }
    if features.recall_profile.transfer >= 7_000 {
        return "transferable".to_string();
    }
    if features.recall_profile.application >= 6_800 {
        return "applicable".to_string();
    }
    if features.recall_profile.free_recall >= 6_400 {
        return "free_recallable".to_string();
    }
    if features.recall_profile.cued_recall >= 5_800 {
        return "cued_recallable".to_string();
    }
    if features.recall_profile.recognition >= 5_200 {
        return "recognizable".to_string();
    }
    if features.rolling_accuracy >= 3_000 {
        return "familiar".to_string();
    }
    "exposed".to_string()
}

fn resolve_decay_status(decay_risk_score: BasisPoints, memory_state: &str) -> String {
    if decay_risk_score >= 8_500 || matches!(memory_state, "exposed" | "familiar") {
        "collapsed".to_string()
    } else if decay_risk_score >= 7_000 {
        "decaying".to_string()
    } else if decay_risk_score >= 5_500 {
        "fragile".to_string()
    } else if decay_risk_score >= 3_500 {
        "watchlist".to_string()
    } else {
        "stable".to_string()
    }
}

fn explainability_primary_driver(features: &DerivedFeatureBundle) -> String {
    if features.primary_failure_mode.as_deref() == Some("interference") {
        "interference".to_string()
    } else if features.primary_failure_mode.as_deref() == Some("pressure_collapse") {
        "pressure_gap".to_string()
    } else if features.decay_risk_score >= 6_000 {
        "decay_risk".to_string()
    } else if features.support_dependency_score >= 5_500 {
        "support_dependency".to_string()
    } else {
        "retrieval_accuracy".to_string()
    }
}

fn review_offset_hours(urgency_score: BasisPoints) -> i64 {
    if urgency_score >= 8_000 {
        6
    } else if urgency_score >= 6_000 {
        18
    } else if urgency_score >= 4_000 {
        36
    } else {
        72
    }
}

fn state_rank(state: &str) -> i64 {
    match state {
        "unexposed" => 0,
        "exposed" => 1,
        "familiar" => 2,
        "recognizable" => 3,
        "cued_recallable" => 4,
        "free_recallable" => 5,
        "applicable" => 6,
        "transferable" => 7,
        "pressure_stable" => 8,
        "durable" => 9,
        _ => 0,
    }
}

fn intervention_family(primary_failure_mode: &str, features: &DerivedFeatureBundle) -> String {
    match primary_failure_mode {
        "interference" => "contrast".to_string(),
        "pressure_collapse" => "timed_stabilization".to_string(),
        "retrieval_blockage" => "cue_fading".to_string(),
        "incomplete_understanding" => {
            if features.recall_profile.application < 4_500 {
                "concept_rebuild".to_string()
            } else {
                "structure_rebuild".to_string()
            }
        }
        "context_lock" => "application_bridge".to_string(),
        "confidence_disruption" => "confidence_calibration".to_string(),
        _ => "reactivation".to_string(),
    }
}

fn intervention_steps_for_family(family: &str) -> Vec<InterventionStep> {
    let step_codes = match family {
        "contrast" => vec![
            ("identify_pair", "Name the competing concept cleanly"),
            ("contrast_drill", "Run a compare-and-contrast retrieval"),
            ("stability_check", "Verify the pair stays separated"),
        ],
        "timed_stabilization" => vec![
            ("calm_recall", "Rebuild calm accurate recall"),
            ("light_timer", "Re-test under a light timer"),
            ("pressure_retest", "Confirm under realistic pressure"),
        ],
        "cue_fading" => vec![
            ("guided_recall", "Start with light supports"),
            ("reduced_cues", "Strip cues and repeat"),
            ("independent_recall", "Finish with no support"),
        ],
        "concept_rebuild" | "structure_rebuild" => vec![
            ("reteach_core", "Rebuild the core concept structure"),
            ("link_examples", "Connect it to worked examples"),
            ("retest", "Re-test application cleanly"),
        ],
        "application_bridge" => vec![
            ("definition_recall", "Recall the rule without prompts"),
            ("application_bridge", "Bridge from recall to application"),
            ("mixed_use", "Use it across mixed contexts"),
        ],
        "confidence_calibration" => vec![
            ("prediction", "Predict confidence before answering"),
            ("answer", "Answer without changing late"),
            ("reflect", "Compare confidence to reality"),
        ],
        _ => vec![
            ("reactivate", "Reactivate the unit with short recall"),
            ("independent_recall", "Test with clean independent recall"),
            ("spaced_verify", "Verify after a short delay"),
        ],
    };

    step_codes
        .into_iter()
        .map(|(step_code, prompt)| InterventionStep {
            step_code: step_code.to_string(),
            prompt: prompt.to_string(),
            status: "pending".to_string(),
        })
        .collect()
}

fn target_state_for_failure(primary_failure_mode: &str) -> &'static str {
    match primary_failure_mode {
        "pressure_collapse" => "pressure_stable",
        "interference" => "free_recallable",
        "incomplete_understanding" | "context_lock" => "applicable",
        _ => "free_recallable",
    }
}

fn intervention_strategy_code(primary_failure_mode: Option<&str>) -> &'static str {
    match primary_failure_mode {
        Some("interference") => "confusion_separation",
        Some("pressure_collapse") => "pressure_stabilize",
        Some("retrieval_blockage") => "cue_strip",
        Some("incomplete_understanding") | Some("context_lock") => "structure_rebuild",
        Some("true_forgetting") => "long_gap_defense",
        _ => "reactivate_basic",
    }
}

fn decay_severity_label(risk: BasisPoints) -> &'static str {
    if risk >= 9_000 {
        "black"
    } else if risk >= 7_500 {
        "red"
    } else if risk >= 6_000 {
        "orange"
    } else if risk >= 3_500 {
        "yellow"
    } else {
        "green"
    }
}

fn trend_label(value: BasisPoints, neutral: i64) -> &'static str {
    if value as i64 >= neutral + 800 {
        "improving"
    } else if value as i64 <= neutral - 800 {
        "slipping"
    } else {
        "steady"
    }
}

fn hours_since(timestamp: Option<&str>) -> Option<i64> {
    timestamp.and_then(|timestamp| {
        chrono::DateTime::parse_from_rfc3339(timestamp).ok().map(|datetime| {
            Utc::now()
                .signed_duration_since(datetime.with_timezone(&Utc))
                .num_hours()
        })
    })
}

fn slugify_label(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_sep = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('_');
            last_was_sep = true;
        }
    }
    slug.trim_matches('_').to_string()
}
