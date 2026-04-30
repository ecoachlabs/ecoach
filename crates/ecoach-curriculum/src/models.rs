use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub code: String,
    pub name: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub id: i64,
    pub subject_id: i64,
    pub parent_topic_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub node_type: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicNode {
    pub id: i64,
    pub topic_id: i64,
    pub node_type: String,
    pub canonical_title: String,
    pub core_meaning: Option<String>,
    pub exam_relevance_score: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceUpload {
    pub id: i64,
    pub uploader_account_id: i64,
    pub source_kind: String,
    pub title: String,
    pub source_path: Option<String>,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_level: Option<String>,
    pub subject_code: Option<String>,
    pub academic_year: Option<String>,
    pub language_code: String,
    pub version_label: Option<String>,
    pub source_status: String,
    pub confidence_score: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumParseCandidate {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_type: String,
    pub parent_candidate_id: Option<i64>,
    pub raw_label: String,
    pub normalized_label: Option<String>,
    pub payload: Value,
    pub confidence_score: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumReviewTask {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_id: Option<i64>,
    pub task_type: String,
    pub status: String,
    pub severity: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceReport {
    pub source_upload: CurriculumSourceUpload,
    pub candidates: Vec<CurriculumParseCandidate>,
    pub review_tasks: Vec<CurriculumReviewTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumFamily {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub country_code: String,
    pub exam_board: Option<String>,
    pub education_stage: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumVersion {
    pub id: i64,
    pub curriculum_family_id: Option<i64>,
    pub name: String,
    pub country: String,
    pub exam_board: Option<String>,
    pub education_stage: Option<String>,
    pub version_label: String,
    pub status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub source_summary: Value,
    pub published_at: Option<String>,
    pub replaced_by_version_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSubjectTrack {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub legacy_subject_id: Option<i64>,
    pub subject_code: String,
    pub subject_name: String,
    pub subject_slug: String,
    pub public_title: String,
    pub description: Option<String>,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumLevel {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub level_code: String,
    pub level_name: String,
    pub stage_order: i64,
    pub public_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumTermPeriod {
    pub id: i64,
    pub level_id: i64,
    pub term_code: String,
    pub term_name: String,
    pub sequence_no: i64,
    pub public_term: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNode {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub subject_track_id: i64,
    pub level_id: Option<i64>,
    pub term_id: Option<i64>,
    pub parent_node_id: Option<i64>,
    pub legacy_topic_id: Option<i64>,
    pub node_type: String,
    pub canonical_title: String,
    pub public_title: String,
    pub slug: String,
    pub official_text: Option<String>,
    pub public_summary: Option<String>,
    pub sequence_no: i64,
    pub depth: i64,
    pub estimated_weight: i64,
    pub exam_relevance_score: i64,
    pub difficulty_hint: String,
    pub status: String,
    pub review_status: String,
    pub confidence_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumObjective {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub legacy_learning_objective_id: Option<i64>,
    pub objective_text: String,
    pub simplified_text: Option<String>,
    pub cognitive_level: Option<String>,
    pub objective_type: String,
    pub sequence_no: i64,
    pub confidence_score: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumConceptAtom {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub legacy_academic_node_id: Option<i64>,
    pub concept_type: String,
    pub canonical_term: String,
    pub public_term: Option<String>,
    pub description: Option<String>,
    pub alias_group_id: Option<String>,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumAlias {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub alias_text: String,
    pub alias_kind: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRelationship {
    pub id: i64,
    pub from_entity_type: String,
    pub from_entity_id: i64,
    pub to_entity_type: String,
    pub to_entity_id: i64,
    pub relationship_type: String,
    pub strength_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumLinkedResource {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub resource_type: String,
    pub resource_id: i64,
    pub link_strength: i64,
    pub source: String,
    pub review_status: String,
    pub display_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPublicSnapshot {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub snapshot_kind: String,
    pub status: String,
    pub snapshot_json: Value,
    pub generated_by_account_id: Option<i64>,
    pub generated_at: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPublishResult {
    pub version: CurriculumVersion,
    pub snapshot: CurriculumPublicSnapshot,
    pub diff_report: Option<CurriculumVersionDiffReport>,
    pub subject_count: i64,
    pub node_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumReviewQueueItem {
    pub task: CurriculumReviewTask,
    pub source_title: String,
    pub source_kind: String,
    pub candidate_type: Option<String>,
    pub candidate_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeBundle {
    pub node: CurriculumNode,
    pub objectives: Vec<CurriculumObjective>,
    pub concepts: Vec<CurriculumConceptAtom>,
    pub aliases: Vec<CurriculumAlias>,
    pub relationships: Vec<CurriculumRelationship>,
    pub resource_links: Vec<CurriculumLinkedResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumTreeNode {
    pub node: CurriculumNode,
    pub objective_count: usize,
    pub concept_count: usize,
    pub prerequisite_count: usize,
    pub children: Vec<CurriculumTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPublicSubjectOverview {
    pub family: CurriculumFamily,
    pub version: CurriculumVersion,
    pub subject: CurriculumSubjectTrack,
    pub levels: Vec<CurriculumLevel>,
    pub total_node_count: i64,
    pub total_objective_count: i64,
    pub difficulty_distribution: Value,
    pub latest_snapshot_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumAssessmentPattern {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPublicTopicDetail {
    pub family: CurriculumFamily,
    pub version: CurriculumVersion,
    pub subject: CurriculumSubjectTrack,
    pub node: CurriculumNode,
    pub objectives: Vec<CurriculumObjective>,
    pub concepts: Vec<CurriculumConceptAtom>,
    pub aliases: Vec<CurriculumAlias>,
    pub prerequisites: Vec<CurriculumNode>,
    pub related_nodes: Vec<CurriculumNode>,
    pub resource_links: Vec<CurriculumLinkedResource>,
    pub assessment_patterns: Vec<CurriculumAssessmentPattern>,
    pub misconceptions: Vec<String>,
    pub latest_snapshot_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSearchResult {
    pub node_id: i64,
    pub subject_track_id: i64,
    pub slug: String,
    pub public_title: String,
    pub canonical_title: String,
    pub node_type: String,
    pub family_slug: String,
    pub version_label: String,
    pub subject_title: String,
    pub relevance_score: i64,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumTopicContext {
    pub detail: CurriculumPublicTopicDetail,
    pub formulas: Vec<CurriculumConceptAtom>,
    pub likely_question_types: Vec<CurriculumAssessmentPattern>,
    pub prerequisite_chain: Vec<CurriculumNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRecommendation {
    pub node_id: i64,
    pub slug: String,
    pub public_title: String,
    pub rationale: String,
    pub priority_score: i64,
    pub blocked_by_prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPrerequisiteStep {
    pub node_id: i64,
    pub slug: String,
    pub public_title: String,
    pub depth: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRemediationStep {
    pub sequence_no: i64,
    pub node_id: i64,
    pub slug: String,
    pub public_title: String,
    pub rationale: String,
    pub linked_resource_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRemediationMap {
    pub target_node_id: i64,
    pub target_slug: String,
    pub target_title: String,
    pub steps: Vec<CurriculumRemediationStep>,
    pub misconceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumCoverageStats {
    pub subject_track_id: i64,
    pub total_nodes: i64,
    pub nodes_with_questions: i64,
    pub nodes_with_glossary: i64,
    pub nodes_with_notes: i64,
    pub nodes_with_games: i64,
    pub published_node_count: i64,
    pub coverage_score_bp: i64,
    pub weak_node_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumVersionDiffEntry {
    pub diff_type: String,
    pub entity_type: String,
    pub entity_key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumVersionDiffReport {
    pub base_version_id: i64,
    pub compare_version_id: i64,
    pub entries: Vec<CurriculumVersionDiffEntry>,
    pub migratable_question_links: i64,
    pub migratable_glossary_links: i64,
    pub migratable_study_plans: i64,
    pub migratable_mastery_records: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeCitation {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub source_upload_id: Option<i64>,
    pub citation_kind: String,
    pub reference_code: Option<String>,
    pub source_file_label: Option<String>,
    pub source_page: Option<i64>,
    pub source_section: Option<String>,
    pub source_snippet: Option<String>,
    pub ocr_confidence_score: i64,
    pub parsing_confidence_score: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeExemplar {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub citation_id: Option<i64>,
    pub exemplar_kind: String,
    pub raw_text: String,
    pub public_text: Option<String>,
    pub metadata: Value,
    pub display_order: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeComment {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub citation_id: Option<i64>,
    pub comment_type: String,
    pub comment_text: String,
    pub public_text: Option<String>,
    pub metadata: Value,
    pub display_order: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeIntelligence {
    pub id: i64,
    pub curriculum_node_id: i64,
    pub friendly_topic_name: Option<String>,
    pub internal_subtopic_atoms: Vec<String>,
    pub knowledge_points: Vec<String>,
    pub skills: Vec<String>,
    pub cognitive_verb: Option<String>,
    pub expected_evidence_type: Option<String>,
    pub instructional_mode: Option<String>,
    pub assessment_mode: Option<String>,
    pub misconception_tags: Vec<String>,
    pub prerequisite_node_ids: Vec<i64>,
    pub dependent_node_ids: Vec<i64>,
    pub difficulty_ladder: Vec<String>,
    pub teaching_strategies: Vec<String>,
    pub question_families: Vec<String>,
    pub worked_example_templates: Vec<String>,
    pub memory_tags: Vec<String>,
    pub local_context_examples: Vec<String>,
    pub exam_mapping: Value,
    pub notes: Value,
    pub approval_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRegistryEntry {
    pub family: CurriculumFamily,
    pub version: CurriculumVersion,
    pub subject_count: i64,
    pub node_count: i64,
    pub pending_review_tasks: i64,
    pub low_confidence_nodes: i64,
    pub current_cohorts: Vec<String>,
    pub latest_source_title: Option<String>,
    pub latest_source_status: Option<String>,
    pub workflow_state: String,
    pub has_source_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumIngestionWorkspace {
    pub source_upload: CurriculumSourceUpload,
    pub parse_candidates: Vec<CurriculumParseCandidate>,
    pub review_tasks: Vec<CurriculumReviewTask>,
    pub low_confidence_count: i64,
    pub duplicate_warning_count: i64,
    pub unresolved_count: i64,
    pub parsed_clean_count: i64,
    pub extraction_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumAdminNodeDetail {
    pub bundle: CurriculumNodeBundle,
    pub citations: Vec<CurriculumNodeCitation>,
    pub exemplars: Vec<CurriculumNodeExemplar>,
    pub comments: Vec<CurriculumNodeComment>,
    pub intelligence: Option<CurriculumNodeIntelligence>,
    pub linked_resource_counts: Value,
    pub learner_signal_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumImpactItem {
    pub entity_type: String,
    pub entity_key: String,
    pub diff_type: String,
    pub severity: String,
    pub action_required: String,
    pub affected_node_id: Option<i64>,
    pub affected_question_count: i64,
    pub affected_lesson_count: i64,
    pub affected_drill_count: i64,
    pub affected_diagnostic_count: i64,
    pub affected_learner_count: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumImpactAnalysis {
    pub base_version_id: i64,
    pub compare_version_id: i64,
    pub items: Vec<CurriculumImpactItem>,
    pub affected_lessons: i64,
    pub affected_questions: i64,
    pub affected_drills: i64,
    pub affected_diagnostics: i64,
    pub stale_content_count: i64,
    pub affected_learners: i64,
    pub affected_cohorts: Vec<String>,
    pub safe_to_update_count: i64,
    pub requires_review_count: i64,
    pub requires_regeneration_count: i64,
    pub requires_staging_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumCohortPin {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub cohort_key: String,
    pub cohort_label: String,
    pub level_code: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub rollout_status: String,
    pub pinned_by_account_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCurriculumAssignment {
    pub id: i64,
    pub student_id: i64,
    pub curriculum_version_id: i64,
    pub cohort_pin_id: Option<i64>,
    pub assignment_source: String,
    pub status: String,
    pub notes: Option<String>,
    pub assigned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRegenerationJob {
    pub id: i64,
    pub base_version_id: Option<i64>,
    pub compare_version_id: i64,
    pub affected_node_id: Option<i64>,
    pub entity_type: String,
    pub entity_key: String,
    pub severity: String,
    pub action_required: String,
    pub resource_type: String,
    pub resource_count: i64,
    pub impact_summary: String,
    pub payload: Value,
    pub status: String,
    pub triggered_by_account_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumStudentSubjectCard {
    pub subject_track_id: i64,
    pub subject_slug: String,
    pub public_title: String,
    pub entered_percent: i64,
    pub stable_percent: i64,
    pub exam_ready_percent: i64,
    pub weak_area_count: i64,
    pub blocked_count: i64,
    pub review_due_count: i64,
    pub trend_label: String,
    pub strongest_topic_title: Option<String>,
    pub weakest_topic_title: Option<String>,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumStudentHomeSnapshot {
    pub student_id: i64,
    pub curriculum_version: CurriculumVersion,
    pub subject_cards: Vec<CurriculumStudentSubjectCard>,
    pub entered_percent: i64,
    pub stable_percent: i64,
    pub exam_readiness_percent: i64,
    pub weak_topics_count: i64,
    pub blocked_topics_count: i64,
    pub review_due_count: i64,
    pub position_statement: String,
    pub recent_movements: Vec<String>,
    pub recommended_topics: Vec<CurriculumRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumStudentNodeState {
    pub node: CurriculumNode,
    pub status_label: String,
    pub blocked: bool,
    pub review_due: bool,
    pub exam_ready: bool,
    pub reason: String,
    pub downstream_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumStudentSubjectMap {
    pub student_id: i64,
    pub subject: CurriculumSubjectTrack,
    pub overview: CurriculumStudentSubjectCard,
    pub nodes: Vec<CurriculumStudentNodeState>,
    pub recommended_topics: Vec<CurriculumRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumParentSummary {
    pub parent_id: i64,
    pub learner_id: i64,
    pub curriculum_version: CurriculumVersion,
    pub subject_cards: Vec<CurriculumStudentSubjectCard>,
    pub on_track: bool,
    pub weak_topics: Vec<String>,
    pub overdue_topics: Vec<String>,
    pub exam_risk_by_subject: Value,
    pub summary_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumFamilyInput {
    pub id: Option<i64>,
    pub slug: Option<String>,
    pub name: String,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_stage: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumVersionInput {
    pub id: Option<i64>,
    pub curriculum_family_id: i64,
    pub name: String,
    pub country: Option<String>,
    pub exam_board: Option<String>,
    pub education_stage: Option<String>,
    pub version_label: String,
    pub status: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub source_summary: Value,
    pub replaced_by_version_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSubjectTrackInput {
    pub id: Option<i64>,
    pub curriculum_version_id: i64,
    pub legacy_subject_id: Option<i64>,
    pub subject_code: String,
    pub subject_name: String,
    pub subject_slug: Option<String>,
    pub public_title: Option<String>,
    pub description: Option<String>,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumLevelInput {
    pub id: Option<i64>,
    pub curriculum_version_id: i64,
    pub level_code: String,
    pub level_name: String,
    pub stage_order: i64,
    pub public_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumTermPeriodInput {
    pub id: Option<i64>,
    pub level_id: i64,
    pub term_code: String,
    pub term_name: String,
    pub sequence_no: i64,
    pub public_term: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeInput {
    pub id: Option<i64>,
    pub curriculum_version_id: i64,
    pub subject_track_id: i64,
    pub level_id: Option<i64>,
    pub term_id: Option<i64>,
    pub parent_node_id: Option<i64>,
    pub legacy_topic_id: Option<i64>,
    pub node_type: String,
    pub canonical_title: String,
    pub public_title: Option<String>,
    pub slug: Option<String>,
    pub official_text: Option<String>,
    pub public_summary: Option<String>,
    pub sequence_no: i64,
    pub depth: i64,
    pub estimated_weight: i64,
    pub exam_relevance_score: i64,
    pub difficulty_hint: String,
    pub confidence_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumObjectiveInput {
    pub id: Option<i64>,
    pub legacy_learning_objective_id: Option<i64>,
    pub objective_text: String,
    pub simplified_text: Option<String>,
    pub cognitive_level: Option<String>,
    pub objective_type: String,
    pub sequence_no: i64,
    pub confidence_score: i64,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumConceptAtomInput {
    pub id: Option<i64>,
    pub legacy_academic_node_id: Option<i64>,
    pub concept_type: String,
    pub canonical_term: String,
    pub public_term: Option<String>,
    pub description: Option<String>,
    pub alias_group_id: Option<String>,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumAliasInput {
    pub id: Option<i64>,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub alias_text: String,
    pub alias_kind: String,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRelationshipInput {
    pub id: Option<i64>,
    pub from_entity_type: String,
    pub from_entity_id: Option<i64>,
    pub to_entity_type: String,
    pub to_entity_id: i64,
    pub relationship_type: String,
    pub strength_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumResourceLinkInput {
    pub id: Option<i64>,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub resource_type: String,
    pub resource_id: i64,
    pub link_strength: i64,
    pub source: Option<String>,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeBundleInput {
    pub node: CurriculumNodeInput,
    pub objectives: Vec<CurriculumObjectiveInput>,
    pub concepts: Vec<CurriculumConceptAtomInput>,
    pub aliases: Vec<CurriculumAliasInput>,
    pub relationships: Vec<CurriculumRelationshipInput>,
    pub resource_links: Vec<CurriculumResourceLinkInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeCitationInput {
    pub id: Option<i64>,
    pub curriculum_node_id: i64,
    pub source_upload_id: Option<i64>,
    pub citation_kind: String,
    pub reference_code: Option<String>,
    pub source_file_label: Option<String>,
    pub source_page: Option<i64>,
    pub source_section: Option<String>,
    pub source_snippet: Option<String>,
    pub ocr_confidence_score: i64,
    pub parsing_confidence_score: i64,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeExemplarInput {
    pub id: Option<i64>,
    pub curriculum_node_id: i64,
    pub citation_id: Option<i64>,
    pub exemplar_kind: String,
    pub raw_text: String,
    pub public_text: Option<String>,
    pub metadata: Value,
    pub display_order: i64,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeCommentInput {
    pub id: Option<i64>,
    pub curriculum_node_id: i64,
    pub citation_id: Option<i64>,
    pub comment_type: String,
    pub comment_text: String,
    pub public_text: Option<String>,
    pub metadata: Value,
    pub display_order: i64,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumNodeIntelligenceInput {
    pub curriculum_node_id: i64,
    pub friendly_topic_name: Option<String>,
    pub internal_subtopic_atoms: Vec<String>,
    pub knowledge_points: Vec<String>,
    pub skills: Vec<String>,
    pub cognitive_verb: Option<String>,
    pub expected_evidence_type: Option<String>,
    pub instructional_mode: Option<String>,
    pub assessment_mode: Option<String>,
    pub misconception_tags: Vec<String>,
    pub prerequisite_node_ids: Vec<i64>,
    pub dependent_node_ids: Vec<i64>,
    pub difficulty_ladder: Vec<String>,
    pub teaching_strategies: Vec<String>,
    pub question_families: Vec<String>,
    pub worked_example_templates: Vec<String>,
    pub memory_tags: Vec<String>,
    pub local_context_examples: Vec<String>,
    pub exam_mapping: Value,
    pub notes: Value,
    pub approval_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumCohortPinInput {
    pub id: Option<i64>,
    pub curriculum_version_id: i64,
    pub cohort_key: String,
    pub cohort_label: String,
    pub level_code: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub rollout_status: Option<String>,
    pub pinned_by_account_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCurriculumAssignmentInput {
    pub id: Option<i64>,
    pub student_id: i64,
    pub curriculum_version_id: i64,
    pub cohort_pin_id: Option<i64>,
    pub assignment_source: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}
