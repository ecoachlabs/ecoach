<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

defineProps<{
  sessionType: string
  eps: number
  accuracy: number
  speed: number
  streak: number
  errorClusters: { type: string; count: number }[]
  coachInsight: string
  personalBest?: boolean
}>()

defineEmits<{ home: []; rematch: []; records: [] }>()
</script>

<template>
  <div class="max-w-2xl mx-auto reveal-stagger">
    <!-- Personal best banner -->
    <div v-if="personalBest" class="text-center mb-4 py-3 rounded-[var(--radius-lg)]"
      :style="{ backgroundColor: 'var(--gold-light)' }">
      <span class="text-sm font-bold" :style="{ color: 'var(--gold)' }">🏆 New Personal Best!</span>
    </div>

    <!-- Score Hero -->
    <AppCard padding="lg" glow="accent" class="mb-6">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <ProgressRing :value="eps" :max="10000" :size="72" :stroke-width="4" color="var(--primary)" label="EPS" />
          <div>
            <AppBadge color="accent" size="sm">{{ sessionType }}</AppBadge>
            <p class="font-display text-lg font-bold mt-1" :style="{ color: 'var(--text)' }">
              {{ (accuracy / 100).toFixed(0) }}% accuracy
            </p>
          </div>
        </div>
        <div class="text-right space-y-1">
          <p class="text-xs" :style="{ color: 'var(--text-3)' }">Speed: {{ (speed / 100).toFixed(0) }}%</p>
          <p class="text-xs" :style="{ color: 'var(--text-3)' }">Streak: {{ streak }}</p>
        </div>
      </div>
    </AppCard>

    <!-- Coach Insight (analytical, not congratulatory) -->
    <AppCard padding="md" class="mb-6">
      <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--accent)' }">Coach Analysis</p>
      <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ coachInsight }}</p>
    </AppCard>

    <!-- Error Clusters -->
    <div v-if="errorClusters.length" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Error Patterns</h3>
      <div class="flex flex-wrap gap-2">
        <AppBadge v-for="ec in errorClusters" :key="ec.type" color="warm" size="sm">
          {{ ec.type.replace(/_/g, ' ') }} ({{ ec.count }})
        </AppBadge>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-3">
      <AppButton variant="primary" @click="$emit('rematch')">Rematch →</AppButton>
      <AppButton variant="secondary" @click="$emit('records')">Records Wall</AppButton>
      <AppButton variant="ghost" @click="$emit('home')">Home</AppButton>
    </div>
  </div>
</template>
