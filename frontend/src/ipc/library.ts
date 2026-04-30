import { ipc } from '.'

export interface LibraryShelfDto {
  shelf_type: string
  title: string
  item_count: number
}

export interface LibraryShelfItemDto {
  item_type: string
  item_ref_id: number | null
  title: string
  subtitle: string | null
  reason: string
  rank_score: number
  metadata: Record<string, unknown>
}

export interface GeneratedLibraryShelfDto {
  shelf_id: number | null
  shelf_type: string
  title: string
  description: string | null
  icon_hint: string | null
  priority_order: number
  generated: boolean
  items: LibraryShelfItemDto[]
}

export interface ContinueLearningCardDto {
  title: string
  activity_type: string
  topic_id: number | null
  topic_name: string | null
  mission_id: number | null
  session_id: number | null
  route: string
  reason: string | null
  priority_score: number | null
  recommended_bundle_ids: number[]
  related_topic_names: string[]
}

export interface SavedQuestionCardDto {
  library_item_id: number
  question_id: number
  topic_id: number
  topic_name: string
  stem: string
  state: string
  related_family_name: string | null
  linked_knowledge_count: number
  urgency_score: number
  saved_at: string
}

export interface PersonalizedLearningPathDto {
  topic_id: number
  topic_name: string
  activity_type: string
  priority_score: number
  reason: string
  mastery_score: number
  gap_score: number
  recommended_bundle_ids: number[]
  recommended_bundle_titles: string[]
  related_topic_names: string[]
}

export interface LibraryHomeSnapshotDto {
  due_now_count: number
  pending_review_count: number
  fading_concept_count: number
  untouched_saved_count: number
  continue_card: ContinueLearningCardDto | null
  learning_paths: PersonalizedLearningPathDto[]
  generated_shelves: GeneratedLibraryShelfDto[]
  saved_questions: SavedQuestionCardDto[]
}

export interface LibraryItemRecordDto {
  id: number
  student_id: number
  item_type: string
  item_ref_id: number
  state: string
  tags: string[]
  note_text: string | null
  topic_id: number | null
  subject_id: number | null
  subtopic_id: number | null
  urgency_score: number
  difficulty_bp: number | null
  exam_frequency_bp: number | null
  source: string | null
  goal_id: number | null
  calendar_event_id: number | null
  last_opened_at: string | null
  open_count: number
  study_count: number
  created_at: string | null
  updated_at: string | null
}

export interface RevisionPackSummaryDto {
  pack_id: number
  title: string
  source_type: string | null
  template_code: string | null
  topic_ids: number[]
  question_count: number
  estimated_minutes: number | null
  difficulty_profile: string | null
  status: string | null
  created_at: string
}

export interface ExamHotspotDto {
  family_id: number
  family_name: string
  subject_id: number
  topic_id: number | null
  topic_name: string | null
  recurrence_rate_bp: number
  persistence_score_bp: number
  current_relevance_bp: number
  student_accuracy_bp: number | null
  last_appearance_year: number | null
  first_appearance_year: number | null
  reason: string
}

export interface LibrarySearchInputDto {
  query?: string | null
  subject_id?: number | null
  topic_id?: number | null
  item_types: string[]
  states: string[]
  tags: string[]
  only_wrong: boolean
  only_near_mastery: boolean
  only_untouched: boolean
  high_frequency_only: boolean
  due_only: boolean
  downloaded_only: boolean
}

export interface LibrarySearchResultDto {
  item_type: string
  item_ref_id: number | null
  library_item_id: number | null
  title: string
  subtitle: string | null
  state: string | null
  topic_id: number | null
  topic_name: string | null
  subject_id: number | null
  subject_name: string | null
  tags: string[]
  reason: string
  match_score: number
  metadata: Record<string, unknown>
}

