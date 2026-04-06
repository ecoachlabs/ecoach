import { ipc } from '.'

export interface EliteProfileDto {
  student_id: number
  subject_id: number
  eps_score: number
  tier: string
  precision_score: number
  speed_score: number
  depth_score: number
  composure_score: number
}

export interface EliteTopicProfileDto {
  topic_id: number
  topic_name: string
  precision_score: number
  speed_score: number
  depth_score: number
  composure_score: number
  consistency_score: number
  trap_resistance_score: number
  domination_score: number
  status: string
}

export interface EliteSessionBlueprintDto {
  student_id: number
  subject_id: number
  session_class: string
  target_topic_ids: number[]
  target_family_ids: number[]
  authoring_modes: string[]
  target_question_count: number
  rationale: string
}

export interface EliteBlueprintReportDto {
  blueprint: EliteSessionBlueprintDto
  profile: EliteProfileDto | null
}

export function getEliteProfile(
  studentId: number,
  subjectId: number,
): Promise<EliteProfileDto | null> {
  return ipc<EliteProfileDto | null>('get_elite_profile', { studentId, subjectId })
}

export function listEliteTopicDomination(
  studentId: number,
  subjectId: number,
  limit: number = 10,
): Promise<EliteTopicProfileDto[]> {
  return ipc<EliteTopicProfileDto[]>('list_elite_topic_domination', { studentId, subjectId, limit })
}

export function buildEliteSessionBlueprint(
  studentId: number,
  subjectId: number,
): Promise<EliteSessionBlueprintDto> {
  return ipc<EliteSessionBlueprintDto>('build_elite_session_blueprint', { studentId, subjectId })
}

export function buildEliteSessionBlueprintReport(
  studentId: number,
  subjectId: number,
): Promise<EliteBlueprintReportDto> {
  return ipc<EliteBlueprintReportDto>('build_elite_session_blueprint_report', { studentId, subjectId })
}
