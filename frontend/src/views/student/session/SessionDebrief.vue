<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { completeDailyClimb } from '@/ipc/coach'
import {
  completeSession,
  completeSessionWithPipeline,
  type SessionCompletionResultDto,
  type SessionSummaryDto,
} from '@/ipc/sessions'
import {
  checkEliteBadges,
  clearEliteSessionTracking,
  isEliteSessionScored,
  readEliteSessionClass,
  scoreEliteSession,
  type EliteSessionScoreDto,
} from '@/ipc/elite'
const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const sessionId = computed(() => Number(route.params.id))
const loading = ref(true)
type DebriefSummary = SessionSummaryDto | SessionCompletionResultDto
type NextAction = { type: string; title: string; route: string }
const summary = ref<DebriefSummary | null>(null)
const eliteScore = ref<EliteSessionScoreDto | null>(null)
const eliteBadges = ref<string[]>([])
const eliteScoring = ref(false)
const beatYesterdayCompleted = ref(false)

function parseNumberQuery(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value !== 'string' || value.trim().length === 0) return null
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

function readSummaryFromQuery(): DebriefSummary | null {
  const answered = parseNumberQuery(route.query.answered)
  const correct = parseNumberQuery(route.query.correct)
  const status = typeof route.query.status === 'string' ? route.query.status : null
  if (answered == null || correct == null || !status) return null

  return {
    session_id: sessionId.value,
    answered_questions: answered,
    correct_questions: correct,
    accuracy_score: parseNumberQuery(route.query.accuracy),
    status,
  }
}

function readNextActionFromQuery() {
  return {
    type: typeof route.query.nextActionType === 'string' ? route.query.nextActionType : null,
    title: typeof route.query.nextActionTitle === 'string' ? route.query.nextActionTitle : null,
    route: typeof route.query.nextActionRoute === 'string' ? route.query.nextActionRoute : null,
  }
}

function isSessionCompletionResult(
  value: DebriefSummary,
): value is SessionCompletionResultDto {
  return 'next_action_title' in value
}

// Debrief runs on its own type stack — a serif display (Fraunces
// variable, wonk + soft axes for character) paired with a variable
// grotesque (Bricolage Grotesque) for metadata. Deliberately different
// from the test screen so the "result moment" feels ceremonial.
onMounted(() => {
  const href =
    'https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,300..700&family=Fraunces:ital,opsz,wght@0,9..144,300..900;1,9..144,300..900&display=swap'
  if (!document.head.querySelector(`link[href="${href}"]`)) {
    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = href
    document.head.appendChild(link)
  }
})

onMounted(async () => {
  const querySummary = readSummaryFromQuery()
  if (querySummary) {
    summary.value = querySummary
    await maybeScoreElite()
    await maybeCompleteDailyClimb()
    loading.value = false
    return
  }

  try {
    summary.value = auth.currentAccount
      ? await completeSessionWithPipeline(auth.currentAccount.id, sessionId.value)
      : await completeSession(sessionId.value)
    await maybeScoreElite()
    await maybeCompleteDailyClimb()
  } catch (e) {
    console.error('Failed to load summary:', e)
  }
  loading.value = false
})

async function maybeScoreElite() {
  if (!auth.currentAccount) return
  if (isEliteSessionScored(sessionId.value)) return

  const sessionClass = readEliteSessionClass(sessionId.value)
  if (!sessionClass) return

  eliteScoring.value = true
  try {
    eliteScore.value = await scoreEliteSession(auth.currentAccount.id, sessionId.value, sessionClass)
    eliteBadges.value = await checkEliteBadges(auth.currentAccount.id, eliteScore.value.subject_id).catch(() => [])
    clearEliteSessionTracking(sessionId.value)
    localStorage.setItem(`ecoach-elite-session-scored:${sessionId.value}`, 'true')
  } catch (e) {
    console.error('Failed to score elite session:', e)
  } finally {
    eliteScoring.value = false
  }
}

