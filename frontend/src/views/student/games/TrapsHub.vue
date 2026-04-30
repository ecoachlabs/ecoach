<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import {
  getContrastPairProfile,
  getTrapReview,
  getTrapSessionSnapshot,
  listTrapMisconceptionReasons,
  listTrapsPairs,
  recordTrapConfusionReason,
  revealTrapUnmaskClue,
  startTrapsSession,
  submitTrapRound,
  type ContrastPairProfileDto,
  type ContrastPairSummaryDto,
  type TrapMisconceptionReasonDto,
  type TrapReviewDto,
  type TrapRoundCardDto,
  type TrapRoundResultDto,
  type TrapSessionSnapshotDto,
} from '@/ipc/games'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import {
  defaultTrapTimerSeconds,
  findActiveTrapRound,
  trapModeLabel,
  trapModeMeta,
  type TrapMode,
} from '@/utils/trapsView'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const starting = ref(false)
const submitting = ref(false)
const error = ref('')

const subjects = ref<SubjectDto[]>([])
const pairs = ref<ContrastPairSummaryDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const selectedPairId = ref<number | null>(null)
const selectedMode = ref<TrapMode>('difference_drill')

const pairProfile = ref<ContrastPairProfileDto | null>(null)
const trapReasons = ref<TrapMisconceptionReasonDto[]>([])

const liveSession = ref<TrapSessionSnapshotDto | null>(null)
const queuedSnapshot = ref<TrapSessionSnapshotDto | null>(null)
const liveReview = ref<TrapReviewDto | null>(null)
const lastResult = ref<TrapRoundResultDto | null>(null)
const selectedChoiceCode = ref<string | null>(null)
const recordedReasonCode = ref<string | null>(null)
const remainingSeconds = ref(0)

let roundStartedAt = 0
let timerHandle: ReturnType<typeof setInterval> | null = null

const selectedPair = computed(() =>
  pairs.value.find(pair => pair.pair_id === selectedPairId.value) ?? null,
)

const activeRound = computed<TrapRoundCardDto | null>(() => {
  if (!liveSession.value) return null
  return findActiveTrapRound({
    current_round_id: liveSession.value.current_round_id,
    rounds: liveSession.value.rounds,
  }) as TrapRoundCardDto | null
})

const modeCards = computed(() => {
  const available = new Set(selectedPair.value?.available_modes ?? Object.keys(trapModeMeta))
  return Object.entries(trapModeMeta).map(([mode, meta]) => ({
    key: mode as TrapMode,
    label: meta.label,
    desc: meta.description,
    difficulty: meta.difficulty,
    timerSeconds: meta.timerSeconds,
    available: available.has(mode),
    recommended: selectedPair.value?.recommended_mode === mode,
  }))
})

const phase = computed<'setup' | 'playing' | 'review'>(() => {
  if (liveReview.value) return 'review'
  if (liveSession.value) return 'playing'
  return 'setup'
})

const answeredRounds = computed(() => {
  if (!liveSession.value) return 0
  return liveSession.value.rounds.filter(round => round.status === 'answered' || round.status === 'completed').length
})

const liveProgress = computed(() => {
  if (!liveSession.value || liveSession.value.round_count === 0) return 0
  return Math.round((answeredRounds.value / liveSession.value.round_count) * 100)
})

const activeReasonChoices = computed(() =>
  trapReasons.value.filter(reason => reason.modes.length === 0 || reason.modes.includes(selectedMode.value)),
)

const pairHighlights = computed(() => pairProfile.value?.comparison_rows.slice(0, 4) ?? [])

function clearTimer() {
  if (timerHandle) {
    clearInterval(timerHandle)
    timerHandle = null
  }
}

function startRoundClock() {
  clearTimer()
  if (!liveSession.value || !activeRound.value || lastResult.value) return

  remainingSeconds.value = defaultTrapTimerSeconds(liveSession.value.mode)
  roundStartedAt = Date.now()
  timerHandle = setInterval(() => {
    if (submitting.value || lastResult.value) return
    remainingSeconds.value -= 1
    if (remainingSeconds.value <= 0) {
      clearTimer()
      void submitCurrentRound(true)
    }
  }, 1000)
}

