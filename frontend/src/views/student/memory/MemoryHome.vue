<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, listSubjects, type LearnerTruthDto, type SubjectDto } from '@/ipc/coach'
import { getMemoryDashboard, getReviewQueue, listMemoryTopicSummaries, type MemoryDashboardDto, type RecheckItemDto, type TopicMemorySummaryDto } from '@/ipc/memory'
import { startPracticeSession } from '@/ipc/sessions'
import MemoryHeatmap from '@/components/modes/memory/MemoryHeatmap.vue'
import RecoveryLadder from '@/components/modes/memory/RecoveryLadder.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const loading = ref(true)
const launching = ref(false)
const error = ref('')
const truth = ref<LearnerTruthDto | null>(null)
const memoryDash = ref<MemoryDashboardDto | null>(null)
const reviewQueue = ref<RecheckItemDto[]>([])
const topicSummaries = ref<TopicMemorySummaryDto[]>([])
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, md, rq, ts, subs] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getMemoryDashboard(auth.currentAccount.id).catch(() => null),
      getReviewQueue(auth.currentAccount.id, 30).catch(() => []),
      listMemoryTopicSummaries(auth.currentAccount.id, 20).catch(() => []),
      listSubjects(),
    ])
    truth.value = t
    memoryDash.value = md
    reviewQueue.value = rq
    topicSummaries.value = ts
    subjects.value = subs
    if (subs.length > 0) selectedSubjectId.value = subs[0].id
  } catch {}
  loading.value = false
})

const memoryHealth = computed(() => memoryDash.value?.average_strength ?? 0)
const fadingCount = computed(() => (memoryDash.value?.fading_count ?? 0) + (memoryDash.value?.at_risk_count ?? 0))

const heatmapTopics = computed(() =>
  topicSummaries.value.map(t => ({
    name: t.topic_name,
    strength: t.average_strength,
    status: (() => {
      if (t.collapsed_items > 0) return 'critical' as const
      if (t.average_strength >= 7000 && t.fragile_items === 0) return 'strong' as const
      if (t.average_strength >= 7000) return 'recovered' as const
      if (t.average_strength >= 5000) return 'vulnerable' as const
      if (t.average_strength >= 2000) return 'fading' as const
      return 'critical' as const
    })(),
  })),
)

const reviewTopicIds = computed(() => {
  const ids = reviewQueue.value.filter(r => r.node_id != null).map(r => r.node_id as number)
  return [...new Set(ids)].slice(0, 5)
})

const recoveryStage = computed(() => {
  const h = memoryHealth.value
  if (h >= 8000) return 4
  if (h >= 6000) return 3
  if (h >= 4000) return 2
  return 1
})

async function launchQuickScan() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value) return
  launching.value = true
  error.value = ''
  try {
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: reviewTopicIds.value,
      question_count: 10,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start scan'
    launching.value = false
  }
}

async function launchRescue() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value) return
  launching.value = true
  error.value = ''
  try {
    const fadingTopics = topicSummaries.value
      .filter(t => t.average_strength < 5000 || t.collapsed_items > 0)
      .slice(0, 5).map(t => t.topic_id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: fadingTopics,
      question_count: 12,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start rescue'
    launching.value = false
  }
}

watch([() => route.hash, loading], () => {
  if (!loading.value && route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    })
  }
}, { immediate: true })