async function maybeCompleteDailyClimb() {
  const targetId = parseNumberQuery(route.query.targetId)
  if (targetId == null || route.query.mode !== 'beat-yesterday') return

  const completionKey = `ecoach-beat-daily-climb-completed:${sessionId.value}:${targetId}`
  if (localStorage.getItem(completionKey) === 'true') {
    beatYesterdayCompleted.value = true
    return
  }

  try {
    await completeDailyClimb(targetId)
    localStorage.setItem(completionKey, 'true')
    beatYesterdayCompleted.value = true
  } catch (e) {
    console.error('Failed to complete daily climb:', e)
  }
}

const accuracyPct = computed(() => {
  if (!summary.value) return 0
  return Math.round((summary.value.accuracy_score ?? 0) / 100)
})

const correct = computed(() => summary.value?.correct_questions ?? 0)
const answered = computed(() => summary.value?.answered_questions ?? 0)
const wrong = computed(() => Math.max(0, answered.value - correct.value))

// Bulletin-voice grade copy. More editorial than "needs work".
const grade = computed(() => {
  const pct = accuracyPct.value
  if (pct >= 90) return 'Distinguished'
  if (pct >= 75) return 'Commended'
  if (pct >= 60) return 'Satisfactory'
  if (pct >= 45) return 'Developing'
  return 'Requires Review'
})

// Variant drives the accent on the grade label + the letterpress
// offset behind the score numeral. Everything else stays in ink.
const gradeVariant = computed(() => {
  const pct = accuracyPct.value
  if (pct >= 90) return 'high'
  if (pct >= 75) return 'good'
  if (pct >= 60) return 'mid'
  if (pct >= 45) return 'low'
  return 'crit'
})

// One-sentence editorial lede, tuned to the result. The drop-cap
// renders off the first letter so the sentence must start with a
// capital noun/phrase.
const ledeCopy = computed(() => {
  const pct = accuracyPct.value
  if (pct >= 90) return 'Decisive. Hold the line, protect the gains, move up.'
  if (pct >= 75) return 'Commanding. A confident run with a handful of corners to polish.'
  if (pct >= 60) return 'Solid. The shape is right — sharpen the pieces that slipped.'
  if (pct >= 45) return 'Work in progress. Several themes to revisit before the next pass.'
  return 'Opportunities for review. Start with the misses and rebuild the base.'
})

// Short session-number label for the masthead: "№ 0004".
const bulletinNo = computed(() => `№ ${String(sessionId.value).padStart(4, '0')}`)

// Human date for the eyebrow — "21 April 2026" style.
const today = computed(() => {
  const d = new Date()
  return d.toLocaleDateString(undefined, { day: '2-digit', month: 'long', year: 'numeric' })
})

function pad2(n: number): string {
  return String(Math.max(0, Math.floor(n))).padStart(2, '0')
}

const nextAction = computed<NextAction>(() => {
  const queryAction = readNextActionFromQuery()
  if (queryAction.title && queryAction.route) {
    return {
      type: queryAction.type ?? 'continue',
      title: queryAction.title,
      route: queryAction.route,
    }
  }
  if (summary.value && isSessionCompletionResult(summary.value)) {
    return {
      type: summary.value.next_action_type,
      title: summary.value.next_action_title,
      route: summary.value.next_action_route,
    }
  }
  if (wrong.value > 0) {
    return {
      type: 'review_mistakes',
      title: 'Review Mistakes',
      route: '/student/mistakes',
    }
  }
  return {
    type: 'continue',
    title: 'Practice Again',
    route: '/student/practice',
  }
})
</script>

