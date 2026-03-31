use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperSet {
    pub id: i64,
    pub subject_id: i64,
    pub exam_year: i64,
    pub paper_code: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperSetSummary {
    pub paper_id: i64,
    pub exam_year: i64,
    pub title: String,
    pub question_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperFamilyAnalytics {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub recurrence_score: BasisPoints,
    pub coappearance_score: BasisPoints,
    pub replacement_score: BasisPoints,
    pub paper_count: i64,
    pub last_seen_year: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperInverseSignal {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub inverse_pressure_score: BasisPoints,
    pub recurrence_score: BasisPoints,
    pub coappearance_score: BasisPoints,
    pub replacement_score: BasisPoints,
    pub paper_count: i64,
    pub last_seen_year: Option<i64>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperComebackSignal {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub comeback_score: BasisPoints,
    pub historical_strength_score: BasisPoints,
    pub dormant_years: i64,
    pub recurrence_score: BasisPoints,
    pub replacement_score: BasisPoints,
    pub paper_count: i64,
    pub last_seen_year: Option<i64>,
    pub rationale: String,
}

// ── Exam Intelligence models (idea13) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperDna {
    pub id: i64,
    pub paper_set_id: i64,
    pub recall_vs_reasoning_ratio: BasisPoints,
    pub novelty_score: BasisPoints,
    pub story_summary: Option<String>,
    pub dominant_families_json: String,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyRelationshipEdge {
    pub id: i64,
    pub source_family_id: i64,
    pub target_family_id: i64,
    pub edge_type: String,
    pub strength_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub support_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFamilyEdgeInput {
    pub source_family_id: i64,
    pub target_family_id: i64,
    pub edge_type: String,
    pub strength_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub support_count: i64,
    pub evidence_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyRecurrenceMetric {
    pub family_id: i64,
    pub subject_id: i64,
    pub total_papers: i64,
    pub papers_appeared: i64,
    pub recurrence_rate_bp: BasisPoints,
    pub persistence_score_bp: BasisPoints,
    pub dormancy_max_years: i64,
    pub last_appearance_year: Option<i64>,
    pub first_appearance_year: Option<i64>,
    pub mutation_trend: Option<String>,
    pub current_relevance_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverseAppearancePair {
    pub family_a_id: i64,
    pub family_b_id: i64,
    pub iai_score_bp: BasisPoints,
    pub directional_a_suppresses_b_bp: BasisPoints,
    pub directional_b_suppresses_a_bp: BasisPoints,
    pub support_papers: i64,
    pub is_mutual: bool,
    pub likely_explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyReplacementTrail {
    pub old_family_id: i64,
    pub new_family_id: i64,
    pub replacement_index_bp: BasisPoints,
    pub iai_component_bp: BasisPoints,
    pub chrono_shift_bp: BasisPoints,
    pub topic_overlap_bp: BasisPoints,
    pub cognitive_overlap_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentFamilyPerformance {
    pub student_id: i64,
    pub family_id: i64,
    pub attempt_count: i64,
    pub accuracy_rate_bp: BasisPoints,
    pub confidence_calibration_bp: BasisPoints,
    pub classical_form_accuracy_bp: Option<BasisPoints>,
    pub mutated_form_accuracy_bp: Option<BasisPoints>,
    pub trap_fall_rate_bp: BasisPoints,
    pub recovery_progress_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyStory {
    pub id: i64,
    pub family_id: i64,
    pub story_type: String,
    pub headline: String,
    pub narrative: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPattern {
    pub id: i64,
    pub pattern_name: String,
    pub pattern_signature: String,
    pub complexity_score: BasisPoints,
    pub parent_family_id: Option<i64>,
    pub instance_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionDnaCard {
    pub question_id: i64,
    pub year: i64,
    pub paper_title: String,
    pub family_name: Option<String>,
    pub recurrence_rate_bp: BasisPoints,
    pub trap_density_score: BasisPoints,
    pub story_role: Option<String>,
    pub examiner_intent: Option<String>,
    pub mutation_class: Option<String>,
    pub cognitive_fingerprint_json: Option<String>,
    pub co_appears_with: Vec<String>,
    pub inverse_families: Vec<String>,
}
