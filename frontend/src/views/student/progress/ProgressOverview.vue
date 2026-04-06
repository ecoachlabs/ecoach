<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppProgress from '@/components/ui/AppProgress.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const topics = ref<TopicCaseDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, topicList] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 20),
    ])
    truth.value = t
    topics.value = topicList
  } catch (e) {
    console.error('Failed to load progress:', e)
  }
  loading.value = false
})

function readinessColor(): string {
  const band = truth.value?.overall_readiness_band ?? 'developing'
  if (band === 'strong') return 'var(--accent)'
  if (band === 'developing') return 'var(--gold)'
  return 'var(--warm)'
}

const bandLabel: Record<string, string> = {
  strong: 'Exam Ready', developing: 'Developing', weak: 'Needs Work', critical: 'Critical',
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Progress</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Academic snapshot
        </h1>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Mastery Map</button>
        <button class="nav-pill" @click="router.push('/student/progress/analytics')">Analytics</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
      </div>
    </div>

    <!-- Stats strip -->
    <div
      class="flex-shrink-0 grid grid-cols-5 divide-x border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <template v-if="loading">
        <div v-for="i in 5" :key="i" class="p-5 flex flex-col items-center gap-2">
          <div class="h-8 w-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          <div class="h-2.5 w-16 rounded animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
      </template>
      <template v-else>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: readinessColor() }">
            {{ bandLabel[truth?.overall_readiness_band ?? 'developing'] ?? '—' }}
          </p>
          <p class="stat-lbl">Readiness</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ truth?.topic_count ?? 0 }}</p>
          <p class="stat-lbl">Topics</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ truth?.memory_count ?? 0 }}</p>
          <p class="stat-lbl">Memories</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ truth?.due_memory_count ?? 0 }}</p>
          <p class="stat-lbl">Due Reviews</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ truth?.pending_review_count ?? 0 }}</p>
          <p class="stat-lbl">Pending</p>
        </div>
      </template>
    </div>

    <!-- Topic mastery list -->
    <div class="flex-1 overflow-hidden flex flex-col">
      <div class="px-7 py-3 border-b flex-shrink-0 flex items-center justify-between"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
        <p class="section-label">Topic Mastery — {{ topics.length }} topics</p>
        <button class="text-[11px] font-semibold" :style="{ color: 'var(--accent)' }"
          @click="router.push('/student/diagnostic')">Run Diagnostic →</button>
      </div>

      <div class="flex-1 overflow-y-auto">
        <div v-if="loading" class="p-4 space-y-2">
          <div v-for="i in 8" :key="i" class="h-12 rounded-xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <div v-else-if="!topics.length" class="flex flex-col items-center justify-center h-full gap-4">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Complete your diagnostic to see topic mastery data.</p>
          <button
            class="px-5 py-2 rounded-xl text-sm font-semibold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/diagnostic')"
          >Start Diagnostic</button>
        </div>

        <div v-else class="divide-y" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            v-for="topic in topics"
            :key="topic.topic_id"
            class="topic-row w-full text-left px-7 py-3.5 flex items-center gap-4"
            @click="router.push('/student/practice')"
          >
            <MasteryBadge :state="topic.mastery_state" size="sm" />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                {{ topic.subject_code }} · {{ topic.intervention_mode.replace(/_/g, ' ') }}
              </p>
            </div>
            <div class="w-36 flex-shrink-0">
              <AppProgress
                :value="topic.mastery_score"
                :max="10000"
                size="sm"
                :color="topic.mastery_score >= 6000 ? 'success' : topic.mastery_score >= 3000 ? 'gold' : 'danger'"
              />
            </div>
            <span class="text-sm tabular-nums font-bold w-12 text-right"
              :style="{
                color: topic.mastery_score >= 6000 ? 'var(--accent)' : topic.mastery_score >= 3000 ? 'var(--gold)' : 'var(--warm)'
              }">
              {{ (topic.mastery_score / 100).toFixed(0) }}%
            </span>
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
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

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

.topic-row {
  transition: background-color 100ms ease;
}
.topic-row:hover { background-color: var(--paper); }
</style>
