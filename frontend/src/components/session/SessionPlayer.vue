<script setup lang="ts">
import { ref, computed } from 'vue'
import QuestionCard from '@/components/question/QuestionCard.vue'
import QuestionFeedback from '@/components/question/QuestionFeedback.vue'
import QuestionTimer from '@/components/question/QuestionTimer.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const props = defineProps<{
  questions: {
    id: number
    stem: string
    format: string
    options: { id: number; label: string; text: string; is_correct?: boolean; misconception_id?: number | null; distractor_intent?: string | null }[]
    difficulty?: number
    estimated_time_seconds?: number
  }[]
  sessionType?: string
  isTimed?: boolean
  timerSeconds?: number
}>()

const emit = defineEmits<{
  answer: [questionId: number, optionId: number, confidence: string, responseTimeMs: number]
  complete: []
}>()

const currentIndex = ref(0)
const feedbackResult = ref<any>(null)
const showFeedback = ref(false)

const currentQuestion = computed(() => props.questions[currentIndex.value] ?? null)
const progress = computed(() => ((currentIndex.value) / props.questions.length) * 100)
const isLastQuestion = computed(() => currentIndex.value >= props.questions.length - 1)

function handleAnswer(optionId: number, confidence: string, responseTimeMs: number) {
  if (!currentQuestion.value) return
  emit('answer', currentQuestion.value.id, optionId, confidence, responseTimeMs)

  // Show feedback (in real app, this would come from the IPC result)
  const selectedOpt = currentQuestion.value.options.find(o => o.id === optionId)
  const correctOpt = currentQuestion.value.options.find(o => o.is_correct)

  feedbackResult.value = {
    isCorrect: selectedOpt?.is_correct ?? false,
    selectedOptionText: selectedOpt?.text ?? '',
    correctOptionText: correctOpt?.text,
    explanation: null,
    errorType: selectedOpt?.is_correct ? null : 'knowledge_gap',
    diagnosisSummary: selectedOpt?.is_correct ? null : 'Review this concept to strengthen your understanding.',
    recommendedAction: selectedOpt?.is_correct ? null : 'Practice similar questions.',
    misconceptionInfo: selectedOpt?.misconception_id ? 'Watch for this common misconception.' : null,
  }
  showFeedback.value = true
}

function nextQuestion() {
  showFeedback.value = false
  feedbackResult.value = null
  if (isLastQuestion.value) {
    emit('complete')
  } else {
    currentIndex.value++
  }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Session header -->
    <div class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }">
      <div class="flex items-center gap-3">
        <AppBadge color="accent" size="sm">{{ sessionType || 'Practice' }}</AppBadge>
        <span class="text-xs tabular-nums" :style="{ color: 'var(--text-3)' }">
          {{ currentIndex + 1 }} / {{ questions.length }}
        </span>
      </div>
      <div class="w-40">
        <AppProgress :value="progress" size="sm" color="accent" />
      </div>
    </div>

    <!-- Timer -->
    <div v-if="isTimed && timerSeconds" class="px-6 pt-3">
      <QuestionTimer :total-seconds="timerSeconds" :running="!showFeedback" @timeout="nextQuestion" />
    </div>

    <!-- Content area -->
    <div class="flex-1 overflow-y-auto p-6 lg:p-8">
      <!-- Question -->
      <div v-if="currentQuestion && !showFeedback">
        <QuestionCard
          :stem="currentQuestion.stem"
          :format="currentQuestion.format"
          :options="currentQuestion.options"
          :difficulty="currentQuestion.difficulty"
          :question-number="currentIndex + 1"
          :total-questions="questions.length"
          @answer="handleAnswer"
        />
      </div>

      <!-- Feedback -->
      <div v-else-if="showFeedback && feedbackResult" class="max-w-2xl mx-auto">
        <QuestionFeedback
          v-bind="feedbackResult"
          @next="nextQuestion"
        />
      </div>

      <!-- No questions -->
      <div v-else class="flex items-center justify-center h-full">
        <div class="text-center">
          <p class="text-sm" :style="{ color: 'var(--text-3)' }">No questions available.</p>
          <AppButton variant="secondary" size="sm" class="mt-4" @click="$emit('complete')">End Session</AppButton>
        </div>
      </div>
    </div>
  </div>
</template>
