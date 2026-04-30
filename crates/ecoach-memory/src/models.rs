use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

// ── Memory state machine ──
// seen → encoded → accessible → { anchoring | fragile }
// anchoring → confirmed → locked_in
// fragile → { fading | rebuilding }
// fading → { collapsed | recovered }
// at_risk can branch from several states

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    Seen,
    Encoded,
    Accessible,
    Fragile,
    Anchoring,
    Confirmed,
    LockedIn,
    AtRisk,
    Fading,
    Rebuilding,
    Recovered,
    Collapsed,
}

impl MemoryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Seen => "seen",
            Self::Encoded => "encoded",
            Self::Accessible => "accessible",
            Self::Fragile => "fragile",
            Self::Anchoring => "anchoring",
            Self::Confirmed => "confirmed",
            Self::LockedIn => "locked_in",
            Self::AtRisk => "at_risk",
            Self::Fading => "fading",
            Self::Rebuilding => "rebuilding",
            Self::Recovered => "recovered",
            Self::Collapsed => "collapsed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "seen" => Some(Self::Seen),
            "encoded" => Some(Self::Encoded),
            "accessible" => Some(Self::Accessible),
            "fragile" => Some(Self::Fragile),
            "anchoring" => Some(Self::Anchoring),
            "confirmed" => Some(Self::Confirmed),
            "locked_in" => Some(Self::LockedIn),
            "at_risk" => Some(Self::AtRisk),
            "fading" => Some(Self::Fading),
            "rebuilding" => Some(Self::Rebuilding),
            "recovered" => Some(Self::Recovered),
            "collapsed" => Some(Self::Collapsed),
            _ => None,
        }
    }

    pub fn is_healthy(self) -> bool {
        matches!(
            self,
            Self::Accessible | Self::Anchoring | Self::Confirmed | Self::LockedIn | Self::Recovered
        )
    }
}

// ── Recall modes ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallMode {
    FreeRecall,
    CuedRecall,
    Recognition,
    Application,
    Transfer,
    Pressure,
}

impl RecallMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreeRecall => "free_recall",
            Self::CuedRecall => "cued_recall",
            Self::Recognition => "recognition",
            Self::Application => "application",
            Self::Transfer => "transfer",
            Self::Pressure => "pressure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CueLevel {
    None,
    Light,
    Heavy,
}

