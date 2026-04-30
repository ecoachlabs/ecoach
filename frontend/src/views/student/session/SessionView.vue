<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  listSessionQuestions,
  submitAttempt,
  type SubmitAttemptInput,
  type SessionQuestionDto,
  type AttemptResultDto,
} from '@/ipc/questions'
import {
  completeSession as completeSessionPlain,
  completeSessionWithPipeline,
  flagSessionItem,
  recordSessionPresenceEvent,
  type SessionCompletionResultDto,
  type SessionSummaryDto,
} from '@/ipc/sessions'
import QuestionCard from '@/components/question/QuestionCard.vue'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

// Nothing-style type stack — Space Grotesk body + Space Mono metadata.
onMounted(() => {
  const href =
    'https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Space+Mono:wght@400;700&display=swap'
  if (!document.head.querySelector(`link[href="${href}"]`)) {
    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = href
    document.head.appendChild(link)
  }
})

const sessionId = computed(() => Number(route.params.id))
const isTimedSession = computed(() => route.query.timed === '1' || route.query.timed === 'true')

// Session data
const questions = ref<SessionQuestionDto[]>([])
const currentIndex = ref(0)
const loading = ref(true)
const error = ref('')

// Per-question timing + in-flight tracking. No per-question feedback
// state — this is a true test: no reveal between questions.
const startTime = ref(Date.now())
const presentedItemIds = new Set<number>()
const pendingSubmissionTasks = new Map<number, Promise<AttemptResultDto>>()
const unsavedSubmissionInputs = new Map<number, SubmitAttemptInput>()

type SessionCompletionLike = SessionSummaryDto | SessionCompletionResultDto

function isSessionCompletionResult(
  summary: SessionCompletionLike,
): summary is SessionCompletionResultDto {
  return 'next_action_type' in summary
}

// Computed
const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)
const totalQuestions = computed(() => questions.value.length)
const answeredCount = computed(() => questions.value.filter((q) => q.is_answered).length)

// Strip LaTeX delimiters, LaTeX commands, and HTML tags so the right-
// hand preview stack shows a clean plaintext excerpt of each stem.
function toPreview(raw: string | null | undefined): string {
  if (!raw) return ''
  return raw
    .replace(/\$\$([\s\S]*?)\$\$/g, '$1')
    .replace(/\$([^$]*?)\$/g, '$1')
    .replace(/\\\(([\s\S]*?)\\\)/g, '$1')
    .replace(/\\\[([\s\S]*?)\\\]/g, '$1')
    .replace(/\\[a-zA-Z]+\s*(\{[^}]*\})?/g, '')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&#?\w+;/g, '')
    .replace(/\s+/g, ' ')
    .trim()
}

// â”€â”€ Lifecycle â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

watch(
  currentQuestion,
  (question) => {
    if (!question || presentedItemIds.has(question.item_id)) return
    presentedItemIds.add(question.item_id)
    void recordSessionPresenceEvent(sessionId.value, {
      event_type: 'question_presented',
      occurred_at: new Date().toISOString(),
      metadata_json: {
        item_id: question.item_id,
        question_id: question.question_id,
        source: 'session_view',
      },
    }).catch(() => {
      presentedItemIds.delete(question.item_id)
    })
  },
  { immediate: true },
)

// â”€â”€ Actions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

// ── Answer + auto-advance ─────────────────────────────────────────
//
// In test mode there is NO per-question reveal. A click on an option
// submits the attempt in the background and moves straight to the next
// unanswered question. The student reviews everything on the debrief
// page after finishing or quitting.
async function handleAnswer(
  optionId: number,
  confidence: string | null,
  responseTimeMs: number,
  multiOptionIds?: number[] | null,
  blankAnswers?: string[] | null,
) {
  const q = currentQuestion.value
  if (!q || !auth.currentAccount) return

  // Mark answered locally so the question stack updates and we skip
  // past it on the next advance. The attempt submit fires fire-and-
  // forget; `completeSession` flushes any in-flight requests.
  questions.value[currentIndex.value] = { ...q, is_answered: true }

  const input: SubmitAttemptInput = {
    student_id: auth.currentAccount.id,
    session_id: sessionId.value,
    session_item_id: q.item_id,
    question_id: q.question_id,
    selected_option_id: optionId,
    response_time_ms: responseTimeMs,
    confidence_level: confidence,
    hint_count: 0,
    changed_answer_count: 0,
    was_timed: isTimedSession.value,
    selected_option_ids: multiOptionIds ?? null,
    blank_answers: blankAnswers ?? null,
  }
  unsavedSubmissionInputs.set(q.item_id, input)
  const submission = submitAttempt(input)
  pendingSubmissionTasks.set(q.item_id, submission)
  void submission
    .then(() => { unsavedSubmissionInputs.delete(q.item_id) })
    .catch(() => { /* retained for flush at completion */ })
    .finally(() => { pendingSubmissionTasks.delete(q.item_id) })

  advance()
}

