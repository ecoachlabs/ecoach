<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getRevengeQueue,
  getStudentDashboard,
  listActiveMisconceptions,
  listTopics,
  type LearnerMisconceptionSnapshotDto,
  type RevengeQueueItemDto,
  type StudentDashboardDto,
} from '@/ipc/coach'
import { startPracticeSessionWithQuestions } from '@/utils/sessionQuestions'
import AppTabs from '@/components/ui/AppTabs.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const loading = ref(true)
const mistakes = ref<RevengeQueueItemDto[]>([])
const dashboard = ref<StudentDashboardDto | null>(null)
const misconceptionSignals = ref<LearnerMisconceptionSnapshotDto[]>([])
const topicSubjectLookup = ref<Record<number, number>>({})
const topicNameMap = ref<Record<number, string>>({})
const activeTab = ref('pending')

const tabs = [
  { key: 'pending', label: 'Pending' },
  { key: 'beaten', label: 'Beaten' },
  { key: 'patterns', label: 'Patterns' },
]

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    const studentId = auth.currentAccount.id
    const [queue, studentDashboard] = await Promise.all([
      getRevengeQueue(studentId),
      getStudentDashboard(studentId).catch(() => null),
    ])

    mistakes.value = queue
    dashboard.value = studentDashboard

    if (studentDashboard?.subjects?.length) {
      const groups = await Promise.all(
        studentDashboard.subjects.map(subject =>
          Promise.all([
            listActiveMisconceptions(studentId, subject.subject_id).catch(() => []),
            listTopics(subject.subject_id).catch(() => []),
          ]).then(([signals, topics]) => ({ subjectId: subject.subject_id, signals, topics })),
        ),
      )
      misconceptionSignals.value = groups
        .flatMap(group => group.signals)
        .sort((left, right) => right.risk_score - left.risk_score)
      topicSubjectLookup.value = Object.fromEntries(
        groups.flatMap(group => group.topics.map(topic => [topic.id, group.subjectId])),
      )
      topicNameMap.value = Object.fromEntries(
        groups.flatMap(group => group.topics.map(topic => [topic.id, topic.name])),
      )
    }
  } catch (error) {
    console.error('Failed to load mistake queue:', error)
  } finally {
    loading.value = false
  }
})

const pending = computed(() => mistakes.value.filter(mistake => !mistake.is_beaten))
const beaten = computed(() => mistakes.value.filter(mistake => mistake.is_beaten))
const activeSignalCount = computed(() => misconceptionSignals.value.length)

const errorPatterns = computed(() => {
  const counts: Record<string, number> = {}
  for (const mistake of mistakes.value) {
    const type = mistake.original_error_type ?? 'unknown'
    counts[type] = (counts[type] ?? 0) + 1
  }

  return Object.entries(counts)
    .map(([type, count]) => ({ type, count }))
    .sort((left, right) => right.count - left.count)
})

const topicNameLookup = computed(() => {
  const lookup: Record<number, string> = { ...topicNameMap.value }
  for (const signal of misconceptionSignals.value) {
    if (signal.topic_id != null && signal.topic_name) {
      lookup[signal.topic_id] = signal.topic_name
    }
  }
  return lookup
})

const topicPatterns = computed(() => {
  const map: Record<number, { topic_id: number; topic_name: string; count: number; pending: number }> = {}
  for (const mistake of mistakes.value) {
    if (!mistake.topic_id) continue
    if (!map[mistake.topic_id]) {
      map[mistake.topic_id] = {
        topic_id: mistake.topic_id,
        topic_name: topicNameLookup.value[mistake.topic_id] ?? `Topic #${mistake.topic_id}`,
        count: 0,
        pending: 0,
      }
    }
    map[mistake.topic_id].count += 1
    if (!mistake.is_beaten) map[mistake.topic_id].pending += 1
  }
  return Object.values(map).sort((left, right) => right.pending - left.pending)
})

