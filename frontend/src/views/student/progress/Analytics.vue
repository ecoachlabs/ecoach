<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getReadinessReport } from '@/ipc/readiness'
import {
  getLearnerTruth,
  getStudentDashboard,
  listActiveMisconceptions,
  type LearnerMisconceptionSnapshotDto,
  type LearnerTruthDto,
  type StudentDashboardDto,
} from '@/ipc/coach'
import AppProgress from '@/components/ui/AppProgress.vue'
import { getReadinessColor, getReadinessLabel, getReadinessProgressColor } from '@/utils/readiness'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const readiness = ref<any>(null)
const truth = ref<LearnerTruthDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const misconceptions = ref<LearnerMisconceptionSnapshotDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    const studentId = auth.currentAccount.id
    const [readinessReport, learnerTruth, studentDashboard] = await Promise.all([
      getReadinessReport(studentId),
      getLearnerTruth(studentId),
      getStudentDashboard(studentId),
    ])

    readiness.value = readinessReport
    truth.value = learnerTruth
    dashboard.value = studentDashboard

    if (studentDashboard.subjects.length > 0) {
      const misconceptionGroups = await Promise.all(
        studentDashboard.subjects.map(subject =>
          listActiveMisconceptions(studentId, subject.subject_id).catch(() => []),
        ),
      )
      misconceptions.value = misconceptionGroups
        .flat()
        .sort((left, right) => right.risk_score - left.risk_score)
    }
  } catch (error) {
    console.error('Failed to load analytics:', error)
  } finally {
    loading.value = false
  }
})

const coveragePct = computed(() => {
  if (!readiness.value) return 0
  return Math.round(readiness.value.coverage_percent * 100)
})

const misconceptionHighlights = computed(() => misconceptions.value.slice(0, 6))

const subjectSignalPressure = computed(() => {
  const subjects = dashboard.value?.subjects ?? []
  const summary = new Map<number, { name: string; count: number; highestRisk: number }>()

  for (const subject of subjects) {
    summary.set(subject.subject_id, {
      name: subject.subject_name,
      count: 0,
      highestRisk: 0,
    })
  }

  for (const signal of misconceptions.value) {
    const entry = summary.get(signal.subject_id)
    if (!entry) continue
    entry.count += 1
    entry.highestRisk = Math.max(entry.highestRisk, signal.risk_score)
  }

  return Array.from(summary.values())
    .filter(entry => entry.count > 0)
    .sort((left, right) => right.highestRisk - left.highestRisk)
})

function riskColor(riskScore: number): string {
  if (riskScore >= 7000) return 'var(--warm)'
  if (riskScore >= 4500) return 'var(--gold)'
  return 'var(--ink)'
}

function statusLabel(status: string): string {
  return status.replace(/_/g, ' ')
}

function formatPercent(bp: number): string {
  return `${Math.round(bp / 100)}%`
}

