import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useGames() {
  const loading = ref(false)
  const error = ref('')
  const gameSession = ref<any>(null)
  const gameSummary = ref<any>(null)

  async function startGame(input: { game_type: string; student_id: number; subject_id: number; topic_ids: number[]; difficulty: string; mode?: string }) {
    loading.value = true
    error.value = ''
    try {
      gameSession.value = await ipc<any>('start_game', { input })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start game'
    } finally {
      loading.value = false
    }
  }

  async function submitAnswer(gameSessionId: number, questionId: number, answer: any) {
    try {
      return await ipc<any>('submit_game_answer', { input: { session_id: gameSessionId, question_id: questionId, answer } })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit'
      return null
    }
  }

  async function getSummary(gameSessionId: number) {
    try {
      gameSummary.value = await ipc<any>('get_game_summary', { sessionId: gameSessionId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load summary'
    }
  }

  async function pauseGame(gameSessionId: number) {
    return ipc('pause_game', { sessionId: gameSessionId }).catch(() => {})
  }

  async function resumeGame(gameSessionId: number) {
    return ipc('resume_game', { sessionId: gameSessionId }).catch(() => {})
  }

  return { loading, error, gameSession, gameSummary, startGame, submitAnswer, getSummary, pauseGame, resumeGame }
}
