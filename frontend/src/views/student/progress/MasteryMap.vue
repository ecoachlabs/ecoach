<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getMasteryMap,
  getStudentDashboard,
  listActiveMisconceptions,
  refreshMasteryMap,
  type LearnerMisconceptionSnapshotDto,
  type MasteryMapNodeDto,
  type StudentDashboardDto,
} from '@/ipc/coach'
import KnowledgeMap, { type MapLink, type MapNode } from '@/components/viz/KnowledgeMap.vue'
import TopicStatusCard from '@/components/viz/TopicStatusCard.vue'
import { getMasteryColor, getMasteryLabel } from '@/utils/mastery'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const dashboard = ref<StudentDashboardDto | null>(null)
const selectedSubjectId = ref<number | null>(null)
const masteryNodes = ref<MasteryMapNodeDto[]>([])
const misconceptionAlerts = ref<LearnerMisconceptionSnapshotDto[]>([])
const selectedNodeId = ref<number | null>(null)
const mapSnapshotLabel = ref('')
const mapSnapshotWarning = ref('')

const subjects = computed(() => dashboard.value?.subjects ?? [])

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    dashboard.value = await getStudentDashboard(auth.currentAccount.id)
    selectedSubjectId.value = dashboard.value.subjects[0]?.subject_id ?? null
    if (selectedSubjectId.value) {
      await loadSubjectSnapshot(selectedSubjectId.value)
    }
  } catch (error) {
    console.error('Failed to load mastery map:', error)
  } finally {
    loading.value = false
  }
})

async function loadSubjectSnapshot(subjectId: number) {
  if (!auth.currentAccount) return

  loading.value = true
  const studentId = auth.currentAccount.id
  const misconceptionsPromise = listActiveMisconceptions(studentId, subjectId).catch(() => [])
  try {
    masteryNodes.value = await refreshMasteryMap(studentId, subjectId)
    mapSnapshotLabel.value = masteryNodes.value.length
      ? 'Fresh recompute from current learner data'
      : 'No active topic map data yet'
    mapSnapshotWarning.value = ''
  } catch (error) {
    console.error('Failed to refresh mastery map live:', error)
    masteryNodes.value = await getMasteryMap(studentId, subjectId).catch(() => [])
    mapSnapshotLabel.value = masteryNodes.value.length
      ? 'Last saved map snapshot'
      : 'No saved mastery map yet'
    mapSnapshotWarning.value = masteryNodes.value.length
      ? 'Live recompute failed. This snapshot may lag behind your latest work.'
      : 'Live recompute failed and no saved map was available.'
  } finally {
    const misconceptions = await misconceptionsPromise
    misconceptionAlerts.value = misconceptions.sort((left, right) => right.risk_score - left.risk_score)
    selectedNodeId.value = masteryNodes.value[0]?.topic_id ?? null
    loading.value = false
  }
}

async function selectSubject(subjectId: number) {
  if (selectedSubjectId.value === subjectId) return
  selectedSubjectId.value = subjectId
  await loadSubjectSnapshot(subjectId)
}

function handleSelect(nodeId: number) {
  selectedNodeId.value = nodeId
}

function topicState(node: MasteryMapNodeDto): string {
  if (node.stability_state === 'mastered') return 'exam_ready'
  if (node.stability_state === 'strong') return 'robust'
  if (node.stability_state === 'stable') return 'stable'
  if (node.stability_state === 'building') return 'partial'
  if (node.stability_state === 'started') return 'emerging'
  if (node.stability_state === 'fragile') return 'fragile'
  return 'unseen'
}

function masteryColor(node: MasteryMapNodeDto): string {
  return getMasteryColor(topicState(node))
}

function masteryLabel(node: MasteryMapNodeDto): string {
  return getMasteryLabel(topicState(node))
}

function statusLabel(status: string): string {
  return status.replace(/_/g, ' ')
}

function formatPercent(bp: number): string {
  return `${Math.round(bp / 100)}%`
}

function lastActivityLabel(value: string | null): string {
  if (!value) return 'No activity yet'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })
}

const topicLookup = computed(() => new Map(masteryNodes.value.map(node => [node.topic_id, node.topic_name])))

const blockedTopicCount = computed(() => masteryNodes.value.filter(node => node.is_blocked).length)
const highYieldCount = computed(() => masteryNodes.value.filter(node => node.is_high_yield).length)

const mapNodes = computed<MapNode[]>(() =>
  masteryNodes.value.map(node => ({
    id: node.topic_id,
    name: node.topic_name,
    masteryState: topicState(node),
    score: node.mastery_percentage_bp,
    type: 'topic',
  })),
)

