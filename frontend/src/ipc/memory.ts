import { ipc } from '.'

export function getMemoryDashboard(studentId: number): Promise<any> {
  return ipc('get_memory_dashboard', { studentId })
}

export function getReviewQueue(studentId: number): Promise<any[]> {
  return ipc('get_review_queue', { studentId })
}

export function recordRetrievalAttempt(input: any): Promise<any> {
  return ipc('record_retrieval_attempt', { input })
}

export function processDecayBatch(studentId: number): Promise<any> {
  return ipc('process_decay_batch', { studentId })
}

export function completeRecheck(input: any): Promise<any> {
  return ipc('complete_recheck', { input })
}
