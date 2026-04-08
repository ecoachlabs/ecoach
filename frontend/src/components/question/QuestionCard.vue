<script setup lang="ts">
import { ref, computed } from 'vue'
import McqQuestion from './formats/McqQuestion.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const props = defineProps<{
  stem: string
  format: string
  options: { id: number; label: string; text: string; is_correct?: boolean; misconception_id?: number | null; distractor_intent?: string | null }[]
  difficulty?: number
  estimatedSeconds?: number
  showTimer?: boolean
  timerSeconds?: number
  questionNumber?: number
  totalQuestions?: number
}>()

const emit = defineEmits<{
  answer: [optionId: number, confidence: string, responseTimeMs: number]
  flag: []
  skip: []
}>()

const selectedOption = ref<number | null>(null)
const confidence = ref<'sure' | 'not_sure' | 'guessed' | null>(null)
const answered = ref(false)
const startTime = ref(Date.now())
const flagged = ref(false)

function selectOption(id: number) {
  if (answered.value) return
  selectedOption.value = id
}

function submitAnswer() {
  if (selectedOption.value === null || confidence.value === null) return
  answered.value = true
  emit('answer', selectedOption.value, confidence.value, Date.now() - startTime.value)
}

function reset() {
  selectedOption.value = null
  confidence.value = null
  answered.value = false
  startTime.value = Date.now()
  flagged.value = false
}

defineExpose({ reset })
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <!-- Question Header -->
    <div v-if="questionNumber" class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold" :style="{ color: 'var(--text-3)' }">
          Question {{ questionNumber }}<span v-if="totalQuestions"> of {{ totalQuestions }}</span>
        </span>
        <AppBadge v-if="difficulty" :color="difficulty > 7000 ? 'danger' : difficulty > 4000 ? 'warm' : 'success'" size="xs">
          {{ difficulty > 7000 ? 'Hard' : difficulty > 4000 ? 'Medium' : 'Easy' }}
        </AppBadge>
      </div>
      <button
        class="w-8 h-8 rounded-lg flex items-center justify-center transition-colors text-sm"
        :class="flagged ? 'bg-amber-100 text-amber-600' : 'text-[var(--text-3)] hover:bg-[var(--primary-light)]'"
        @click="flagged = !flagged; $emit('flag')"
        title="Flag for review"
      >
        {{ flagged ? '⚑' : '⚐' }}
      </button>
    </div>

    <!-- Question Stem -->
    <div
      class="p-5 rounded-[var(--radius-lg)] mb-5"
      :style="{ backgroundColor: 'var(--card-bg)' }"
    >
      <p class="text-base leading-relaxed font-medium" :style="{ color: 'var(--text)' }">
        {{ stem }}
      </p>
    </div>

    <!-- Answer Options -->
    <McqQuestion
      v-if="format === 'mcq' || !format"
      :options="options"
      :selected="selectedOption"
      :answered="answered"
      @select="selectOption"
    />

    <!-- Confidence Capture -->
    <div v-if="selectedOption !== null && !answered" class="mt-5">
      <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">
        How sure are you?
      </p>
      <div class="flex gap-2">
        <button
          v-for="c in [{ key: 'sure', label: 'Sure', icon: '✓' }, { key: 'not_sure', label: 'Not Sure', icon: '?' }, { key: 'guessed', label: 'Guessed', icon: '~' }]"
          :key="c.key"
          class="flex-1 py-2.5 rounded-[var(--radius-md)] text-sm font-medium border transition-all"
          :class="confidence === c.key
            ? 'bg-[var(--accent)] text-white border-[var(--accent)]'
            : 'border-[var(--card-border)] text-[var(--text-2)] hover:border-[var(--accent)]'"
          :style="{ backgroundColor: confidence === c.key ? undefined : 'var(--card-bg)', transitionDuration: 'var(--dur-fast)' }"
          @click="confidence = c.key as any"
        >
          {{ c.icon }} {{ c.label }}
        </button>
      </div>
    </div>

    <!-- Submit -->
    <div v-if="selectedOption !== null && confidence !== null && !answered" class="mt-4">
      <AppButton variant="primary" size="lg" class="w-full" @click="submitAnswer">
        Submit Answer
      </AppButton>
    </div>
  </div>
</template>