// Jump to the next unanswered question, or complete the session when
// none remain. Also called by skip / timeout handlers.
function advance() {
  let next = -1
  for (let i = currentIndex.value + 1; i < questions.value.length; i++) {
    if (!questions.value[i].is_answered) { next = i; break }
  }
  if (next === -1) {
    for (let i = 0; i < currentIndex.value; i++) {
      if (!questions.value[i].is_answered) { next = i; break }
    }
  }
  if (next === -1) {
    completeSession()
    return
  }
  currentIndex.value = next
  startTime.value = Date.now()
}

// Jump directly to a specific question from the right-hand stack. Only
// unanswered cells are clickable — answered ones are visibly disabled.
function jumpTo(index: number) {
  if (index < 0 || index >= questions.value.length) return
  if (questions.value[index].is_answered) return
  currentIndex.value = index
  startTime.value = Date.now()
}

async function completeSession() {
  const inFlight = Array.from(pendingSubmissionTasks.values())
  if (inFlight.length > 0) {
    await Promise.allSettled(inFlight)
  }

  if (unsavedSubmissionInputs.size > 0) {
    let failureMessage = 'Some answers could not be saved yet. Please try again.'
    for (const [itemId, input] of Array.from(unsavedSubmissionInputs.entries())) {
      try {
        await submitAttempt(input)
        unsavedSubmissionInputs.delete(itemId)
      } catch (e: any) {
        failureMessage = typeof e === 'string' ? e : e?.message ?? failureMessage
      }
    }
    if (unsavedSubmissionInputs.size > 0) {
      error.value = failureMessage
      return
    }
  }

  let summary: SessionCompletionLike | null = null
  try {
    summary = auth.currentAccount
      ? await completeSessionWithPipeline(auth.currentAccount.id, sessionId.value)
      : await completeSessionPlain(sessionId.value)
  } catch {}
  const query: Record<string, string> = {}
  if (summary) {
    query.answered = String(summary.answered_questions)
    query.correct = String(summary.correct_questions)
    query.status = summary.status
    if (summary.accuracy_score != null) {
      query.accuracy = String(summary.accuracy_score)
    }
    if (isSessionCompletionResult(summary)) {
      query.nextActionType = summary.next_action_type
      query.nextActionTitle = summary.next_action_title
      query.nextActionRoute = summary.next_action_route
    }
  }
  if (typeof route.query.targetId === 'string' && route.query.targetId.trim().length > 0) {
    query.targetId = route.query.targetId
  }
  if (typeof route.query.mode === 'string' && route.query.mode.trim().length > 0) {
    query.mode = route.query.mode
  }
  router.push({
    path: `/student/session/${sessionId.value}/debrief`,
    query,
  })
}

async function handleFlag(flagged: boolean) {
  const q = currentQuestion.value
  if (!q) return

  const previous = q.flagged
  questions.value[currentIndex.value] = { ...q, flagged }
  try {
    await flagSessionItem(sessionId.value, q.item_id, flagged)
  } catch (e: any) {
    questions.value[currentIndex.value] = { ...q, flagged: previous }
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to update flag'
  }
}

async function handleSkip() {
  const q = currentQuestion.value
  if (!q || !auth.currentAccount) return
  try {
    await submitAttempt({
      student_id: auth.currentAccount.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: null,
      response_time_ms: Math.max(0, Date.now() - startTime.value),
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      skipped: true,
      timed_out: false,
      was_timed: isTimedSession.value,
    })
    questions.value[currentIndex.value] = { ...q, is_answered: true }
    advance()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to skip question'
  }
}

async function handleTimeout() {
  const q = currentQuestion.value
  if (!q || !auth.currentAccount) return
  try {
    await submitAttempt({
      student_id: auth.currentAccount.id,
      session_id: sessionId.value,
      session_item_id: q.item_id,
      question_id: q.question_id,
      selected_option_id: null,
      response_time_ms: Math.max(0, Date.now() - startTime.value),
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      skipped: false,
      timed_out: true,
      was_timed: isTimedSession.value,
    })
    questions.value[currentIndex.value] = { ...q, is_answered: true }
    advance()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to record timeout'
  }
}
</script>

