<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const router = useRouter()

const overview = ref({
  readiness: 4800, mastered: 24, weak: 10, unseen: 5, dueReviews: 7, streakDays: 12,
})

const topicStates = ref([
  { name: 'Number Operations', state: 'robust', score: 8200 },
  { name: 'Fractions & Decimals', state: 'fragile', score: 3900 },
  { name: 'Algebraic Expressions', state: 'emerging', score: 2800 },
  { name: 'Geometry & Shapes', state: 'stable', score: 6500 },
  { name: 'Measurement', state: 'partial', score: 4100 },
  { name: 'Statistics & Probability', state: 'exposed', score: 1500 },
  { name: 'Ratio & Proportion', state: 'fragile', score: 3200 },
  { name: 'Linear Equations', state: 'unseen', score: 0 },
])
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Progress</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Your academic readiness at a glance.</p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/mastery-map')">◐ Mastery Map</AppButton>
        <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/analytics')">📊 Analytics</AppButton>
      </div>
    </div>

    <div class="grid grid-cols-5 gap-3 mb-8">
      <AppCard padding="md" class="text-center">
        <ProgressRing :value="overview.readiness" :max="10000" :size="56" :stroke-width="3.5" color="var(--accent)" />
        <p class="text-[10px] font-medium mt-2 uppercase" :style="{ color: 'var(--text-3)' }">Readiness</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ overview.mastered }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Mastered</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ overview.weak }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Weak</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">{{ overview.streakDays }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Day Streak</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ overview.dueReviews }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Due Reviews</p>
      </AppCard>
    </div>

    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Topic Mastery</h3>
      <div class="space-y-2">
        <AppCard v-for="topic in topicStates" :key="topic.name" padding="sm" hover>
          <div class="flex items-center gap-3">
            <MasteryBadge :state="topic.state" size="sm" glow />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.name }}</p>
            </div>
            <div class="w-32">
              <AppProgress :value="topic.score" :max="10000" size="sm" :color="topic.score >= 6000 ? 'success' : topic.score >= 3000 ? 'gold' : 'danger'" />
            </div>
            <span class="text-xs tabular-nums font-medium w-10 text-right" :style="{ color: 'var(--text-3)' }">
              {{ (topic.score / 100).toFixed(0) }}%
            </span>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
