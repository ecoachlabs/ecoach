<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  listActiveMisconceptions,
  listSubjects,
  type LearnerMisconceptionSnapshotDto,
  type SubjectDto,
} from '@/ipc/coach'
import {
  getMemoryDashboard,
  getReviewQueue,
  listMemoryTopicSummaries,
  type MemoryDashboardDto,
  type RecheckItemDto,
  type TopicMemorySummaryDto,
} from '@/ipc/memory'
import { startPracticeSession } from '@/ipc/sessions'
import MemoryHeatmap from '@/components/modes/memory/MemoryHeatmap.vue'
import RecoveryLadder from '@/components/modes/memory/RecoveryLadder.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const loading = ref(true)
const subjectLoading = ref(false)
const launching = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const memoryDash = ref<MemoryDashboardDto | null>(null)
const reviewQueue = ref<RecheckItemDto[]>([])
const topicSummaries = ref<TopicMemorySummaryDto[]>([])
const misconceptionSignals = ref<LearnerMisconceptionSnapshotDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    subjects.value = await listSubjects()
    const firstSubjectId = subjects.value[0]?.id ?? null
    selectedSubjectId.value = firstSubjectId

    if (firstSubjectId != null) {
      await loadSubjectState(firstSubjectId)
    }
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to load memory data'
  } finally {
    loading.value = false
  }
})

async function loadSubjectState(subjectId: number) {
  if (!auth.currentAccount) return

  subjectLoading.value = true
  try {
    const studentId = auth.currentAccount.id
    const [dashboard, queue, summaries, signals] = await Promise.all([
      getMemoryDashboard(studentId, subjectId).catch(() => null),
      getReviewQueue(studentId, 30, subjectId).catch(() => []),
      listMemoryTopicSummaries(studentId, 20, subjectId).catch(() => []),
      listActiveMisconceptions(studentId, subjectId).catch(() => []),
    ])

    memoryDash.value = dashboard
    reviewQueue.value = queue
    topicSummaries.value = summaries
    misconceptionSignals.value = signals.sort((left, right) => right.risk_score - left.risk_score)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to refresh this subject'
  } finally {
    subjectLoading.value = false
  }
}

async function selectSubject(subjectId: number) {
  if (selectedSubjectId.value === subjectId) return
  selectedSubjectId.value = subjectId
  error.value = ''
  await loadSubjectState(subjectId)
}

const selectedSubjectName = computed(() => {
  return subjects.value.find(subject => subject.id === selectedSubjectId.value)?.name ?? 'Memory Mode'
})

const memoryHealth = computed(() => memoryDash.value?.average_strength ?? 0)
const fadingCount = computed(() => (memoryDash.value?.fading_count ?? 0) + (memoryDash.value?.at_risk_count ?? 0))
const activeSignalCount = computed(() => misconceptionSignals.value.length)

const heatmapTopics = computed(() =>
  topicSummaries.value.map(topic => ({
    name: topic.topic_name,
    strength: topic.average_strength,
    status: (() => {
      if (topic.collapsed_items > 0) return 'critical' as const
      if (topic.average_strength >= 7000 && topic.fragile_items === 0) return 'strong' as const
      if (topic.average_strength >= 7000) return 'recovered' as const
      if (topic.average_strength >= 5000) return 'vulnerable' as const
      if (topic.average_strength >= 2000) return 'fading' as const
      return 'critical' as const
    })(),
  })),
)

const reviewTopicIds = computed(() => {
  const queuedIds = reviewQueue.value
    .filter(item => item.topic_id != null)
    .map(item => item.topic_id as number)

  if (queuedIds.length > 0) {
    return [...new Set(queuedIds)].slice(0, 5)
  }

  const fallbackIds = topicSummaries.value
    .filter(topic => topic.overdue_reviews > 0 || topic.average_strength < 5000 || topic.collapsed_items > 0 || topic.fragile_items > 0)
    .map(topic => topic.topic_id)

  const source = fallbackIds.length > 0 ? fallbackIds : topicSummaries.value.map(topic => topic.topic_id)
  return [...new Set(source)].slice(0, 5)
})

const recoveryStage = computed(() => {
  const strength = memoryHealth.value
  if (strength >= 8000) return 5
  if (strength >= 6500) return 4
  if (strength >= 4500) return 3
  if (strength >= 2500) return 2
  if (strength > 0) return 1
  return 0
})

const topSignals = computed(() => misconceptionSignals.value.slice(0, 8))

