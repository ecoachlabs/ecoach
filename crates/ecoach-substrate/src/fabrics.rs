use crate::{BasisPoints, EngineRegistry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricSignal {
    pub engine_key: String,
    pub signal_type: String,
    pub status: Option<String>,
    pub score: Option<BasisPoints>,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub question_id: Option<i64>,
    pub observed_at: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricEvidenceRecord {
    pub stream: String,
    pub reference_id: String,
    pub event_type: String,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub question_id: Option<i64>,
    pub occurred_at: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FabricConsumerTarget {
    pub engine_key: String,
    pub engine_title: String,
    pub matched_inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FabricOrchestrationSummary {
    pub available_inputs: Vec<String>,
    pub consumer_targets: Vec<FabricConsumerTarget>,
}

impl FabricOrchestrationSummary {
    pub fn from_available_inputs(
        registry: &EngineRegistry,
        available_inputs: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut normalized_inputs = available_inputs.into_iter().collect::<Vec<_>>();
        normalized_inputs.sort();
        normalized_inputs.dedup();

        let mut consumers = BTreeMap::<String, FabricConsumerTarget>::new();
        for input in &normalized_inputs {
            for engine in registry.engines_consuming(input) {
                let target =
                    consumers
                        .entry(engine.key.clone())
                        .or_insert_with(|| FabricConsumerTarget {
                            engine_key: engine.key.clone(),
                            engine_title: engine.title.clone(),
                            matched_inputs: Vec::new(),
                        });
                if !target
                    .matched_inputs
                    .iter()
                    .any(|candidate| candidate == input)
                {
                    target.matched_inputs.push(input.clone());
                    target.matched_inputs.sort();
                }
            }
        }

        Self {
            available_inputs: normalized_inputs,
            consumer_targets: consumers.into_values().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerEvidenceFabric {
    pub student_id: i64,
    pub student_name: String,
    pub overall_mastery_score: BasisPoints,
    pub overall_readiness_band: String,
    pub pending_review_count: i64,
    pub due_memory_count: i64,
    pub signals: Vec<FabricSignal>,
    pub evidence_records: Vec<FabricEvidenceRecord>,
    #[serde(default)]
    pub orchestration: FabricOrchestrationSummary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestration_summary_routes_inputs_to_declared_consumers() {
        let registry = EngineRegistry::core_runtime();
        let summary = FabricOrchestrationSummary::from_available_inputs(
            &registry,
            vec![
                "learner_evidence_fabric".to_string(),
                "session_outcomes".to_string(),
                "memory_truth".to_string(),
            ],
        );

        assert!(
            summary
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "coach_brain")
        );
        assert!(
            summary
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "reporting")
        );
    }
}
