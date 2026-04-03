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
    pub subject_id: Option<i64>,
    pub subtopic_id: Option<i64>,
    pub urgency_score: BasisPoints,
    pub difficulty_bp: Option<BasisPoints>,
    pub exam_frequency_bp: Option<BasisPoints>,
    pub source: Option<String>,
    pub goal_id: Option<i64>,
    pub calendar_event_id: Option<i64>,
    pub last_opened_at: Option<String>,
    pub open_count: i64,
    pub study_count: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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
    pub subject_id: Option<i64>,
    pub subtopic_id: Option<i64>,
    pub difficulty_bp: Option<BasisPoints>,
    pub exam_frequency_bp: Option<BasisPoints>,
    pub source: Option<String>,
    pub goal_id: Option<i64>,
    pub calendar_event_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLibraryItemInput {
    pub state: String,
    pub tags: Vec<String>,
    pub note_text: Option<String>,
    pub urgency_score: BasisPoints,
    pub topic_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub subtopic_id: Option<i64>,
    pub difficulty_bp: Option<BasisPoints>,
    pub exam_frequency_bp: Option<BasisPoints>,
    pub source: Option<String>,
    pub goal_id: Option<i64>,
    pub calendar_event_id: Option<i64>,
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
    pub description: Option<String>,
    pub icon_hint: Option<String>,
    pub priority_order: i64,
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
    pub reason: Option<String>,
    pub priority_score: Option<BasisPoints>,
    pub recommended_bundle_ids: Vec<i64>,
    pub related_topic_names: Vec<String>,
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
    pub template_code: Option<String>,
    pub topic_ids: Vec<i64>,
    pub question_count: i64,
    pub estimated_minutes: Option<i64>,
    pub difficulty_profile: Option<String>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionPackTemplate {
    pub id: i64,
    pub template_code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub pack_type: String,
    pub selection_strategy: String,
    pub default_item_count: i64,
    pub difficulty_profile: String,
    pub topic_scope: String,
    pub include_explanations: bool,
    pub include_worked_examples: bool,
    pub time_estimate_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRevisionPackFromTemplateInput {
    pub template_code: String,
    pub title: Option<String>,
    pub item_limit: Option<usize>,
    pub subject_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryHomeSnapshot {
    pub due_now_count: i64,
    pub pending_review_count: i64,
    pub fading_concept_count: i64,
    pub untouched_saved_count: i64,
    pub continue_card: Option<ContinueLearningCard>,
    pub learning_paths: Vec<PersonalizedLearningPath>,
    pub generated_shelves: Vec<GeneratedLibraryShelf>,
    pub saved_questions: Vec<SavedQuestionCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicRelationshipHint {
    pub relation_type: String,
    pub from_title: String,
    pub to_title: String,
    pub explanation: String,
    pub hop_count: i64,
    pub strength_score: BasisPoints,
    pub focus_topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemStateHistoryEntry {
    pub id: i64,
    pub library_item_id: i64,
    pub from_state: Option<String>,
    pub to_state: String,
    pub reason: Option<String>,
    pub changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemAction {
    pub id: i64,
    pub student_id: i64,
    pub library_item_id: i64,
    pub action_type: String,
    pub context: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordLibraryItemActionInput {
    pub action_type: String,
    pub context: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryNote {
    pub id: i64,
    pub student_id: i64,
    pub library_item_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub note_type: String,
    pub title: Option<String>,
    pub note_text: String,
    pub context: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLibraryNoteInput {
    pub student_id: i64,
    pub library_item_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub note_type: String,
    pub title: Option<String>,
    pub note_text: String,
    pub context: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySearchInput {
    pub query: Option<String>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub item_types: Vec<String>,
    pub states: Vec<String>,
    pub tags: Vec<String>,
    pub only_wrong: bool,
    pub only_near_mastery: bool,
    pub only_untouched: bool,
    pub high_frequency_only: bool,
    pub due_only: bool,
    pub downloaded_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySearchResult {
    pub item_type: String,
    pub item_ref_id: Option<i64>,
    pub library_item_id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub state: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub tags: Vec<String>,
    pub reason: String,
    pub match_score: BasisPoints,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamHotspot {
    pub family_id: i64,
    pub family_name: String,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub recurrence_rate_bp: BasisPoints,
    pub persistence_score_bp: BasisPoints,
    pub current_relevance_bp: BasisPoints,
    pub student_accuracy_bp: Option<BasisPoints>,
    pub last_appearance_year: Option<i64>,
    pub first_appearance_year: Option<i64>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicLibrarySnapshot {
    pub topic_id: i64,
    pub topic_name: String,
    pub subject_id: i64,
    pub subject_name: String,
    pub mastery_score: Option<i64>,
    pub gap_score: Option<i64>,
    pub mastery_state: Option<String>,
    pub exam_weight: BasisPoints,
    pub saved_item_count: i64,
    pub note_count: i64,
    pub weak_diagnoses: Vec<String>,
    pub question_family_names: Vec<String>,
    pub formula_titles: Vec<String>,
    pub concept_chain_titles: Vec<String>,
    pub saved_items: Vec<LibraryItem>,
    pub notes: Vec<LibraryNote>,
    pub relationship_hints: Vec<TopicRelationshipHint>,
    pub exam_hotspots: Vec<ExamHotspot>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLibraryShelf {
    pub id: i64,
    pub shelf_type: String,
    pub title: String,
    pub description: Option<String>,
    pub icon_hint: Option<String>,
    pub generated: bool,
    pub item_count: i64,
    pub items: Vec<LibraryShelfItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomShelfInput {
    pub title: String,
    pub description: Option<String>,
    pub icon_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddShelfItemInput {
    pub item_type: String,
    pub item_ref_id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub reason: String,
    pub rank_score: BasisPoints,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineLibraryItem {
    pub library_item_id: i64,
    pub item_type: String,
    pub item_ref_id: i64,
    pub title: String,
    pub topic_name: Option<String>,
    pub downloaded_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryTagDefinition {
    pub id: i64,
    pub tag_code: String,
    pub display_name: String,
    pub category: String,
    pub description: Option<String>,
    pub color_hint: Option<String>,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPathStep {
    pub sequence_no: i64,
    pub step_type: String,
    pub title: String,
    pub detail: String,
    pub topic_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub question_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedLearningPath {
    pub topic_id: i64,
    pub topic_name: String,
    pub activity_type: String,
    pub priority_score: BasisPoints,
    pub reason: String,
    pub mastery_score: i64,
    pub gap_score: i64,
    pub recommended_bundle_ids: Vec<i64>,
    pub recommended_bundle_titles: Vec<String>,
    pub related_topic_names: Vec<String>,
    pub relationship_hints: Vec<TopicRelationshipHint>,
    pub steps: Vec<LearningPathStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachActionStep {
    pub sequence_no: i64,
    pub step_type: String,
    pub title: String,
    pub prompt: String,
    pub linked_question_ids: Vec<i64>,
    pub linked_entry_ids: Vec<i64>,
    pub focus_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachActionPlan {
    pub student_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub action_type: String,
    pub readiness_band: String,
    pub support_intensity: String,
    pub mastery_score: i64,
    pub gap_score: i64,
    pub fragile_memory_count: i64,
    pub primary_prompt: String,
    pub diagnostic_focuses: Vec<String>,
    pub recent_diagnoses: Vec<String>,
    pub linked_question_ids: Vec<i64>,
    pub linked_entry_ids: Vec<i64>,
    pub linked_entry_titles: Vec<String>,
    pub target_node_titles: Vec<String>,
    pub relationship_hints: Vec<TopicRelationshipHint>,
    pub recommended_sequence: Vec<TeachActionStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachExplanation {
    pub id: i64,
    pub node_id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub explanation_level: String,
    pub hero_summary: Option<String>,
    pub why_it_matters: Option<String>,
    pub simple_explanation: Option<String>,
    pub structured_breakdown: Value,
    pub worked_examples: Value,
    pub common_mistakes: Value,
    pub exam_appearance_notes: Option<String>,
    pub pattern_recognition_tips: Option<String>,
    pub related_concepts: Value,
    pub visual_asset_refs: Value,
    pub subject_style: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachExplanationUpsertInput {
    pub explanation_level: String,
    pub hero_summary: Option<String>,
    pub why_it_matters: Option<String>,
    pub simple_explanation: Option<String>,
    pub structured_breakdown: Value,
    pub worked_examples: Value,
    pub common_mistakes: Value,
    pub exam_appearance_notes: Option<String>,
    pub pattern_recognition_tips: Option<String>,
    pub related_concepts: Value,
    pub visual_asset_refs: Value,
    pub subject_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachMicroCheck {
    pub id: i64,
    pub explanation_id: i64,
    pub check_type: String,
    pub prompt: String,
    pub correct_answer: String,
    pub distractor_answers: Vec<String>,
    pub explanation_if_wrong: Option<String>,
    pub position_index: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachMicroCheckInput {
    pub check_type: String,
    pub prompt: String,
    pub correct_answer: String,
    pub distractor_answers: Vec<String>,
    pub explanation_if_wrong: Option<String>,
    pub position_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachLesson {
    pub topic_id: i64,
    pub topic_name: String,
    pub node_id: Option<i64>,
    pub node_title: Option<String>,
    pub explanation_level: String,
    pub explanation: Option<TeachExplanation>,
    pub micro_checks: Vec<TeachMicroCheck>,
    pub generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorInteraction {
    pub id: i64,
    pub student_id: i64,
    pub session_id: Option<i64>,
    pub question_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub interaction_type: String,
    pub prompt_text: Option<String>,
    pub response_text: Option<String>,
    pub context: Value,
    pub was_helpful: Option<bool>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorInteractionInput {
    pub student_id: i64,
    pub session_id: Option<i64>,
    pub question_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub interaction_type: String,
    pub prompt_text: Option<String>,
    pub response_text: Option<String>,
    pub context: Value,
    pub was_helpful: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorResponse {
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub question_id: Option<i64>,
    pub interaction_type: String,
    pub prompt_text: Option<String>,
    pub response_text: String,
    pub context_summary: String,
    pub suggested_next_steps: Vec<String>,
    pub related_question_ids: Vec<i64>,
    pub related_entry_ids: Vec<i64>,
    pub related_topic_names: Vec<String>,
}