async function loadPairs(subjectId: number) {
  if (!auth.currentAccount) return
  pairs.value = await listTrapsPairs(auth.currentAccount.id, subjectId, [])
  if (!pairs.value.length) {
    selectedPairId.value = null
    pairProfile.value = null
    return
  }

  if (!pairs.value.some(pair => pair.pair_id === selectedPairId.value)) {
    selectedPairId.value = pairs.value[0].pair_id
  }

  const chosenPair = pairs.value.find(pair => pair.pair_id === selectedPairId.value) ?? pairs.value[0]
  selectedPairId.value = chosenPair.pair_id
  if (!chosenPair.available_modes.includes(selectedMode.value)) {
    selectedMode.value = chosenPair.recommended_mode as TrapMode
  }
  await loadPairProfile(chosenPair.pair_id)
}

async function loadPairProfile(pairId: number) {
  if (!auth.currentAccount) return
  pairProfile.value = await getContrastPairProfile(auth.currentAccount.id, pairId)
}

async function loadReasonChoices() {
  trapReasons.value = await listTrapMisconceptionReasons(selectedMode.value).catch(() => [])
}

function applyLiveSnapshot(snapshot: TrapSessionSnapshotDto) {
  liveSession.value = snapshot
  queuedSnapshot.value = null
  liveReview.value = null
  lastResult.value = null
  selectedChoiceCode.value = null
  recordedReasonCode.value = null
  startRoundClock()
}

async function selectSubject(subjectId: number) {
  if (selectedSubjectId.value === subjectId) return
  selectedSubjectId.value = subjectId
  loading.value = true
  error.value = ''
  try {
    await loadPairs(subjectId)
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'The traps pairs could not be loaded.'
  } finally {
    loading.value = false
  }
}

async function selectPair(pairId: number) {
  if (selectedPairId.value === pairId) return
  selectedPairId.value = pairId
  error.value = ''
  try {
    const pair = pairs.value.find(item => item.pair_id === pairId)
    if (pair && !pair.available_modes.includes(selectedMode.value)) {
      selectedMode.value = pair.recommended_mode as TrapMode
      await loadReasonChoices()
    }
    await loadPairProfile(pairId)
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'The contrast pair details could not be loaded.'
  }
}

async function startSelectedMode() {
  if (!auth.currentAccount || !selectedSubjectId.value || !selectedPairId.value || starting.value) return

  starting.value = true
  error.value = ''
  clearTimer()

  try {
    const snapshot = await startTrapsSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: [],
      pair_id: selectedPairId.value,
      mode: selectedMode.value,
      round_count: 6,
      timer_seconds: defaultTrapTimerSeconds(selectedMode.value),
    })
    applyLiveSnapshot(snapshot)
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'That traps run could not be started.'
  } finally {
    starting.value = false
  }
}

async function revealClue() {
  if (!liveSession.value || !activeRound.value || activeRound.value.mode !== 'unmask') return
  try {
    const nextRound = await revealTrapUnmaskClue(liveSession.value.session_id, activeRound.value.id)
    liveSession.value = {
      ...liveSession.value,
      rounds: liveSession.value.rounds.map(round => round.id === nextRound.id ? nextRound : round),
    }
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'The next clue could not be revealed.'
  }
}

async function submitCurrentRound(timedOut = false) {
  if (!liveSession.value || !activeRound.value || submitting.value) return
  if (!timedOut && !selectedChoiceCode.value) return

  submitting.value = true
  error.value = ''
  clearTimer()

  try {
    const result = await submitTrapRound({
      game_session_id: liveSession.value.session_id,
      round_id: activeRound.value.id,
      selected_choice_code: timedOut ? null : selectedChoiceCode.value,
      response_time_ms: Math.max(Date.now() - roundStartedAt, 0),
      timed_out: timedOut,
    })
    lastResult.value = result
    recordedReasonCode.value = null

    if (result.session_complete) {
      liveReview.value = await getTrapReview(liveSession.value.session_id)
      liveSession.value = null
      queuedSnapshot.value = null
      return
    }

    queuedSnapshot.value = await getTrapSessionSnapshot(liveSession.value.session_id)
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'That round could not be submitted.'
    startRoundClock()
  } finally {
    submitting.value = false
  }
}

