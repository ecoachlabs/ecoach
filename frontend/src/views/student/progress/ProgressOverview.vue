<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getLearnerTruth,
  getPriorityTopics,
  getStudentDashboard,
  listActiveMisconceptions,
  listStudentActivityHistory,
  type LearnerMisconceptionSnapshotDto,
  type LearnerTruthDto,
  type StudentActivityHistoryItemDto,
  type StudentDashboardDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import AppProgress from '@/components/ui/AppProgress.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import { getReadinessColor, getReadinessLabel } from '@/utils/readiness'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topics = ref<TopicCaseDto[]>([])
const activity = ref<StudentActivityHistoryItemDto[]>([])
const misconceptionSignals = ref<LearnerMisconceptionSnapshotDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    const studentId = auth.currentAccount.id
    const [learnerTruth, studentDashboard, topicList, history] = await Promise.all([
      getLearnerTruth(studentId),
      getStudentDashboard(studentId).catch(() => null),
      getPriorityTopics(studentId, 20),
      listStudentActivityHistory(studentId, 10).catch(() => []),
    ])

    truth.value = learnerTruth
    dashboard.value = studentDashboard
    topics.value = topicList
    activity.value = history

    if (studentDashboard?.subjects?.length) {
      const groups = await Promise.all(
        studentDashboard.subjects.map(subject =>
          listActiveMisconceptions(studentId, subject.subject_id).catch(() => []),
        ),
      )
      misconceptionSignals.value = groups.flat().sort((left, right) => right.risk_score - left.risk_score)
    }
  } catch (cause) {
    console.error('Failed to load progress overview:', cause)
  } finally {
    loading.value = false
  }
})

const activeSignalCount = computed(() => misconceptionSignals.value.length)

const signalCountByTopic = computed(() => {
  const counts = new Map<number, number>()
  for (const signal of misconceptionSignals.value) {
    if (signal.topic_id == null) continue
    counts.set(signal.topic_id, (counts.get(signal.topic_id) ?? 0) + 1)
  }
  return counts
})

const subjectSignals = computed(() => {
  const grouped = new Map<number, { count: number; highestRisk: number }>()
  for (const signal of misconceptionSignals.value) {
    const current = grouped.get(signal.subject_id)
    if (current) {
      current.count += 1
      current.highestRisk = Math.max(current.highestRisk, signal.risk_score)
      continue
    }
    grouped.set(signal.subject_id, { count: 1, highestRisk: signal.risk_score })
  }
  return grouped
})

const subjectCards = computed(() => {
  return (dashboard.value?.subjects ?? []).map(subject => ({
    ...subject,
    signalCount: subjectSignals.value.get(subject.subject_id)?.count ?? 0,
    highestRisk: subjectSignals.value.get(subject.subject_id)?.highestRisk ?? 0,
  }))
})

const topSignals = computed(() => misconceptionSignals.value.slice(0, 8))

function activityTypeLabel(type: string): string {
  return type.replace(/_/g, ' ')
}

function riskColor(score: number): string {
  if (score >= 7000) return 'var(--warm)'
  if (score >= 4500) return 'var(--gold)'
  return 'var(--ink)'
}

function formatPercent(bp: number): string {
  return `${Math.round(bp / 100)}%`
}

