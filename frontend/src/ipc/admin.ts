import { ipc } from '.'

export interface CountDto {
  label: string
  count: number
}

export interface AdminQuestionBankStatsDto {
  total_questions: number
  active_questions: number
  total_options: number
  total_attempts: number
  family_count: number
  installed_pack_count: number
  source_upload_count: number
  pending_review_count: number
  approved_review_count: number
  by_format: CountDto[]
  by_source_type: CountDto[]
  by_review_status: CountDto[]
}

export interface AdminQuestionListFilter {
  search?: string | null
  subject_id?: number | null
  topic_id?: number | null
  review_status?: string | null
  source_type?: string | null
  active_status?: 'active' | 'archived' | 'all' | null
  limit: number
}

export interface AdminQuestionListItemDto {
  question_id: number
  subject_id: number
  subject_name: string
  topic_id: number
  topic_name: string
  family_id: number | null
  family_name: string | null
  stem: string
  question_format: string
  difficulty_level: number
  marks: number
  source_type: string
  review_status: string
  machine_confidence_score: number
  option_count: number
  attempt_count: number
  correct_count: number
  average_response_time_ms: number | null
  is_active: boolean
  updated_at: string
}

export interface AdminQuestionOptionInput {
  id?: number | null
  option_label: string
  option_text: string
  is_correct: boolean
  misconception_id?: number | null
  distractor_intent?: string | null
  position?: number | null
}

export interface AdminQuestionOptionDto {
  id: number
  option_label: string
  option_text: string
  is_correct: boolean
  misconception_id: number | null
  distractor_intent: string | null
  position: number
}

export interface AdminQuestionUpsertInput {
  question_id?: number | null
  subject_id: number
  topic_id: number
  subtopic_id?: number | null
  family_id?: number | null
  stem: string
  question_format: string
  explanation_text?: string | null
  difficulty_level: number
  estimated_time_seconds: number
  marks: number
  source_type: string
  source_ref?: string | null
  exam_year?: number | null
  primary_knowledge_role?: string | null
  primary_cognitive_demand?: string | null
  primary_solve_pattern?: string | null
  primary_pedagogic_function?: string | null
  primary_content_grain?: string | null
  cognitive_level?: string | null
  options: AdminQuestionOptionInput[]
}

export interface AdminQuestionUpsertResultDto {
  question_id: number
  stem: string
  option_count: number
  review_status: string
  machine_confidence_score: number
}

export interface AdminQuestionArchiveResultDto {
  question_id: number
  is_active: boolean
  updated_at: string
}

export interface AdminQuestionBulkActionInput {
  question_ids: number[]
  action: 'archive' | 'restore' | 'move'
  subject_id?: number | null
  topic_id?: number | null
}

export interface AdminQuestionBulkActionResultDto {
  requested_count: number
  updated_count: number
  active_count: number
  archived_count: number
}

export interface AdminQuestionEditorDto {
  question_id: number
  subject_id: number
  topic_id: number
  subtopic_id: number | null
  family_id: number | null
  stem: string
  question_format: string
  explanation_text: string | null
  difficulty_level: number
  estimated_time_seconds: number
  marks: number
  source_type: string
  source_ref: string | null
  exam_year: number | null
  primary_knowledge_role: string | null
  primary_cognitive_demand: string | null
  primary_solve_pattern: string | null
  primary_pedagogic_function: string | null
  primary_content_grain: string | null
  cognitive_level: string | null
  review_status: string
  machine_confidence_score: number
  is_active: boolean
  options: AdminQuestionOptionDto[]
}

export interface QuestionReviewQueueItemDto {
  question_id: number
  stem: string
  topic_id: number
  machine_confidence_score: number
  review_status: string
  review_reason: string | null
  family_candidate: any | null
  misconception_candidates: any[]
}

export interface ContentHealthReadModelDto {
  generated_at: string
  source_count: number
  stale_source_count: number
  overdue_source_review_count: number
  active_mission_count: number
  mission_review_required_count: number
  blocked_publish_count: number
  preview_publish_count: number
  average_quality_bp: number
  low_quality_topics: any[]
  action_recommendations: any[]
}

export interface SourceUploadInput {
  uploader_account_id: number
  source_kind: string
  title: string
  source_path?: string | null
  country_code?: string | null
  exam_board?: string | null
  education_level?: string | null
  subject_code?: string | null
  academic_year?: string | null
  language_code?: string | null
  version_label?: string | null
  metadata?: Record<string, unknown>
}

export interface CurriculumSourceUploadDto {
  id: number
  source_kind: string
  title: string
  subject_code: string | null
  source_status: string
  confidence_score: number
}

export interface ParseCandidateInput {
  candidate_type: string
  parent_candidate_id?: number | null
  raw_label: string
  normalized_label?: string | null
  payload: unknown
  confidence_score: number
}

