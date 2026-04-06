<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getHouseholdDashboard, type HouseholdDashboardSnapshotDto } from '@/ipc/reporting'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const dashboard = ref<HouseholdDashboardSnapshotDto | null>(null)

onMounted(() => {
  void loadDashboard()
})

async function loadDashboard() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    dashboard.value = await getHouseholdDashboard(auth.currentAccount.id)
  } catch (err) {
    error.value = extractError(err, 'Failed to load household dashboard')
  }

  loading.value = false
}

function attentionColor(level: string) {
  const normalized = level.toLowerCase()
  if (normalized === 'high') return 'danger'
  if (normalized === 'medium') return 'warm'
  return 'success'
}

function urgencyColor(level: string) {
  const normalized = level.toLowerCase()
  if (normalized === 'high' || normalized === 'critical') return 'danger'
  if (normalized === 'medium') return 'warm'
  return 'muted'
}

function readinessColor(band: string) {
  const normalized = band.toLowerCase()
  if (normalized.includes('ready') || normalized.includes('strong')) return 'success'
  if (normalized.includes('risk') || normalized.includes('not ready')) return 'danger'
  return 'accent'
}

function formatLabel(value: string | null | undefined) {
  return (value ?? 'Unavailable').replace(/_/g, ' ')
}

function formatDate(value: string | null) {
  if (!value) return 'No exam date set'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })
}

function extractError(err: unknown, fallback: string) {
  if (typeof err === 'string') return err
  if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
    return err.message
  }
  return fallback
}
</script>