export interface SaveLibraryItemInputDto {
  item_type: string
  item_ref_id: number
  state: string
  tags: string[]
  note_text: string | null
  topic_id: number | null
  urgency_score: number
  subject_id: number | null
  subtopic_id: number | null
  difficulty_bp: number | null
  exam_frequency_bp: number | null
  source: string | null
  goal_id: number | null
  calendar_event_id: number | null
}

export interface GlossaryEntryDto {
  id: number
  title: string
  entry_type: string
  short_text: string | null
  topic_id: number | null
}

export interface GlossarySearchInputDto {
  query: string
  student_id?: number | null
  subject_id?: number | null
  topic_id?: number | null
  include_bundles: boolean
  include_questions: boolean
  include_confusions: boolean
  include_audio_ready_only: boolean
}

export interface GlossarySearchResultDto {
  result_type: string
  entry_id: number | null
  bundle_id: number | null
  question_id: number | null
  title: string
  subtitle: string | null
  entry_type: string | null
  topic_id: number | null
  topic_name: string | null
  match_reason: string
  match_score: number
  metadata: Record<string, unknown>
}

export interface GlossarySearchGroupDto {
  group_key: string
  title: string
  results: GlossarySearchResultDto[]
}

export interface GlossarySearchResponseDto {
  normalized_query: string
  query_intent: string
  groups: GlossarySearchGroupDto[]
}

export interface GlossarySearchSuggestionDto {
  suggestion: string
  suggestion_type: string
  entry_id: number | null
  bundle_id: number | null
  score: number
}

export interface EntryAliasDto {
  alias_text: string
  alias_type: string
}

export interface EntryContentBlockDto {
  id: number
  block_type: string
  order_index: number
  content: unknown
}

export interface DefinitionMetaDto {
  definition_text: string
  short_definition: string | null
  formal_definition: string | null
  plain_english_definition: string | null
  real_world_meaning: string | null
  non_examples: string | null
  context_clues: string | null
  pronunciation_text: string | null
}

export interface FormulaMetaDto {
  formula_expression: string
  formula_speech: string | null
  formula_latex: string | null
  variables: unknown
  units: unknown
  when_to_use: string | null
  when_not_to_use: string | null
  rearrangements: string[]
  assumptions: string[]
  common_errors: string[]
  derivation_summary: string | null
  worked_example_ids: number[]
}

export interface ConceptMetaDto {
  concept_explanation: string
  intuition_summary: string | null
  related_visual_keywords: string[]
  misconception_signals: string[]
  why_it_matters: string | null
  mastery_indicators: string[]
}

export interface EntryExampleDto {
  id: number
  sequence_order: number
  example_text: string
  context_type: string
  difficulty_level: number
  worked_solution_text: string | null
  is_exam_style: boolean
}

export interface EntryMisconceptionDto {
  id: number
  misconception_text: string
  cause_explanation: string | null
  correction_explanation: string | null
  confusion_pair_entry_id: number | null
  misconception_source: string | null
  severity_bp: number
}

export interface KnowledgeBundleSequenceItemDto {
  bundle_id: number
  title: string
  bundle_type: string
  sequence_order: number
  focus_reason: string
  due_review_count: number
  focus_entry_ids: number[]
  focus_entry_titles: string[]
}

export interface EntryBundleReferenceDto {
  bundle_id: number
  title: string
  bundle_type: string
  description: string | null
  item_role: string | null
  sequence_order: number | null
  required: boolean
  difficulty_level: number
  exam_relevance_score: number
}

export interface LinkedQuestionSummaryDto {
  question_id: number
  relation_type: string
  confidence_score: number
  is_primary: boolean
  link_source: string
  link_reason: string | null
  stem: string | null
}

export interface KnowledgeRelationLinkDto {
  relation_id: number
  relation_type: string
  strength_score: number
  explanation: string | null
  from_entry_id: number
  to_entry_id: number
  related_entry_title: string
  related_entry_type: string
  related_topic_id: number | null
}