async function continueToNextRound() {
  if (!queuedSnapshot.value) return
  applyLiveSnapshot(queuedSnapshot.value)
}

async function recordReason(reasonCode: string) {
  if (!lastResult.value || recordedReasonCode.value === reasonCode) return

  try {
    await recordTrapConfusionReason({
      round_id: lastResult.value.round_id,
      reason_code: reasonCode,
      reason_text: null,
    })
    recordedReasonCode.value = reasonCode
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'That confusion reason could not be saved.'
  }
}

function resetToSetup() {
  clearTimer()
  liveSession.value = null
  queuedSnapshot.value = null
  liveReview.value = null
  lastResult.value = null
  selectedChoiceCode.value = null
  recordedReasonCode.value = null
  remainingSeconds.value = 0
}

function retryRecommendedMode() {
  if (!liveReview.value) return
  selectedMode.value = liveReview.value.recommended_next_mode as TrapMode
  liveReview.value = null
  void startSelectedMode()
}

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    subjects.value = await listSubjects()
    const firstSubject = subjects.value[0]
    selectedSubjectId.value = firstSubject?.id ?? null
    if (firstSubject) {
      await loadPairs(firstSubject.id)
    }
    await loadReasonChoices()
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : 'The traps mode could not be loaded.'
  } finally {
    loading.value = false
  }
})

watch(selectedMode, () => {
  void loadReasonChoices()
})

