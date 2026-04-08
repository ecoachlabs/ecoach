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