const subjectSignalPressure = computed(() => {
  const subjectNames = new Map<number, string>(
    (dashboard.value?.subjects ?? []).map(subject => [subject.subject_id, subject.subject_name]),
  )
  const summary = new Map<number, { subject_id: number; subject_name: string; count: number; highest_risk: number }>()

  for (const signal of misconceptionSignals.value) {
    const existing = summary.get(signal.subject_id)
    const subjectName = subjectNames.get(signal.subject_id) ?? `Subject ${signal.subject_id}`
    if (existing) {
      existing.count += 1
      existing.highest_risk = Math.max(existing.highest_risk, signal.risk_score)
      continue
    }
    summary.set(signal.subject_id, {
      subject_id: signal.subject_id,
      subject_name: subjectName,
      count: 1,
      highest_risk: signal.risk_score,
    })
  }

  return Array.from(summary.values()).sort((left, right) => right.highest_risk - left.highest_risk)
})

async function launchTopicDrill(topicId: number | null | undefined, questionCount: number = 10) {
  if (!auth.currentAccount || !topicId) {
    router.push('/student/practice')
    return
  }

  const subjectId = topicSubjectLookup.value[topicId]
  if (!subjectId) {
    router.push('/student/practice')
    return
  }

  try {
    const snapshot = await startPracticeSessionWithQuestions({
      student_id: auth.currentAccount.id,
      subject_id: subjectId,
      topic_ids: [topicId],
      question_count: questionCount,
      is_timed: false,
    })
    router.push(`/student/session/${snapshot.sessionId}`)
  } catch (error) {
    console.error('Failed to start targeted repair drill:', error)
    router.push('/student/practice')
  }
}

async function launchSignalRepair(signal: LearnerMisconceptionSnapshotDto) {
  if (signal.topic_id != null) {
    await launchTopicDrill(signal.topic_id, 12)
    return
  }
  router.push('/student/knowledge-gap')
}

async function launchErrorPatternRepair(type: string) {
  const topicId = pending.value.find(
    mistake => !mistake.is_beaten && mistake.original_error_type === type && mistake.topic_id != null,
  )?.topic_id
  await launchTopicDrill(topicId, 12)
}

function errorTypeLabel(type: string): string {
  return type.replace(/_/g, ' ').replace(/\b\w/g, char => char.toUpperCase())
}

function statusLabel(status: string): string {
  return errorTypeLabel(status)
}

function riskColor(riskScore: number): string {
  if (riskScore >= 7000) return 'var(--warm)'
  if (riskScore >= 4500) return 'var(--gold)'
  return 'var(--ink)'
}

function formatPercent(bp: number): string {
  return `${Math.round(bp / 100)}%`
}

