<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import RadarChart from '@/components/viz/RadarChart.vue'

defineProps<{
  readiness: number
  readinessBand: string
  topicsAssessed: number
  strongCount: number
  developingCount: number
  weakCount: number
  dimensions?: { label: string; value: number }[]
}>()
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center gap-8">
      <ProgressRing :value="readiness" :max="10000" :size="100" :stroke-width="6"
        :color="readiness >= 7000 ? 'var(--success)' : readiness >= 4000 ? 'var(--gold)' : 'var(--danger)'" label="Readiness" />
      <div>
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--text)' }">{{ (readiness/100).toFixed(0) }}%</p>
        <p class="text-sm capitalize" :style="{ color: 'var(--text-2)' }">{{ readinessBand }}</p>
        <p class="text-xs mt-1" :style="{ color: 'var(--text-3)' }">{{ topicsAssessed }} topics assessed</p>
      </div>
    </div>
    <div class="grid grid-cols-3 gap-3">
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{color:'var(--success)'}">{{ strongCount }}</p><p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Strong</p></AppCard>
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{color:'var(--gold)'}">{{ developingCount }}</p><p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Developing</p></AppCard>
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{color:'var(--danger)'}">{{ weakCount }}</p><p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Weak</p></AppCard>
    </div>
    <div v-if="dimensions?.length" class="flex justify-center">
      <RadarChart :dimensions="dimensions" :size="220" />
    </div>
  </div>
</template>
