import { ipc } from '.'

export interface GapScoreCardDto {
  topic_id: number
  topic_name: string
  gap_score: number
  mastery_score: number
  severity_label: string
  repair_priority: number
  has_active_repair_plan: boolean
}

export interface GapRepairPlanDto {
  id: number
  topic_id: number
  topic_name: string | null
  status: string
  priority_score: number
  severity_label: string
  dominant_focus: string
  recommended_session_type: string
  item_count: number
  progress_percent: number
}

export interface GapSnapshotResultDto {
  total_gap_percent: number
  unknown_percent: number
  weak_percent: number
  declining_percent: number
  forgetting_percent: number
  critical_percent: number
  total_skills: number
  mastered_skills: number
  critical_blockers: number
}

export interface GapTrendPointDto {
  gap_percent: number
  snapshot_at: string
}

export interface GapFeedItemDto {
  id: number
  topic_id: number | null
  event_type: string
  message: string
  severity: string
  created_at: string
}

export interface GapDashboardDto {
  critical_gap_count: number
  active_repair_count: number
  topics_solidified: number
  gaps: GapScoreCardDto[]
  repairs: GapRepairPlanDto[]
}

export function listPriorityGaps(studentId: number, limit: number = 20): Promise<GapScoreCardDto[]> {
  return ipc<GapScoreCardDto[]>('list_priority_gaps', { studentId, limit })
}

export function generateRepairPlan(studentId: number, topicId: number): Promise<GapRepairPlanDto> {
  return ipc<GapRepairPlanDto>('generate_repair_plan', { studentId, topicId })
}

export function advanceRepairItem(itemId: number, result: Record<string, unknown>): Promise<GapRepairPlanDto> {
  return ipc<GapRepairPlanDto>('advance_repair_item', { itemId, result })
}

export function getGapDashboard(studentId: number): Promise<GapDashboardDto> {
  return ipc<GapDashboardDto>('get_gap_dashboard', { studentId })
}

export function captureGapSnapshot(
  studentId: number,
  subjectId: number,
): Promise<GapSnapshotResultDto> {
  return ipc<GapSnapshotResultDto>('capture_gap_snapshot', { studentId, subjectId })
}

export function listGapTrend(
  studentId: number,
  subjectId: number,
  limit: number = 12,
): Promise<GapTrendPointDto[]> {
  return ipc<GapTrendPointDto[]>('list_gap_trend', { studentId, subjectId, limit })
}

export function listGapFeed(
  studentId: number,
  subjectId: number,
  limit: number = 10,
): Promise<GapFeedItemDto[]> {
  return ipc<GapFeedItemDto[]>('list_gap_feed', { studentId, subjectId, limit })
}
