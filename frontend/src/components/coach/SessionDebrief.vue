<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

defineProps<{
  accuracyScore: number // BasisPoints
  answeredQuestions: number
  correctQuestions: number
  sessionType?: string
  topicName?: string
  insights?: string[]
}>()

defineEmits<{ home: []; practice: []; review: [] }>()
</script>

<template>
  <div class="max-w-2xl mx-auto reveal-stagger">
    <!-- Score Hero -->
    <AppCard padding="lg" :glow="accuracyScore >= 7000 ? 'accent' : accuracyScore >= 4000 ? 'gold' : 'warm'" class="mb-6">
      <div class="flex items-center gap-6">
        <ProgressRing
          :value="accuracyScore"
          :max="10000"
          :size="80"
          :stroke-width="5"
          :color="accuracyScore >= 7000 ? 'var(--success)' : accuracyScore >= 4000 ? 'var(--gold)' : 'var(--danger)'"
          label="Accuracy"
        />
        <div>
          <h2 class="font-display text-xl font-semibold" :style="{ color: 'var(--text)' }">
            {{ correctQuestions }}/{{ answeredQuestions }} correct
          </h2>
          <p v-if="topicName" class="text-sm mt-0.5" :style="{ color: 'var(--text-2)' }">{{ topicName }}</p>
          <AppBadge v-if="sessionType" color="muted" size="xs" class="mt-1">{{ sessionType }}</AppBadge>
        </div>
      </div>
    </AppCard>

    <!-- Stats grid -->
    <div class="grid grid-cols-3 gap-3 mb-6">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ answeredQuestions }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Answered</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ correctQuestions }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Correct</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ answeredQuestions - correctQuestions }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Wrong</p>
      </AppCard>
    </div>

    <!-- Insights -->
    <div v-if="insights?.length" class="mb-6 space-y-2">
      <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">What Changed</h3>
      <AppCard v-for="(insight, i) in insights" :key="i" padding="sm">
        <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ insight }}</p>
      </AppCard>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-3">
      <AppButton variant="primary" @click="$emit('home')">Back to Home</AppButton>
      <AppButton variant="secondary" @click="$emit('practice')">Practice Again</AppButton>
      <AppButton v-if="answeredQuestions > correctQuestions" variant="ghost" size="sm" @click="$emit('review')">
        Review Mistakes
      </AppButton>
    </div>
  </div>
</template>
