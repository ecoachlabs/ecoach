<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listMockQuestions, getMockSession, submitMockAnswer, pauseMock, abandonMock, flagMockQuestion, resumeMock } from '@/ipc/mock'
import type { SessionQuestionDto } from '@/ipc/questions'
import { recordSessionPresenceEvent } from '@/ipc/sessions'
import QuestionCard from '@/components/question/QuestionCard.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const route = useRoute()
const router = useRouter()
const mockSessionId = computed(() => Number(route.params.id))

const questions = ref<SessionQuestionDto[]>([])
const currentIndex = ref(0)
const loading = ref(true)
const submitting = ref(false)
const error = ref('')
const answeredCount = ref(0)
const questionStartedAt = ref(Date.now())
const underlyingSessionId = ref<number | null>(null)
const presentedItemIds = new Set<number>()

// Timer
const timeRemainingSeconds = ref(0)
let timerInterval: ReturnType<typeof setInterval> | null = null

const questionCardRef = ref<InstanceType<typeof QuestionCard> | null>(null)

const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)
const totalQuestions = computed(() => questions.value.length)
const isLast = computed(() => currentIndex.value >= totalQuestions.value - 1)
const progress = computed(() => totalQuestions.value > 0 ? answeredCount.value / totalQuestions.value : 0)