export interface StudentEntrySnapshotDto {
  user_id: number
  familiarity_state: string | null
  mastery_score: number
  confusion_score: number
  recall_strength: number
  open_count: number
  linked_wrong_answer_count: number
  recognition_score: number
  connection_score: number
  application_score: number
  retention_score: number
  test_count: number
  test_pass_count: number
  last_viewed_at: string | null
  last_played_at: string | null
  last_tested_at: string | null
  review_due_at: string | null
  spaced_review_due_at: string | null
  at_risk_threshold_date: string | null
  mastery_state: string | null
  at_risk_flag: boolean
}

export interface ConfusionPairDetailDto {
  paired_entry_id: number
  paired_entry_title: string
  distinction_explanation: string
  common_confusion_reason: string | null
  clue_to_distinguish: string | null
  example_sentence_1: string | null
  example_sentence_2: string | null
  confusion_frequency_bp: number
}

export interface NeighborIntruderMappingDto {
  neighbors: number[]
  intruders: number[]
}

export interface GlossaryAudioSegmentDto {
  sequence_no: number
  segment_type: string
  title: string
  script_text: string
  entry_id: number | null
  prompt_text: string | null
  focus_reason: string | null
  duration_seconds: number
}

export interface GlossaryAudioProgramDto {
  program_title: string
  source_type: string
  teaching_mode: string
  topic_id: number | null
  question_id: number | null
  bundle_ids: number[]
  recommended_bundles: KnowledgeBundleSequenceItemDto[]
  entry_ids: number[]
  listener_signals: string[]
  contrast_titles: string[]
  review_entry_ids: number[]
  review_entry_titles: string[]
  relationship_review_prompts: string[]
  segments: GlossaryAudioSegmentDto[]
}

export interface GlossaryEntryDetailDto {
  entry: KnowledgeEntryProfileDto
  aliases: EntryAliasDto[]
  content_blocks: EntryContentBlockDto[]
  definition_meta: DefinitionMetaDto | null
  formula_meta: FormulaMetaDto | null
  concept_meta: ConceptMetaDto | null
  examples: EntryExampleDto[]
  misconceptions: EntryMisconceptionDto[]
  relations: KnowledgeRelationLinkDto[]
  bundles: EntryBundleReferenceDto[]
  linked_questions: LinkedQuestionSummaryDto[]
  audio_segments: GlossaryAudioSegmentDto[]
  student_state: StudentEntrySnapshotDto | null
  confusion_pairs: ConfusionPairDetailDto[]
  neighbor_intruder: NeighborIntruderMappingDto | null
}

export interface KnowledgeEntryProfileDto {
  id: number
  subject_id: number | null
  topic_id: number | null
  subtopic_id: number | null
  entry_type: string
  title: string
  canonical_name: string | null
  slug: string | null
  short_text: string | null
  full_text: string | null
  simple_text: string | null
  technical_text: string | null
  exam_text: string | null
  importance_score: number
  difficulty_level: number
  grade_band: string | null
  status: string
  audio_available: boolean
  has_formula: boolean
  confusion_pair_count: number
  example_count: number
  misconception_count: number
  exam_relevance_score: number
  priority_score: number
  phonetic_text: string | null
}

export interface GlossaryInteractionInputDto {
  student_id?: number | null
  entry_id?: number | null
  bundle_id?: number | null
  question_id?: number | null
  event_type: string
  query_text?: string | null
  metadata: Record<string, unknown>
}

export interface StartGlossaryAudioQueueInputDto {
  source_type: string
  source_id: number
  limit: number
  teaching_mode?: string | null
  include_examples: boolean
  include_misconceptions: boolean
}

export interface UpdateGlossaryAudioQueueInputDto {
  playback_speed?: number | null
  include_examples?: boolean | null
  include_misconceptions?: boolean | null
  is_playing?: boolean | null
}

