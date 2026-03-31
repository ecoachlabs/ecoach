use ecoach_substrate::{BasisPoints, FabricOrchestrationSummary};
use serde::{Deserialize, Serialize};

// ── Readiness bands (idea12) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessBand {
    Fragile,
    Building,
    Strengthening,
    Strong,
    ExamReady,
    EliteReady,
}

impl ReadinessBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fragile => "fragile",
            Self::Building => "building",
            Self::Strengthening => "strengthening",
            Self::Strong => "strong",
            Self::ExamReady => "exam_ready",
            Self::EliteReady => "elite_ready",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "fragile" => Self::Fragile,
            "building" => Self::Building,
            "strengthening" => Self::Strengthening,
            "strong" => Self::Strong,
            "exam_ready" => Self::ExamReady,
            "elite_ready" => Self::EliteReady,
            _ => Self::Building,
        }
    }

    pub fn from_bp(bp: BasisPoints) -> Self {
        match bp {
            0..=1999 => Self::Fragile,
            2000..=3999 => Self::Building,
            4000..=5999 => Self::Strengthening,
            6000..=7499 => Self::Strong,
            7500..=8999 => Self::ExamReady,
            _ => Self::EliteReady,
        }
    }
}

// ── Risk categories (idea12 taxonomy) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Knowledge,
    Reasoning,
    Performance,
    Behavioral,
    Strategic,
    Confidence,
}

impl RiskCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Knowledge => "knowledge",
            Self::Reasoning => "reasoning",
            Self::Performance => "performance",
            Self::Behavioral => "behavioral",
            Self::Strategic => "strategic",
            Self::Confidence => "confidence",
        }
    }
}

// ── Intervention classes (idea12) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionClass {
    ConceptRebuild,
    MisconceptionCorrection,
    RecallReinforcement,
    SpeedConditioning,
    PressureAdaptation,
    ExamTechniqueRepair,
    ConfidenceStabilization,
    EliteStretch,
}

impl InterventionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConceptRebuild => "concept_rebuild",
            Self::MisconceptionCorrection => "misconception_correction",
            Self::RecallReinforcement => "recall_reinforcement",
            Self::SpeedConditioning => "speed_conditioning",
            Self::PressureAdaptation => "pressure_adaptation",
            Self::ExamTechniqueRepair => "exam_technique_repair",
            Self::ConfidenceStabilization => "confidence_stabilization",
            Self::EliteStretch => "elite_stretch",
        }
    }
}

// ── Concierge question families ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConciergeQuestionFamily {
    Status,
    Risk,
    Strategy,
    Forecast,
    Action,
    Explanation,
    Custom,
}

impl ConciergeQuestionFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Risk => "risk",
            Self::Strategy => "strategy",
            Self::Forecast => "forecast",
            Self::Action => "action",
            Self::Explanation => "explanation",
            Self::Custom => "custom",
        }
    }
}

// ── Milestone review types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneReviewType {
    ThirtyDay,
    SixtyDay,
    PreMock,
    PreExam,
    Custom,
}

impl MilestoneReviewType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThirtyDay => "thirty_day",
            Self::SixtyDay => "sixty_day",
            Self::PreMock => "pre_mock",
            Self::PreExam => "pre_exam",
            Self::Custom => "custom",
        }
    }
}

// ── Communication types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentCommType {
    RiskAlert,
    StrategyShift,
    MilestoneProgress,
    WeeklyMemo,
    MilestoneReview,
    ConciergeResponse,
    EffortUpdate,
    ReadinessUpdate,
}

impl ParentCommType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RiskAlert => "risk_alert",
            Self::StrategyShift => "strategy_shift",
            Self::MilestoneProgress => "milestone_progress",
            Self::WeeklyMemo => "weekly_memo",
            Self::MilestoneReview => "milestone_review",
            Self::ConciergeResponse => "concierge_response",
            Self::EffortUpdate => "effort_update",
            Self::ReadinessUpdate => "readiness_update",
        }
    }
}

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
    #[serde(default)]
    pub recent_focus_signals: Vec<String>,
    #[serde(default)]
    pub recommended_game_modes: Vec<String>,
    pub coach_actions: Vec<String>,
    pub household_actions: Vec<String>,
    #[serde(default)]
    pub orchestration: FabricOrchestrationSummary,
}

