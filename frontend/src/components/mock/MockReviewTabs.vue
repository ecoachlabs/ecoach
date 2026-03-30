<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppTabs from '@/components/ui/AppTabs.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

defineProps<{
  score: number
  answeredQuestions: number
  correctQuestions: number
  topicScores: { topicName: string; score: number; correct: number; total: number }[]
}>()

const activeTab = ref('overview')
const tabs = [
  { key: 'overview', label: 'Overview' },
  { key: 'strengths', label: 'Strengths & Weaknesses' },
  { key: 'questions', label: 'Question Review' },
  { key: 'timing', label: 'Timing' },
  { key: 'next', label: 'Next Steps' },
]

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}
</script>

<template>
  <div>
    <!-- Score Hero -->
    <AppCard padding="lg" class="mb-6" :glow="score >= 7000 ? 'accent' : score >= 4000 ? 'gold' : 'warm'">
      <div class="flex items-center gap-6">
        <ProgressRing :value="score" :max="10000" :size="88" :stroke-width="5"
          :color="score >= 7000 ? 'var(--success)' : score >= 4000 ? 'var(--gold)' : 'var(--danger)'"
          label="Score" />
        <div>
          <h2 class="font-display text-2xl font-bold" :style="{ color: 'var(--text)' }">
            {{ correctQuestions }}/{{ answeredQuestions }}
          </h2>
          <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ formatBp(score) }} accuracy</p>
        </div>
      </div>
    </AppCard>

    <AppTabs :tabs="tabs" v-model="activeTab" class="mb-6" />

    <!-- Overview -->
    <div v-if="activeTab === 'overview'" class="grid grid-cols-3 gap-3">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ correctQuestions }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Correct</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ answeredQuestions - correctQuestions }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Wrong</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">{{ formatBp(score) }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Score</p>
      </AppCard>
    </div>

    <!-- Strengths & Weaknesses -->
    <div v-if="activeTab === 'strengths'" class="space-y-2">
      <AppCard v-for="topic in topicScores" :key="topic.topicName" padding="sm">
        <div class="flex items-center gap-3">
          <div class="flex-1">
            <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ topic.topicName }}</p>
            <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ topic.correct }}/{{ topic.total }} correct</p>
          </div>
          <div class="w-24">
            <AppProgress :value="topic.score" :max="10000" size="sm"
              :color="topic.score >= 7000 ? 'success' : topic.score >= 4000 ? 'gold' : 'danger'" />
          </div>
        </div>
      </AppCard>
    </div>

    <!-- Question Review placeholder -->
    <div v-if="activeTab === 'questions'">
      <AppCard padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">Per-question review with multi-lens filtering (by order, wrong answers, weak topics, confidence mismatch, timing, traps) renders here.</p>
      </AppCard>
    </div>

    <!-- Timing placeholder -->
    <div v-if="activeTab === 'timing'">
      <AppCard padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">Time per question chart, pacing analysis, and speed patterns render here.</p>
      </AppCard>
    </div>

    <!-- Next Steps -->
    <div v-if="activeTab === 'next'">
      <AppCard padding="md">
        <p class="text-sm" :style="{ color: 'var(--text-2)' }">Based on your performance, your coach recommends focusing on weak topics identified above.</p>
      </AppCard>
    </div>
  </div>
</template>
