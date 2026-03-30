<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import { getMemoryDashboard, getReviewQueue } from '@/ipc/memory'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppCard from '@/components/ui/AppCard.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MemoryHeatmap from '@/components/modes/memory/MemoryHeatmap.vue'
import RecoveryLadder from '@/components/modes/memory/RecoveryLadder.vue'
import MemoryConfirmationGates from '@/components/modes/memory/MemoryConfirmationGates.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const memoryDash = ref<any>(null)
const reviewQueue = ref<any[]>([])

const memoryHealth = ref(6800)
const fadingCount = ref(5)
const recoveredToday = ref(2)

const heatmapTopics = ref([
  { name: 'Fractions', strength: 3200, status: 'fading' as const },
  { name: 'Algebra', strength: 5400, status: 'vulnerable' as const },
  { name: 'Geometry', strength: 8200, status: 'strong' as const },
  { name: 'Ratios', strength: 2100, status: 'critical' as const },
  { name: 'Statistics', strength: 6800, status: 'strong' as const },
  { name: 'Number Ops', strength: 9100, status: 'strong' as const },
  { name: 'Decimals', strength: 4500, status: 'vulnerable' as const },
  { name: 'Measurement', strength: 7200, status: 'recovered' as const },
])

const sessionTypes = [
  { key: 'scan', label: 'Memory Scan', desc: 'Quick check of what is fading', icon: '◎' },
  { key: 'rescue', label: 'Rescue Burst', desc: 'Rapid repair of critical memories', icon: '⚡' },
  { key: 'deep', label: 'Deep Repair', desc: 'Intensive reconstruction', icon: '◈' },
  { key: 'recall', label: 'Recall Builder', desc: 'Progressive strengthening', icon: '△' },
  { key: 'chain', label: 'Chain Repair', desc: 'Fix prerequisite chains', icon: '⟶' },
  { key: 'rapid', label: 'Rapid Drill', desc: 'Speed-focused recall', icon: '⏱' },
]

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, md, rq] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getMemoryDashboard(auth.currentAccount.id).catch(() => null),
      getReviewQueue(auth.currentAccount.id).catch(() => []),
    ])
    truth.value = t
    memoryDash.value = md
    reviewQueue.value = rq
  } catch {}
  loading.value = false
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <PageHeader title="Memory Mode" subtitle="Strengthen what you know. Catch fading knowledge before it affects your score." back-to="/student">
      <template #actions>
        <ProgressRing :value="memoryHealth" :max="10000" :size="56" :stroke-width="4" color="var(--success)" label="Health" />
      </template>
    </PageHeader>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else>
      <!-- Stats -->
      <div class="grid grid-cols-3 gap-3 mb-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--warning)'}">{{ fadingCount }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Fading</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--accent)'}">{{ truth?.due_memory_count ?? reviewQueue.length }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Due Recheck</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--success)'}">{{ recoveredToday }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Recovered</p>
        </AppCard>
      </div>

      <!-- Memory Heatmap -->
      <div class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Memory Health Map</h3>
        <AppCard padding="md">
          <MemoryHeatmap :topics="heatmapTopics" />
        </AppCard>
      </div>

      <!-- Session Types -->
      <div class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Session Types</h3>
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
          <AppCard v-for="st in sessionTypes" :key="st.key" hover padding="md">
            <div class="flex items-start gap-2.5">
              <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm" :style="{backgroundColor:'var(--primary-light)',color:'var(--primary)'}">{{ st.icon }}</div>
              <div>
                <p class="text-xs font-semibold" :style="{color:'var(--text)'}">{{ st.label }}</p>
                <p class="text-[10px]" :style="{color:'var(--text-3)'}">{{ st.desc }}</p>
              </div>
            </div>
          </AppCard>
        </div>
      </div>

      <!-- Recovery Ladder preview -->
      <div class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Recovery Stages</h3>
        <AppCard padding="md">
          <RecoveryLadder :current-stage="2" />
        </AppCard>
      </div>

      <div class="flex gap-3">
        <AppButton variant="primary">Quick Scan →</AppButton>
        <AppButton v-if="fadingCount > 0" variant="warm">Rescue {{ fadingCount }} Fading</AppButton>
      </div>
    </template>
  </div>
</template>
