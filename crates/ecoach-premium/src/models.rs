use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

// ── Risk severity levels ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskFlagStatus {
    Active,
    Monitoring,
    Resolved,
}

impl RiskFlagStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Monitoring => "monitoring",
            Self::Resolved => "resolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionStatus {
    Active,
    Review,
    Resolved,
    Escalated,
}

impl InterventionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Review => "review",
            Self::Resolved => "resolved",
            Self::Escalated => "escalated",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRiskFlagInput {
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub severity: RiskSeverity,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInterventionInput {
    pub student_id: i64,
    pub risk_flag_id: Option<i64>,
    pub title: String,
    pub steps: Vec<InterventionStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionStep {
    pub action: String,
    pub target_topic_id: Option<i64>,
    pub target_minutes: Option<i64>,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRecord {
    pub id: i64,
    pub student_id: i64,
    pub risk_flag_id: Option<i64>,
    pub title: String,
    pub status: String,
    pub steps: Vec<InterventionStep>,
    pub progress_percent: BasisPoints,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumFeature {
    pub feature_key: String,
    pub display_name: String,
    pub tier_required: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentEntitlementSnapshot {
    pub student_id: i64,
    pub tier: String,
    pub active_risk_flags: i64,
    pub active_interventions: i64,
    pub premium_features_enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDashboard {
    pub student_id: i64,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
    pub active_interventions: i64,
    pub flags: Vec<RiskFlag>,
    pub interventions: Vec<InterventionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumPriorityTopic {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub gap_score: BasisPoints,
    pub priority_score: BasisPoints,
    pub trend_state: String,
    pub is_blocked: bool,
    pub next_review_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumStrategySnapshot {
    pub student_id: i64,
    pub student_name: String,
    pub tier: String,
    pub strategy_mode: String,
    pub overall_readiness_score: BasisPoints,
    pub overall_readiness_band: String,
    pub exam_target: Option<String>,
    pub exam_target_date: Option<String>,
    pub current_phase: Option<String>,
    pub daily_budget_minutes: Option<i64>,
    pub inactive_days: Option<i64>,
    pub overdue_review_count: i64,
    pub active_risk_count: i64,
    pub critical_risk_count: i64,
    pub active_intervention_count: i64,
    pub priority_topics: Vec<PremiumPriorityTopic>,
    pub top_risk_titles: Vec<String>,
    pub coach_actions: Vec<String>,
    pub household_actions: Vec<String>,
}
