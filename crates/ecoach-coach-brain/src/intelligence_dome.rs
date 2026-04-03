use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, EngineRegistry, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    CanonicalIntelligenceStore, TopicProofEngine, build_topic_case,
    journey_adaptation::JourneyAdaptationEngine, list_priority_topic_cases,
    plan_engine::PlanEngine, prerequisite_graph::PrerequisiteGraph, topic_case::TopicCase,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRegisterItem {
    pub goal_code: String,
    pub goal_label: String,
    pub target_state: String,
    pub current_score: BasisPoints,
    pub target_score: BasisPoints,
    pub tension_score: BasisPoints,
    pub evidence_confidence: BasisPoints,
    pub urgency_rank: i64,
    pub priority_rank: i64,
    pub status: String,
    pub details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionSignal {
    pub tension_code: String,
    pub tension_label: String,
    pub topic_id: Option<i64>,
    pub severity_score: BasisPoints,
    pub desired_state: String,
    pub status: String,
    pub evidence_summary: String,
    pub recommended_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSnapshot {
    pub plan_credibility_score: BasisPoints,
    pub uncertainty_score: BasisPoints,
    pub intervention_effectiveness_score: BasisPoints,
    pub recovery_readiness_score: BasisPoints,
    pub relational_stability_score: BasisPoints,
    pub motivation_score: BasisPoints,
    pub resilience_score: BasisPoints,
    pub snapshot: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineRule {
    pub code: String,
    pub title: String,
    pub principle: String,
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicIntentCoreSnapshot {
    pub student_id: i64,
    pub focus_goal_code: String,
    pub focus_reason: String,
    pub goal_register: Vec<GoalRegisterItem>,
    pub tensions: Vec<TensionSignal>,
    pub system_health: SystemHealthSnapshot,
    pub doctrine: Vec<DoctrineRule>,
    pub engine_registry: EngineRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConceptRank {
    pub node_id: i64,
    pub title: String,
    pub node_type: String,
    pub foundation_weight: BasisPoints,
    pub exam_relevance_score: BasisPoints,
    pub mastery_score: BasisPoints,
    pub stability_score: BasisPoints,
    pub transfer_strength: BasisPoints,
    pub pressure_tolerance: BasisPoints,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTeachingStrategy {
    pub topic_id: i64,
    pub topic_name: String,
    pub subject_id: i64,
    pub strategy_mode: String,
    pub teaching_modes: Vec<String>,
    pub concept_rank: Vec<TopicConceptRank>,
    pub fallback_routes: Vec<String>,
    pub primary_hypothesis_code: String,
    pub plan_confidence_score: BasisPoints,
    pub explanation: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionEffectivenessProfile {
    pub topic_id: Option<i64>,
    pub intervention_family: String,
    pub times_used: i64,
    pub success_rate_score: BasisPoints,
    pub avg_gain_score: i64,
    pub last_outcome: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub uncertainty_score: BasisPoints,
    pub false_mastery_risk: BasisPoints,
    pub information_gain_score: BasisPoints,
    pub evidence_needed: Vec<String>,
    pub counterfactuals: Vec<String>,
    pub confidence_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProbeRecommendation {
    pub probe_code: String,
    pub title: String,
    pub rationale: String,
    pub target_signal: String,
    pub estimated_minutes: i64,
    pub confidence_gain_score: BasisPoints,
    pub question_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptInterferenceCase {
    pub subject_id: i64,
    pub topic_a_id: i64,
    pub topic_a_name: String,
    pub topic_b_id: i64,
    pub topic_b_name: String,
    pub interference_type: String,
    pub severity_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub response_mode: String,
    pub regression_audit_due: bool,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurpriseEventRecommendation {
    pub event_code: String,
    pub event_label: String,
    pub purpose: String,
    pub topic_id: i64,
    pub topic_name: String,
    pub readiness_state: String,
    pub resilience_score: BasisPoints,
    pub estimated_minutes: i64,
    pub rationale: String,
    pub question_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachReflectionCycle {
    pub topic_id: Option<i64>,
    pub cycle_stage: String,
    pub prior_strategy: Option<String>,
    pub outcome_signal: Option<String>,
    pub revision_reason: String,
    pub reopened_case: bool,
    pub follow_up_action: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachIntelligenceDomeSnapshot {
    pub intent_core: AcademicIntentCoreSnapshot,
    pub topic_strategy: Option<TopicTeachingStrategy>,
    pub uncertainty_profile: Option<UncertaintyProfile>,
    pub best_next_evidence: Vec<EvidenceProbeRecommendation>,
    pub interference_cases: Vec<ConceptInterferenceCase>,
    pub surprise_events: Vec<SurpriseEventRecommendation>,
    pub intervention_effectiveness: Vec<InterventionEffectivenessProfile>,
    pub reflection_cycles: Vec<CoachReflectionCycle>,
}

#[derive(Debug, Clone)]
struct FocusContext {
    subject_id: i64,
    subject_code: String,
    topic_id: i64,
    topic_name: String,
}

#[derive(Debug, Clone)]
struct TopicAggregate {
    avg_mastery: i64,
    avg_accuracy: i64,
    avg_transfer: i64,
    avg_consistency: i64,
    avg_memory_strength: i64,
    avg_decay_risk: i64,
    avg_pressure: i64,
    avg_recognition: i64,
    avg_disguised_accuracy: i64,
    topic_count: i64,
}

#[derive(Debug, Clone)]
struct EvidenceAggregate {
    event_count: i64,
    average_transfer_delta: i64,
    average_timed_delta: i64,
    challenged_count: i64,
    confirmed_count: i64,
    mixed_context_failures: i64,
}

#[derive(Debug, Clone)]
struct NeighborSignal {
    topic_id: i64,
    topic_name: String,
    mastery_score: i64,
    fragility_score: i64,
    pressure_collapse_index: i64,
    shared_diagnosis_count: i64,
    prerequisite_strength: i64,
}

pub struct CoachIntelligenceDomeService<'a> {
    conn: &'a Connection,
}

impl<'a> CoachIntelligenceDomeService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_engine_registry(&self) -> EngineRegistry {
        EngineRegistry::core_runtime()
    }

    pub fn build_intelligence_dome(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<CoachIntelligenceDomeSnapshot> {
        let focus = self.resolve_focus_context(student_id, subject_id, topic_id)?;
        let priority_cases = list_priority_topic_cases(self.conn, student_id, 6)?;
        let intervention_effectiveness = self.sync_intervention_effectiveness(
            student_id,
            focus.as_ref().map(|item| item.topic_id),
        )?;
        let topic_strategy = match focus.as_ref() {
            Some(context) => Some(self.build_topic_strategy(student_id, context.topic_id)?),
            None => None,
        };
        let uncertainty_profile = match (focus.as_ref(), topic_strategy.as_ref()) {
            (Some(context), Some(strategy)) => {
                Some(self.build_uncertainty_profile(student_id, context, strategy)?)
            }
            _ => None,
        };
        let interference_cases = match focus.as_ref() {
            Some(context) => self.evaluate_concept_interference(student_id, context, 3)?,
            None => Vec::new(),
        };
        let best_next_evidence = match (focus.as_ref(), uncertainty_profile.as_ref()) {
            (Some(context), Some(profile)) => {
                self.list_best_next_evidence(student_id, context, profile, 4)?
            }
            _ => Vec::new(),
        };
        let surprise_events = match (focus.as_ref(), uncertainty_profile.as_ref()) {
            (Some(context), Some(profile)) => self.list_surprise_event_recommendations(
                student_id,
                context,
                profile,
                &interference_cases,
                4,
            )?,
            _ => Vec::new(),
        };
        self.maybe_record_reflections(
            student_id,
            topic_strategy.as_ref(),
            uncertainty_profile.as_ref(),
            &interference_cases,
            &intervention_effectiveness,
        )?;
        let reflection_cycles = self.list_recent_reflection_cycles(student_id, 5)?;
        if let Some(context) = focus.as_ref() {
            let canonical_store = CanonicalIntelligenceStore::new(self.conn);
            canonical_store.sync_intervention_history(
                student_id,
                Some(context.subject_id),
                &intervention_effectiveness,
            )?;
            canonical_store.refresh_subject_runtime(student_id, context.subject_id, None, None)?;
        }
        let intent_core = self.build_academic_intent_core(
            student_id,
            focus.as_ref(),
            &priority_cases,
            uncertainty_profile.as_ref(),
            &interference_cases,
            &intervention_effectiveness,
        )?;

        Ok(CoachIntelligenceDomeSnapshot {
            intent_core,
            topic_strategy,
            uncertainty_profile,
            best_next_evidence,
            interference_cases,
            surprise_events,
            intervention_effectiveness,
            reflection_cycles,
        })
    }

    pub fn build_topic_strategy(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<TopicTeachingStrategy> {
        let context = self
            .resolve_focus_context(student_id, None, Some(topic_id))?
            .ok_or_else(|| EcoachError::NotFound(format!("topic {} not found", topic_id)))?;
        let topic_case = build_topic_case(self.conn, student_id, topic_id)?;
        let concept_rank = self.load_concept_rank(student_id, topic_id)?;
        let reentry_probe = PrerequisiteGraph::new(self.conn)
            .compute_reentry_probe(student_id, context.subject_id, topic_id)
            .unwrap_or_else(|_| crate::ReentryProbeResult {
                fragile_topic_ids: Vec::new(),
                at_risk_topic_ids: Vec::new(),
                blocking_topic_ids: Vec::new(),
                needs_reactivation: false,
                probe_question_count: 0,
            });
        let teaching_modes = self.derive_teaching_modes(&topic_case, &concept_rank, &reentry_probe);
        let fallback_routes = self.derive_fallback_routes(&topic_case, &reentry_probe);
        let strategy_mode = topic_case.recommended_intervention.mode.clone();
        let primary_hypothesis_code = topic_case.primary_hypothesis_code.clone();
        let plan_confidence_score = clamp_bp(
            ((topic_case.diagnosis_certainty as f64 * 0.55)
                + ((10_000 - topic_case.fragility_score as i64).max(0) as f64 * 0.20)
                + ((10_000 - topic_case.pressure_collapse_index as i64).max(0) as f64 * 0.10)
                + ((10_000 - reentry_probe.blocking_topic_ids.len() as i64 * 1800).max(0) as f64
                    * 0.15))
                .round() as i64,
        );
        let explanation = json!({
            "topic_case_reason": topic_case.recommended_intervention.reason,
            "proof_gaps": topic_case.proof_gaps,
            "open_questions": topic_case.open_questions,
            "requires_probe": topic_case.requires_probe,
            "hypotheses": topic_case.active_hypotheses.iter().map(|item| json!({
                "code": item.code,
                "confidence_score": item.confidence_score,
                "recommended_probe": item.recommended_probe,
                "recommended_response": item.recommended_response,
            })).collect::<Vec<_>>(),
            "reentry_probe": {
                "blocking_topic_ids": reentry_probe.blocking_topic_ids,
                "at_risk_topic_ids": reentry_probe.at_risk_topic_ids,
                "fragile_topic_ids": reentry_probe.fragile_topic_ids,
            },
            "teaching_modes": teaching_modes,
            "fallback_routes": fallback_routes,
        });

        self.conn
            .execute(
                "INSERT INTO coach_topic_strategies (
                    student_id, topic_id, strategy_mode, teaching_modes_json,
                    concept_rank_json, fallback_route_json, primary_hypothesis_code,
                    plan_confidence_score, explanation_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    strategy_mode = excluded.strategy_mode,
                    teaching_modes_json = excluded.teaching_modes_json,
                    concept_rank_json = excluded.concept_rank_json,
                    fallback_route_json = excluded.fallback_route_json,
                    primary_hypothesis_code = excluded.primary_hypothesis_code,
                    plan_confidence_score = excluded.plan_confidence_score,
                    explanation_json = excluded.explanation_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    topic_id,
                    strategy_mode,
                    serde_json::to_string(&teaching_modes)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&concept_rank)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&fallback_routes)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    primary_hypothesis_code,
                    plan_confidence_score,
                    serde_json::to_string(&explanation)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let strategy = TopicTeachingStrategy {
            topic_id,
            topic_name: context.topic_name,
            subject_id: context.subject_id,
            strategy_mode: topic_case.recommended_intervention.mode.clone(),
            teaching_modes,
            concept_rank,
            fallback_routes,
            primary_hypothesis_code: topic_case.primary_hypothesis_code.clone(),
            plan_confidence_score,
            explanation,
        };
        let proof = TopicProofEngine::new(self.conn)
            .assess_topic_proof(student_id, context.subject_id, topic_id)
            .ok();
        CanonicalIntelligenceStore::new(self.conn).sync_topic_bundle(
            student_id,
            context.subject_id,
            &topic_case,
            Some(&strategy),
            proof.as_ref(),
        )?;

        Ok(strategy)
    }

    fn build_uncertainty_profile(
        &self,
        student_id: i64,
        focus: &FocusContext,
        topic_strategy: &TopicTeachingStrategy,
    ) -> EcoachResult<UncertaintyProfile> {
        let topic_case = build_topic_case(self.conn, student_id, focus.topic_id)?;
        let evidence = self.load_evidence_aggregate(student_id, focus.topic_id)?;
        let recent_accuracy = topic_case
            .recent_accuracy
            .unwrap_or(topic_case.mastery_score) as i64;
        let low_volume_penalty = if evidence.event_count < 4 {
            2800
        } else if evidence.event_count < 8 {
            1400
        } else {
            400
        };
        let contradictory_signal = if recent_accuracy + 1200 < topic_case.mastery_score as i64 {
            1800
        } else {
            0
        };
        let uncertainty_score = clamp_bp(
            ((10_000 - topic_case.diagnosis_certainty as i64)
                + low_volume_penalty
                + contradictory_signal
                + evidence.challenged_count * 250
                + evidence.mixed_context_failures * 300)
                .min(10_000),
        );

        let transfer_gap = (topic_case.mastery_score as i64
            - self
                .load_topic_transfer_signal(student_id, focus.topic_id)?
                .unwrap_or(topic_case.mastery_score as i64))
        .max(0);
        let false_mastery_risk = clamp_bp(
            ((topic_case.mastery_score as i64 * 30 / 100)
                + (topic_case.pressure_collapse_index as i64 * 25 / 100)
                + (transfer_gap * 20 / 100)
                + ((-evidence.average_timed_delta).max(0) * 2)
                + ((-evidence.average_transfer_delta).max(0) * 2)
                + if topic_case.requires_probe { 1200 } else { 0 })
            .min(10_000),
        );

        let mut evidence_needed = Vec::new();
        if topic_case.requires_probe || evidence.event_count < 4 {
            evidence_needed.push("anchor_probe".to_string());
        }
        if topic_case.pressure_collapse_index > 5500 || evidence.average_timed_delta < -80 {
            evidence_needed.push("timed_probe".to_string());
        }
        if topic_case.decay_risk > 5200 || topic_case.memory_strength < 4200 {
            evidence_needed.push("retention_probe".to_string());
        }
        if transfer_gap > 1200 || evidence.average_transfer_delta < -80 {
            evidence_needed.push("transfer_probe".to_string());
        }
        if topic_strategy
            .teaching_modes
            .iter()
            .any(|mode| mode == "prerequisite_rollback")
        {
            evidence_needed.push("prerequisite_probe".to_string());
        }
        if topic_strategy
            .teaching_modes
            .iter()
            .any(|mode| mode == "misconception_repair" || mode == "contrast_training")
        {
            evidence_needed.push("contrast_probe".to_string());
        }
        evidence_needed.sort();
        evidence_needed.dedup();

        let information_gain_score = clamp_bp(
            ((uncertainty_score as f64 * 0.45)
                + (false_mastery_risk as f64 * 0.25)
                + (evidence_needed.len() as f64 * 900.0)
                + if evidence.event_count < 5 {
                    1100.0
                } else {
                    200.0
                })
            .round() as i64,
        );

        let mut counterfactuals = Vec::new();
        if false_mastery_risk > 6000 {
            counterfactuals.push(
                "If the topic only breaks under pressure, switch from reteaching to pressure hardening."
                    .to_string(),
            );
        }
        if evidence_needed
            .iter()
            .any(|item| item == "prerequisite_probe")
        {
            counterfactuals.push(
                "If the weakness lives in a prerequisite, roll back before repeating the topic."
                    .to_string(),
            );
        }
        if evidence_needed.iter().any(|item| item == "contrast_probe") {
            counterfactuals.push(
                "If the student knows the idea in isolation but not in mixed sets, run contrast and discrimination drills."
                    .to_string(),
            );
        }
        if counterfactuals.is_empty() {
            counterfactuals.push(
                "If the current route still fails, reopen the case instead of repeating the same intervention."
                    .to_string(),
            );
        }

        let confidence_label = match uncertainty_score {
            0..=2500 => "high_confidence",
            2501..=5000 => "watching",
            5001..=7500 => "uncertain",
            _ => "reopen_case",
        }
        .to_string();

        self.conn
            .execute(
                "INSERT INTO coach_uncertainty_profiles (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    uncertainty_score, false_mastery_risk, information_gain_score,
                    evidence_needed_json, counterfactuals_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key) DO UPDATE SET
                    uncertainty_score = excluded.uncertainty_score,
                    false_mastery_risk = excluded.false_mastery_risk,
                    information_gain_score = excluded.information_gain_score,
                    evidence_needed_json = excluded.evidence_needed_json,
                    counterfactuals_json = excluded.counterfactuals_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    focus.subject_id,
                    focus.subject_id,
                    focus.topic_id,
                    focus.topic_id,
                    uncertainty_score,
                    false_mastery_risk,
                    information_gain_score,
                    serde_json::to_string(&evidence_needed)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&counterfactuals)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(UncertaintyProfile {
            student_id,
            subject_id: focus.subject_id,
            topic_id: focus.topic_id,
            uncertainty_score,
            false_mastery_risk,
            information_gain_score,
            evidence_needed,
            counterfactuals,
            confidence_label,
        })
    }

    fn list_best_next_evidence(
        &self,
        student_id: i64,
        focus: &FocusContext,
        profile: &UncertaintyProfile,
        limit: usize,
    ) -> EcoachResult<Vec<EvidenceProbeRecommendation>> {
        let topic_case = build_topic_case(self.conn, student_id, focus.topic_id)?;
        let mut recommendations = Vec::new();
        let prerequisite_probe = PrerequisiteGraph::new(self.conn)
            .compute_reentry_probe(student_id, focus.subject_id, focus.topic_id)
            .unwrap_or_else(|_| crate::ReentryProbeResult {
                fragile_topic_ids: Vec::new(),
                at_risk_topic_ids: Vec::new(),
                blocking_topic_ids: Vec::new(),
                needs_reactivation: false,
                probe_question_count: 0,
            });

        for probe_code in profile.evidence_needed.iter().take(limit) {
            let (title, target_signal, rationale, estimated_minutes, question_ids) =
                match probe_code.as_str() {
                    "prerequisite_probe" => {
                        let probe_topics = if !prerequisite_probe.blocking_topic_ids.is_empty() {
                            prerequisite_probe.blocking_topic_ids.clone()
                        } else if !prerequisite_probe.at_risk_topic_ids.is_empty() {
                            prerequisite_probe.at_risk_topic_ids.clone()
                        } else {
                            vec![focus.topic_id]
                        };
                        (
                            "Prerequisite Probe".to_string(),
                            "foundational_truth".to_string(),
                            "Check whether earlier concepts are silently blocking current progress."
                                .to_string(),
                            10,
                            self.find_supporting_questions(focus.subject_id, &probe_topics, false, 3)?,
                        )
                    }
                    "timed_probe" => (
                        "Pressure Probe".to_string(),
                        "pressure_stability".to_string(),
                        "Test whether the topic collapses only when time pressure appears."
                            .to_string(),
                        8,
                        self.find_supporting_questions(focus.subject_id, &[focus.topic_id], true, 3)?,
                    ),
                    "retention_probe" => (
                        "Retention Probe".to_string(),
                        "memory_stability".to_string(),
                        "Re-check whether the knowledge is accessible after delay, not just immediately after help."
                            .to_string(),
                        6,
                        self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 2)?,
                    ),
                    "transfer_probe" => (
                        "Transfer Probe".to_string(),
                        "transfer_strength".to_string(),
                        "Move the concept into a disguised or application form to test real flexibility."
                            .to_string(),
                        9,
                        self.find_transfer_questions(focus.subject_id, focus.topic_id, 3)?,
                    ),
                    "contrast_probe" => (
                        "Contrast Probe".to_string(),
                        "boundary_clarity".to_string(),
                        "Check whether the learner can tell this concept apart from a nearby one."
                            .to_string(),
                        9,
                        self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 3)?,
                    ),
                    _ => (
                        "Anchor Probe".to_string(),
                        "base_truth".to_string(),
                        format!(
                            "Collect one clean signal before committing to a larger intervention in {}.",
                            topic_case.topic_name
                        ),
                        7,
                        self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 3)?,
                    ),
                };

            recommendations.push(EvidenceProbeRecommendation {
                probe_code: probe_code.clone(),
                title,
                rationale,
                target_signal,
                estimated_minutes,
                confidence_gain_score: clamp_bp(
                    (profile.information_gain_score as i64 - recommendations.len() as i64 * 650)
                        .max(2800),
                ),
                question_ids,
            });
        }

        if recommendations.is_empty() {
            recommendations.push(EvidenceProbeRecommendation {
                probe_code: "anchor_probe".to_string(),
                title: "Anchor Probe".to_string(),
                rationale: "Collect one clean evidence point before changing the strategy."
                    .to_string(),
                target_signal: "base_truth".to_string(),
                estimated_minutes: 6,
                confidence_gain_score: 4200,
                question_ids: self.find_supporting_questions(
                    focus.subject_id,
                    &[focus.topic_id],
                    false,
                    2,
                )?,
            });
        }

        Ok(recommendations)
    }

    fn evaluate_concept_interference(
        &self,
        student_id: i64,
        focus: &FocusContext,
        limit: usize,
    ) -> EcoachResult<Vec<ConceptInterferenceCase>> {
        let current_case = build_topic_case(self.conn, student_id, focus.topic_id)?;
        let neighbors = self.load_neighbor_signals(student_id, focus, limit.max(1))?;
        let mut cases = Vec::new();

        for neighbor in neighbors {
            let shared_pressure = ((current_case.pressure_collapse_index as i64
                + neighbor.pressure_collapse_index)
                / 2)
            .max(0);
            let severity_score = clamp_bp(
                ((neighbor.shared_diagnosis_count * 1400)
                    + (neighbor.prerequisite_strength / 2)
                    + (neighbor.fragility_score / 3)
                    + (shared_pressure / 4)
                    + if current_case.mastery_score >= 5500 && neighbor.mastery_score >= 5000 {
                        900
                    } else {
                        0
                    })
                .min(10_000),
            );
            if severity_score < 2600 {
                continue;
            }

            let interference_type =
                if neighbor.shared_diagnosis_count >= 2 && shared_pressure >= 5000 {
                    "pressure_amplified_interference"
                } else if neighbor.prerequisite_strength >= 7000 {
                    "overwrite_tension"
                } else if neighbor.shared_diagnosis_count >= 2 {
                    "pair_confusion_tension"
                } else {
                    "mixed_selection_tension"
                }
                .to_string();

            let response_mode = match interference_type.as_str() {
                "pressure_amplified_interference" => "pressure_hardening".to_string(),
                "overwrite_tension" => "regression_repair".to_string(),
                "pair_confusion_tension" => "contrast_mode".to_string(),
                _ => "selection_training".to_string(),
            };
            let confidence_score = clamp_bp(
                ((neighbor.shared_diagnosis_count * 2200)
                    + (neighbor.prerequisite_strength / 3)
                    + (if neighbor.fragility_score >= 5500 {
                        1800
                    } else {
                        900
                    }))
                .min(10_000),
            );
            let regression_audit_due =
                severity_score >= 5600 || neighbor.prerequisite_strength >= 7000;
            let evidence_summary = format!(
                "{} and {} are showing collision risk through shared diagnosis signals and rising fragility under mixed conditions.",
                focus.topic_name, neighbor.topic_name
            );

            self.upsert_interference_case(
                student_id,
                focus.subject_id,
                focus.topic_id,
                neighbor.topic_id,
                &interference_type,
                severity_score,
                confidence_score,
                &response_mode,
                regression_audit_due,
                &evidence_summary,
            )?;

            cases.push(ConceptInterferenceCase {
                subject_id: focus.subject_id,
                topic_a_id: focus.topic_id,
                topic_a_name: focus.topic_name.clone(),
                topic_b_id: neighbor.topic_id,
                topic_b_name: neighbor.topic_name,
                interference_type,
                severity_score,
                confidence_score,
                response_mode,
                regression_audit_due,
                evidence_summary,
            });
        }

        Ok(cases)
    }

    fn list_surprise_event_recommendations(
        &self,
        student_id: i64,
        focus: &FocusContext,
        profile: &UncertaintyProfile,
        interference_cases: &[ConceptInterferenceCase],
        limit: usize,
    ) -> EcoachResult<Vec<SurpriseEventRecommendation>> {
        let topic_case = build_topic_case(self.conn, student_id, focus.topic_id)?;
        let readiness_state =
            self.readiness_state_for_topic(&topic_case, profile.false_mastery_risk);
        let recent_accuracy = topic_case
            .recent_accuracy
            .unwrap_or(topic_case.mastery_score) as i64;
        let resilience_score = clamp_bp(
            ((10_000 - topic_case.pressure_collapse_index as i64) * 55 / 100
                + (10_000 - topic_case.fragility_score as i64) * 25 / 100
                + (recent_accuracy * 20 / 100))
                .clamp(0, 10_000),
        );

        let mut events = Vec::new();

        if profile.false_mastery_risk >= 5200 {
            push_surprise_event(
                &mut events,
                "shape_shift_challenge",
                "Shape-Shift Challenge",
                "Expose whether the topic holds outside the familiar format.",
                "Mastery looks better than flexible performance, so the coach should challenge the same idea in a different skin."
                    .to_string(),
                9,
                self.find_transfer_questions(focus.subject_id, focus.topic_id, 3)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }
        if topic_case.pressure_collapse_index >= 5200 {
            push_surprise_event(
                &mut events,
                "pressure_flash",
                "Pressure Flash",
                "Probe calm-vs-timed stability without turning the full session into a mock.",
                "The topic is vulnerable under time pressure, so a short timed burst is more informative than more notes."
                    .to_string(),
                7,
                self.find_supporting_questions(focus.subject_id, &[focus.topic_id], true, 3)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }
        if !interference_cases.is_empty() {
            push_surprise_event(
                &mut events,
                "ambush_contrast_check",
                "Ambush Contrast Check",
                "Test whether neighboring concepts are colliding.",
                "A nearby concept is interfering, so the coach should interrupt the main path with a discrimination check."
                    .to_string(),
                8,
                self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 3)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }
        if topic_case.decay_risk >= 5200 || topic_case.memory_strength <= 4200 {
            push_surprise_event(
                &mut events,
                "delayed_ghost_check",
                "Delayed Ghost Check",
                "See whether the topic survives a light delay and comes back cleanly.",
                "Retention is fragile, so the coach should spring a short recall check before assuming the lesson stuck."
                    .to_string(),
                6,
                self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 2)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }
        if recent_accuracy < 4800 {
            push_surprise_event(
                &mut events,
                "recovery_check",
                "Recovery Check",
                "Measure whether the learner can recover after a stumble.",
                "The learner needs a small win signal, so the coach should test for rebound instead of piling on difficulty."
                    .to_string(),
                6,
                self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 2)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }
        if events.is_empty() {
            push_surprise_event(
                &mut events,
                "pop_retrieval_burst",
                "Pop Retrieval Burst",
                "Keep the coach's private testing loop alive.",
                "A light surprise retrieval keeps the coach learning even when no emergency signal is dominant."
                    .to_string(),
                5,
                self.find_supporting_questions(focus.subject_id, &[focus.topic_id], false, 2)?,
                focus,
                &readiness_state,
                resilience_score,
            );
        }

        events.truncate(limit.max(1));
        for event in &events {
            self.insert_surprise_event_if_new(student_id, focus.subject_id, event)?;
        }
        Ok(events)
    }

    fn build_academic_intent_core(
        &self,
        student_id: i64,
        focus: Option<&FocusContext>,
        priority_cases: &[TopicCase],
        uncertainty_profile: Option<&UncertaintyProfile>,
        interference_cases: &[ConceptInterferenceCase],
        intervention_effectiveness: &[InterventionEffectivenessProfile],
    ) -> EcoachResult<AcademicIntentCoreSnapshot> {
        let topic_aggregate = self.load_topic_aggregate(student_id)?;
        let plan = PlanEngine::new(self.conn).get_coach_roadmap(student_id, 7)?;
        let plan_credibility_score = self.compute_plan_credibility(plan.as_ref(), &topic_aggregate);
        let uncertainty_score = uncertainty_profile
            .map(|profile| profile.uncertainty_score)
            .unwrap_or_else(|| {
                clamp_bp(
                    priority_cases
                        .iter()
                        .map(|item| 10_000 - item.diagnosis_certainty as i64)
                        .max()
                        .unwrap_or(3800),
                )
            });
        let confidence_integrity =
            self.load_confidence_integrity(student_id, focus.map(|item| item.subject_id))?;
        let intervention_effectiveness_score = if intervention_effectiveness.is_empty() {
            5000
        } else {
            clamp_bp(
                intervention_effectiveness
                    .iter()
                    .map(|item| item.success_rate_score as i64)
                    .sum::<i64>()
                    / intervention_effectiveness.len() as i64,
            )
        };
        let relational_stability_score = clamp_bp(
            10_000
                - interference_cases
                    .iter()
                    .map(|item| item.severity_score as i64)
                    .max()
                    .unwrap_or(0),
        );
        let motivation_score =
            self.load_motivation_score(student_id, focus.map(|item| item.subject_id))?;
        let resilience_score = clamp_bp((10_000 - topic_aggregate.avg_pressure).max(0));
        let recovery_readiness_score = clamp_bp(
            ((10_000 - uncertainty_score as i64) * 35 / 100
                + intervention_effectiveness_score as i64 * 35 / 100
                + resilience_score as i64 * 30 / 100)
                .clamp(0, 10_000),
        );

        let system_health = SystemHealthSnapshot {
            plan_credibility_score,
            uncertainty_score,
            intervention_effectiveness_score,
            recovery_readiness_score,
            relational_stability_score,
            motivation_score,
            resilience_score,
            snapshot: json!({
                "topic_count": topic_aggregate.topic_count,
                "active_case_count": priority_cases.len(),
                "plan_present": plan.is_some(),
                "highest_case_priority": priority_cases.first().map(|item| item.priority_score),
            }),
        };
        self.conn
            .execute(
                "INSERT INTO coach_system_health_snapshots (
                    student_id, plan_credibility_score, uncertainty_score,
                    intervention_effectiveness_score, recovery_readiness_score,
                    relational_stability_score, motivation_score, resilience_score,
                    snapshot_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    student_id,
                    system_health.plan_credibility_score,
                    system_health.uncertainty_score,
                    system_health.intervention_effectiveness_score,
                    system_health.recovery_readiness_score,
                    system_health.relational_stability_score,
                    system_health.motivation_score,
                    system_health.resilience_score,
                    serde_json::to_string(&system_health.snapshot)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mastery_score = clamp_bp(topic_aggregate.avg_mastery);
        let stability_score = clamp_bp(
            ((topic_aggregate.avg_memory_strength * 55 / 100)
                + ((10_000 - topic_aggregate.avg_decay_risk).max(0) * 45 / 100))
                .clamp(0, 10_000),
        );
        let transfer_score = clamp_bp(
            ((topic_aggregate.avg_transfer * 60 / 100)
                + (topic_aggregate.avg_disguised_accuracy * 40 / 100))
                .clamp(0, 10_000),
        );
        let uncertainty_reduction_score = clamp_bp(10_000 - uncertainty_score as i64);

        let mut goals = vec![
            self.goal_item(
                "conceptual_mastery",
                "Conceptual Mastery",
                "Core ideas stay correct without prompt support.",
                mastery_score,
                8500,
                clamp_bp(8500 - mastery_score as i64),
                clamp_bp(topic_aggregate.avg_consistency.max(2800)),
                json!({
                    "topic_count": topic_aggregate.topic_count,
                    "average_accuracy": topic_aggregate.avg_accuracy,
                }),
            ),
            self.goal_item(
                "stability",
                "Stability",
                "Knowledge remains usable across time and review cycles.",
                stability_score,
                8200,
                clamp_bp(8200 - stability_score as i64),
                clamp_bp(topic_aggregate.avg_memory_strength.max(2500)),
                json!({
                    "memory_strength": topic_aggregate.avg_memory_strength,
                    "decay_risk": topic_aggregate.avg_decay_risk,
                }),
            ),
            self.goal_item(
                "transfer",
                "Transfer",
                "The learner can use ideas in disguised or mixed forms.",
                transfer_score,
                7800,
                clamp_bp(7800 - transfer_score as i64),
                clamp_bp(topic_aggregate.avg_recognition.max(2600)),
                json!({
                    "transfer_signal": topic_aggregate.avg_transfer,
                    "disguised_accuracy": topic_aggregate.avg_disguised_accuracy,
                }),
            ),
            self.goal_item(
                "resilience",
                "Resilience",
                "Performance survives pressure, drift, and small setbacks.",
                resilience_score,
                7800,
                clamp_bp(7800 - resilience_score as i64),
                clamp_bp(topic_aggregate.avg_accuracy.max(2500)),
                json!({
                    "pressure_collapse_index": topic_aggregate.avg_pressure,
                }),
            ),
            self.goal_item(
                "confidence_integrity",
                "Confidence Integrity",
                "Confidence should track truth closely enough to guide coaching decisions.",
                confidence_integrity,
                7600,
                clamp_bp(7600 - confidence_integrity as i64),
                confidence_integrity,
                json!({
                    "confidence_integrity": confidence_integrity,
                }),
            ),
            self.goal_item(
                "uncertainty_reduction",
                "Uncertainty Reduction",
                "The coach should narrow root-cause uncertainty before committing to a strategy.",
                uncertainty_reduction_score,
                8200,
                uncertainty_score,
                clamp_bp(10_000 - uncertainty_score as i64),
                json!({
                    "uncertainty_score": uncertainty_score,
                    "best_next_evidence_ready": uncertainty_profile.is_some(),
                }),
            ),
            self.goal_item(
                "plan_credibility",
                "Plan Credibility",
                "The active plan should still look believable against the exam and current evidence.",
                plan_credibility_score,
                7800,
                clamp_bp(7800 - plan_credibility_score as i64),
                plan_credibility_score,
                json!({
                    "plan_present": plan.is_some(),
                    "current_phase": plan.as_ref().map(|item| item.current_phase.clone()),
                }),
            ),
        ];
        goals.sort_by(|left, right| {
            right
                .tension_score
                .cmp(&left.tension_score)
                .then_with(|| left.goal_code.cmp(&right.goal_code))
        });
        for (index, goal) in goals.iter_mut().enumerate() {
            goal.priority_rank = index as i64 + 1;
            goal.urgency_rank = if goal.tension_score >= 6500 {
                1
            } else if goal.tension_score >= 4200 {
                2
            } else {
                3
            };
            goal.status = match goal.tension_score {
                0..=1800 => "stable".to_string(),
                1801..=4200 => "improving".to_string(),
                4201..=6500 => "watch".to_string(),
                _ => "urgent".to_string(),
            };
            self.upsert_goal_register(student_id, goal)?;
        }

        let mut tensions = vec![
            self.tension_item(
                "mastery_tension",
                "Mastery Tension",
                None,
                clamp_bp(10_000 - mastery_score as i64),
                "Important ideas feel owned and reconstructable.",
                format!(
                    "Average mastery is {}, so the coach still sees unfinished conceptual gaps.",
                    mastery_score
                ),
                Some("Keep diagnosis tied to concept-level blockers, not only topic labels.".to_string()),
            ),
            self.tension_item(
                "retention_tension",
                "Retention Tension",
                None,
                clamp_bp(topic_aggregate.avg_decay_risk),
                "Knowledge stays recoverable without immediate reteaching.",
                format!(
                    "Average decay risk is {}, so the coach still needs deliberate review pressure.",
                    topic_aggregate.avg_decay_risk
                ),
                Some("Use the memory queue and delayed checks before declaring topics safe.".to_string()),
            ),
            self.tension_item(
                "transfer_tension",
                "Transfer Tension",
                None,
                clamp_bp(10_000 - transfer_score as i64),
                "The learner can shift format, context, and wording without collapse.",
                "Transfer remains weaker than direct mastery, so disguised application still needs proof."
                    .to_string(),
                Some("Prefer best-next-evidence probes that force representation shift.".to_string()),
            ),
            self.tension_item(
                "resilience_tension",
                "Resilience Tension",
                focus.map(|item| item.topic_id),
                clamp_bp(topic_aggregate.avg_pressure),
                "Timed or stressful moments should not erase accurate reasoning.",
                format!(
                    "Pressure collapse averages {}, so timed stability is still under strain.",
                    topic_aggregate.avg_pressure
                ),
                Some("Use pressure flashes and recovery checks instead of only reteaching.".to_string()),
            ),
            self.tension_item(
                "confidence_tension",
                "Confidence Tension",
                None,
                clamp_bp(10_000 - confidence_integrity as i64),
                "Confidence should become a reliable coaching signal.",
                "Confidence calibration is still incomplete, so self-report cannot be trusted blindly."
                    .to_string(),
                Some("Keep confidence-linked probes in the loop.".to_string()),
            ),
            self.tension_item(
                "plan_tension",
                "Plan Tension",
                None,
                clamp_bp(10_000 - plan_credibility_score as i64),
                "The active route should still make sense against the exam clock.",
                "The plan needs to remain evidence-driven instead of becoming a stale schedule."
                    .to_string(),
                Some("Re-score the campaign whenever reality drifts from the roadmap.".to_string()),
            ),
            self.tension_item(
                "uncertainty_tension",
                "Uncertainty Tension",
                focus.map(|item| item.topic_id),
                uncertainty_score,
                "The coach should act from the strongest current explanation, not guesswork.",
                "Open causes still remain, so the next move should collect evidence before escalating."
                    .to_string(),
                Some("Split best next evidence from best next action.".to_string()),
            ),
        ];
        if let Some(context) = focus {
            let pressure = JourneyAdaptationEngine::new(self.conn)
                .compute_deadline_pressure(student_id, context.subject_id)
                .ok();
            if let Some(pressure) = pressure {
                tensions.push(
                    self.tension_item(
                        "time_tension",
                        "Time Tension",
                        None,
                        pressure.pressure_score,
                        "The campaign should fit the exam horizon realistically.",
                        format!(
                            "{} study days remain and the route mode is {:?}.",
                            pressure.study_days_remaining, pressure.recommended_route_mode
                        ),
                        Some(
                            "Let deadline pressure change mode without overriding diagnosis."
                                .to_string(),
                        ),
                    ),
                );
            }
            let quality_tension = self.compute_quality_tension(context.topic_id)?;
            tensions.push(
                self.tension_item(
                    "quality_tension",
                    "Quality Tension",
                    Some(context.topic_id),
                    quality_tension,
                    "The topic should have enough high-signal content and probes to teach cleanly.",
                    "The coach should know when thin content quality is part of the problem."
                        .to_string(),
                    Some(
                        "Route resource generation when the evidence surface is too thin."
                            .to_string(),
                    ),
                ),
            );
        }
        if let Some(interference) = interference_cases
            .iter()
            .max_by_key(|item| item.severity_score)
        {
            tensions.push(self.tension_item(
                "interference_tension",
                "Interference Tension",
                Some(interference.topic_a_id),
                interference.severity_score,
                "Neighboring concepts should remain clear and selectable under mixed conditions.",
                interference.evidence_summary.clone(),
                Some(format!(
                    "Use {} until the relational instability drops.",
                    interference.response_mode
                )),
            ));
        }
        tensions.sort_by(|left, right| {
            right
                .severity_score
                .cmp(&left.severity_score)
                .then_with(|| left.tension_code.cmp(&right.tension_code))
        });
        for tension in &tensions {
            self.upsert_tension(student_id, tension)?;
        }

        let doctrine = doctrine_rules();
        let focus_goal_code = goals
            .first()
            .map(|item| item.goal_code.clone())
            .unwrap_or_else(|| "conceptual_mastery".to_string());
        let focus_reason = tensions
            .first()
            .map(|item| item.evidence_summary.clone())
            .unwrap_or_else(|| "No active tension is dominating right now.".to_string());

        Ok(AcademicIntentCoreSnapshot {
            student_id,
            focus_goal_code,
            focus_reason,
            goal_register: goals,
            tensions,
            system_health,
            doctrine,
            engine_registry: EngineRegistry::core_runtime(),
        })
    }

    fn resolve_focus_context(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<Option<FocusContext>> {
        if let Some(topic_id) = topic_id {
            return self.load_focus_context_by_topic(topic_id);
        }

        let mut candidates = Vec::new();
        if let Some(subject_id) = subject_id {
            candidates.push(
                self.conn
                    .query_row(
                        "SELECT t.subject_id, s.code, t.id, t.name
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2
                 ORDER BY sts.priority_score DESC, sts.repair_priority DESC, sts.topic_id ASC
                 LIMIT 1",
                        params![student_id, subject_id],
                        |row| {
                            Ok(FocusContext {
                                subject_id: row.get(0)?,
                                subject_code: row.get(1)?,
                                topic_id: row.get(2)?,
                                topic_name: row.get(3)?,
                            })
                        },
                    )
                    .optional(),
            );
        }

        candidates.push(
            self.conn
                .query_row(
                    "SELECT t.subject_id, s.code, t.id, t.name
             FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             INNER JOIN subjects s ON s.id = t.subject_id
             WHERE sts.student_id = ?1
             ORDER BY sts.priority_score DESC, sts.repair_priority DESC, sts.topic_id ASC
             LIMIT 1",
                    [student_id],
                    |row| {
                        Ok(FocusContext {
                            subject_id: row.get(0)?,
                            subject_code: row.get(1)?,
                            topic_id: row.get(2)?,
                            topic_name: row.get(3)?,
                        })
                    },
                )
                .optional(),
        );

        for candidate in candidates {
            if let Some(context) = candidate.map_err(|err| EcoachError::Storage(err.to_string()))? {
                return Ok(Some(context));
            }
        }

        if let Some(subject_id) = subject_id {
            return self
                .conn
                .query_row(
                    "SELECT t.subject_id, s.code, t.id, t.name
                     FROM topics t
                     INNER JOIN subjects s ON s.id = t.subject_id
                     WHERE t.subject_id = ?1
                     ORDER BY t.id ASC
                     LIMIT 1",
                    [subject_id],
                    |row| {
                        Ok(FocusContext {
                            subject_id: row.get(0)?,
                            subject_code: row.get(1)?,
                            topic_id: row.get(2)?,
                            topic_name: row.get(3)?,
                        })
                    },
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()));
        }

        Ok(None)
    }

    fn load_focus_context_by_topic(&self, topic_id: i64) -> EcoachResult<Option<FocusContext>> {
        self.conn
            .query_row(
                "SELECT t.subject_id, s.code, t.id, t.name
                 FROM topics t
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE t.id = ?1",
                [topic_id],
                |row| {
                    Ok(FocusContext {
                        subject_id: row.get(0)?,
                        subject_code: row.get(1)?,
                        topic_id: row.get(2)?,
                        topic_name: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_concept_rank(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Vec<TopicConceptRank>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    an.id,
                    an.canonical_title,
                    an.node_type,
                    COALESCE(an.foundation_weight, 5000),
                    COALESCE(an.exam_relevance_score, 5000),
                    COALESCE(sss.mastery_score, 0),
                    COALESCE(sss.stability_score, 5000),
                    COALESCE(sss.transfer_strength, 0),
                    COALESCE(sss.pressure_tolerance, 5000),
                    COALESCE(sss.node_status, COALESCE(sss.state, 'unseen'))
                 FROM academic_nodes an
                 LEFT JOIN student_skill_states sss
                    ON sss.node_id = an.id AND sss.student_id = ?1
                 WHERE an.topic_id = ?2
                 ORDER BY COALESCE(an.foundation_weight, 5000) DESC,
                          COALESCE(an.exam_relevance_score, 5000) DESC,
                          an.id ASC
                 LIMIT 8",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, topic_id], |row| {
                Ok(TopicConceptRank {
                    node_id: row.get(0)?,
                    title: row.get(1)?,
                    node_type: row.get(2)?,
                    foundation_weight: clamp_bp(row.get::<_, i64>(3)?),
                    exam_relevance_score: clamp_bp(row.get::<_, i64>(4)?),
                    mastery_score: clamp_bp(row.get::<_, i64>(5)?),
                    stability_score: clamp_bp(row.get::<_, i64>(6)?),
                    transfer_strength: clamp_bp(row.get::<_, i64>(7)?),
                    pressure_tolerance: clamp_bp(row.get::<_, i64>(8)?),
                    status: row.get(9)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut concepts = Vec::new();
        for row in rows {
            concepts.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(concepts)
    }

    fn derive_teaching_modes(
        &self,
        topic_case: &TopicCase,
        concept_rank: &[TopicConceptRank],
        reentry_probe: &crate::ReentryProbeResult,
    ) -> Vec<String> {
        let mut modes = Vec::new();
        let primary = topic_case.primary_hypothesis_code.as_str();
        if primary.contains("misconception")
            || topic_case
                .recommended_intervention
                .mode
                .contains("misconception")
        {
            modes.push("misconception_repair".to_string());
        }
        if primary.contains("foundation")
            || primary.contains("knowledge")
            || !reentry_probe.blocking_topic_ids.is_empty()
        {
            modes.push("prerequisite_rollback".to_string());
        }
        if topic_case.pressure_collapse_index >= 5200 {
            modes.push("speed_conditioning".to_string());
        }
        if topic_case.memory_strength <= 4200 || topic_case.decay_risk >= 5200 {
            modes.push("retrieval_strengthening".to_string());
        }
        if topic_case
            .proof_gaps
            .iter()
            .any(|item| item.to_lowercase().contains("timed"))
            || concept_rank
                .iter()
                .any(|item| item.transfer_strength < 4500)
        {
            modes.push("transfer_training".to_string());
        }
        if topic_case
            .active_hypotheses
            .iter()
            .any(|item| item.code.contains("confusion") || item.code.contains("boundary"))
        {
            modes.push("contrast_training".to_string());
        }
        if topic_case.gap_score >= 6000 {
            modes.push("concept_rebuild".to_string());
        }
        if topic_case.fragility_score >= 5500 {
            modes.push("confidence_repair".to_string());
        }
        if topic_case.pressure_collapse_index >= 6000
            && topic_case.recent_attempt_count >= 4
            && topic_case.recent_accuracy.unwrap_or(0) < 5000
        {
            modes.push("endurance_training".to_string());
        }
        if modes.is_empty() {
            modes.push("evidence_guided_practice".to_string());
        }
        modes.sort();
        modes.dedup();
        modes
    }

    fn derive_fallback_routes(
        &self,
        topic_case: &TopicCase,
        reentry_probe: &crate::ReentryProbeResult,
    ) -> Vec<String> {
        let mut routes = Vec::new();
        if !reentry_probe.blocking_topic_ids.is_empty() {
            routes.push("roll_back_to_prerequisites_then_retest".to_string());
        }
        if topic_case.pressure_collapse_index >= 5200 {
            routes.push("switch_to_calm_then_timed_bursts".to_string());
        }
        if topic_case
            .active_hypotheses
            .iter()
            .any(|item| item.code.contains("misconception"))
        {
            routes.push("contrast_examples_then_micro_explanations".to_string());
        }
        if topic_case.requires_probe {
            routes.push("collect_best_next_evidence_before_scaling".to_string());
        }
        if routes.is_empty() {
            routes.push("stay_on_route_with_validation_checks".to_string());
        }
        routes
    }

    fn sync_intervention_effectiveness(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<InterventionEffectivenessProfile>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    wi.intervention_type,
                    COUNT(*) AS times_used,
                    COALESCE(CAST(AVG(CASE wi.status
                        WHEN 'completed' THEN 10000
                        WHEN 'started' THEN 5000
                        WHEN 'assigned' THEN 3500
                        WHEN 'failed' THEN 1200
                        ELSE 1800
                    END) AS INTEGER), 0) AS success_rate_score,
                    COALESCE(CAST(AVG(COALESCE(wi.outcome_mastery_delta, 0)) AS INTEGER), 0) AS avg_gain_score
                 FROM wrong_answer_interventions wi
                 INNER JOIN wrong_answer_diagnoses wad ON wad.id = wi.diagnosis_id
                 WHERE wi.student_id = ?1
                   AND (?2 IS NULL OR wad.topic_id = ?2)
                 GROUP BY wi.intervention_type
                 ORDER BY success_rate_score DESC, times_used DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, topic_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut profiles = Vec::new();
        for row in rows {
            let (intervention_family, times_used, success_rate_score, avg_gain_score) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let last_outcome = if success_rate_score >= 7000 && avg_gain_score >= 0 {
                "effective"
            } else if success_rate_score >= 4500 {
                "mixed"
            } else {
                "needs_revision"
            }
            .to_string();
            let recommendation = match last_outcome.as_str() {
                "effective" => "Reuse this intervention family when the same cause family appears.",
                "mixed" => "Use this only with a validation probe before scaling.",
                _ => "Do not repeat this blindly; reopen the case first.",
            }
            .to_string();
            let topic_key = topic_id.unwrap_or(0);

            self.conn
                .execute(
                    "INSERT INTO coach_intervention_effectiveness (
                        student_id, topic_key, topic_id, intervention_family,
                        times_used, success_rate_score, avg_gain_score, last_outcome, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
                     ON CONFLICT(student_id, topic_key, intervention_family) DO UPDATE SET
                        times_used = excluded.times_used,
                        success_rate_score = excluded.success_rate_score,
                        avg_gain_score = excluded.avg_gain_score,
                        last_outcome = excluded.last_outcome,
                        updated_at = datetime('now')",
                    params![
                        student_id,
                        topic_key,
                        topic_id,
                        intervention_family,
                        times_used,
                        clamp_bp(success_rate_score),
                        avg_gain_score,
                        last_outcome,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            profiles.push(InterventionEffectivenessProfile {
                topic_id,
                intervention_family,
                times_used,
                success_rate_score: clamp_bp(success_rate_score),
                avg_gain_score,
                last_outcome,
                recommendation,
            });
        }

        if profiles.is_empty() {
            if let Some(topic_id) = topic_id {
                if let Ok(topic_case) = build_topic_case(self.conn, student_id, topic_id) {
                    let intervention_family = topic_case.recommended_intervention.mode.clone();
                    self.conn
                        .execute(
                            "INSERT INTO coach_intervention_effectiveness (
                                student_id, topic_key, topic_id, intervention_family,
                                times_used, success_rate_score, avg_gain_score, last_outcome, updated_at
                             ) VALUES (?1, ?2, ?3, ?4, 0, 5000, 0, 'not_tried', datetime('now'))
                             ON CONFLICT(student_id, topic_key, intervention_family) DO UPDATE SET
                                success_rate_score = excluded.success_rate_score,
                                avg_gain_score = excluded.avg_gain_score,
                                last_outcome = excluded.last_outcome,
                                updated_at = datetime('now')",
                            params![student_id, topic_id, topic_id, intervention_family],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    profiles.push(InterventionEffectivenessProfile {
                        topic_id: Some(topic_id),
                        intervention_family,
                        times_used: 0,
                        success_rate_score: 5000,
                        avg_gain_score: 0,
                        last_outcome: "not_tried".to_string(),
                        recommendation: "Use this as the current best-fit intervention family, but validate it quickly."
                            .to_string(),
                    });
                }
            }
        }

        Ok(profiles)
    }

    fn load_topic_aggregate(&self, student_id: i64) -> EcoachResult<TopicAggregate> {
        self.conn
            .query_row(
                "SELECT
                    COALESCE(CAST(AVG(mastery_score) AS INTEGER), 0),
                    COALESCE(CAST(AVG(accuracy_score) AS INTEGER), 0),
                    COALESCE(CAST(AVG(transfer_score) AS INTEGER), 0),
                    COALESCE(CAST(AVG(consistency_score) AS INTEGER), 0),
                    COALESCE(CAST(AVG(memory_strength) AS INTEGER), 0),
                    COALESCE(CAST(AVG(decay_risk) AS INTEGER), 0),
                    COALESCE(CAST(AVG(pressure_collapse_index) AS INTEGER), 0),
                    COALESCE(CAST(AVG(recognition_strength) AS INTEGER), 0),
                    COALESCE(CAST(AVG(COALESCE(disguised_form_accuracy_bp, mastery_score)) AS INTEGER), 0),
                    COUNT(*)
                 FROM student_topic_states
                 WHERE student_id = ?1",
                [student_id],
                |row| {
                    Ok(TopicAggregate {
                        avg_mastery: row.get(0)?,
                        avg_accuracy: row.get(1)?,
                        avg_transfer: row.get(2)?,
                        avg_consistency: row.get(3)?,
                        avg_memory_strength: row.get(4)?,
                        avg_decay_risk: row.get(5)?,
                        avg_pressure: row.get(6)?,
                        avg_recognition: row.get(7)?,
                        avg_disguised_accuracy: row.get(8)?,
                        topic_count: row.get(9)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_evidence_aggregate(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<EvidenceAggregate> {
        self.conn
            .query_row(
                "SELECT
                    COUNT(*),
                    COALESCE(CAST(AVG(transfer_delta) AS INTEGER), 0),
                    COALESCE(CAST(AVG(timed_delta) AS INTEGER), 0),
                    COALESCE(SUM(CASE WHEN hypothesis_result = 'challenged' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN hypothesis_result = 'confirmed' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN a.was_mixed_context = 1 AND a.is_correct = 0 THEN 1 ELSE 0 END), 0)
                 FROM evidence_events ee
                 LEFT JOIN student_question_attempts a ON a.id = ee.attempt_id
                 WHERE ee.student_id = ?1 AND ee.topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok(EvidenceAggregate {
                        event_count: row.get(0)?,
                        average_transfer_delta: row.get(1)?,
                        average_timed_delta: row.get(2)?,
                        challenged_count: row.get(3)?,
                        confirmed_count: row.get(4)?,
                        mixed_context_failures: row.get(5)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_topic_transfer_signal(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT CAST(AVG(COALESCE(transfer_score, 0)) AS INTEGER)
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_neighbor_signals(
        &self,
        student_id: i64,
        focus: &FocusContext,
        limit: usize,
    ) -> EcoachResult<Vec<NeighborSignal>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    sts.topic_id,
                    t.name,
                    sts.mastery_score,
                    sts.fragility_score,
                    sts.pressure_collapse_index,
                    COALESCE(shared.shared_diagnosis_count, 0),
                    COALESCE(prereq.prerequisite_strength, 0)
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 LEFT JOIN (
                    SELECT wad2.topic_id, COUNT(*) AS shared_diagnosis_count
                    FROM wrong_answer_diagnoses wad1
                    INNER JOIN wrong_answer_diagnoses wad2
                        ON wad2.student_id = wad1.student_id
                       AND wad2.topic_id <> wad1.topic_id
                       AND wad2.primary_diagnosis = wad1.primary_diagnosis
                    WHERE wad1.student_id = ?1 AND wad1.topic_id = ?2
                    GROUP BY wad2.topic_id
                 ) shared ON shared.topic_id = sts.topic_id
                 LEFT JOIN (
                    SELECT an_to.topic_id AS topic_id, MAX(ne.strength_score) AS prerequisite_strength
                    FROM node_edges ne
                    INNER JOIN academic_nodes an_from ON an_from.id = ne.from_node_id
                    INNER JOIN academic_nodes an_to ON an_to.id = ne.to_node_id
                    WHERE an_from.topic_id = ?2
                      AND an_to.topic_id <> an_from.topic_id
                      AND ne.edge_type IN ('prerequisite', 'soft_prerequisite', 'depends_on', 'related')
                    GROUP BY an_to.topic_id
                 ) prereq ON prereq.topic_id = sts.topic_id
                 WHERE sts.student_id = ?1
                   AND sts.topic_id <> ?2
                   AND t.subject_id = ?3
                 ORDER BY
                    COALESCE(shared.shared_diagnosis_count, 0) DESC,
                    COALESCE(prereq.prerequisite_strength, 0) DESC,
                    sts.fragility_score DESC,
                    sts.priority_score DESC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(
                params![student_id, focus.topic_id, focus.subject_id, limit as i64],
                |row| {
                    Ok(NeighborSignal {
                        topic_id: row.get(0)?,
                        topic_name: row.get(1)?,
                        mastery_score: row.get(2)?,
                        fragility_score: row.get(3)?,
                        pressure_collapse_index: row.get(4)?,
                        shared_diagnosis_count: row.get(5)?,
                        prerequisite_strength: row.get(6)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut neighbors = Vec::new();
        for row in rows {
            neighbors.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(neighbors)
    }

    fn upsert_interference_case(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_a_id: i64,
        topic_b_id: i64,
        interference_type: &str,
        severity_score: BasisPoints,
        confidence_score: BasisPoints,
        response_mode: &str,
        regression_audit_due: bool,
        evidence_summary: &str,
    ) -> EcoachResult<()> {
        let (graph_a, graph_b) = if topic_a_id <= topic_b_id {
            (topic_a_id, topic_b_id)
        } else {
            (topic_b_id, topic_a_id)
        };
        self.conn
            .execute(
                "INSERT INTO concept_interference_graph (
                    subject_id, topic_a_id, topic_b_id, edge_type,
                    confusion_risk_score, pressure_amplifier_score, spacing_recommendation_days,
                    best_response_mode, evidence_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
                 ON CONFLICT(subject_id, topic_a_id, topic_b_id, edge_type) DO UPDATE SET
                    confusion_risk_score = excluded.confusion_risk_score,
                    pressure_amplifier_score = excluded.pressure_amplifier_score,
                    spacing_recommendation_days = excluded.spacing_recommendation_days,
                    best_response_mode = excluded.best_response_mode,
                    evidence_json = excluded.evidence_json,
                    updated_at = datetime('now')",
                params![
                    subject_id,
                    graph_a,
                    graph_b,
                    interference_type,
                    severity_score,
                    severity_score,
                    if regression_audit_due { 3 } else { 1 },
                    response_mode,
                    serde_json::to_string(&json!({ "summary": evidence_summary }))
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO learner_interference_cases (
                    student_id, subject_id, topic_a_id, topic_b_id, interference_type,
                    severity_score, confidence_score, response_mode, regression_audit_due,
                    evidence_summary, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
                 ON CONFLICT(student_id, topic_a_id, topic_b_id, interference_type) DO UPDATE SET
                    severity_score = excluded.severity_score,
                    confidence_score = excluded.confidence_score,
                    response_mode = excluded.response_mode,
                    regression_audit_due = excluded.regression_audit_due,
                    evidence_summary = excluded.evidence_summary,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    topic_a_id,
                    topic_b_id,
                    interference_type,
                    severity_score,
                    confidence_score,
                    response_mode,
                    if regression_audit_due { 1 } else { 0 },
                    evidence_summary,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn find_supporting_questions(
        &self,
        subject_id: i64,
        topic_ids: &[i64],
        timed_bias: bool,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        if topic_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = topic_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT id
             FROM questions
             WHERE subject_id = ?
               AND topic_id IN ({})
               AND is_active = 1
             ORDER BY
                CASE
                    WHEN ? = 1 AND estimated_time_seconds <= 45 THEN 0
                    WHEN ? = 0 THEN 0
                    ELSE 1
                END,
                difficulty_level DESC,
                id ASC
             LIMIT ?",
            placeholders
        );
        let mut values: Vec<rusqlite::types::Value> = Vec::with_capacity(topic_ids.len() + 4);
        values.push(subject_id.into());
        for topic_id in topic_ids {
            values.push((*topic_id).into());
        }
        values.push((if timed_bias { 1 } else { 0 }).into());
        values.push((if timed_bias { 1 } else { 0 }).into());
        values.push((limit.max(1) as i64).into());

        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(values.iter()), |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut question_ids = Vec::new();
        for row in rows {
            question_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(question_ids)
    }

    fn find_transfer_questions(
        &self,
        subject_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id
                 FROM questions
                 WHERE subject_id = ?1
                   AND topic_id = ?2
                   AND is_active = 1
                 ORDER BY
                    CASE
                        WHEN question_format IN ('word_problem', 'structured', 'essay', 'explain') THEN 0
                        WHEN COALESCE(primary_content_type, '') IN ('application', 'interpretation') THEN 0
                        ELSE 1
                    END,
                    difficulty_level DESC,
                    id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![subject_id, topic_id, limit.max(1) as i64], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut question_ids = Vec::new();
        for row in rows {
            question_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        if question_ids.is_empty() {
            return self.find_supporting_questions(subject_id, &[topic_id], false, limit);
        }
        Ok(question_ids)
    }

    fn compute_plan_credibility(
        &self,
        plan: Option<&crate::CoachRoadmapSnapshot>,
        topic_aggregate: &TopicAggregate,
    ) -> BasisPoints {
        let Some(plan) = plan else {
            return 4200;
        };
        let readiness_alignment = 10_000
            - (plan.target_readiness_score as i64 - plan.current_readiness_score as i64)
                .abs()
                .min(10_000);
        let completion = plan.weekly_completion_bp as i64;
        let phase_alignment = match plan.current_phase.as_str() {
            "build" | "stabilize" | "strengthen" | "performance" => 7600,
            _ => 6200,
        };
        clamp_bp(
            ((readiness_alignment * 45 / 100)
                + (completion * 35 / 100)
                + (phase_alignment * 20 / 100)
                - ((10_000 - topic_aggregate.avg_mastery) * 10 / 100))
                .clamp(0, 10_000),
        )
    }

    fn load_confidence_integrity(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
    ) -> EcoachResult<BasisPoints> {
        self.conn
            .query_row(
                "SELECT COALESCE(CAST(AVG(confidence_reliability_bp) AS INTEGER), 5000)
                 FROM student_confidence_profile
                 WHERE student_id = ?1
                   AND (?2 IS NULL OR subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get::<_, i64>(0),
            )
            .map(clamp_bp)
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_motivation_score(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
    ) -> EcoachResult<BasisPoints> {
        if let Some(subject_id) = subject_id {
            if let Ok(consistency) = JourneyAdaptationEngine::new(self.conn)
                .get_consistency_snapshot(student_id, subject_id)
            {
                return Ok(clamp_bp(
                    ((consistency.avg_accuracy_bp as i64 * 35 / 100)
                        + ((consistency.study_days_last_14 * 500).min(5000) * 25 / 100)
                        + ((consistency.streak_days * 700).min(5000) * 20 / 100)
                        + ((consistency.avg_daily_minutes_last_14 * 40).min(5000) * 20 / 100))
                        .clamp(0, 10_000),
                ));
            }
        }

        self.conn
            .query_row(
                "SELECT
                    COALESCE(COUNT(DISTINCT date(COALESCE(completed_at, last_activity_at, started_at, created_at))), 0),
                    COALESCE(CAST(AVG(COALESCE(active_study_time_ms, 0) / 60000) AS INTEGER), 0)
                 FROM sessions
                 WHERE student_id = ?1
                   AND date(COALESCE(completed_at, last_activity_at, started_at, created_at)) >= date('now', '-14 day')",
                [student_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map(|(days, minutes)| clamp_bp(((days * 550) + (minutes * 55)).clamp(0, 10_000)))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn compute_quality_tension(&self, topic_id: i64) -> EcoachResult<BasisPoints> {
        let (question_count, node_count, explanation_count): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE((SELECT COUNT(*) FROM questions WHERE topic_id = ?1 AND is_active = 1), 0),
                    COALESCE((SELECT COUNT(*) FROM academic_nodes WHERE topic_id = ?1), 0),
                    COALESCE((
                        SELECT COUNT(*)
                        FROM teach_explanations te
                        INNER JOIN academic_nodes an ON an.id = te.node_id
                        WHERE an.topic_id = ?1
                    ), 0)",
                [topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let weakness_penalty = if question_count < 4 { 2600 } else { 800 }
            + if node_count < 4 { 2200 } else { 700 }
            + if explanation_count == 0 { 1800 } else { 500 };
        Ok(clamp_bp(weakness_penalty))
    }

    fn maybe_record_reflections(
        &self,
        student_id: i64,
        topic_strategy: Option<&TopicTeachingStrategy>,
        uncertainty_profile: Option<&UncertaintyProfile>,
        interference_cases: &[ConceptInterferenceCase],
        intervention_effectiveness: &[InterventionEffectivenessProfile],
    ) -> EcoachResult<()> {
        let focus_topic_id = topic_strategy
            .map(|strategy| strategy.topic_id)
            .or_else(|| interference_cases.first().map(|item| item.topic_a_id))
            .or_else(|| {
                intervention_effectiveness
                    .iter()
                    .find_map(|item| item.topic_id)
            });
        let reflection_count_before = self.count_recent_reflections(student_id, focus_topic_id)?;

        if let Some(strategy) = topic_strategy {
            if let Some(profile) = uncertainty_profile {
                if profile.false_mastery_risk >= 6200 {
                    self.insert_reflection_if_new(
                        student_id,
                        Some(strategy.topic_id),
                        "reopen_case",
                        Some(strategy.strategy_mode.as_str()),
                        Some("false_mastery_risk"),
                        "Mastery signals look inflated relative to transfer and pressure evidence.",
                        true,
                        Some("Switch to best-next-evidence probes before scaling the plan."),
                    )?;
                }
            }
        }

        for profile in intervention_effectiveness {
            if profile.times_used >= 2 && profile.success_rate_score < 4500 {
                self.insert_reflection_if_new(
                    student_id,
                    profile.topic_id,
                    "strategy_revision",
                    Some(profile.intervention_family.as_str()),
                    Some(profile.last_outcome.as_str()),
                    "A repeated intervention family is not producing reliable gains.",
                    true,
                    Some("Reopen the topic case and swap intervention family."),
                )?;
            }
        }

        if let Some(interference) = interference_cases
            .iter()
            .max_by_key(|item| item.severity_score)
        {
            if interference.severity_score >= 5400 {
                self.insert_reflection_if_new(
                    student_id,
                    Some(interference.topic_a_id),
                    "relational_repair",
                    Some("mixed_progression"),
                    Some(interference.interference_type.as_str()),
                    "Neighboring concepts are destabilizing each other, so the route needs relational repair.",
                    true,
                    Some(interference.response_mode.as_str()),
                )?;
            }
        }

        let reflection_count_after = self.count_recent_reflections(student_id, focus_topic_id)?;
        if reflection_count_after == reflection_count_before {
            if let Some(strategy) = topic_strategy {
                let outcome_signal = uncertainty_profile
                    .map(|profile| {
                        if profile.information_gain_score >= 6200 {
                            "evidence_gap"
                        } else if profile.uncertainty_score >= 5200 {
                            "uncertainty_watch"
                        } else if profile.false_mastery_risk >= 5000 {
                            "false_mastery_watch"
                        } else {
                            "campaign_checkpoint"
                        }
                    })
                    .unwrap_or("campaign_checkpoint");
                let follow_up_action = uncertainty_profile
                    .map(|profile| {
                        if profile.information_gain_score >= 6200 {
                            "Run the highest-gain probe next, then refresh the teaching route."
                        } else if profile.false_mastery_risk >= 5000 {
                            "Keep the route active, but validate it with a pressure or transfer probe."
                        } else {
                            "Continue the route and compare the next evidence cycle before scaling it."
                        }
                    })
                    .unwrap_or(
                        "Continue the route and compare the next evidence cycle before scaling it.",
                    );
                self.insert_reflection_if_new(
                    student_id,
                    Some(strategy.topic_id),
                    "plan_checkpoint",
                    Some(strategy.strategy_mode.as_str()),
                    Some(outcome_signal),
                    "Coach reviewed the live campaign and preserved an adaptive checkpoint for the next evidence cycle.",
                    false,
                    Some(follow_up_action),
                )?;
            }
        }

        Ok(())
    }

    fn count_recent_reflections(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<i64> {
        let topic_key = topic_id.unwrap_or(0);
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_reflection_cycles
                 WHERE student_id = ?1
                   AND (?2 IS NULL OR topic_key = ?3)
                   AND created_at >= datetime('now', '-1 day')",
                params![student_id, topic_id, topic_key],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn insert_reflection_if_new(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        cycle_stage: &str,
        prior_strategy: Option<&str>,
        outcome_signal: Option<&str>,
        revision_reason: &str,
        reopened_case: bool,
        follow_up_action: Option<&str>,
    ) -> EcoachResult<()> {
        let topic_key = topic_id.unwrap_or(0);
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_reflection_cycles
                 WHERE student_id = ?1
                   AND topic_key = ?2
                   AND cycle_stage = ?3
                   AND revision_reason = ?4
                   AND created_at >= datetime('now', '-1 day')",
                params![student_id, topic_key, cycle_stage, revision_reason],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists > 0 {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO coach_reflection_cycles (
                    student_id, topic_key, topic_id, cycle_stage,
                    prior_strategy, outcome_signal, revision_reason,
                    reopened_case, follow_up_action
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    student_id,
                    topic_key,
                    topic_id,
                    cycle_stage,
                    prior_strategy,
                    outcome_signal,
                    revision_reason,
                    if reopened_case { 1 } else { 0 },
                    follow_up_action,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn list_recent_reflection_cycles(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<CoachReflectionCycle>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id, cycle_stage, prior_strategy, outcome_signal,
                        revision_reason, reopened_case, follow_up_action, created_at
                 FROM coach_reflection_cycles
                 WHERE student_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, limit.max(1) as i64], |row| {
                Ok(CoachReflectionCycle {
                    topic_id: row.get(0)?,
                    cycle_stage: row.get(1)?,
                    prior_strategy: row.get(2)?,
                    outcome_signal: row.get(3)?,
                    revision_reason: row.get(4)?,
                    reopened_case: row.get::<_, i64>(5)? == 1,
                    follow_up_action: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn upsert_goal_register(&self, student_id: i64, goal: &GoalRegisterItem) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO coach_intent_goal_register (
                    student_id, goal_code, goal_label, target_state,
                    current_score, target_score, tension_score, evidence_confidence,
                    urgency_rank, priority_rank, status, details_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
                 ON CONFLICT(student_id, goal_code) DO UPDATE SET
                    goal_label = excluded.goal_label,
                    target_state = excluded.target_state,
                    current_score = excluded.current_score,
                    target_score = excluded.target_score,
                    tension_score = excluded.tension_score,
                    evidence_confidence = excluded.evidence_confidence,
                    urgency_rank = excluded.urgency_rank,
                    priority_rank = excluded.priority_rank,
                    status = excluded.status,
                    details_json = excluded.details_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    goal.goal_code,
                    goal.goal_label,
                    goal.target_state,
                    goal.current_score,
                    goal.target_score,
                    goal.tension_score,
                    goal.evidence_confidence,
                    goal.urgency_rank,
                    goal.priority_rank,
                    goal.status,
                    serde_json::to_string(&goal.details)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn upsert_tension(&self, student_id: i64, tension: &TensionSignal) -> EcoachResult<()> {
        let topic_key = tension.topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_tension_map (
                    student_id, tension_code, tension_label, topic_key, topic_id,
                    severity_score, desired_state, status, evidence_summary,
                    recommended_response, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
                 ON CONFLICT(student_id, tension_code, topic_key) DO UPDATE SET
                    tension_label = excluded.tension_label,
                    topic_id = excluded.topic_id,
                    severity_score = excluded.severity_score,
                    desired_state = excluded.desired_state,
                    status = excluded.status,
                    evidence_summary = excluded.evidence_summary,
                    recommended_response = excluded.recommended_response,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    tension.tension_code,
                    tension.tension_label,
                    topic_key,
                    tension.topic_id,
                    tension.severity_score,
                    tension.desired_state,
                    tension.status,
                    tension.evidence_summary,
                    tension.recommended_response,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_surprise_event_if_new(
        &self,
        student_id: i64,
        subject_id: i64,
        event: &SurpriseEventRecommendation,
    ) -> EcoachResult<()> {
        let recent_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_surprise_event_runs
                 WHERE student_id = ?1
                   AND subject_id = ?2
                   AND topic_id = ?3
                   AND event_code = ?4
                   AND created_at >= datetime('now', '-1 day')",
                params![student_id, subject_id, event.topic_id, event.event_code],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if recent_count > 0 {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO coach_surprise_event_runs (
                    student_id, subject_id, topic_id, event_code, event_label,
                    purpose, readiness_state, resilience_score, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    student_id,
                    subject_id,
                    event.topic_id,
                    event.event_code,
                    event.event_label,
                    event.purpose,
                    event.readiness_state,
                    event.resilience_score,
                    serde_json::to_string(&json!({
                        "rationale": event.rationale,
                        "question_ids": event.question_ids,
                        "estimated_minutes": event.estimated_minutes,
                    }))
                    .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn goal_item(
        &self,
        goal_code: &str,
        goal_label: &str,
        target_state: &str,
        current_score: BasisPoints,
        target_score: BasisPoints,
        tension_score: BasisPoints,
        evidence_confidence: BasisPoints,
        details: Value,
    ) -> GoalRegisterItem {
        GoalRegisterItem {
            goal_code: goal_code.to_string(),
            goal_label: goal_label.to_string(),
            target_state: target_state.to_string(),
            current_score,
            target_score,
            tension_score,
            evidence_confidence,
            urgency_rank: 0,
            priority_rank: 0,
            status: "watch".to_string(),
            details,
        }
    }

    fn tension_item(
        &self,
        tension_code: &str,
        tension_label: &str,
        topic_id: Option<i64>,
        severity_score: BasisPoints,
        desired_state: &str,
        evidence_summary: String,
        recommended_response: Option<String>,
    ) -> TensionSignal {
        TensionSignal {
            tension_code: tension_code.to_string(),
            tension_label: tension_label.to_string(),
            topic_id,
            severity_score,
            desired_state: desired_state.to_string(),
            status: if severity_score >= 6500 {
                "critical".to_string()
            } else if severity_score >= 4200 {
                "active".to_string()
            } else {
                "watch".to_string()
            },
            evidence_summary,
            recommended_response,
        }
    }

    fn readiness_state_for_topic(
        &self,
        topic_case: &TopicCase,
        false_mastery_risk: BasisPoints,
    ) -> String {
        if false_mastery_risk >= 6500 {
            "false_mastery_risk".to_string()
        } else if topic_case.requires_probe {
            "repair_pending".to_string()
        } else if topic_case.mastery_score >= 7000 && topic_case.fragility_score <= 3000 {
            "stable".to_string()
        } else if topic_case.mastery_score >= 5000 {
            "warming".to_string()
        } else {
            "fragile".to_string()
        }
    }
}

fn doctrine_rules() -> Vec<DoctrineRule> {
    vec![
        DoctrineRule {
            code: "diagnose_before_repeating".to_string(),
            title: "Diagnose Before Repeating".to_string(),
            principle: "A bad result is a symptom, not the full explanation.".to_string(),
            trigger: "When weak performance appears or returns after help.".to_string(),
        },
        DoctrineRule {
            code: "separate_evidence_from_action".to_string(),
            title: "Split Evidence From Action".to_string(),
            principle: "The best next probe is sometimes more valuable than the obvious next task."
                .to_string(),
            trigger: "When uncertainty is still materially high.".to_string(),
        },
        DoctrineRule {
            code: "reopen_the_case".to_string(),
            title: "Reopen The Case".to_string(),
            principle: "When an intervention fails, inspect the diagnosis instead of blaming the learner."
                .to_string(),
            trigger: "When the same topic stalls after prior support.".to_string(),
        },
        DoctrineRule {
            code: "audit_relational_stability".to_string(),
            title: "Audit Relational Stability".to_string(),
            principle: "Never declare mastery from isolated success alone when nearby concepts can interfere."
                .to_string(),
            trigger: "After adjacent or easily confused concepts are taught.".to_string(),
        },
        DoctrineRule {
            code: "pressure_is_a_formal_axis".to_string(),
            title: "Treat Pressure As Its Own Axis".to_string(),
            principle: "Timed collapse is a distinct truth state, not just another wrong answer."
                .to_string(),
            trigger: "When calm and timed performance diverge.".to_string(),
        },
        DoctrineRule {
            code: "learn_the_coach".to_string(),
            title: "Learn The Coach".to_string(),
            principle: "The system should remember which intervention families actually work for this learner."
                .to_string(),
            trigger: "After every completed repair attempt.".to_string(),
        },
    ]
}

fn push_surprise_event(
    events: &mut Vec<SurpriseEventRecommendation>,
    event_code: &str,
    event_label: &str,
    purpose: &str,
    rationale: String,
    estimated_minutes: i64,
    question_ids: Vec<i64>,
    focus: &FocusContext,
    readiness_state: &str,
    resilience_score: BasisPoints,
) {
    events.push(SurpriseEventRecommendation {
        event_code: event_code.to_string(),
        event_label: event_label.to_string(),
        purpose: purpose.to_string(),
        topic_id: focus.topic_id,
        topic_name: focus.topic_name.clone(),
        readiness_state: readiness_state.to_string(),
        resilience_score,
        estimated_minutes,
        rationale,
        question_ids,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;
    use std::path::PathBuf;

    #[test]
    fn intelligence_dome_surfaces_idea23_layers() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        seed_student_state(&conn);

        let service = CoachIntelligenceDomeService::new(&conn);
        let snapshot = service
            .build_intelligence_dome(1, Some(1), None)
            .expect("intelligence dome should build");

        assert!(
            snapshot
                .intent_core
                .goal_register
                .iter()
                .any(|item| item.goal_code == "uncertainty_reduction")
        );
        assert!(snapshot.topic_strategy.is_some());
        assert!(snapshot.uncertainty_profile.is_some());
        assert!(!snapshot.best_next_evidence.is_empty());
        assert!(!snapshot.interference_cases.is_empty());
        assert!(!snapshot.surprise_events.is_empty());
        assert!(!snapshot.intervention_effectiveness.is_empty());
        assert!(!snapshot.reflection_cycles.is_empty());
        let canonical_counts: (i64, i64, i64) = conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM ic_topic_teaching WHERE learner_id = 1),
                    (SELECT COUNT(*) FROM ic_coverage_classification WHERE learner_id = 1),
                    (SELECT COUNT(*) FROM ic_sequencing_decisions WHERE learner_id = 1)",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("canonical idea28 rows should query");
        assert!(canonical_counts.0 >= 1);
        assert!(canonical_counts.1 >= 1);
        assert!(canonical_counts.2 >= 1);
    }

    fn seed_student_state(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (
                id, account_type, display_name, pin_hash, pin_salt, entitlement_tier
             ) VALUES (1, 'student', 'Ama', 'hash', 'salt', 'standard')",
            [],
        )
        .expect("student account should insert");
        conn.execute(
            "INSERT INTO student_profiles (
                account_id, exam_target, exam_target_date, preferred_subjects
             ) VALUES (1, 'BECE', date('now', '+90 day'), '[\"MTH\"]')",
            [],
        )
        .expect("student profile should insert");

        let topics = {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name
                     FROM topics
                     WHERE subject_id = 1
                     ORDER BY id ASC
                     LIMIT 2",
                )
                .expect("topic query should prepare");
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })
                .expect("topic rows should query");
            rows.collect::<Result<Vec<_>, _>>()
                .expect("topics should collect")
        };
        let primary_topic_id = topics[0].0;
        let neighbor_topic_id = topics[1].0;

        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, accuracy_score,
                transfer_score, consistency_score, gap_score, priority_score,
                fragility_score, pressure_collapse_index, total_attempts, correct_attempts,
                recent_attempts_window, recent_correct_window, evidence_count, decay_risk,
                memory_strength, repair_priority, recognition_strength,
                explicit_form_accuracy_bp, disguised_form_accuracy_bp, recognition_gap_bp
             ) VALUES (
                1, ?1, 6900, 'stable', 6200,
                4200, 5400, 3600, 8100,
                6100, 6700, 8, 5,
                4, 2, 8, 5600,
                3900, 7600, 7200,
                7600, 4100, 3500
             )",
            [primary_topic_id],
        )
        .expect("primary topic state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, accuracy_score,
                transfer_score, consistency_score, gap_score, priority_score,
                fragility_score, pressure_collapse_index, total_attempts, correct_attempts,
                recent_attempts_window, recent_correct_window, evidence_count, decay_risk,
                memory_strength, repair_priority, recognition_strength,
                explicit_form_accuracy_bp, disguised_form_accuracy_bp, recognition_gap_bp
             ) VALUES (
                1, ?1, 6100, 'fragile', 5200,
                4300, 5000, 4300, 7200,
                5700, 5900, 7, 4,
                3, 1, 6, 5100,
                4500, 6900, 6800,
                7000, 4300, 3100
             )",
            [neighbor_topic_id],
        )
        .expect("neighbor topic state should insert");

        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (1, ?1, 'fragile', 3900, 5600, datetime('now', '+1 day'))",
            [primary_topic_id],
        )
        .expect("memory state should insert");

        let questions = {
            let mut stmt = conn
                .prepare(
                    "SELECT q.id, q.topic_id,
                            (SELECT qo.id FROM question_options qo
                             WHERE qo.question_id = q.id
                             ORDER BY qo.is_correct ASC, qo.position ASC
                             LIMIT 1) AS wrong_option_id
                     FROM questions q
                     WHERE q.topic_id IN (?1, ?2)
                     ORDER BY q.topic_id ASC, q.id ASC
                     LIMIT 4",
                )
                .expect("question query should prepare");
            let rows = stmt
                .query_map(params![primary_topic_id, neighbor_topic_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                    ))
                })
                .expect("question rows should query");
            rows.collect::<Result<Vec<_>, _>>()
                .expect("questions should collect")
        };

        for (index, (question_id, topic_id, wrong_option_id)) in questions.iter().enumerate() {
            conn.execute(
                "INSERT INTO student_question_attempts (
                    student_id, question_id, attempt_number, started_at, submitted_at,
                    response_time_ms, selected_option_id, is_correct, confidence_level,
                    changed_answer_count, skipped, timed_out, error_type, was_timed,
                    was_transfer_variant, was_retention_check, was_mixed_context, evidence_weight
                 ) VALUES (
                    1, ?1, 1, datetime('now', '-30 minutes'), datetime('now', '-29 minutes'),
                    ?2, ?3, 0, 'sure', 1, 0, 0, 'pressure_breakdown', ?4, ?5, 0, 1, 7000
                 )",
                params![
                    question_id,
                    32_000 + index as i64 * 6_000,
                    wrong_option_id,
                    if *topic_id == primary_topic_id { 1 } else { 0 },
                    if *topic_id == primary_topic_id { 1 } else { 0 },
                ],
            )
            .expect("attempt should insert");
            let attempt_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO evidence_events (
                    attempt_id, student_id, subject_id, topic_id, testing_reason, evidence_weight,
                    mastery_delta, stability_delta, retention_delta, transfer_delta, timed_delta,
                    hypothesis_result, created_at
                 ) VALUES (
                    ?1, 1, 1, ?2, ?3, 7000,
                    -180, -120, -90, ?4, ?5, 'challenged', datetime('now')
                 )",
                params![
                    attempt_id,
                    topic_id,
                    if *topic_id == primary_topic_id {
                        "transfer"
                    } else {
                        "misconception_probe"
                    },
                    if *topic_id == primary_topic_id {
                        -210
                    } else {
                        -80
                    },
                    if *topic_id == primary_topic_id {
                        -240
                    } else {
                        -60
                    },
                ],
            )
            .expect("evidence event should insert");
            conn.execute(
                "INSERT INTO wrong_answer_diagnoses (
                    student_id, question_id, topic_id, misconception_id, error_type,
                    primary_diagnosis, secondary_diagnosis, severity, diagnosis_summary,
                    recommended_action, confidence_score, diagnostic_confidence_score
                 ) VALUES (
                    1, ?1, ?2, NULL, 'pressure_breakdown',
                    'boundary_confusion', 'timed_transfer_collapse', 'high',
                    'Learner is confusing nearby forms and gets worse under time.',
                    'run a contrast and pressure repair loop', 7200, 7200
                 )",
                params![question_id, topic_id],
            )
            .expect("diagnosis should insert");
            let diagnosis_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO wrong_answer_interventions (
                    diagnosis_id, student_id, intervention_type, status,
                    outcome_mastery_delta, outcome_notes, assigned_at, completed_at
                 ) VALUES (
                    ?1, 1, ?2, ?3, ?4, 'backend test seed', datetime('now', '-1 day'), datetime('now')
                 )",
                params![
                    diagnosis_id,
                    if *topic_id == primary_topic_id {
                        "contrast_repair"
                    } else {
                        "prerequisite_repair"
                    },
                    if *topic_id == primary_topic_id {
                        "failed"
                    } else {
                        "completed"
                    },
                    if *topic_id == primary_topic_id { -220 } else { 380 },
                ],
            )
            .expect("intervention should insert");
        }

        let node_ids = {
            let mut stmt = conn
                .prepare(
                    "SELECT id
                     FROM academic_nodes
                     WHERE topic_id = ?1
                     ORDER BY id ASC
                     LIMIT 3",
                )
                .expect("node query should prepare");
            let rows = stmt
                .query_map([primary_topic_id], |row| row.get::<_, i64>(0))
                .expect("node rows should query");
            rows.collect::<Result<Vec<_>, _>>()
                .expect("nodes should collect")
        };
        for (index, node_id) in node_ids.iter().enumerate() {
            conn.execute(
                "INSERT INTO student_skill_states (
                    student_id, node_id, mastery_score, gap_score, priority_score,
                    evidence_count, total_attempts, correct_attempts, state,
                    stability_score, retention_confidence, transfer_strength, timed_strength,
                    coverage_depth, proof_tier, node_status, misconception_risk,
                    recurrence_score, dependency_impact_score, forgetting_risk_score,
                    recognition_strength, reconstruction_strength, reasoning_strength,
                    pressure_tolerance, confidence_calibration
                 ) VALUES (
                    1, ?1, ?2, 4300, 7000,
                    4, 5, 3, 'emerging',
                    4600, 4300, 3900, 3400,
                    4100, 'weak', 'fragile', 5200,
                    3400, 2800, 5100,
                    4700, 4200, 4300,
                    3600, 4100
                 )",
                params![node_id, 5200 - index as i64 * 250],
            )
            .expect("skill state should insert");
        }

        conn.execute(
            "INSERT INTO student_confidence_profile (
                student_id, subject_id, overconfidence_rate_bp, underconfidence_rate_bp,
                guess_rate_bp, calibration_accuracy_bp, confidence_reliability_bp,
                total_responses
             ) VALUES (
                1, 1, 4200, 1800,
                2500, 4300, 3900, 12
             )",
            [],
        )
        .expect("confidence profile should insert");
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
