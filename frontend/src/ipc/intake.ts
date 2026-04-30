import { ipc } from '.'

export interface SubmissionBundleDto {
  id: number
  student_id: number
  title: string
  status: string
}

export interface BundleFileDto {
  id: number
  bundle_id: number
  file_name: string
  file_path: string
  mime_type: string | null
  file_kind: string
}

export interface ExtractedInsightDto {
  id: number
  bundle_id: number
  insight_type: string
  payload: unknown
  created_at: string
}

export interface BundleInboxItemDto {
  bundle: SubmissionBundleDto
  confirmation_state: string
  coach_application_status: string
  review_priority: string
  needs_confirmation: boolean
  detected_subjects: string[]
  detected_topics: string[]
  summary_points: string[]
}

export interface BundleSharedPromotionDto {
  id: number
  bundle_id: number
  source_upload_id: number | null
  requested_by_account_id: number | null
  promotion_status: string
  promotion_summary: unknown
  created_at: string
  updated_at: string
}

export interface PersonalAcademicVaultEntryDto {
  bundle: SubmissionBundleDto
  bundle_kind: string
  review_priority: string
  confirmation_state: string
  coach_application_status: string
  detected_subjects: string[]
  detected_topics: string[]
  summary_points: string[]
  file_count: number
  files: BundleFileDto[]
  promotion: BundleSharedPromotionDto | null
}

export interface PersonalAcademicVaultSnapshotDto {
  student_id: number
  total_bundle_count: number
  pending_review_count: number
  coach_applied_count: number
  promoted_bundle_count: number
  active_topics: string[]
  bundles: PersonalAcademicVaultEntryDto[]
}

export interface BundleProcessReportDto {
  bundle: SubmissionBundleDto
  files: BundleFileDto[]
  insights: ExtractedInsightDto[]
  detected_subjects: string[]
  detected_exam_years: number[]
  detected_topics: string[]
  detected_dates: string[]
  question_like_file_count: number
  answer_like_file_count: number
  ocr_candidate_file_count: number
  ocr_recovered_file_count: number
  layout_recovered_file_count: number
  estimated_question_count: number
  estimated_answer_count: number
  reconstructed_document_count: number
  paired_assessment_document_count: number
  reconstruction_confidence_score: number
  extracted_question_block_count: number
  aligned_question_pair_count: number
  high_confidence_alignment_count: number
  medium_confidence_alignment_count: number
  low_confidence_alignment_count: number
  score_signal_count: number
  remark_signal_count: number
  needs_confirmation: boolean
  unresolved_alignment_count: number
  review_priority: string
  reconstruction_confidence_band: string
  bundle_kind: string
  detected_document_roles: string[]
  weakness_signals: string[]
  recommended_actions: string[]
  review_reasons: string[]
  coach_goal_signals: unknown[]
  topic_action_summaries: unknown[]
  follow_up_recommendations: unknown[]
}

export interface BundleReviewNoteDto {
  id: number
  bundle_id: number
  question_ref: string
  topic_label: string | null
  review_side: string
  reflection_kind: string
  reflection_text: string
  recommended_action: string | null
  severity_bp: number
  created_at: string
}

export interface UploadedReviewItemDto {
  question_ref: string
  topic_label: string | null
  alignment_confidence: string
  weakness_signals: string[]
  coach_explanation: string
  recommended_actions: string[]
  reflections: BundleReviewNoteDto[]
}

export interface UploadedPaperReviewSnapshotDto {
  bundle: SubmissionBundleDto
  coach_impact_summary: unknown
  parent_summary: string[]
  items: UploadedReviewItemDto[]
}

export interface BundleCoachApplicationResultDto {
  bundle_id: number
  coach_application_status: string
  created_goal_ids: number[]
  updated_topic_labels: string[]
  parent_alert_count: number
  question_environment_profile: unknown
  summary: string[]
}

export function createSubmissionBundle(
  studentId: number,
  title: string,
): Promise<SubmissionBundleDto> {
  return ipc<SubmissionBundleDto>('create_submission_bundle', { studentId, title })
}

export function addSubmissionBundleFile(
  bundleId: number,
  fileName: string,
  filePath: string,
): Promise<number> {
  return ipc<number>('add_submission_bundle_file', { bundleId, fileName, filePath })
}

export function reconstructSubmissionBundle(
  bundleId: number,
): Promise<BundleProcessReportDto> {
  return ipc<BundleProcessReportDto>('reconstruct_submission_bundle', { bundleId })
}

export function getSubmissionBundleReport(
  bundleId: number,
): Promise<BundleProcessReportDto> {
  return ipc<BundleProcessReportDto>('get_submission_bundle_report', { bundleId })
}

export function listSubmissionBundleInbox(
  studentId: number,
  limit: number = 8,
): Promise<BundleInboxItemDto[]> {
  return ipc<BundleInboxItemDto[]>('list_submission_bundle_inbox', { studentId, limit })
}

export function getUploadedPaperReview(
  bundleId: number,
): Promise<UploadedPaperReviewSnapshotDto> {
  return ipc<UploadedPaperReviewSnapshotDto>('get_uploaded_paper_review', { bundleId })
}

export function applySubmissionBundleToCoach(
  bundleId: number,
): Promise<BundleCoachApplicationResultDto> {
  return ipc<BundleCoachApplicationResultDto>('apply_submission_bundle_to_coach', { bundleId })
}

export function getPersonalAcademicVault(
  studentId: number,
  limit: number | null = null,
): Promise<PersonalAcademicVaultSnapshotDto> {
  return ipc<PersonalAcademicVaultSnapshotDto>('get_personal_academic_vault', { studentId, limit })
}
