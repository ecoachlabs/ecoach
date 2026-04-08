<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getReadinessReport } from '@/ipc/readiness'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const readiness = ref<any>(null)
const truth = ref<LearnerTruthDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [r, t] = await Promise.all([
      getReadinessReport(auth.currentAccount.id),
      getLearnerTruth(auth.currentAccount.id),
    ])
    readiness.value = r
    truth.value = t
  } catch (e) {
    console.error('Failed to load analytics:', e)
  }
  loading.value = false
})

const coveragePct = computed(() => {
  if (!readiness.value) return 0
  return Math.round(readiness.value.coverage_percent * 100)
})

function bandColor(band: string): string {
  if (band === 'strong') return 'var(--accent)'
  if (band === 'developing') return 'var(--gold)'
  return 'var(--warm)'
}

const overallBandColor = computed(() => bandColor(readiness.value?.overall_readiness_band ?? 'developing'))
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
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
        <button class="nav-pill" @click="router.push('/student/progress')">← Overview</button>
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Mastery Map</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
      </div>
    </div>

    <!-- Stat strip -->
    <div
      class="flex-shrink-0 grid grid-cols-4 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ coveragePct }}%</p>
        <p class="stat-lbl">Coverage</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ truth?.diagnosis_count ?? 0 }}</p>
        <p class="stat-lbl">Diagnoses</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ truth?.skill_count ?? 0 }}</p>
        <p class="stat-lbl">Skills</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big capitalize" :style="{ color: overallBandColor }">
          {{ readiness?.overall_readiness_band ?? '—' }}
        </p>
        <p class="stat-lbl">Band</p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div v-for="i in 5" :key="i" class="h-16 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Main: subject breakdown -->
      <div class="flex-1 overflow-y-auto p-6">

        <div v-if="readiness?.subjects?.length">
          <p class="section-label mb-4">Subject Readiness</p>
          <div class="space-y-3">
            <div
              v-for="s in readiness.subjects"
              :key="s.subject_id"
              class="subject-card px-5 py-4 rounded-2xl border"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
            >
              <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-3">
                  <div class="w-9 h-9 rounded-xl flex items-center justify-center text-sm font-black"
                    :style="{ background: 'var(--paper)', color: 'var(--ink)', border: '1px solid transparent' }">
                    {{ s.subject_name?.charAt(0) ?? '?' }}
                  </div>
                  <div>
                    <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ s.subject_name }}</p>
                    <p class="text-[10px] capitalize" :style="{ color: bandColor(s.readiness_band) }">{{ s.readiness_band }}</p>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-lg font-black tabular-nums" :style="{ color: bandColor(s.readiness_band) }">
                    {{ s.mastered_topic_count }}<span class="text-sm font-semibold" :style="{ color: 'var(--ink-muted)' }">/{{ s.total_topic_count }}</span>
                  </p>
                  <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">topics mastered</p>
                </div>
              </div>
              <AppProgress
                :value="s.mastered_topic_count"
                :max="s.total_topic_count || 1"
                size="md"
                :color="s.readiness_band === 'strong' ? 'success' : s.readiness_band === 'developing' ? 'gold' : 'danger'"
              />
            </div>
          </div>
        </div>

        <!-- Empty state -->
        <div v-else class="flex flex-col items-center justify-center h-full gap-4 text-center">
          <p class="text-sm font-semibold" :style="{ color: 'var(--ink-muted)' }">Complete your diagnostic to unlock analytics</p>
          <button class="px-5 py-2.5 rounded-xl font-semibold text-sm"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/diagnostic')">Start Diagnostic</button>
        </div>
      </div>

      <!-- Right panel: metrics -->
      <div
        class="w-60 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Key Metrics</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-2">
          <div class="metric-row">
            <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Overall Band</p>
            <p class="text-sm font-bold capitalize" :style="{ color: overallBandColor }">
              {{ readiness?.overall_readiness_band ?? '—' }}
            </p>
          </div>
          <div class="metric-row">
            <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Syllabus Coverage</p>
            <p class="text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ coveragePct }}%</p>
          </div>
          <div class="metric-row">
            <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Topics Tracked</p>
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ truth?.topic_count ?? 0 }}</p>
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
            <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">Diagnoses Run</p>
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ truth?.diagnosis_count ?? 0 }}</p>
          </div>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full py-2 rounded-xl text-xs font-semibold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/diagnostic')">Run Diagnostic →</button>
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
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

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
.subject-card:hover { border-color: var(--ink-muted) !important; }

.metric-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
}
</style>


