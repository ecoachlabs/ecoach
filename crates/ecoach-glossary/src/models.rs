use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: i64,
    pub title: String,
    pub entry_type: String,
    pub short_text: Option<String>,
    pub topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntryProfile {
    pub id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub subtopic_id: Option<i64>,
    pub entry_type: String,
    pub title: String,
    pub canonical_name: Option<String>,
    pub slug: Option<String>,
    pub short_text: Option<String>,
    pub full_text: Option<String>,
    pub simple_text: Option<String>,
    pub technical_text: Option<String>,
    pub exam_text: Option<String>,
    pub importance_score: i64,
    pub difficulty_level: i64,
    pub grade_band: Option<String>,
    pub status: String,
    pub audio_available: bool,
    pub has_formula: bool,
    pub confusion_pair_count: i64,
    pub example_count: i64,
    pub misconception_count: i64,
    pub exam_relevance_score: i64,
    pub priority_score: i64,
    pub phonetic_text: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryAlias {
    pub alias_text: String,
    pub alias_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryContentBlock {
    pub id: i64,
    pub block_type: String,
    pub order_index: i64,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionMeta {
    pub definition_text: String,
    pub short_definition: Option<String>,
    pub formal_definition: Option<String>,
    pub plain_english_definition: Option<String>,
    pub real_world_meaning: Option<String>,
    pub non_examples: Option<String>,
    pub context_clues: Option<String>,
    pub pronunciation_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaMeta {
    pub formula_expression: String,
    pub formula_speech: Option<String>,
    pub formula_latex: Option<String>,
    pub variables: Value,
    pub units: Option<Value>,
    pub when_to_use: Option<String>,
    pub when_not_to_use: Option<String>,
    pub rearrangements: Vec<String>,
    pub assumptions: Vec<String>,
    pub common_errors: Vec<String>,
    pub derivation_summary: Option<String>,
    pub worked_example_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMeta {
    pub concept_explanation: String,
    pub intuition_summary: Option<String>,
    pub related_visual_keywords: Vec<String>,
    pub misconception_signals: Vec<String>,
    pub why_it_matters: Option<String>,
    pub mastery_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryExample {
    pub id: i64,
    pub sequence_order: i64,
    pub example_text: String,
    pub context_type: String,
    pub difficulty_level: i64,
    pub worked_solution_text: Option<String>,
    pub is_exam_style: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMisconception {
    pub id: i64,
    pub misconception_text: String,
    pub cause_explanation: Option<String>,
    pub correction_explanation: Option<String>,
    pub confusion_pair_entry_id: Option<i64>,
    pub misconception_source: Option<String>,
    pub severity_bp: i64,
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
pub struct EntryBundleReference {
    pub bundle_id: i64,
    pub title: String,
    pub bundle_type: String,
    pub description: Option<String>,
    pub item_role: Option<String>,
    pub sequence_order: Option<i64>,
    pub required: bool,
    pub difficulty_level: i64,
    pub exam_relevance_score: i64,
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
pub struct LinkedQuestionSummary {
    pub question_id: i64,
    pub relation_type: String,
    pub confidence_score: i64,
    pub is_primary: bool,
    pub link_source: String,
    pub link_reason: Option<String>,
    pub stem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelationLink {
    pub relation_id: i64,
    pub relation_type: String,
    pub strength_score: i64,
    pub explanation: Option<String>,
    pub from_entry_id: i64,
    pub to_entry_id: i64,
    pub related_entry_title: String,
    pub related_entry_type: String,
    pub related_topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentEntrySnapshot {
    pub user_id: i64,
    pub familiarity_state: Option<String>,
    pub mastery_score: i64,
    pub confusion_score: i64,
    pub recall_strength: i64,
    pub open_count: i64,
    pub linked_wrong_answer_count: i64,
    pub recognition_score: i64,
    pub connection_score: i64,
    pub application_score: i64,
    pub retention_score: i64,
    pub test_count: i64,
    pub test_pass_count: i64,
    pub last_viewed_at: Option<String>,
    pub last_played_at: Option<String>,
    pub last_tested_at: Option<String>,
    pub review_due_at: Option<String>,
    pub spaced_review_due_at: Option<String>,
    pub at_risk_threshold_date: Option<String>,
    pub mastery_state: Option<String>,
    pub at_risk_flag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionPairDetail {
    pub paired_entry_id: i64,
    pub paired_entry_title: String,
    pub distinction_explanation: String,
    pub common_confusion_reason: Option<String>,
    pub clue_to_distinguish: Option<String>,
    pub example_sentence_1: Option<String>,
    pub example_sentence_2: Option<String>,
    pub confusion_frequency_bp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborIntruderMapping {
    pub neighbors: Vec<i64>,
    pub intruders: Vec<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntryDetail {
    pub entry: KnowledgeEntryProfile,
    pub aliases: Vec<EntryAlias>,
    pub content_blocks: Vec<EntryContentBlock>,
    pub definition_meta: Option<DefinitionMeta>,
    pub formula_meta: Option<FormulaMeta>,
    pub concept_meta: Option<ConceptMeta>,
    pub examples: Vec<EntryExample>,
    pub misconceptions: Vec<EntryMisconception>,
    pub relations: Vec<KnowledgeRelationLink>,
    pub bundles: Vec<EntryBundleReference>,
    pub linked_questions: Vec<LinkedQuestionSummary>,
    pub audio_segments: Vec<GlossaryAudioSegment>,
    pub student_state: Option<StudentEntrySnapshot>,
    pub confusion_pairs: Vec<ConfusionPairDetail>,
    pub neighbor_intruder: Option<NeighborIntruderMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossarySearchInput {
    pub query: String,
    pub student_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub include_bundles: bool,
    pub include_questions: bool,
    pub include_confusions: bool,
    pub include_audio_ready_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossarySearchResult {
    pub result_type: String,
    pub entry_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub question_id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub entry_type: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub match_reason: String,
    pub match_score: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossarySearchGroup {
    pub group_key: String,
    pub title: String,
    pub results: Vec<GlossarySearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossarySearchResponse {
    pub normalized_query: String,
    pub query_intent: String,
    pub groups: Vec<GlossarySearchGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossarySearchSuggestion {
    pub suggestion: String,
    pub suggestion_type: String,
    pub entry_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryInteractionInput {
    pub student_id: Option<i64>,
    pub entry_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub question_id: Option<i64>,
    pub event_type: String,
    pub query_text: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartGlossaryAudioQueueInput {
    pub source_type: String,
    pub source_id: i64,
    pub limit: usize,
    pub teaching_mode: Option<String>,
    pub include_examples: bool,
    pub include_misconceptions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateGlossaryAudioQueueInput {
    pub playback_speed: Option<f64>,
    pub include_examples: Option<bool>,
    pub include_misconceptions: Option<bool>,
    pub is_playing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryAudioQueueSnapshot {
    pub current_program_id: Option<i64>,
    pub current_position: i64,
    pub is_playing: bool,
    pub playback_speed: f64,
    pub include_examples: bool,
    pub include_misconceptions: bool,
    pub current_segment: Option<GlossaryAudioSegment>,
    pub program: Option<GlossaryAudioProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntryFocus {
    pub entry_id: i64,
    pub title: String,
    pub entry_type: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub need_score: i64,
    pub exam_relevance_score: i64,
    pub match_reason: String,
    pub mastery_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryHomeSnapshot {
    pub discover: Vec<GlossaryEntryFocus>,
    pub weak_entries: Vec<GlossaryEntryFocus>,
    pub exam_hotspots: Vec<GlossaryEntryFocus>,
    pub recommended_bundles: Vec<KnowledgeBundleSequenceItem>,
    pub audio_station_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryComparisonView {
    pub left_entry: KnowledgeEntryProfile,
    pub right_entry: KnowledgeEntryProfile,
    pub shared_relation_types: Vec<String>,
    pub distinction_explanation: Option<String>,
    pub clue_to_distinguish: Option<String>,
    pub shared_bundles: Vec<EntryBundleReference>,
    pub linked_question_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaLabView {
    pub entry: KnowledgeEntryProfile,
    pub formula_meta: FormulaMeta,
    pub related_entries: Vec<KnowledgeRelationLink>,
    pub examples: Vec<EntryExample>,
    pub misconceptions: Vec<EntryMisconception>,
    pub linked_questions: Vec<LinkedQuestionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapNode {
    pub entry_id: i64,
    pub title: String,
    pub entry_type: String,
    pub topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapEdge {
    pub from_entry_id: i64,
    pub to_entry_id: i64,
    pub relation_type: String,
    pub strength_score: i64,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapView {
    pub root_entry_id: i64,
    pub nodes: Vec<ConceptMapNode>,
    pub edges: Vec<ConceptMapEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTestItem {
    pub sequence_no: i64,
    pub entry_id: i64,
    pub prompt_type: String,
    pub prompt_text: String,
    pub expected_answer: Option<String>,
    pub options: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGlossaryTestInput {
    pub test_mode: String,
    pub topic_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub entry_ids: Vec<i64>,
    pub entry_count: usize,
    pub duration_seconds: Option<i64>,
    pub difficulty_level: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitGlossaryTestAttemptInput {
    pub entry_id: i64,
    pub student_response: String,
    pub time_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTestAttemptResult {
    pub is_correct: bool,
    pub feedback: String,
    pub updated_mastery_state: Option<String>,
    pub review_due_at: Option<String>,
    pub mastery_score: i64,
    pub recognition_score: i64,
    pub connection_score: i64,
    pub application_score: i64,
    pub retention_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTestSessionDetail {
    pub session_id: i64,
    pub student_id: i64,
    pub test_mode: String,
    pub topic_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub entry_count: i64,
    pub duration_seconds: Option<i64>,
    pub difficulty_level: i64,
    pub recall_score_bp: Option<i64>,
    pub recognition_score_bp: Option<i64>,
    pub connection_score_bp: Option<i64>,
    pub application_score_bp: Option<i64>,
    pub retention_score_bp: Option<i64>,
    pub confidence_score_bp: Option<i64>,
    pub completion_rate_bp: i64,
    pub items: Vec<GlossaryTestItem>,
}
