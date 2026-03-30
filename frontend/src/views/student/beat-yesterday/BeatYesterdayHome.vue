<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)

// Simulated daily data until beat_yesterday backend commands are wired
const yesterday = ref({ attempted: 12, correct: 8, avgTime: 34 })
const today = ref({ attempted: 0, correct: 0, avgTime: 0 })
const targets = ref({ attempted: 14, correct: 10, avgTime: 31 })
const streak = ref(12)
const growthMode = ref<'volume' | 'accuracy' | 'speed' | 'mixed' | 'recovery'>('accuracy')

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    truth.value = await getLearnerTruth(auth.currentAccount.id)
  } catch {}
  loading.value = false
})

const blocks = [
  { name: 'Warm Start', duration: '2 min', desc: 'Easy wins to get you going', color: 'var(--success)', icon: '☀' },
  { name: 'Core Climb', duration: '5 min', desc: 'Push beyond yesterday', color: 'var(--accent)', icon: '△' },
  { name: 'Speed Burst', duration: '1 min', desc: '60-second rapid fire', color: 'var(--warm)', icon: '⚡' },
  { name: 'Finish Strong', duration: '1 min', desc: 'End on a high note', color: 'var(--gold)', icon: '★' },
]

const modeLabels: Record<string, string> = {
  volume: 'Volume Day — attempt more questions',
  accuracy: 'Accuracy Day — get more right',
  speed: 'Speed Day — answer faster',
  mixed: 'Mixed Day — all dimensions',
  recovery: 'Recovery Day — gentle rebuild',
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <!-- Header -->
    <div class="flex items-start justify-between mb-8">
      <div>
        <p class="text-sm font-medium mb-1" :style="{ color: 'var(--gold)' }">Day {{ streak }} streak</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Beat Yesterday</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Small daily gains compound into transformation.</p>
      </div>
      <div class="text-right">
        <div class="text-3xl font-display font-bold" :style="{ color: 'var(--gold)' }">🔥 {{ streak }}</div>
        <p class="text-[10px] uppercase font-medium" :style="{ color: 'var(--text-3)' }">Day Streak</p>
      </div>
    </div>

    <!-- Yesterday vs Today Hero Card -->
    <AppCard padding="lg" glow="gold" class="mb-6">
      <div class="grid grid-cols-3 gap-6">
        <!-- Yesterday -->
        <div class="text-center">
          <p class="text-[10px] font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Yesterday</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--text-2)' }">{{ yesterday.correct }}/{{ yesterday.attempted }}</p>
          <p class="text-xs mt-1" :style="{ color: 'var(--text-3)' }">{{ yesterday.avgTime }}s avg</p>
        </div>
        <!-- Target (center) -->
        <div class="text-center border-x px-6" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="text-[10px] font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--gold)' }">Today's Target</p>
          <p class="font-display text-3xl font-bold" :style="{ color: 'var(--gold)' }">{{ targets.correct }}/{{ targets.attempted }}</p>
          <p class="text-xs mt-1" :style="{ color: 'var(--gold)' }">{{ targets.avgTime }}s avg</p>
        </div>
        <!-- Today (so far) -->
        <div class="text-center">
          <p class="text-[10px] font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Today So Far</p>
          <p class="font-display text-3xl font-bold" :style="{ color: today.attempted > 0 ? 'var(--accent)' : 'var(--text-3)' }">
            {{ today.correct }}/{{ today.attempted }}
          </p>
          <p class="text-xs mt-1" :style="{ color: 'var(--text-3)' }">{{ today.attempted > 0 ? today.avgTime + 's avg' : 'Not started' }}</p>
        </div>
      </div>

      <!-- Growth Mode Badge -->
      <div class="mt-5 pt-4 border-t text-center" :style="{ borderColor: 'var(--border-soft)' }">
        <AppBadge color="gold" size="md">{{ modeLabels[growthMode] }}</AppBadge>
      </div>
    </AppCard>

    <!-- Session Blocks -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Today's Climb</h3>
      <div class="grid grid-cols-4 gap-3">
        <AppCard v-for="block in blocks" :key="block.name" padding="md" class="text-center">
          <div class="text-2xl mb-2">{{ block.icon }}</div>
          <p class="text-xs font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ block.name }}</p>
          <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ block.duration }}</p>
          <p class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">{{ block.desc }}</p>
        </AppCard>
      </div>
    </div>

    <!-- CTA -->
    <div class="text-center">
      <AppButton variant="warm" size="lg" @click="router.push('/student/session/beat-yesterday')">
        Start Today's Climb →
      </AppButton>
      <p class="text-xs mt-3" :style="{ color: 'var(--text-3)' }">~9 minutes · {{ targets.attempted }} questions</p>
    </div>
  </div>
</template>
