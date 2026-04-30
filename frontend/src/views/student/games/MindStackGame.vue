<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, getPriorityTopics, type SubjectDto } from '@/ipc/coach'
import { startPracticeSession, completeSession, completeSessionWithPipeline } from '@/ipc/sessions'
import {
  listSessionQuestions,
  submitAttempt,
  type AttemptResultDto,
  type QuestionOptionDto,
  type SessionQuestionDto,
  type SubmitAttemptInput,
} from '@/ipc/questions'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import MathText from '@/components/question/MathText.vue'

const auth = useAuthStore()
const router = useRouter()

type Phase = 'setup' | 'playing' | 'summary'

const phase = ref<Phase>('setup')
const loading = ref(false)
const error = ref('')

const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)

// Game state
const sessionId = ref<number | null>(null)
const questions = ref<SessionQuestionDto[]>([])
const currentIndex = ref(0)
const boardHeight = ref(0) // 0 = safe, 15 = game over
const clearedRows = ref(0)
const score = ref(0)
const streak = ref(0)
const correctCount = ref(0)
const attemptedCount = ref(0)
const selectedOptionId = ref<number | null>(null)
const answerResult = ref<{ correct: boolean } | null>(null)
const startTime = ref(0)

const MAX_HEIGHT = 10
const CORRECT_CLEAR = 1
const INCORRECT_STACK = 2

const timeLeft = ref(15)
let timer: ReturnType<typeof setInterval> | null = null
const pendingSubmissions = new Map<number, Promise<AttemptResultDto>>()
const unsavedInputs = new Map<number, SubmitAttemptInput>()

onMounted(async () => {
  subjects.value = await listSubjects().catch(() => [])
  if (subjects.value.length > 0) selectedSubjectId.value = subjects.value[0].id
})

onUnmounted(() => clearTimer())

function clearTimer() {
  if (timer) { clearInterval(timer); timer = null }
}

async function startGame() {
  if (!auth.currentAccount || !selectedSubjectId.value) return
  loading.value = true
  error.value = ''
  try {
    const [priorityTopics, subjectTopics] = await Promise.all([
      getPriorityTopics(auth.currentAccount.id, 20),
      listTopics(selectedSubjectId.value).catch(() => []),
    ])
    const subjectTopicIds = new Set(subjectTopics.map(topic => topic.id))
    const topicIds = priorityTopics
      .filter(topic => subjectTopicIds.has(topic.topic_id))
      .slice(0, 5)
      .map(topic => topic.topic_id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topicIds,
      question_count: 12,
      is_timed: true,
    })
    sessionId.value = session.session_id
    questions.value = await listSessionQuestions(session.session_id)
    if (!questions.value.length) { error.value = 'No questions available.'; return }

    currentIndex.value = 0
    boardHeight.value = 0
    clearedRows.value = 0
    score.value = 0
    streak.value = 0
    correctCount.value = 0
    attemptedCount.value = 0
    selectedOptionId.value = null
    answerResult.value = null
    phase.value = 'playing'
    startQuestionTimer()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start'
  }
  loading.value = false
}

function startQuestionTimer() {
  clearTimer()
  timeLeft.value = 15
  startTime.value = Date.now()
  timer = setInterval(() => {
    timeLeft.value--
    if (timeLeft.value <= 0) {
      clearTimer()
      handleTimeout()
    }
  }, 1000)
}

function handleTimeout() {
  if (selectedOptionId.value !== null) return
  selectedOptionId.value = -1
  const q = questions.value[currentIndex.value]
  if (sessionId.value && auth.currentAccount && q) {
    const input: SubmitAttemptInput = {
      student_id: auth.currentAccount.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: null,
      response_time_ms: 15000,
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      skipped: false,
      timed_out: true,
      was_timed: true,
    }
    unsavedInputs.set(q.item_id, input)
    const submission = submitAttempt(input)
    pendingSubmissions.set(q.item_id, submission)
    void submission
      .then(() => {
        unsavedInputs.delete(q.item_id)
      })
      .catch(() => {
        // Retry during the end-of-game flush if needed.
      })
      .finally(() => {
        pendingSubmissions.delete(q.item_id)
      })
  }
  applyResult(false)
  answerResult.value = { correct: false }
  setTimeout(nextQuestion, 1000)
}