export interface CurriculumParseCandidateDto {
  id: number
  candidate_type: string
  raw_label: string
  normalized_label: string | null
  confidence_score: number
  review_status: string
}

export interface CurriculumReviewTaskDto {
  id: number
  candidate_id: number | null
  task_type: string
  status: string
  severity: string
  notes: string | null
}

export interface ParseCandidateCountDto {
  candidate_type: string
  count: number
}

export interface ContentSourceRegistryEntryDto {
  id: number
  uploader_account_id?: number
  title: string
  source_kind: string
  source_path?: string | null
  country_code?: string | null
  exam_board?: string | null
  education_level?: string | null
  source_status: string
  subject_code: string | null
  academic_year?: string | null
  language_code?: string
  version_label?: string | null
  confidence_score?: number
  canonical_uri?: string | null
  publisher?: string | null
  author?: string | null
  publication_date?: string | null
  license_type?: string | null
  crawl_permission?: string
  source_tier: string | null
  trust_score_bp: number
  freshness_score_bp: number
  parse_status_detail: string | null
  allowlisted_domain?: boolean
  last_verified_at?: string | null
  stale_flag: boolean
  review_due_at: string | null
  metadata?: unknown
  created_at?: string
  updated_at?: string
}

export interface ContentSourceSegmentDto {
  id: number
  source_upload_id: number
  topic_id: number | null
  concept_id: number | null
  section_title: string | null
  raw_text: string
  normalized_text: string | null
  markdown_text: string | null
  image_refs: unknown
  equation_refs: unknown
  page_range: string | null
  checksum: string | null
  semantic_hash: string | null
  extraction_confidence_bp: number
  relevance_score_bp: number
  metadata: unknown
  created_at: string
}

export interface ContentSourceGovernanceEventDto {
  id: number
  source_upload_id: number
  source_status: string
  decided_by_account_id: number | null
  note: string | null
  created_at: string
}

export interface ContentSourceDetailDto {
  source: ContentSourceRegistryEntryDto
  segments: ContentSourceSegmentDto[]
  missions: any[]
  evidence_records: any[]
  publish_decisions: any[]
  governance_events: ContentSourceGovernanceEventDto[]
}

export interface FoundryJobDto {
  id: number
  subject_id: number | null
  topic_id: number | null
  source_upload_id: number | null
  job_type: string
  target_type: string
  trigger_type: string
  status: string
  priority: number
  result_summary: Record<string, unknown>
  failure_reason: string | null
  created_at: string
  updated_at: string
}

export interface FoundryJobBoardDto {
  queued_count: number
  running_count: number
  blocked_count: number
  failed_count: number
  completed_count: number
  jobs: FoundryJobDto[]
}

export interface ContentFoundrySourceReportDto {
  source_upload: CurriculumSourceUploadDto
  candidate_counts: ParseCandidateCountDto[]
  low_confidence_candidate_count: number
  approved_candidate_count: number
  unresolved_review_count: number
  duplicate_label_count: number
  publish_readiness_score: number
  can_mark_reviewed: boolean
  recommended_actions: string[]
  parse_candidates: CurriculumParseCandidateDto[]
  review_tasks: CurriculumReviewTaskDto[]
  fabric_signals: any[]
  orchestration: any
}

export interface SuperAdminControlTowerDto {
  admin_id: number
  admin_name: string
  generated_at: string
  entitlement_manager: any
  content_health: ContentHealthReadModelDto
  source_objects: any[]
  ocr_recovery_objects: any[]
  pending_shared_promotions: any[]
  recent_areal_guidance: any[]
  action_recommendations: any[]
}

export function getSuperAdminControlTower(adminId: number, limit = 12): Promise<SuperAdminControlTowerDto> {
  return ipc<SuperAdminControlTowerDto>('get_super_admin_control_tower', { adminId, limit })
}

export function getContentHealthReadModel(): Promise<ContentHealthReadModelDto> {
  return ipc<ContentHealthReadModelDto>('get_content_health_read_model')
}

export function listContentSources(status: string | null = null, sourceKind: string | null = null, limit = 50): Promise<ContentSourceRegistryEntryDto[]> {
  return ipc<ContentSourceRegistryEntryDto[]>('list_content_sources', { status, sourceKind, limit })
}

export function getFoundryJobBoard(subjectId: number | null = null): Promise<FoundryJobBoardDto> {
  return ipc<FoundryJobBoardDto>('get_foundry_job_board', { subjectId })
}

export function listFoundryJobs(status: string | null = null, targetType: string | null = null, subjectId: number | null = null): Promise<FoundryJobDto[]> {
  return ipc<FoundryJobDto[]>('list_foundry_jobs', { status, targetType, subjectId })
}

export function runFoundryJob(jobId: number): Promise<FoundryJobDto> {
  return ipc<FoundryJobDto>('run_foundry_job', { jobId })
}

export function runNextFoundryJob(subjectId: number | null = null): Promise<FoundryJobDto | null> {
  return ipc<FoundryJobDto | null>('run_next_foundry_job', { subjectId })
}

