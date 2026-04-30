<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getDiagnosticBattery,
  listDiagnosticPhaseItems,
  submitDiagnosticAttempt,
  advanceDiagnosticPhase,
  completeDiagnosticAndSync,
  type DiagnosticBatteryDto,
  type DiagnosticPhaseItemDto,
} from '@/ipc/diagnostic'
import { getQuestionOptions, type QuestionOptionDto } from '@/ipc/questions'
import QuestionCard from '@/components/question/QuestionCard.vue'
import QuestionFeedback from '@/components/question/QuestionFeedback.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const diagnosticId = computed(() => Number(route.params.id))

// Battery and phases
const battery = ref<DiagnosticBatteryDto | null>(null)
const currentPhaseIndex = ref(0)

// Current phase questions
const phaseItems = ref<DiagnosticPhaseItemDto[]>([])
const currentItemIndex = ref(0)

// Options for current question (loaded separately since DiagnosticPhaseItem has stem but no options)
const currentOptions = ref<QuestionOptionDto[]>([])

// State machine
type Stage = 'loading' | 'phase-intro' | 'question' | 'feedback' | 'phase-complete' | 'all-complete' | 'error'
const stage = ref<Stage>('loading')
const error = ref('')

// Answer state
const lastResult = ref<{
  isCorrect: boolean
  selectedOptionText: string
  correctOptionText: string | null
  explanation: string | null
  errorType: string | null
  diagnosisSummary: string | null
  recommendedAction: string | null
} | null>(null)

const questionCardRef = ref<InstanceType<typeof QuestionCard> | null>(null)
const startTime = ref(Date.now())
const questionFocusedAt = ref(new Date().toISOString())

// ── Computed helpers ─────────────────────────────────────────────────────────

const currentPhase = computed(() =>
  battery.value?.phases[currentPhaseIndex.value] ?? null,
)

const currentItem = computed(() =>
  phaseItems.value[currentItemIndex.value] ?? null,
)

const totalQuestionsInPhase = computed(() => phaseItems.value.length)

const overallProgress = computed(() => {
  if (!battery.value) return { answered: 0, total: 0 }
  const phasesComplete = currentPhaseIndex.value
  const questionsPerPhase = battery.value.phases.map((p) => p.question_count)
  const answered =
    questionsPerPhase.slice(0, phasesComplete).reduce((a, b) => a + b, 0) +
    currentItemIndex.value
  const total = questionsPerPhase.reduce((a, b) => a + b, 0)
  return { answered, total }
})

// ── Lifecycle ────────────────────────────────────────────────────────────────

onMounted(async () => {
  try {
    battery.value = await getDiagnosticBattery(diagnosticId.value)
    if (!battery.value || battery.value.phases.length === 0) {
      error.value = 'This diagnostic has no phases.'
      stage.value = 'error'
      return
    }
    // Find the first non-completed phase
    const firstPending = battery.value.phases.findIndex((p) => p.status !== 'completed')
    currentPhaseIndex.value = firstPending >= 0 ? firstPending : 0
    stage.value = 'phase-intro'
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load diagnostic'
    stage.value = 'error'
  }
})

// ── Actions ──────────────────────────────────────────────────────────────────

async function startPhase() {
  if (!currentPhase.value) return
  stage.value = 'loading'
  try {
    phaseItems.value = await listDiagnosticPhaseItems(
      diagnosticId.value,
      currentPhase.value.phase_number,
    )
    currentItemIndex.value = 0
    await loadCurrentOptions()
    startTime.value = Date.now()
    questionFocusedAt.value = new Date().toISOString()
    stage.value = 'question'
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load questions'
    stage.value = 'error'
  }
}

async function loadCurrentOptions() {
  if (!currentItem.value) return
  try {
    currentOptions.value = await getQuestionOptions(currentItem.value.question_id)
  } catch {
    currentOptions.value = []
  }
}

async function handleAnswer(optionId: number, confidence: string, responseTimeMs: number) {
  if (!currentItem.value) return
  stage.value = 'loading'

  const selectedOpt = currentOptions.value.find((o) => o.id === optionId)
  const correctOpt = currentOptions.value.find((o) => {
    // We don't know which is correct yet — backend will tell us after submission
    return false
  })

  try {
    await submitDiagnosticAttempt(diagnosticId.value, {
      attempt_id: currentItem.value.attempt_id,
      selected_option_id: optionId,
      response_time_ms: responseTimeMs,
      confidence_level: confidence,
      changed_answer_count: 0,
      skipped: false,
      timed_out: false,
      first_focus_at: questionFocusedAt.value,
      first_input_at: new Date().toISOString(),
      concept_guess: null,
      final_answer: null,
      interaction_log: {
        source: 'diagnostic_session',
        events: [
          { type: 'question_presented', at: questionFocusedAt.value },
          { type: 'answer_submitted', at: new Date().toISOString() },
        ],
      },
    })

    // Simple correctness check — diagnostic doesn't reveal correct answer in real time
    // (it's an assessment, not a teaching moment), so we just show "Answer recorded"
    lastResult.value = {
      isCorrect: false, // will not show correctness in diagnostic
      selectedOptionText: selectedOpt?.text ?? '',
      correctOptionText: null,
      explanation: null,
      errorType: null,
      diagnosisSummary: null,
      recommendedAction: null,
    }
    stage.value = 'question' // stay in question mode but advance
    await nextQuestion()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to submit answer'
    stage.value = 'question'
  }
}

