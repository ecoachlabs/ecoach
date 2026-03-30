import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface ExamEvent {
  id: number
  name: string
  date: string
  subject?: string
  type: 'mock' | 'exam' | 'final' | 'event'
}

export interface DailyPlanItem {
  id: number
  subject: string
  topic: string
  sessionType: string
  duration: number
  status: 'pending' | 'active' | 'completed' | 'skipped'
}

export const useCalendarStore = defineStore('calendar', () => {
  const examEvents = ref<ExamEvent[]>([])
  const dailyPlan = ref<DailyPlanItem[]>([])
  const weeklyAllocations = ref<Record<string, number>>({})
  const currentPhase = ref('build')
  const loading = ref(false)

  const nextExam = computed(() => {
    const now = new Date()
    return examEvents.value
      .filter(e => new Date(e.date) > now)
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())[0] ?? null
  })

  const daysToNextExam = computed(() => {
    if (!nextExam.value) return null
    return Math.ceil((new Date(nextExam.value.date).getTime() - Date.now()) / 86400000)
  })

  function setExamEvents(events: ExamEvent[]) { examEvents.value = events }
  function setDailyPlan(plan: DailyPlanItem[]) { dailyPlan.value = plan }
  function setPhase(phase: string) { currentPhase.value = phase }

  function updatePlanItemStatus(itemId: number, status: DailyPlanItem['status']) {
    const item = dailyPlan.value.find(i => i.id === itemId)
    if (item) item.status = status
  }

  return {
    examEvents, dailyPlan, weeklyAllocations, currentPhase, loading,
    nextExam, daysToNextExam,
    setExamEvents, setDailyPlan, setPhase, updatePlanItemStatus,
  }
})
