<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

defineProps<{
  topics: { name: string; mastery: number; fluency: number; precision: number; pressure: number; classification: string }[]
}>()
</script>

<template>
  <div class="space-y-2">
    <AppCard v-for="t in topics" :key="t.name" padding="sm">
      <div class="flex items-center gap-3 mb-2">
        <MasteryBadge :state="t.classification" size="sm" glow />
        <p class="text-sm font-medium flex-1 truncate" :style="{color:'var(--text)'}">{{ t.name }}</p>
        <span class="text-xs tabular-nums font-medium" :style="{color:'var(--text-3)'}">{{ (t.mastery/100).toFixed(0) }}%</span>
      </div>
      <div class="grid grid-cols-4 gap-2">
        <div v-for="d in [{l:'Mastery',v:t.mastery},{l:'Fluency',v:t.fluency},{l:'Precision',v:t.precision},{l:'Pressure',v:t.pressure}]" :key="d.l">
          <p class="text-[8px] uppercase mb-0.5" :style="{color:'var(--text-3)'}">{{ d.l }}</p>
          <AppProgress :value="d.v" :max="10000" size="sm" :color="d.v >= 6000 ? 'success' : d.v >= 3000 ? 'gold' : 'danger'" />
        </div>
      </div>
    </AppCard>
  </div>
</template>
