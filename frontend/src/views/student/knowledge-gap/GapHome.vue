<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  captureGapSnapshot,
  getGapDashboard,
  listPriorityGaps,
  listGapFeed,
  type GapDashboardDto,
  type GapFeedItemDto,
  type GapScoreCardDto,
  type GapSnapshotResultDto,
} from '@/ipc/gap'
import {
  getLearnerTruth,
  getStudentDashboard,
  type LearnerTruthDto,
  type StudentDashboardDto,
} from '@/ipc/coach'
import AppBadge from '@/components/ui/AppBadge.vue'
import GapSectionCard from '@/components/modes/knowledge-gap/GapSectionCard.vue'
import GapMapNode from '@/components/modes/knowledge-gap/GapMapNode.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const subjectLoading = ref(false)
const truth = ref<LearnerTruthDto | null>(null)
const gapDash = ref<GapDashboardDto | null>(null)
const priorityGaps = ref<GapScoreCardDto[]>([])
const dashboard = ref<StudentDashboardDto | null>(null)
const selectedSubjectId = ref<number | null>(null)
const subjectSnapshot = ref<GapSnapshotResultDto | null>(null)
const gapFeed = ref<GapFeedItemDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    const studentId = auth.currentAccount.id
    const [learnerTruth, gapDashboard, gapCards, studentDashboard] = await Promise.all([
      getLearnerTruth(studentId),
      getGapDashboard(studentId),
      listPriorityGaps(studentId, 40),
      getStudentDashboard(studentId).catch(() => null),
    ])

    truth.value = learnerTruth
    gapDash.value = gapDashboard
    priorityGaps.value = gapCards
    dashboard.value = studentDashboard

    const firstSubjectId = studentDashboard?.subjects[0]?.subject_id ?? null
    selectedSubjectId.value = firstSubjectId

    if (firstSubjectId != null) {
      await loadSubjectSignals(firstSubjectId)
    }
  } catch (error) {
    console.error('Failed to load gap dashboard:', error)
  } finally {
    loading.value = false
  }
})

async function loadSubjectSignals(subjectId: number) {
  if (!auth.currentAccount) return

  subjectLoading.value = true
  try {
    const studentId = auth.currentAccount.id
    const [snapshot, feed] = await Promise.all([
      captureGapSnapshot(studentId, subjectId).catch(() => null),
      listGapFeed(studentId, subjectId, 8).catch(() => []),
    ])

    subjectSnapshot.value = snapshot
    gapFeed.value = feed
  } finally {
    subjectLoading.value = false
  }
}

async function selectSubject(subjectId: number) {
  if (selectedSubjectId.value === subjectId) return
  selectedSubjectId.value = subjectId
  await loadSubjectSignals(subjectId)
}

const subjects = computed(() => dashboard.value?.subjects ?? [])

const selectedSubjectName = computed(() => {
  return subjects.value.find(subject => subject.subject_id === selectedSubjectId.value)?.subject_name ?? 'Current subject'
})

const gapPercentage = computed(() => {
  if (!truth.value) return 0
  return Math.max(0, 100 - Math.round(truth.value.overall_mastery_score / 100))
})

const sections = computed<GapSection[]>(() => {
  const definitions = [
    { key: 'critical', title: 'Critical Gaps', severity: 'critical' as const },
    { key: 'high', title: 'High Priority', severity: 'warning' as const },
    { key: 'medium', title: 'Medium Priority', severity: 'slipping' as const },
    { key: 'low', title: 'Low Priority', severity: 'improving' as const },
  ]

  const grouped: GapSection[] = definitions.map((definition) => {
    const matches = priorityGaps.value.filter(gap => gap.severity_label === definition.key)
    return {
      title: definition.title,
      severity: definition.severity,
      count: matches.length,
      items: matches.map(gap => ({ name: gap.topic_name, score: gap.gap_score })),
    }
  })

  grouped.push({
    title: 'Recently Solidified',
    severity: 'fixed' as const,
    count: gapDash.value?.topics_solidified ?? 0,
    items: [],
  })

  return grouped
})

const allGapTopics = computed(() => priorityGaps.value)

