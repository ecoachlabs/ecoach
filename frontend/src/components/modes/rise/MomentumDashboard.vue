<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import TrendLine from '@/components/viz/TrendLine.vue'
import StageIndicator from './StageIndicator.vue'
import BeforeAfterProof from './BeforeAfterProof.vue'

defineProps<{
  currentStage: string
  scores: { label: string; value: number }[]
  trendData: { label: string; value: number }[]
  topicName?: string
  beforeScore?: number
  afterScore?: number
}>()
</script>

<template>
  <div class="space-y-6">
    <StageIndicator :current-stage="currentStage" />

    <!-- 8 scoring dimensions -->
    <div class="grid grid-cols-4 gap-2">
      <AppCard v-for="s in scores" :key="s.label" padding="sm" class="text-center">
        <p class="font-display text-lg font-bold tabular-nums" :style="{color:'var(--accent)'}">{{ (s.value / 100).toFixed(0) }}%</p>
        <p class="text-[8px] uppercase" :style="{color:'var(--text-3)'}">{{ s.label }}</p>
      </AppCard>
    </div>

    <!-- Trend -->
    <AppCard padding="md">
      <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--text-3)'}">Progress Trend</p>
      <TrendLine :data="trendData" :max-value="10000" :height="100" color="var(--warm)" />
    </AppCard>

    <!-- Before/After -->
    <BeforeAfterProof v-if="topicName && beforeScore !== undefined && afterScore !== undefined"
      :topic-name="topicName" :before-score="beforeScore" :after-score="afterScore"
      before-date="Start" after-date="Now" :improvements="['Accuracy improved', 'Speed increased']" />
  </div>
</template>
