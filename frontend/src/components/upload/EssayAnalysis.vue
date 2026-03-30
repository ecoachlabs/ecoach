<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  questionText: string
  studentResponse: string
  structureScore: number
  contentScore: number
  languageScore: number
  overallScore: number
  feedback: string[]
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Essay Analysis</h3>

    <!-- Question -->
    <AppCard padding="sm" class="mb-3">
      <p class="text-xs font-semibold mb-1" :style="{ color: 'var(--text-3)' }">Question</p>
      <p class="text-sm" :style="{ color: 'var(--text)' }">{{ questionText }}</p>
    </AppCard>

    <!-- Student response -->
    <AppCard padding="sm" class="mb-4">
      <p class="text-xs font-semibold mb-1" :style="{ color: 'var(--text-3)' }">Student's Response</p>
      <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ studentResponse }}</p>
    </AppCard>

    <!-- Scores -->
    <div class="grid grid-cols-4 gap-3 mb-4">
      <AppCard v-for="s in [
        { l: 'Overall', v: overallScore },
        { l: 'Structure', v: structureScore },
        { l: 'Content', v: contentScore },
        { l: 'Language', v: languageScore },
      ]" :key="s.l" padding="sm" class="text-center">
        <p class="font-display text-lg font-bold tabular-nums"
          :style="{ color: s.v >= 70 ? 'var(--success)' : s.v >= 40 ? 'var(--gold)' : 'var(--danger)' }">{{ s.v }}%</p>
        <p class="text-[9px] uppercase mt-0.5" :style="{ color: 'var(--text-3)' }">{{ s.l }}</p>
      </AppCard>
    </div>

    <!-- Feedback -->
    <div v-if="feedback.length">
      <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--accent)' }">Feedback</p>
      <ul class="space-y-1.5">
        <li v-for="f in feedback" :key="f" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
          <span :style="{ color: 'var(--accent)' }">→</span> {{ f }}
        </li>
      </ul>
    </div>
  </div>
</template>
