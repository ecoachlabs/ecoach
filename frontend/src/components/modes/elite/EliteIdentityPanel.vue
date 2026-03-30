<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  dimensions: { label: string; score: number; icon: string; trend?: number }[]
}>()

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}
</script>

<template>
  <div class="grid grid-cols-6 gap-2">
    <AppCard v-for="dim in dimensions" :key="dim.label" padding="sm" class="text-center">
      <div class="text-xl mb-1">{{ dim.icon }}</div>
      <p class="font-display text-lg font-bold tabular-nums" :style="{ color: 'var(--primary)' }">
        {{ formatBp(dim.score) }}
      </p>
      <p class="text-[9px] font-medium uppercase" :style="{ color: 'var(--text-3)' }">{{ dim.label }}</p>
      <span v-if="dim.trend !== undefined" class="text-[9px] font-semibold"
        :class="dim.trend >= 0 ? 'text-emerald-600' : 'text-red-500'">
        {{ dim.trend >= 0 ? '↑' : '↓' }}{{ Math.abs(dim.trend) }}
      </span>
    </AppCard>
  </div>
</template>
