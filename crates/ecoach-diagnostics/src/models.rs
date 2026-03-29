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
        }
    }
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
    pub phase_id: i64,
    pub question_id: i64,
    pub display_order: i64,
    pub condition_type: String,
    pub stem: String,
    pub question_format: String,
    pub topic_id: i64,
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
