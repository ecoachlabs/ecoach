<script setup lang="ts">
import { ref, computed } from 'vue'
import QuestionCard from '@/components/question/QuestionCard.vue'
import QuestionTimer from '@/components/question/QuestionTimer.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const props = defineProps<{
  questions: any[]
  totalTimeSeconds: number
  mockTitle?: string
}>()

const emit = defineEmits<{ submit: []; answer: [questionId: number, optionId: number, confidence: string, timeMs: number] }>()

const currentIndex = ref(0)
const answeredCount = ref(0)
const flaggedQuestions = ref<Set<number>>(new Set())

const currentQuestion = computed(() => props.questions[currentIndex.value])
const progress = computed(() => (answeredCount.value / props.questions.length) * 100)

// Pacing
const pacingStatus = computed(() => {
  const expectedPct = 50 // simplified
  if (progress.value >= expectedPct + 10) return { label: 'Ahead', color: 'var(--success)', icon: '▲' }
  if (progress.value <= expectedPct - 10) return { label: 'Behind', color: 'var(--danger)', icon: '▼' }
  return { label: 'On Pace', color: 'var(--accent)', icon: '●' }
})

function goTo(idx: number) {
  currentIndex.value = idx
}

function handleAnswer(optionId: number, confidence: string, timeMs: number) {
  if (!currentQuestion.value) return
  answeredCount.value++
  emit('answer', currentQuestion.value.id, optionId, confidence, timeMs)
}

function handleFlag(_flagged: boolean) {
  if (currentQuestion.value) {
    if (flaggedQuestions.value.has(currentIndex.value)) flaggedQuestions.value.delete(currentIndex.value)
    else flaggedQuestions.value.add(currentIndex.value)
  }
}
</script>

<template>
  <div class="h-full flex flex-col" data-mode="pressure">
    <!-- Exam Header -->
    <div class="shrink-0 px-4 py-2 flex items-center justify-between"
      :style="{ backgroundColor: 'var(--card-bg)' }">
      <div class="flex items-center gap-3">
        <AppBadge color="danger" size="sm">EXAM</AppBadge>
        <span class="text-xs font-medium" :style="{ color: 'var(--text)' }">{{ mockTitle || 'Mock Exam' }}</span>
      </div>
      <div class="flex items-center gap-4">
        <!-- Pacing -->
        <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full" :style="{ color: pacingStatus.color, backgroundColor: pacingStatus.color + '15' }">
          {{ pacingStatus.icon }} {{ pacingStatus.label }}
        </span>
        <!-- Timer -->
        <div class="w-32">
          <QuestionTimer :total-seconds="totalTimeSeconds" variant="strict" :running="true" />
        </div>
        <!-- Submit -->
        <AppButton variant="danger" size="sm" @click="$emit('submit')">Submit Exam</AppButton>
      </div>
    </div>

    <!-- Question Navigation Grid -->
    <div class="shrink-0 px-4 py-2 flex items-center gap-1 overflow-x-auto"
      :style="{ backgroundColor: 'var(--card-bg)' }">
      <button
        v-for="(q, i) in questions"
        :key="i"
        class="w-8 h-8 rounded text-[10px] font-bold shrink-0 transition-all"
        :class="[
          currentIndex === i ? 'ring-2 ring-[var(--accent)]' : '',
          flaggedQuestions.has(i) ? 'bg-amber-100 text-amber-700' :
          'bg-[var(--primary-light)] text-[var(--text-3)] hover:bg-[var(--accent-light)]',
        ]"
        @click="goTo(i)"
      >
        {{ i + 1 }}
      </button>
    </div>

    <!-- Question Area (distraction-free) -->
    <div class="flex-1 overflow-y-auto p-6 lg:p-8">
      <QuestionCard
        v-if="currentQuestion"
        :stem="currentQuestion.stem"
        :format="currentQuestion.format || 'mcq'"
        :options="currentQuestion.options || []"
        :question-number="currentIndex + 1"
        :total-questions="questions.length"
        @answer="handleAnswer"
        @flag="handleFlag"
      />
    </div>

    <!-- Bottom bar -->
    <div class="shrink-0 px-4 py-2 flex items-center justify-between"
      :style="{ backgroundColor: 'var(--card-bg)' }">
      <AppButton variant="ghost" size="sm" :disabled="currentIndex === 0" @click="currentIndex--">← Previous</AppButton>
      <span class="text-xs tabular-nums" :style="{ color: 'var(--text-3)' }">
        {{ answeredCount }}/{{ questions.length }} answered
      </span>
      <AppButton variant="ghost" size="sm" :disabled="currentIndex >= questions.length - 1" @click="currentIndex++">Next →</AppButton>
    </div>
  </div>
</template>
