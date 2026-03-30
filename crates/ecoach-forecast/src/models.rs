use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

/// A complete probabilistic exam blueprint for a subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastBlueprint {
    pub snapshot_id: i64,
    pub subject_id: i64,
    pub total_papers_analyzed: i64,
    pub year_range_start: Option<i64>,
    pub year_range_end: Option<i64>,
    pub confidence_score: BasisPoints,
    pub topic_scores: Vec<ForecastTopicScore>,
    pub format_distribution: Vec<ForecastFormatScore>,
    pub difficulty_distribution: Vec<ForecastDifficultyScore>,
    pub bundles: Vec<ForecastBundle>,
    pub computed_at: String,
}

/// Per-topic forecast score with the 7-factor decomposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastTopicScore {
    pub topic_id: i64,
    pub topic_name: String,
    pub frequency_score: BasisPoints,
    pub recency_score: BasisPoints,
    pub trend_score: BasisPoints,
    pub bundle_strength: BasisPoints,
    pub syllabus_priority: BasisPoints,
    pub style_regime_fit: BasisPoints,
    pub examiner_goal_fit: BasisPoints,
    pub composite_score: BasisPoints,
    pub uncertainty_band: UncertaintyBand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastFormatScore {
    pub format_code: String,
    pub probability_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDifficultyScore {
    pub difficulty_band: String,
    pub probability_score: BasisPoints,
}

/// A group of topics that tend to co-occur in the same exam paper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastBundle {
    pub bundle_key: String,
    pub topic_ids: Vec<i64>,
    pub co_occurrence_count: i64,
    pub strength_score: BasisPoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyBand {
    High,
    Medium,
    Low,
}

impl UncertaintyBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn from_composite(composite: BasisPoints, evidence_count: i64) -> Self {
        if evidence_count < 3 || composite < 2000 {
            Self::High
        } else if evidence_count < 6 || composite < 5000 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

// ---------------------------------------------------------------------------
// Pattern miner intermediate types
// ---------------------------------------------------------------------------

/// Raw frequency data for a single topic across past papers.
#[derive(Debug, Clone)]
pub struct TopicPaperPresence {
    pub topic_id: i64,
    pub topic_name: String,
    pub years_present: Vec<i64>,
    pub total_questions: i64,
    pub question_formats: Vec<String>,
    pub difficulty_bands: Vec<String>,
    pub cognitive_demands: Vec<String>,
}

/// Co-occurrence pair: two topics appearing in the same paper.
#[derive(Debug, Clone)]
pub struct TopicCoOccurrence {
    pub topic_a: i64,
    pub topic_b: i64,
    pub papers_together: i64,
}

/// Aggregate pattern profile for a subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternProfile {
    pub subject_id: i64,
    pub total_papers: i64,
    pub year_range_start: Option<i64>,
    pub year_range_end: Option<i64>,
    pub topic_count: i64,
    pub format_counts: Vec<(String, i64)>,
    pub difficulty_counts: Vec<(String, i64)>,
    pub cognitive_demand_counts: Vec<(String, i64)>,
}

// ---------------------------------------------------------------------------
// Blueprint resolver types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockType {
    Forecast,
    Diagnostic,
    Remediation,
    FinalExam,
    Shock,
    Wisdom,
}

impl MockType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Forecast => "forecast",
            Self::Diagnostic => "diagnostic",
            Self::Remediation => "remediation",
            Self::FinalExam => "final_exam",
            Self::Shock => "shock",
            Self::Wisdom => "wisdom",
        }
    }
}

/// Resolved quotas that a mock compiler uses to pick questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintQuotas {
    pub mock_type: MockType,
    pub total_questions: usize,
    pub topic_quotas: Vec<TopicQuota>,
    pub target_difficulty_mix: Vec<(String, usize)>,
    pub target_format_mix: Vec<(String, usize)>,
    pub min_surprise_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicQuota {
    pub topic_id: i64,
    pub target_count: usize,
    pub is_surprise: bool,
}

/// Lightweight student weakness signal for the blueprint resolver.
#[derive(Debug, Clone)]
pub struct StudentWeaknessSignal {
    pub topic_id: i64,
    pub gap_score: BasisPoints,
    pub mastery_score: BasisPoints,
}
