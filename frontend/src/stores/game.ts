import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type GameType = 'mindstack' | 'tugofwar' | 'traps'

export const useGameStore = defineStore('game', () => {
  const activeGame = ref<GameType | null>(null)
  const gameSessionId = ref<number | null>(null)
  const score = ref(0)
  const streak = ref(0)
  const questionsAnswered = ref(0)
  const questionsCorrect = ref(0)
  const isActive = computed(() => activeGame.value !== null)

  function startGame(type: GameType, sessionId: number) {
    activeGame.value = type
    gameSessionId.value = sessionId
    score.value = 0
    streak.value = 0
    questionsAnswered.value = 0
    questionsCorrect.value = 0
  }

  function recordAnswer(correct: boolean, points: number) {
    questionsAnswered.value++
    if (correct) {
      questionsCorrect.value++
      streak.value++
      score.value += points * (1 + streak.value * 0.1) // streak multiplier
    } else {
      streak.value = 0
    }
  }

  function endGame() {
    activeGame.value = null
  }

  return {
    activeGame, gameSessionId, score, streak, questionsAnswered, questionsCorrect, isActive,
    startGame, recordAnswer, endGame,
  }
})