const subjectSignalCards = computed(() => {
  const snapshot = subjectSnapshot.value
  if (!snapshot) return []

  return [
    { label: 'Total Gap', value: `${snapshot.total_gap_percent}%`, tone: snapshot.total_gap_percent >= 50 ? 'var(--warm)' : 'var(--gold)' },
    { label: 'Unknown', value: `${snapshot.unknown_percent}%`, tone: 'var(--gold)' },
    { label: 'Weak', value: `${snapshot.weak_percent}%`, tone: 'var(--ink)' },
    { label: 'Critical', value: `${snapshot.critical_percent}%`, tone: 'var(--warm)' },
    { label: 'Mastered', value: `${snapshot.mastered_skills}/${snapshot.total_skills}`, tone: 'var(--accent)' },
    { label: 'Blockers', value: `${snapshot.critical_blockers}`, tone: 'var(--warm)' },
  ]
})

function repairBadgeColor(severity: string): 'danger' | 'warm' | 'muted' {
  if (severity === 'critical') return 'danger'
  if (severity === 'high') return 'warm'
  return 'muted'
}

function feedColor(severity: string): string {
  if (severity === 'critical') return 'var(--warm)'
  if (severity === 'warning') return 'var(--gold)'
  if (severity === 'positive') return 'var(--accent)'
  return 'var(--ink-muted)'
}

function formatDate(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
}

