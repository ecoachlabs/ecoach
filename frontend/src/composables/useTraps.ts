import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useTraps() {
  const loading = ref(false)
  const error = ref('')
  const pairs = ref<any[]>([])
  const session = ref<any>(null)
  const review = ref<any>(null)

  async function loadPairs(studentId: number, subjectId: number, topicIds: number[]) {
    loading.value = true
    try {
      pairs.value = await ipc<any[]>('list_traps_pairs', { studentId, subjectId, topicIds })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load pairs'
    } finally {
      loading.value = false
    }
  }

  async function startSession(input: any) {
    loading.value = true
    try {
      session.value = await ipc<any>('start_traps_session', { input })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
    } finally {
      loading.value = false
    }
  }

  async function submitRound(input: any) {
    try {
      return await ipc<any>('submit_trap_round', { input })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit'
      return null
    }
  }

  async function getReview(sessionId: number) {
    try {
      review.value = await ipc<any>('get_trap_review', { sessionId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load review'
    }
  }

  async function recordConfusion(input: any) {
    return ipc('record_trap_confusion_reason', { input }).catch(() => {})
  }

  return { loading, error, pairs, session, review, loadPairs, startSession, submitRound, getReview, recordConfusion }
}
