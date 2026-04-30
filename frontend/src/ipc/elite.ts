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

export interface EliteBlueprintTopicTargetDto {
  topic_id: number
  topic_name: string
  domination_score: number
  precision_score: number
  trap_resistance_score: number
  status: string
  selection_reason: string
}

export interface EliteBlueprintFamilyTargetDto {
  family_id: number
  family_code: string | null
  family_name: string
  topic_id: number | null
  topic_name: string | null
  health_status: string | null
  recurrence_score: number
  replacement_score: number
  selection_reason: string
}

export interface EliteTrapBlueprintSignalDto {
  topic_id: number | null
  topic_name: string | null
  confusion_score: number
  similarity_trap_bp: number
  which_is_which_bp: number
  timed_out_count: number
  force_trapsense: boolean
  rationale: string | null
}

export interface EliteBlueprintReportDto {
  blueprint: EliteSessionBlueprintDto
  profile: EliteProfileDto | null
  topic_targets: EliteBlueprintTopicTargetDto[]
  family_targets: EliteBlueprintFamilyTargetDto[]
  trap_signal: EliteTrapBlueprintSignalDto | null
}

export interface EliteSessionScoreDto {
  session_id: number
  student_id: number
  subject_id: number
  session_class: string
  accuracy_score: number
  precision_score: number
  speed_score: number
  depth_score: number
  trap_resistance_score: number
  composure_score: number
  consistency_score: number
  eps_score: number
  session_label: string
  debrief_text: string
  recommended_next_session: string
  metadata: unknown
}

export type ElitePersonalBestRow = [recordType: string, recordValue: number, achievedAt: string]
export type EliteEarnedBadgeRow = [badgeCode: string, badgeName: string, earnedAt: string]

export const eliteUiModeToSessionClass: Record<string, string> = {
  precision: 'precision_lab',
  precision_lab: 'precision_lab',
  sprint: 'elite_sprint',
  elite_sprint: 'elite_sprint',
  depth: 'depth_lab',
  depth_lab: 'depth_lab',
  trapsense: 'trapsense',
  endurance: 'endurance_track',
  endurance_track: 'endurance_track',
  perfect: 'perfect_run',
  perfect_run: 'perfect_run',
  apex: 'apex_mock',
  apex_mock: 'apex_mock',
}

const eliteSessionClassKey = (sessionId: number) => `ecoach-elite-session-class:${sessionId}`
const eliteSessionScoredKey = (sessionId: number) => `ecoach-elite-session-scored:${sessionId}`

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

export function scoreEliteSession(
  studentId: number,
  sessionId: number,
  sessionClass: string,
): Promise<EliteSessionScoreDto> {
  return ipc<EliteSessionScoreDto>('score_elite_session', { studentId, sessionId, sessionClass })
}

export function checkEliteBadges(
  studentId: number,
  subjectId: number,
): Promise<string[]> {
  return ipc<string[]>('check_elite_badges', { studentId, subjectId })
}

export function listElitePersonalBests(
  studentId: number,
  subjectId: number,
): Promise<ElitePersonalBestRow[]> {
  return ipc<ElitePersonalBestRow[]>('list_elite_personal_bests', { studentId, subjectId })
}

export function listEliteEarnedBadges(
  studentId: number,
  subjectId: number,
): Promise<EliteEarnedBadgeRow[]> {
  return ipc<EliteEarnedBadgeRow[]>('list_elite_earned_badges', { studentId, subjectId })
}

export function rememberEliteSessionClass(sessionId: number, sessionClass: string) {
  localStorage.setItem(eliteSessionClassKey(sessionId), sessionClass)
  localStorage.removeItem(eliteSessionScoredKey(sessionId))
}

export function readEliteSessionClass(sessionId: number): string | null {
  return localStorage.getItem(eliteSessionClassKey(sessionId))
}

export function markEliteSessionScored(sessionId: number) {
  localStorage.setItem(eliteSessionScoredKey(sessionId), 'true')
}

export function isEliteSessionScored(sessionId: number) {
  return localStorage.getItem(eliteSessionScoredKey(sessionId)) === 'true'
}

export function clearEliteSessionTracking(sessionId: number) {
  localStorage.removeItem(eliteSessionClassKey(sessionId))
}