export interface GlossaryAudioQueueSnapshotDto {
  current_program_id: number | null
  current_position: number
  is_playing: boolean
  playback_speed: number
  include_examples: boolean
  include_misconceptions: boolean
  current_segment: GlossaryAudioSegmentDto | null
  program: GlossaryAudioProgramDto | null
}

export interface GlossaryEntryFocusDto {
  entry_id: number
  title: string
  entry_type: string
  topic_id: number | null
  topic_name: string | null
  need_score: number
  exam_relevance_score: number
  match_reason: string
  mastery_state: string | null
}

export interface GlossaryHomeSnapshotDto {
  discover: GlossaryEntryFocusDto[]
  weak_entries: GlossaryEntryFocusDto[]
  exam_hotspots: GlossaryEntryFocusDto[]
  recommended_bundles: KnowledgeBundleSequenceItemDto[]
  audio_station_labels: string[]
}

export interface GlossaryComparisonViewDto {
  left_entry: KnowledgeEntryProfileDto
  right_entry: KnowledgeEntryProfileDto
  shared_relation_types: string[]
  distinction_explanation: string | null
  clue_to_distinguish: string | null
  shared_bundles: EntryBundleReferenceDto[]
  linked_question_ids: number[]
}

export interface FormulaLabViewDto {
  entry: KnowledgeEntryProfileDto
  formula_meta: FormulaMetaDto
  related_entries: KnowledgeRelationLinkDto[]
  examples: EntryExampleDto[]
  misconceptions: EntryMisconceptionDto[]
  linked_questions: LinkedQuestionSummaryDto[]
}

export interface GlossaryTestItemDto {
  sequence_no: number
  entry_id: number
  prompt_type: string
  prompt_text: string
  expected_answer: string | null
  options: string[]
  metadata: Record<string, unknown>
}

export interface CreateGlossaryTestInputDto {
  test_mode: string
  topic_id?: number | null
  bundle_id?: number | null
  entry_ids: number[]
  entry_count: number
  duration_seconds?: number | null
  difficulty_level?: number | null
}

export interface SubmitGlossaryTestAttemptInputDto {
  entry_id: number
  student_response: string
  time_seconds?: number | null
}

export interface GlossaryTestAttemptResultDto {
  is_correct: boolean
  feedback: string
  updated_mastery_state: string | null
  review_due_at: string | null
  mastery_score: number
  recognition_score: number
  connection_score: number
  application_score: number
  retention_score: number
}

export interface GlossaryTestSessionDetailDto {
  session_id: number
  student_id: number
  test_mode: string
  topic_id: number | null
  bundle_id: number | null
  entry_count: number
  duration_seconds: number | null
  difficulty_level: number
  recall_score_bp: number | null
  recognition_score_bp: number | null
  connection_score_bp: number | null
  application_score_bp: number | null
  retention_score_bp: number | null
  confidence_score_bp: number | null
  completion_rate_bp: number
  items: GlossaryTestItemDto[]
}

export interface TopicRelationshipHintDto {
  relation_type: string
  from_title: string
  to_title: string
  explanation: string
  hop_count: number
  strength_score: number
  focus_topic_id: number | null
}

export interface TeachActionStepDto {
  sequence_no: number
  step_type: string
  title: string
  prompt: string
  linked_question_ids: number[]
  linked_entry_ids: number[]
  focus_labels: string[]
}

export interface TeachActionPlanDto {
  student_id: number
  topic_id: number
  topic_name: string
  action_type: string
  readiness_band: string
  support_intensity: string
  mastery_score: number
  gap_score: number
  fragile_memory_count: number
  primary_prompt: string
  diagnostic_focuses: string[]
  recent_diagnoses: string[]
  linked_question_ids: number[]
  linked_entry_ids: number[]
  linked_entry_titles: string[]
  target_node_titles: string[]
  relationship_hints: TopicRelationshipHintDto[]
  recommended_sequence: TeachActionStepDto[]
}