const mapLinks = computed<MapLink[]>(() => {
  const nodeIds = new Set(masteryNodes.value.map(node => node.topic_id))
  const seen = new Set<string>()
  const links: MapLink[] = []

  for (const node of masteryNodes.value) {
    for (const blockerId of node.blocked_by_topic_ids) {
      if (!nodeIds.has(blockerId)) continue
      const key = `${blockerId}->${node.topic_id}`
      if (seen.has(key)) continue
      seen.add(key)
      links.push({
        source: blockerId,
        target: node.topic_id,
        type: 'prerequisite',
      })
    }
  }

  return links
})

const selectedTopic = computed(() =>
  masteryNodes.value.find(node => node.topic_id === selectedNodeId.value) ?? null,
)

const selectedBlockers = computed(() =>
  selectedTopic.value?.blocked_by_topic_ids.map(id => topicLookup.value.get(id) ?? `Topic ${id}`) ?? [],
)

const selectedSignals = computed(() => {
  if (!selectedTopic.value) return []
  return misconceptionAlerts.value.filter(signal => signal.topic_id === selectedTopic.value?.topic_id)
})
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
          Mastery Map
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Live dependency pressure, blockers, and misconception signals by subject
        </p>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress')">Overview</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
        <button class="nav-pill" @click="router.push('/student/progress/analytics')">Analytics</button>
      </div>
    </div>

    <div
      class="flex-shrink-0 px-7 py-3 border-b flex items-center justify-between gap-4"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div v-if="subjects.length" class="flex items-center gap-2 flex-wrap">
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
      <p v-else class="text-xs" :style="{ color: 'var(--ink-muted)' }">
        No subject mastery data yet.
      </p>
      <div v-if="mapSnapshotLabel" class="text-right shrink-0">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.14em]"
          :style="{ color: mapSnapshotWarning ? 'var(--warm)' : 'var(--accent)' }"
        >
          {{ mapSnapshotLabel }}
        </p>
        <p v-if="mapSnapshotWarning" class="text-[10px] mt-1 max-w-[280px]" :style="{ color: 'var(--warm)' }">
          {{ mapSnapshotWarning }}
        </p>
      </div>
    </div>

    <div
      class="flex-shrink-0 grid grid-cols-4 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ masteryNodes.length }}</p>
        <p class="stat-lbl">Tracked Topics</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: blockedTopicCount ? 'var(--warm)' : 'var(--ink-muted)' }">
          {{ blockedTopicCount }}
        </p>
        <p class="stat-lbl">Blocked</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: highYieldCount ? 'var(--gold)' : 'var(--ink-muted)' }">
          {{ highYieldCount }}
        </p>
        <p class="stat-lbl">High Yield</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: misconceptionAlerts.length ? 'var(--warm)' : 'var(--ink-muted)' }">
          {{ misconceptionAlerts.length }}
        </p>
        <p class="stat-lbl">Misconception Alerts</p>
      </div>
    </div>

    <div v-if="loading" class="flex-1 p-6">
      <div class="h-full rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-hidden p-6">
        <KnowledgeMap
          :nodes="mapNodes"
          :links="mapLinks"
          :selected-node-id="selectedNodeId"
          :height="520"
          @select-node="handleSelect"
        />
      </div>

      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Topic Detail</p>
        </div>

        <div class="flex-1 overflow-y-auto p-4">
          <div v-if="selectedTopic" class="space-y-4">
            <TopicStatusCard
              :topic-name="selectedTopic.topic_name"
              :mastery-state="topicState(selectedTopic)"
              :mastery-score="selectedTopic.mastery_percentage_bp"
            />

            <div class="detail-card">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <p class="text-xs font-bold" :style="{ color: 'var(--ink)' }">{{ masteryLabel(selectedTopic) }}</p>
                  <p class="text-[11px] capitalize" :style="{ color: 'var(--ink-muted)' }">
                    {{ selectedTopic.stability_state.replace(/_/g, ' ') }}
                  </p>
                </div>
                <div class="flex items-center gap-2 flex-wrap justify-end">
                  <span
                    v-if="selectedTopic.is_blocked"
                    class="signal-chip"
                    :style="{ backgroundColor: 'rgba(194,65,12,0.1)', color: 'var(--warm)' }"
                  >
                    Blocked
                  </span>
                  <span
                    v-if="selectedTopic.is_high_yield"
                    class="signal-chip"
                    :style="{ backgroundColor: 'rgba(202,138,4,0.1)', color: 'var(--gold)' }"
                  >
                    High Yield
                  </span>
                </div>
              </div>
            </div>

            <div class="metric-grid">
              <div class="metric-box">
                <p class="metric-kicker">Mastery</p>
                <p class="metric-value" :style="{ color: masteryColor(selectedTopic) }">
                  {{ formatPercent(selectedTopic.mastery_percentage_bp) }}
                </p>
              </div>
              <div class="metric-box">
                <p class="metric-kicker">Exam Risk</p>
                <p class="metric-value" :style="{ color: selectedTopic.exam_risk_bp >= 6000 ? 'var(--warm)' : 'var(--ink)' }">
                  {{ formatPercent(selectedTopic.exam_risk_bp) }}
                </p>
              </div>
              <div class="metric-box">
                <p class="metric-kicker">Score Impact</p>
                <p class="metric-value" :style="{ color: selectedTopic.score_impact_bp >= 6000 ? 'var(--gold)' : 'var(--ink)' }">
                  {{ formatPercent(selectedTopic.score_impact_bp) }}
                </p>
              </div>
              <div class="metric-box">
                <p class="metric-kicker">Dependents</p>
                <p class="metric-value" :style="{ color: 'var(--ink)' }">{{ selectedTopic.dependent_count }}</p>
              </div>
            </div>

            <div class="detail-card">
              <p class="section-label mb-2">Prerequisites</p>
              <p class="text-[11px] mb-2" :style="{ color: 'var(--ink-muted)' }">
                {{ selectedTopic.dependency_count }} dependency links tracked
              </p>
              <div v-if="selectedBlockers.length" class="flex flex-wrap gap-2">
                <span
                  v-for="blocker in selectedBlockers"
                  :key="blocker"
                  class="signal-chip"
                  :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-secondary)' }"
                >
                  {{ blocker }}
                </span>
              </div>
              <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                No active blocker topics for this node right now.
              </p>
            </div>

            <div class="detail-card">
              <p class="section-label mb-2">Latest Activity</p>
              <p class="text-[11px]" :style="{ color: 'var(--ink-secondary)' }">
                {{ lastActivityLabel(selectedTopic.last_activity_at) }}
              </p>
            </div>

            <div class="detail-card">
              <p class="section-label mb-2">Misconception Pressure</p>
              <div v-if="selectedSignals.length" class="space-y-2">
                <div
                  v-for="signal in selectedSignals"
                  :key="signal.misconception_id"
                  class="signal-row"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div>
                      <p class="text-[12px] font-semibold" :style="{ color: 'var(--ink)' }">{{ signal.title }}</p>
                      <p class="text-[10px] capitalize" :style="{ color: 'var(--ink-muted)' }">
                        {{ statusLabel(signal.current_status) }} · detected {{ signal.times_detected }}x
                      </p>
                    </div>
                    <span class="text-[10px] font-bold" :style="{ color: signal.risk_score >= 6000 ? 'var(--warm)' : 'var(--ink)' }">
                      {{ formatPercent(signal.risk_score) }}
                    </span>
                  </div>
                </div>
              </div>
              <p v-else class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                No active misconception state is currently linked to this topic.
              </p>
            </div>
          </div>

          <div v-else class="py-16 text-center px-4">
            <div
              class="w-12 h-12 rounded-full border-2 flex items-center justify-center mx-auto mb-3"
              :style="{ borderColor: 'transparent' }"
            >
              <span class="text-lg" :style="{ color: 'var(--ink-muted)' }">O</span>
            </div>
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
              Select a topic node to inspect its real mastery state.
            </p>
          </div>
        </div>

          <div class="px-5 py-4 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="start-btn w-full"
            @click="router.push(selectedTopic ? `/student/practice?topic=${selectedTopic.topic_id}` : '/student/practice')"
          >Practice This Topic -></button>
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
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.nav-pill,
.subject-pill {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
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

.detail-card,
.metric-box,
.signal-row {
  border: 1px solid transparent;
  border-radius: 12px;
  background: var(--paper);
}

.detail-card,
.signal-row {
  padding: 12px;
}

.signal-chip {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.metric-box {
  padding: 12px;
}

.metric-kicker {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
  margin-bottom: 6px;
}

.metric-value {
  font-size: 18px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.start-btn {
  padding: 10px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  border: none;
  transition: opacity 140ms;
}

.start-btn:hover {
  opacity: 0.87;
}
</style>
