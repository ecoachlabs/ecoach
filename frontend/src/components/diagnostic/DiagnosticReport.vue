<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppTabs from '@/components/ui/AppTabs.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import { ref } from 'vue'

defineProps<{
  overallReadiness: number
  readinessBand: string
  topicResults: {
    topic_id: number
    topic_name: string
    mastery_score: number
    fluency_score: number
    precision_score: number
    pressure_score: number
    classification: string
  }[]
  recommendedActions: string[]
}>()

defineEmits<{ close: []; export: [] }>()

const activeTab = ref('overview')
const tabs = [
  { key: 'overview', label: 'Overview' },
  { key: 'topics', label: 'Topic Breakdown' },
  { key: 'behavior', label: 'Exam Behavior' },
  { key: 'actions', label: 'Next Steps' },
]

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}
</script>

<template>
  <div class="reveal-stagger">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Diagnostic Report</h1>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="$emit('export')">Export PDF</AppButton>
        <AppButton variant="ghost" size="sm" @click="$emit('close')">Close</AppButton>
      </div>
    </div>

    <!-- Readiness Hero -->
    <AppCard padding="lg" glow="accent" class="mb-6">
      <div class="flex items-center gap-8">
        <ProgressRing :value="overallReadiness" :max="10000" :size="100" :stroke-width="6"
          :color="overallReadiness >= 7000 ? 'var(--success)' : overallReadiness >= 4000 ? 'var(--gold)' : 'var(--danger)'"
          label="Readiness" />
        <div>
          <h2 class="font-display text-xl font-semibold mb-1" :style="{ color: 'var(--text)' }">
            Overall Readiness: {{ formatBp(overallReadiness) }}
          </h2>
          <AppBadge :color="readinessBand === 'strong' ? 'success' : readinessBand === 'developing' ? 'gold' : 'danger'" size="md">
            {{ readinessBand }}
          </AppBadge>
        </div>
      </div>
    </AppCard>

    <!-- Tabs -->
    <AppTabs :tabs="tabs" v-model="activeTab" class="mb-6" />

    <!-- Overview -->
    <div v-if="activeTab === 'overview'" class="grid grid-cols-4 gap-3">
      <AppCard v-for="s in [
        { l: 'Topics Assessed', v: topicResults.length, c: 'var(--accent)' },
        { l: 'Strong', v: topicResults.filter(t => t.mastery_score >= 6000).length, c: 'var(--success)' },
        { l: 'Developing', v: topicResults.filter(t => t.mastery_score >= 3000 && t.mastery_score < 6000).length, c: 'var(--gold)' },
        { l: 'Weak', v: topicResults.filter(t => t.mastery_score < 3000).length, c: 'var(--danger)' },
      ]" :key="s.l" padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: s.c }">{{ s.v }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">{{ s.l }}</p>
      </AppCard>
    </div>

    <!-- Topic Breakdown -->
    <div v-if="activeTab === 'topics'" class="space-y-2">
      <AppCard v-for="topic in topicResults" :key="topic.topic_id" padding="sm">
        <div class="flex items-center gap-3">
          <MasteryBadge :state="topic.classification" size="sm" glow />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
            <div class="flex gap-3 mt-1 text-[10px]" :style="{ color: 'var(--text-3)' }">
              <span>Mastery: {{ formatBp(topic.mastery_score) }}</span>
              <span>Fluency: {{ formatBp(topic.fluency_score) }}</span>
              <span>Precision: {{ formatBp(topic.precision_score) }}</span>
              <span>Pressure: {{ formatBp(topic.pressure_score) }}</span>
            </div>
          </div>
          <div class="w-24">
            <AppProgress :value="topic.mastery_score" :max="10000" size="sm"
              :color="topic.mastery_score >= 6000 ? 'success' : topic.mastery_score >= 3000 ? 'gold' : 'danger'" />
          </div>
        </div>
      </AppCard>
    </div>

    <!-- Exam Behavior -->
    <div v-if="activeTab === 'behavior'">
      <AppCard padding="lg">
        <p class="text-sm" :style="{ color: 'var(--text-2)' }">
          Exam behavior analysis including speed patterns, pressure response, and confidence calibration will be displayed here when diagnostic phase data is available.
        </p>
      </AppCard>
    </div>

    <!-- Next Steps -->
    <div v-if="activeTab === 'actions'" class="space-y-2">
      <AppCard v-for="(action, i) in recommendedActions" :key="i" padding="md">
        <div class="flex items-start gap-3">
          <div class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">{{ i + 1 }}</div>
          <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ action }}</p>
        </div>
      </AppCard>
    </div>
  </div>
</template>
