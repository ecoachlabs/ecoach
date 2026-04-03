use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineDecisionContract {
    #[serde(default)]
    pub allowed_decisions: Vec<String>,
    #[serde(default)]
    pub forbidden_decisions: Vec<String>,
    #[serde(default)]
    pub required_outputs: Vec<String>,
    #[serde(default)]
    pub challengeable_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineContract {
    pub key: String,
    pub title: String,
    pub purpose: String,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub domain_family: String,
    #[serde(default)]
    pub authority_class: String,
    #[serde(default)]
    pub implementation_status: String,
    #[serde(default)]
    pub priority_band: String,
    pub owns_state: Vec<String>,
    pub primary_inputs: Vec<String>,
    pub primary_outputs: Vec<String>,
    pub control_tags: Vec<String>,
    pub offline_required: bool,
    pub modules: Vec<String>,
    #[serde(default)]
    pub decision_contract: EngineDecisionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineRegistry {
    pub engines: Vec<EngineContract>,
}

impl EngineRegistry {
    pub fn core_runtime() -> Self {
        Self {
            engines: vec![
                contract(
                    "student_truth",
                    "Learner Truth Engine",
                    "Maintains topic, skill, memory, and error truth for the student.",
                    &[
                        "student_topic_states",
                        "student_skill_states",
                        "memory_states",
                    ],
                    &["answer_submissions", "session_outcomes", "memory_evidence"],
                    &[
                        "topic_truth",
                        "skill_truth",
                        "memory_truth",
                        "error_profiles",
                        "learner_evidence_fabric",
                    ],
                    &["observe", "update_truth", "readiness_input"],
                    &["ecoach-student-model"],
                ),
                contract(
                    "diagnostics",
                    "Diagnostic Battery Engine",
                    "Builds multi-phase probes and turns weak performance into root-cause evidence.",
                    &["diagnostic_instances", "wrong_answer_diagnoses"],
                    &[
                        "topic_truth",
                        "skill_truth",
                        "memory_truth",
                        "question_metadata",
                        "session_evidence",
                        "session_outcomes",
                        "learner_evidence_fabric",
                    ],
                    &[
                        "diagnostic_battery",
                        "diagnosis_claims",
                        "recommended_actions",
                    ],
                    &["diagnose", "probe", "explain"],
                    &["ecoach-diagnostics"],
                ),
                contract(
                    "coach_brain",
                    "Coach Judgment Engine",
                    "Chooses the next coaching move using content readiness, mission memory, and topic cases.",
                    &["coach_plans", "coach_missions", "coach_topic_profiles"],
                    &[
                        "topic_truth",
                        "skill_truth",
                        "memory_truth",
                        "diagnosis_claims",
                        "mission_memory",
                        "mission_memory_inputs",
                        "availability",
                        "session_outcomes",
                        "learner_evidence_fabric",
                    ],
                    &["coach_state", "topic_cases", "missions", "next_actions"],
                    &["decide", "route", "repair"],
                    &["ecoach-coach-brain"],
                ),
                contract(
                    "intent_core",
                    "Academic Intent Core",
                    "Maintains the goal register, tension map, doctrine, and system-health view that governs the coach.",
                    &[
                        "coach_intent_goal_register",
                        "coach_tension_map",
                        "coach_system_health_snapshots",
                        "coach_reflection_cycles",
                    ],
                    &[
                        "topic_cases",
                        "learner_evidence_fabric",
                        "session_outcomes",
                        "coach_plans",
                        "uncertainty_profiles",
                        "interference_cases",
                    ],
                    &[
                        "goal_register",
                        "tension_map",
                        "system_health",
                        "reflection_cycles",
                    ],
                    &["govern", "prioritize", "self_evaluate"],
                    &["ecoach-coach-brain"],
                ),
                contract(
                    "uncertainty_intelligence",
                    "Uncertainty and Evidence Engine",
                    "Separates best-next-evidence from best-next-action and watches for false mastery.",
                    &["coach_uncertainty_profiles"],
                    &[
                        "topic_cases",
                        "evidence_events",
                        "diagnosis_claims",
                        "session_outcomes",
                    ],
                    &[
                        "uncertainty_profiles",
                        "best_next_evidence",
                        "false_mastery_flags",
                    ],
                    &["probe", "rank", "reopen"],
                    &["ecoach-coach-brain", "ecoach-diagnostics"],
                ),
                contract(
                    "interference_intelligence",
                    "Concept Interference Engine",
                    "Detects relational instability between nearby concepts and recommends contrast, separation, or repair.",
                    &["concept_interference_graph", "learner_interference_cases"],
                    &[
                        "topic_cases",
                        "wrong_answer_diagnoses",
                        "evidence_events",
                        "topic_graph",
                    ],
                    &[
                        "interference_cases",
                        "regression_audits",
                        "sequencing_constraints",
                    ],
                    &["contrast", "separate", "repair"],
                    &["ecoach-coach-brain"],
                ),
                contract(
                    "surprise_testing",
                    "Surprise Testing Engine",
                    "Owns the coach's private challenge events for resilience, readiness, and hidden weakness checks.",
                    &["coach_surprise_event_runs"],
                    &[
                        "topic_cases",
                        "uncertainty_profiles",
                        "question_metadata",
                        "interference_cases",
                    ],
                    &["surprise_events", "resilience_signals", "readiness_state"],
                    &["interrupt", "challenge", "stress_test"],
                    &["ecoach-coach-brain", "ecoach-sessions"],
                ),
                contract(
                    "time_orchestration",
                    "Time Orchestration Engine",
                    "Translates availability and daily targets into free-now and replanning decisions.",
                    &[
                        "availability_profiles",
                        "availability_windows",
                        "beat_yesterday_daily_targets",
                    ],
                    &[
                        "student_truth",
                        "memory_truth",
                        "session_outcomes",
                        "calendar_events",
                        "today_progress",
                    ],
                    &["availability", "free_now_recommendations", "daily_replans"],
                    &["schedule", "rebalance", "trigger"],
                    &["ecoach-goals-calendar"],
                ),
                contract(
                    "session_runtime",
                    "Session Runtime Engine",
                    "Owns practice, mock, custom test, and mission execution state.",
                    &["sessions", "session_items", "runtime_events"],
                    &[
                        "coach_missions",
                        "question_sets",
                        "student_actions",
                        "topic_truth",
                        "diagnosis_claims",
                        "learner_evidence_fabric",
                    ],
                    &[
                        "session_outcomes",
                        "runtime_events",
                        "mission_memory_inputs",
                    ],
                    &["run", "measure", "complete"],
                    &["ecoach-sessions"],
                ),
                contract(
                    "content_packs",
                    "Content and Pack Engine",
                    "Installs signed offline content and maps it into runtime tables.",
                    &["content_packs", "curriculum_versions", "knowledge_entries"],
                    &["pack_manifests", "pack_assets"],
                    &["curriculum_nodes", "question_content", "knowledge_links"],
                    &["install", "validate", "publish"],
                    &["ecoach-content", "ecoach-curriculum"],
                ),
                contract(
                    "library",
                    "Library Intelligence Engine",
                    "Turns saved content and weak-topic evidence into shelves, bundles, and revision packs.",
                    &["library_saved_items", "generated_shelves"],
                    &[
                        "student_truth",
                        "learner_evidence_fabric",
                        "knowledge_entries",
                        "question_links",
                    ],
                    &["revision_packs", "continue_learning", "smart_shelves"],
                    &["organize", "bundle", "recommend"],
                    &["ecoach-library"],
                ),
                contract(
                    "glossary",
                    "Glossary Intelligence Engine",
                    "Maps questions and concepts into repairable glossary knowledge.",
                    &["knowledge_entries", "question_glossary_links"],
                    &["question_failures", "concept_queries"],
                    &["glossary_links", "concept_repair_items", "audio_inputs"],
                    &["define", "connect", "repair"],
                    &["ecoach-glossary"],
                ),
                contract(
                    "reporting",
                    "Reporting and Projection Engine",
                    "Projects learner truth into parent/admin-readable summaries and risk views.",
                    &["report_snapshots"],
                    &[
                        "student_truth",
                        "learner_evidence_fabric",
                        "coach_state",
                        "readiness_signals",
                    ],
                    &["parent_reports", "admin_projections"],
                    &["project", "summarize", "explain"],
                    &["ecoach-reporting"],
                ),
                contract(
                    "intake",
                    "Intake and Reconstruction Engine",
                    "Reconstructs uploaded material into structured artifacts and extracted insights.",
                    &["upload_bundles", "extracted_artifacts"],
                    &["pdfs", "images", "scans"],
                    &["insights", "artifact_summaries", "reconstruction_reports"],
                    &["ingest", "extract", "classify"],
                    &["ecoach-intake"],
                ),
            ],
        }
    }

    pub fn constitutional_runtime() -> Self {
        Self {
            engines: vec![
                constitutional_contract(
                    "perception",
                    "evidence",
                    "response_evidence",
                    "Response Evidence Engine",
                    "Ingests learner responses and converts them into normalized evidence packets.",
                    "required",
                    "p0",
                    "active",
                    &["student_question_attempts", "evidence_events"],
                    &[
                        "answer_selected",
                        "typed_response",
                        "session_context",
                        "question_metadata",
                    ],
                    &["normalized_response_event", "response_evidence_packet"],
                    &["ingest", "normalize", "score"],
                    &["ecoach-student-model", "ecoach-sessions"],
                    &["response scoring", "timing normalization"],
                    &["topic sequencing", "mastery certification"],
                    &[
                        "normalized_response_event",
                        "response_evidence_packet",
                        "event_quality_flags",
                    ],
                    &["evidence_normalization", "consistency_validator"],
                ),
                constitutional_contract(
                    "perception",
                    "evidence",
                    "evidence_normalization",
                    "Evidence Normalization Engine",
                    "Weights evidence by freshness, trust, and context before downstream engines consume it.",
                    "required",
                    "p0",
                    "active",
                    &["evidence_events"],
                    &[
                        "response_evidence_packet",
                        "content_signal_packet",
                        "observation_packets",
                    ],
                    &["normalized_evidence", "trust_scores"],
                    &["weight", "dedupe", "route"],
                    &["ecoach-student-model", "ecoach-substrate"],
                    &["evidence weighting", "trust scoring"],
                    &["session composition", "hard policy routing"],
                    &["normalized_evidence", "trust_scores"],
                    &["consistency_validator", "hypothesis_competition"],
                ),
                constitutional_contract(
                    "understanding",
                    "knowledge",
                    "topic_state",
                    "Topic State Engine",
                    "Maintains the durable topic-level learner truth.",
                    "required",
                    "p0",
                    "active",
                    &["student_topic_states"],
                    &["normalized_evidence", "memory_truth", "diagnosis_claims"],
                    &["topic_truth", "topic_fragility"],
                    &["update_truth", "score", "summarize"],
                    &["ecoach-student-model", "ecoach-coach-brain"],
                    &["topic mastery updates", "fragility updates"],
                    &["session layout", "policy overrides"],
                    &["topic_truth", "topic_fragility"],
                    &["confidence_gate", "consistency_validator"],
                ),
                constitutional_contract(
                    "understanding",
                    "diagnostic",
                    "hypothesis_competition",
                    "Hypothesis Competition Engine",
                    "Ranks competing explanations for weakness without collapsing uncertainty too early.",
                    "required",
                    "p0",
                    "active",
                    &["coach_uncertainty_profiles", "wrong_answer_diagnoses"],
                    &[
                        "normalized_evidence",
                        "topic_truth",
                        "learner_signal_packet",
                    ],
                    &["ranked_hypotheses", "uncertainty_profiles"],
                    &["diagnose", "rank", "reopen"],
                    &["ecoach-coach-brain", "ecoach-diagnostics"],
                    &[
                        "ranked possible explanations",
                        "uncertainty level",
                        "need for more probes",
                    ],
                    &[
                        "final session layout",
                        "hard risk policy",
                        "mastery certification",
                    ],
                    &["top hypotheses", "confidence bands", "missing evidence"],
                    &[
                        "diagnostic_experiment",
                        "contradiction_check",
                        "coach_self_evaluation",
                    ],
                ),
                constitutional_contract(
                    "understanding",
                    "diagnostic",
                    "interference_detection",
                    "Interference Detection Engine",
                    "Detects when nearby concepts corrupt one another under variation or pressure.",
                    "required",
                    "p0",
                    "active",
                    &["learner_interference_cases", "concept_interference_graph"],
                    &[
                        "normalized_evidence",
                        "topic_graph",
                        "misconception_instances",
                    ],
                    &["interference_cases", "contrast_constraints"],
                    &["contrast", "separate", "reopen"],
                    &["ecoach-coach-brain"],
                    &["interference case generation", "contrast constraints"],
                    &["final session certification", "content publishing"],
                    &["interference_cases", "contrast_constraints"],
                    &["contradiction_check", "session_composer"],
                ),
                constitutional_contract(
                    "understanding",
                    "learner",
                    "learner_state",
                    "Learner State Engine",
                    "Models pressure, recovery, fatigue, confidence fragility, and engagement drift.",
                    "required",
                    "p0",
                    "fused",
                    &[
                        "engagement_risk_profiles",
                        "student_confidence_profile",
                        "memory_states",
                    ],
                    &[
                        "learner_signal_packet",
                        "session_outcomes",
                        "normalized_evidence",
                    ],
                    &[
                        "learner_state_snapshot",
                        "pressure_profile",
                        "recovery_profile",
                    ],
                    &["profile", "bound", "warn"],
                    &[
                        "ecoach-goals-calendar",
                        "ecoach-student-model",
                        "ecoach-coach-brain",
                    ],
                    &["learner-state updates", "pressure profile updates"],
                    &["mastery certification", "content publishing"],
                    &[
                        "learner_state_snapshot",
                        "pressure_profile",
                        "recovery_profile",
                    ],
                    &["risk_engine", "adaptation_engine", "consistency_validator"],
                ),
                constitutional_contract(
                    "decision",
                    "planning",
                    "teaching_strategy",
                    "Teaching Strategy Engine",
                    "Chooses the instructional family that fits the current diagnosis and learner state.",
                    "required",
                    "p0",
                    "active",
                    &["coach_topic_profiles"],
                    &["ranked_hypotheses", "learner_state_snapshot", "topic_truth"],
                    &["strategy_choice", "mode_fallbacks"],
                    &["choose_mode", "fallback", "explain"],
                    &["ecoach-coach-brain"],
                    &["teaching mode selection", "fallback routing"],
                    &["policy blocking", "final mastery proof"],
                    &["strategy_choice", "mode_fallbacks"],
                    &["risk_engine", "confidence_gate", "coach_self_evaluation"],
                ),
                constitutional_contract(
                    "decision",
                    "risk",
                    "risk_engine",
                    "Risk Engine",
                    "Estimates academic danger and instability across topic, learner, and exam scopes.",
                    "blocking",
                    "p0",
                    "active",
                    &["engagement_risk_profiles", "coach_blockers"],
                    &["topic_truth", "learner_state_snapshot", "timing_plan"],
                    &["risk_profile", "protection_posture"],
                    &["block", "stabilize", "escalate"],
                    &["ecoach-coach-brain", "ecoach-goals-calendar"],
                    &["protection posture", "stabilization demand", "escalation"],
                    &["content publishing", "mastery proof verdict"],
                    &["risk_profile", "protection_posture"],
                    &["coach_self_evaluation", "policy_guardrail"],
                ),
                constitutional_contract(
                    "execution_design",
                    "execution",
                    "session_composer",
                    "Session Composer Engine",
                    "Turns approved decisions into session blocks, probes, and reinforcement arcs.",
                    "required",
                    "p0",
                    "active",
                    &["session_composition"],
                    &[
                        "strategy_choice",
                        "timing_plan",
                        "risk_profile",
                        "topic_truth",
                    ],
                    &["session_plan", "session_blocks"],
                    &["compose", "mix", "sequence"],
                    &["ecoach-coach-brain", "ecoach-sessions"],
                    &["session block composition", "intent mix"],
                    &["risk policy overrides", "mastery proof"],
                    &["session_plan", "session_blocks"],
                    &["risk_engine", "confidence_gate", "policy_guardrail"],
                ),
                constitutional_contract(
                    "execution_design",
                    "content",
                    "content_selection",
                    "Content Selection Engine",
                    "Chooses the best artifact for the current educational job and learner state.",
                    "required",
                    "p0",
                    "active",
                    &["resource_metadata_index", "generated_shelves"],
                    &[
                        "strategy_choice",
                        "topic_truth",
                        "content_quality_flags",
                        "learner_state_snapshot",
                    ],
                    &["content_selection", "artifact_rejections"],
                    &["select", "filter", "prefer"],
                    &["ecoach-content", "ecoach-library"],
                    &["artifact selection", "artifact rejection"],
                    &["final policy overrides", "mastery certification"],
                    &["content_selection", "artifact_rejections"],
                    &["policy_guardrail", "consistency_validator"],
                ),
                constitutional_contract(
                    "execution_design",
                    "proof",
                    "mastery_proof",
                    "Mastery Proof Engine",
                    "Checks whether explain, solve, transfer, recall, and pressure gates are genuinely satisfied.",
                    "constitutional",
                    "p0",
                    "active",
                    &["topic_proof_certifications"],
                    &["topic_truth", "session_outcomes", "normalized_evidence"],
                    &["mastery_contract", "proof_verdict", "retest_obligation"],
                    &["certify", "reopen", "withhold"],
                    &["ecoach-coach-brain"],
                    &["proof verdict", "retest obligation", "review obligation"],
                    &["topic sequencing", "content publishing"],
                    &["mastery_contract", "proof_verdict", "retest_obligation"],
                    &["contradiction_check", "coach_self_evaluation"],
                ),
                constitutional_contract(
                    "learning_memory",
                    "meta_learning",
                    "coach_self_evaluation",
                    "Coach Self-Evaluation Engine",
                    "Judges whether the coach's diagnosis and decisions were actually good.",
                    "constitutional",
                    "p0",
                    "active",
                    &["coach_reflection_cycles", "coach_system_health_snapshots"],
                    &[
                        "intervention_outcomes",
                        "arbitration_verdict",
                        "proof_verdict",
                    ],
                    &["self_evaluation", "system_health"],
                    &["evaluate", "criticize", "reopen"],
                    &["ecoach-coach-brain"],
                    &["coach self-evaluation", "system health updates"],
                    &["raw evidence writes", "session execution"],
                    &["self_evaluation", "system_health"],
                    &["consistency_validator"],
                ),
                constitutional_contract(
                    "governance",
                    "governance",
                    "confidence_gate",
                    "Confidence Gate Engine",
                    "Downgrades or blocks decisions when certainty is too weak for hard action.",
                    "constitutional",
                    "p0",
                    "active",
                    &["coach_governance_checks"],
                    &["uncertainty_profiles", "proof_verdict", "strategy_choice"],
                    &["confidence_gate_verdict"],
                    &["downgrade", "block", "probe_first"],
                    &["ecoach-coach-brain"],
                    &["downgrade", "block", "probe_first"],
                    &["raw evidence writes", "content publishing"],
                    &["confidence_gate_verdict"],
                    &["decision_arbitration", "coach_self_evaluation"],
                ),
                constitutional_contract(
                    "governance",
                    "governance",
                    "decision_arbitration",
                    "Decision Arbitration Engine",
                    "Resolves conflicts between risk, confidence, strategy, and progression engines.",
                    "constitutional",
                    "p0",
                    "active",
                    &["coach_arbitration_records"],
                    &[
                        "risk_profile",
                        "uncertainty_profiles",
                        "strategy_choice",
                        "timing_plan",
                    ],
                    &["arbitration_verdict", "winning_engine"],
                    &["arbitrate", "downgrade", "reopen"],
                    &["ecoach-coach-brain"],
                    &["winning engine selection", "decision hardness downgrade"],
                    &["raw evidence writes", "content publishing"],
                    &["arbitration_verdict", "winning_engine"],
                    &["consistency_validator", "coach_self_evaluation"],
                ),
                constitutional_contract(
                    "governance",
                    "governance",
                    "policy_guardrail",
                    "Policy Guardrail Engine",
                    "Blocks unsafe moves when content, readiness, or doctrine rules are violated.",
                    "blocking",
                    "p0",
                    "active",
                    &["coach_governance_checks"],
                    &["content_quality_flags", "content_readiness", "risk_profile"],
                    &["policy_guardrail_verdict"],
                    &["block", "reroute", "explain"],
                    &["ecoach-coach-brain"],
                    &["policy block", "safe reroute"],
                    &["raw evidence writes", "proof certification"],
                    &["policy_guardrail_verdict"],
                    &["decision_arbitration", "coach_self_evaluation"],
                ),
                constitutional_contract(
                    "governance",
                    "governance",
                    "engine_health_monitor",
                    "Engine Health Monitor",
                    "Surfaces whether core engines have enough evidence and consistency to be trusted.",
                    "advisory",
                    "p1",
                    "active",
                    &["coach_engine_health_snapshots"],
                    &["decision_trace", "uncertainty_profiles", "proof_verdict"],
                    &["engine_health_snapshot"],
                    &["monitor", "warn", "score"],
                    &["ecoach-coach-brain"],
                    &["engine health scoring"],
                    &["policy overrides", "content publishing"],
                    &["engine_health_snapshot"],
                    &["coach_self_evaluation", "consistency_validator"],
                ),
                constitutional_contract(
                    "governance",
                    "governance",
                    "consistency_validator",
                    "Consistency Validator",
                    "Checks that engine outputs agree with one another before the coach commits.",
                    "constitutional",
                    "p0",
                    "active",
                    &["coach_governance_checks"],
                    &[
                        "decision_trace",
                        "strategy_choice",
                        "risk_profile",
                        "proof_verdict",
                    ],
                    &["consistency_verdict"],
                    &["validate", "reopen", "warn"],
                    &["ecoach-coach-brain"],
                    &["consistency verdict", "reopen request"],
                    &["raw evidence writes", "content publishing"],
                    &["consistency_verdict"],
                    &["decision_arbitration", "coach_self_evaluation"],
                ),
            ],
        }
    }

    pub fn find_engine(&self, key: &str) -> Option<&EngineContract> {
        self.engines.iter().find(|engine| engine.key == key)
    }

    pub fn engines_producing(&self, output: &str) -> Vec<&EngineContract> {
        self.engines
            .iter()
            .filter(|engine| engine.primary_outputs.iter().any(|item| item == output))
            .collect()
    }

    pub fn engines_consuming(&self, input: &str) -> Vec<&EngineContract> {
        self.engines
            .iter()
            .filter(|engine| engine.primary_inputs.iter().any(|item| item == input))
            .collect()
    }
}

fn contract(
    key: &str,
    title: &str,
    purpose: &str,
    owns_state: &[&str],
    primary_inputs: &[&str],
    primary_outputs: &[&str],
    control_tags: &[&str],
    modules: &[&str],
) -> EngineContract {
    EngineContract {
        key: key.to_string(),
        title: title.to_string(),
        purpose: purpose.to_string(),
        layer: "runtime".to_string(),
        domain_family: "runtime".to_string(),
        authority_class: "preferred".to_string(),
        implementation_status: "active".to_string(),
        priority_band: "p1".to_string(),
        owns_state: owns_state.iter().map(|value| value.to_string()).collect(),
        primary_inputs: primary_inputs
            .iter()
            .map(|value| value.to_string())
            .collect(),
        primary_outputs: primary_outputs
            .iter()
            .map(|value| value.to_string())
            .collect(),
        control_tags: control_tags.iter().map(|value| value.to_string()).collect(),
        offline_required: true,
        modules: modules.iter().map(|value| value.to_string()).collect(),
        decision_contract: EngineDecisionContract {
            allowed_decisions: control_tags.iter().map(|value| value.to_string()).collect(),
            forbidden_decisions: Vec::new(),
            required_outputs: primary_outputs
                .iter()
                .map(|value| value.to_string())
                .collect(),
            challengeable_by: Vec::new(),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn constitutional_contract(
    layer: &str,
    domain_family: &str,
    key: &str,
    title: &str,
    purpose: &str,
    authority_class: &str,
    priority_band: &str,
    implementation_status: &str,
    owns_state: &[&str],
    primary_inputs: &[&str],
    primary_outputs: &[&str],
    control_tags: &[&str],
    modules: &[&str],
    allowed_decisions: &[&str],
    forbidden_decisions: &[&str],
    required_outputs: &[&str],
    challengeable_by: &[&str],
) -> EngineContract {
    EngineContract {
        key: key.to_string(),
        title: title.to_string(),
        purpose: purpose.to_string(),
        layer: layer.to_string(),
        domain_family: domain_family.to_string(),
        authority_class: authority_class.to_string(),
        implementation_status: implementation_status.to_string(),
        priority_band: priority_band.to_string(),
        owns_state: owns_state.iter().map(|value| value.to_string()).collect(),
        primary_inputs: primary_inputs
            .iter()
            .map(|value| value.to_string())
            .collect(),
        primary_outputs: primary_outputs
            .iter()
            .map(|value| value.to_string())
            .collect(),
        control_tags: control_tags.iter().map(|value| value.to_string()).collect(),
        offline_required: true,
        modules: modules.iter().map(|value| value.to_string()).collect(),
        decision_contract: EngineDecisionContract {
            allowed_decisions: allowed_decisions
                .iter()
                .map(|value| value.to_string())
                .collect(),
            forbidden_decisions: forbidden_decisions
                .iter()
                .map(|value| value.to_string())
                .collect(),
            required_outputs: required_outputs
                .iter()
                .map(|value| value.to_string())
                .collect(),
            challengeable_by: challengeable_by
                .iter()
                .map(|value| value.to_string())
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_runtime_registry_exposes_expected_engine_contracts() {
        let registry = EngineRegistry::core_runtime();

        assert!(registry.find_engine("coach_brain").is_some());
        assert!(registry.find_engine("time_orchestration").is_some());
        assert!(registry.find_engine("student_truth").is_some());
        assert!(!registry.engines_producing("topic_cases").is_empty());
        assert!(!registry.engines_consuming("question_failures").is_empty());
        assert!(
            !registry
                .engines_consuming("learner_evidence_fabric")
                .is_empty()
        );
        assert!(!registry.engines_consuming("session_outcomes").is_empty());
        assert!(!registry.engines_consuming("memory_truth").is_empty());
    }

    #[test]
    fn constitutional_registry_exposes_governance_contracts() {
        let registry = EngineRegistry::constitutional_runtime();

        let confidence_gate = registry
            .find_engine("confidence_gate")
            .expect("confidence gate contract should exist");
        assert_eq!(confidence_gate.layer, "governance");
        assert_eq!(confidence_gate.authority_class, "constitutional");
        assert!(
            confidence_gate
                .decision_contract
                .allowed_decisions
                .iter()
                .any(|item| item == "downgrade")
        );

        let hypothesis = registry
            .find_engine("hypothesis_competition")
            .expect("hypothesis competition contract should exist");
        assert!(
            hypothesis
                .decision_contract
                .challengeable_by
                .iter()
                .any(|item| item == "diagnostic_experiment")
        );
        assert!(registry.find_engine("session_composer").is_some());
        assert!(registry.find_engine("engine_health_monitor").is_some());
    }
}
