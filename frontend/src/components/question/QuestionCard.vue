<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import McqQuestion from './formats/McqQuestion.vue'
import MultiMcqQuestion from './formats/MultiMcqQuestion.vue'
import PastPaperFillBlank from './formats/PastPaperFillBlank.vue'
import MathText from './MathText.vue'
import QuestionTimer from './QuestionTimer.vue'
import QuestionAssetGallery from './QuestionAssetGallery.vue'

const props = defineProps<{
  stem: string
  format: string
  options: { id: number; label: string; text: string; is_correct?: boolean; misconception_id?: number | null; distractor_intent?: string | null }[]
  difficulty?: number
  estimatedSeconds?: number
  showTimer?: boolean
  timerSeconds?: number
  /** Position-in-session props are accepted for backward compatibility
   *  (SessionPlayer, MockHall, DiagnosticSession still pass them) but
   *  the redesigned card doesn't render them — the parent shell shows
   *  the position strip instead. */
  questionNumber?: number
  totalQuestions?: number
  initialFlagged?: boolean
  /** Optional — when present, fetch attached images for stem/options
   *  and render them inline. Past-paper sections always pass this. */
  questionId?: number
}>()

const emit = defineEmits<{
  /**
   * Fires the student's answer upward. In test mode this is a direct
   * auto-submit — single MCQ fires on option click, multi-MCQ and
   * fill-blank fire on the explicit Submit button since they have
   * multi-step selection. `confidence` is carried for attempt history
   * but defaults to null (no confidence capture in the test flow).
   */
  answer: [
    optionId: number,
    confidence: string | null,
    responseTimeMs: number,
    multiOptionIds?: number[] | null,
    blankAnswers?: string[] | null,
  ]
  flag: [flagged: boolean]
  skip: []
  timeout: []
}>()

// ── Rendering-mode detection ──────────────────────────────────────
//
// We derive the mode from the props so SessionView (and any other
// surface) doesn't need to plumb a new enum through. Fill-blank is
// detected by [[N]] markers in the stem; multi-correct MCQ by >1
// options flagged is_correct. The heuristic covers both past-paper
// authoring (where pedagogic_function is set) and hand-seeded data
// where only the option flags exist.
type Mode = 'mcq' | 'mcq_multi' | 'fill_blank'

const mode = computed<Mode>(() => {
  if (/\[\[\d+\]\]/.test(props.stem)) return 'fill_blank'
  const correctCount = props.options.filter(o => o.is_correct === true).length
  if (correctCount > 1) return 'mcq_multi'
  return 'mcq'
})

// ── Per-mode state ────────────────────────────────────────────────
// Single MCQ: one option id. Selected == submitted (auto-advance at
// the parent). Multi-MCQ and fill-blank are multi-step so they keep
// an explicit Submit button.
const selectedOption = ref<number | null>(null)
const selectedOptions = ref<number[]>([])
const blankValues = ref<Record<string, string>>({})

// `answered` is a one-shot latch to prevent double-click from firing
// two answer emits before the parent has had a chance to swap the
// current question (the parent's :key re-mount will clear it).
const answered = ref(false)
const startTime = ref(Date.now())
const flagged = ref(!!props.initialFlagged)
const timerResetKey = ref(0)

watch(
  () => props.initialFlagged,
  (value) => { flagged.value = !!value },
  { immediate: true },
)

// ── Selection handlers ────────────────────────────────────────────
// Single MCQ: clicking an option IS the answer. Emit immediately.
function selectOption(id: number) {
  if (answered.value) return
  selectedOption.value = id
  answered.value = true
  emit('answer', id, null, Date.now() - startTime.value, null, null)
}

function toggleOption(id: number) {
  if (answered.value) return
  selectedOptions.value = selectedOptions.value.includes(id)
    ? selectedOptions.value.filter(x => x !== id)
    : [...selectedOptions.value, id]
}

function updateBlank(blank: string, value: string) {
  if (answered.value) return
  blankValues.value = { ...blankValues.value, [blank]: value }
}

// ── Submit-ready gate (multi-MCQ / fill-blank only) ──────────────
const canSubmit = computed(() => {
  switch (mode.value) {
    case 'mcq':        return false
    case 'mcq_multi':  return selectedOptions.value.length > 0
    case 'fill_blank': {
      const found = Array.from(props.stem.matchAll(/\[\[(\d+)\]\]/g)).map(m => m[1])
      const unique = Array.from(new Set(found))
      return unique.length > 0 && unique.every(i => (blankValues.value[i] ?? '').trim().length > 0)
    }
  }
})