<template>
  <div class="sv-shell">

    <!-- ── STATUS BAR ────────────────────────────────────────────
         Tertiary metadata. No teal pill, no card chrome — just
         spaced-out mono labels anchored to the edges. The progress
         fill uses a hairline rule, not a progress-bar component. -->
    <header class="sv-status">
      <div class="sv-status-left">
        <span class="sv-mono-tag">SESSION</span>
        <span v-if="isTimedSession" class="sv-mono-tag sv-mono-tag--warm">TIMED</span>
        <span class="sv-mono-meta">
          <span class="sv-mono-num">{{ answeredCount }}</span>
          <span class="sv-mono-sep">/</span>
          <span class="sv-mono-num">{{ totalQuestions }}</span>
          <span class="sv-mono-unit">ANSWERED</span>
        </span>
      </div>
      <div class="sv-status-right">
        <div class="sv-progress" :aria-label="`${answeredCount} of ${totalQuestions} answered`">
          <span
            class="sv-progress-fill"
            :style="{ width: totalQuestions ? `${(answeredCount / totalQuestions) * 100}%` : '0%' }"
          />
        </div>
        <button class="sv-end" @click="completeSession">END SESSION</button>
      </div>
    </header>

    <!-- ── CONTENT ──────────────────────────────────────────────── -->
    <main class="sv-main">

      <!-- Loading -->
      <div v-if="loading" class="sv-state">
        <p class="sv-mono-tag">[ LOADING... ]</p>
      </div>

      <!-- Error -->
      <div v-else-if="error" class="sv-state">
        <p class="sv-mono-tag sv-mono-tag--warm">[ ERROR ]</p>
        <p class="sv-state-copy">{{ error }}</p>
        <button class="sv-end" @click="router.push('/student/practice')">BACK</button>
      </div>

      <!-- Empty -->
      <div v-else-if="questions.length === 0" class="sv-state">
        <p class="sv-mono-tag">[ NO QUESTIONS ]</p>
        <button class="sv-end" @click="completeSession">FINISH</button>
      </div>

      <!-- TEST STAGE — two-column, bold. Left: the live question.
           Right: the question stack (progress + jump). No per-question
           feedback. Review only happens on the debrief route. -->
      <div v-else-if="currentQuestion" class="sv-stage">

        <section class="sv-board">

          <!-- Position strip — tertiary, restrained. The STEM is the
               primary element on this screen; the counter yields to it.
               Small Space Mono on both edges, with a single middle-dot
               separator in the year glyph. No underline rule — spacing
               below does the work. -->
          <header class="sv-position">
            <span class="sv-position-count">
              <span class="sv-mono-eyebrow">QUESTION</span>
              <span class="sv-position-num">{{ String(currentIndex + 1).padStart(2, '0') }}</span>
              <span class="sv-position-sep">/</span>
              <span class="sv-position-num sv-position-num--muted">{{ String(totalQuestions).padStart(2, '0') }}</span>
            </span>
            <span
              v-if="currentQuestion.paper_exam_year != null"
              class="sv-origin"
            >
              <span class="sv-origin-year">{{ currentQuestion.paper_exam_year }}</span>
              <span
                v-if="currentQuestion.paper_question_number"
                class="sv-origin-sep"
                aria-hidden="true"
              >·</span>
              <span v-if="currentQuestion.paper_question_number" class="sv-origin-num">
                Q{{ currentQuestion.paper_question_number }}
              </span>
            </span>
          </header>

          <QuestionCard
            :key="currentQuestion.item_id"
            :stem="currentQuestion.stem"
            :format="currentQuestion.question_format"
            :difficulty="currentQuestion.difficulty"
            :estimated-seconds="currentQuestion.estimated_time_seconds ?? undefined"
            :show-timer="isTimedSession"
            :timer-seconds="currentQuestion.estimated_time_seconds ?? undefined"
            :initial-flagged="currentQuestion.flagged"
            :options="currentQuestion.options.map(o => ({
              id: o.id,
              label: o.label,
              text: o.text,
              is_correct: o.is_correct,
              misconception_id: o.misconception_id,
              distractor_intent: o.distractor_intent,
            }))"
            :question-id="currentQuestion.question_id"
            :question-number="currentIndex + 1"
            :total-questions="totalQuestions"
            @answer="handleAnswer"
            @flag="handleFlag"
            @skip="handleSkip"
            @timeout="handleTimeout"
          />
        </section>

        <!-- Right rail: vertical preview stack. Each row is a truncated
             stem next to its number, separated by hairline rules. The
             current row inverts (ink bg, paper text); answered rows are
             ghosted with a dot indicator; unanswered rows are hoverable
             and jumpable. The preview strips LaTeX/HTML so the panel
             stays compact and readable. -->
        <aside class="sv-stack" aria-label="Question previews">
          <div class="sv-stack-head">
            <span class="sv-mono-eyebrow">QUESTIONS</span>
            <span class="sv-mono-meta">
              <span class="sv-mono-num">{{ answeredCount }}</span>
              <span class="sv-mono-sep">/</span>
              <span class="sv-mono-num">{{ totalQuestions }}</span>
            </span>
          </div>

          <ol class="sv-previews" role="list">
            <li
              v-for="(q, i) in questions"
              :key="q.item_id"
              class="sv-preview-item"
            >
              <button
                type="button"
                class="sv-preview"
                :class="{
                  'sv-preview--current': i === currentIndex,
                  'sv-preview--done':    q.is_answered && i !== currentIndex,
                  'sv-preview--flagged': q.flagged,
                }"
                :disabled="q.is_answered && i !== currentIndex"
                :aria-current="i === currentIndex ? 'true' : undefined"
                @click="jumpTo(i)"
              >
                <span class="sv-preview-num">{{ String(i + 1).padStart(2, '0') }}</span>
                <span class="sv-preview-text">{{ toPreview(q.stem) || '—' }}</span>
                <span v-if="q.is_answered && i !== currentIndex" class="sv-preview-dot" aria-hidden="true" />
              </button>
            </li>
          </ol>
        </aside>

      </div>

      <!-- All answered -->
      <div v-else class="sv-state">
        <p class="sv-mono-tag">[ COMPLETE ]</p>
        <p class="sv-state-copy">All questions answered.</p>
        <button class="sv-end" @click="completeSession">VIEW RESULTS →</button>
      </div>

    </main>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   Nothing-inspired session shell. Full-bleed, typographic, monochrome.
   All chrome dissolves into spacing and type scale.
   ═══════════════════════════════════════════════════════════════════ */
