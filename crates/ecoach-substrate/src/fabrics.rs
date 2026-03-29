use crate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
}
