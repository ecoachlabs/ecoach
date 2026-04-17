import { ipc } from '.'

export interface SubjectDto {
  id: number; curriculum_version_id: number; code: string; name: string; display_order: number
}
export interface TopicDto {
  id: number; subject_id: number; parent_topic_id: number | null; code: string | null; name: string; node_type: string; display_order: number
}

export interface CurriculumVersionDto {
  id: number
  curriculum_family_id: number | null
  name: string
  country: string
  exam_board: string | null
  education_stage: string | null
  version_label: string
  status: string
  effective_from: string | null
  effective_to: string | null
  source_summary: Record<string, unknown>
  published_at: string | null
  replaced_by_version_id: number | null
}

export interface CurriculumStudentSubjectCardDto {
  subject_track_id: number
  subject_slug: string
  public_title: string
  entered_percent: number
  stable_percent: number
  exam_ready_percent: number
  weak_area_count: number
  blocked_count: number
  review_due_count: number
  trend_label: string
  strongest_topic_title: string | null
  weakest_topic_title: string | null
  next_action: string | null
}

export interface CurriculumSubjectTrack {
  id: number
  curriculum_version_id: number
  legacy_subject_id: number | null
  subject_code: string
  subject_name: string
  subject_slug: string
  public_title: string
  description: string | null
  display_order: number
}

export interface CurriculumNode {
  id: number
  curriculum_version_id: number
  subject_track_id: number
  level_id: number | null
  term_id: number | null
  parent_node_id: number | null
  legacy_topic_id: number | null
  node_type: string
  canonical_title: string
  public_title: string
  slug: string
  official_text: string | null
  public_summary: string | null
  sequence_no: number
  depth: number
  estimated_weight: number
  exam_relevance_score: number
  difficulty_hint: string
  status: string
  review_status: string
  confidence_score: number
}

export interface CurriculumRecommendation {
  node_id: number
  slug: string
  public_title: string
  rationale: string
  priority_score: number
  blocked_by_prerequisites: string[]
}

export interface CurriculumStudentNodeState {
  node: CurriculumNode
  status_label: string
  blocked: boolean
  review_due: boolean
  exam_ready: boolean
  reason: string
  downstream_titles: string[]
}

export interface CurriculumStudentHomeSnapshot {
  student_id: number
  curriculum_version: CurriculumVersionDto
  subject_cards: CurriculumStudentSubjectCardDto[]
  entered_percent: number
  stable_percent: number
  exam_readiness_percent: number
  weak_topics_count: number
  blocked_topics_count: number
  review_due_count: number
  position_statement: string
  recent_movements: string[]
  recommended_topics: CurriculumRecommendation[]
}

export interface CurriculumStudentSubjectMap {
  student_id: number
  subject: CurriculumSubjectTrack
  overview: CurriculumStudentSubjectCardDto
  nodes: CurriculumStudentNodeState[]
  recommended_topics: CurriculumRecommendation[]
}

export interface CurriculumParentSummaryDto {
  parent_id: number
  learner_id: number
  curriculum_version: CurriculumVersionDto
  subject_cards: CurriculumStudentSubjectCardDto[]
  on_track: boolean
  weak_topics: string[]
  overdue_topics: string[]
  exam_risk_by_subject: Record<string, string | number>
  summary_text: string
}

export function listSubjects(versionId: number = 1): Promise<SubjectDto[]> {
  return ipc<SubjectDto[]>('list_subjects', { curriculumVersionId: versionId })
}

export function listTopics(subjectId: number): Promise<TopicDto[]> {
  return ipc<TopicDto[]>('list_topics', { subjectId })
}

export function getParentCurriculumSummary(
  parentId: number,
  learnerId: number,
  curriculumVersionId: number | null = null,
): Promise<CurriculumParentSummaryDto> {
  return ipc<CurriculumParentSummaryDto>('get_parent_curriculum_summary', {
    parentId,
    learnerId,
    curriculumVersionId,
  })
}

export function getStudentCurriculumHome(
  studentId: number,
  curriculumVersionId: number | null = null,
): Promise<CurriculumStudentHomeSnapshot> {
  return ipc<CurriculumStudentHomeSnapshot>('get_student_curriculum_home', {
    studentId,
    curriculumVersionId,
  })
}

export function getStudentSubjectCurriculumMap(
  studentId: number,
  subjectTrackId: number,
): Promise<CurriculumStudentSubjectMap> {
  return ipc<CurriculumStudentSubjectMap>('get_student_subject_curriculum_map', {
    studentId,
    subjectTrackId,
  })
}

// Admin curriculum commands
export function registerCurriculumSource(input: any): Promise<any> {
  return ipc('register_curriculum_source', { input })
}

export function addParseCandiate(sourceId: number, input: any): Promise<any> {
  return ipc('add_curriculum_parse_candidate', { sourceId, input })
}

export function finalizeCurriculumSource(sourceId: number): Promise<any> {
  return ipc('finalize_curriculum_source', { sourceId })
}

export function markSourceReviewed(sourceId: number): Promise<any> {
  return ipc('mark_curriculum_source_reviewed', { sourceId })
}