async function pickOption(option: QuestionOptionDto) {
  if (selectedOptionId.value !== null || !sessionId.value) return
  clearTimer()
  selectedOptionId.value = option.id
  const q = questions.value[currentIndex.value]
  const responseMs = Date.now() - startTime.value
  const isCorrect = option.is_correct

  const input: SubmitAttemptInput = {
      student_id: auth.currentAccount!.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: option.id,
      response_time_ms: responseMs,
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      was_timed: true,
  }
  unsavedInputs.set(q.item_id, input)
  const submission = submitAttempt(input)
  pendingSubmissions.set(q.item_id, submission)
  void submission
    .then(() => {
      unsavedInputs.delete(q.item_id)
    })
    .catch(() => {
      // Retry during the end-of-game flush if needed.
    })
    .finally(() => {
      pendingSubmissions.delete(q.item_id)
    })

  applyResult(isCorrect)
  answerResult.value = { correct: isCorrect }
  setTimeout(nextQuestion, 900)
}

function applyResult(correct: boolean) {
  attemptedCount.value++
  if (correct) {
    streak.value++
    correctCount.value++
    const bonus = streak.value >= 3 ? 50 : 0
    score.value += 100 + bonus
    boardHeight.value = Math.max(0, boardHeight.value - CORRECT_CLEAR)
    clearedRows.value++
  } else {
    streak.value = 0
    boardHeight.value = Math.min(MAX_HEIGHT, boardHeight.value + INCORRECT_STACK)
  }
  if (boardHeight.value >= MAX_HEIGHT) endGame()
}

function nextQuestion() {
  selectedOptionId.value = null
  answerResult.value = null
  if (boardHeight.value >= MAX_HEIGHT || currentIndex.value >= questions.value.length - 1) {
    endGame(); return
  }
  currentIndex.value++
  startQuestionTimer()
}

async function endGame() {
  clearTimer()
  const inFlight = Array.from(pendingSubmissions.values())
  if (inFlight.length > 0) {
    await Promise.allSettled(inFlight)
  }
  for (const [itemId, input] of Array.from(unsavedInputs.entries())) {
    try {
      await submitAttempt(input)
      unsavedInputs.delete(itemId)
    } catch {
      // Best effort only for the arcade layer.
    }
  }
  if (sessionId.value && auth.currentAccount) {
    await completeSessionWithPipeline(auth.currentAccount.id, sessionId.value)
      .catch(() => completeSession(sessionId.value!).catch(() => null))
  }
  phase.value = 'summary'
}

const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)
const heightPercent = computed(() => (boardHeight.value / MAX_HEIGHT) * 100)
const survived = computed(() => boardHeight.value < MAX_HEIGHT)
const accuracy = computed(() =>
  attemptedCount.value > 0 ? Math.round((correctCount.value / attemptedCount.value) * 100) : 0,
)

// Block visualization: rows of height MAX_HEIGHT
const blocks = computed(() => {
  return Array.from({ length: MAX_HEIGHT }, (_, i) => ({
    filled: MAX_HEIGHT - 1 - i < boardHeight.value,
    danger: MAX_HEIGHT - 1 - i < 3 && MAX_HEIGHT - 1 - i < boardHeight.value,
  }))
})
</script>

