use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticMode {
    Quick,
    Standard,
    Deep,
}

impl DiagnosticMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticPhaseCode {
    Baseline,
    Speed,
    Precision,
    Pressure,
    Flex,
    RootCause,
    Endurance,
    Recovery,
}

impl DiagnosticPhaseCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Speed => "speed",
            Self::Precision => "precision",
            Self::Pressure => "pressure",
            Self::Flex => "flex",
            Self::RootCause => "root_cause",
            Self::Endurance => "endurance",
            Self::Recovery => "recovery",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Baseline => "Baseline Mastery",
            Self::Speed => "Speed Response",
            Self::Precision => "Accuracy Control",
            Self::Pressure => "Pressure Tolerance",
            Self::Flex => "Fragility and Transfer",
            Self::RootCause => "Root-Cause Isolation",
            Self::Endurance => "Endurance Drift",
            Self::Recovery => "Recovery After Error",
        }
    }

    pub fn storage_phase_type(self) -> &'static str {
        match self {
            Self::Baseline => "broad_scan",
            Self::Speed => "adaptive_zoom",
            Self::Precision => "adaptive_zoom",
            Self::Pressure => "condition_testing",
            Self::Flex => "stability_recheck",
            Self::RootCause => "confidence_snapshot",
            Self::Endurance => "condition_testing",
            Self::Recovery => "confidence_snapshot",
        }
    }

    pub fn condition_type(self) -> &'static str {
        match self {
            Self::Baseline => "normal",
            Self::Speed => "timed",
            Self::Precision => "normal",
            Self::Pressure => "timed",
            Self::Flex => "transfer",
            Self::RootCause => "recognition",
            Self::Endurance => "stability",
            Self::Recovery => "recognition",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticOverallSummary {
    pub mastery_level: String,
    #[serde(default)]
    pub strong_zones: Vec<String>,
    #[serde(default)]
    pub firming_zones: Vec<String>,
    #[serde(default)]
    pub fragile_zones: Vec<String>,
    #[serde(default)]
    pub critical_zones: Vec<String>,
    pub top_recommended_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDiagnosticResult {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub fluency_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub pressure_score: BasisPoints,
    pub flexibility_score: BasisPoints,
    pub stability_score: BasisPoints,
    pub classification: String,
    pub endurance_score: BasisPoints,
    pub weakness_type: Option<String>,
    pub failure_stage: Option<String>,
    #[serde(default)]
    pub longitudinal_signal: Option<TopicDiagnosticLongitudinalSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongAnswerDiagnosis {
    pub id: i64,
    pub student_id: i64,
    pub question_id: i64,
    pub topic_id: i64,
    pub error_type: String,
    pub primary_diagnosis: String,
    pub secondary_diagnosis: Option<String>,
    pub severity: String,
    pub diagnosis_summary: String,
    pub recommended_action: String,
    pub confidence_score: BasisPoints,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub overall_readiness: BasisPoints,
    pub readiness_band: String,
    pub topic_results: Vec<TopicDiagnosticResult>,
    pub recommended_next_actions: Vec<String>,
    #[serde(default)]
    pub overall_summary: DiagnosticOverallSummary,
    #[serde(default)]
    pub session_scores: Vec<DiagnosticSessionScore>,
    #[serde(default)]
    pub condition_metrics: DiagnosticConditionMetrics,
    #[serde(default)]
    pub skill_results: Vec<DiagnosticSkillResult>,
    #[serde(default)]
    pub recommendations: Vec<DiagnosticRecommendation>,
    #[serde(default)]
    pub learning_profile: Option<DiagnosticLearningProfile>,
    #[serde(default)]
    pub audience_reports: Vec<DiagnosticAudienceReport>,
    #[serde(default)]
    pub longitudinal_summary: Option<DiagnosticLongitudinalSummary>,
    #[serde(default)]
    pub problem_cause_fix_cards: Vec<DiagnosticProblemCauseFixCard>,
    #[serde(default)]
    pub intervention_prescriptions: Vec<DiagnosticInterventionPrescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSubjectBlueprint {
    pub subject_id: i64,
    pub blueprint_code: String,
    pub subject_name: String,
    pub session_modes: Value,
    pub stage_rules: Value,
    pub item_family_mix: Vec<Value>,
    pub routing_contract: Value,
    pub report_contract: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItemRoutingProfile {
    pub question_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub family_id: Option<i64>,
    pub item_family: String,
    pub recognition_suitable: bool,
    pub recall_suitable: bool,
    pub transfer_suitable: bool,
    pub timed_suitable: bool,
    pub confidence_prompt: String,
    pub recommended_stages: Vec<String>,
    pub sibling_variant_modes: Vec<String>,
    pub routing_notes: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticProblemCauseFixCard {
    pub topic_id: i64,
    pub topic_name: String,
    pub problem_summary: String,
    pub cause_summary: String,
    pub fix_summary: String,
    pub confidence_score: BasisPoints,
    pub impact_score: BasisPoints,
    pub unlock_summary: Option<String>,
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInterventionPrescription {
    pub topic_id: i64,
    pub topic_name: String,
    pub primary_mode_code: String,
    pub support_mode_code: Option<String>,
    pub recheck_mode_code: Option<String>,
    pub mode_chain: Vec<String>,
    pub contraindications: Vec<String>,
    pub success_signals: Vec<String>,
    pub confidence_score: BasisPoints,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDiagnosticLongitudinalSignal {
    pub previous_diagnostic_id: Option<i64>,
    pub previous_completed_at: Option<String>,
    pub previous_classification: Option<String>,
    pub previous_mastery_score: Option<BasisPoints>,
    pub mastery_delta: Option<i64>,
    pub pressure_delta: Option<i64>,
    pub flexibility_delta: Option<i64>,
    pub trend: String,
    #[serde(default)]
    pub cause_evolution: Option<DiagnosticCauseEvolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCauseEvolution {
    pub topic_id: i64,
    pub topic_name: String,
    pub current_hypothesis_code: Option<String>,
    pub previous_hypothesis_code: Option<String>,
    pub evolution_status: String,
    pub recurrence_count: i64,
    pub confidence_delta: Option<i64>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLongitudinalSummary {
    pub previous_diagnostic_id: Option<i64>,
    pub previous_completed_at: Option<String>,
    pub overall_readiness_delta: Option<i64>,
    pub trend: String,
    pub improved_topic_count: usize,
    pub declined_topic_count: usize,
    pub stable_topic_count: usize,
    pub persistent_cause_count: usize,
    pub shifted_cause_count: usize,
    pub new_cause_count: usize,
    pub top_regressions: Vec<String>,
    pub cause_evolution: Vec<DiagnosticCauseEvolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPhasePlan {
    pub phase_id: i64,
    pub phase_number: i64,
    pub phase_code: String,
    pub phase_title: String,
    pub phase_type: String,
    pub status: String,
    pub question_count: i64,
    pub time_limit_seconds: Option<i64>,
    pub condition_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPhaseItem {
    pub attempt_id: i64,
    pub phase_id: i64,
    pub question_id: i64,
    pub display_order: i64,
    pub condition_type: String,
    pub stem: String,
    pub question_format: String,
    pub topic_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticSessionScore {
    pub phase_code: String,
    pub phase_title: String,
    pub raw_accuracy: BasisPoints,
    pub adjusted_accuracy: BasisPoints,
    pub median_response_time_ms: Option<i64>,
    pub stability_measure: BasisPoints,
    pub careless_error_rate: BasisPoints,
    pub timeout_rate: BasisPoints,
    pub misread_rate: BasisPoints,
    pub pressure_volatility: BasisPoints,
    pub early_segment_accuracy: Option<BasisPoints>,
    pub middle_segment_accuracy: Option<BasisPoints>,
    pub final_segment_accuracy: Option<BasisPoints>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticConditionMetrics {
    pub fragility_index: BasisPoints,
    pub pressure_collapse_index: BasisPoints,
    pub recognition_gap_index: BasisPoints,
    pub formula_recall_use_delta: BasisPoints,
    pub early_late_delta: BasisPoints,
    pub confidence_correctness_delta: BasisPoints,
    pub endurance_drop: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticSkillResult {
    pub skill_key: String,
    pub skill_name: String,
    pub skill_type: String,
    pub topic_id: i64,
    pub topic_name: String,
    pub baseline_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub pressure_score: BasisPoints,
    pub flex_score: BasisPoints,
    pub root_cause_score: BasisPoints,
    pub endurance_score: BasisPoints,
    pub recovery_score: BasisPoints,
    pub mastery_score: BasisPoints,
    pub fragility_index: BasisPoints,
    pub pressure_collapse_index: BasisPoints,
    pub recognition_gap_index: BasisPoints,
    pub formula_recall_use_delta: BasisPoints,
    pub stability_score: BasisPoints,
    pub mastery_state: String,
    pub weakness_type_primary: String,
    pub weakness_type_secondary: Option<String>,
    pub recommended_intervention: String,
    #[serde(default)]
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticRecommendation {
    pub category: String,
    pub action_code: String,
    pub title: String,
    pub rationale: String,
    pub priority: i64,
    pub target_kind: Option<String>,
    pub target_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticLearningProfile {
    pub profile_type: String,
    pub confidence_score: BasisPoints,
    #[serde(default)]
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticAudienceReport {
    pub audience: String,
    pub headline: String,
    pub narrative: String,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub fragile_areas: Vec<String>,
    #[serde(default)]
    pub critical_areas: Vec<String>,
    #[serde(default)]
    pub action_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticBattery {
    pub diagnostic_id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub session_mode: String,
    pub status: String,
    pub phases: Vec<DiagnosticPhasePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTopicAnalytics {
    pub diagnostic_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub fluency_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub pressure_score: BasisPoints,
    pub flexibility_score: BasisPoints,
    pub stability_score: BasisPoints,
    pub classification: String,
    pub confidence_score: BasisPoints,
    pub recommended_action: String,
    pub endurance_score: BasisPoints,
    #[serde(default)]
    pub error_distribution: Value,
    pub weakness_type: Option<String>,
    pub failure_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRootCauseHypothesis {
    pub id: i64,
    pub diagnostic_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub hypothesis_code: String,
    pub confidence_score: BasisPoints,
    pub recommended_action: String,
    pub evidence: Value,
    pub created_at: String,
}
