import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SessionSnapshotDto, SessionSummaryDto } from '@/types'

export const useSessionStore = defineStore('session', () => {
  const activeSnapshot = ref<SessionSnapshotDto | null>(null)
  const lastSummary = ref<SessionSummaryDto | null>(null)
  const currentItemIndex = ref(0)
  const answeredCount = ref(0)
  const correctCount = ref(0)
  const startTime = ref<number | null>(null)

  const sessionId = computed(() => activeSnapshot.value?.session_id ?? null)
  const isActive = computed(() => activeSnapshot.value !== null)
  const totalItems = computed(() => activeSnapshot.value?.item_count ?? 0)
  const progress = computed(() => totalItems.value > 0 ? (answeredCount.value / totalItems.value) * 100 : 0)
  const accuracy = computed(() => answeredCount.value > 0 ? (correctCount.value / answeredCount.value) * 100 : 0)
  const elapsedMs = computed(() => startTime.value ? Date.now() - startTime.value : 0)

  function setSnapshot(snapshot: SessionSnapshotDto) {
    activeSnapshot.value = snapshot
    currentItemIndex.value = 0
    answeredCount.value = 0
    correctCount.value = 0
    startTime.value = Date.now()
  }

  function recordAnswer(isCorrect: boolean) {
    answeredCount.value++
    if (isCorrect) correctCount.value++
    currentItemIndex.value++
  }

  function setSummary(summary: SessionSummaryDto) {
    lastSummary.value = summary
  }

  function clear() {
    activeSnapshot.value = null
    lastSummary.value = null
    currentItemIndex.value = 0
    answeredCount.value = 0
    correctCount.value = 0
    startTime.value = null
  }

  return {
    activeSnapshot, lastSummary, currentItemIndex, answeredCount, correctCount, startTime,
    sessionId, isActive, totalItems, progress, accuracy, elapsedMs,
    setSnapshot, recordAnswer, setSummary, clear,
  }
})