async function launchQuickScan() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value || subjectLoading.value) return

  launching.value = true
  error.value = ''
  try {
    if (reviewTopicIds.value.length === 0) {
      throw new Error('No review topics are ready for a quick scan yet.')
    }

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: reviewTopicIds.value,
      question_count: 10,
      is_timed: false,
    })

    router.push(`/student/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to start scan'
    launching.value = false
  }
}

async function launchRescue() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value || subjectLoading.value) return

  launching.value = true
  error.value = ''
  try {
    const fadingTopics = topicSummaries.value
      .filter(topic => topic.average_strength < 5000 || topic.collapsed_items > 0)
      .slice(0, 5)
      .map(topic => topic.topic_id)

    if (!fadingTopics.length) {
      throw new Error('No fading topics are available for rescue in this subject yet.')
    }

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: fadingTopics,
      question_count: 12,
      is_timed: false,
    })

    router.push(`/student/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to start rescue'
    launching.value = false
  }
}

watch([() => route.hash, loading, subjectLoading], () => {
  if (!loading.value && !subjectLoading.value && route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    })
  }
}, { immediate: true })

function strengthColor(value: number) {
  if (value >= 7000) return 'var(--accent)'
  if (value >= 4000) return 'var(--gold)'
  return 'var(--warm)'
}

function riskColor(riskScore: number) {
  if (riskScore >= 7000) return 'var(--warm)'
  if (riskScore >= 4500) return 'var(--gold)'
  return 'var(--ink)'
}

function formatPercent(bp: number) {
  return `${Math.round(bp / 100)}%`
}