watch([() => route.hash, loading], () => {
  if (!loading.value && route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    })
  }
}, { immediate: true })
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Mistake Lab</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Every mistake is data
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Revenge queue plus persistent misconception state
        </p>
      </div>
      <div class="flex items-center gap-5">
        <div class="text-right">
          <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--warm)' }">{{ pending.length }}</p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Pending</p>
        </div>
        <div class="text-right">
          <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ beaten.length }}</p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Beaten</p>
        </div>
        <div class="text-right">
          <p class="text-2xl font-black tabular-nums" :style="{ color: activeSignalCount ? 'var(--gold)' : 'var(--ink-muted)' }">
            {{ activeSignalCount }}
          </p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Signals</p>
        </div>
      </div>
    </div>

    <div
      id="retry-zone"
      class="flex-shrink-0 px-7 pt-4 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <AppTabs v-model="activeTab" :tabs="tabs" />
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div
        v-for="i in 5"
        :key="i"
        class="h-16 rounded-lg animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }"
      />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div v-if="activeTab === 'pending'" class="flex-1 overflow-y-auto">
        <div v-if="!pending.length" class="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
          <div
            class="w-14 h-14 rounded-lg flex items-center justify-center text-sm font-bold"
            :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }"
          >
            CLEAR
          </div>
          <h3 class="font-display text-lg font-bold" :style="{ color: 'var(--ink)' }">No pending mistakes</h3>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Practice more to repopulate the queue.</p>
          <button
            class="action-btn"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/practice')"
          >
            Start Practice
          </button>
        </div>

        <div v-else class="divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
          <div v-for="mistake in pending" :key="mistake.id" class="mistake-row px-7 py-4 flex items-start gap-4">
            <div
              class="w-8 h-8 rounded-lg flex items-center justify-center text-[10px] font-bold flex-shrink-0 mt-0.5"
              :style="{ background: 'rgba(194,65,12,0.1)', color: 'var(--warm)' }"
            >
              ERR
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold line-clamp-2 mb-1" :style="{ color: 'var(--ink)' }">
                {{ mistake.question_text ?? `Question #${mistake.question_id}` }}
              </p>
              <div class="flex items-center gap-2 flex-wrap">
                <span
                  v-if="mistake.original_error_type"
                  class="chip"
                  :style="{ background: 'var(--paper)', color: 'var(--ink-secondary)', border: '1px solid transparent' }"
                >
                  {{ errorTypeLabel(mistake.original_error_type) }}
                </span>
                <span
                  v-if="mistake.original_wrong_answer"
                  class="text-[11px] truncate max-w-xs"
                  :style="{ color: 'var(--warm)' }"
                >
                  You chose: {{ mistake.original_wrong_answer }}
                </span>
              </div>
            </div>
            <div class="flex-shrink-0">
              <span
                v-if="mistake.attempts_to_beat > 0"
                class="chip"
                :style="{ background: 'var(--paper)', color: 'var(--ink-muted)', border: '1px solid transparent' }"
              >
                {{ mistake.attempts_to_beat }}x tried
              </span>
            </div>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'beaten'" class="flex-1 overflow-y-auto">
        <div v-if="!beaten.length" class="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
          <div
            class="w-14 h-14 rounded-lg flex items-center justify-center text-sm font-bold"
            :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }"
          >
            READY
          </div>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No beaten mistakes yet. Keep practicing.</p>
        </div>
        <div v-else class="divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
          <div v-for="mistake in beaten" :key="mistake.id" class="mistake-row px-7 py-4 flex items-center gap-4">
            <div
              class="w-8 h-8 rounded-lg flex items-center justify-center text-[10px] font-bold flex-shrink-0"
              :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }"
            >
              OK
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold line-clamp-1" :style="{ color: 'var(--ink)' }">
                {{ mistake.question_text ?? `Question #${mistake.question_id}` }}
              </p>
              <p v-if="mistake.original_wrong_answer" class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                Was choosing: {{ mistake.original_wrong_answer }}
              </p>
            </div>
            <span
              class="chip flex-shrink-0"
              :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }"
            >
              beaten
            </span>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'patterns'" class="flex-1 overflow-y-auto p-6 space-y-6">
        <div
          v-if="!errorPatterns.length && !topicPatterns.length && !misconceptionSignals.length"
          class="flex flex-col items-center justify-center h-full gap-4 text-center"
        >
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
            No stable error patterns yet. Practice more and they will appear here.
          </p>
        </div>

        <template v-else>
          <div v-if="misconceptionSignals.length">
            <p class="section-label mb-3">Persistent Misconception State</p>
            <div class="space-y-2">
              <div
                v-for="signal in misconceptionSignals"
                :key="`${signal.subject_id}-${signal.misconception_id}`"
                class="signal-row flex items-start gap-4 px-5 py-4 rounded-lg border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              >
                <div class="w-2 h-2 rounded-full flex-shrink-0 mt-2" :style="{ backgroundColor: riskColor(signal.risk_score) }" />
                <div class="flex-1 min-w-0">
                  <div class="flex items-start justify-between gap-3">
                    <div>
                      <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ signal.title }}</p>
                      <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                        {{ signal.topic_name ?? `Subject ${signal.subject_id}` }}
                      </p>
                      <p class="text-[10px] mt-1" :style="{ color: 'var(--ink-secondary)' }">
                        {{ statusLabel(signal.current_status) }} - detected {{ signal.times_detected }}x
                      </p>
                    </div>
                    <span class="text-sm font-black tabular-nums flex-shrink-0" :style="{ color: riskColor(signal.risk_score) }">
                      {{ formatPercent(signal.risk_score) }}
                    </span>
                  </div>
                </div>
                <button
                  class="text-xs font-semibold px-3 py-1.5 rounded-lg flex-shrink-0"
                  :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
                  @click="launchSignalRepair(signal)"
                >
                  Repair ->
                </button>
              </div>
            </div>
          </div>

          <div v-if="subjectSignalPressure.length">
            <p class="section-label mb-3">By Subject Pressure</p>
            <div class="space-y-2">
              <div
                v-for="subject in subjectSignalPressure"
                :key="subject.subject_id"
                class="flex items-center gap-4 px-5 py-3.5 rounded-lg border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              >
                <div
                  class="w-9 h-9 rounded-lg flex items-center justify-center text-sm font-black flex-shrink-0"
                  :style="{ background: 'var(--paper)', color: 'var(--ink)', border: '1px solid transparent' }"
                >
                  {{ subject.count }}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ subject.subject_name }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    {{ subject.count }} active signal{{ subject.count === 1 ? '' : 's' }}
                  </p>
                </div>
                <span class="text-sm font-black tabular-nums" :style="{ color: riskColor(subject.highest_risk) }">
                  {{ formatPercent(subject.highest_risk) }}
                </span>
              </div>
            </div>
          </div>

          <div v-if="errorPatterns.length">
            <p class="section-label mb-3">By Error Type</p>
            <div class="space-y-2">
              <div
                v-for="pattern in errorPatterns"
                :key="pattern.type"
                class="flex items-center gap-4 px-5 py-3.5 rounded-lg border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              >
                <div class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: 'var(--ink-muted)' }" />
                <p class="text-sm font-semibold flex-1" :style="{ color: 'var(--ink)' }">{{ errorTypeLabel(pattern.type) }}</p>
                <span class="text-sm font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ pattern.count }}</span>
                <button
                  class="text-xs font-semibold px-3 py-1.5 rounded-lg"
                  :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
                  @click="launchErrorPatternRepair(pattern.type)"
                >
                  Fix ->
                </button>
              </div>
            </div>
          </div>

          <div v-if="topicPatterns.length">
            <p class="section-label mb-3">By Topic Cluster</p>
            <div class="space-y-2">
              <div
                v-for="topic in topicPatterns"
                :key="topic.topic_id"
                class="flex items-center gap-4 px-5 py-3.5 rounded-lg border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              >
                <div
                  class="w-9 h-9 rounded-lg flex items-center justify-center text-sm font-black flex-shrink-0"
                  :style="{ background: 'var(--paper)', color: 'var(--ink)', border: '1px solid transparent' }"
                >
                  {{ topic.pending }}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    {{ topic.count }} mistake{{ topic.count > 1 ? 's' : '' }} - {{ topic.pending }} pending
                  </p>
                </div>
                <button
                  class="text-xs font-semibold px-3 py-1.5 rounded-lg"
                  :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
                  @click="launchTopicDrill(topic.topic_id, 12)"
                >
                  Drill ->
                </button>
              </div>
            </div>
          </div>
        </template>
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
  color: var(--warm);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.action-btn {
  padding: 10px 18px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  border: none;
}

.chip {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.mistake-row,
.signal-row {
  transition: background-color 100ms;
}

.mistake-row:hover,
.signal-row:hover {
  background-color: var(--paper) !important;
}
</style>
