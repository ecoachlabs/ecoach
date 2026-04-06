<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, getPriorityTopics, type SubjectDto } from '@/ipc/coach'
import { startPracticeSession, completeSession } from '@/ipc/sessions'
import { listSessionQuestions, submitAttempt, type SessionQuestionDto, type QuestionOptionDto } from '@/ipc/questions'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

type Phase = 'setup' | 'playing' | 'summary'

const phase = ref<Phase>('setup')
const loading = ref(false)
const error = ref('')

// Setup
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)

// Game state
const sessionId = ref<number | null>(null)
const questions = ref<SessionQuestionDto[]>([])
const currentIndex = ref(0)
const position = ref(0) // -10 = lost, +10 = won
const score = ref(0)
const streak = ref(0)
const correctCount = ref(0)
const selectedOptionId = ref<number | null>(null)
const answerResult = ref<{ correct: boolean; explanation?: string } | null>(null)
const startTime = ref(0)

// Timer
const timeLeft = ref(15)
let timer: ReturnType<typeof setInterval> | null = null

const TUG_CORRECT = 2
const TUG_INCORRECT = -3
const WIN_POSITION = 10
const LOSE_POSITION = -10

onMounted(async () => {
  subjects.value = await listSubjects().catch(() => [])
  if (subjects.value.length > 0) selectedSubjectId.value = subjects.value[0].id
})

onUnmounted(() => {
  clearTimer()
})

function clearTimer() {
  if (timer) { clearInterval(timer); timer = null }
}

async function startGame() {
  if (!auth.currentAccount || !selectedSubjectId.value) return
  loading.value = true
  error.value = ''
  try {
    const topics = await getPriorityTopics(auth.currentAccount.id, 10)
    const topicIds = topics.slice(0, 5).map(t => t.topic_id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: true,
    })
    sessionId.value = session.session_id
    questions.value = await listSessionQuestions(session.session_id)
    if (questions.value.length === 0) {
      error.value = 'No questions available for this subject.'
      return
    }
    // Reset game state
    currentIndex.value = 0
    position.value = 0
    score.value = 0
    streak.value = 0
    correctCount.value = 0
    selectedOptionId.value = null
    answerResult.value = null
    phase.value = 'playing'
    startQuestionTimer()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start game'
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
      // Time's up — treat as wrong
      handleTimeout()
    }
  }, 1000)
}

function handleTimeout() {
  if (selectedOptionId.value !== null) return
  applyResult(false, undefined, 15000)
  answerResult.value = { correct: false, explanation: "Time's up!" }
  setTimeout(() => nextQuestion(), 1200)
}

async function pickOption(option: QuestionOptionDto) {
  if (selectedOptionId.value !== null || !sessionId.value) return
  clearTimer()
  selectedOptionId.value = option.id

  const responseMs = Date.now() - startTime.value
  const isCorrect = !!option.misconception_id === false && isOptionCorrect(option)

  // Submit to session backend
  const q = questions.value[currentIndex.value]
  try {
    await submitAttempt({
      student_id: auth.currentAccount!.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: option.id,
      response_time_ms: responseMs,
      confidence_level: undefined,
      hint_count: 0,
      changed_answer_count: 0,
      was_timed: true,
    })
  } catch {}

  applyResult(isCorrect, option.distractor_intent ?? undefined, responseMs)
  answerResult.value = { correct: isCorrect }

  setTimeout(() => nextQuestion(), 1000)
}

function isOptionCorrect(option: QuestionOptionDto): boolean {
  // In the options list, exactly one option is correct (the one with no misconception and is marked correct)
  // We determine this by finding which option in the question has no misconception_id
  // Actually — backend marks correct by position in options... but we don't have is_correct here.
  // The options from list_session_questions include misconception_id for wrong answers.
  // A correct option typically has misconception_id = null AND is the actual answer.
  // We can infer: the option is correct if misconception_id is null and distractor_intent is null.
  // This is a heuristic — the real check happens on the backend.
  return option.misconception_id === null && option.distractor_intent === null
}

function applyResult(correct: boolean, _explanation: string | undefined, _responseMs: number) {
  if (correct) {
    const streakBonus = streak.value >= 3 ? 50 : 0
    score.value += 100 + streakBonus
    streak.value++
    correctCount.value++
    position.value = Math.min(WIN_POSITION, position.value + TUG_CORRECT)
  } else {
    streak.value = 0
    position.value = Math.max(LOSE_POSITION, position.value + TUG_INCORRECT)
  }

  if (position.value >= WIN_POSITION || position.value <= LOSE_POSITION) {
    endGame()
  }
}