<template>
  <div class="rpt-shell" :data-variant="gradeVariant">

    <!-- MASTHEAD — small-caps broadsheet nameplate. Runs across the
         top the way a newspaper masthead does. -->
    <header class="rpt-mast">
      <span class="rpt-mast-title">Result Bulletin</span>
      <span class="rpt-mast-rule" aria-hidden="true" />
      <span class="rpt-mast-no">{{ bulletinNo }}</span>
      <span class="rpt-mast-rule" aria-hidden="true" />
      <time class="rpt-mast-date">{{ today }}</time>
      <button class="rpt-close" @click="router.push('/student')" aria-label="Close">
        <span class="rpt-close-x" aria-hidden="true">✕</span>
        <span class="rpt-close-label">Close</span>
      </button>
    </header>

    <!-- Loading / error -->
    <div v-if="loading" class="rpt-state"><p>Loading…</p></div>

    <div v-else-if="!summary" class="rpt-state">
      <p class="rpt-state-head">Results unavailable</p>
      <p>We couldn't load the session summary.</p>
      <button class="rpt-btn" @click="router.push('/student')">Return home</button>
    </div>

    <!-- BULLETIN BODY -->
    <article v-else class="rpt-article">

      <!-- HEADLINE — the one unforgettable moment. "Result." in
           Fraunces italic display at full scale. Immediately below,
           the editorial eyebrow sets bulletin context. -->
      <section class="rpt-headline-block">
        <p class="rpt-eyebrow"><span>Issue</span> · <span>Session Complete</span></p>
        <h1 class="rpt-headline">Result<span class="rpt-headline-dot">.</span></h1>
      </section>

      <!-- TRI-RULE — classic bulletin divider: hairline / heavy / hairline. -->
      <div class="rpt-triple-rule" aria-hidden="true">
        <span /><span /><span />
      </div>

      <!-- SCORE — the hero numeral. Set in Fraunces display with
           oldstyle figures. A letterpress second-impression prints
           in the grade accent at low opacity, offset two pixels down
           and right — the mis-registered ink mark that makes it feel
           hand-pressed. -->
      <section class="rpt-score-block">
        <div
          class="rpt-score"
          :data-num="pad2(correct)"
        >
          <span class="rpt-score-num">{{ pad2(correct) }}</span>
          <span class="rpt-score-slash">/</span>
          <span class="rpt-score-den">{{ pad2(answered) }}</span>
        </div>

        <p class="rpt-grade">
          <span class="rpt-grade-pct">{{ accuracyPct }}%</span>
          <span class="rpt-grade-dash" aria-hidden="true">—</span>
          <em class="rpt-grade-label">{{ grade }}</em>
        </p>

        <p class="rpt-lede">{{ ledeCopy }}</p>
      </section>

      <!-- STATS ROW — three columns, vertical hairlines between. -->
      <div class="rpt-triple-rule" aria-hidden="true">
        <span /><span /><span />
      </div>

      <dl class="rpt-stats">
        <div class="rpt-stat">
          <dt>Correct</dt>
          <dd>{{ pad2(correct) }}</dd>
        </div>
        <div class="rpt-stat">
          <dt>Incorrect</dt>
          <dd>{{ pad2(wrong) }}</dd>
        </div>
        <div class="rpt-stat">
          <dt>Attempted</dt>
          <dd>{{ pad2(answered) }}</dd>
        </div>
      </dl>

      <div class="rpt-triple-rule" aria-hidden="true">
        <span /><span /><span />
      </div>

      <!-- DISPATCH — elite result, when available. -->
      <section v-if="eliteScore || eliteScoring" class="rpt-dispatch">
        <p class="rpt-eyebrow">Elite Dispatch</p>
        <p class="rpt-dispatch-body">
          {{
            eliteScoring
              ? 'Finalising the elite reading…'
              : eliteScore?.debrief_text ?? 'Elite scoring completed.'
          }}
        </p>
        <p v-if="eliteScore" class="rpt-dispatch-meta">
          <span>{{ Math.round(eliteScore.eps_score / 100) }}% EPS</span>
          <span class="rpt-dispatch-sep" aria-hidden="true">·</span>
          <span>{{ eliteScore.recommended_next_session }}</span>
          <template v-if="eliteBadges.length">
            <span class="rpt-dispatch-sep" aria-hidden="true">·</span>
            <span>{{ eliteBadges.length }} badge update{{ eliteBadges.length > 1 ? 's' : '' }}</span>
          </template>
        </p>
      </section>

      <p v-if="beatYesterdayCompleted" class="rpt-note">
        <span class="rpt-note-mark">✓</span>
        Daily climb recorded.
      </p>

      <!-- ACTIONS -->
      <nav class="rpt-actions" aria-label="What next">
        <button
          v-if="wrong > 0"
          class="rpt-btn rpt-btn--primary"
          @click="router.push('/student/mistakes')"
        >
          Review mistakes <span aria-hidden="true">→</span>
        </button>
        <button
          v-if="nextAction.route && nextAction.route !== '/student/mistakes'"
          class="rpt-btn"
          @click="router.push(nextAction.route)"
        >
          {{ nextAction.title }} <span aria-hidden="true">→</span>
        </button>
        <button class="rpt-btn" @click="router.push('/student/progress')">
          My progress
        </button>
        <button class="rpt-btn rpt-btn--ghost" @click="router.push('/student')">
          Home
        </button>
      </nav>

      <!-- COLOPHON — tiny marginalia at the bottom. Session meta. -->
      <footer class="rpt-colophon">
        <span>Session {{ bulletinNo }}</span>
        <span class="rpt-colophon-sep">·</span>
        <span>{{ accuracyPct }}% accuracy</span>
        <span class="rpt-colophon-sep">·</span>
        <span>{{ correct }} of {{ answered }} correct</span>
      </footer>
    </article>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   SessionDebrief — "Result Bulletin". An editorial broadsheet treatment
   of the score moment. Fraunces display italic carries the headline
   and the big numerals, Bricolage Grotesque handles the running copy.
   Warm cream paper that overrides the app theme — this view owns its
   own palette.

   Creative moves:
   • Oldstyle lining figures via font-feature-settings
   • Letterpress mis-registration on the score (second-impression
     accent ink offset 2px down-right) driven by CSS attr()
   • Tri-rule dividers (hairline / heavy / hairline)
   • Drop-cap on the editorial lede
   • Small-caps tracked-out metadata
   • Paper grain via an SVG noise filter
   ═══════════════════════════════════════════════════════════════════ */
