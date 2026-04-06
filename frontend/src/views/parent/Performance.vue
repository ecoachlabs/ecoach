<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const dashboard = ref<ParentDashboardSnapshot | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    dashboard.value = await buildParentDashboard(auth.currentAccount.id)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load performance data'
  }
  loading.value = false
})

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
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Family</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Performance</h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Academic performance overview for all your children</p>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7">
      <div v-if="loading" class="space-y-4">
        <div v-for="i in 2" :key="i" class="h-48 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <template v-else-if="dashboard && dashboard.students.length > 0">
        <div class="space-y-5">
          <div
            v-for="student in dashboard.students"
            :key="student.student_id"
            class="student-card px-6 py-5 rounded-2xl border"
            :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
          >
            <!-- Student header -->
            <div class="flex items-center justify-between mb-5">
              <div class="flex items-center gap-3">
                <div class="student-avatar">{{ student.student_name.charAt(0).toUpperCase() }}</div>
                <div>
                  <h2 class="font-display text-lg font-bold" :style="{ color: 'var(--ink)' }">{{ student.student_name }}</h2>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    Target: {{ student.exam_target ?? 'Not set' }}
                  </p>
                </div>
              </div>
              <span class="readiness-label" :style="{ color: bandColor(student.overall_readiness_band) }">
                {{ student.overall_readiness_band.replace(/_/g, ' ') }}
              </span>
            </div>

            <!-- Subject breakdown -->
            <div v-if="(student as any).subject_summaries?.length" class="space-y-3 mb-5">
              <div v-for="subj in (student as any).subject_summaries" :key="subj.subject_id">
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ subj.subject_name }}</span>
                  <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">
                    {{ subj.mastered_topic_count }}/{{ subj.total_topic_count }} topics mastered
                  </span>
                </div>
                <AppProgress
                  :value="subj.mastered_topic_count"
                  :max="subj.total_topic_count || 1"
                  size="sm"
                  :color="progressColor(subj.readiness_band)"
                />
              </div>
            </div>

            <!-- Active risks -->
            <div v-if="student.active_risks?.length" class="pt-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
              <p class="text-[10px] uppercase font-bold mb-2" :style="{ color: 'var(--ink-muted)' }">Active Concerns</p>
              <div class="space-y-1">
                <p v-for="risk in student.active_risks.slice(0, 3)" :key="risk.title"
                  class="text-xs" :style="{ color: 'var(--warm)' }">
                  · {{ risk.title }}
                </p>
              </div>
            </div>

            <div class="mt-4">
              <button class="view-btn" @click="router.push('/parent/child/' + student.student_id)">
                View Dashboard →
              </button>
            </div>
          </div>
        </div>
      </template>

      <div v-else-if="!loading" class="flex flex-col items-center justify-center h-64 gap-3">
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No children linked to your account yet.</p>
        <button class="view-btn" @click="router.push('/parent/children')">Add Children →</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }

.student-card { transition: background-color 100ms; }
.student-card:hover { background-color: var(--paper) !important; }

.student-avatar {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: var(--ink);
  color: var(--paper);
  font-size: 16px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
}

.readiness-label {
  font-size: 12px;
  font-weight: 700;
  text-transform: capitalize;
}

.view-btn {
  padding: 7px 18px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.view-btn:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
</style>