async function handleSkip() {
  if (!currentItem.value) return
  stage.value = 'loading'

  try {
    await submitDiagnosticAttempt(diagnosticId.value, {
      attempt_id: currentItem.value.attempt_id,
      selected_option_id: null,
      response_time_ms: Math.max(0, Date.now() - startTime.value),
      confidence_level: null,
      changed_answer_count: 0,
      skipped: true,
      timed_out: false,
      first_focus_at: questionFocusedAt.value,
      first_input_at: null,
      concept_guess: null,
      final_answer: null,
      interaction_log: {
        source: 'diagnostic_session',
        events: [
          { type: 'question_presented', at: questionFocusedAt.value },
          { type: 'question_skipped', at: new Date().toISOString() },
        ],
      },
    })

    lastResult.value = null
    await nextQuestion()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to skip question'
    stage.value = 'question'
  }
}

async function handleTimeout() {
  if (!currentItem.value) return
  stage.value = 'loading'

  try {
    await submitDiagnosticAttempt(diagnosticId.value, {
      attempt_id: currentItem.value.attempt_id,
      selected_option_id: null,
      response_time_ms: currentPhase.value?.time_limit_seconds
        ? currentPhase.value.time_limit_seconds * 1000
        : Math.max(0, Date.now() - startTime.value),
      confidence_level: null,
      changed_answer_count: 0,
      skipped: false,
      timed_out: true,
      first_focus_at: questionFocusedAt.value,
      first_input_at: null,
      concept_guess: null,
      final_answer: null,
      interaction_log: {
        source: 'diagnostic_session',
        events: [
          { type: 'question_presented', at: questionFocusedAt.value },
          { type: 'question_timed_out', at: new Date().toISOString() },
        ],
      },
    })

    lastResult.value = null
    await nextQuestion()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to record timeout'
    stage.value = 'question'
  }
}

async function nextQuestion() {
  questionCardRef.value?.reset()
  if (currentItemIndex.value < phaseItems.value.length - 1) {
    currentItemIndex.value++
    await loadCurrentOptions()
    startTime.value = Date.now()
    questionFocusedAt.value = new Date().toISOString()
    stage.value = 'question'
  } else {
    // Phase complete — try to advance
    await handlePhaseComplete()
  }
}