export interface TeachExplanationDto {
  id: number
  node_id: number
  topic_id: number | null
  topic_name: string | null
  explanation_level: string
  hero_summary: string | null
  why_it_matters: string | null
  simple_explanation: string | null
  structured_breakdown: unknown
  worked_examples: unknown
  common_mistakes: unknown
  exam_appearance_notes: string | null
  pattern_recognition_tips: string | null
  related_concepts: unknown
  visual_asset_refs: unknown
  subject_style: string | null
  created_at: string
  updated_at: string
}

export interface TeachMicroCheckDto {
  id: number
  explanation_id: number
  check_type: string
  prompt: string
  correct_answer: string
  distractor_answers: string[]
  explanation_if_wrong: string | null
  position_index: number
  created_at: string
}

export interface TeachLessonDto {
  topic_id: number
  topic_name: string
  node_id: number | null
  node_title: string | null
  explanation_level: string
  explanation: TeachExplanationDto | null
  micro_checks: TeachMicroCheckDto[]
  generated: boolean
}

export function getLibraryHome(studentId: number): Promise<LibraryShelfDto[]> {
  return ipc<LibraryShelfDto[]>('get_library_home', { studentId })
}

export function getLibrarySnapshot(studentId: number): Promise<LibraryHomeSnapshotDto> {
  return ipc<LibraryHomeSnapshotDto>('get_library_snapshot', { studentId })
}

export function getContinueLearningCard(studentId: number): Promise<ContinueLearningCardDto | null> {
  return ipc<ContinueLearningCardDto | null>('get_continue_learning_card', { studentId })
}

export function listLibraryItems(studentId: number): Promise<LibraryItemRecordDto[]> {
  return ipc<LibraryItemRecordDto[]>('list_library_items', { studentId })
}

export function listRevisionPacks(
  studentId: number,
  limit: number = 6,
): Promise<RevisionPackSummaryDto[]> {
  return ipc<RevisionPackSummaryDto[]>('list_revision_packs', { studentId, limit })
}

export function listExamHotspots(
  studentId: number,
  subjectId: number | null = null,
  limit: number = 8,
): Promise<ExamHotspotDto[]> {
  return ipc<ExamHotspotDto[]>('list_exam_hotspots', { studentId, subjectId, limit })
}

export function buildRevisionPack(
  studentId: number,
  title: string,
  questionLimit: number = 12,
): Promise<RevisionPackSummaryDto> {
  return ipc<RevisionPackSummaryDto>('build_revision_pack', {
    studentId,
    title,
    questionLimit,
  })
}

export function searchLibrary(
  studentId: number,
  input: LibrarySearchInputDto,
  limit: number = 12,
): Promise<LibrarySearchResultDto[]> {
  return ipc<LibrarySearchResultDto[]>('search_library', {
    studentId,
    input,
    limit,
  })
}

export function saveLibraryItem(
  studentId: number,
  itemType: string,
  referenceId: number,
): Promise<number> {
  return ipc<number>('save_library_item', {
    studentId,
    itemType,
    referenceId,
  })
}

export function saveLibraryItemWithMetadata(
  studentId: number,
  input: SaveLibraryItemInputDto,
): Promise<number> {
  return ipc<number>('save_library_item_with_metadata', {
    studentId,
    input,
  })
}

export function searchGlossary(query: string): Promise<GlossaryEntryDto[]> {
  return ipc<GlossaryEntryDto[]>('search_glossary', { query })
}

export function searchCatalog(
  input: GlossarySearchInputDto,
  limit: number = 12,
): Promise<GlossarySearchResponseDto> {
  return ipc<GlossarySearchResponseDto>('search_catalog', { input, limit })
}

export function searchSuggestions(
  query: string,
  limit: number = 8,
): Promise<GlossarySearchSuggestionDto[]> {
  return ipc<GlossarySearchSuggestionDto[]>('search_suggestions', { query, limit })
}

