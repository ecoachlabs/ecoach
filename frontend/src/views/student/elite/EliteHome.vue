<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const topics = ref<TopicCaseDto[]>([])

const tier = ref('Core')
const eps = ref(6240) // Elite Performance Score (BasisPoints)
const streak = ref(7)

const identityPanel = ref([
  { label: 'Precision', score: 7200, icon: '◎' },
  { label: 'Speed', score: 5800, icon: '⚡' },
  { label: 'Depth', score: 6100, icon: '◈' },
  { label: 'Endurance', score: 5400, icon: '∞' },
  { label: 'Trap Sense', score: 4900, icon: '⚠' },
  { label: 'Independence', score: 6800, icon: '★' },
])

const sessionTypes = [
  { key: 'precision', label: 'Precision Lab', desc: 'Accuracy-focused. Steady timer.', icon: '◎' },
  { key: 'sprint', label: 'Elite Sprint', desc: 'Speed under pressure. Shrinking timer.', icon: '⚡' },
  { key: 'depth', label: 'Depth Lab', desc: 'Multi-step application problems.', icon: '◈' },
  { key: 'trapsense', label: 'TrapSense', desc: 'Distractor-heavy. Trap detection.', icon: '⚠' },
  { key: 'endurance', label: 'Endurance Track', desc: '30+ questions. Consistency under fatigue.', icon: '∞' },
  { key: 'perfect', label: 'Perfect Run', desc: 'One error breaks the streak.', icon: '★' },
  { key: 'apex', label: 'Apex Mock', desc: 'Full exam at elite difficulty.', icon: '◉' },
]

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, tc] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 5),
    ])
    truth.value = t
    topics.value = tc
  } catch {}
  loading.value = false
})
</script>

<template>
  <div class="min-h-screen" data-mode="elite">
    <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">

      <!-- Elite Header -->
      <div class="flex items-start justify-between mb-8">
        <div>
          <div class="flex items-center gap-2 mb-1">
            <AppBadge color="accent" size="sm">◆ {{ tier }} Tier</AppBadge>
            <AppBadge color="muted" size="xs">Streak: {{ streak }}</AppBadge>
          </div>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Elite Mode</h1>
          <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Push your performance limits within the syllabus.</p>
        </div>
        <ProgressRing :value="eps" :max="10000" :size="72" :stroke-width="4" color="var(--primary)" label="EPS" />
      </div>

      <!-- Identity Panel (6 Dimensions) -->
      <div class="grid grid-cols-6 gap-2 mb-8">
        <AppCard v-for="dim in identityPanel" :key="dim.label" padding="sm" class="text-center">
          <div class="text-xl mb-1">{{ dim.icon }}</div>
          <p class="font-display text-lg font-bold tabular-nums" :style="{ color: 'var(--primary)' }">{{ (dim.score / 100).toFixed(0) }}%</p>
          <p class="text-[9px] font-medium uppercase" :style="{ color: 'var(--text-3)' }">{{ dim.label }}</p>
        </AppCard>
      </div>

      <!-- Session Types -->
      <div class="mb-8">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Challenge Arena</h3>
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
          <AppCard v-for="session in sessionTypes" :key="session.key" hover padding="md"
            @click="router.push('/student/elite/arena')">
            <div class="flex items-start gap-3">
              <div class="w-9 h-9 rounded-lg flex items-center justify-center text-base"
                :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">
                {{ session.icon }}
              </div>
              <div>
                <p class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ session.label }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ session.desc }}</p>
              </div>
            </div>
          </AppCard>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="flex flex-wrap gap-2">
        <AppButton variant="secondary" size="sm" @click="router.push('/student/elite/records')">★ Records Wall</AppButton>
        <AppButton variant="secondary" size="sm" @click="router.push('/student/elite/insights')">◐ Insights</AppButton>
        <AppButton variant="ghost" size="sm" @click="router.push('/student')">← Back to Home</AppButton>
      </div>
    </div>
  </div>
</template>