/* ─── palette (scoped to this view — overrides the app theme on purpose) ─── */
.rpt-shell {
  /* Paper tones */
  --p-paper:  #f3ead4;   /* warm cream */
  --p-paper-2:#ead9b4;   /* soft gilded edge */
  --p-ink:    #1a1410;   /* near-black ink */
  --p-ink-2:  #5a4936;   /* aged ink */
  --p-ink-3:  #9a846a;   /* umber */
  --p-rule:   #2a1e14;   /* heavy printed rule */

  /* Grade accents — one colored value per screen, driven by [data-variant]. */
  --p-accent: #2f5132;
}
.rpt-shell[data-variant="good"] { --p-accent: #3f6638; }
.rpt-shell[data-variant="mid"]  { --p-accent: #8a6a1a; }
.rpt-shell[data-variant="low"]  { --p-accent: #a05020; }
.rpt-shell[data-variant="crit"] { --p-accent: #8a2e20; }

.rpt-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--p-paper);
  color: var(--p-ink);
  font-family: 'Bricolage Grotesque', 'Helvetica Neue', ui-sans-serif, system-ui, sans-serif;
  font-optical-sizing: auto;
  font-feature-settings: 'ss01', 'ss02';
  letter-spacing: 0;
  overflow: hidden;
  position: relative;
}

/* Paper grain — an SVG fractal noise filter fixed over the whole view
   at ~6% opacity. Creates the subtle letterpress tooth without any
   external images. */
.rpt-shell::before {
  content: '';
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='220' height='220'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix type='matrix' values='0 0 0 0 0.1  0 0 0 0 0.08  0 0 0 0 0.06  0 0 0 0.35 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>");
  background-size: 220px 220px;
  opacity: 0.18;
  mix-blend-mode: multiply;
  pointer-events: none;
  z-index: 0;
}

/* Faint hand-printed gilded edge along the top, 1px hairline. */
.rpt-shell::after {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 1px;
  background: linear-gradient(to right, transparent 0%, var(--p-rule) 8%, var(--p-rule) 92%, transparent 100%);
  z-index: 1;
  pointer-events: none;
}

.rpt-shell > * { position: relative; z-index: 2; }

/* ─── MASTHEAD ─── */
.rpt-mast {
  flex-shrink: 0;
  display: grid;
  grid-template-columns: auto 1fr auto 1fr auto auto;
  align-items: center;
  gap: clamp(12px, 2vw, 24px);
  padding: 22px clamp(24px, 5vw, 72px);
  color: var(--p-ink-2);
  font-family: 'Bricolage Grotesque', ui-sans-serif, sans-serif;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.26em;
  text-transform: uppercase;
}
.rpt-mast-title { font-weight: 700; color: var(--p-ink); }
.rpt-mast-no    { font-variant-numeric: lining-nums tabular-nums; }
.rpt-mast-date  { color: var(--p-ink-3); letter-spacing: 0.18em; }
.rpt-mast-rule {
  display: block;
  height: 1px;
  background: var(--p-rule);
  opacity: 0.55;
}
.rpt-close {
  grid-column: 6 / 7;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  color: var(--p-ink-2);
  font: inherit;
  transition: color 140ms ease;
}
.rpt-close:hover { color: var(--p-accent); }
.rpt-close-x { font-size: 14px; line-height: 1; }
.rpt-close-label { letter-spacing: 0.26em; }

/* ─── STATE MESSAGES ─── */
.rpt-state {
  flex: 1;
  display: grid;
  place-content: center;
  gap: 12px;
  padding: 0 clamp(24px, 5vw, 72px);
  color: var(--p-ink-2);
  font-size: 15px;
}
.rpt-state-head {
  font-family: 'Fraunces', 'Times New Roman', serif;
  font-weight: 500;
  font-size: 32px;
  color: var(--p-ink);
  margin: 0;
  font-style: italic;
}

/* ─── ARTICLE ─── */
.rpt-article {
  flex: 1;
  overflow-y: auto;
  padding: clamp(32px, 6vh, 72px) clamp(28px, 7vw, 96px) clamp(40px, 6vh, 80px);
  display: grid;
  gap: clamp(28px, 3.5vh, 48px);
  max-width: 1080px;
  width: 100%;
  margin: 0 auto;
  align-content: start;
}

/* ─── HEADLINE BLOCK ─── */
.rpt-headline-block {
  display: grid;
  gap: 14px;
}

.rpt-eyebrow {
  margin: 0;
  font-family: 'Bricolage Grotesque', sans-serif;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.32em;
  text-transform: uppercase;
  color: var(--p-ink-2);
}
.rpt-eyebrow span { white-space: nowrap; }

.rpt-headline {
  margin: 0;
  font-family: 'Fraunces', 'Times New Roman', serif;
  font-style: italic;
  font-weight: 300;
  font-size: clamp(80px, 14vw, 188px);
  line-height: 0.88;
  letter-spacing: -0.035em;
  color: var(--p-ink);
  font-variation-settings: 'opsz' 144, 'SOFT' 100, 'WONK' 1;
  font-feature-settings: 'ss01', 'ss02', 'ss03';
}
.rpt-headline-dot {
  color: var(--p-accent);
  font-style: normal;
}

/* ─── TRI-RULE ─── */
.rpt-triple-rule {
  display: grid;
  grid-template-rows: 1px 3px 1px;
  gap: 3px;
}
.rpt-triple-rule > span {
  display: block;
  background: var(--p-rule);
}
.rpt-triple-rule > span:nth-child(1) { opacity: 0.55; }
.rpt-triple-rule > span:nth-child(2) { opacity: 1; }
.rpt-triple-rule > span:nth-child(3) { opacity: 0.55; }

/* ─── SCORE BLOCK ─── */
.rpt-score-block {
  display: grid;
  gap: clamp(16px, 2.4vh, 28px);
  padding: clamp(16px, 2vh, 28px) 0;
}

/* The hero numeral. Fraunces display with oldstyle lining nums, a
   subtle letterpress second-impression stamped in the accent via
   text-shadow (no extra DOM). */
.rpt-score {
  position: relative;
  display: flex;
  align-items: baseline;
  gap: clamp(12px, 1.6vw, 24px);
  line-height: 0.85;
  font-family: 'Fraunces', serif;
  font-variation-settings: 'opsz' 144, 'SOFT' 60, 'WONK' 0;
  font-feature-settings: 'lnum', 'tnum', 'ss01';
}
.rpt-score-num {
  font-weight: 400;
  font-size: clamp(120px, 20vw, 280px);
  letter-spacing: -0.045em;
  color: var(--p-ink);
  text-shadow: 5px 6px 0 var(--p-accent);
  /* The shadow is the letterpress second-impression: the accent
     ink printed 5px right / 6px down behind the black stroke, as if
     the paper registered a hair out of alignment under the plate. */
}
.rpt-score-slash {
  font-weight: 200;
  font-size: clamp(72px, 12vw, 168px);
  color: var(--p-ink-3);
  font-style: italic;
  letter-spacing: -0.02em;
  transform: translateY(-0.02em);
}
.rpt-score-den {
  font-weight: 400;
  font-size: clamp(72px, 12vw, 168px);
  color: var(--p-ink-3);
  letter-spacing: -0.03em;
}

.rpt-grade {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 16px;
  font-family: 'Bricolage Grotesque', sans-serif;
}
.rpt-grade-pct {
  font-size: clamp(22px, 2.4vw, 32px);
  font-weight: 500;
  color: var(--p-ink);
  font-variant-numeric: lining-nums tabular-nums;
  letter-spacing: -0.01em;
}
.rpt-grade-dash { color: var(--p-ink-3); }
.rpt-grade-label {
  font-family: 'Fraunces', serif;
  font-style: italic;
  font-weight: 500;
  font-size: clamp(22px, 2.4vw, 32px);
  color: var(--p-accent);
  font-variation-settings: 'opsz' 36, 'SOFT' 100;
  letter-spacing: -0.01em;
}

/* Editorial lede with a proper drop-cap. The first letter prints in
   Fraunces at huge size with leading-trim so it drops ~3 lines. */
.rpt-lede {
  margin: 0;
  font-family: 'Fraunces', serif;
  font-weight: 400;
  font-size: clamp(17px, 1.5vw, 20px);
  line-height: 1.5;
  color: var(--p-ink);
  max-width: 60ch;
  font-variation-settings: 'opsz' 14, 'SOFT' 50;
}
.rpt-lede::first-letter {
  font-family: 'Fraunces', serif;
  font-style: italic;
  font-weight: 500;
  font-size: clamp(56px, 6vw, 84px);
  line-height: 0.86;
  float: left;
  padding: 2px 10px 0 0;
  color: var(--p-accent);
  font-variation-settings: 'opsz' 144, 'SOFT' 100, 'WONK' 1;
}

/* ─── STATS ─── */
.rpt-stats {
  margin: 0;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  padding: clamp(8px, 1.5vh, 20px) 0;
}
.rpt-stat {
  display: grid;
  gap: 8px;
  padding: 4px clamp(16px, 2.4vw, 36px);
  border-right: 1px solid var(--p-rule);
  border-right-color: rgba(42, 30, 20, 0.25);
}
.rpt-stat:first-child { padding-left: 0; }
.rpt-stat:last-child  { padding-right: 0; border-right: none; }
.rpt-stat dt {
  margin: 0;
  font-family: 'Bricolage Grotesque', sans-serif;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.32em;
  text-transform: uppercase;
  color: var(--p-ink-2);
}
.rpt-stat dd {
  margin: 0;
  font-family: 'Fraunces', serif;
  font-weight: 400;
  font-size: clamp(44px, 5.4vw, 84px);
  line-height: 0.92;
  color: var(--p-ink);
  font-variant-numeric: lining-nums tabular-nums;
  font-variation-settings: 'opsz' 96, 'SOFT' 30;
  letter-spacing: -0.02em;
}

/* ─── DISPATCH (elite) ─── */
.rpt-dispatch {
  display: grid;
  gap: 10px;
  padding: clamp(16px, 2vh, 24px) 0;
}
.rpt-dispatch-body {
  margin: 0;
  font-family: 'Fraunces', serif;
  font-weight: 400;
  font-style: italic;
  font-size: clamp(16px, 1.4vw, 19px);
  line-height: 1.5;
  color: var(--p-ink);
  max-width: 62ch;
  font-variation-settings: 'opsz' 18, 'SOFT' 50;
}
.rpt-dispatch-meta {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
  font-family: 'Bricolage Grotesque', sans-serif;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--p-ink-2);
  flex-wrap: wrap;
}
.rpt-dispatch-sep { color: var(--p-ink-3); }

.rpt-note {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
  font-family: 'Fraunces', serif;
  font-style: italic;
  font-weight: 500;
  font-size: 15px;
  color: var(--p-accent);
}
.rpt-note-mark { font-family: 'Bricolage Grotesque', sans-serif; font-weight: 700; }

/* ─── ACTIONS ─── */
.rpt-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding-top: clamp(12px, 2vh, 24px);
}
.rpt-btn {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 14px 26px;
  background: transparent;
  border: 1px solid var(--p-rule);
  border-color: rgba(42, 30, 20, 0.55);
  color: var(--p-ink);
  font-family: 'Bricolage Grotesque', sans-serif;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.06em;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, border-color 140ms ease, transform 200ms cubic-bezier(0.16, 1, 0.3, 1);
  border-radius: 2px;
}
.rpt-btn:hover {
  border-color: var(--p-ink);
  background: rgba(26, 20, 16, 0.04);
}
.rpt-btn--primary {
  background: var(--p-ink);
  border-color: var(--p-ink);
  color: var(--p-paper);
  padding: 14px 28px;
}
.rpt-btn--primary:hover {
  background: var(--p-accent);
  border-color: var(--p-accent);
  transform: translateX(2px);
}
.rpt-btn--ghost {
  border-color: transparent;
  color: var(--p-ink-2);
}
.rpt-btn--ghost:hover {
  color: var(--p-ink);
  background: transparent;
  border-color: transparent;
  text-decoration: underline;
  text-decoration-thickness: 1px;
  text-underline-offset: 3px;
}