function strengthColor(v: number) {
  if (v >= 7000) return 'var(--accent)'
  if (v >= 4000) return 'var(--gold)'
  return 'var(--warm)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Memory Mode</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Strengthen what you know
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Catch fading knowledge before it costs you marks</p>
      </div>
      <div v-if="!loading" class="text-right">
        <p class="text-3xl font-black tabular-nums" :style="{ color: memoryHealth >= 7000 ? 'var(--accent)' : memoryHealth >= 4000 ? 'var(--gold)' : 'var(--warm)' }">
          {{ (memoryHealth / 100).toFixed(0) }}%
        </p>
        <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Memory Strength</p>
      </div>
    </div>

    <!-- Stat strip -->
    <div
      class="flex-shrink-0 grid grid-cols-4 divide-x border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: fadingCount > 0 ? 'var(--gold)' : 'var(--ink-muted)' }">{{ fadingCount }}</p>
        <p class="stat-lbl">Fading</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: (truth?.due_memory_count ?? 0) > 0 ? 'var(--warm)' : 'var(--ink-muted)' }">
          {{ truth?.due_memory_count ?? memoryDash?.overdue_reviews ?? 0 }}
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
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div v-for="i in 5" :key="i" class="h-14 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: heatmap + recovery + actions -->
      <div class="flex-1 overflow-y-auto p-6 space-y-5">

        <!-- Subject tabs -->
        <div class="flex items-center gap-2 flex-wrap">
          <button
            v-for="subj in subjects"
            :key="subj.id"
            class="subj-tab"
            :class="{ active: selectedSubjectId === subj.id }"
            @click="selectedSubjectId = subj.id"
          >{{ subj.name }}</button>
        </div>

        <!-- Memory Heatmap -->
        <div v-if="heatmapTopics.length" id="memory-map">
          <p class="section-label mb-3">Memory Health Map</p>
          <div class="rounded-2xl border p-4" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
            <MemoryHeatmap :topics="heatmapTopics" />
          </div>
        </div>

        <!-- Recovery Ladder -->
        <div id="recovery">
          <p class="section-label mb-3">Recovery Stage</p>
          <div class="rounded-2xl border p-4" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
            <RecoveryLadder :current-stage="recoveryStage" />
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-3">
          <button class="action-btn primary" :disabled="launching" @click="launchQuickScan">
            Quick Scan →
          </button>
          <button v-if="fadingCount > 0" class="action-btn secondary" :disabled="launching" @click="launchRescue">
            Rescue {{ fadingCount }} Fading
          </button>
        </div>
      </div>

      <!-- Right: Review queue -->
      <div
        id="reviews"
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0 flex items-center justify-between"
          :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Due Reviews</p>
          <span v-if="reviewQueue.length"
            class="text-[10px] font-bold px-2 py-0.5 rounded-full"
            :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }">{{ reviewQueue.length }}</span>
        </div>
        <div class="flex-1 overflow-y-auto p-3 space-y-1.5">
          <div v-if="!reviewQueue.length" class="py-10 text-center">
            <p class="text-sm font-bold mb-1" :style="{ color: 'var(--accent)' }">✓</p>
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">All caught up!<br>No reviews due.</p>
          </div>
          <div
            v-for="item in reviewQueue"
            :key="item.id"
            class="review-item px-3 py-3 rounded-xl border"
            :style="{ borderColor: 'var(--border-soft)' }"
          >
            <div class="flex items-center gap-3">
              <div class="w-2 h-2 rounded-full flex-shrink-0"
                :style="{ backgroundColor: strengthColor(item.memory_strength ?? 0) }" />
              <div class="flex-1 min-w-0">
                <p class="text-[11px] font-semibold truncate" :style="{ color: 'var(--ink)' }">
                  {{ item.topic_name ?? item.node_title ?? 'Review item' }}
                </p>
                <p class="text-[9px]" :style="{ color: 'var(--ink-muted)' }">{{ item.schedule_type }}</p>
              </div>
              <span class="text-[9px] font-bold flex-shrink-0"
                :style="{ color: strengthColor(item.memory_strength ?? 0) }">
                {{ Math.round((item.memory_strength ?? 0) / 100) }}%
              </span>
            </div>
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

.subj-tab {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--border-soft);
  color: var(--ink-secondary);
  background: transparent;
  transition: all 120ms;
}
.subj-tab.active, .subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.review-item {
  transition: background-color 100ms;
}
.review-item:hover { background-color: var(--paper); }

.action-btn {
  padding: 10px 20px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 140ms, transform 140ms;
}
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.action-btn.primary { background: var(--accent); color: white; }
.action-btn.primary:hover:not(:disabled) { opacity: 0.87; transform: translateY(-1px); }
.action-btn.secondary { background: var(--accent-glow); color: var(--accent); border: 1px solid var(--accent); }
.action-btn.secondary:hover:not(:disabled) { background: var(--accent); color: white; }
</style>