async function handlePhaseComplete() {
  if (!currentPhase.value || !battery.value) return
  stage.value = 'loading'
  try {
    const nextPhase = await advanceDiagnosticPhase(
      diagnosticId.value,
      currentPhase.value.phase_number,
    )
    if (nextPhase) {
      // There's another phase
      currentPhaseIndex.value++
      stage.value = 'phase-intro'
    } else {
      // No more phases — complete the diagnostic
      await finishDiagnostic()
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to advance phase'
    stage.value = 'error'
  }
}

async function finishDiagnostic() {
  stage.value = 'loading'
  try {
    await completeDiagnosticAndSync(diagnosticId.value)
    stage.value = 'all-complete'
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to complete diagnostic'
    stage.value = 'error'
  }
}

function goToReport() {
  router.push(`/student/diagnostic/${diagnosticId.value}/report`)
}

function endDiagnostic() {
  router.push('/student')
}

// Phase condition labels
const conditionLabels: Record<string, string> = {
  standard: 'Standard',
  timed: 'Timed',
  pressure: 'Pressure',
  speed: 'Speed',
  mixed: 'Mixed',
}

const phaseColors: Record<string, string> = {
  recall: 'accent',
  speed: 'warm',
  precision: 'success',
  application: 'gold',
  mixed: 'muted',
}
</script>

<template>
  <div class="h-full flex flex-col" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="shrink-0 px-6 py-3 flex items-center justify-between border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center gap-3">
        <AppBadge color="accent" size="sm" dot>Diagnostic</AppBadge>
        <span v-if="currentPhase" class="text-xs font-medium" :style="{ color: 'var(--ink-muted)' }">
          Phase {{ currentPhase.phase_number }}: {{ currentPhase.phase_title }}
        </span>
      </div>
      <div class="flex items-center gap-4">
        <div v-if="overallProgress.total > 0" class="flex items-center gap-2 text-xs" :style="{ color: 'var(--ink-muted)' }">
          <AppProgress
            :value="overallProgress.answered"
            :max="overallProgress.total"
            size="sm"
            color="accent"
            class="w-28"
          />
          <span>{{ overallProgress.answered }}/{{ overallProgress.total }}</span>
        </div>
        <AppButton variant="ghost" size="sm" @click="endDiagnostic">Exit</AppButton>
      </div>
    </div>

    <!-- Main content -->
    <div class="flex-1 overflow-y-auto">

      <!-- Loading spinner -->
      <div v-if="stage === 'loading'" class="flex h-full items-center justify-center">
        <div class="w-8 h-8 border-2 rounded-full animate-spin"
          :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
      </div>

      <!-- Error -->
      <div v-else-if="stage === 'error'" class="flex h-full items-center justify-center p-8">
        <AppCard padding="lg" class="max-w-sm text-center">
          <p class="text-sm mb-4" :style="{ color: 'var(--warm)' }">{{ error }}</p>
          <AppButton variant="secondary" @click="router.push('/student/diagnostic')">Go Back</AppButton>
        </AppCard>
      </div>

      <!-- Phase intro -->
      <div v-else-if="stage === 'phase-intro' && currentPhase" class="flex h-full items-center justify-center p-8">
        <AppCard padding="lg" class="max-w-md text-center" glow="accent">
          <div
            class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
            :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }"
          >
            {{ currentPhase.phase_number === 1 ? '◇' : currentPhase.phase_number === 2 ? '◈' : '◉' }}
          </div>
          <AppBadge :color="(phaseColors[currentPhase.phase_type] as any) || 'accent'" size="xs" class="mb-3">
            Phase {{ currentPhase.phase_number }} of {{ battery?.phases.length }}
          </AppBadge>
          <h2 class="font-display text-xl font-bold mb-2" :style="{ color: 'var(--ink)' }">
            {{ currentPhase.phase_title }}
          </h2>
          <p class="text-sm mb-2" :style="{ color: 'var(--ink-secondary)' }">
            {{ currentPhase.question_count }} questions
            <span v-if="currentPhase.time_limit_seconds">
              · {{ Math.round(currentPhase.time_limit_seconds / 60) }} min limit
            </span>
          </p>
          <p class="text-xs mb-6" :style="{ color: 'var(--ink-muted)' }">
            <template v-if="currentPhase.condition_type === 'timed'">
              Answer as fast as you can — speed matters in this phase.
            </template>
            <template v-else-if="currentPhase.condition_type === 'pressure'">
              Stay focused — these questions test how you perform under pressure.
            </template>
            <template v-else>
              Answer honestly — there is no passing or failing here.
            </template>
          </p>
          <AppButton variant="primary" size="lg" @click="startPhase">
            Start Phase {{ currentPhase.phase_number }} →
          </AppButton>
        </AppCard>
      </div>

      <!-- Question -->
      <div v-else-if="stage === 'question' && currentItem" class="p-6 lg:p-8 max-w-3xl mx-auto w-full">
        <!-- Phase progress -->
        <div class="flex items-center justify-between mb-6">
          <span class="text-xs font-medium" :style="{ color: 'var(--ink-muted)' }">
            Question {{ currentItemIndex + 1 }} of {{ totalQuestionsInPhase }}
          </span>
          <AppBadge :color="(phaseColors[currentPhase?.phase_type ?? ''] as any) || 'muted'" size="xs">
            {{ conditionLabels[currentItem.condition_type] ?? currentItem.condition_type }}
          </AppBadge>
        </div>

        <QuestionCard
          ref="questionCardRef"
          :stem="currentItem.stem"
          :format="currentItem.question_format"
          :options="currentOptions.map(o => ({ id: o.id, label: o.label, text: o.text, is_correct: undefined, misconception_id: o.misconception_id, distractor_intent: o.distractor_intent }))"
          :question-number="currentItemIndex + 1"
          :total-questions="totalQuestionsInPhase"
          :show-timer="currentPhase?.condition_type === 'timed'"
          :timer-seconds="currentPhase?.time_limit_seconds ?? undefined"
          @answer="handleAnswer"
          @skip="handleSkip"
          @timeout="handleTimeout"
        />
      </div>

      <!-- All phases complete -->
      <div v-else-if="stage === 'all-complete'" class="flex h-full items-center justify-center p-8">
        <AppCard padding="lg" class="max-w-md text-center" glow="success">
          <div
            class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
            :style="{ backgroundColor: 'var(--success-light)', color: 'var(--accent)' }"
          >
            ✓
          </div>
          <h2 class="font-display text-xl font-bold mb-2" :style="{ color: 'var(--ink)' }">
            Diagnostic Complete!
          </h2>
          <p class="text-sm mb-6" :style="{ color: 'var(--ink-secondary)' }">
            Your coach is analysing your performance across all phases. Your personalised study plan is being built.
          </p>
          <AppButton variant="primary" size="lg" class="w-full mb-3" @click="goToReport">
            View My Report →
          </AppButton>
          <AppButton variant="ghost" size="sm" @click="router.push('/student/coach')">
            Go to Coach Hub
          </AppButton>
        </AppCard>
      </div>

    </div>
  </div>
</template>

