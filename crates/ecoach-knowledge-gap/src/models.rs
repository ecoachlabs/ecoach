use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapRepairStatus {
    Draft,
    Active,
    Completed,
    Abandoned,
}

impl GapRepairStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairItemStatus {
    Pending,
    Active,
    Completed,
    Skipped,
}

impl RepairItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGapRepairPlanInput {
    pub student_id: i64,
    pub topic_id: i64,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapScoreCard {
    pub student_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub gap_score: BasisPoints,
    pub mastery_score: BasisPoints,
    pub knowledge_gap_score: BasisPoints,
    pub conceptual_confusion_score: BasisPoints,
    pub recognition_failure_score: BasisPoints,
    pub execution_error_score: BasisPoints,
    pub severity_label: String,
    pub repair_priority: BasisPoints,
    pub has_active_repair_plan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRepairPlan {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: i64,
    pub topic_name: Option<String>,
    pub status: String,
    pub priority_score: BasisPoints,
    pub severity_label: String,
    pub dominant_focus: String,
    pub recommended_session_type: String,
    pub rationale: String,
    pub focus_breakdown: Vec<GapRepairFocus>,
    pub items: Vec<GapRepairPlanItem>,
    pub progress_percent: BasisPoints,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRepairFocus {
    pub focus_key: String,
    pub score: BasisPoints,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRepairPlanItem {
    pub id: i64,
    pub plan_id: i64,
    pub node_id: Option<i64>,
    pub node_title: Option<String>,
    pub node_type: Option<String>,
    pub sequence_order: i64,
    pub repair_action: String,
    pub status: String,
    pub reason: String,
    pub target_outcome: String,
    pub suggested_duration_minutes: i64,
    pub candidate_question_ids: Vec<i64>,
    pub misconception_titles: Vec<String>,
    pub resource_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidificationSession {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: i64,
    pub repair_plan_id: Option<i64>,
    pub session_id: Option<i64>,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapDashboard {
    pub student_id: i64,
    pub critical_gaps: Vec<GapScoreCard>,
    pub active_repairs: Vec<GapRepairPlan>,
    pub solidification_progress: SolidificationProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidificationProgress {
    pub total_sessions: i64,
    pub completed_sessions: i64,
    pub active_sessions: i64,
    pub topics_solidified: i64,
}
