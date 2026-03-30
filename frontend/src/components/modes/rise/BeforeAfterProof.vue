<script setup lang="ts">
import ComparisonCard from '@/components/viz/ComparisonCard.vue'
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  topicName: string
  beforeScore: number
  afterScore: number
  beforeDate: string
  afterDate: string
  improvements: string[]
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
      Proof of Progress: {{ topicName }}
    </h3>

    <ComparisonCard leftLabel="Before" rightLabel="Now"
      :leftValue="(beforeScore / 100).toFixed(0) + '%'" :rightValue="(afterScore / 100).toFixed(0) + '%'"
      :leftSubtext="beforeDate" :rightSubtext="afterDate"
      highlight="right" />

    <div v-if="improvements.length" class="mt-3">
      <AppCard padding="sm">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" :style="{ color: 'var(--success)' }">What Changed</p>
        <ul class="space-y-1">
          <li v-for="imp in improvements" :key="imp" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
            <span class="text-emerald-500 mt-0.5">↑</span> {{ imp }}
          </li>
        </ul>
      </AppCard>
    </div>
  </div>
</template>
