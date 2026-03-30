use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub eps_score: BasisPoints,
    pub tier: String,
    pub precision_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub depth_score: BasisPoints,
    pub composure_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteTopicProfile {
    pub topic_id: i64,
    pub topic_name: String,
    pub precision_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub depth_score: BasisPoints,
    pub composure_score: BasisPoints,
    pub consistency_score: BasisPoints,
    pub trap_resistance_score: BasisPoints,
    pub domination_score: BasisPoints,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteSessionScore {
    pub session_id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub session_class: String,
    pub accuracy_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub depth_score: BasisPoints,
    pub trap_resistance_score: BasisPoints,
    pub composure_score: BasisPoints,
    pub consistency_score: BasisPoints,
    pub eps_score: BasisPoints,
    pub session_label: String,
    pub debrief_text: String,
    pub recommended_next_session: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteSessionBlueprint {
    pub student_id: i64,
    pub subject_id: i64,
    pub session_class: String,
    pub target_topic_ids: Vec<i64>,
    pub target_family_ids: Vec<i64>,
    pub authoring_modes: Vec<String>,
    pub target_question_count: i64,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintTopicTarget {
    pub topic_id: i64,
    pub topic_name: String,
    pub domination_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub trap_resistance_score: BasisPoints,
    pub status: String,
    pub selection_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintFamilyTarget {
    pub family_id: i64,
    pub family_code: Option<String>,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub health_status: Option<String>,
    pub recurrence_score: BasisPoints,
    pub replacement_score: BasisPoints,
    pub selection_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteTrapBlueprintSignal {
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub confusion_score: BasisPoints,
    pub similarity_trap_bp: BasisPoints,
    pub which_is_which_bp: BasisPoints,
    pub timed_out_count: i64,
    pub force_trapsense: bool,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintReport {
    pub blueprint: EliteSessionBlueprint,
    pub profile: Option<EliteProfile>,
    pub topic_targets: Vec<EliteBlueprintTopicTarget>,
    pub family_targets: Vec<EliteBlueprintFamilyTarget>,
    pub trap_signal: Option<EliteTrapBlueprintSignal>,
}
