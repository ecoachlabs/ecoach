<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getCoachNextAction,
  getStudentDashboard,
  getPriorityTopics,
  type CoachNextActionDto,
  type StudentDashboardDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const error = ref('')

const nextAction = ref<CoachNextActionDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topicCases = ref<TopicCaseDto[]>([])

const greeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
})

onMounted(async () => {
  if (!auth.currentAccount) return
  const sid = auth.currentAccount.id

  try {
    const [action, dash, topics] = await Promise.all([
      getCoachNextAction(sid),
      getStudentDashboard(sid),
      getPriorityTopics(sid, 5),
    ])
    nextAction.value = action
    dashboard.value = dash
    topicCases.value = topics
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load coach data'
  } finally {
    loading.value = false
  }
})

function startAction() {
  if (nextAction.value?.route) {
    router.push(nextAction.value.route)
  }
}

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}

const readinessColors: Record<string, string> = {
  strong: 'success', developing: 'gold', weak: 'danger', critical: 'danger',
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">

    <!-- Greeting -->
    <div class="mb-8">
      <p class="text-sm font-medium mb-1" :style="{ color: 'var(--accent)' }">{{ greeting }}</p>
      <h1 class="font-display text-2xl lg:text-3xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">
        {{ dashboard?.student_name || auth.currentAccount?.display_name || 'Student' }}
      </h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">
        Here is what your coach recommends today.
      </p>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-4">
      <div class="h-40 rounded-[var(--radius-xl)] animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div class="grid grid-cols-3 gap-4">
        <div v-for="i in 3" :key="i" class="h-28 rounded-[var(--radius-lg)] animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>

    <!-- Error -->
    <AppCard v-else-if="error" padding="lg">
      <div class="text-center py-8">
        <p class="text-sm" :style="{ color: 'var(--danger)' }">{{ error }}</p>
        <AppButton variant="secondary" size="sm" class="mt-4" @click="$router.go(0)">Retry</AppButton>
      </div>
    </AppCard>

    <template v-else>

      <!-- Primary Coach Directive -->
      <AppCard v-if="nextAction" class="mb-6 relative overflow-hidden" glow="accent" padding="lg">
        <div class="absolute top-0 left-0 w-1 h-full rounded-l-[var(--radius-lg)]" :style="{ backgroundColor: 'var(--accent)' }" />
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-2">
              <AppBadge color="accent" size="xs" dot>Coach Recommendation</AppBadge>
              <AppBadge v-if="nextAction.action_type" color="muted" size="xs">{{ nextAction.action_type.replace(/_/g, ' ') }}</AppBadge>
            </div>
            <h2 class="font-display text-xl font-semibold mb-1.5" :style="{ color: 'var(--text)' }">
              {{ nextAction.title }}
            </h2>
            <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">
              {{ nextAction.subtitle }}
            </p>
          </div>
          <span v-if="nextAction.estimated_minutes" class="text-xs font-medium px-2.5 py-1 rounded-full shrink-0"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            ~{{ nextAction.estimated_minutes }} min
          </span>
        </div>
        <div class="mt-5 flex items-center gap-3">
          <AppButton variant="primary" @click="startAction">Start Now →</AppButton>
          <AppButton variant="ghost" size="sm">Why this?</AppButton>
        </div>
      </AppCard>

      <!-- Subject Cards -->
      <div v-if="dashboard?.subjects?.length" class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Your Subjects</h3>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <AppCard v-for="subject in dashboard.subjects" :key="subject.subject_id" hover padding="md"
            @click="router.push('/student/progress')">
            <div class="flex items-start justify-between mb-3">
              <h4 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.subject_name }}</h4>
              <AppBadge :color="(readinessColors[subject.readiness_band] as any) || 'muted'" size="xs">
                {{ subject.readiness_band }}
              </AppBadge>
            </div>
            <div class="mt-2 text-[11px]" :style="{ color: 'var(--text-3)' }">
              {{ subject.mastered_topic_count }}/{{ subject.total_topic_count }} mastered
              <span v-if="subject.weak_topic_count > 0" class="ml-2" :style="{ color: 'var(--danger)' }">
                {{ subject.weak_topic_count }} weak
              </span>
            </div>
          </AppCard>
        </div>
      </div>

      <!-- Priority Topics -->
      <div v-if="topicCases.length" class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Priority Topics</h3>
        <div class="space-y-2">
          <AppCard v-for="topic in topicCases" :key="topic.topic_id" padding="sm" hover>
            <div class="flex items-center gap-3">
              <MasteryBadge :state="topic.mastery_state" size="sm" glow />
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
                <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">
                  Gap: {{ formatBp(topic.gap_score) }} · Priority: {{ formatBp(topic.priority_score) }}
                </p>
              </div>
              <AppBadge
                :color="topic.intervention_urgency === 'high' ? 'danger' : topic.intervention_urgency === 'medium' ? 'warm' : 'accent'"
                size="xs">
                {{ topic.intervention_mode }}
              </AppBadge>
            </div>
          </AppCard>
        </div>
      </div>

      <!-- Quick Actions -->
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Quick Actions</h3>
        <div class="flex flex-wrap gap-2">
          <AppButton variant="secondary" size="sm" @click="router.push('/student/practice')">✎ Practice</AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/student/mock')">⊞ Mock Test</AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/student/glossary')">⊿ Glossary</AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/student/mistakes')">✕ Mistakes</AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/mastery-map')">◐ Mastery Map</AppButton>
        </div>
      </div>
    </template>
  </div>
</template>