impl CueLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Light => "light",
            Self::Heavy => "heavy",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMemoryEvidenceInput {
    pub student_id: i64,
    pub node_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub recall_mode: RecallMode,
    pub cue_level: CueLevel,
    pub delay_bucket: String,
    pub interference_detected: bool,
    pub was_correct: bool,
    pub confidence_level: Option<String>,
    #[serde(default)]
    pub session_id: Option<i64>,
    #[serde(default)]
    pub question_id: Option<i64>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub timed: bool,
    #[serde(default)]
    pub time_limit_ms: Option<i64>,
    #[serde(default)]
    pub response_time_ms: Option<i64>,
    #[serde(default)]
    pub first_commit_time_ms: Option<i64>,
    #[serde(default)]
    pub raw_score_bp: Option<BasisPoints>,
    #[serde(default)]
    pub confidence_self_report_bp: Option<BasisPoints>,
    #[serde(default)]
    pub hints_used: i64,
    #[serde(default)]
    pub hint_strength_bp: Option<BasisPoints>,
    #[serde(default)]
    pub options_visible: bool,
    #[serde(default)]
    pub formula_bank_visible: bool,
    #[serde(default)]
    pub answer_text: Option<String>,
    #[serde(default)]
    pub expected_node_id: Option<i64>,
    #[serde(default)]
    pub intruding_node_id: Option<i64>,
    #[serde(default)]
    pub switched_answer: bool,
    #[serde(default)]
    pub guess_likelihood_bp: Option<BasisPoints>,
    #[serde(default)]
    pub freeze_marker: bool,
    #[serde(default)]
    pub hesitation_score_bp: Option<BasisPoints>,
    #[serde(default)]
    pub derived_tags: Vec<String>,
    #[serde(default)]
    pub attempt_key: Option<String>,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateRecord {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub memory_state: String,
    pub memory_strength: BasisPoints,
    pub recall_fluency: BasisPoints,
    pub decay_risk: BasisPoints,
    pub review_due_at: Option<String>,
    pub last_recalled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecheckItem {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub due_at: String,
    pub schedule_type: String,
    pub status: String,
    pub memory_state: Option<String>,
    pub memory_strength: Option<BasisPoints>,
    pub decay_risk: Option<BasisPoints>,
    pub topic_name: Option<String>,
    pub node_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReviewQueueItem {
    pub memory_state_id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub node_id: Option<i64>,
    pub node_title: Option<String>,
    pub memory_state: String,
    pub schedule_type: String,
    pub action_type: String,
    pub priority_score: BasisPoints,
    pub memory_strength: BasisPoints,
    pub decay_risk: BasisPoints,
    pub due_at: Option<String>,
    pub interference_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMemorySummary {
    pub topic_id: i64,
    pub topic_name: String,
    pub total_items: i64,
    pub healthy_items: i64,
    pub fragile_items: i64,
    pub collapsed_items: i64,
    pub overdue_reviews: i64,
    pub average_strength: BasisPoints,
    pub next_review_due: Option<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReturnSession {
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub action_type: String,
    pub urgency_band: String,
    pub estimated_minutes: i64,
    pub due_count: i64,
    pub fragile_count: i64,
    pub collapsed_count: i64,
    pub item_ids: Vec<i64>,
    pub node_ids: Vec<i64>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReturnLoop {
    pub student_id: i64,
    pub total_due_items: i64,
    pub total_topics_in_play: i64,
    pub recommended_today_minutes: i64,
    pub dominant_mode: String,
    pub next_review_due: Option<String>,
    pub sessions: Vec<MemoryReturnSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayBatchResult {
    pub items_processed: usize,
    pub items_decayed: usize,
    pub items_collapsed: usize,
    pub new_rechecks_scheduled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceEdge {
    pub id: i64,
    pub from_node_id: i64,
    pub to_node_id: i64,
    pub strength_score: BasisPoints,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDashboard {
    pub student_id: i64,
    pub total_items: i64,
    pub healthy_count: i64,
    pub at_risk_count: i64,
    pub fading_count: i64,
    pub collapsed_count: i64,
    pub overdue_reviews: i64,
    pub average_strength: BasisPoints,
    pub next_review_due: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecallProfile {
    pub recognition: BasisPoints,
    pub cued_recall: BasisPoints,
    pub free_recall: BasisPoints,
    pub application: BasisPoints,
    pub transfer: BasisPoints,
    pub pressure: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUnitRecord {
    pub id: i64,
    pub node_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub subtopic_id: Option<i64>,
    pub title: String,
    pub canonical_label: String,
    pub description: Option<String>,
    pub unit_type: String,
    pub difficulty_bp: BasisPoints,
    pub importance_weight_bp: BasisPoints,
    pub dependency_weight_bp: BasisPoints,
    pub confusion_proneness_bp: BasisPoints,
    pub exam_frequency_weight_bp: BasisPoints,
    pub canonical_representations_json: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUnitEdgeRecord {
    pub id: i64,
    pub source_unit_id: i64,
    pub target_unit_id: i64,
    pub edge_type: String,
    pub weight_bp: BasisPoints,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryExplainability {
    pub primary_driver: String,
    pub secondary_driver: Option<String>,
    pub feature_summary_json: String,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentKnowledgeStateRecord {
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub node_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub memory_state: String,
    pub state_confidence_bp: BasisPoints,
    pub state_updated_at: String,
    pub decay_status: String,
    pub decay_risk_score: BasisPoints,
    pub recall_profile: RecallProfile,
    pub support_dependency_score: BasisPoints,
    pub confidence_calibration_score: BasisPoints,
    pub latency_score: BasisPoints,
    pub resilience_score: BasisPoints,
    pub primary_failure_mode: Option<String>,
    pub secondary_failure_mode: Option<String>,
    pub interference_risk_score: BasisPoints,
    pub downstream_risk_score: BasisPoints,
    pub exposure_count: i64,
    pub attempt_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_seen_at: Option<String>,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_free_recall_success_at: Option<String>,
    pub last_application_success_at: Option<String>,
    pub last_pressure_success_at: Option<String>,
    pub current_intervention_plan_id: Option<i64>,
    pub next_review_at: Option<String>,
    pub review_urgency_score: Option<BasisPoints>,
    pub flags: Vec<String>,
    pub explainability: MemoryExplainability,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalAttemptRecord {
    pub id: i64,
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub session_id: Option<i64>,
    pub question_id: Option<i64>,
    pub mode: String,
    pub format: String,
    pub timed: bool,
    pub time_limit_ms: Option<i64>,
    pub response_time_ms: Option<i64>,
    pub first_commit_time_ms: Option<i64>,
    pub correctness: String,
    pub raw_score_bp: BasisPoints,
    pub confidence_self_report_bp: Option<BasisPoints>,
    pub hints_used: i64,
    pub hint_strength_bp: Option<BasisPoints>,
    pub options_visible: bool,
    pub formula_bank_visible: bool,
    pub answer_text: Option<String>,
    pub expected_node_id: Option<i64>,
    pub intruding_node_id: Option<i64>,
    pub switched_answer: bool,
    pub guess_likelihood_bp: Option<BasisPoints>,
    pub freeze_marker: bool,
    pub hesitation_score_bp: Option<BasisPoints>,
    pub derived_tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentInterferenceEdge {
    pub id: i64,
    pub student_id: Option<i64>,
    pub from_node_id: i64,
    pub to_node_id: i64,
    pub source_knowledge_unit_id: Option<i64>,
    pub target_knowledge_unit_id: Option<i64>,
    pub confusion_strength: BasisPoints,
    pub directionality: String,
    pub timed_confusion_strength: BasisPoints,
    pub calm_confusion_strength: BasisPoints,
    pub total_confusions: i64,
    pub context_tags: Vec<String>,
    pub status: String,
    pub last_confusion_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionStep {
    pub step_code: String,
    pub prompt: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetestPlan {
    pub recommended_mode: String,
    pub target_success_bp: BasisPoints,
    pub review_after_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionPlanRecord {
    pub id: i64,
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub family: String,
    pub reason: String,
    pub primary_failure_mode: String,
    pub target_state: String,
    pub steps: Vec<InterventionStep>,
    pub retest_plan: RetestPlan,
    pub estimated_difficulty_bp: BasisPoints,
    pub estimated_duration_min: i64,
    pub priority_score: BasisPoints,
    pub status: String,
    pub completed_step_count: i64,
    pub total_step_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteInterventionStepInput {
    pub plan_id: i64,
    pub step_code: String,
    pub outcome: String,
    pub successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScheduleItemRecord {
    pub id: i64,
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub due_at: String,
    pub urgency_score: BasisPoints,
    pub recommended_mode: String,
    pub reason: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStateTransitionRecord {
    pub id: i64,
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub from_state: String,
    pub to_state: String,
    pub reason: String,
    pub evidence_snapshot_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineEventRecord {
    pub id: i64,
    pub event_type: String,
    pub student_id: Option<i64>,
    pub knowledge_unit_id: Option<i64>,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureProfileRecord {
    pub student_id: i64,
    pub knowledge_unit_id: i64,
    pub calm_accuracy_bp: BasisPoints,
    pub timed_accuracy_bp: BasisPoints,
    pub pressure_gap_score: BasisPoints,
    pub switch_risk_score: BasisPoints,
    pub freeze_risk_score: BasisPoints,
    pub pressure_state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryKnowledgeStateDetail {
    pub knowledge_unit: KnowledgeUnitRecord,
    pub state: StudentKnowledgeStateRecord,
    pub active_intervention: Option<InterventionPlanRecord>,
    pub review_items: Vec<ReviewScheduleItemRecord>,
    pub recent_attempts: Vec<RetrievalAttemptRecord>,
    pub interference_edges: Vec<StudentInterferenceEdge>,
    pub recent_transitions: Vec<KnowledgeStateTransitionRecord>,
    pub recent_engine_events: Vec<MemoryEngineEventRecord>,
    pub pressure_profile: Option<PressureProfileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalyticsHotspot {
    pub knowledge_unit_id: i64,
    pub node_id: Option<i64>,
    pub title: String,
    pub fragile_count: i64,
    pub collapsed_count: i64,
    pub active_interventions: i64,
    pub due_reviews: i64,
    pub average_decay_risk: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCohortAnalytics {
    pub topic_id: i64,
    pub student_count: i64,
    pub knowledge_unit_count: i64,
    pub fragile_count: i64,
    pub decaying_count: i64,
    pub collapsed_count: i64,
    pub due_reviews: i64,
    pub active_interventions: i64,
    pub average_decay_risk: BasisPoints,
    pub average_pressure_gap: BasisPoints,
    pub top_failure_modes: Vec<String>,
    pub hotspots: Vec<MemoryAnalyticsHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicKnowledgeMap {
    pub topic_id: i64,
    pub units: Vec<KnowledgeUnitRecord>,
    pub edges: Vec<KnowledgeUnitEdgeRecord>,
}
