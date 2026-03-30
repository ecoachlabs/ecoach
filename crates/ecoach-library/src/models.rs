use chrono::{DateTime, Utc};
use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: i64,
    pub student_id: i64,
    pub item_type: String,
    pub item_ref_id: i64,
    pub state: String,
    pub tags: Vec<String>,
    pub note_text: Option<String>,
    pub topic_id: Option<i64>,
    pub urgency_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLibraryItemInput {
    pub item_type: String,
    pub item_ref_id: i64,
    pub state: String,
    pub tags: Vec<String>,
    pub note_text: Option<String>,
    pub topic_id: Option<i64>,
    pub urgency_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuestionCard {
    pub library_item_id: i64,
    pub question_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub stem: String,
    pub state: String,
    pub related_family_name: Option<String>,
    pub linked_knowledge_count: i64,
    pub urgency_score: BasisPoints,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryShelfItem {
    pub item_type: String,
    pub item_ref_id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub reason: String,
    pub rank_score: BasisPoints,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedLibraryShelf {
    pub shelf_id: Option<i64>,
    pub shelf_type: String,
    pub title: String,
    pub generated: bool,
    pub items: Vec<LibraryShelfItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueLearningCard {
    pub title: String,
    pub activity_type: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub mission_id: Option<i64>,
    pub session_id: Option<i64>,
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionPackItem {
    pub id: i64,
    pub item_type: String,
    pub item_ref_id: i64,
    pub sequence_order: i64,
    pub required: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionPackSummary {
    pub pack_id: i64,
    pub title: String,
    pub source_type: Option<String>,
    pub topic_ids: Vec<i64>,
    pub question_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryHomeSnapshot {
    pub due_now_count: i64,
    pub pending_review_count: i64,
    pub fading_concept_count: i64,
    pub untouched_saved_count: i64,
    pub continue_card: Option<ContinueLearningCard>,
    pub generated_shelves: Vec<GeneratedLibraryShelf>,
    pub saved_questions: Vec<SavedQuestionCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicRelationshipHint {
    pub relation_type: String,
    pub from_title: String,
    pub to_title: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachActionPlan {
    pub student_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub action_type: String,
    pub primary_prompt: String,
    pub linked_question_ids: Vec<i64>,
    pub linked_entry_ids: Vec<i64>,
    pub linked_entry_titles: Vec<String>,
    pub target_node_titles: Vec<String>,
    pub relationship_hints: Vec<TopicRelationshipHint>,
}