type GapSectionSeverity = 'critical' | 'warning' | 'slipping' | 'improving' | 'fixed'
type GapSection = {
  title: string
  severity: GapSectionSeverity
  count: number
  items: { name: string; score: number }[]
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Knowledge Gaps</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          See what you do not know yet
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Real gap pressure, subject signals, and repair plans in one place
        </p>
      </div>
      <div class="flex items-center gap-4">
        <div v-if="!loading" class="text-right">
          <p class="text-3xl font-black tabular-nums" :style="{ color: gapPercentage > 50 ? 'var(--warm)' : gapPercentage > 25 ? 'var(--gold)' : 'var(--accent)' }">
            {{ gapPercentage }}%
          </p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Overall Gap</p>
        </div>
        <button class="cta-btn" @click="router.push('/student/knowledge-gap/scan')">Scan My Gaps -></button>
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Full Map</button>
      </div>
    </div>

    <div
      class="flex-shrink-0 grid grid-cols-4 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ gapPercentage }}%</p>
        <p class="stat-lbl">Gap</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ gapDash?.critical_gap_count ?? 0 }}</p>
        <p class="stat-lbl">Critical</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ gapDash?.active_repair_count ?? 0 }}</p>
        <p class="stat-lbl">In Repair</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ gapDash?.topics_solidified ?? 0 }}</p>
        <p class="stat-lbl">Solidified</p>
      </div>
    </div>

    <div
      v-if="subjects.length"
      class="flex-shrink-0 px-7 py-3 border-b flex items-center gap-2 flex-wrap"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <button
        v-for="subject in subjects"
        :key="subject.subject_id"
        class="subject-pill"
        :class="{ active: selectedSubjectId === subject.subject_id }"
        @click="selectSubject(subject.subject_id)"
      >
        {{ subject.subject_name }}
      </button>
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
        <div v-if="!allGapTopics.length" class="flex flex-col items-center justify-center h-full gap-4 text-center">
          <p class="text-2xl font-black" :style="{ color: 'var(--accent)' }">CLEAR</p>
          <p class="font-display text-xl font-bold" :style="{ color: 'var(--ink)' }">No significant gaps</p>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Keep practicing to stay strong.</p>
        </div>

        <template v-else>
          <div v-if="sections.some(section => section.count > 0)" class="grid grid-cols-2 gap-4">
            <GapSectionCard
              v-for="section in sections.filter(section => section.count > 0)"
              :key="section.title"
              :title="section.title"
              :severity="section.severity"
              :count="section.count"
              :items="section.items"
            />
          </div>

          <div v-if="gapDash?.repairs.length">
            <p class="section-label mb-3">Active Repair Plans</p>
            <div class="space-y-2">
              <div
                v-for="plan in gapDash.repairs"
                :key="plan.id"
                class="repair-row flex items-center gap-3 px-4 py-3 rounded-lg border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              >
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ plan.topic_name ?? 'Topic' }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    {{ plan.dominant_focus }} - {{ plan.recommended_session_type }}
                  </p>
                </div>
                <div class="text-right flex-shrink-0 flex items-center gap-3">
                  <div class="w-16">
                    <div class="h-1.5 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                      <div class="h-full rounded-full" :style="{ width: `${plan.progress_percent}%`, backgroundColor: 'var(--accent)' }" />
                    </div>
                    <p class="text-[10px] font-bold mt-0.5 text-right" :style="{ color: 'var(--accent)' }">{{ plan.progress_percent }}%</p>
                  </div>
                  <AppBadge :color="repairBadgeColor(plan.severity_label)" size="xs">{{ plan.severity_label }}</AppBadge>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <div
        class="w-88 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Subject Signal</p>
          <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ selectedSubjectName }}</p>
        </div>

        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div v-if="subjectLoading" class="space-y-2">
            <div
              v-for="i in 3"
              :key="i"
              class="h-16 rounded-lg animate-pulse"
              :style="{ backgroundColor: 'var(--border-soft)' }"
            />
          </div>

          <template v-else>
            <div class="signal-block">
              <div class="flex items-center justify-between gap-3 mb-3">
                <div>
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Current Snapshot</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    Fresh subject-level gap telemetry
                  </p>
                </div>
                <button class="mini-link" @click="router.push('/student/knowledge-gap/scan')">Rescan</button>
              </div>

              <div v-if="subjectSignalCards.length" class="signal-grid">
                <div
                  v-for="card in subjectSignalCards"
                  :key="card.label"
                  class="signal-card"
                >
                  <p class="signal-kicker">{{ card.label }}</p>
                  <p class="signal-value" :style="{ color: card.tone }">{{ card.value }}</p>
                </div>
              </div>
              <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                No subject snapshot yet. Run a targeted gap scan to generate one.
              </p>
            </div>

            <div class="signal-block">
              <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--ink)' }">Gap Feed</p>
              <div v-if="gapFeed.length" class="space-y-2">
                <div
                  v-for="event in gapFeed"
                  :key="event.id"
                  class="feed-row"
                >
                  <div class="flex items-start gap-3">
                    <div class="w-2 h-2 rounded-full flex-shrink-0 mt-1.5" :style="{ backgroundColor: feedColor(event.severity) }" />
                    <div class="min-w-0 flex-1">
                      <p class="text-[12px] leading-5" :style="{ color: 'var(--ink)' }">{{ event.message }}</p>
                      <p class="text-[10px] mt-1 uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                        {{ event.event_type.replace(/_/g, ' ') }} - {{ formatDate(event.created_at) }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
              <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                No repair feed events recorded for this subject yet.
              </p>
            </div>

            <div class="signal-block">
              <div class="flex items-center justify-between gap-3 mb-3">
                <div>
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Priority Gap Map</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ allGapTopics.length }} topics tracked</p>
                </div>
                <button class="mini-link" @click="router.push('/student/progress/mastery-map')">Open Map</button>
              </div>
              <div v-if="allGapTopics.length" class="flex flex-wrap gap-2">
                <GapMapNode
                  v-for="topic in allGapTopics"
                  :key="topic.topic_id"
                  :name="topic.topic_name"
                  :gap-score="topic.gap_score"
                  :state="topic.severity_label"
                  :is-blocker="topic.severity_label === 'critical'"
                  :selected="false"
                />
              </div>
              <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                No gap topics have been recorded yet.
              </p>
            </div>
          </template>
        </div>

        <div class="p-4 border-t flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="run-btn"
            @click="router.push('/student/knowledge-gap/scan')"
          >
            Run Gap Scan ->
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
  color: var(--warm);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.cta-btn {
  padding: 9px 18px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  transition: opacity 140ms, transform 140ms;
  border: none;
}

.cta-btn:hover {
  opacity: 0.85;
  transform: translateY(-1px);
}

.nav-pill,
.subject-pill {
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

.nav-pill:hover,
.subject-pill:hover,
.subject-pill.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
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

.repair-row,
.feed-row {
  transition: background-color 100ms;
}

.repair-row:hover,
.feed-row:hover {
  background-color: var(--paper) !important;
}

.signal-block {
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--paper);
  padding: 14px;
}

.signal-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.signal-card {
  border: 1px solid transparent;
  border-radius: 8px;
  background: var(--surface);
  padding: 12px;
}

.signal-kicker {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
  margin-bottom: 6px;
}

.signal-value {
  font-size: 18px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
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

.run-btn {
  width: 100%;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  background: var(--accent);
  color: white;
  border: none;
  cursor: pointer;
}
</style>
