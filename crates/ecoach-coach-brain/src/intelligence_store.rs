use chrono::{Duration, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    AdaptationResult, CoachJudgmentSnapshot, CoachOrchestrationSnapshot,
    InterventionEffectivenessProfile, RouteMode, TopicCase, TopicProofCertification,
    TopicTeachingStrategy, build_topic_case,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSessionInputs {
    pub station_type: String,
    pub route_mode: RouteMode,
    pub rationale: Value,
}

#[derive(Debug, Clone)]
struct TimingDecisionRow {
    action_type: String,
    action_scope: String,
    topic_id: Option<i64>,
    current_phase: Option<String>,
    rationale: Value,
}

#[derive(Debug, Clone)]
struct RiskAssessmentRow {
    scope: String,
    risk_code: String,
    risk_level: String,
    risk_score: BasisPoints,
    rationale: Value,
}

pub struct CanonicalIntelligenceStore<'a> {
    conn: &'a Connection,
}

impl<'a> CanonicalIntelligenceStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn sync_topic_bundle(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_case: &TopicCase,
        strategy: Option<&TopicTeachingStrategy>,
        proof: Option<&TopicProofCertification>,
    ) -> EcoachResult<()> {
        self.sync_topic_evidence_points(student_id, subject_id, topic_case.topic_id)?;

        let false_mastery_score = compute_false_mastery_score(topic_case, proof);
        let delayed_recall_required = requires_delayed_recall(topic_case);
        let existing = self
            .conn
            .query_row(
                "SELECT false_mastery_score, mastery_state, delayed_recall_required
                 FROM ic_topic_teaching
                 WHERE learner_id = ?1 AND topic_id = ?2",
                params![student_id, topic_case.topic_id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let evidence_point_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ic_topic_evidence_points
                 WHERE learner_id = ?1 AND topic_id = ?2",
                params![student_id, topic_case.topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let evidence_spine = json!({
            "topic_id": topic_case.topic_id,
            "topic_name": topic_case.topic_name,
            "evidence_point_count": evidence_point_count,
            "evidence_count": topic_case.evidence_count,
            "recent_attempt_count": topic_case.recent_attempt_count,
            "recent_accuracy": topic_case.recent_accuracy,
            "recent_diagnoses": topic_case.recent_diagnoses,
            "active_hypotheses": topic_case.active_hypotheses,
            "requires_probe": topic_case.requires_probe,
            "open_questions": topic_case.open_questions,
        });
        let proof_contract = json!({
            "proof_tier": proof.as_ref().map(|item| item.proof_tier.as_str()),
            "proof_label": proof.as_ref().map(|item| item.proof_label.as_str()),
            "proof_score": proof.as_ref().map(|item| item.composite_score),
            "evidence_count": proof.as_ref().map(|item| item.evidence_count),
            "proof_gaps": topic_case.proof_gaps,
            "pressure_score": proof.as_ref().map(|item| item.pressure_score),
            "reasoning_score": proof.as_ref().map(|item| item.reasoning_score),
        });
        let strategy_file = json!({
            "topic_case": topic_case,
            "strategy": strategy,
            "proof": proof,
        });
        let confidence_bundle = json!({
            "priority_score": topic_case.priority_score,
            "diagnosis_certainty": topic_case.diagnosis_certainty,
            "plan_confidence_score": strategy.map(|item| item.plan_confidence_score),
            "proof_score": proof.map(|item| item.composite_score),
            "recent_accuracy": topic_case.recent_accuracy,
            "fragility_score": topic_case.fragility_score,
            "pressure_collapse_index": topic_case.pressure_collapse_index,
        });
        let decision_id = runtime_id(
            "topic_decision",
            &[
                student_id.to_string(),
                subject_id.to_string(),
                topic_case.topic_id.to_string(),
            ],
        );
        let co_causes: Vec<String> = topic_case
            .active_hypotheses
            .iter()
            .skip(1)
            .map(|item| item.code.clone())
            .collect();
        let teaching_mode = strategy
            .map(|item| item.strategy_mode.clone())
            .unwrap_or_else(|| topic_case.recommended_intervention.mode.clone());
        let entry_point = strategy
            .and_then(|item| item.teaching_modes.first().cloned())
            .unwrap_or_else(|| topic_case.recommended_intervention.next_action_type.clone());
        let bottleneck_concept_id = strategy.and_then(lowest_concept_rank);

        self.conn
            .execute(
                "INSERT INTO ic_topic_teaching (
                    learner_id, subject_id, topic_id, decision_id, dominant_hypothesis,
                    co_causes_json, teaching_mode, entry_point, mastery_state,
                    false_mastery_score, bottleneck_concept_id, evidence_spine_json,
                    proof_contract_json, strategy_file_json, delayed_recall_required,
                    confidence_bundle_json, owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9,
                    ?10, ?11, ?12,
                    ?13, ?14, ?15,
                    ?16, 'topic', datetime('now')
                 )
                 ON CONFLICT(learner_id, topic_id) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    decision_id = excluded.decision_id,
                    dominant_hypothesis = excluded.dominant_hypothesis,
                    co_causes_json = excluded.co_causes_json,
                    teaching_mode = excluded.teaching_mode,
                    entry_point = excluded.entry_point,
                    mastery_state = excluded.mastery_state,
                    false_mastery_score = excluded.false_mastery_score,
                    bottleneck_concept_id = excluded.bottleneck_concept_id,
                    evidence_spine_json = excluded.evidence_spine_json,
                    proof_contract_json = excluded.proof_contract_json,
                    strategy_file_json = excluded.strategy_file_json,
                    delayed_recall_required = excluded.delayed_recall_required,
                    confidence_bundle_json = excluded.confidence_bundle_json,
                    owner_engine_key = excluded.owner_engine_key,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    topic_case.topic_id,
                    decision_id,
                    topic_case.primary_hypothesis_code,
                    to_json(&co_causes)?,
                    teaching_mode,
                    entry_point,
                    topic_case.mastery_state,
                    false_mastery_score as i64,
                    bottleneck_concept_id,
                    to_json(&evidence_spine)?,
                    to_json(&proof_contract)?,
                    to_json(&strategy_file)?,
                    if delayed_recall_required { 1 } else { 0 },
                    to_json(&confidence_bundle)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((old_false_mastery, old_mastery_state, old_delayed_recall)) = existing {
            let mastery_changed = old_mastery_state
                .as_deref()
                .map(|item| item != topic_case.mastery_state.as_str())
                .unwrap_or(true);
            let false_mastery_changed = old_false_mastery
                .map(|item| (item - false_mastery_score as i64).abs() >= 900)
                .unwrap_or(true);
            let delayed_recall_changed =
                old_delayed_recall != if delayed_recall_required { 1 } else { 0 };
            if mastery_changed || false_mastery_changed || delayed_recall_changed {
                self.record_invalidation(
                    student_id,
                    Some(subject_id),
                    Some(topic_case.topic_id),
                    "coverage",
                    "topic_teaching_material_change",
                )?;
                self.record_invalidation(
                    student_id,
                    Some(subject_id),
                    Some(topic_case.topic_id),
                    "sequencing",
                    "topic_teaching_material_change",
                )?;
                self.record_invalidation(
                    student_id,
                    Some(subject_id),
                    Some(topic_case.topic_id),
                    "timing",
                    "topic_teaching_material_change",
                )?;
                self.record_invalidation(
                    student_id,
                    Some(subject_id),
                    Some(topic_case.topic_id),
                    "risk",
                    "topic_teaching_material_change",
                )?;
            }
        }

        Ok(())
    }

    pub fn sync_intervention_history(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        profiles: &[InterventionEffectivenessProfile],
    ) -> EcoachResult<()> {
        for profile in profiles {
            let Some(topic_id) = profile.topic_id else {
                continue;
            };
            let intervention_id = format!(
                "intervention:{}:{}:{}:{}",
                student_id, topic_id, profile.intervention_family, profile.last_outcome
            );
            let evidence = json!({
                "times_used": profile.times_used,
                "success_rate_score": profile.success_rate_score,
                "avg_gain_score": profile.avg_gain_score,
                "recommendation": profile.recommendation,
            });
            self.conn
                .execute(
                    "INSERT INTO ic_intervention_history (
                        intervention_id, learner_id, subject_id, topic_id, intervention_family,
                        outcome_state, gain_score, confidence_score, trigger_reason,
                        evidence_json, owner_engine_key, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'topic', datetime('now'))
                     ON CONFLICT(learner_id, topic_id, intervention_family, outcome_state) DO UPDATE SET
                        subject_id = excluded.subject_id,
                        gain_score = excluded.gain_score,
                        confidence_score = excluded.confidence_score,
                        trigger_reason = excluded.trigger_reason,
                        evidence_json = excluded.evidence_json,
                        owner_engine_key = excluded.owner_engine_key,
                        created_at = datetime('now')",
                    params![
                        intervention_id,
                        student_id,
                        subject_id,
                        topic_id,
                        profile.intervention_family,
                        profile.last_outcome,
                        profile.avg_gain_score,
                        profile.success_rate_score,
                        profile.recommendation,
                        to_json(&evidence)?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    pub fn refresh_subject_runtime(
        &self,
        student_id: i64,
        subject_id: i64,
        route_mode_hint: Option<RouteMode>,
        pressure_hint: Option<i64>,
    ) -> EcoachResult<()> {
        let topic_cases = self.load_subject_topic_cases_from_raw(student_id, subject_id, 24)?;
        if topic_cases.is_empty() {
            return Ok(());
        }
        self.sync_coverage_bundle(student_id, subject_id, &topic_cases)?;
        self.sync_sequencing_bundle(student_id, subject_id, &topic_cases, route_mode_hint)?;
        self.sync_timing_bundle(
            student_id,
            subject_id,
            &topic_cases,
            pressure_hint,
            route_mode_hint,
        )?;
        Ok(())
    }

    pub fn sync_risk_snapshot(&self, snapshot: &CoachJudgmentSnapshot) -> EcoachResult<()> {
        let protection_policy = json!({
            "feature_activations": snapshot.feature_activations.iter().take(4).map(|item| json!({
                "feature_code": item.feature_code,
                "readiness_guardrail": item.readiness_guardrail,
                "rationale": item.rationale,
            })).collect::<Vec<_>>(),
            "next_best_move": snapshot.next_best_move,
        });
        let rationale = json!({
            "biggest_risk": snapshot.biggest_risk,
            "judgment_confidence_score": snapshot.judgment_confidence_score,
            "evidence_ledger": snapshot.evidence_ledger.iter().map(|item| json!({
                "ledger_code": item.ledger_code,
                "evidence_score": item.evidence_score,
                "confidence_score": item.confidence_score,
                "status": item.status,
            })).collect::<Vec<_>>(),
        });
        if let Some(subject_id) = snapshot.subject_id {
            let subject_risk = clamp_bp(
                ((10_000 - snapshot.overall_judgment_score as i64) * 45 / 100)
                    + ((10_000 - snapshot.judgment_confidence_score as i64) * 35 / 100)
                    + snapshot
                        .content_governor
                        .as_ref()
                        .map(|item| item.contradiction_risk_score as i64 / 2)
                        .unwrap_or(0)
                    + snapshot
                        .feature_activations
                        .first()
                        .map(|item| item.urgency_score as i64 / 3)
                        .unwrap_or(0),
            );
            self.insert_risk_row(
                snapshot.student_id,
                Some(subject_id),
                None,
                "subject",
                "coach_readiness",
                subject_risk,
                &protection_policy,
                &rationale,
            )?;
        }
        if let Some(topic_id) = snapshot.topic_id {
            let topic_risk = clamp_bp(
                ((10_000 - snapshot.overall_judgment_score as i64) * 35 / 100)
                    + ((10_000 - snapshot.judgment_confidence_score as i64) * 25 / 100)
                    + snapshot
                        .content_governor
                        .as_ref()
                        .map(|item| item.contradiction_risk_score as i64 * 35 / 100)
                        .unwrap_or(0)
                    + snapshot
                        .feature_activations
                        .first()
                        .map(|item| item.activation_priority_score as i64 / 4)
                        .unwrap_or(0),
            );
            self.insert_risk_row(
                snapshot.student_id,
                snapshot.subject_id,
                Some(topic_id),
                "topic",
                "topic_failure_risk",
                topic_risk,
                &protection_policy,
                &rationale,
            )?;
        }
        Ok(())
    }

    pub fn sync_adaptation_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
        result: &AdaptationResult,
    ) -> EcoachResult<()> {
        let adaptation_id = runtime_id(
            "adaptation",
            &[
                student_id.to_string(),
                subject_id.to_string(),
                result.new_mode.as_str().to_string(),
            ],
        );
        let trigger_reason = result
            .actions
            .first()
            .cloned()
            .unwrap_or_else(|| "periodic_route_recalibration".to_string());
        let what_changed = json!(result.actions);
        let previous_strategy = json!({
            "route_mode": result.previous_mode.as_str(),
            "streak_days": result.consistency.streak_days,
            "pace_label": result.consistency.pace_label,
        });
        let new_strategy = json!({
            "route_mode": result.new_mode.as_str(),
            "needs_rebuild": result.needs_rebuild,
            "pressure_score": result.pressure.pressure_score,
            "urgency_label": result.pressure.urgency_label,
            "weekly_sessions_needed": result.pressure.weekly_sessions_needed,
        });
        let tension = json!({
            "pressure": result.pressure,
            "consistency": result.consistency,
            "morale_signals": result.morale_signals,
        });
        let previous_mode_value = self
            .conn
            .query_row(
                "SELECT mode FROM ic_adaptation_log
                 WHERE learner_id = ?1 AND subject_id = ?2
                 ORDER BY created_at DESC
                 LIMIT 1",
                params![student_id, subject_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO ic_adaptation_log (
                    adaptation_id, learner_id, subject_id, topic_id, mode, trigger_reason,
                    what_changed_json, previous_strategy_json, new_strategy_json,
                    tension_at_time_json, owner_engine_key, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, NULL, ?4, ?5,
                    ?6, ?7, ?8,
                    ?9, 'adaptation', datetime('now'), datetime('now')
                 )",
                params![
                    adaptation_id,
                    student_id,
                    subject_id,
                    result.new_mode.as_str(),
                    trigger_reason,
                    to_json(&what_changed)?,
                    to_json(&previous_strategy)?,
                    to_json(&new_strategy)?,
                    to_json(&tension)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let readiness_state = learner_readiness_state(result);
        let preferred_families = preferred_intervention_families(result.new_mode);
        let avoided_families = avoided_intervention_families(result.new_mode);
        let pressure_response = json!({
            "pressure_score": result.pressure.pressure_score,
            "urgency_label": result.pressure.urgency_label,
            "feasibility_label": result.pressure.feasibility_label,
        });
        let mode_preferences = json!({
            subject_id.to_string(): {
                "route_mode": result.new_mode.as_str(),
                "study_pace": result.consistency.pace_label,
            }
        });
        let evidence_confidence = json!({
            "avg_accuracy_bp": result.consistency.avg_accuracy_bp,
            "study_days_last_14": result.consistency.study_days_last_14,
            "morale_signal_count": result.morale_signals.len(),
        });
        let confidence_scaffolding_need = clamp_bp(
            ((10_000 - result.consistency.avg_accuracy_bp as i64) * 55 / 100)
                + (result.pressure.pressure_score as i64 * 25 / 100)
                + if result
                    .morale_signals
                    .iter()
                    .any(|item| item.signal_type == "absence_warning")
                {
                    1_200
                } else {
                    0
                },
        );
        let learns_from_mistakes_score = clamp_bp(
            (result.consistency.avg_accuracy_bp as i64 * 45 / 100)
                + (result.consistency.total_questions_last_14.min(40) * 90)
                + if result.actions.iter().any(|item| item.contains("repair")) {
                    900
                } else {
                    0
                },
        );

        self.conn
            .execute(
                "INSERT INTO ic_learner_state (
                    learner_id, readiness_state, preferred_families_json, avoided_families_json,
                    avg_successful_session_minutes, pressure_response_json,
                    learns_from_mistakes_score, confidence_scaffolding_need,
                    mode_preferences_json, evidence_confidence_json, owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6,
                    ?7, ?8,
                    ?9, ?10, 'adaptation', datetime('now')
                 )
                 ON CONFLICT(learner_id) DO UPDATE SET
                    readiness_state = excluded.readiness_state,
                    preferred_families_json = excluded.preferred_families_json,
                    avoided_families_json = excluded.avoided_families_json,
                    avg_successful_session_minutes = excluded.avg_successful_session_minutes,
                    pressure_response_json = excluded.pressure_response_json,
                    learns_from_mistakes_score = excluded.learns_from_mistakes_score,
                    confidence_scaffolding_need = excluded.confidence_scaffolding_need,
                    mode_preferences_json = excluded.mode_preferences_json,
                    evidence_confidence_json = excluded.evidence_confidence_json,
                    owner_engine_key = excluded.owner_engine_key,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    readiness_state,
                    to_json(&preferred_families)?,
                    to_json(&avoided_families)?,
                    result.consistency.avg_daily_minutes_last_14.max(20),
                    to_json(&pressure_response)?,
                    learns_from_mistakes_score as i64,
                    confidence_scaffolding_need as i64,
                    to_json(&mode_preferences)?,
                    to_json(&evidence_confidence)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if previous_mode_value
            .as_deref()
            .map(|item| item != result.new_mode.as_str())
            .unwrap_or(true)
        {
            self.record_invalidation(
                student_id,
                Some(subject_id),
                None,
                "timing",
                "adaptation_mode_changed",
            )?;
            self.record_invalidation(
                student_id,
                Some(subject_id),
                None,
                "sequencing",
                "adaptation_mode_changed",
            )?;
        }

        Ok(())
    }

    pub fn record_orchestration_cycle(
        &self,
        trigger: &str,
        snapshot: &CoachOrchestrationSnapshot,
    ) -> EcoachResult<()> {
        let cycle_id = runtime_id(
            "cycle",
            &[
                snapshot.student_id.to_string(),
                snapshot.subject_id.unwrap_or_default().to_string(),
                snapshot.topic_id.unwrap_or_default().to_string(),
            ],
        );
        let trace_id = runtime_id("trace", std::slice::from_ref(&cycle_id));
        let engines_run: Vec<String> = snapshot
            .engine_health
            .iter()
            .map(|item| item.engine_key.clone())
            .collect();
        let changed_engines: Vec<String> = snapshot
            .governance_checks
            .iter()
            .filter(|item| item.status != "clear")
            .map(|item| item.owner_engine_key.clone())
            .collect();
        let conflicts_resolved: Vec<Value> = snapshot
            .arbitrations
            .iter()
            .map(|item| {
                json!({
                    "arbitration_code": item.arbitration_code,
                    "winning_engine_key": item.winning_engine_key,
                    "authority_class": item.authority_class,
                })
            })
            .collect();
        let planner_summary = json!({
            "guardrail_status": snapshot.guardrail_status,
            "next_action": snapshot.next_action,
            "overall_confidence_score": snapshot.overall_confidence_score,
            "suggested_session": snapshot.suggested_session,
        });

        self.conn
            .execute(
                "INSERT INTO ic_engine_cycle_audit (
                    cycle_id, trace_id, trigger, learner_id, subject_id, topic_id,
                    engines_run_json, changed_engines_json, conflicts_resolved_json,
                    planner_summary_json, owner_engine_key, started_at, finished_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9,
                    ?10, 'constitution', datetime('now'), datetime('now')
                 )",
                params![
                    cycle_id,
                    trace_id,
                    trigger,
                    snapshot.student_id,
                    snapshot.subject_id,
                    snapshot.topic_id,
                    to_json(&engines_run)?,
                    to_json(&changed_engines)?,
                    to_json(&conflicts_resolved)?,
                    to_json(&planner_summary)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.record_outbox_event(
            &cycle_id,
            "constitution",
            "planner.updated",
            &json!({
                "student_id": snapshot.student_id,
                "subject_id": snapshot.subject_id,
                "topic_id": snapshot.topic_id,
                "next_action": snapshot.next_action,
                "guardrail_status": snapshot.guardrail_status,
            }),
        )?;
        if snapshot.suggested_session.is_some() {
            self.record_outbox_event(
                &cycle_id,
                "constitution",
                "session.suggested",
                &json!({
                    "student_id": snapshot.student_id,
                    "subject_id": snapshot.subject_id,
                    "topic_id": snapshot.topic_id,
                    "suggested_session": snapshot.suggested_session,
                }),
            )?;
        }
        if snapshot.guardrail_status != "clear" {
            self.record_invalidation(
                snapshot.student_id,
                snapshot.subject_id,
                snapshot.topic_id,
                "planner",
                "governance_guardrail_active",
            )?;
        }

        Ok(())
    }

    pub fn list_priority_topic_cases(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TopicCase>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT strategy_file_json, confidence_bundle_json, false_mastery_score, delayed_recall_required
                 FROM ic_topic_teaching
                 WHERE learner_id = ?1
                 ORDER BY updated_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([student_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut materialized = Vec::new();
        for row in rows {
            let (strategy_file_json, confidence_bundle_json, false_mastery_score, delayed_recall) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let strategy_value: Value = parse_json_text(&strategy_file_json)?;
            let Some(topic_case_value) = strategy_value.get("topic_case") else {
                continue;
            };
            let topic_case: TopicCase = serde_json::from_value(topic_case_value.clone())
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
            let confidence_bundle: Value = parse_json_text(&confidence_bundle_json)?;
            let priority_score = confidence_bundle
                .get("priority_score")
                .and_then(Value::as_i64)
                .unwrap_or(topic_case.priority_score as i64);
            let diagnosis_certainty = confidence_bundle
                .get("diagnosis_certainty")
                .and_then(Value::as_i64)
                .unwrap_or(topic_case.diagnosis_certainty as i64);
            materialized.push((
                priority_score,
                false_mastery_score.unwrap_or(0),
                delayed_recall,
                diagnosis_certainty,
                topic_case,
            ));
        }

        materialized.sort_by(|left, right| {
            right
                .2
                .cmp(&left.2)
                .then_with(|| right.0.cmp(&left.0))
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| right.3.cmp(&left.3))
                .then_with(|| left.4.topic_id.cmp(&right.4.topic_id))
        });

        Ok(materialized
            .into_iter()
            .take(limit)
            .map(|(_, _, _, _, topic_case)| topic_case)
            .collect())
    }

    pub fn suggest_activity_override(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_case: &TopicCase,
        current_activity: &str,
        plan_day_phase: &str,
    ) -> EcoachResult<Option<String>> {
        let timing =
            self.load_timing_decisions(student_id, subject_id, Some(topic_case.topic_id))?;
        if timing
            .iter()
            .any(|item| matches!(item.action_type.as_str(), "delayed_recall" | "review_topic"))
        {
            return Ok(Some(
                if plan_day_phase == "review_day" || requires_delayed_recall(topic_case) {
                    "memory_reactivation"
                } else {
                    "review"
                }
                .to_string(),
            ));
        }
        if timing.iter().any(|item| item.action_type == "retest_topic") {
            return Ok(Some("checkpoint".to_string()));
        }
        if timing
            .iter()
            .any(|item| matches!(item.action_type.as_str(), "start_timed_work" | "run_mock"))
        {
            return Ok(Some(
                if current_activity == "speed_drill" {
                    "speed_drill"
                } else {
                    "pressure_conditioning"
                }
                .to_string(),
            ));
        }

        let risk =
            self.latest_risk_assessment(student_id, Some(subject_id), Some(topic_case.topic_id))?;
        if risk
            .as_ref()
            .map(|item| item.risk_score >= 7_500 || item.risk_level == "critical")
            .unwrap_or(false)
        {
            return Ok(Some("repair".to_string()));
        }

        if let Some(mode) = self.latest_subject_adaptation_mode(student_id, subject_id)? {
            if mode == "reactivation" && current_activity != "memory_reactivation" {
                return Ok(Some("memory_reactivation".to_string()));
            }
            if mode == "rescue" && current_activity != "repair" {
                return Ok(Some("repair".to_string()));
            }
        }

        Ok(None)
    }

    pub fn resolve_session_inputs(
        &self,
        student_id: i64,
        subject_id: i64,
        requested_station_type: &str,
        requested_route_mode: RouteMode,
    ) -> EcoachResult<CanonicalSessionInputs> {
        let timing = self.load_timing_decisions(student_id, subject_id, None)?;
        let risk = self.latest_risk_assessment(student_id, Some(subject_id), None)?;
        let adaptation_mode = self.latest_subject_adaptation_mode(student_id, subject_id)?;
        let learner_state: Option<String> = self
            .conn
            .query_row(
                "SELECT readiness_state FROM ic_learner_state WHERE learner_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut station_type = requested_station_type.to_string();
        if timing
            .iter()
            .any(|item| matches!(item.action_type.as_str(), "delayed_recall" | "review_topic"))
        {
            station_type = "review".to_string();
        } else if timing.iter().any(|item| item.action_type == "retest_topic") {
            station_type = "readiness_gate".to_string();
        } else if timing.iter().any(|item| item.action_type == "run_mock") {
            station_type = "mini_mock".to_string();
        } else if timing
            .iter()
            .any(|item| item.action_type == "start_timed_work")
        {
            station_type = "challenge".to_string();
        }

        let mut route_mode = requested_route_mode;
        if risk
            .as_ref()
            .map(|item| item.risk_score >= 8_000 || item.risk_level == "critical")
            .unwrap_or(false)
        {
            route_mode = RouteMode::Rescue;
        } else if let Some(mode) = adaptation_mode.as_deref() {
            route_mode = RouteMode::from_str(mode);
        } else if learner_state
            .as_deref()
            .map(|item| matches!(item, "recovering" | "fragile"))
            .unwrap_or(false)
        {
            route_mode = RouteMode::Reactivation;
        }

        Ok(CanonicalSessionInputs {
            station_type,
            route_mode,
            rationale: json!({
                "timing_actions": timing.iter().map(|item| json!({
                    "action_type": item.action_type,
                    "action_scope": item.action_scope,
                    "topic_id": item.topic_id,
                    "current_phase": item.current_phase,
                    "rationale": item.rationale,
                })).collect::<Vec<_>>(),
                "risk": risk.as_ref().map(|item| json!({
                    "scope": item.scope,
                    "risk_code": item.risk_code,
                    "risk_level": item.risk_level,
                    "risk_score": item.risk_score,
                    "rationale": item.rationale,
                })),
                "adaptation_mode": adaptation_mode,
                "learner_state": learner_state,
            }),
        })
    }

    fn sync_topic_evidence_points(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
    ) -> EcoachResult<()> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sqa.id, sqa.was_timed, sqa.was_transfer_variant, sqa.was_mixed_context,
                        sqa.was_retention_check, sqa.hint_count, sqa.is_correct,
                        sqa.response_time_ms, q.question_format, sqa.misconception_triggered_id,
                        sqa.evidence_weight
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1 AND q.topic_id = ?2
                 ORDER BY sqa.id DESC
                 LIMIT 400",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, i64>(10)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for row in rows {
            let (
                attempt_id,
                was_timed,
                was_transfer_variant,
                was_mixed_context,
                was_retention_check,
                hint_count,
                is_correct,
                response_time_ms,
                question_format,
                misconception_triggered_id,
                evidence_weight,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let misconception_tags = if misconception_triggered_id.is_some() {
                vec!["misconception_triggered".to_string()]
            } else {
                Vec::new()
            };
            self.conn
                .execute(
                    "INSERT INTO ic_topic_evidence_points (
                        evidence_id, learner_id, subject_id, topic_id, source_type, source_id,
                        timed, transfer, mixed_context, delayed_recall, retrieval,
                        hints_used, correctness, latency_ms, representation_type,
                        misconception_tags_json, weight, owner_engine_key, updated_at, created_at
                     ) VALUES (
                        ?1, ?2, ?3, ?4, 'student_question_attempt', ?5,
                        ?6, ?7, ?8, ?9, ?10,
                        ?11, ?12, ?13, ?14,
                        ?15, ?16, 'topic', datetime('now'), datetime('now')
                     )
                     ON CONFLICT(learner_id, source_type, source_id) DO UPDATE SET
                        subject_id = excluded.subject_id,
                        topic_id = excluded.topic_id,
                        timed = excluded.timed,
                        transfer = excluded.transfer,
                        mixed_context = excluded.mixed_context,
                        delayed_recall = excluded.delayed_recall,
                        retrieval = excluded.retrieval,
                        hints_used = excluded.hints_used,
                        correctness = excluded.correctness,
                        latency_ms = excluded.latency_ms,
                        representation_type = excluded.representation_type,
                        misconception_tags_json = excluded.misconception_tags_json,
                        weight = excluded.weight,
                        owner_engine_key = excluded.owner_engine_key,
                        updated_at = datetime('now')",
                    params![
                        format!("attempt-{}", attempt_id),
                        student_id,
                        subject_id,
                        topic_id,
                        attempt_id,
                        was_timed,
                        was_transfer_variant,
                        was_mixed_context,
                        was_retention_check,
                        was_retention_check,
                        hint_count,
                        if is_correct == 1 { 10_000 } else { 0 },
                        response_time_ms,
                        question_format,
                        to_json(&misconception_tags)?,
                        clamp_bp(evidence_weight),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn sync_coverage_bundle(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_cases: &[TopicCase],
    ) -> EcoachResult<()> {
        let mut portfolio_counts = serde_json::Map::new();
        let mut review_obligations = Vec::new();
        let mut total_repair_minutes = 0i64;
        let mut total_review_minutes = 0i64;
        let mut total_growth_minutes = 0i64;

        for topic_case in topic_cases {
            let (bucket, base_bucket, review_due, gap_types, session_demand, urgency_score) =
                classify_topic_case(topic_case);
            *portfolio_counts
                .entry(bucket.to_string())
                .or_insert_with(|| json!(0)) = json!(
                portfolio_counts
                    .get(bucket)
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    + 1
            );
            if review_due {
                review_obligations.push(json!({
                    "topic_id": topic_case.topic_id,
                    "topic_name": topic_case.topic_name,
                    "decay_risk": topic_case.decay_risk,
                    "memory_state": topic_case.memory_state,
                }));
            }
            match bucket {
                "repair_now" => {
                    total_repair_minutes += topic_case
                        .recommended_intervention
                        .recommended_minutes
                        .max(15)
                }
                "review_due" => {
                    total_review_minutes += topic_case
                        .recommended_intervention
                        .recommended_minutes
                        .max(10)
                }
                _ => {
                    total_growth_minutes += topic_case
                        .recommended_intervention
                        .recommended_minutes
                        .max(10)
                }
            }

            self.conn
                .execute(
                    "INSERT INTO ic_coverage_classification (
                        learner_id, subject_id, topic_id, bucket, base_bucket, override_applied,
                        gap_types_json, session_demand, review_due, urgency_score,
                        owner_engine_key, updated_at
                     ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, 0,
                        ?6, ?7, ?8, ?9,
                        'coverage', datetime('now')
                     )
                     ON CONFLICT(learner_id, topic_id) DO UPDATE SET
                        subject_id = excluded.subject_id,
                        bucket = excluded.bucket,
                        base_bucket = excluded.base_bucket,
                        override_applied = excluded.override_applied,
                        gap_types_json = excluded.gap_types_json,
                        session_demand = excluded.session_demand,
                        review_due = excluded.review_due,
                        urgency_score = excluded.urgency_score,
                        owner_engine_key = excluded.owner_engine_key,
                        updated_at = datetime('now')",
                    params![
                        student_id,
                        subject_id,
                        topic_case.topic_id,
                        bucket,
                        base_bucket,
                        to_json(&gap_types)?,
                        session_demand,
                        if review_due { 1 } else { 0 },
                        urgency_score as i64,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        let snapshot_id = runtime_id(
            "coverage_snapshot",
            &[student_id.to_string(), subject_id.to_string()],
        );
        let trajectory = json!({
            "avg_mastery_score": average_bp(topic_cases.iter().map(|item| item.mastery_score as i64)),
            "avg_fragility_score": average_bp(topic_cases.iter().map(|item| item.fragility_score as i64)),
            "avg_priority_score": average_bp(topic_cases.iter().map(|item| item.priority_score as i64)),
        });
        let recommendations = json!([
            if total_repair_minutes > total_review_minutes {
                "Keep the next block repair-heavy until false mastery stops rising."
            } else {
                "Protect review obligations before adding fresh content."
            },
            if topic_cases.iter().any(|item| item.requires_probe) {
                "At least one top-priority topic still needs evidence before the route can fully trust it."
            } else {
                "Most top-priority topics have enough evidence to support stronger sequencing decisions."
            },
        ]);

        self.conn
            .execute(
                "INSERT INTO ic_coverage_snapshots (
                    snapshot_id, learner_id, subject_id, portfolio_json, gap_map_json,
                    review_obligations_json, time_budget_json, trajectory_json,
                    recommendations_json, owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8,
                    ?9, 'coverage', datetime('now')
                 )",
                params![
                    snapshot_id,
                    student_id,
                    subject_id,
                    to_json(&Value::Object(portfolio_counts))?,
                    to_json(&json!(
                        topic_cases
                            .iter()
                            .map(|item| json!({
                                "topic_id": item.topic_id,
                                "topic_name": item.topic_name,
                                "primary_hypothesis_code": item.primary_hypothesis_code,
                                "proof_gaps": item.proof_gaps,
                                "open_questions": item.open_questions,
                            }))
                            .collect::<Vec<_>>()
                    ))?,
                    to_json(&review_obligations)?,
                    to_json(&json!({
                        "repair_minutes": total_repair_minutes,
                        "review_minutes": total_review_minutes,
                        "growth_minutes": total_growth_minutes,
                    }))?,
                    to_json(&trajectory)?,
                    to_json(&recommendations)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn sync_sequencing_bundle(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_cases: &[TopicCase],
        route_mode_hint: Option<RouteMode>,
    ) -> EcoachResult<()> {
        let mut ordered = topic_cases.to_vec();
        ordered.sort_by(|left, right| {
            sequencing_priority(right)
                .cmp(&sequencing_priority(left))
                .then_with(|| right.priority_score.cmp(&left.priority_score))
                .then_with(|| left.topic_id.cmp(&right.topic_id))
        });

        let chosen_model = if ordered.iter().any(|item| item.active_blocker.is_some()) {
            "repair_first"
        } else if ordered
            .iter()
            .filter(|item| requires_delayed_recall(item))
            .count()
            >= 3
        {
            "review_wave"
        } else {
            match route_mode_hint.unwrap_or(RouteMode::Balanced) {
                RouteMode::DeepMastery => "depth_first",
                RouteMode::HighYield => "high_yield",
                RouteMode::Rescue => "triage",
                RouteMode::Reactivation => "reactivation",
                RouteMode::Balanced => "balanced_growth",
            }
        };
        let decision_id = runtime_id(
            "sequence",
            &[
                student_id.to_string(),
                subject_id.to_string(),
                chosen_model.to_string(),
            ],
        );
        let model_scores = json!({
            "repair_pressure": average_bp(ordered.iter().map(|item| item.priority_score as i64)),
            "review_pressure": average_bp(ordered.iter().map(|item| item.decay_risk as i64)),
            "fragility_pressure": average_bp(ordered.iter().map(|item| item.fragility_score as i64)),
        });
        let alternatives = json!([
            "balanced_growth",
            "repair_first",
            "review_wave",
            route_mode_hint.unwrap_or(RouteMode::Balanced).as_str(),
        ]);
        let pivot_conditions = json!([
            "false_mastery_rises",
            "pressure_collapse_spikes",
            "review_backlog_expands",
        ]);
        let stability_report = json!({
            "requires_probe_count": ordered.iter().filter(|item| item.requires_probe).count(),
            "review_due_count": ordered.iter().filter(|item| requires_delayed_recall(item)).count(),
            "blocked_count": ordered.iter().filter(|item| item.active_blocker.is_some()).count(),
        });
        let lock_recommendation = json!({
            "should_lock": ordered.iter().take(2).all(|item| !item.requires_probe),
            "lock_window_sessions": if chosen_model == "repair_first" { 2 } else { 3 },
        });

        self.conn
            .execute(
                "INSERT INTO ic_sequencing_decisions (
                    decision_id, learner_id, subject_id, chosen_model, model_scores_json,
                    alternatives_json, pivot_conditions_json, stability_report_json,
                    lock_recommendation_json, owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8,
                    ?9, 'sequencing', datetime('now')
                 )",
                params![
                    decision_id,
                    student_id,
                    subject_id,
                    chosen_model,
                    to_json(&model_scores)?,
                    to_json(&alternatives)?,
                    to_json(&pivot_conditions)?,
                    to_json(&stability_report)?,
                    to_json(&lock_recommendation)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for (index, topic_case) in ordered.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO ic_sequencing_decision_topics (
                        decision_id, learner_id, subject_id, topic_id, position,
                        priority_score, moved_for_interference, notes_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        decision_id,
                        student_id,
                        subject_id,
                        topic_case.topic_id,
                        index as i64 + 1,
                        topic_case.priority_score as i64,
                        if topic_case.primary_hypothesis_code == "conceptual_confusion" {
                            1
                        } else {
                            0
                        },
                        to_json(&json!({
                            "topic_name": topic_case.topic_name,
                            "requires_probe": topic_case.requires_probe,
                            "primary_hypothesis_code": topic_case.primary_hypothesis_code,
                            "review_due": requires_delayed_recall(topic_case),
                        }))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn sync_timing_bundle(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_cases: &[TopicCase],
        pressure_hint: Option<i64>,
        route_mode_hint: Option<RouteMode>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM ic_timing_decisions
                 WHERE learner_id = ?1 AND subject_id = ?2 AND consumed = 0",
                params![student_id, subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(route_mode) = route_mode_hint {
            self.insert_timing_decision(
                student_id,
                subject_id,
                None,
                "phase_transition",
                "subject",
                Some(route_mode.as_str().to_string()),
                &json!({
                    "route_mode": route_mode.as_str(),
                    "pressure_score": pressure_hint,
                }),
            )?;
        }
        if pressure_hint.unwrap_or(0) >= 8_500 {
            self.insert_timing_decision(
                student_id,
                subject_id,
                None,
                "stop_new_learning",
                "subject",
                route_mode_hint.map(|item| item.as_str().to_string()),
                &json!({
                    "reason": "critical_exam_pressure",
                    "pressure_score": pressure_hint,
                }),
            )?;
        } else if pressure_hint.unwrap_or(0) >= 6_500 {
            self.insert_timing_decision(
                student_id,
                subject_id,
                None,
                "start_timed_work",
                "subject",
                route_mode_hint.map(|item| item.as_str().to_string()),
                &json!({
                    "reason": "exam_pressure_rising",
                    "pressure_score": pressure_hint,
                }),
            )?;
        }

        for topic_case in topic_cases.iter().take(6) {
            if requires_delayed_recall(topic_case) {
                self.insert_timing_decision(
                    student_id,
                    subject_id,
                    Some(topic_case.topic_id),
                    "delayed_recall",
                    "topic",
                    Some("review".to_string()),
                    &json!({
                        "topic_name": topic_case.topic_name,
                        "memory_state": topic_case.memory_state,
                        "decay_risk": topic_case.decay_risk,
                    }),
                )?;
            }
            if topic_case.requires_probe {
                self.insert_timing_decision(
                    student_id,
                    subject_id,
                    Some(topic_case.topic_id),
                    "retest_topic",
                    "topic",
                    Some("probe".to_string()),
                    &json!({
                        "topic_name": topic_case.topic_name,
                        "open_questions": topic_case.open_questions,
                        "diagnosis_certainty": topic_case.diagnosis_certainty,
                    }),
                )?;
            }
            if topic_case.pressure_collapse_index >= 6_000 && pressure_hint.unwrap_or(0) >= 5_000 {
                self.insert_timing_decision(
                    student_id,
                    subject_id,
                    Some(topic_case.topic_id),
                    "start_timed_work",
                    "topic",
                    Some("pressure".to_string()),
                    &json!({
                        "topic_name": topic_case.topic_name,
                        "pressure_collapse_index": topic_case.pressure_collapse_index,
                        "pressure_score": pressure_hint,
                    }),
                )?;
            }
        }

        Ok(())
    }

    fn load_subject_topic_cases_from_raw(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TopicCase>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2
                 ORDER BY sts.priority_score DESC, sts.repair_priority DESC, sts.topic_id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_cases = Vec::new();
        for row in rows {
            let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            topic_cases.push(build_topic_case(self.conn, student_id, topic_id)?);
        }
        Ok(topic_cases)
    }

    fn insert_timing_decision(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
        action_type: &str,
        action_scope: &str,
        current_phase: Option<String>,
        rationale: &Value,
    ) -> EcoachResult<()> {
        let scheduled_for = if action_type == "delayed_recall" {
            Some((Utc::now() + Duration::days(1)).to_rfc3339())
        } else {
            Some(Utc::now().to_rfc3339())
        };
        self.conn
            .execute(
                "INSERT INTO ic_timing_decisions (
                    decision_id, learner_id, subject_id, topic_id, action_type, action_scope,
                    scheduled_for, current_phase, rationale_json, source_engine, consumed,
                    owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, 'timing', 0,
                    'timing', datetime('now')
                 )",
                params![
                    runtime_id(
                        "timing",
                        &[
                            student_id.to_string(),
                            subject_id.to_string(),
                            topic_id.unwrap_or_default().to_string(),
                            action_type.to_string(),
                        ],
                    ),
                    student_id,
                    subject_id,
                    topic_id,
                    action_type,
                    action_scope,
                    scheduled_for,
                    current_phase,
                    to_json(rationale)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_risk_row(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        scope: &str,
        risk_code: &str,
        risk_score: BasisPoints,
        protection_policy: &Value,
        rationale: &Value,
    ) -> EcoachResult<()> {
        let previous = self.latest_risk_assessment(student_id, subject_id, topic_id)?;
        let risk_level = risk_level_for_score(risk_score).to_string();
        self.conn
            .execute(
                "INSERT INTO ic_risk_assessments (
                    assessment_id, learner_id, subject_id, topic_id, scope, risk_code,
                    risk_level, risk_score, protection_policy_json, rationale_json,
                    owner_engine_key, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10,
                    'risk', datetime('now')
                 )",
                params![
                    runtime_id(
                        "risk",
                        &[
                            student_id.to_string(),
                            subject_id.unwrap_or_default().to_string(),
                            topic_id.unwrap_or_default().to_string(),
                            risk_code.to_string(),
                        ],
                    ),
                    student_id,
                    subject_id,
                    topic_id,
                    scope,
                    risk_code,
                    risk_level,
                    risk_score as i64,
                    to_json(protection_policy)?,
                    to_json(rationale)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if previous
            .as_ref()
            .map(|item| (item.risk_score as i64 - risk_score as i64).abs() >= 800)
            .unwrap_or(true)
        {
            self.record_invalidation(
                student_id,
                subject_id,
                topic_id,
                "planner",
                "risk_assessment_changed",
            )?;
        }
        Ok(())
    }

    fn load_timing_decisions(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<TimingDecisionRow>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT action_type, action_scope, topic_id, current_phase, rationale_json
                 FROM ic_timing_decisions
                 WHERE learner_id = ?1
                   AND subject_id = ?2
                   AND consumed = 0
                   AND (?3 IS NULL OR topic_id = ?3 OR topic_id IS NULL)
                 ORDER BY updated_at DESC, scheduled_for ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, subject_id, topic_id], |row| {
                Ok(TimingDecisionRow {
                    action_type: row.get(0)?,
                    action_scope: row.get(1)?,
                    topic_id: row.get(2)?,
                    current_phase: row.get(3)?,
                    rationale: parse_json_text::<Value>(&row.get::<_, String>(4)?)
                        .unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut decisions = Vec::new();
        for row in rows {
            decisions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(decisions)
    }

    fn latest_risk_assessment(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<Option<RiskAssessmentRow>> {
        self.conn
            .query_row(
                "SELECT scope, risk_code, risk_level, risk_score, rationale_json
                 FROM ic_risk_assessments
                 WHERE learner_id = ?1
                   AND (?2 IS NULL OR subject_id = ?2)
                   AND (?3 IS NULL OR topic_id = ?3)
                 ORDER BY updated_at DESC
                 LIMIT 1",
                params![student_id, subject_id, topic_id],
                |row| {
                    Ok(RiskAssessmentRow {
                        scope: row.get(0)?,
                        risk_code: row.get(1)?,
                        risk_level: row.get(2)?,
                        risk_score: clamp_bp(row.get::<_, i64>(3)?),
                        rationale: parse_json_text::<Value>(&row.get::<_, String>(4)?)
                            .unwrap_or_else(|_| json!({})),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn latest_subject_adaptation_mode(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT mode FROM ic_adaptation_log
                 WHERE learner_id = ?1 AND subject_id = ?2
                 ORDER BY created_at DESC
                 LIMIT 1",
                params![student_id, subject_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn record_outbox_event(
        &self,
        cycle_id: &str,
        emitted_by: &str,
        event_type: &str,
        payload: &Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO ic_outbox_events (
                    outbox_id, cycle_id, emitted_by, event_type, payload_json,
                    owner_engine_key, created_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    'constitution', datetime('now')
                 )",
                params![
                    runtime_id("outbox", &[cycle_id.to_string(), event_type.to_string()]),
                    cycle_id,
                    emitted_by,
                    event_type,
                    to_json(payload)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn record_invalidation(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        snapshot_kind: &str,
        reason: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO ic_snapshot_invalidation (
                    invalidation_id, learner_id, subject_id, topic_id, snapshot_kind,
                    reason, owner_engine_key, created_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, 'constitution', datetime('now')
                 )",
                params![
                    runtime_id(
                        "invalidate",
                        &[
                            student_id.to_string(),
                            subject_id.unwrap_or_default().to_string(),
                            topic_id.unwrap_or_default().to_string(),
                            snapshot_kind.to_string(),
                            reason.to_string(),
                        ],
                    ),
                    student_id,
                    subject_id,
                    topic_id,
                    snapshot_kind,
                    reason,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn lowest_concept_rank(strategy: &TopicTeachingStrategy) -> Option<i64> {
    strategy
        .concept_rank
        .iter()
        .min_by_key(|item| item.mastery_score as i64 + item.stability_score as i64)
        .map(|item| item.node_id)
}

fn requires_delayed_recall(topic_case: &TopicCase) -> bool {
    topic_case.decay_risk >= 5_500
        || matches!(
            topic_case.memory_state.as_str(),
            "fading" | "fragile" | "unstable"
        )
        || topic_case.memory_strength <= 4_300
}

fn compute_false_mastery_score(
    topic_case: &TopicCase,
    proof: Option<&TopicProofCertification>,
) -> BasisPoints {
    let accuracy_gap = (topic_case.mastery_score as i64
        - topic_case
            .recent_accuracy
            .unwrap_or(topic_case.mastery_score) as i64)
        .max(0);
    let proof_penalty = proof
        .map(|item| (10_000 - item.composite_score as i64).max(0) * 20 / 100)
        .unwrap_or(900);
    clamp_bp(
        accuracy_gap * 2
            + topic_case.pressure_collapse_index as i64 * 35 / 100
            + topic_case.fragility_score as i64 * 20 / 100
            + proof_penalty
            + if topic_case.requires_probe { 1_400 } else { 0 },
    )
}

fn classify_topic_case(
    topic_case: &TopicCase,
) -> (
    &'static str,
    &'static str,
    bool,
    Vec<String>,
    i64,
    BasisPoints,
) {
    let review_due = requires_delayed_recall(topic_case);
    let bucket = if topic_case.active_blocker.is_some()
        || topic_case.primary_hypothesis_code == "blocked_topic"
    {
        "repair_now"
    } else if review_due {
        "review_due"
    } else if topic_case.requires_probe {
        "probe_before_push"
    } else if topic_case.mastery_score >= 7_500 && topic_case.fragility_score <= 3_000 {
        "performance_push"
    } else {
        "build_up"
    };
    let base_bucket = if bucket == "repair_now" {
        "repair"
    } else if bucket == "review_due" {
        "review"
    } else if bucket == "performance_push" {
        "performance"
    } else {
        "growth"
    };
    let mut gap_types = Vec::new();
    if topic_case.active_blocker.is_some() {
        gap_types.push("blocked".to_string());
    }
    if topic_case.requires_probe {
        gap_types.push("evidence_gap".to_string());
    }
    if review_due {
        gap_types.push("memory_decay".to_string());
    }
    gap_types.extend(topic_case.proof_gaps.iter().cloned());
    gap_types.sort();
    gap_types.dedup();
    let session_demand = match bucket {
        "repair_now" => 3,
        "review_due" => 2,
        "probe_before_push" => 2,
        "performance_push" => 1,
        _ => 2,
    };
    (
        bucket,
        base_bucket,
        review_due,
        gap_types,
        session_demand,
        topic_case.priority_score,
    )
}

fn sequencing_priority(topic_case: &TopicCase) -> i64 {
    let repair_bias = if topic_case.active_blocker.is_some() {
        4_000
    } else {
        0
    };
    let review_bias = if requires_delayed_recall(topic_case) {
        2_800
    } else {
        0
    };
    let probe_bias = if topic_case.requires_probe { 1_600 } else { 0 };
    topic_case.priority_score as i64 + repair_bias + review_bias + probe_bias
}

fn learner_readiness_state(result: &AdaptationResult) -> &'static str {
    match result.new_mode {
        RouteMode::Rescue => "fragile",
        RouteMode::Reactivation => "recovering",
        RouteMode::HighYield => "pressured",
        RouteMode::DeepMastery => "stable",
        RouteMode::Balanced => "steady",
    }
}

fn preferred_intervention_families(route_mode: RouteMode) -> Vec<&'static str> {
    match route_mode {
        RouteMode::DeepMastery => vec!["teach", "guided_practice", "transfer_check"],
        RouteMode::Balanced => vec!["guided_practice", "checkpoint", "review"],
        RouteMode::HighYield => vec!["timed_drill", "exam_practice", "mini_mock"],
        RouteMode::Rescue => vec!["repair", "triage", "rapid_confirm"],
        RouteMode::Reactivation => vec!["retention", "repair", "recall_probe"],
    }
}

fn avoided_intervention_families(route_mode: RouteMode) -> Vec<&'static str> {
    match route_mode {
        RouteMode::Rescue => vec!["new_learning", "long_foundation_loop"],
        RouteMode::Reactivation => vec!["full_mock"],
        RouteMode::HighYield => vec!["slow_reteach"],
        RouteMode::DeepMastery | RouteMode::Balanced => Vec::new(),
    }
}

fn risk_level_for_score(risk_score: BasisPoints) -> &'static str {
    match risk_score {
        0..=3_000 => "watch",
        3_001..=5_500 => "elevated",
        5_501..=7_500 => "high",
        _ => "critical",
    }
}

fn average_bp<I>(values: I) -> i64
where
    I: IntoIterator<Item = i64>,
{
    let mut total = 0i64;
    let mut count = 0i64;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 { 0 } else { total / count }
}

fn runtime_id(prefix: &str, parts: &[String]) -> String {
    let suffix = if parts.is_empty() {
        "root".to_string()
    } else {
        parts.join("-")
    };
    format!("{}-{}-{}", prefix, Utc::now().timestamp_micros(), suffix)
}

fn to_json<T: Serialize>(value: &T) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_text<T: DeserializeOwned>(text: &str) -> EcoachResult<T> {
    serde_json::from_str(text).map_err(|err| EcoachError::Serialization(err.to_string()))
}
