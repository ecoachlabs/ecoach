use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub subtopic_id: Option<i64>,
    pub family_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub explanation_text: Option<String>,
    pub difficulty_level: BasisPoints,
    pub estimated_time_seconds: i64,
    pub marks: i64,
    pub primary_skill_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub id: i64,
    pub question_id: i64,
    pub option_label: String,
    pub option_text: String,
    pub is_correct: bool,
    pub misconception_id: Option<i64>,
    pub distractor_intent: Option<String>,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionSelectionRequest {
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub target_question_count: usize,
    pub target_difficulty: Option<BasisPoints>,
    pub weakness_topic_ids: Vec<i64>,
    pub recently_seen_question_ids: Vec<i64>,
    pub timed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedQuestion {
    pub question: Question,
    pub fit_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionIntelligenceLink {
    pub axis_code: String,
    pub concept_code: String,
    pub display_name: String,
    pub confidence_score: BasisPoints,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionIntelligenceProfile {
    pub question: Question,
    pub links: Vec<QuestionIntelligenceLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionIntelligenceQuery {
    pub axis_code: String,
    pub concept_code: String,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionSlotSpec {
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub target_cognitive_demand: Option<String>,
    pub target_question_format: Option<String>,
    pub max_generated_share: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFamilyChoice {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub total_instances: i64,
    pub generated_instances: i64,
    pub fit_score: BasisPoints,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionVariantMode {
    Isomorphic,
    RepresentationShift,
    MisconceptionProbe,
    Rescue,
    Stretch,
}

impl QuestionVariantMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Isomorphic => "isomorphic",
            Self::RepresentationShift => "representation_shift",
            Self::MisconceptionProbe => "misconception_probe",
            Self::Rescue => "rescue",
            Self::Stretch => "stretch",
        }
    }

    pub fn relation_type(self) -> &'static str {
        match self {
            Self::RepresentationShift => "representation_shift",
            Self::MisconceptionProbe => "misconception_probe",
            Self::Rescue => "repair_variant",
            Self::Stretch => "difficulty_ladder",
            Self::Isomorphic => "variant_of",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionGenerationRequestInput {
    pub slot_spec: QuestionSlotSpec,
    pub family_id: Option<i64>,
    pub source_question_id: Option<i64>,
    pub request_kind: String,
    pub variant_mode: QuestionVariantMode,
    pub requested_count: usize,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionGenerationRequest {
    pub id: i64,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub family_id: i64,
    pub source_question_id: Option<i64>,
    pub request_kind: String,
    pub variant_mode: String,
    pub requested_count: i64,
    pub status: String,
    pub rationale: Option<String>,
    pub generated_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuestionDraft {
    pub request_id: i64,
    pub source_question_id: i64,
    pub question: Question,
    pub options: Vec<QuestionOption>,
    pub variant_mode: String,
    pub transform_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageNode {
    pub question_id: i64,
    pub family_id: Option<i64>,
    pub lineage_key: String,
    pub node_role: String,
    pub origin_kind: String,
    pub fingerprint_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageEdge {
    pub from_question_id: i64,
    pub to_question_id: i64,
    pub relation_type: String,
    pub transform_mode: Option<String>,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageGraph {
    pub focus_question_id: i64,
    pub nodes: Vec<QuestionLineageNode>,
    pub edges: Vec<QuestionLineageEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFamilyHealth {
    pub family_id: i64,
    pub total_instances: i64,
    pub generated_instances: i64,
    pub active_instances: i64,
    pub recent_attempts: i64,
    pub recent_correct_attempts: i64,
    pub avg_response_time_ms: i64,
    pub misconception_hit_count: i64,
    pub freshness_score: BasisPoints,
    pub calibration_score: BasisPoints,
    pub quality_score: BasisPoints,
    pub health_status: String,
    pub last_generated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionGraphEdge {
    pub from_question_id: i64,
    pub to_question_id: i64,
    pub relation_type: String,
    pub similarity_score: BasisPoints,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedQuestion {
    pub question: Question,
    pub edge: QuestionGraphEdge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    pub matched_question_id: Option<i64>,
    pub similarity_score: BasisPoints,
    pub is_exact_duplicate: bool,
    pub is_near_duplicate: bool,
}
