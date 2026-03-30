<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import BarChart from '@/components/viz/BarChart.vue'

defineProps<{
  metrics: { label: string; value: number; target: number }[]
  contentHealth: { label: string; value: number; color?: string }[]
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="grid grid-cols-3 gap-3">
      <AppCard v-for="m in metrics" :key="m.label" padding="md">
        <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--text-3)'}">{{ m.label }}</p>
        <p class="font-display text-xl font-bold tabular-nums mb-1"
          :style="{color: m.value >= m.target ? 'var(--success)' : m.value >= m.target * 0.7 ? 'var(--gold)' : 'var(--danger)'}">
          {{ m.value }}%
        </p>
        <AppProgress :value="m.value" size="sm" :color="m.value >= m.target ? 'success' : m.value >= m.target * 0.7 ? 'gold' : 'danger'" />
        <p class="text-[8px] mt-1" :style="{color:'var(--text-3)'}">Target: {{ m.target }}%</p>
      </AppCard>
    </div>

    <AppCard padding="md">
      <p class="text-[10px] font-semibold uppercase mb-3" :style="{color:'var(--text-3)'}">Content Health</p>
      <BarChart :data="contentHealth" :height="20" show-values />
    </AppCard>
  </div>
</template>
