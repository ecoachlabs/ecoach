<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getParentCurriculumSummary, type CurriculumParentSummaryDto } from '@/ipc/curriculum'
import { listLinkedStudents } from '@/ipc/identity'
import type { AccountSummaryDto } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppSelect from '@/components/ui/AppSelect.vue'

const auth = useAuthStore()
const router = useRouter()

const loadingChildren = ref(true)
const loadingSummary = ref(false)
const error = ref('')
const children = ref<AccountSummaryDto[]>([])
const selectedStudentId = ref('')
const summary = ref<CurriculumParentSummaryDto | null>(null)

const studentOptions = computed(() =>
  children.value.map((child) => ({ value: String(child.id), label: child.display_name })),
)

const examRiskEntries = computed(() =>
  Object.entries(summary.value?.exam_risk_by_subject ?? {}),
)

onMounted(() => {
  void loadChildren()
})

async function loadChildren() {
  if (!auth.currentAccount) {
    loadingChildren.value = false
    return
  }

  loadingChildren.value = true
  error.value = ''

  try {
    children.value = await listLinkedStudents(auth.currentAccount.id)
    if (children.value[0]) {
      selectedStudentId.value = String(children.value[0].id)
      await loadSummary()
    }
  } catch (err) {
    error.value = extractError(err, 'Failed to load linked children')
  }

  loadingChildren.value = false
}

async function loadSummary() {
  if (!auth.currentAccount || !selectedStudentId.value) {
    summary.value = null
    return
  }

  loadingSummary.value = true
  error.value = ''

  try {
    summary.value = await getParentCurriculumSummary(
      auth.currentAccount.id,
      Number(selectedStudentId.value),
      null,
    )
  } catch (err) {
    error.value = extractError(err, 'Failed to load curriculum summary')
  }

  loadingSummary.value = false
}

function handleStudentChange(value: string | number) {
  selectedStudentId.value = String(value)
  void loadSummary()
}

function progressTone(percent: number) {
  if (percent >= 70) return 'success'
  if (percent < 40) return 'danger'
  return 'accent'
}

