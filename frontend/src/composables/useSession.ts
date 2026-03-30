import { ref, computed } from 'vue'
import {
  startPracticeSession, composeCustomTest, completeSession,
  type SessionSnapshotDto, type SessionSummaryDto, type PracticeSessionStartInput, type CustomTestStartInput,
} from '@/ipc/sessions'
import { ipc } from '@/ipc'

export function useSession() {
  const snapshot = ref<SessionSnapshotDto | null>(null)
  const summary = ref<SessionSummaryDto | null>(null)
  const loading = ref(false)
  const error = ref('')

  const sessionId = computed(() => snapshot.value?.session_id ?? null)
  const isActive = computed(() => snapshot.value?.status === 'active' || snapshot.value?.status === 'in_progress')
  const totalQuestions = computed(() => snapshot.value?.item_count ?? 0)

  async function startPractice(input: PracticeSessionStartInput) {
    loading.value = true
    error.value = ''
    try {
      snapshot.value = await startPracticeSession(input)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
    } finally {
      loading.value = false
    }
  }

  async function startCustom(input: CustomTestStartInput) {
    loading.value = true
    error.value = ''
    try {
      snapshot.value = await composeCustomTest(input)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start test'
    } finally {
      loading.value = false
    }
  }

  async function complete() {
    if (!sessionId.value) return
    loading.value = true
    try {
      summary.value = await completeSession(sessionId.value)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to complete session'
    } finally {
      loading.value = false
    }
  }

  async function submitAnswer(itemId: number, selectedOptionId: number, responseTimeMs: number) {
    try {
      return await ipc('submit_attempt', {
        input: { item_id: itemId, selected_option_id: selectedOptionId, response_time_ms: responseTimeMs }
      })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit answer'
      return null
    }
  }

  function reset() {
    snapshot.value = null
    summary.value = null
    error.value = ''
  }

  return {
    snapshot, summary, loading, error,
    sessionId, isActive, totalQuestions,
    startPractice, startCustom, complete, submitAnswer, reset,
  }
}