// ── Submission (multi-MCQ + fill-blank) ──────────────────────────
function submitAnswer() {
  if (!canSubmit.value || answered.value) return

  let optionIdToSubmit: number | null = null
  let multiIds: number[] | null = null
  let blanks: string[] | null = null

  if (mode.value === 'mcq_multi') {
    optionIdToSubmit = selectedOptions.value[0] ?? null
    multiIds = [...selectedOptions.value]
  } else if (mode.value === 'fill_blank') {
    optionIdToSubmit = props.options[0]?.id ?? null
    const found = Array.from(props.stem.matchAll(/\[\[(\d+)\]\]/g)).map(m => m[1])
    const ordered = Array.from(new Set(found)).sort((a, b) => Number(a) - Number(b))
    blanks = ordered.map(i => blankValues.value[i] ?? '')
  }

  if (optionIdToSubmit === null) return
  answered.value = true
  emit('answer', optionIdToSubmit, null, Date.now() - startTime.value, multiIds, blanks)
}

function skipQuestion() {
  if (answered.value) return
  emit('skip')
}

function handleTimeout() {
  if (answered.value) return
  answered.value = true
  emit('timeout')
}

// Legacy reset/setFlagged retained for callers that still hold a ref
// and may call them (SessionPlayer, MockHall, DiagnosticSession). The
// redesigned SessionView re-mounts the card on each question via
// :key, so reset() is a no-op in that flow.
function reset() {
  selectedOption.value = null
  selectedOptions.value = []
  blankValues.value = {}
  answered.value = false
  startTime.value = Date.now()
  flagged.value = !!props.initialFlagged
  timerResetKey.value++
}
function setFlagged(value: boolean) { flagged.value = value }
defineExpose({ reset, setFlagged })
</script>

<template>
  <div class="qc-shell">

    <!-- Timer for timed mode — sits above the card, paper-level. -->
    <div v-if="showTimer && timerSeconds" class="qc-timer">
      <QuestionTimer
        :key="timerResetKey"
        :total-seconds="timerSeconds"
        :running="!answered"
        variant="pressure"
        @timeout="handleTimeout"
      />
    </div>

    <!-- ── QUESTION CARD ─────────────────────────────────────────
         The stem + options live together on a white surface. The
         card spans the full column width (no max-width constraint)
         so the stem spreads. Four industrial corner crop-marks
         give it architectural character without violating Nothing's
         "no shadows, no blur" rule — character through registration
         marks, not depth. -->
    <article class="qc-card" :data-qno="questionNumber ? String(questionNumber).padStart(2, '0') : undefined">
      <span class="qc-corner qc-corner--tl" aria-hidden="true" />
      <span class="qc-corner qc-corner--tr" aria-hidden="true" />
      <span class="qc-corner qc-corner--bl" aria-hidden="true" />
      <span class="qc-corner qc-corner--br" aria-hidden="true" />

      <!-- Meta strip inside the card: difficulty dot + flag toggle. -->
      <div class="qc-meta">
        <span v-if="difficulty" class="qc-diff">
          <span class="qc-diff-dot" :data-level="difficulty > 7000 ? 'hard' : difficulty > 4000 ? 'medium' : 'easy'" aria-hidden="true" />
          <span class="qc-diff-text">
            {{ difficulty > 7000 ? 'HARD' : difficulty > 4000 ? 'MEDIUM' : 'EASY' }}
          </span>
        </span>
        <button
          type="button"
          class="qc-flag"
          :class="{ 'qc-flag--on': flagged }"
          @click="flagged = !flagged; $emit('flag', flagged)"
          :aria-pressed="flagged"
          :title="flagged ? 'Unflag' : 'Flag for review'"
        >
          {{ flagged ? 'FLAGGED' : 'FLAG' }}
        </button>
      </div>

      <!-- Stem — big, left-aligned, spreads to the card's inner width. -->
      <div class="qc-stem">
        <div class="qc-stem-text">
          <MathText :text="stem" />
        </div>
        <QuestionAssetGallery
          v-if="questionId != null"
          :question-id="questionId"
          scope="stem"
        />
      </div>

      <!-- Answer surface dispatched by mode. -->
      <div class="qc-answer">
        <PastPaperFillBlank
          v-if="mode === 'fill_blank'"
          :stem="stem"
          :options="options"
          :values="blankValues"
          :answered="answered"
          @input="updateBlank"
        />
        <MultiMcqQuestion
          v-else-if="mode === 'mcq_multi' && options.length > 0"
          :options="options"
          :selected="selectedOptions"
          :answered="answered"
          :question-id="questionId"
          @toggle="toggleOption"
        />
        <McqQuestion
          v-else-if="options.length > 0"
          :options="options"
          :selected="selectedOption"
          :answered="answered"
          :question-id="questionId"
          @select="selectOption"
        />
        <p v-else class="qc-mono-tag">[ ANSWER CHOICES NOT AVAILABLE ]</p>
      </div>
    </article>

    <!-- ── ACTIONS (outside the card, paper-level) ───────────────
         Single-MCQ auto-advances on option click so no Submit is
         shown. Multi-MCQ and fill-blank keep an explicit Submit.
         Skip is always available. -->
    <div v-if="!answered" class="qc-actions">
      <button
        type="button"
        class="qc-action qc-action--ghost"
        @click="skipQuestion"
      >
        SKIP
      </button>
      <button
        v-if="mode !== 'mcq'"
        type="button"
        class="qc-action qc-action--primary"
        :disabled="!canSubmit"
        @click="submitAnswer"
      >
        SUBMIT →
      </button>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   QuestionCard — stem + options ride on a white card with industrial
   corner crop-marks. Actions live outside the card, at paper level.
   ═══════════════════════════════════════════════════════════════════ */