function riskTone(value: string) {
  const normalized = value.toLowerCase()
  if (normalized.includes('high') || normalized.includes('blocked') || normalized.includes('overdue')) return 'danger'
  if (normalized.includes('medium') || normalized.includes('fragile') || normalized.includes('watch')) return 'warm'
  if (normalized.includes('low') || normalized.includes('ready') || normalized.includes('track')) return 'success'
  return 'accent'
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
      title="Curriculum"
      subtitle="Track how each child is moving through the syllabus, weak spots, and exam readiness."
    >
      <template #actions>
        <div class="min-w-[220px]" v-if="studentOptions.length">
          <AppSelect
            :model-value="selectedStudentId"
            label="Child"
            :options="studentOptions"
            @update:model-value="handleStudentChange"
          />
        </div>
      </template>
    </PageHeader>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loadingChildren || loadingSummary" class="space-y-4">
      <div class="grid grid-cols-1 sm:grid-cols-4 gap-4">
        <div v-for="i in 4" :key="i" class="h-24 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
      <div v-for="i in 2" :key="'curriculum-' + i" class="h-52 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else-if="children.length === 0" class="text-center py-16">
      <p class="text-sm font-medium mb-2" :style="{ color: 'var(--text)' }">No children linked yet.</p>
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">
        Create or link a child account before opening curriculum oversight.
      </p>
      <AppButton variant="primary" @click="router.push('/parent/children')">
        Go To Children
      </AppButton>
    </div>

    <div v-else-if="summary" class="space-y-6">
      <div class="grid grid-cols-1 sm:grid-cols-4 gap-4">
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Curriculum</p>
          <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ summary.curriculum_version.name }}</p>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ summary.curriculum_version.version_label }}</p>
        </AppCard>
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Track Status</p>
          <AppBadge :color="(summary.on_track ? 'success' : 'danger') as any" size="sm">
            {{ summary.on_track ? 'On Track' : 'Needs Repair' }}
          </AppBadge>
        </AppCard>
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Weak Topics</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--warm)' }">{{ summary.weak_topics.length }}</p>
        </AppCard>
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Overdue Topics</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--gold)' }">{{ summary.overdue_topics.length }}</p>
        </AppCard>
      </div>

      <AppCard padding="lg">
        <div class="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
          <div>
            <p class="text-xs uppercase font-semibold mb-2" :style="{ color: 'var(--ink-muted)' }">Parent Summary</p>
            <p class="text-sm leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">{{ summary.summary_text }}</p>
          </div>
          <AppButton variant="secondary" size="sm" @click="router.push('/parent/child/' + summary.learner_id)">
            Open Child Dashboard
          </AppButton>
        </div>
      </AppCard>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Weak Topics</p>
          <div v-if="summary.weak_topics.length" class="space-y-2">
            <p v-for="topic in summary.weak_topics" :key="topic" class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
              {{ topic }}
            </p>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No major weak topic cluster right now.</p>
        </AppCard>

        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Overdue Topics</p>
          <div v-if="summary.overdue_topics.length" class="space-y-2">
            <p v-for="topic in summary.overdue_topics" :key="topic" class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
              {{ topic }}
            </p>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Nothing is overdue in the curriculum map right now.</p>
        </AppCard>

        <AppCard padding="md">
          <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Exam Risk By Subject</p>
          <div v-if="examRiskEntries.length" class="space-y-2">
            <div
              v-for="[subject, risk] in examRiskEntries"
              :key="subject"
              class="flex items-center justify-between gap-3"
            >
              <span class="text-sm" :style="{ color: 'var(--ink-secondary)' }">{{ subject }}</span>
              <AppBadge :color="riskTone(String(risk)) as any" size="xs">
                {{ String(risk).replace(/_/g, ' ') }}
              </AppBadge>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No subject risk labels available yet.</p>
        </AppCard>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <AppCard
          v-for="subject in summary.subject_cards"
          :key="subject.subject_track_id"
          padding="lg"
        >
          <div class="flex items-start justify-between gap-3 mb-4">
            <div>
              <h2 class="font-display text-lg font-bold" :style="{ color: 'var(--text)' }">
                {{ subject.public_title }}
              </h2>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
                {{ subject.trend_label }}
              </p>
            </div>
            <AppBadge :color="progressTone(subject.exam_ready_percent) as any" size="sm">
              {{ subject.exam_ready_percent }}% exam ready
            </AppBadge>
          </div>

          <div class="space-y-3 mb-4">
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-sm" :style="{ color: 'var(--ink-secondary)' }">Entered</span>
                <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ subject.entered_percent }}%</span>
              </div>
              <AppProgress :value="subject.entered_percent" :max="100" size="sm" color="accent" />
            </div>
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-sm" :style="{ color: 'var(--ink-secondary)' }">Stable</span>
                <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ subject.stable_percent }}%</span>
              </div>
              <AppProgress :value="subject.stable_percent" :max="100" size="sm" color="success" />
            </div>
            <div>
              <div class="flex items-center justify-between mb-1">
                <span class="text-sm" :style="{ color: 'var(--ink-secondary)' }">Exam Ready</span>
                <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ subject.exam_ready_percent }}%</span>
              </div>
              <AppProgress :value="subject.exam_ready_percent" :max="100" size="sm" :color="progressTone(subject.exam_ready_percent) as any" />
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3 mb-4">
            <div class="rounded-xl p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-[11px] uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Weak</p>
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.weak_area_count }}</p>
            </div>
            <div class="rounded-xl p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-[11px] uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Blocked</p>
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.blocked_count }}</p>
            </div>
            <div class="rounded-xl p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-[11px] uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Review Due</p>
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.review_due_count }}</p>
            </div>
          </div>

          <div class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <p v-if="subject.strongest_topic_title">
              Strongest topic: {{ subject.strongest_topic_title }}
            </p>
            <p v-if="subject.weakest_topic_title">
              Weakest topic: {{ subject.weakest_topic_title }}
            </p>
            <p v-if="subject.next_action">
              Next move: {{ subject.next_action }}
            </p>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