export function queueTopicFoundryJobs(topicId: number, triggerType = 'manual_admin_seed'): Promise<FoundryJobDto[]> {
  return ipc<FoundryJobDto[]>('queue_topic_foundry_jobs', { topicId, triggerType })
}

export function queueSourceFollowUpJobs(sourceUploadId: number, triggerType = 'manual_admin_source_follow_up'): Promise<FoundryJobDto[]> {
  return ipc<FoundryJobDto[]>('queue_source_follow_up_jobs', { sourceUploadId, triggerType })
}

export function registerCurriculumSource(input: SourceUploadInput): Promise<CurriculumSourceUploadDto> {
  return ipc<CurriculumSourceUploadDto>('register_curriculum_source', { input })
}

export function getContentFoundrySourceReport(sourceUploadId: number): Promise<ContentFoundrySourceReportDto> {
  return ipc<ContentFoundrySourceReportDto>('get_content_foundry_source_report', { sourceUploadId })
}

export function getContentSourceDetail(sourceUploadId: number): Promise<ContentSourceDetailDto> {
  return ipc<ContentSourceDetailDto>('get_content_source_detail', { sourceUploadId })
}

export function addCurriculumParseCandidate(sourceUploadId: number, input: ParseCandidateInput): Promise<CurriculumParseCandidateDto> {
  return ipc<CurriculumParseCandidateDto>('add_curriculum_parse_candidate', { sourceUploadId, input })
}

export function finalizeCurriculumSource(sourceUploadId: number): Promise<ContentFoundrySourceReportDto> {
  return ipc<ContentFoundrySourceReportDto>('finalize_curriculum_source', { sourceUploadId })
}

export function markCurriculumSourceReviewed(sourceUploadId: number): Promise<ContentFoundrySourceReportDto> {
  return ipc<ContentFoundrySourceReportDto>('mark_curriculum_source_reviewed', { sourceUploadId })
}

export function getAdminQuestionBankStats(): Promise<AdminQuestionBankStatsDto> {
  return ipc<AdminQuestionBankStatsDto>('get_admin_question_bank_stats')
}

export function listAdminQuestions(filter: AdminQuestionListFilter): Promise<AdminQuestionListItemDto[]> {
  return ipc<AdminQuestionListItemDto[]>('list_admin_questions', { filter })
}

export function getAdminQuestionEditor(questionId: number): Promise<AdminQuestionEditorDto> {
  return ipc<AdminQuestionEditorDto>('get_admin_question_editor', { questionId })
}

export function getAdminQuestionEditorAny(questionId: number): Promise<AdminQuestionEditorDto> {
  return ipc<AdminQuestionEditorDto>('get_admin_question_editor_any', { questionId })
}

export function listAdminQuestionOptions(questionId: number): Promise<AdminQuestionOptionDto[]> {
  return ipc<AdminQuestionOptionDto[]>('list_admin_question_options', { questionId })
}

export function upsertAdminQuestion(input: AdminQuestionUpsertInput): Promise<AdminQuestionUpsertResultDto> {
  return ipc<AdminQuestionUpsertResultDto>('upsert_admin_question', { input })
}

export function archiveAdminQuestion(questionId: number): Promise<AdminQuestionArchiveResultDto> {
  return ipc<AdminQuestionArchiveResultDto>('archive_admin_question', { questionId })
}

export function restoreAdminQuestion(questionId: number): Promise<AdminQuestionArchiveResultDto> {
  return ipc<AdminQuestionArchiveResultDto>('restore_admin_question', { questionId })
}

export function bulkUpdateAdminQuestions(input: AdminQuestionBulkActionInput): Promise<AdminQuestionBulkActionResultDto> {
  return ipc<AdminQuestionBulkActionResultDto>('bulk_update_admin_questions', { input })
}

export function listQuestionReviewQueue(reviewStatus: string | null = null, limit = 50): Promise<QuestionReviewQueueItemDto[]> {
  return ipc<QuestionReviewQueueItemDto[]>('list_question_review_queue', { reviewStatus, limit })
}

export function reviewQuestionIntelligence(questionId: number, input: any): Promise<any> {
  return ipc('review_question_intelligence', { questionId, input })
}

export function getQuestionIntelligence(questionId: number): Promise<any | null> {
  return ipc('get_question_intelligence', { questionId })
}

export function findQuestionsByIntelligenceFilter(filter: any): Promise<any[]> {
  return ipc('find_questions_by_intelligence_filter', { filter })
}

export function chooseReactorFamily(slotSpec: any): Promise<any | null> {
  return ipc('choose_reactor_family', { slotSpec })
}

export function createQuestionGenerationRequest(input: any): Promise<any> {
  return ipc('create_question_generation_request', { input })
}

export function processQuestionGenerationRequest(requestId: number): Promise<any[]> {
  return ipc('process_question_generation_request', { requestId })
}
