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
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryAudioProgram {
    pub program_title: String,
    pub source_type: String,
    pub topic_id: Option<i64>,
    pub question_id: Option<i64>,
    pub bundle_ids: Vec<i64>,
    pub entry_ids: Vec<i64>,
    pub segments: Vec<GlossaryAudioSegment>,
}