const overallBandColor = computed(() => getReadinessColor(readiness.value?.overall_readiness_band))
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Analytics</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Performance deep dive
        </h1>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress')">Overview</button>
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Mastery Map</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
      </div>
    </div>

    <div
      class="flex-shrink-0 grid grid-cols-5 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ coveragePct }}%</p>
        <p class="stat-lbl">Tracked Coverage</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ truth?.diagnosis_count ?? 0 }}</p>
        <p class="stat-lbl">Recent Diagnoses</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ truth?.topic_count ?? 0 }}</p>
        <p class="stat-lbl">Tracked Topics</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: misconceptions.length ? 'var(--warm)' : 'var(--ink-muted)' }">
          {{ misconceptions.length }}
        </p>
        <p class="stat-lbl">Misconceptions</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big capitalize" :style="{ color: overallBandColor }">
          {{ getReadinessLabel(readiness?.overall_readiness_band) }}
        </p>
        <p class="stat-lbl">Band</p>
      </div>
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div
        v-for="i in 5"
        :key="i"
        class="h-16 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }"
      />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div v-if="readiness?.subjects?.length">
          <p class="section-label mb-4">Subject Readiness</p>
          <div class="space-y-3">
            <div
              v-for="subject in readiness.subjects"
              :key="subject.subject_id"
              class="subject-card px-5 py-4 rounded-2xl border"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
            >
              <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-3">
                  <div
                    class="w-9 h-9 rounded-xl flex items-center justify-center text-sm font-black"
                    :style="{ background: 'var(--paper)', color: 'var(--ink)', border: '1px solid transparent' }"
                  >
                    {{ subject.subject_name?.charAt(0) ?? '?' }}
                  </div>
                  <div>
                    <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ subject.subject_name }}</p>
                    <p class="text-[10px] capitalize" :style="{ color: getReadinessColor(subject.readiness_band) }">
                      {{ getReadinessLabel(subject.readiness_band) }}
                    </p>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-lg font-black tabular-nums" :style="{ color: getReadinessColor(subject.readiness_band) }">
                    {{ subject.mastered_topic_count }}
                    <span class="text-sm font-semibold" :style="{ color: 'var(--ink-muted)' }">/{{ subject.total_topic_count }}</span>
                  </p>
                  <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">topics mastered</p>
                </div>
              </div>
              <AppProgress
                :value="subject.mastered_topic_count"
                :max="subject.total_topic_count || 1"
                size="md"
                :color="getReadinessProgressColor(subject.readiness_band)"
              />
            </div>
          </div>
        </div>

        <div v-if="subjectSignalPressure.length" class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-1">Derived Signal Pressure</p>
          <p class="text-[11px] mb-3" :style="{ color: 'var(--ink-muted)' }">
            Built from the current active misconception rows in each subject
          </p>
          <div class="space-y-2">
            <div
              v-for="subject in subjectSignalPressure"
              :key="subject.name"
              class="signal-pressure-row"
            >
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ subject.name }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ subject.count }} active misconception signal{{ subject.count === 1 ? '' : 's' }}
                </p>
              </div>
              <span class="text-sm font-bold tabular-nums" :style="{ color: riskColor(subject.highestRisk) }">
                {{ formatPercent(subject.highestRisk) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="!readiness?.subjects?.length" class="flex flex-col items-center justify-center h-full gap-4 text-center">
          <p class="text-sm font-semibold" :style="{ color: 'var(--ink-muted)' }">Complete your diagnostic to unlock analytics</p>
          <button
            class="px-5 py-2.5 rounded-xl font-semibold text-sm"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/diagnostic')"
          >
            Start Diagnostic
          </button>
        </div>
      </div>

      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Signal Feed</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div class="space-y-2">
            <div class="metric-row">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Overall Band</p>
              <p class="text-sm font-bold capitalize" :style="{ color: overallBandColor }">
                {{ getReadinessLabel(readiness?.overall_readiness_band) }}
              </p>
            </div>
            <div class="metric-row">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Tracked Coverage</p>
              <p class="text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ coveragePct }}%</p>
            </div>
            <div class="metric-row">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Memories Stored</p>
              <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ truth?.memory_count ?? 0 }}</p>
            </div>
            <div class="metric-row">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Due Reviews</p>
              <p class="text-sm font-bold" :style="{ color: 'var(--gold)' }">{{ truth?.due_memory_count ?? 0 }}</p>
            </div>
            <div class="metric-row">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Pending Review</p>
              <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ truth?.pending_review_count ?? 0 }}</p>
            </div>
          </div>

          <div>
            <p class="section-label mb-2">Active Misconceptions</p>
            <div v-if="misconceptionHighlights.length" class="space-y-2">
              <div
                v-for="signal in misconceptionHighlights"
                :key="`${signal.subject_id}-${signal.misconception_id}`"
                class="signal-card"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <p class="text-[12px] font-semibold" :style="{ color: 'var(--ink)' }">{{ signal.title }}</p>
                    <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                      {{ signal.topic_name ?? `Subject ${signal.subject_id}` }}
                    </p>
                    <p class="text-[10px] capitalize" :style="{ color: 'var(--ink-secondary)' }">
                      {{ statusLabel(signal.current_status) }} · detected {{ signal.times_detected }}x
                    </p>
                  </div>
                  <span class="text-[10px] font-bold" :style="{ color: riskColor(signal.risk_score) }">
                    {{ formatPercent(signal.risk_score) }}
                  </span>
                </div>
              </div>
            </div>
            <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
              No active misconception state is currently flagged.
            </p>
          </div>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="w-full py-2 rounded-xl text-xs font-semibold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/diagnostic')"
          >
            Run Diagnostic ->
          </button>
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
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.stat-big {
  font-size: 22px;
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

.subject-card {
  transition: border-color 120ms ease;
}

.subject-card:hover {
  border-color: var(--ink-muted) !important;
}

.metric-row,
.signal-card,
.signal-pressure-row {
  border: 1px solid transparent;
  border-radius: 10px;
  background: var(--paper);
}

.metric-row,
.signal-pressure-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
}

.signal-card {
  padding: 10px;
}
</style>