// ── Premium concierge models (idea12) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessProfile {
    pub id: i64,
    pub student_id: i64,
    pub snapshot_date: String,
    pub overall_readiness_bp: BasisPoints,
    pub overall_band: String,
    pub knowledge_solidity_bp: BasisPoints,
    pub application_strength_bp: BasisPoints,
    pub reasoning_quality_bp: BasisPoints,
    pub speed_under_pressure_bp: BasisPoints,
    pub memory_stability_bp: BasisPoints,
    pub confidence_resilience_bp: BasisPoints,
    pub consistency_bp: BasisPoints,
    pub exam_technique_bp: BasisPoints,
    pub target_band: Option<String>,
    pub trajectory: Option<String>,
    pub interpretation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReadinessProfileInput {
    pub student_id: i64,
    pub overall_readiness_bp: BasisPoints,
    pub knowledge_solidity_bp: BasisPoints,
    pub application_strength_bp: BasisPoints,
    pub reasoning_quality_bp: BasisPoints,
    pub speed_under_pressure_bp: BasisPoints,
    pub memory_stability_bp: BasisPoints,
    pub confidence_resilience_bp: BasisPoints,
    pub consistency_bp: BasisPoints,
    pub exam_technique_bp: BasisPoints,
    pub target_band: Option<String>,
    pub trajectory: Option<String>,
    pub interpretation: Option<String>,
    pub subject_readiness_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReview {
    pub id: i64,
    pub student_id: i64,
    pub parent_id: Option<i64>,
    pub review_type: String,
    pub review_date: String,
    pub readiness_band: String,
    pub overall_trend: String,
    pub executive_position: String,
    pub subject_progression_json: String,
    pub intervention_effectiveness_json: Option<String>,
    pub confirmed_strengths_json: Option<String>,
    pub unresolved_risks_json: Option<String>,
    pub strategic_adjustments: Option<String>,
    pub forecast_summary: Option<String>,
    pub parent_guidance: Option<String>,
    pub reviewer_type: String,
    pub reviewer_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMilestoneReviewInput {
    pub student_id: i64,
    pub parent_id: Option<i64>,
    pub review_type: MilestoneReviewType,
    pub readiness_band: String,
    pub overall_trend: String,
    pub executive_position: String,
    pub subject_progression_json: String,
    pub intervention_effectiveness_json: Option<String>,
    pub confirmed_strengths_json: Option<String>,
    pub unresolved_risks_json: Option<String>,
    pub strategic_adjustments: Option<String>,
    pub forecast_summary: Option<String>,
    pub parent_guidance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConciergeResponse {
    pub id: i64,
    pub student_id: i64,
    pub parent_id: i64,
    pub question_family: Option<String>,
    pub parent_question: String,
    pub direct_answer: String,
    pub evidence_summary: Option<String>,
    pub academic_interpretation: Option<String>,
    pub current_action: Option<String>,
    pub expected_outcome: Option<String>,
    pub parent_action_needed: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConciergeResponseInput {
    pub student_id: i64,
    pub parent_id: i64,
    pub question_family: ConciergeQuestionFamily,
    pub parent_question: String,
    pub direct_answer: String,
    pub evidence_summary: Option<String>,
    pub academic_interpretation: Option<String>,
    pub current_action: Option<String>,
    pub expected_outcome: Option<String>,
    pub parent_action_needed: Option<String>,
    pub evidence_refs_json: Option<String>,
    pub strategy_state_snapshot_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyState {
    pub id: i64,
    pub student_id: i64,
    pub primary_focus: Option<String>,
    pub secondary_focus: Option<String>,
    pub focus_reason: Option<String>,
    pub expected_outcome: Option<String>,
    pub outcome_window_days: Option<i64>,
    pub mode_selection: Option<String>,
    pub escalation_recommendation: Option<String>,
    pub next_review_date: Option<String>,
    pub last_shift_date: Option<String>,
    pub last_shift_reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategyStateInput {
    pub student_id: i64,
    pub primary_focus: Option<String>,
    pub secondary_focus: Option<String>,
    pub focus_reason: Option<String>,
    pub expected_outcome: Option<String>,
    pub outcome_window_days: Option<i64>,
    pub mode_selection: Option<String>,
    pub subject_priority_json: Option<String>,
    pub topic_priority_json: Option<String>,
    pub escalation_recommendation: Option<String>,
    pub next_review_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTimelineEntry {
    pub id: i64,
    pub student_id: i64,
    pub shift_date: String,
    pub shift_title: String,
    pub reason: String,
    pub evidence_snapshot: Option<String>,
    pub expected_result: Option<String>,
    pub actual_outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentCommunication {
    pub id: i64,
    pub parent_id: i64,
    pub student_id: i64,
    pub comm_type: String,
    pub priority: i64,
    pub title: String,
    pub body: String,
    pub evidence_summary: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateParentCommunicationInput {
    pub parent_id: i64,
    pub student_id: i64,
    pub comm_type: ParentCommType,
    pub priority: i64,
    pub title: String,
    pub body: String,
    pub evidence_summary: Option<String>,
    pub linked_entity_type: Option<String>,
    pub linked_entity_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumIntake {
    pub id: i64,
    pub student_id: i64,
    pub parent_id: i64,
    pub school_name: Option<String>,
    pub school_type: Option<String>,
    pub curriculum: Option<String>,
    pub exam_board: Option<String>,
    pub target_performance: Option<String>,
    pub urgency_level: Option<String>,
    pub biggest_worry: Option<String>,
    pub success_definition: Option<String>,
    pub confidence_level: Option<String>,
    pub anxiety_level: Option<String>,
    pub intake_status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePremiumIntakeInput {
    pub student_id: i64,
    pub parent_id: i64,
    pub school_name: Option<String>,
    pub school_type: Option<String>,
    pub curriculum: Option<String>,
    pub exam_board: Option<String>,
    pub subjects_json: Option<String>,
    pub target_performance: Option<String>,
    pub target_school: Option<String>,
    pub priority_subjects_json: Option<String>,
    pub urgency_level: Option<String>,
    pub biggest_worry: Option<String>,
    pub success_definition: Option<String>,
    pub recent_results_json: Option<String>,
    pub known_strengths: Option<String>,
    pub known_weaknesses: Option<String>,
    pub avoided_subjects: Option<String>,
    pub previous_tutoring: Option<String>,
    pub available_hours_per_week: Option<i64>,
    pub confidence_level: Option<String>,
    pub anxiety_level: Option<String>,
    pub attention_consistency: Option<String>,
    pub resilience_when_corrected: Option<String>,
    pub tendency_to_rush: bool,
    pub tendency_to_hesitate: bool,
}
