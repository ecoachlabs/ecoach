import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { CoachNextActionDto, CoachStateDto, StudentDashboardDto, TopicCaseDto, LearnerTruthDto } from '@/ipc/coach'

export const useCoachStore = defineStore('coach', () => {
  const state = ref<CoachStateDto | null>(null)
  const nextAction = ref<CoachNextActionDto | null>(null)
  const dashboard = ref<StudentDashboardDto | null>(null)
  const priorityTopics = ref<TopicCaseDto[]>([])
  const truth = ref<LearnerTruthDto | null>(null)
  const loading = ref(false)
  const lastRefresh = ref<Date | null>(null)

  const journeyState = computed(() => state.value?.state ?? null)
  const readinessBand = computed(() => dashboard.value?.overall_readiness_band ?? 'unknown')
  const studentName = computed(() => dashboard.value?.student_name ?? truth.value?.student_name ?? '')
  const subjects = computed(() => dashboard.value?.subjects ?? [])
  const needsOnboarding = computed(() =>
    journeyState.value === 'OnboardingRequired' || journeyState.value === 'SubjectSelectionRequired'
  )

  function setCoachData(data: {
    state?: CoachStateDto
    nextAction?: CoachNextActionDto
    dashboard?: StudentDashboardDto
    priorityTopics?: TopicCaseDto[]
    truth?: LearnerTruthDto
  }) {
    if (data.state) state.value = data.state
    if (data.nextAction) nextAction.value = data.nextAction
    if (data.dashboard) dashboard.value = data.dashboard
    if (data.priorityTopics) priorityTopics.value = data.priorityTopics
    if (data.truth) truth.value = data.truth
    lastRefresh.value = new Date()
  }

  function clear() {
    state.value = null
    nextAction.value = null
    dashboard.value = null
    priorityTopics.value = []
    truth.value = null
    lastRefresh.value = null
  }

  return {
    state, nextAction, dashboard, priorityTopics, truth, loading, lastRefresh,
    journeyState, readinessBand, studentName, subjects, needsOnboarding,
    setCoachData, clear,
  }
})
