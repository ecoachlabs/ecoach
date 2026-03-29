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