/* ─── COLOPHON ─── */
.rpt-colophon {
  margin-top: clamp(24px, 4vh, 48px);
  padding-top: 16px;
  border-top: 1px solid rgba(42, 30, 20, 0.25);
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: baseline;
  font-family: 'Bricolage Grotesque', sans-serif;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--p-ink-3);
}
.rpt-colophon-sep { opacity: 0.6; }

/* ─── narrow viewports ─── */
@media (max-width: 900px) {
  .rpt-mast {
    grid-template-columns: auto 1fr auto;
    grid-auto-rows: auto;
  }
  .rpt-mast-date,
  .rpt-mast-rule:nth-child(4) { display: none; }
  .rpt-close { grid-column: auto; justify-self: end; }

  .rpt-stats { grid-template-columns: 1fr; gap: 0; }
  .rpt-stat {
    grid-template-columns: 1fr auto;
    border-right: none;
    border-bottom: 1px solid rgba(42, 30, 20, 0.25);
    padding: 18px 0;
    align-items: baseline;
  }
  .rpt-stat:last-child { border-bottom: none; }
  .rpt-stat dd { font-size: clamp(36px, 10vw, 56px); }

  .rpt-actions { flex-direction: column; align-items: stretch; }
  .rpt-btn { justify-content: center; }
}

/* The bulletin owns its palette — we don't flip for dark mode. The
   grain, warm paper and dark ink stay readable on any desktop screen. */
</style>


