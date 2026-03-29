use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineContract {
    pub key: String,
    pub title: String,
    pub purpose: String,
    pub owns_state: Vec<String>,
    pub primary_inputs: Vec<String>,
    pub primary_outputs: Vec<String>,
    pub control_tags: Vec<String>,
    pub offline_required: bool,
    pub modules: Vec<String>,
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
                    &["topic_truth", "question_metadata", "session_evidence"],
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
                        "diagnosis_claims",
                        "mission_memory",
                        "availability",
                    ],
                    &["coach_state", "topic_cases", "missions", "next_actions"],
                    &["decide", "route", "repair"],
                    &["ecoach-coach-brain"],
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
                    &["student_truth", "calendar_events", "today_progress"],
                    &["availability", "free_now_recommendations", "daily_replans"],
                    &["schedule", "rebalance", "trigger"],
                    &["ecoach-goals-calendar"],
                ),
                contract(
                    "session_runtime",
                    "Session Runtime Engine",
                    "Owns practice, mock, custom test, and mission execution state.",
                    &["sessions", "session_items", "runtime_events"],
                    &["coach_missions", "question_sets", "student_actions"],
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
    }
}
