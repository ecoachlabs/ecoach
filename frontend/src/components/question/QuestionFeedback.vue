<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  isCorrect: boolean
  explanation?: string | null
  errorType?: string | null
  diagnosisSummary?: string | null
  recommendedAction?: string | null
  selectedOptionText: string
  correctOptionText?: string | null
  misconceptionInfo?: string | null
}>()

defineEmits<{ next: []; review: [] }>()

const errorLabels: Record<string, string> = {
  knowledge_gap: 'Knowledge Gap',
  conceptual_confusion: 'Conceptual Confusion',
  recognition_failure: 'Recognition Failure',
  execution_error: 'Execution Error',
  carelessness: 'Careless Error',
  pressure_breakdown: 'Pressure Breakdown',
  expression_weakness: 'Expression Weakness',
  speed_error: 'Speed Error',
  guessing_detected: 'Guessing Detected',
  misconception_triggered: 'Misconception Triggered',
}
</script>

<template>
  <div class="reveal">
    <!-- Result banner -->
    <div
      class="px-5 py-4 rounded-t-[var(--radius-lg)] flex items-center gap-3"
      :class="isCorrect ? 'bg-emerald-50' : 'bg-amber-50'"
    >
      <div
        class="w-10 h-10 rounded-xl flex items-center justify-center text-lg font-bold text-white"
        :class="isCorrect ? 'bg-emerald-500' : 'bg-amber-500'"
      >
        {{ isCorrect ? '✓' : '✕' }}
      </div>
      <div>
        <p class="text-sm font-semibold" :style="{ color: isCorrect ? 'var(--success)' : 'var(--warning)' }">
          {{ isCorrect ? 'Correct!' : 'Not quite right' }}
        </p>
        <p v-if="!isCorrect && errorType" class="text-xs" :style="{ color: 'var(--text-3)' }">
          {{ errorLabels[errorType] || errorType }}
        </p>
      </div>
    </div>

    <!-- Detail card -->
    <AppCard padding="none" class="rounded-t-none border-t-0">
      <div class="p-5 space-y-4">
        <!-- Diagnosis -->
        <div v-if="diagnosisSummary && !isCorrect">
          <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--text-3)' }">What happened</p>
          <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ diagnosisSummary }}</p>
        </div>

        <!-- Explanation -->
        <div v-if="explanation">
          <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--text-3)' }">Explanation</p>
          <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ explanation }}</p>
        </div>

        <!-- Correct answer (if wrong) -->
        <div v-if="!isCorrect && correctOptionText">
          <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--success)' }">Correct answer</p>
          <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ correctOptionText }}</p>
        </div>

        <!-- Misconception -->
        <div v-if="misconceptionInfo && !isCorrect">
          <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--warm)' }">Watch out</p>
          <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ misconceptionInfo }}</p>
        </div>

        <!-- Recommended action -->
        <div v-if="recommendedAction && !isCorrect">
          <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--accent)' }">Next step</p>
          <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ recommendedAction }}</p>
        </div>
      </div>

      <!-- Actions -->
      <div class="px-5 pb-5 flex items-center gap-2">
        <AppButton variant="primary" @click="$emit('next')">Next Question →</AppButton>
        <AppButton v-if="!isCorrect" variant="ghost" size="sm" @click="$emit('review')">Review in Detail</AppButton>
      </div>
    </AppCard>
  </div>
</template>