const timerDisplay = computed(() => {
  const secs = timeRemainingSeconds.value
  if (secs <= 0) return '0:00'
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}:${s.toString().padStart(2, '0')}`
})

const timerUrgent = computed(() => timeRemainingSeconds.value > 0 && timeRemainingSeconds.value <= 300)

onMounted(async () => {
  try {
    let mockSession = await getMockSession(mockSessionId.value)
    if (mockSession.status === 'paused') {
      mockSession = await resumeMock(mockSessionId.value)
    }
    underlyingSessionId.value = mockSession.session_id
    questions.value = await listMockQuestions(mockSessionId.value)
    answeredCount.value = questions.value.filter(q => q.is_answered).length
    timeRemainingSeconds.value = Math.max(0, mockSession.time_remaining_seconds ?? 0)

    // Find first unanswered
    const firstUnanswered = questions.value.findIndex(q => !q.is_answered)
    currentIndex.value = firstUnanswered >= 0 ? firstUnanswered : 0
    questionStartedAt.value = Date.now()
    presentedItemIds.clear()
    if (mockSession.status === 'active') {
      if (timeRemainingSeconds.value > 0) {
        startTimer(timeRemainingSeconds.value)
      } else {
        await expireMockSession()
        return
      }
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load exam'
  }
  loading.value = false
})

onUnmounted(() => {
  if (timerInterval) clearInterval(timerInterval)
  timerInterval = null
})

watch(
  currentQuestion,
  (question) => {
    if (!question || !underlyingSessionId.value || presentedItemIds.has(question.item_id)) return
    presentedItemIds.add(question.item_id)
    void recordSessionPresenceEvent(underlyingSessionId.value, {
      event_type: 'question_presented',
      occurred_at: new Date().toISOString(),
      metadata_json: {
        item_id: question.item_id,
        question_id: question.question_id,
        source: 'mock_hall',
      },
    }).catch(() => {
      presentedItemIds.delete(question.item_id)
    })
  },
  { immediate: true },
)

function startTimer(seconds: number) {
  if (timerInterval) {
    clearInterval(timerInterval)
  }
  timeRemainingSeconds.value = seconds
  timerInterval = setInterval(() => {
    if (timeRemainingSeconds.value <= 0) {
      clearInterval(timerInterval!)
      timerInterval = null
      void expireMockSession()
    } else {
      timeRemainingSeconds.value--
    }
  }, 1000)
}

async function handleAnswer(optionId: number, confidence: string, responseTimeMs: number) {
  const q = currentQuestion.value
  if (!q || submitting.value) return
  submitting.value = true

  try {
    const result = await submitMockAnswer({
      mock_session_id: mockSessionId.value,
      question_id: q.question_id,
      selected_option_id: optionId,
      confidence_level: confidence || null,
      response_time_ms: responseTimeMs,
      skipped: false,
      timed_out: false,
    })

    // Mark answered in local state
    questions.value[currentIndex.value] = { ...q, is_answered: true }
    answeredCount.value = result.answered_count

    if (result.time_remaining_seconds != null) {
      timeRemainingSeconds.value = Math.max(0, result.time_remaining_seconds)
      if (timerInterval === null && timeRemainingSeconds.value > 0) {
        startTimer(timeRemainingSeconds.value)
      }
    }

    if (result.remaining_count === 0) {
      finishExam()
      return
    }

    moveNext()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit answer'
  } finally {
    submitting.value = false
  }
}

async function handleSkip() {
  const q = currentQuestion.value
  if (!q || submitting.value) return
  submitting.value = true

  try {
    const result = await submitMockAnswer({
      mock_session_id: mockSessionId.value,
      question_id: q.question_id,
      selected_option_id: null,
      confidence_level: null,
      response_time_ms: Math.max(0, Date.now() - questionStartedAt.value),
      skipped: true,
      timed_out: false,
    })

    questions.value[currentIndex.value] = { ...q, is_answered: true }
    answeredCount.value = result.answered_count

    if (result.time_remaining_seconds != null) {
      timeRemainingSeconds.value = Math.max(0, result.time_remaining_seconds)
      if (timerInterval === null && timeRemainingSeconds.value > 0) {
        startTimer(timeRemainingSeconds.value)
      }
    }

    if (result.remaining_count === 0) {
      finishExam()
      return
    }

    moveNext()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to skip question'
  } finally {
    submitting.value = false
  }
}

function moveNext() {
  questionCardRef.value?.reset()
  let next = currentIndex.value + 1
  while (next < questions.value.length && questions.value[next].is_answered) {
    next++
  }
  if (next < questions.value.length) {
    currentIndex.value = next
    questionStartedAt.value = Date.now()
  } else {
    finishExam()
  }
}

async function finishExam() {
  if (timerInterval) clearInterval(timerInterval)
  timerInterval = null
  router.push(`/student/mock/review/${mockSessionId.value}`)
}

async function expireMockSession() {
  const q = currentQuestion.value
  if (q && !q.is_answered) {
    try {
      await submitMockAnswer({
        mock_session_id: mockSessionId.value,
        question_id: q.question_id,
        selected_option_id: null,
        confidence_level: null,
        response_time_ms: Math.max(0, Date.now() - questionStartedAt.value),
        skipped: false,
        timed_out: true,
      })
    } catch {
      // The backend force-completes expired mocks on timed answer attempts.
    }
  }
  await finishExam()
}

async function handlePause() {
  try {
    await pauseMock(mockSessionId.value)
  } catch {}
  router.push('/student/mock')
}

async function handleAbandon() {
  if (!confirm('Abandon this mock exam? Progress will be lost.')) return
  try {
    await abandonMock(mockSessionId.value)
  } catch {}
  router.push('/student/mock')
}

async function handleFlag(flagged: boolean) {
  const q = currentQuestion.value
  if (!q) return
  if (!flagged) {
    questionCardRef.value?.setFlagged(true)
    error.value = 'Mock review flags can be added right now, but not removed yet.'
    return
  }

  try {
    await flagMockQuestion(mockSessionId.value, q.question_id)
    questions.value[currentIndex.value] = { ...q, flagged: true }
  } catch (e: any) {
    questionCardRef.value?.setFlagged(q.flagged)
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to flag question'
  }
}
</script>

<template>
  <div class="h-full flex flex-col" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Exam Header -->
    <div
      class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center gap-3">
        <AppBadge color="danger" size="sm" dot>EXAM IN PROGRESS</AppBadge>
        <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          {{ answeredCount }}/{{ totalQuestions }} answered
        </span>
      </div>

      <div class="flex items-center gap-4">
        <!-- Timer -->
        <span
          v-if="timeRemainingSeconds > 0"
          class="text-sm font-mono font-semibold tabular-nums"
          :style="{ color: timerUrgent ? 'var(--warm)' : 'var(--ink-secondary)' }"
        >
          {{ timerDisplay }}
        </span>

        <AppProgress
          :value="answeredCount"
          :max="totalQuestions || 1"
          size="sm"
          color="accent"
          class="w-28"
        />

        <AppButton variant="ghost" size="sm" @click="handlePause">Pause</AppButton>
        <AppButton variant="ghost" size="sm" @click="handleAbandon">Abandon</AppButton>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6 lg:p-8">

      <!-- Loading -->
      <div v-if="loading" class="flex justify-center py-16">
        <div class="w-8 h-8 border-2 rounded-full animate-spin"
          :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
      </div>

      <!-- Error -->
      <div v-else-if="error" class="max-w-md mx-auto text-center py-16">
        <p class="text-sm mb-4" :style="{ color: 'var(--warm)' }">{{ error }}</p>
        <AppButton variant="secondary" @click="router.push('/student/mock')">Back</AppButton>
      </div>

      <!-- Empty -->
      <div v-else-if="questions.length === 0" class="max-w-md mx-auto text-center py-16">
        <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">No questions in this exam.</p>
        <AppButton variant="primary" @click="finishExam">Finish</AppButton>
      </div>

      <!-- Question -->
      <div v-else-if="currentQuestion" class="max-w-2xl mx-auto">
        <div class="flex items-center justify-between mb-6">
          <span class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }">
            Question {{ currentIndex + 1 }} of {{ totalQuestions }}
          </span>
          <span v-if="timerUrgent" class="text-xs font-semibold" :style="{ color: 'var(--warm)' }">
            Time running low!
          </span>
        </div>

        <!-- Submitting overlay -->
        <AppCard v-if="submitting" padding="lg" class="text-center">
          <div class="w-6 h-6 border-2 rounded-full animate-spin mx-auto"
            :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
          <p class="text-sm mt-3" :style="{ color: 'var(--ink-muted)' }">Recording answer…</p>
        </AppCard>

        <!-- NOTE: In exam mode there is no answer feedback — the exam continues immediately -->
        <QuestionCard
          v-else
          ref="questionCardRef"
          :stem="currentQuestion.stem"
          :format="currentQuestion.question_format"
          :difficulty="currentQuestion.difficulty"
          :estimated-seconds="currentQuestion.estimated_time_seconds ?? undefined"
          :initial-flagged="currentQuestion.flagged"
          :options="currentQuestion.options.map(o => ({
            id: o.id,
            label: o.label,
            text: o.text,
            is_correct: undefined,
            misconception_id: o.misconception_id,
            distractor_intent: o.distractor_intent,
          }))"
          :question-number="currentIndex + 1"
          :total-questions="totalQuestions"
          @answer="handleAnswer"
          @flag="handleFlag"
          @skip="handleSkip"
        />
      </div>

      <!-- All done -->
      <div v-else class="max-w-md mx-auto text-center py-16">
        <p class="text-sm mb-4 font-medium" :style="{ color: 'var(--ink)' }">All questions answered!</p>
        <AppButton variant="primary" size="lg" @click="finishExam">View Results →</AppButton>
      </div>

    </div>
  </div>
</template>

