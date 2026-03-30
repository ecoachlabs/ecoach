<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import BarChart from '@/components/viz/BarChart.vue'
import ComparisonCard from '@/components/viz/ComparisonCard.vue'

defineProps<{
  calmAccuracy: number
  pressureAccuracy: number
  avgSpeed: number
  speedUnderPressure: number
  guessedCount: number
  totalQuestions: number
}>()
</script>

<template>
  <div class="space-y-4">
    <ComparisonCard leftLabel="Calm" rightLabel="Under Pressure"
      :leftValue="(calmAccuracy/100).toFixed(0) + '%'" :rightValue="(pressureAccuracy/100).toFixed(0) + '%'"
      leftSubtext="Accuracy" rightSubtext="Accuracy"
      :highlight="calmAccuracy > pressureAccuracy ? 'left' : 'right'" />

    <div class="grid grid-cols-2 gap-3">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-xl font-bold" :style="{color:'var(--accent)'}">{{ avgSpeed }}s</p>
        <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Avg Response</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-xl font-bold" :style="{color: guessedCount > 3 ? 'var(--danger)' : 'var(--text-3)'}">{{ guessedCount }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Guessed</p>
      </AppCard>
    </div>

    <AppCard padding="md">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{color:'var(--text-3)'}">Pressure Delta</p>
      <p class="text-sm" :style="{color:'var(--text-2)'}">
        Your accuracy drops {{ ((calmAccuracy - pressureAccuracy) / 100).toFixed(0) }}% under timed conditions.
        {{ (calmAccuracy - pressureAccuracy) > 1500 ? 'This is significant. Pressure training is recommended.' : 'This is within normal range.' }}
      </p>
    </AppCard>
  </div>
</template>
