import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useCalendar(studentId: number) {
  const loading = ref(false)
  const error = ref('')
  const examTimeline = ref<any>(null)
  const weeklyPlan = ref<any>(null)
  const dailyPlan = ref<any>(null)
  const goals = ref<any[]>([])

  async function loadTimeline() {
    try {
      examTimeline.value = await ipc<any>('get_exam_timeline', { studentId })
    } catch {}
  }

  async function loadWeeklyPlan() {
    try {
      weeklyPlan.value = await ipc<any>('get_weekly_plan', { studentId })
    } catch {}
  }

  async function loadDailyPlan() {
    try {
      dailyPlan.value = await ipc<any>('get_daily_plan', { studentId })
    } catch {}
  }

  async function loadGoals() {
    try {
      goals.value = await ipc<any[]>('list_goals', { studentId })
    } catch {}
  }

  async function addCalendarEvent(event: any) {
    try {
      return await ipc('add_calendar_event', { studentId, event })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to add event'
      return null
    }
  }

  async function loadAll() {
    loading.value = true
    await Promise.all([loadTimeline(), loadWeeklyPlan(), loadDailyPlan(), loadGoals()])
    loading.value = false
  }

  return { loading, error, examTimeline, weeklyPlan, dailyPlan, goals, loadTimeline, loadWeeklyPlan, loadDailyPlan, loadGoals, addCalendarEvent, loadAll }
}
