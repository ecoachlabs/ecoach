<script setup lang="ts">
import CoverageMatrix from '@/components/viz/CoverageMatrix.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  subjectName: string
  topics: string[]
  contentTypes: string[]
  coverage: number[][]
  overallScore: number
}>()
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-bold" :style="{color:'var(--text)'}">{{ subjectName }} Coverage</h3>
      <AppBadge :color="overallScore >= 80 ? 'success' : overallScore >= 50 ? 'gold' : 'danger'" size="sm">
        {{ overallScore }}% coverage
      </AppBadge>
    </div>
    <AppCard padding="md">
      <CoverageMatrix :topics="topics" :content-types="contentTypes" :coverage="coverage" />
    </AppCard>
  </div>
</template>
