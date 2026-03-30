import { ipc } from '.'

export interface CoachStateDto {
  state: string
  reason: string
}

export interface CoachNextActionDto {
  state: string
  action_type: string
  title: string
  subtitle: string
  estimated_minutes: number | null
  route: string
}

export interface ContentReadinessDto {
  status: string
  subject_codes: string[]
  active_pack_count: number
  topic_count: number
  question_count: number
  reason: string | null
}

export interface TopicCaseDto {
  topic_id: number
  topic_name: string
  subject_code: string
  priority_score: number
  mastery_score: number
  mastery_state: string
  gap_score: number
  fragility_score: number
  memory_strength: number
  decay_risk: number
  evidence_count: number
  requires_probe: boolean
  intervention_mode: string
  intervention_urgency: string
  intervention_reason: string
}

export interface StudentDashboardDto {
  student_name: string
  exam_target: string | null
  overall_readiness_band: string
  subjects: SubjectSummaryDto[]
}

export interface SubjectSummaryDto {
  subject_id: number
  subject_name: string
  readiness_band: string
  mastered_topic_count: number
  weak_topic_count: number
  total_topic_count: number
}

export interface LearnerTruthDto {
  student_id: number
  student_name: string
  overall_mastery_score: number
  overall_readiness_band: string
  pending_review_count: number
  due_memory_count: number
  topic_count: number
  skill_count: number
  memory_count: number
  diagnosis_count: number
}

export interface SubjectDto {
  id: number
  curriculum_version_id: number
  code: string
  name: string
  display_order: number
}

export interface TopicDto {
  id: number
  subject_id: number
  parent_topic_id: number | null
  code: string | null
  name: string
  node_type: string
  display_order: number
}

// ── Coach Commands ──

export function getCoachState(studentId: number): Promise<CoachStateDto> {
  return ipc<CoachStateDto>('get_coach_state', { studentId })
}

export function getCoachNextAction(studentId: number): Promise<CoachNextActionDto> {
  return ipc<CoachNextActionDto>('get_coach_next_action', { studentId })
}

export function getContentReadiness(studentId: number): Promise<ContentReadinessDto> {
  return ipc<ContentReadinessDto>('get_content_readiness', { studentId })
}

export function getPriorityTopics(studentId: number, limit: number = 10): Promise<TopicCaseDto[]> {
  return ipc<TopicCaseDto[]>('get_priority_topics', { studentId, limit })
}

export function getStudentDashboard(studentId: number): Promise<StudentDashboardDto> {
  return ipc<StudentDashboardDto>('get_student_dashboard', { studentId })
}

export function getLearnerTruth(studentId: number): Promise<LearnerTruthDto> {
  return ipc<LearnerTruthDto>('get_learner_truth', { studentId })
}

// ── Curriculum Commands ──

export function listSubjects(curriculumVersionId: number = 1): Promise<SubjectDto[]> {
  return ipc<SubjectDto[]>('list_subjects', { curriculumVersionId })
}

export function listTopics(subjectId: number): Promise<TopicDto[]> {
  return ipc<TopicDto[]>('list_topics', { subjectId })
}