.sv-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--paper);
  color: var(--ink);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-feature-settings: 'ss01', 'ss02';
  letter-spacing: -0.005em;
}

/* ─────── STATUS BAR ─────── */
.sv-status {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 32px;
  padding: 20px clamp(24px, 5vw, 72px);
}
.sv-status-left,
.sv-status-right {
  display: flex;
  align-items: center;
  gap: 20px;
  min-width: 0;
}
.sv-mono-tag {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
  white-space: nowrap;
}
.sv-mono-tag--warm { color: var(--warm, #c2410c); }
.sv-mono-meta {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-weight: 700;
  letter-spacing: 0.18em;
}
.sv-mono-num {
  color: var(--ink);
  font-size: 12px;
  letter-spacing: 0.12em;
}
.sv-mono-num--lg { font-size: 18px; letter-spacing: 0.02em; }
.sv-mono-sep  { color: var(--ink-muted); font-size: 12px; }
.sv-mono-unit { color: var(--ink-muted); font-size: 11px; letter-spacing: 0.22em; }
.sv-mono-eyebrow {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}

/* Hairline progress — a line, not a pill */
.sv-progress {
  position: relative;
  width: clamp(120px, 18vw, 220px);
  height: 2px;
  background: rgba(0, 0, 0, 0.08);
  overflow: hidden;
}
.sv-progress-fill {
  display: block;
  height: 100%;
  background: var(--ink);
  transition: width 240ms cubic-bezier(0.16, 1, 0.3, 1);
}

.sv-end {
  background: transparent;
  border: none;
  padding: 6px 0;
  margin: 0;
  color: var(--ink-secondary);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: color 140ms ease;
}
.sv-end:hover { color: var(--ink); }

/* ─────── MAIN STAGE (two-column: board + question stack) ─────── */
.sv-main {
  flex: 1;
  overflow-y: auto;
  padding: clamp(32px, 5vh, 56px) clamp(24px, 5vw, 72px) 96px;
}

.sv-state {
  margin: 10vh auto 0;
  display: grid;
  gap: 18px;
  justify-items: start;
  max-width: 480px;
}
.sv-state-copy {
  margin: 0;
  font-family: 'Space Grotesk', sans-serif;
  font-size: 16px;
  font-weight: 400;
  color: var(--ink);
}

.sv-stage {
  width: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) clamp(260px, 22vw, 340px);
  gap: clamp(48px, 6vw, 96px);
  align-items: start;
}
.sv-board {
  min-width: 0;
  display: grid;
  gap: clamp(32px, 4vh, 56px);
}

/* ─────── POSITION STRIP — restrained, yields to the stem ─────── */
.sv-position {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 24px;
  padding-bottom: clamp(20px, 3vh, 36px);
}
.sv-position-count {
  display: inline-flex;
  align-items: baseline;
  gap: 12px;
}
.sv-position-num {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: clamp(18px, 1.8vw, 24px);
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--ink);
  line-height: 1;
}
.sv-position-num--muted { color: var(--ink-muted); }
.sv-position-sep {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: clamp(16px, 1.6vw, 22px);
  font-weight: 400;
  color: var(--ink-muted);
  padding: 0 2px;
}