<template>
  <div class="min-h-screen flex flex-col" data-mode="game" :style="{ backgroundColor: 'var(--paper)' }">
    <!-- Header -->
    <div class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
      <div class="flex items-center gap-2">
        <span class="font-display font-bold" :style="{ color: 'var(--accent)' }">▣ MindStack</span>
        <span v-if="phase === 'playing'" class="text-xs px-2 py-0.5 rounded-full font-semibold"
          :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--accent)' }">
          {{ currentIndex + 1 }}/{{ questions.length }}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <span v-if="phase === 'playing'" class="font-display text-lg font-bold tabular-nums" :style="{ color: 'var(--gold)' }">{{ score }}</span>
        <AppButton variant="ghost" size="sm" @click="router.push('/student/games')">Exit</AppButton>
      </div>
    </div>

    <!-- Setup -->
    <div v-if="phase === 'setup'" class="flex-1 flex items-center justify-center p-6">
      <AppCard padding="lg" class="w-full max-w-md text-center">
        <div class="w-20 h-20 rounded-3xl mx-auto mb-6 flex items-center justify-center text-4xl"
          :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--accent)' }">▣</div>
        <h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--ink)' }">MindStack</h2>
        <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">
          Answer correctly to clear rows. Wrong answers stack the blocks higher.
          Don't let the stack reach the top!
        </p>

        <div v-if="error" class="mb-4 p-3 rounded-lg text-sm text-left"
          :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">{{ error }}</div>

        <div class="mb-6 text-left">
          <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Subject</p>
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="subj in subjects"
              :key="subj.id"
              class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
              :style="{
                backgroundColor: selectedSubjectId === subj.id ? 'var(--accent)' : 'var(--border-soft)',
                color: selectedSubjectId === subj.id ? 'white' : 'var(--ink-secondary)',
              }"
              @click="selectedSubjectId = subj.id"
            >
              {{ subj.name }}
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-3 mb-6 text-center">
          <div class="p-2 rounded-lg" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">−1</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Correct clears</p>
          </div>
          <div class="p-2 rounded-lg" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--warm)' }">+2</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Wrong stacks</p>
          </div>
          <div class="p-2 rounded-lg" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--gold)' }">10</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Max height</p>
          </div>
        </div>

        <AppButton variant="primary" size="lg" class="w-full" :loading="loading" @click="startGame">
          Start Game →
        </AppButton>
      </AppCard>
    </div>

    <!-- Playing -->
    <div v-else-if="phase === 'playing'" class="flex-1 flex gap-6 p-4 max-w-3xl mx-auto w-full">
      <!-- Stack Visualization -->
      <div class="flex flex-col items-center gap-1 flex-shrink-0">
        <p class="text-[10px] uppercase font-bold mb-1" :style="{ color: 'var(--ink-muted)' }">Stack</p>
        <div v-for="(block, i) in blocks" :key="i"
          class="w-8 h-5 rounded-sm transition-all duration-300"
          :style="{
            backgroundColor: !block.filled ? 'var(--border-soft)' : block.danger ? 'var(--warm)' : 'var(--accent)',
            opacity: block.filled ? 1 : 0.3,
          }" />
        <p class="text-[10px] mt-1 tabular-nums" :style="{ color: 'var(--ink-muted)' }">{{ boardHeight }}/{{ MAX_HEIGHT }}</p>
      </div>

      <!-- Question area -->
      <div class="flex-1 flex flex-col">
        <!-- Danger bar -->
        <div v-if="boardHeight > 6" class="mb-3 p-2 rounded-lg text-xs font-bold text-center"
          :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
          ⚠ Stack getting high! Answer correctly to clear!
        </div>

        <!-- Timer -->
        <div class="mb-3">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">Timer</span>
            <span class="text-xs font-bold tabular-nums"
              :style="{ color: timeLeft <= 5 ? 'var(--warm)' : 'var(--ink-secondary)' }">{{ timeLeft }}s</span>
          </div>
          <AppProgress :value="timeLeft" :max="15" size="sm"
            :color="timeLeft <= 5 ? 'danger' : timeLeft <= 8 ? 'warm' : 'accent'" />
        </div>

        <!-- Streak -->
        <div v-if="streak >= 3" class="mb-2 text-xs font-bold text-center"
          :style="{ color: 'var(--gold)' }">⚡ {{ streak }}-streak — Combo bonus active!</div>

        <!-- Question -->
        <AppCard v-if="currentQuestion" padding="md" class="flex-1">
          <!-- Result -->
          <div v-if="answerResult" class="mb-3 p-2 rounded-lg text-xs font-semibold text-center"
            :style="{
              backgroundColor: answerResult.correct ? 'var(--success-light)' : 'rgba(194,65,12,0.08)',
              color: answerResult.correct ? 'var(--accent)' : 'var(--warm)',
            }">
            {{ answerResult.correct ? '✓ Row cleared!' : '✗ Block added!' }}
          </div>

          <p class="text-sm font-medium leading-relaxed mb-4" :style="{ color: 'var(--ink)' }">
            <MathText :text="currentQuestion.stem" size="sm" />
          </p>

          <div class="space-y-2">
            <button
              v-for="opt in currentQuestion.options"
              :key="opt.id"
              class="w-full text-left px-3 py-2.5 rounded-xl border transition-all text-sm"
              :disabled="selectedOptionId !== null"
              :style="{
                borderColor: selectedOptionId === null ? 'var(--border-soft)'
                  : selectedOptionId === opt.id
                    ? (answerResult?.correct ? 'var(--accent)' : 'var(--warm)')
                    : 'var(--border-soft)',
                backgroundColor: selectedOptionId === null ? 'var(--surface)'
                  : selectedOptionId === opt.id
                    ? (answerResult?.correct ? 'var(--success-light)' : 'rgba(194,65,12,0.08)')
                    : 'var(--surface)',
                color: 'var(--ink)',
                opacity: selectedOptionId !== null && selectedOptionId !== opt.id ? '0.5' : '1',
              }"
              @click="pickOption(opt)"
            >
              <span class="font-semibold mr-2" :style="{ color: 'var(--ink-muted)' }">{{ opt.label }}.</span>
              <MathText :text="opt.text" size="sm" />
            </button>
          </div>
        </AppCard>

        <!-- Stats row -->
        <div class="flex gap-3 mt-3">
          <AppCard padding="sm" class="text-center flex-1">
            <p class="font-display text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ clearedRows }}</p>
            <p class="text-[8px] uppercase" :style="{ color: 'var(--ink-muted)' }">Cleared</p>
          </AppCard>
          <AppCard padding="sm" class="text-center flex-1">
            <p class="font-display text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ score }}</p>
            <p class="text-[8px] uppercase" :style="{ color: 'var(--ink-muted)' }">Score</p>
          </AppCard>
        </div>
      </div>
    </div>

    <!-- Summary -->
    <div v-else-if="phase === 'summary'" class="flex-1 flex items-center justify-center p-6">
      <AppCard padding="lg" class="w-full max-w-md text-center">
        <div class="text-4xl mb-4">{{ survived ? '🏆' : '💥' }}</div>
        <h2 class="font-display text-2xl font-bold mb-1"
          :style="{ color: survived ? 'var(--accent)' : 'var(--warm)' }">
          {{ survived ? 'Stack Cleared!' : 'Stack Overflow!' }}
        </h2>
        <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">
          {{ survived ? `You cleared ${clearedRows} rows and survived all questions!` : "The stack reached the top. Keep practising!" }}
        </p>

        <div class="grid grid-cols-3 gap-3 mb-6">
          <div class="p-3 rounded-xl" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">{{ score }}</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Score</p>
          </div>
          <div class="p-3 rounded-xl" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ accuracy }}%</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Accuracy</p>
          </div>
          <div class="p-3 rounded-xl" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ clearedRows }}</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Rows Cleared</p>
          </div>
        </div>

        <div class="flex gap-3">
          <AppButton variant="primary" class="flex-1" @click="phase = 'setup'">Play Again</AppButton>
          <AppButton variant="secondary" @click="router.push('/student/games')">Back</AppButton>
        </div>
      </AppCard>
    </div>
  </div>
</template>

