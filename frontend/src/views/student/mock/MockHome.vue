<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import { listMockSessions, type MockSessionSummaryDto } from '@/ipc/mock'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const subjects = ref<SubjectDto[]>([])
const history = ref<MockSessionSummaryDto[]>([])

const mockTypes = [
  { key: 'full', label: 'Full Mock', desc: 'Complete exam simulation with all sections', icon: '⊞', time: '2h 30m', duration: 150, count: 40 },
  { key: 'topic', label: 'Topic Mock', desc: 'Focus on specific topics you need to strengthen', icon: '◎', time: '30-45m', duration: 40, count: 20 },
  { key: 'mini', label: 'Mini Mock', desc: 'Quick 20-question check across all topics', icon: '◇', time: '15-20m', duration: 20, count: 20 },
  { key: 'pressure', label: 'Pressure Mock', desc: 'Tighter timing. Harder questions. Exam pressure.', icon: '⚡', time: '45m', duration: 45, count: 25 },
]

const readinessBand = ref('developing')
const readinessScore = ref(0)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, subs, sessions] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      listSubjects(1),
      listMockSessions(auth.currentAccount.id, 5),
    ])
    truth.value = t
    subjects.value = subs
    history.value = sessions
    readinessBand.value = t.overall_readiness_band
    // Map band to approximate score for ring display
    const bandScores: Record<string, number> = { strong: 7500, developing: 4800, weak: 2800, critical: 1500 }
    readinessScore.value = bandScores[t.overall_readiness_band] ?? 4800
  } catch (e) {
    console.error('Failed to load mock home:', e)
  }
  loading.value = false
})

function goToSetup(mockType: string) {
  router.push(`/student/mock/setup?type=${mockType}`)
}

function gradeColor(grade: string | null): string {
  if (!grade) return 'bg-gray-100 text-gray-500'
  if (['A1', 'B2', 'B3'].includes(grade)) return 'bg-emerald-50 text-emerald-600'
  if (['C4', 'C5', 'C6'].includes(grade)) return 'bg-amber-50 text-amber-600'
  return 'bg-red-50 text-red-500'
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Mock Centre</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Simulate real exam conditions. Discover where you truly stand.</p>
      </div>
      <ProgressRing
        v-if="!loading"
        :value="readinessScore"
        :max="10000"
        :size="64"
        :stroke-width="4"
        :color="readinessBand === 'strong' ? 'var(--success)' : readinessBand === 'developing' ? 'var(--accent)' : 'var(--danger)'"
        label="Readiness"
      />
      <div v-else class="w-16 h-16 rounded-full animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Mock Type Cards -->
    <div class="grid grid-cols-2 gap-4 mb-8">
      <AppCard v-for="mock in mockTypes" :key="mock.key" hover padding="lg" @click="goToSetup(mock.key)">
        <div class="flex items-start gap-3">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            {{ mock.icon }}
          </div>
          <div class="flex-1">
            <h3 class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ mock.label }}</h3>
            <p class="text-[11px] leading-relaxed mb-3" :style="{ color: 'var(--text-3)' }">{{ mock.desc }}</p>
            <AppBadge color="accent" size="xs">{{ mock.time }}</AppBadge>
          </div>
        </div>
      </AppCard>
    </div>

    <!-- Battle History -->
    <div>
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Battle History</h3>
        <AppButton variant="ghost" size="sm" @click="router.push('/student/mock/history')">View All</AppButton>
      </div>

      <!-- Loading skeleton -->
      <div v-if="loading" class="space-y-2">
        <div v-for="i in 3" :key="i" class="h-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else-if="history.length" class="space-y-2">
        <AppCard v-for="entry in history" :key="entry.id" padding="sm" hover
          @click="router.push(`/student/mock/review/${entry.id}`)">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold tabular-nums"
              :class="gradeColor(entry.grade)">
              {{ entry.grade ?? (entry.status === 'in_progress' ? '…' : '—') }}
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ entry.mock_type.replace(/_/g, ' ') }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">
                {{ entry.percentage != null ? entry.percentage.toFixed(0) + '%' : entry.status }}
                <span v-if="entry.paper_year" class="ml-2">· {{ entry.paper_year }}</span>
              </p>
            </div>
            <AppBadge
              :color="entry.status === 'completed' ? 'success' : entry.status === 'in_progress' ? 'warm' : 'muted'"
              size="xs"
            >
              {{ entry.status }}
            </AppBadge>
          </div>
        </AppCard>
      </div>

      <AppCard v-else padding="md" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">No mock sessions yet. Start your first one!</p>
      </AppCard>
    </div>
  </div>
</template>