.sv-origin {
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-weight: 700;
}
.sv-origin-year {
  font-size: clamp(22px, 2vw, 28px);
  color: var(--ink);
  letter-spacing: 0.02em;
  line-height: 1;
}
.sv-origin-sep { color: var(--ink-muted); font-size: 14px; }
.sv-origin-num {
  font-size: 13px;
  color: var(--ink-secondary);
  letter-spacing: 0.14em;
}

/* ─────── RIGHT RAIL: VERTICAL PREVIEW STACK ─────── */
.sv-stack {
  position: sticky;
  top: clamp(16px, 2vh, 32px);
  max-height: calc(100vh - clamp(120px, 14vh, 180px));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 14px;
  padding-top: 4px;
}
.sv-stack-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--ink);
}

.sv-previews {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(0,0,0,0.15) transparent;
}
.sv-previews::-webkit-scrollbar { width: 6px; }
.sv-previews::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.15); }

.sv-preview-item { display: block; }
.sv-preview-item + .sv-preview-item .sv-preview { border-top: 1px solid var(--rule, rgba(0, 0, 0, 0.08)); }

.sv-preview {
  position: relative;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) 10px;
  align-items: start;
  gap: 14px;
  width: 100%;
  padding: 16px 8px 16px 4px;
  border: none;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    background 120ms ease,
    padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1),
    color 140ms ease;
}
.sv-preview:hover:not(:disabled) {
  background: var(--paper-warm, rgba(0, 0, 0, 0.04));
  padding-left: 12px;
}
.sv-preview:disabled { cursor: default; }

.sv-preview-num {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
  padding-top: 2px;
  transition: color 140ms ease;
}
.sv-preview:hover:not(:disabled) .sv-preview-num { color: var(--ink); }

.sv-preview-text {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 14px;
  font-weight: 400;
  line-height: 1.38;
  color: var(--ink-secondary);
  min-width: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}
.sv-preview:hover:not(:disabled) .sv-preview-text { color: var(--ink); }

/* Answered: ghosted, with a trailing dot for quick scanning. */
.sv-preview--done .sv-preview-num  { color: var(--ink-muted); opacity: 0.6; }
.sv-preview--done .sv-preview-text { color: var(--ink-muted); }
.sv-preview-dot {
  align-self: center;
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--ink);
  opacity: 0.5;
}

/* Current — the one break from the list's quiet uniformity. Inverts
   into ink, with a stronger left-inset so it reads as a pinned marker
   rather than a floating chip. */
.sv-preview--current {
  background: var(--ink);
  padding-left: 12px;
}
.sv-preview--current .sv-preview-num,
.sv-preview--current .sv-preview-text { color: var(--paper); }
.sv-preview--current .sv-preview-num  { opacity: 0.72; }
.sv-preview--current .sv-preview-text { font-weight: 500; }
.sv-preview--current:hover:not(:disabled) { background: var(--ink); padding-left: 12px; }

/* Flagged marker — warm accent, top-right. Restraint via size. */
.sv-preview--flagged::after {
  content: '';
  position: absolute;
  top: 14px;
  right: 8px;
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: var(--warm, #c2410c);
  opacity: 0.9;
}

/* Narrow viewports — rail flows below the stage. */
@media (max-width: 1024px) {
  .sv-stage { grid-template-columns: minmax(0, 1fr); gap: 40px; }
  .sv-stack { position: static; max-height: none; }
  .sv-previews { max-height: 40vh; }
}

@media (max-width: 760px) {
  .sv-status { padding: 16px 20px; }
  .sv-main   { padding: 24px 20px 72px; }
  .sv-progress { width: 80px; }
  .sv-position { flex-wrap: wrap; gap: 10px; }
}

@media (prefers-color-scheme: dark) {
  .sv-progress { background: rgba(255, 255, 255, 0.12); }
  .sv-preview-item + .sv-preview-item .sv-preview { border-top-color: rgba(255, 255, 255, 0.08); }
  .sv-previews::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); }
}
</style>



