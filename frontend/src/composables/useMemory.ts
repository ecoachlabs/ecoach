import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useMemory() {
  const loading = ref(false)
  const error = ref('')
  const dashboard = ref<any>(null)
  const reviewQueue = ref<any[]>([])
  const decayBatch = ref<any>(null)

  async function loadDashboard(studentId: number) {
    loading.value = true
    try {
      dashboard.value = await ipc<any>('get_memory_dashboard', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load memory dashboard'
    } finally {
      loading.value = false
    }
  }

  async function loadReviewQueue(studentId: number) {
    try {
      reviewQueue.value = await ipc<any[]>('get_review_queue', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load review queue'
    }
  }

  async function recordRetrieval(input: any) {
    try {
      return await ipc<any>('record_retrieval_attempt', { input })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to record retrieval'
      return null
    }
  }

  async function processDecay(studentId: number) {
    try {
      decayBatch.value = await ipc<any>('process_decay_batch', { studentId })
    } catch {}
  }

  return { loading, error, dashboard, reviewQueue, decayBatch, loadDashboard, loadReviewQueue, recordRetrieval, processDecay }
}