export function getGlossaryEntryDetail(
  studentId: number | null,
  entryId: number,
  relationLimit: number = 8,
  bundleLimit: number = 4,
): Promise<GlossaryEntryDetailDto> {
  return ipc<GlossaryEntryDetailDto>('get_entry_detail', {
    studentId,
    entryId,
    relationLimit,
    bundleLimit,
  })
}

export function buildGlossaryHomeSnapshot(
  studentId: number,
  subjectId: number | null = null,
  limit: number = 12,
): Promise<GlossaryHomeSnapshotDto> {
  return ipc<GlossaryHomeSnapshotDto>('build_home_snapshot', {
    studentId,
    subjectId,
    limit,
  })
}

export function buildGlossaryCompareView(
  leftEntryId: number,
  rightEntryId: number,
): Promise<GlossaryComparisonViewDto> {
  return ipc<GlossaryComparisonViewDto>('build_compare_view', {
    leftEntryId,
    rightEntryId,
  })
}

export function getFormulaLabView(entryId: number): Promise<FormulaLabViewDto> {
  return ipc<FormulaLabViewDto>('get_formula_lab', { entryId })
}

export function recordGlossaryInteraction(
  input: GlossaryInteractionInputDto,
): Promise<number> {
  return ipc<number>('record_interaction', { input })
}

export function startGlossaryAudioQueue(
  studentId: number,
  input: StartGlossaryAudioQueueInputDto,
): Promise<GlossaryAudioQueueSnapshotDto> {
  return ipc<GlossaryAudioQueueSnapshotDto>('start_audio_queue', { studentId, input })
}

export function currentGlossaryAudioQueue(
  studentId: number,
): Promise<GlossaryAudioQueueSnapshotDto> {
  return ipc<GlossaryAudioQueueSnapshotDto>('current_audio_queue', { studentId })
}

export function nextGlossaryAudioQueue(
  studentId: number,
): Promise<GlossaryAudioQueueSnapshotDto> {
  return ipc<GlossaryAudioQueueSnapshotDto>('next_audio_queue', { studentId })
}

export function previousGlossaryAudioQueue(
  studentId: number,
): Promise<GlossaryAudioQueueSnapshotDto> {
  return ipc<GlossaryAudioQueueSnapshotDto>('previous_audio_queue', { studentId })
}

export function updateGlossaryAudioQueue(
  studentId: number,
  input: UpdateGlossaryAudioQueueInputDto,
): Promise<GlossaryAudioQueueSnapshotDto> {
  return ipc<GlossaryAudioQueueSnapshotDto>('update_audio_queue', { studentId, input })
}

export function createGlossaryTestSession(
  studentId: number,
  input: CreateGlossaryTestInputDto,
): Promise<GlossaryTestSessionDetailDto> {
  return ipc<GlossaryTestSessionDetailDto>('create_glossary_test_session', { studentId, input })
}

export function getGlossaryTestSession(
  sessionId: number,
): Promise<GlossaryTestSessionDetailDto> {
  return ipc<GlossaryTestSessionDetailDto>('get_glossary_test_session', { sessionId })
}

export function submitGlossaryTestAttempt(
  studentId: number,
  sessionId: number,
  input: SubmitGlossaryTestAttemptInputDto,
): Promise<GlossaryTestAttemptResultDto> {
  return ipc<GlossaryTestAttemptResultDto>('submit_glossary_test_attempt', {
    studentId,
    sessionId,
    input,
  })
}

export function buildTeachActionPlan(
  studentId: number,
  topicId: number,
  limit: number = 8,
): Promise<TeachActionPlanDto> {
  return ipc<TeachActionPlanDto>('build_teach_action_plan', {
    studentId,
    topicId,
    limit,
  })
}

export function getTeachLesson(
  topicId: number,
  explanationLevel: string | null = null,
  microCheckLimit: number = 4,
): Promise<TeachLessonDto> {
  return ipc<TeachLessonDto>('get_teach_lesson', {
    topicId,
    explanationLevel,
    microCheckLimit,
  })
}
