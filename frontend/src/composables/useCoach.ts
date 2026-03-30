import { ref, computed } from 'vue'
import {
  getCoachState, getCoachNextAction, getContentReadiness, getStudentDashboard, getPriorityTopics, getLearnerTruth,
  type CoachStateDto, type CoachNextActionDto, type ContentReadinessDto, type StudentDashboardDto, type TopicCaseDto, type LearnerTruthDto,
} from '@/ipc/coach'

export function useCoach(studentId: number) {
  const loading = ref(false)
  const error = ref('')

  const state = ref<CoachStateDto | null>(null)
  const nextAction = ref<CoachNextActionDto | null>(null)
  const contentReadiness = ref<ContentReadinessDto | null>(null)
  const dashboard = ref<StudentDashboardDto | null>(null)
  const priorityTopics = ref<TopicCaseDto[]>([])
  const truth = ref<LearnerTruthDto | null>(null)

  const journeyState = computed(() => state.value?.state ?? null)
  const isContentReady = computed(() => contentReadiness.value?.status === 'Ready')
  const overallReadiness = computed(() => dashboard.value?.overall_readiness_band ?? 'unknown')

  async function loadAll() {
    loading.value = true
    error.value = ''
    try {
      const [s, a, cr, d, pt, t] = await Promise.all([
        getCoachState(studentId),
        getCoachNextAction(studentId),
        getContentReadiness(studentId),
        getStudentDashboard(studentId),
        getPriorityTopics(studentId, 10),
        getLearnerTruth(studentId),
      ])
      state.value = s
      nextAction.value = a
      contentReadiness.value = cr
      dashboard.value = d
      priorityTopics.value = pt
      truth.value = t
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load coach data'
    } finally {
      loading.value = false
    }
  }

  async function refreshAction() {
    try {
      nextAction.value = await getCoachNextAction(studentId)
    } catch {}
  }

  return {
    loading, error,
    state, nextAction, contentReadiness, dashboard, priorityTopics, truth,
    journeyState, isContentReady, overallReadiness,
    loadAll, refreshAction,
  }
}
