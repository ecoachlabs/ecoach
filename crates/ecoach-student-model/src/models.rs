use chrono::{DateTime, Utc};
use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSubmission {
    pub question_id: i64,
    pub selected_option_id: i64,
    pub session_id: Option<i64>,
    pub session_type: Option<String>,
    pub started_at: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
    pub hint_count: i64,
    pub changed_answer_count: i64,
    pub skipped: bool,
    pub timed_out: bool,
    pub support_level: Option<String>,
    pub was_timed: bool,
    pub was_transfer_variant: bool,
    pub was_retention_check: bool,
    pub was_mixed_context: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    KnowledgeGap,
    ConceptualConfusion,
    RecognitionFailure,
    ExecutionError,
    Carelessness,
    PressureBreakdown,
    ExpressionWeakness,
    SpeedError,
    GuessingDetected,
    MisconceptionTriggered,
}

impl ErrorType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KnowledgeGap => "knowledge_gap",
            Self::ConceptualConfusion => "conceptual_confusion",
            Self::RecognitionFailure => "recognition_failure",
            Self::ExecutionError => "execution_error",
            Self::Carelessness => "carelessness",
            Self::PressureBreakdown => "pressure_breakdown",
            Self::ExpressionWeakness => "expression_weakness",
            Self::SpeedError => "speed_error",
            Self::GuessingDetected => "guessing_detected",
            Self::MisconceptionTriggered => "misconception_triggered",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MasteryState {
    Unseen,
    Exposed,
    Emerging,
    Partial,
    Fragile,
    Stable,
    Robust,
    ExamReady,
}

impl MasteryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unseen => "unseen",
            Self::Exposed => "exposed",
            Self::Emerging => "emerging",
            Self::Partial => "partial",
            Self::Fragile => "fragile",
            Self::Stable => "stable",
            Self::Robust => "robust",
            Self::ExamReady => "exam_ready",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentTopicState {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: i64,
    pub mastery_score: BasisPoints,
    pub mastery_state: MasteryState,
    pub accuracy_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub retention_score: BasisPoints,
    pub transfer_score: BasisPoints,
    pub consistency_score: BasisPoints,
    pub gap_score: BasisPoints,
    pub priority_score: BasisPoints,
    pub trend_state: String,
    pub fragility_score: BasisPoints,
    pub pressure_collapse_index: BasisPoints,
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub evidence_count: i64,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub last_correct_at: Option<DateTime<Utc>>,
    pub memory_strength: BasisPoints,
    pub next_review_at: Option<DateTime<Utc>>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerProcessingResult {
    pub is_correct: bool,
    pub error_type: Option<ErrorType>,
    pub diagnosis_summary: Option<String>,
    pub recommended_action: Option<String>,
    pub explanation: Option<String>,
    pub selected_option_text: String,
    pub correct_option_text: Option<String>,
    pub updated_mastery: BasisPoints,
    pub updated_gap: BasisPoints,
    pub misconception_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthTopicSummary {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub mastery_state: String,
    pub gap_score: BasisPoints,
    pub priority_score: BasisPoints,
    pub memory_strength: BasisPoints,
    pub next_review_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthSkillSummary {
    pub node_id: i64,
    pub title: String,
    pub mastery_score: BasisPoints,
    pub gap_score: BasisPoints,
    pub priority_score: BasisPoints,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthMemorySummary {
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub node_id: Option<i64>,
    pub node_title: Option<String>,
    pub memory_state: String,
    pub memory_strength: BasisPoints,
    pub recall_fluency: BasisPoints,
    pub decay_risk: BasisPoints,
    pub review_due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthDiagnosisSummary {
    pub diagnosis_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub primary_diagnosis: String,
    pub severity: String,
    pub recommended_action: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthSnapshot {
    pub student_id: i64,
    pub student_name: String,
    pub overall_mastery_score: BasisPoints,
    pub overall_readiness_band: String,
    pub pending_review_count: i64,
    pub due_memory_count: i64,
    pub topic_summaries: Vec<LearnerTruthTopicSummary>,
    pub skill_summaries: Vec<LearnerTruthSkillSummary>,
    pub memory_summaries: Vec<LearnerTruthMemorySummary>,
    pub recent_diagnoses: Vec<LearnerTruthDiagnosisSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecayUpdate {
    pub memory_state_id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub previous_state: String,
    pub next_state: String,
    pub previous_strength: BasisPoints,
    pub next_strength: BasisPoints,
    pub decay_risk: BasisPoints,
    pub review_due_at: Option<DateTime<Utc>>,
    pub overdue_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecheckItem {
    pub schedule_id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub node_id: Option<i64>,
    pub node_title: Option<String>,
    pub due_at: DateTime<Utc>,
    pub schedule_type: String,
    pub status: String,
    pub memory_state: Option<String>,
    pub decay_risk: Option<BasisPoints>,
}
