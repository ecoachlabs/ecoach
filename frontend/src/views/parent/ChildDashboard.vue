<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import { getReadinessReport, type ReadinessReportDto } from '@/ipc/readiness'
import AppProgress from '@/components/ui/AppProgress.vue'

const props = defineProps<{ id: string }>()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const truth = ref<LearnerTruthDto | null>(null)
const readiness = ref<ReadinessReportDto | null>(null)

onMounted(async () => {
  const studentId = Number(props.id)
  try {
    ;[truth.value, readiness.value] = await Promise.all([
      getLearnerTruth(studentId),
      getReadinessReport(studentId),
    ])
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load data'
  }
  loading.value = false
})

const masteryPercent = computed(() =>
  truth.value ? Math.round(truth.value.overall_mastery_score / 100) : 0,
)

function bandColor(band: string): string {
  const n = band.toLowerCase()
  if (n.includes('strong') || n.includes('ready')) return 'var(--accent)'
  if (n.includes('risk')) return 'var(--warm)'
  return 'var(--gold)'
}

function progressColor(band: string): 'accent' | 'warm' | 'gold' {
  const n = band.toLowerCase()
  if (n.includes('strong') || n.includes('ready')) return 'accent'
  if (n.includes('risk')) return 'warm'
  return 'gold'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center gap-4"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <button class="back-btn" @click="router.push('/parent')">← Overview</button>
      <div class="flex-1">
        <p class="eyebrow">Child Profile</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          {{ truth?.student_name ?? 'Loading…' }}
        </h1>
      </div>
      <span v-if="truth" class="readiness-label"
        :style="{ color: bandColor(truth.overall_readiness_band) }">
        {{ truth.overall_readiness_band.replace(/_/g, ' ') }}
      </span>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7">
      <div v-if="loading" class="space-y-4">
        <div class="h-24 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        <div class="h-48 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else-if="error" class="text-center py-16">
        <p class="text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
      </div>

      <template v-else-if="truth">

        <!-- Stats strip -->
        <div class="grid grid-cols-4 gap-3 mb-6">
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ masteryPercent }}%</p>
            <p class="stat-label">Mastery</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ truth.topic_count }}</p>
            <p class="stat-label">Topics Covered</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--gold)' }">{{ truth.memory_count }}</p>
            <p class="stat-label">Memories Built</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums"
              :style="{ color: truth.pending_review_count > 0 ? 'var(--warm)' : 'var(--ink-muted)' }">
              {{ truth.pending_review_count }}
            </p>
            <p class="stat-label">Reviews Due</p>
          </div>
        </div>

        <!-- Subject readiness -->
        <div v-if="readiness" class="px-6 py-5 rounded-2xl border mb-4"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-4">Subject Readiness</p>
          <div class="space-y-4">
            <div v-for="subject in readiness.subjects" :key="subject.subject_id">
              <div class="flex items-center justify-between mb-1.5">
                <span class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ subject.subject_name }}</span>
                <div class="flex items-center gap-3">
                  <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">
                    {{ subject.mastered_topic_count }}/{{ subject.total_topic_count }} mastered
                  </span>
                  <span class="readiness-chip" :style="{ color: bandColor(subject.readiness_band) }">
                    {{ subject.readiness_band.replace(/_/g, ' ') }}
                  </span>
                </div>
              </div>
              <AppProgress
                :value="subject.total_topic_count > 0 ? subject.mastered_topic_count : 0"
                :max="subject.total_topic_count || 1"
                size="sm"
                :color="progressColor(subject.readiness_band)"
              />
            </div>
          </div>
        </div>

        <!-- Coverage -->
        <div v-if="readiness" class="flex items-center justify-between px-6 py-4 rounded-2xl border"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
          <div>
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Overall BECE Coverage</p>
            <p class="text-xs mt-0.5" :style="{ color: 'var(--ink-muted)' }">Percentage of syllabus covered</p>
          </div>
          <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">
            {{ readiness.coverage_percent }}%
          </p>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.section-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.14em; color: var(--ink-muted); }

.back-btn {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
  flex-shrink: 0;
}
.back-btn:hover { background: var(--border-soft); color: var(--ink); }

.readiness-label { font-size: 13px; font-weight: 700; text-transform: capitalize; }

.stat-card { padding: 16px; border-radius: 14px; border: 1px solid var(--border-soft); background: var(--surface); }
.stat-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--ink-muted); margin-top: 4px; }

.readiness-chip { font-size: 10px; font-weight: 700; text-transform: capitalize; }
</style>
