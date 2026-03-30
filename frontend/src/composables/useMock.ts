import { ref } from 'vue'
import { ipc } from '@/ipc'
import { generateMockBlueprint, startMockSession, type MockBlueprintInput, type MockBlueprintDto, type SessionSnapshotDto } from '@/ipc/sessions'

export function useMock() {
  const loading = ref(false)
  const error = ref('')
  const blueprint = ref<MockBlueprintDto | null>(null)
  const session = ref<SessionSnapshotDto | null>(null)
  const report = ref<any>(null)

  async function createBlueprint(input: MockBlueprintInput) {
    loading.value = true
    error.value = ''
    try {
      blueprint.value = await generateMockBlueprint(input)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to generate blueprint'
    } finally {
      loading.value = false
    }
  }

  async function startMock(blueprintId: number) {
    loading.value = true
    error.value = ''
    try {
      session.value = await startMockSession(blueprintId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start mock'
    } finally {
      loading.value = false
    }
  }

  async function submitAnswer(sessionId: number, questionId: number, optionId: number, timeMs: number) {
    try {
      return await ipc<any>('submit_mock_answer', {
        input: { session_id: sessionId, question_id: questionId, selected_option_id: optionId, response_time_ms: timeMs }
      })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit'
      return null
    }
  }

  async function getReport(sessionId: number) {
    try {
      report.value = await ipc<any>('get_mock_report', { sessionId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load report'
    }
  }

  return { loading, error, blueprint, session, report, createBlueprint, startMock, submitAnswer, getReport }
}
