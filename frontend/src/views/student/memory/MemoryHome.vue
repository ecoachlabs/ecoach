<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    truth.value = await getLearnerTruth(auth.currentAccount.id)
  } catch {}
  loading.value = false
})

const memoryHealth = ref(6800) // BasisPoints
const fadingCount = ref(5)
const dueRechecks = ref(truth.value?.due_memory_count ?? 3)
const recoveredToday = ref(2)

const sessionTypes = [
  { key: 'scan', label: 'Memory Scan', desc: 'Quick assessment of what is fading', icon: '◎', color: 'accent' },
  { key: 'rescue', label: 'Rescue Burst', desc: 'Rapid repair of critical memories', icon: '⚡', color: 'danger' },
  { key: 'deep', label: 'Deep Repair', desc: 'Intensive reconstruction', icon: '◈', color: 'warm' },
  { key: 'recall', label: 'Recall Builder', desc: 'Progressive recall strengthening', icon: '△', color: 'gold' },
  { key: 'chain', label: 'Chain Repair', desc: 'Fix prerequisite chains', icon: '⟶', color: 'accent' },
  { key: 'rapid', label: 'Rapid Drill', desc: 'Speed-focused recall', icon: '⏱', color: 'warm' },
]
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <!-- Visual: warm/restorative feel -->
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Memory Mode</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Strengthen what you already know. Catch fading knowledge before it affects your score.</p>
      </div>
      <ProgressRing :value="memoryHealth" :max="10000" :size="72" :stroke-width="5" color="var(--success)" label="Health" />
    </div>

    <!-- Memory Stats -->
    <div class="grid grid-cols-3 gap-3 mb-8">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--warning)' }">{{ fadingCount }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Fading</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ truth?.due_memory_count ?? dueRechecks }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Due Recheck</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ recoveredToday }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Recovered</p>
      </AppCard>
    </div>

    <!-- Session Types -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Session Types</h3>
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        <AppCard v-for="st in sessionTypes" :key="st.key" hover padding="md">
          <div class="flex items-start gap-2.5">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm"
              :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">{{ st.icon }}</div>
            <div>
              <p class="text-xs font-semibold" :style="{ color: 'var(--text)' }">{{ st.label }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ st.desc }}</p>
            </div>
          </div>
        </AppCard>
      </div>
    </div>

    <div class="flex gap-3">
      <AppButton variant="primary">Quick Scan →</AppButton>
      <AppButton variant="secondary" v-if="fadingCount > 0">Rescue {{ fadingCount }} Fading</AppButton>
    </div>
  </div>
</template>