onBeforeUnmount(() => {
  clearTimer()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <PageHeader
      title="Concept Traps"
      subtitle="Real contrast pairs, persisted trap rounds, and review loops that feed the misconception engine."
      back-to="/student/games"
    />

    <div v-if="error" class="mb-4 rounded-xl px-4 py-3 text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 4" :key="i" class="h-28 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else>
      <div class="mb-6 flex flex-wrap items-center gap-3">
        <span class="text-xs font-semibold uppercase" :style="{ color: 'var(--ink-muted)' }">Subject:</span>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
            :style="{
              backgroundColor: selectedSubjectId === subject.id ? 'var(--accent)' : 'var(--border-soft)',
              color: selectedSubjectId === subject.id ? 'white' : 'var(--ink-secondary)',
            }"
            @click="selectSubject(subject.id)"
          >
            {{ subject.name }}
          </button>
        </div>
      </div>

      <div class="grid gap-6 xl:grid-cols-[minmax(0,1.25fr)_minmax(320px,0.75fr)]">
        <div class="space-y-6">
          <AppCard padding="md">
            <div class="mb-4 flex items-center justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Available Pairs</p>
                <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">Pick a live contrast pair, then launch one of its available trap modes.</p>
              </div>
              <AppBadge v-if="selectedPair" color="gold" size="xs">
                Recommended {{ trapModeLabel(selectedPair.recommended_mode) }}
              </AppBadge>
            </div>

            <div v-if="pairs.length" class="grid gap-2 md:grid-cols-2">
              <button
                v-for="pair in pairs"
                :key="pair.pair_id"
                class="w-full rounded-xl border px-4 py-3 text-left transition-all"
                :style="{
                  borderColor: selectedPairId === pair.pair_id ? 'var(--accent)' : 'var(--border-soft)',
                  backgroundColor: selectedPairId === pair.pair_id ? 'var(--accent-glow)' : 'var(--surface)',
                }"
                @click="selectPair(pair.pair_id)"
              >
                <div class="flex items-center justify-between gap-3">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ pair.title }}</p>
                  <AppBadge color="muted" size="xs">{{ Math.round(pair.confusion_score / 100) }}% confusion</AppBadge>
                </div>
                <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ pair.left_label }} vs {{ pair.right_label }}</p>
                <p v-if="pair.summary_text" class="mt-2 text-xs" :style="{ color: 'var(--ink-secondary)' }">{{ pair.summary_text }}</p>
              </button>
            </div>

            <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No contrast pairs are ready for this subject yet.</p>
          </AppCard>

          <AppCard padding="md">
            <div class="mb-4 flex items-center justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Trap Modes</p>
                <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">The mode cards below come from the real trap engine timing and availability rules.</p>
              </div>
              <AppButton :disabled="!selectedPairId || starting" @click="startSelectedMode">
                {{ starting ? 'Starting...' : 'Start Selected Mode' }}
              </AppButton>
            </div>

            <div class="space-y-3">
              <AppCard
                v-for="mode in modeCards"
                :key="mode.key"
                hover
                padding="md"
                :class="selectedMode === mode.key ? 'ring-2 ring-[var(--accent)]' : ''"
                @click="mode.available ? (selectedMode = mode.key) : null"
              >
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-xl flex items-center justify-center text-[11px] font-black" :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">
                    {{ mode.label.slice(0, 2).toUpperCase() }}
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ mode.label }}</p>
                      <AppBadge color="muted" size="xs">{{ mode.difficulty }}</AppBadge>
                      <AppBadge v-if="mode.recommended" color="gold" size="xs">recommended</AppBadge>
                    </div>
                    <p class="text-[11px]" :style="{ color: mode.available ? 'var(--ink-muted)' : 'var(--warm)' }">
                      {{ mode.available ? mode.desc : 'Not available for the selected pair yet.' }}
                    </p>
                  </div>
                  <div class="text-right">
                    <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">{{ mode.timerSeconds }}s</p>
                    <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">per round</p>
                  </div>
                </div>
              </AppCard>
            </div>
          </AppCard>

          <AppCard v-if="phase === 'playing' && liveSession && activeRound" padding="lg">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div>
                <div class="flex items-center gap-2">
                  <h2 class="text-lg font-semibold" :style="{ color: 'var(--ink)' }">{{ liveSession.pair_title }}</h2>
                  <AppBadge color="accent" size="xs">{{ trapModeLabel(liveSession.mode) }}</AppBadge>
                </div>
                <p class="mt-1 text-sm" :style="{ color: 'var(--ink-muted)' }">{{ liveSession.left_label }} vs {{ liveSession.right_label }}</p>
              </div>
              <div class="flex items-center gap-3">
                <div class="text-right">
                  <p class="text-xl font-black" :style="{ color: 'var(--accent)' }">{{ liveSession.score }}</p>
                  <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">score</p>
                </div>
                <div class="text-right">
                  <p class="text-xl font-black" :style="{ color: remainingSeconds <= 2 ? 'var(--warm)' : 'var(--gold)' }">{{ remainingSeconds }}s</p>
                  <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">timer</p>
                </div>
              </div>
            </div>

            <AppProgress :value="liveProgress" class="mb-5" />

            <div class="mb-5 rounded-2xl border p-5" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }">
              <div class="mb-3 flex items-center justify-between gap-3">
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                  Round {{ activeRound.round_number }} / {{ liveSession.round_count }}
                </p>
                <AppBadge color="muted" size="xs">{{ activeRound.lane }}</AppBadge>
              </div>
              <p class="text-lg font-semibold" :style="{ color: 'var(--ink)' }">{{ activeRound.prompt_text }}</p>
              <p v-if="liveSession.summary_text" class="mt-3 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ liveSession.summary_text }}</p>
            </div>

            <div class="grid gap-2 md:grid-cols-2">
              <button
                v-for="option in activeRound.answer_options"
                :key="option.code"
                class="rounded-xl border px-4 py-3 text-left transition-all"
                :style="{
                  borderColor: selectedChoiceCode === option.code ? 'var(--accent)' : 'var(--border-soft)',
                  backgroundColor: selectedChoiceCode === option.code ? 'var(--accent-glow)' : 'var(--surface)',
                }"
                @click="selectedChoiceCode = option.code"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ option.label }}</p>
                <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ option.code.replace(/_/g, ' ') }}</p>
              </button>
            </div>

            <div class="mt-5 flex flex-wrap items-center gap-3">
              <AppButton
                v-if="activeRound.mode === 'unmask' && activeRound.reveal_count < activeRound.max_reveal_count"
                variant="secondary"
                @click="revealClue"
              >
                Reveal Clue {{ activeRound.reveal_count + 1 }}
              </AppButton>
              <AppButton :disabled="!selectedChoiceCode || submitting" @click="submitCurrentRound(false)">
                {{ submitting ? 'Submitting...' : 'Submit Round' }}
              </AppButton>
              <AppButton variant="secondary" @click="resetToSetup">Exit Run</AppButton>
            </div>
          </AppCard>

          <AppCard v-if="phase === 'playing' && lastResult" padding="md" glow="gold">
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: lastResult.is_correct ? 'var(--accent)' : 'var(--warm)' }">
                  {{ lastResult.is_correct ? 'Correct discrimination' : 'Trap triggered' }}
                </p>
                <p class="mt-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">{{ lastResult.explanation_text }}</p>
              </div>
              <div class="text-right">
                <p class="text-xl font-black" :style="{ color: 'var(--ink)' }">{{ lastResult.new_score }}</p>
                <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">new score</p>
              </div>
            </div>

            <div class="mt-4 rounded-xl border px-4 py-3 text-sm" :style="{ borderColor: 'var(--border-soft)' }">
              <p :style="{ color: 'var(--ink)' }"><strong>Correct:</strong> {{ lastResult.correct_choice_label }}</p>
              <p v-if="lastResult.selected_choice_label" class="mt-1" :style="{ color: 'var(--ink-secondary)' }">
                <strong>You chose:</strong> {{ lastResult.selected_choice_label }}
              </p>
              <p class="mt-1" :style="{ color: 'var(--ink-muted)' }"><strong>Signal:</strong> {{ lastResult.confusion_signal.replace(/_/g, ' ') }}</p>
            </div>

            <div v-if="!lastResult.is_correct && activeReasonChoices.length" class="mt-4">
              <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Why did that trap you?</p>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="reason in activeReasonChoices"
                  :key="reason.code"
                  class="rounded-full px-3 py-1.5 text-xs font-semibold transition-all"
                  :style="{
                    backgroundColor: recordedReasonCode === reason.code ? 'var(--accent)' : 'var(--paper)',
                    color: recordedReasonCode === reason.code ? 'white' : 'var(--ink-secondary)',
                  }"
                  @click="recordReason(reason.code)"
                >
                  {{ reason.label }}
                </button>
              </div>
            </div>

            <div class="mt-4 flex gap-3">
              <AppButton :disabled="!queuedSnapshot" @click="continueToNextRound">Continue</AppButton>
              <AppButton variant="secondary" @click="resetToSetup">Back to Setup</AppButton>
            </div>
          </AppCard>

          <AppCard v-if="phase === 'review' && liveReview" padding="lg" glow="gold">
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--gold)' }">Trap Review</p>
                <h2 class="text-lg font-semibold" :style="{ color: 'var(--ink)' }">{{ liveReview.pair_title }}</h2>
                <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">{{ trapModeLabel(liveReview.mode) }}</p>
              </div>
              <div class="grid grid-cols-3 gap-3 text-center">
                <div>
                  <p class="text-xl font-black" :style="{ color: 'var(--accent)' }">{{ Math.round(liveReview.accuracy_bp / 100) }}%</p>
                  <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">accuracy</p>
                </div>
                <div>
                  <p class="text-xl font-black" :style="{ color: 'var(--gold)' }">{{ Math.round(liveReview.confusion_score / 100) }}%</p>
                  <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">confusion</p>
                </div>
                <div>
                  <p class="text-xl font-black" :style="{ color: 'var(--ink)' }">{{ liveReview.score }}</p>
                  <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">score</p>
                </div>
              </div>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
              <AppCard padding="md">
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Weakest Lane</p>
                <p class="mt-2 text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ liveReview.weakest_lane ?? 'None recorded' }}</p>
                <p class="mt-3 text-xs" :style="{ color: 'var(--ink-secondary)' }">
                  {{ liveReview.dominant_confusion_reason ? `Most common reason: ${liveReview.dominant_confusion_reason.replace(/_/g, ' ')}` : 'No dominant confusion reason has been recorded yet.' }}
                </p>
              </AppCard>
              <AppCard padding="md">
                <p class="text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Next Recommendation</p>
                <p class="mt-2 text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ trapModeLabel(liveReview.recommended_next_mode) }}</p>
                <ul class="mt-3 space-y-2 text-xs" :style="{ color: 'var(--ink-secondary)' }">
                  <li v-for="action in liveReview.remediation_actions" :key="action">{{ action }}</li>
                </ul>
              </AppCard>
            </div>

            <div class="mt-5 space-y-2">
              <div
                v-for="round in liveReview.rounds"
                :key="round.round_id"
                class="rounded-xl border px-4 py-3"
                :style="{ borderColor: 'var(--border-soft)' }"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ round.prompt_text }}</p>
                    <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">
                      {{ round.selected_choice_label ?? 'Timed out' }} -> {{ round.correct_choice_label }}
                    </p>
                    <p class="mt-2 text-xs" :style="{ color: 'var(--ink-secondary)' }">{{ round.explanation_text }}</p>
                  </div>
                  <AppBadge :color="round.is_correct ? 'success' : 'warm'" size="xs">
                    {{ round.is_correct ? 'correct' : 'missed' }}
                  </AppBadge>
                </div>
              </div>
            </div>

            <div class="mt-5 flex flex-wrap gap-3">
              <AppButton @click="retryRecommendedMode">
                Retry Recommended Mode
              </AppButton>
              <AppButton variant="secondary" @click="resetToSetup">Back to Setup</AppButton>
              <AppButton variant="secondary" @click="router.push('/student/mistakes')">Mistake Lab</AppButton>
            </div>
          </AppCard>
        </div>

        <div class="space-y-6">
          <AppCard v-if="selectedPair" padding="md">
            <p class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Pair Snapshot</p>
            <p class="text-lg font-semibold" :style="{ color: 'var(--ink)' }">{{ selectedPair.title }}</p>
            <p class="mt-1 text-sm" :style="{ color: 'var(--ink-muted)' }">{{ selectedPair.left_label }} vs {{ selectedPair.right_label }}</p>
            <div class="mt-4 grid grid-cols-2 gap-3">
              <div class="rounded-xl border px-3 py-2" :style="{ borderColor: 'var(--border-soft)' }">
                <p class="text-xl font-black" :style="{ color: 'var(--gold)' }">{{ Math.round(selectedPair.confusion_score / 100) }}%</p>
                <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">confusion</p>
              </div>
              <div class="rounded-xl border px-3 py-2" :style="{ borderColor: 'var(--border-soft)' }">
                <p class="text-xl font-black" :style="{ color: 'var(--accent)' }">{{ Math.round(selectedPair.last_accuracy_bp / 100) }}%</p>
                <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">last accuracy</p>
              </div>
            </div>
          </AppCard>

          <AppCard v-if="pairProfile" padding="md">
            <p class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Decisive Clues</p>
            <div v-if="pairHighlights.length" class="space-y-3">
              <div
                v-for="row in pairHighlights"
                :key="row.id"
                class="rounded-xl border px-4 py-3"
                :style="{ borderColor: 'var(--border-soft)' }"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ row.compare_label }}</p>
                <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ row.left_value }} vs {{ row.right_value }}</p>
                <p v-if="row.decisive_clue" class="mt-2 text-xs" :style="{ color: 'var(--accent)' }">{{ row.decisive_clue }}</p>
              </div>
            </div>
            <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">The pair profile does not expose comparison rows yet.</p>
          </AppCard>

          <AppCard v-if="pairProfile?.common_confusions?.length" padding="md">
            <p class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Common Confusions</p>
            <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
              <li v-for="confusion in pairProfile.common_confusions.slice(0, 5)" :key="confusion">{{ confusion }}</li>
            </ul>
          </AppCard>
        </div>
      </div>
    </template>
  </div>
</template>