function practiceRouteFor(topicId: number): string {
  return `/student/practice?topic=${topicId}`
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Progress</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Academic snapshot
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Real subject state, recent activity, and active misconception pressure
        </p>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Mastery Map</button>
        <button class="nav-pill" @click="router.push('/student/progress/analytics')">Analytics</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
      </div>
    </div>

    <div
      class="flex-shrink-0 grid grid-cols-5 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <template v-if="loading">
        <div v-for="i in 5" :key="i" class="p-5 flex flex-col items-center gap-2">
          <div class="h-8 w-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          <div class="h-2.5 w-16 rounded animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
      </template>
      <template v-else>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: getReadinessColor(truth?.overall_readiness_band) }">
            {{ getReadinessLabel(truth?.overall_readiness_band) }}
          </p>
          <p class="stat-lbl">Readiness</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ truth?.topic_count ?? 0 }}</p>
          <p class="stat-lbl">Tracked Topics</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ truth?.memory_count ?? 0 }}</p>
          <p class="stat-lbl">Tracked Memories</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ truth?.due_memory_count ?? 0 }}</p>
          <p class="stat-lbl">Due Reviews</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: activeSignalCount ? 'var(--warm)' : 'var(--ink-muted)' }">{{ activeSignalCount }}</p>
          <p class="stat-lbl">Signals</p>
        </div>
      </template>
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div
        v-for="i in 4"
        :key="i"
        class="h-20 rounded-lg animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }"
      />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div v-if="subjectCards.length">
          <p class="section-label mb-3">Subject State</p>
          <div class="grid grid-cols-2 gap-3">
            <div
              v-for="subject in subjectCards"
              :key="subject.subject_id"
              class="subject-card"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ subject.subject_name }}</p>
                  <p class="text-[11px] mt-1" :style="{ color: getReadinessColor(subject.readiness_band) }">
                    {{ getReadinessLabel(subject.readiness_band) }}
                  </p>
                </div>
                <span
                  v-if="subject.signalCount"
                  class="count-pill"
                  :style="{ background: 'rgba(202,138,4,0.12)', color: riskColor(subject.highestRisk) }"
                >
                  {{ subject.signalCount }} signal{{ subject.signalCount === 1 ? '' : 's' }}
                </span>
              </div>

              <div class="grid grid-cols-3 gap-2 mt-4">
                <div class="mini-stat">
                  <p class="mini-value">{{ subject.mastered_topic_count }}</p>
                  <p class="mini-label">Mastered</p>
                </div>
                <div class="mini-stat">
                  <p class="mini-value">{{ subject.weak_topic_count }}</p>
                  <p class="mini-label">Weak</p>
                </div>
                <div class="mini-stat">
                  <p class="mini-value">{{ subject.total_topic_count }}</p>
                  <p class="mini-label">Total</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex-1 overflow-hidden flex flex-col">
          <div
            class="flex items-center justify-between px-1 pb-3"
          >
            <p class="section-label">Priority Topics - {{ topics.length }} topics</p>
            <button
              class="mini-link"
              @click="router.push('/student/diagnostic')"
            >
              Run Diagnostic ->
            </button>
          </div>

          <div v-if="!topics.length" class="surface-card text-center py-8">
            <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Complete your diagnostic to see topic mastery data.</p>
          </div>

          <div v-else class="divide-y divide-[var(--border-soft)] surface-card p-0" :style="{ borderColor: 'transparent' }">
            <button
              v-for="topic in topics"
              :key="topic.topic_id"
              class="topic-row w-full text-left px-5 py-3.5 flex items-center gap-4"
              @click="router.push(practiceRouteFor(topic.topic_id))"
            >
              <MasteryBadge :state="topic.mastery_state" size="sm" />
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 flex-wrap">
                  <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                  <span
                    v-if="signalCountByTopic.get(topic.topic_id)"
                    class="count-pill"
                    :style="{ background: 'rgba(202,138,4,0.12)', color: 'var(--gold)' }"
                  >
                    {{ signalCountByTopic.get(topic.topic_id) }} signal{{ signalCountByTopic.get(topic.topic_id) === 1 ? '' : 's' }}
                  </span>
                </div>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ topic.subject_code }} - {{ topic.intervention_mode.replace(/_/g, ' ') }}
                </p>
                <p class="text-[10px] mt-1" :style="{ color: 'var(--ink-secondary)' }">{{ topic.intervention_reason }}</p>
              </div>
              <div class="w-36 flex-shrink-0">
                <AppProgress
                  :value="topic.mastery_score"
                  :max="10000"
                  size="sm"
                  :color="topic.mastery_score >= 6000 ? 'success' : topic.mastery_score >= 3000 ? 'gold' : 'danger'"
                />
              </div>
              <span
                class="text-sm tabular-nums font-bold w-12 text-right"
                :style="{ color: topic.mastery_score >= 6000 ? 'var(--accent)' : topic.mastery_score >= 3000 ? 'var(--gold)' : 'var(--warm)' }"
              >
                {{ (topic.mastery_score / 100).toFixed(0) }}%
              </span>
            </button>
          </div>
        </div>
      </div>

      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">State Feed</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div class="surface-card">
            <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--ink)' }">Active Misconceptions</p>
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
                    <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ signal.topic_name ?? `Subject ${signal.subject_id}` }}</p>
                    <p class="text-[10px] mt-1 capitalize" :style="{ color: 'var(--ink-secondary)' }">
                      {{ signal.current_status.replace(/_/g, ' ') }} - {{ formatPercent(signal.risk_score) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">No active misconception state yet.</p>
          </div>

          <div class="surface-card">
            <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--ink)' }">Recent Activity</p>
            <div v-if="activity.length" class="space-y-2">
              <div
                v-for="entry in activity"
                :key="entry.id"
                class="activity-row"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0 flex-1">
                    <p class="text-[12px] font-semibold" :style="{ color: 'var(--ink)' }">{{ entry.label }}</p>
                    <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                      {{ entry.subject }} - {{ activityTypeLabel(entry.type_key) }}
                    </p>
                    <p class="text-[10px] mt-1" :style="{ color: 'var(--ink-secondary)' }">
                      {{ entry.correct_questions }}/{{ entry.total_questions }} correct
                    </p>
                  </div>
                  <span class="text-[10px] font-bold flex-shrink-0" :style="{ color: entry.score >= 70 ? 'var(--accent)' : entry.score >= 45 ? 'var(--gold)' : 'var(--warm)' }">
                    {{ entry.score }}%
                  </span>
                </div>
              </div>
            </div>
            <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">No recent activity has been recorded yet.</p>
          </div>
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

.nav-pill {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 100ms;
}

.nav-pill:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.stat-cell {
  padding: 18px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  text-align: center;
}

.stat-big {
  font-size: 22px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
  text-transform: capitalize;
}

.stat-lbl {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-muted);
}

.subject-card,
.surface-card,
.signal-row,
.activity-row,
.mini-stat {
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--surface);
}

.subject-card,
.surface-card {
  padding: 14px;
}

.mini-stat {
  padding: 10px;
  background: var(--paper);
}

.mini-value {
  font-size: 16px;
  font-weight: 800;
  line-height: 1;
  color: var(--ink);
}

.mini-label {
  margin-top: 4px;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
}

.topic-row {
  transition: background-color 100ms ease;
}

.topic-row:hover {
  background-color: var(--paper);
}

.count-pill {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
}

.signal-row,
.activity-row {
  padding: 12px;
  background: var(--paper);
}

.mini-link {
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  padding: 0;
}
</style>
