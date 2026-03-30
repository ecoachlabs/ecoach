<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import TrendLine from '@/components/viz/TrendLine.vue'
import MicroGainIndicator from './MicroGainIndicator.vue'

defineProps<{
  weekData: { day: string; attempted: number; correct: number; avgTime: number }[]
  weeklyGains: { volume: number; accuracy: number; speed: number }
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">This Week's Progress</h3>

    <!-- Micro gains summary -->
    <div class="flex items-center gap-3 mb-4">
      <MicroGainIndicator type="volume" :value="weeklyGains.volume" />
      <MicroGainIndicator type="accuracy" :value="weeklyGains.accuracy" />
      <MicroGainIndicator type="speed" :value="weeklyGains.speed" unit="s" />
    </div>

    <!-- Accuracy trend -->
    <AppCard padding="md" class="mb-3">
      <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--text-3)'}">Accuracy Trend</p>
      <TrendLine
        :data="weekData.map(d => ({ label: d.day, value: d.correct * 100 / Math.max(1, d.attempted) * 100 }))"
        :max-value="10000" :height="80" color="var(--accent)"
      />
    </AppCard>

    <!-- Daily breakdown -->
    <div class="space-y-1">
      <div v-for="d in weekData" :key="d.day" class="flex items-center gap-3 text-xs py-1">
        <span class="w-8 font-semibold" :style="{color:'var(--text)'}">{{ d.day }}</span>
        <span class="flex-1 tabular-nums" :style="{color:'var(--text-2)'}">{{ d.correct }}/{{ d.attempted }}</span>
        <span class="tabular-nums" :style="{color:'var(--text-3)'}">{{ d.avgTime }}s avg</span>
      </div>
    </div>
  </div>
</template>
