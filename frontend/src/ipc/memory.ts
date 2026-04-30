import { ipc } from '.'

export interface RecheckItemDto {
  id: number
  topic_id: number | null
  node_id: number | null
  topic_name: string | null
  node_title: string | null
  memory_state: string | null
  memory_strength: number | null
  decay_risk: number | null
  due_at: string
  schedule_type: string
}

export interface MemoryDashboardDto {
  total_items: number
  healthy_count: number
  at_risk_count: number
  fading_count: number
  collapsed_count: number
  overdue_reviews: number
  average_strength: number
  next_review_due: string | null
}

export interface TopicMemorySummaryDto {
  topic_id: number
  topic_name: string
  total_items: number
  healthy_items: number
  fragile_items: number
  collapsed_items: number
  overdue_reviews: number
  average_strength: number
  next_review_due: string | null
  recommended_action: string
}

export function getMemoryDashboard(
  studentId: number,
  subjectId?: number | null,
): Promise<MemoryDashboardDto> {
  return ipc<MemoryDashboardDto>('get_memory_dashboard', { studentId, subjectId })
}

export function getReviewQueue(
  studentId: number,
  limit: number = 30,
  subjectId?: number | null,
): Promise<RecheckItemDto[]> {
  return ipc<RecheckItemDto[]>('get_review_queue', { studentId, limit, subjectId })
}

export function listMemoryTopicSummaries(
  studentId: number,
  limit: number = 20,
  subjectId?: number | null,
): Promise<TopicMemorySummaryDto[]> {
  return ipc<TopicMemorySummaryDto[]>('list_memory_topic_summaries', { studentId, limit, subjectId })
}

export function recordRetrievalAttempt(input: Record<string, unknown>): Promise<unknown> {
  return ipc('record_retrieval_attempt', { input })
}

export function processDecayBatch(limit: number = 100): Promise<unknown> {
  return ipc('process_decay_batch', { limit })
}

export function completeRecheck(recheckId: number): Promise<unknown> {
  return ipc('complete_recheck', { recheckId })
}