function statusLabel(status: string) {
  return status.replace(/_/g, ' ')
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Memory Mode</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Strengthen what you know
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          {{ selectedSubjectName }} - real memory decay plus misconception pressure
        </p>
      </div>
      <div v-if="!loading" class="text-right">
        <p class="text-3xl font-black tabular-nums" :style="{ color: memoryHealth >= 7000 ? 'var(--accent)' : memoryHealth >= 4000 ? 'var(--gold)' : 'var(--warm)' }">
          {{ (memoryHealth / 100).toFixed(0) }}%
        </p>
        <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Memory Strength</p>
      </div>
    </div>

    <div
      class="flex-shrink-0 grid grid-cols-5 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: fadingCount > 0 ? 'var(--gold)' : 'var(--ink-muted)' }">{{ fadingCount }}</p>
        <p class="stat-lbl">Fading</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: (memoryDash?.overdue_reviews ?? 0) > 0 ? 'var(--warm)' : 'var(--ink-muted)' }">
          {{ memoryDash?.overdue_reviews ?? 0 }}
        </p>
        <p class="stat-lbl">Due Reviews</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ memoryDash?.healthy_count ?? 0 }}</p>
        <p class="stat-lbl">Healthy</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ topicSummaries.length }}</p>
        <p class="stat-lbl">Topics</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: activeSignalCount ? 'var(--gold)' : 'var(--ink-muted)' }">{{ activeSignalCount }}</p>
        <p class="stat-lbl">Signals</p>
      </div>
    </div>

    <div
      v-if="error"
      class="px-7 py-2 text-xs flex-shrink-0"
      :style="{ background: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }"
    >
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div
        v-for="i in 5"
        :key="i"
        class="h-14 rounded-lg animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }"
      />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div class="flex items-center gap-2 flex-wrap">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="subject-tab"
            :class="{ active: selectedSubjectId === subject.id }"
            @click="selectSubject(subject.id)"
          >
            {{ subject.name }}
          </button>
        </div>

        <div v-if="subjectLoading" class="space-y-3">
          <div
            v-for="i in 3"
            :key="i"
            class="h-24 rounded-lg animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }"
          />
        </div>

        <template v-else>
          <div v-if="heatmapTopics.length" id="memory-map">
            <p class="section-label mb-3">Memory Health Map</p>
            <div class="surface-card">
              <MemoryHeatmap :topics="heatmapTopics" />
            </div>
          </div>

          <div v-else class="surface-card text-center py-8">
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">No memory heatmap data yet</p>
            <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">
              Practice this subject to start recording memory state.
            </p>
          </div>

          <div id="recovery">
            <div class="mb-3">
              <p class="section-label">Derived Recovery Ladder</p>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                Based on current average memory strength in this subject
              </p>
            </div>
            <div class="surface-card">
              <RecoveryLadder :current-stage="recoveryStage" />
            </div>
          </div>

          <div class="flex gap-3">
            <button class="action-btn primary" :disabled="launching || subjectLoading" @click="launchQuickScan">
              Quick Scan ->
            </button>
            <button
              v-if="fadingCount > 0"
              class="action-btn secondary"
              :disabled="launching || subjectLoading"
              @click="launchRescue"
            >
              Rescue {{ fadingCount }} Fading
            </button>
          </div>
        </template>
      </div>

      <div
        id="reviews"
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'transparent' }">
          <p class="section-label">Pressure Feed</p>
          <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">{{ selectedSubjectName }}</p>
        </div>

        <div v-if="subjectLoading" class="flex-1 p-4 space-y-3">
          <div
            v-for="i in 4"
            :key="i"
            class="h-16 rounded-lg animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }"
          />
        </div>

        <div v-else class="flex-1 overflow-y-auto p-4 space-y-4">
          <div class="panel-block">
            <div class="flex items-center justify-between gap-3 mb-3">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Due Reviews</p>
              <span
                v-if="reviewQueue.length"
                class="count-pill"
                :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
              >
                {{ reviewQueue.length }}
              </span>
            </div>

            <div v-if="reviewQueue.length" class="space-y-1.5">
              <div
                v-for="item in reviewQueue"
                :key="item.id"
                class="review-item px-3 py-3 rounded-lg border"
                :style="{ borderColor: 'transparent' }"
              >
                <div class="flex items-center gap-3">
                  <div class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: strengthColor(item.memory_strength ?? 0) }" />
                  <div class="flex-1 min-w-0">
                    <p class="text-[11px] font-semibold truncate" :style="{ color: 'var(--ink)' }">
                      {{ item.topic_name ?? item.node_title ?? 'Review item' }}
                    </p>
                    <p class="text-[9px]" :style="{ color: 'var(--ink-muted)' }">{{ item.schedule_type }}</p>
                  </div>
                  <span class="text-[9px] font-bold flex-shrink-0" :style="{ color: strengthColor(item.memory_strength ?? 0) }">
                    {{ Math.round((item.memory_strength ?? 0) / 100) }}%
                  </span>
                </div>
              </div>
            </div>

            <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
              No due reviews are currently queued for this subject.
            </p>
          </div>

          <div class="panel-block">
            <div class="flex items-center justify-between gap-3 mb-3">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Misconception Pressure</p>
              <span
                v-if="topSignals.length"
                class="count-pill"
                :style="{ background: 'rgba(202,138,4,0.12)', color: 'var(--gold)' }"
              >
                {{ topSignals.length }}
              </span>
            </div>

            <div v-if="topSignals.length" class="space-y-2">
              <div
                v-for="signal in topSignals"
                :key="`${signal.subject_id}-${signal.misconception_id}`"
                class="signal-row"
              >
                <div class="flex items-start gap-3">
                  <div class="w-2 h-2 rounded-full flex-shrink-0 mt-1.5" :style="{ backgroundColor: riskColor(signal.risk_score) }" />
                  <div class="flex-1 min-w-0">
                    <p class="text-[12px] font-semibold" :style="{ color: 'var(--ink)' }">{{ signal.title }}</p>
                    <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                      {{ signal.topic_name ?? `Subject ${signal.subject_id}` }}
                    </p>
                    <p class="text-[10px] mt-1 capitalize" :style="{ color: 'var(--ink-secondary)' }">
                      {{ statusLabel(signal.current_status) }} - detected {{ signal.times_detected }}x
                    </p>
                  </div>
                  <span class="text-[10px] font-bold flex-shrink-0" :style="{ color: riskColor(signal.risk_score) }">
                    {{ formatPercent(signal.risk_score) }}
                  </span>
                </div>
              </div>
            </div>

            <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
              No active misconception pressure is currently linked to this subject.
            </p>
          </div>

          <button class="repair-link" @click="router.push('/student/knowledge-gap')">Open Repair Mode -></button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
  margin-bottom: 4px;
}

.stat-cell {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.stat-big {
  font-size: 26px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.stat-lbl {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-muted);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.subject-tab {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  color: var(--ink-secondary);
  background: transparent;
  transition: all 120ms;
}

.subject-tab.active,
.subject-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.surface-card,
.panel-block,
.signal-row {
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--surface);
}

.surface-card,
.panel-block {
  padding: 14px;
}

.review-item {
  transition: background-color 100ms;
}

.review-item:hover {
  background-color: var(--paper);
}

.signal-row {
  padding: 12px;
  background: var(--paper);
}

.action-btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 140ms, transform 140ms;
  border: none;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.primary:hover:not(:disabled) {
  opacity: 0.87;
  transform: translateY(-1px);
}

.action-btn.secondary {
  background: var(--accent-glow);
  color: var(--accent);
  border: 1px solid var(--accent);
}

.action-btn.secondary:hover:not(:disabled) {
  background: var(--accent);
  color: white;
}

.count-pill {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
}

.repair-link {
  width: 100%;
  border: none;
  border-radius: 8px;
  padding: 10px 12px;
  background: var(--accent);
  color: white;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}
</style>
