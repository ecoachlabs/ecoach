<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ipc } from '@/ipc'
import {
  listSessionQuestions,
  submitAttempt,
  type SessionQuestionDto,
  type AttemptResultDto,
} from '@/ipc/questions'
import QuestionCard from '@/components/question/QuestionCard.vue'
import QuestionFeedback from '@/components/question/QuestionFeedback.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const sessionId = computed(() => Number(route.params.id))

// Session data
const questions = ref<SessionQuestionDto[]>([])
const currentIndex = ref(0)
const loading = ref(true)
const error = ref('')

// Per-question state
const answering = ref(false)
const answered = ref(false)
const result = ref<AttemptResultDto | null>(null)
const startTime = ref(Date.now())
const selectedOptionId = ref<number | null>(null)

const questionCardRef = ref<InstanceType<typeof QuestionCard> | null>(null)

// Computed
const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)
const totalQuestions = computed(() => questions.value.length)
const answeredCount = computed(() => questions.value.filter((q) => q.is_answered).length)
const isLast = computed(() => currentIndex.value >= totalQuestions.value - 1)

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  try {
    questions.value = await listSessionQuestions(sessionId.value)
    if (questions.value.length === 0) {
      error.value = 'This session has no questions.'
    }
    // Start at the first unanswered question
    const firstUnanswered = questions.value.findIndex((q) => !q.is_answered)
    currentIndex.value = firstUnanswered >= 0 ? firstUnanswered : 0
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load session'
  } finally {
    loading.value = false
  }
})

// ── Actions ───────────────────────────────────────────────────────────────────

async function handleAnswer(optionId: number, confidence: string, responseTimeMs: number) {
  const q = currentQuestion.value
  if (!q || !auth.currentAccount || answering.value) return

  answering.value = true
  selectedOptionId.value = optionId
  try {
    result.value = await submitAttempt({
      student_id: auth.currentAccount.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: optionId,
      response_time_ms: responseTimeMs,
      confidence_level: confidence,
      hint_count: 0,
      changed_answer_count: 0,
      was_timed: false,
    })
    // Mark as answered in local state
    questions.value[currentIndex.value] = { ...q, is_answered: true }
    answered.value = true
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit answer'
  } finally {
    answering.value = false
  }
}

function nextQuestion() {
  const sessionComplete = result.value?.session_complete ?? false
  answered.value = false
  result.value = null
  selectedOptionId.value = null
  questionCardRef.value?.reset()

  if (sessionComplete || isLast.value) {
    completeSession()
    return
  }

  // Move to next unanswered question
  let next = currentIndex.value + 1
  while (next < questions.value.length && questions.value[next].is_answered) {
    next++
  }
  if (next < questions.value.length) {
    currentIndex.value = next
    startTime.value = Date.now()
  } else {
    completeSession()
  }
}

async function completeSession() {
  try {
    await ipc('complete_session', { sessionId: sessionId.value })
  } catch {}
  router.push(`/student/session/${sessionId.value}/debrief`)
}

function handleFlag() {
  // Flag is handled inside QuestionCard — no extra action needed here
}
</script>

<template>
  <div class="h-full flex flex-col" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center gap-3">
        <span class="session-badge">Session</span>
        <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          {{ answeredCount }}/{{ totalQuestions }} answered
        </span>
      </div>
      <div class="flex items-center gap-4">
        <AppProgress
          :value="answeredCount"
          :max="totalQuestions || 1"
          size="sm"
          color="accent"
          class="w-28"
        />
        <button class="end-btn" @click="completeSession">End Session</button>
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
        <AppButton variant="secondary" @click="router.push('/student/practice')">Back</AppButton>
      </div>

      <!-- Empty -->
      <div v-else-if="questions.length === 0" class="max-w-md mx-auto text-center py-16">
        <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">No questions in this session.</p>
        <AppButton variant="primary" @click="completeSession">Finish</AppButton>
      </div>

      <!-- Question + Feedback -->
      <div v-else-if="currentQuestion" class="max-w-2xl mx-auto">

        <!-- Position indicator -->
        <div class="flex items-center justify-between mb-6">
          <span class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }">
            Question {{ currentIndex + 1 }} of {{ totalQuestions }}
          </span>
        </div>

        <!-- Question card (hidden after answering) -->
        <QuestionCard
          v-if="!answered"
          ref="questionCardRef"
          :stem="currentQuestion.stem"
          :format="currentQuestion.question_format"
          :difficulty="currentQuestion.difficulty"
          :estimated-seconds="currentQuestion.estimated_time_seconds ?? undefined"
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
          @skip="nextQuestion"
        />

        <!-- Loading overlay while submitting -->
        <AppCard v-else-if="answering" padding="lg" class="text-center">
          <div class="w-6 h-6 border-2 rounded-full animate-spin mx-auto"
            :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
          <p class="text-sm mt-3" :style="{ color: 'var(--ink-muted)' }">Analysing your answer…</p>
        </AppCard>

        <!-- Feedback after answering -->
        <QuestionFeedback
          v-else-if="answered && result"
          :is-correct="result.is_correct"
          :explanation="result.explanation"
          :error-type="result.error_type"
          :diagnosis-summary="result.diagnosis_summary"
          :recommended-action="result.recommended_action"
          :selected-option-text="result.selected_option_text ?? ''"
          :correct-option-text="result.correct_option_text"
          :misconception-info="result.misconception_info"
          :selected-option-id="selectedOptionId"
          :correct-option-id="currentQuestion?.options.find(o => o.text === result?.correct_option_text)?.id ?? null"
          :options="currentQuestion?.options"
          @next="nextQuestion"
          @review="router.push('/student/mistakes')"
        />
      </div>

      <!-- All answered -->
      <div v-else class="max-w-md mx-auto text-center py-16">
        <p class="text-sm mb-4 font-medium" :style="{ color: 'var(--ink)' }">All questions answered!</p>
        <AppButton variant="primary" size="lg" @click="completeSession">View Results →</AppButton>
      </div>

    </div>
  </div>
</template>

<style scoped>
.session-badge {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--accent-glow);
  color: var(--accent);
  border: 1px solid rgba(13,148,136,0.2);
}
.end-btn {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.end-btn:hover { background: rgba(194,65,12,0.08); color: var(--warm); border-color: rgba(194,65,12,0.3); }
</style>
