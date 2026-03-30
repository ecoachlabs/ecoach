use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: i64,
    pub title: String,
    pub entry_type: String,
    pub short_text: Option<String>,
    pub topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBundle {
    pub id: i64,
    pub title: String,
    pub bundle_type: String,
    pub topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBundleSequenceItem {
    pub bundle_id: i64,
    pub title: String,
    pub bundle_type: String,
    pub sequence_order: i64,
    pub focus_reason: String,
    pub due_review_count: i64,
    pub focus_entry_ids: Vec<i64>,
    pub focus_entry_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionKnowledgeLink {
    pub question_id: i64,
    pub entry_id: i64,
    pub title: String,
    pub entry_type: String,
    pub short_text: Option<String>,
    pub topic_id: Option<i64>,
    pub relation_type: String,
    pub link_source: String,
    pub link_reason: Option<String>,
    pub confidence_score: i64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryAudioSegment {
    pub sequence_no: i64,
    pub segment_type: String,
    pub title: String,
    pub script_text: String,
    pub entry_id: Option<i64>,
    pub prompt_text: Option<String>,
    pub focus_reason: Option<String>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryAudioProgram {
    pub program_title: String,
    pub source_type: String,
    pub teaching_mode: String,
    pub topic_id: Option<i64>,
    pub question_id: Option<i64>,
    pub bundle_ids: Vec<i64>,
    pub recommended_bundles: Vec<KnowledgeBundleSequenceItem>,
    pub entry_ids: Vec<i64>,
    pub listener_signals: Vec<String>,
    pub contrast_titles: Vec<String>,
    pub review_entry_ids: Vec<i64>,
    pub review_entry_titles: Vec<String>,
    pub relationship_review_prompts: Vec<String>,
    pub segments: Vec<GlossaryAudioSegment>,
}