.qc-shell {
  width: 100%;
  display: grid;
  gap: clamp(20px, 3vh, 32px);
  color: var(--ink);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}

/* ─── card ─── */
.qc-card {
  position: relative;
  background: var(--surface, #ffffff);
  padding: clamp(28px, 3.6vw, 56px) clamp(24px, 3vw, 48px);
  display: grid;
  gap: clamp(24px, 3.5vh, 40px);
  overflow: hidden;
}

/* Ghosted watermark numeral in the top-right of the card — the
   question index printed huge in Space Mono at ~4.5% ink opacity.
   Pure data-as-beauty: you only notice it when you squint, but it
   tells the card it carries a number. */
.qc-card::before {
  content: attr(data-qno);
  position: absolute;
  top: clamp(-10px, -0.8vw, -4px);
  right: clamp(16px, 2.4vw, 40px);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: clamp(120px, 16vw, 260px);
  font-weight: 700;
  color: var(--ink);
  opacity: 0.06;
  line-height: 0.9;
  letter-spacing: -0.05em;
  pointer-events: none;
  user-select: none;
  z-index: 0;
}
.qc-card > *:not(.qc-corner) { position: relative; z-index: 1; }

/* Four industrial corner crop-marks. L-shapes, 2px ink strokes, sitting
   inside the card's padded edge. Reads like a registration mark on a
   printed page — the card feels fabricated, not floating. */
.qc-corner {
  position: absolute;
  width: 22px;
  height: 22px;
  border-color: var(--ink);
  border-style: solid;
  border-width: 0;
  pointer-events: none;
  z-index: 2;
}
.qc-corner--tl { top: 10px;    left: 10px;    border-top-width: 2px; border-left-width: 2px; }
.qc-corner--tr { top: 10px;    right: 10px;   border-top-width: 2px; border-right-width: 2px; }
.qc-corner--bl { bottom: 10px; left: 10px;    border-bottom-width: 2px; border-left-width: 2px; }
.qc-corner--br { bottom: 10px; right: 10px;   border-bottom-width: 2px; border-right-width: 2px; }

/* ─── meta strip ─── */
.qc-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.qc-diff {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}
.qc-diff-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--ink);
  opacity: 0.3;
}
.qc-diff-dot[data-level="medium"] { opacity: 0.6; }
.qc-diff-dot[data-level="hard"]   { opacity: 1; }
.qc-diff-text {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.qc-flag {
  background: transparent;
  border: none;
  padding: 6px 0;
  color: var(--ink-muted);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: color 140ms ease;
}
.qc-flag:hover { color: var(--ink); }
.qc-flag--on  { color: var(--ink); }

.qc-timer { margin: 4px 0; }

/* ─── stem (the primary element) — spreads to the card's inner width ─── */
.qc-stem {
  display: grid;
  gap: 24px;
}
.qc-stem-text {
  margin: 0;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 400;
  font-size: clamp(26px, 2.8vw, 40px);
  line-height: 1.24;
  letter-spacing: -0.018em;
  color: var(--ink);
}
/* Give KaTeX display math room to breathe inside the stem. */
.qc-stem-text :deep(.katex-display) { margin: 0.8em 0; text-align: left; }
/* Basic inline HTML the authors use — keep typography coherent. */
.qc-stem-text :deep(sup) { font-size: 0.7em; vertical-align: super; line-height: 0; }
.qc-stem-text :deep(sub) { font-size: 0.7em; vertical-align: sub;   line-height: 0; }
.qc-stem-text :deep(em)  { font-style: italic; }
.qc-stem-text :deep(strong) { font-weight: 600; }

/* ─── answer surface wrapper (styling lives in the format components) ─── */
.qc-answer { width: 100%; }

.qc-mono-tag {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
}

/* ─── actions ─── */
.qc-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  flex-wrap: wrap;
  margin-top: 8px;
}
.qc-action {
  background: transparent;
  border: none;
  padding: 14px 22px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: color 140ms ease, background 140ms ease, transform 200ms cubic-bezier(0.16, 1, 0.3, 1);
  border-radius: 999px;
}
.qc-action--ghost {
  color: var(--ink-muted);
}
.qc-action--ghost:hover {
  color: var(--ink);
}
.qc-action--primary {
  background: var(--ink);
  color: var(--paper);
  padding: 14px 26px;
}
.qc-action--primary:hover {
  transform: translateX(2px);
}

@media (max-width: 760px) {
  .qc-actions { justify-content: stretch; }
  .qc-action { flex: 1 1 auto; text-align: center; }
}
</style>