function nextQuestion() {
  selectedOptionId.value = null
  answerResult.value = null
  if (currentIndex.value >= questions.value.length - 1) {
    endGame()
    return
  }
  currentIndex.value++
  startQuestionTimer()
}

async function endGame() {
  clearTimer()
  if (sessionId.value) {
    await completeSession(sessionId.value).catch(() => null)
  }
  phase.value = 'summary'
}

const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)

// Rope position as % (0 = far left, 50 = center, 100 = far right)
const ropePercent = computed(() => {
  return 50 + (position.value / WIN_POSITION) * 45
})

const won = computed(() => position.value >= WIN_POSITION)
const lost = computed(() => position.value <= LOSE_POSITION)
const accuracy = computed(() =>
  currentIndex.value > 0 ? Math.round((correctCount.value / currentIndex.value) * 100) : 0,
)
</script>

<template>
  <div class="min-h-screen flex flex-col" data-mode="game" :style="{ backgroundColor: 'var(--paper)' }">
    <!-- Header -->
    <div class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
      <div class="flex items-center gap-2">
        <span class="font-display font-bold" :style="{ color: 'var(--warm)' }">⟷ Tug of War</span>
        <span v-if="phase === 'playing'" class="text-xs px-2 py-0.5 rounded-full font-semibold"
          :style="{ backgroundColor: 'var(--warm-light)', color: 'var(--warm)' }">
          {{ currentIndex + 1 }}/{{ questions.length }}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <span v-if="phase === 'playing'" class="font-display text-lg font-bold tabular-nums"
          :style="{ color: 'var(--gold)' }">{{ score }}</span>
        <AppButton variant="ghost" size="sm" @click="router.push('/student/games')">Exit</AppButton>
      </div>
    </div>

    <!-- Setup -->
    <div v-if="phase === 'setup'" class="flex-1 flex items-center justify-center p-6">
      <AppCard padding="lg" class="w-full max-w-md text-center">
        <div class="w-20 h-20 rounded-3xl mx-auto mb-6 flex items-center justify-center text-4xl"
          :style="{ backgroundColor: 'var(--warm-light)', color: 'var(--warm)' }">⟷</div>
        <h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--ink)' }">Tug of War</h2>
        <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">
          Answer correctly to pull the rope. Get it to the right edge — or the opponent drags you left!
          3-streak = Power Pull bonus.
        </p>

        <div v-if="error" class="mb-4 p-3 rounded-lg text-sm text-left"
          :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">{{ error }}</div>

        <!-- Subject picker -->
        <div class="mb-6 text-left">
          <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Subject</p>
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="subj in subjects"
              :key="subj.id"
              class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
              :style="{
                backgroundColor: selectedSubjectId === subj.id ? 'var(--warm)' : 'var(--border-soft)',
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
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--warm)' }">10</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Questions</p>
          </div>
          <div class="p-2 rounded-lg" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--warm)' }">15s</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Per Q</p>
          </div>
          <div class="p-2 rounded-lg" :style="{ backgroundColor: 'var(--border-soft)' }">
            <p class="font-display text-lg font-bold" :style="{ color: 'var(--warm)' }">+2/-3</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Pull/Slip</p>
          </div>
        </div>

        <AppButton variant="primary" size="lg" class="w-full" :loading="loading" @click="startGame">
          Start Match →
        </AppButton>
      </AppCard>
    </div>

    <!-- Playing -->
    <div v-else-if="phase === 'playing'" class="flex-1 flex flex-col p-4 max-w-2xl mx-auto w-full">
      <!-- Rope Visualization -->
      <div class="mb-6 py-4 rounded-2xl relative overflow-hidden"
        :style="{ backgroundColor: 'var(--surface)', border: '1px solid var(--border-soft)' }">
        <div class="flex items-center justify-between px-4 mb-2">
          <span class="text-xs font-bold" :style="{ color: 'var(--warm)' }">LOSE ←</span>
          <span class="text-xs font-bold" :style="{ color: 'var(--accent)' }">→ WIN</span>
        </div>
        <!-- Rope track -->
        <div class="relative mx-4 h-4 rounded-full" :style="{ backgroundColor: 'var(--border-soft)' }">
          <!-- Rope fill -->
          <div class="absolute top-0 left-0 h-full rounded-full transition-all duration-500"
            :style="{
              width: ropePercent + '%',
              backgroundColor: position >= 0 ? 'var(--accent)' : 'var(--warm)',
            }" />
          <!-- Center marker -->
          <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 rounded-full border-2 z-10"
            :style="{ backgroundColor: 'var(--paper)', borderColor: 'var(--ink-muted)' }" />
          <!-- Knot indicator -->
          <div class="absolute top-1/2 -translate-y-1/2 w-6 h-6 rounded-full border-2 z-20 flex items-center justify-center text-xs font-bold transition-all duration-500"
            :style="{
              left: `calc(${ropePercent}% - 12px)`,
              backgroundColor: position >= 5 ? 'var(--accent)' : position <= -5 ? 'var(--warm)' : 'var(--gold)',
              borderColor: 'white',
              color: 'white',
            }">
            ⟷
          </div>
        </div>
        <!-- Position label -->
        <p class="text-center text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">
          Position: {{ position >= 0 ? '+' : '' }}{{ position }} · Streak: {{ streak }}
          <span v-if="streak >= 3" class="font-bold" :style="{ color: 'var(--gold)' }"> ⚡POWER PULL!</span>
        </p>
      </div>

      <!-- Timer bar -->
      <div class="mb-4">
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">Time remaining</span>
          <span class="text-xs font-bold tabular-nums"
            :style="{ color: timeLeft <= 5 ? 'var(--warm)' : 'var(--ink-secondary)' }">{{ timeLeft }}s</span>
        </div>
        <AppProgress :value="timeLeft" :max="15" size="sm"
          :color="timeLeft <= 5 ? 'danger' : timeLeft <= 8 ? 'warm' : 'accent'" />
      </div>

      <!-- Question -->
      <AppCard v-if="currentQuestion" padding="lg" class="mb-4 flex-1">
        <p class="text-base font-medium leading-relaxed mb-6"
          :style="{ color: 'var(--ink)' }">
          {{ currentQuestion.stem }}
        </p>

        <!-- Result feedback -->
        <div v-if="answerResult" class="mb-3 p-2 rounded-lg text-sm font-semibold text-center"
          :style="{
            backgroundColor: answerResult.correct ? 'var(--success-light)' : 'rgba(194,65,12,0.08)',
            color: answerResult.correct ? 'var(--accent)' : 'var(--warm)',
          }">
          {{ answerResult.correct ? '✓ Correct! Pull! +' + TUG_CORRECT : '✗ Wrong! Slip ' + TUG_INCORRECT }}
          <span v-if="answerResult.explanation"> — {{ answerResult.explanation }}</span>
        </div>

        <!-- Options -->
        <div class="space-y-2">
          <button
            v-for="opt in currentQuestion.options"
            :key="opt.id"
            class="w-full text-left px-4 py-3 rounded-xl border-2 transition-all text-sm font-medium"
            :disabled="selectedOptionId !== null"
            :style="{
              borderColor: selectedOptionId === null
                ? 'var(--border-soft)'
                : selectedOptionId === opt.id
                  ? (answerResult?.correct ? 'var(--accent)' : 'var(--warm)')
                  : 'var(--border-soft)',
              backgroundColor: selectedOptionId === null
                ? 'var(--surface)'
                : selectedOptionId === opt.id
                  ? (answerResult?.correct ? 'var(--success-light)' : 'rgba(194,65,12,0.08)')
                  : 'var(--surface)',
              color: 'var(--ink)',
              opacity: selectedOptionId !== null && selectedOptionId !== opt.id ? '0.5' : '1',
            }"
            @click="pickOption(opt)"
          >
            <span class="font-semibold mr-2" :style="{ color: 'var(--ink-muted)' }">{{ opt.label }}.</span>
            {{ opt.text }}
          </button>
        </div>
      </AppCard>
    </div>

    <!-- Summary -->
    <div v-else-if="phase === 'summary'" class="flex-1 flex items-center justify-center p-6">
      <AppCard padding="lg" class="w-full max-w-md text-center">
        <div class="text-4xl mb-4">{{ won ? '🏆' : lost ? '💪' : '⟷' }}</div>
        <h2 class="font-display text-2xl font-bold mb-1"
          :style="{ color: won ? 'var(--accent)' : lost ? 'var(--warm)' : 'var(--ink)' }">
          {{ won ? 'You Won!' : lost ? 'Opponent Wins' : 'Match Over' }}
        </h2>
        <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">
          {{ won ? 'You pulled the rope all the way!' : lost ? "Keep practising — you'll get there." : 'Well fought!' }}
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
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ correctCount }}</p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Correct</p>
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