<template>
  <div class="max-w-6xl mx-auto reveal-stagger">
    <PageHeader
      title="Household"
      subtitle="See the family-wide attention level, live interventions, and next actions in one place."
    >
      <template #actions>
        <div class="flex flex-wrap gap-2 justify-end">
          <AppButton variant="secondary" size="sm" @click="router.push('/parent/children')">
            Manage Children
          </AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/parent/concierge')">
            Open Concierge
          </AppButton>
        </div>
      </template>
    </PageHeader>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="space-y-4">
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div v-for="i in 3" :key="i" class="h-24 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
      <div v-for="i in 2" :key="'student-' + i" class="h-56 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else-if="dashboard && dashboard.students.length > 0">
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6">
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Attention Level</p>
          <div class="flex items-center gap-2">
            <AppBadge :color="attentionColor(dashboard.household_attention_level) as any" size="sm" dot>
              {{ formatLabel(dashboard.household_attention_level) }}
            </AppBadge>
          </div>
        </AppCard>
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Students Needing Attention</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--text)' }">{{ dashboard.students_needing_attention }}</p>
        </AppCard>
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Active Interventions</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--accent)' }">{{ dashboard.active_interventions }}</p>
        </AppCard>
      </div>

      <div v-if="dashboard.household_actions.length" class="mb-6">
        <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Household Actions</p>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <AppCard
            v-for="action in dashboard.household_actions"
            :key="action.title"
            padding="md"
          >
            <div class="flex items-center justify-between gap-2 mb-2">
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ action.title }}</p>
              <AppBadge :color="urgencyColor(action.urgency) as any" size="xs">
                {{ formatLabel(action.urgency) }}
              </AppBadge>
            </div>
            <p class="text-sm leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">{{ action.detail }}</p>
          </AppCard>
        </div>
      </div>

      <div class="space-y-4">
        <AppCard
          v-for="student in dashboard.students"
          :key="student.student_id"
          padding="lg"
        >
          <div class="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4 mb-5">
            <div>
              <div class="flex items-center gap-2 mb-1">
                <h2 class="font-display text-xl font-bold" :style="{ color: 'var(--text)' }">
                  {{ student.student_name }}
                </h2>
                <AppBadge :color="attentionColor(student.attention_level) as any" size="xs">
                  {{ formatLabel(student.attention_level) }}
                </AppBadge>
                <AppBadge :color="readinessColor(student.overall_readiness_band) as any" size="xs">
                  {{ formatLabel(student.overall_readiness_band) }}
                </AppBadge>
              </div>
              <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
                {{ student.exam_target ?? 'Exam target not set' }} - {{ formatDate(student.exam_target_date) }}
              </p>
            </div>

            <div class="flex flex-wrap gap-2">
              <AppButton variant="primary" size="sm" @click="router.push('/parent/child/' + student.student_id)">
                Open Dashboard
              </AppButton>
              <AppButton variant="secondary" size="sm" @click="router.push('/parent/curriculum')">
                Curriculum
              </AppButton>
            </div>
          </div>

          <div class="grid grid-cols-1 xl:grid-cols-3 gap-4">
            <div class="space-y-3">
              <div v-if="student.active_risks.length">
                <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Risks</p>
                <div class="space-y-2">
                  <div
                    v-for="risk in student.active_risks.slice(0, 3)"
                    :key="risk.title"
                    class="rounded-xl p-3"
                    :style="{ backgroundColor: 'var(--paper)' }"
                  >
                    <div class="flex items-center justify-between gap-2 mb-1">
                      <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ risk.title }}</p>
                      <AppBadge :color="urgencyColor(risk.severity) as any" size="xs">
                        {{ formatLabel(risk.severity) }}
                      </AppBadge>
                    </div>
                    <p class="text-xs leading-relaxed" :style="{ color: 'var(--ink-muted)' }">{{ risk.description }}</p>
                  </div>
                </div>
              </div>

              <div>
                <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Weekly Memo</p>
                <p class="text-sm leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">{{ student.weekly_memo }}</p>
              </div>
            </div>

            <div>
              <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Subject Coverage</p>
              <div class="space-y-3">
                <div v-for="subject in student.subject_summaries.slice(0, 4)" :key="subject.subject_id">
                  <div class="flex items-center justify-between mb-1">
                    <span class="text-sm" :style="{ color: 'var(--ink-secondary)' }">{{ subject.subject_name }}</span>
                    <AppBadge :color="readinessColor(subject.readiness_band) as any" size="xs">
                      {{ formatLabel(subject.readiness_band) }}
                    </AppBadge>
                  </div>
                  <AppProgress
                    :value="subject.mastered_topic_count"
                    :max="subject.total_topic_count || 1"
                    size="sm"
                    :color="readinessColor(subject.readiness_band) as any"
                    show-label
                  />
                </div>
              </div>
            </div>

            <div class="space-y-3">
              <div>
                <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Interventions</p>
                <div v-if="student.active_interventions.length" class="space-y-2">
                  <div
                    v-for="intervention in student.active_interventions.slice(0, 3)"
                    :key="intervention.intervention_id"
                    class="rounded-xl p-3"
                    :style="{ backgroundColor: 'var(--paper)' }"
                  >
                    <div class="flex items-center justify-between gap-2 mb-2">
                      <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ intervention.title }}</p>
                      <AppBadge :color="urgencyColor(intervention.risk_severity ?? intervention.status) as any" size="xs">
                        {{ formatLabel(intervention.status) }}
                      </AppBadge>
                    </div>
                    <AppProgress :value="intervention.progress_percent" :max="10000" size="sm" color="accent" show-label />
                    <p v-if="intervention.next_step" class="text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">
                      {{ intervention.next_step }}
                    </p>
                  </div>
                </div>
                <AppCard v-else padding="sm">
                  <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No active interventions for this child right now.</p>
                </AppCard>
              </div>

              <div v-if="student.household_actions.length">
                <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Student Actions</p>
                <div class="space-y-2">
                  <div
                    v-for="action in student.household_actions.slice(0, 3)"
                    :key="action.title"
                    class="rounded-xl p-3"
                    :style="{ backgroundColor: 'var(--paper)' }"
                  >
                    <p class="text-sm font-medium mb-1" :style="{ color: 'var(--text)' }">{{ action.title }}</p>
                    <p class="text-xs leading-relaxed" :style="{ color: 'var(--ink-muted)' }">{{ action.detail }}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </AppCard>
      </div>
    </div>

    <div v-else class="text-center py-16">
      <p class="text-sm font-medium mb-2" :style="{ color: 'var(--text)' }">No children linked yet.</p>
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">
        Create a child account first so the household dashboard has someone to track.
      </p>
      <AppButton variant="primary" @click="router.push('/parent/children')">
        Go To Children
      </AppButton>
    </div>
  </div>
</template>
