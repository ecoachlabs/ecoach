<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getCoachNextAction,
  getHomeLearningStats,
  getStudentDashboard,
  getPriorityTopics,
  type CoachNextActionDto,
  type HomeLearningStatsDto,
  type StudentDashboardDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import { useHomepageArena } from '@/composables/useHomepageArena'
import {
  PhArrowRight,
  PhBell,
  PhCalendarBlank,
  PhCaretRight,
  PhCaretUp,
  PhCheckCircle,
  PhClock,
  PhFire,
  PhHourglassHigh,
  PhLightning,
  PhRobot,
  PhSparkle,
  PhTarget,
} from '@phosphor-icons/vue'
import CoachHubVersionSwitch from '@/components/coach/CoachHubVersionSwitch.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')

const nextAction = ref<CoachNextActionDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topicCases = ref<TopicCaseDto[]>([])
const homeStats = ref<HomeLearningStatsDto | null>(null)

async function refreshHomeData() {
  if (!auth.currentAccount) return
  const sid = auth.currentAccount.id
  const [action, dash, topics, stats] = await Promise.all([
    getCoachNextAction(sid),
    getStudentDashboard(sid),
    getPriorityTopics(sid, 6),
    getHomeLearningStats(sid),
  ])
  nextAction.value = action
  dashboard.value = dash
  topicCases.value = topics
  homeStats.value = stats
}

const {
  currentQuestion: quickCheck,
  selectedOptionId: selectedChoice,
  locked: quickLocked,
  error: quickError,
  loadArena,
  submitOption: submitQuickChoice,
  nextQuestion: nextQuickQuestion,
  isCorrectOption: isQuickCorrectOption,
  isWrongOption: isQuickWrongOption,
} = useHomepageArena({
  getStudentId: () => auth.currentAccount?.id,
  getTopicCases: () => topicCases.value,
  onRecorded: refreshHomeData,
  questionCount: 4,
  isTimed: true,
})

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning'
  if (h < 17) return 'Good afternoon'
  return 'Good evening'
})

const dateLabel = computed(() =>
  new Date().toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' })
)

const studentName = computed(
  () => dashboard.value?.student_name || auth.currentAccount?.display_name || 'Student'
)

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }
  try {
    await refreshHomeData()
    await loadArena()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  } finally {
    loading.value = false
  }
})

function startAction() {
  if (nextAction.value?.route) router.push(nextAction.value.route)
}

const vitalStats = computed(() => {
  const stats = homeStats.value
  return {
    streak: stats?.streak_days ?? 0,
    accuracy: stats?.accuracy_percent ?? 0,
    todayMin: stats?.today_minutes ?? 0,
    weekQuestions: stats?.week_questions ?? 0,
  }
})

const subjectRings = computed(() => {
  const subs = dashboard.value?.subjects ?? []
  return subs.map(s => {
    const pct =
      s.total_topic_count > 0
        ? Math.round((s.mastered_topic_count / s.total_topic_count) * 100)
        : 0
    return {
      id: s.subject_id,
      name: s.subject_name,
      pct,
      weak: s.weak_topic_count,
      gradient: pct >= 75 ? 'high' : pct >= 50 ? 'mid' : 'low',
    }
  })
})

const overallMastery = computed(() => {
  const subs = subjectRings.value
  if (!subs.length) return 0
  return Math.round(subs.reduce((acc, s) => acc + s.pct, 0) / subs.length)
})

const RING_R = 45
const RING_C = 2 * Math.PI * RING_R
const SUB_R = 42
const SUB_C = 2 * Math.PI * SUB_R

function ringOffset(circ: number, pct: number): number {
  return circ * (1 - Math.max(0, Math.min(1, pct / 100)))
}

// Quick check ─────────────────────────────────────────────
async function pickChoice(id: string) {
  if (quickLocked.value) return
  const result = await submitQuickChoice(id)
  if (result) {
    window.setTimeout(() => { void nextQuickQuestion() }, 950)
  }
}

// Priority cases — fall back to mock if IPC empty
type CaseTone = 'terracotta' | 'amber' | 'muted'
const priorityCases = computed(() => {
  const real = topicCases.value.slice(0, 3).map(t => ({
    id: `t-${t.topic_id}`,
    title: t.topic_name,
    detail: t.intervention_reason || t.intervention_mode.replace(/_/g, ' '),
    tone: (t.intervention_urgency === 'high'
      ? 'terracotta'
      : t.intervention_urgency === 'medium'
      ? 'amber'
      : 'muted') as CaseTone,
    primary: 'Drill now',
    secondary: t.intervention_urgency === 'medium' ? 'Skip for now' : null,
    route: '/student/practice',
  }))
  return real
  return [
    {
      id: 'm-thermo',
      title: 'Thermodynamics',
      detail: 'Formula application speed is well below cohort average.',
      tone: 'terracotta' as CaseTone,
      primary: 'Drill formulas',
      secondary: null,
      route: '/student/practice',
    },
    {
      id: 'm-stoich',
      title: 'Stoichiometry',
      detail: 'Frequent conceptual errors in limiting reactant calculations.',
      tone: 'amber' as CaseTone,
      primary: 'Review theory',
      secondary: 'Skip for now',
      route: '/student/curriculum',
    },
    {
      id: 'm-calc',
      title: 'Calculus · Integration',
      detail: "Decay detected. Hasn't been reviewed in 14 days.",
      tone: 'muted' as CaseTone,
      primary: 'Quick refresh',
      secondary: null,
      route: '/student/memory',
    },
  ]
})

// Activity feed
type FeedTone = 'teal' | 'terracotta' | 'muted'
const feedItems = computed(() => {
  const items: Array<{
    id: string
    title: string
    detail: string
    when: string
    tone: FeedTone
    cta?: { label: string; route: string }
    icon?: any
  }> = []

  if (nextAction.value) {
    items.push({
      id: 'feed-coach',
      title: nextAction.value.title,
      detail: nextAction.value.subtitle,
      when: 'just now',
      tone: 'teal',
      cta: { label: 'Start now', route: nextAction.value.route },
    })
  }

  if (topicCases.value.length) {
    const top = topicCases.value[0]
    items.push({
      id: 'feed-topic',
      title: `${top.topic_name} flagged for recovery`,
      detail: top.intervention_mode.replace(/_/g, ' '),
      when: '2h ago',
      tone: 'terracotta',
      cta: { label: 'Enter recovery', route: '/student/knowledge-gap' },
    })
  }

  items.push({
    id: 'feed-system',
    title: 'Syllabus update applied',
    detail:
      'Tutor AI recalibrated your weekend mock exam to weight Chemistry weak points heavier.',
    when: 'yesterday',
    tone: 'muted',
    icon: PhRobot,
  })

  return items.slice(0, 4)
})

// Leaderboard mock
const leaderboard = [
  { rank: 1, name: 'Amara O.', score: 14200 },
  { rank: 2, name: 'Tariq M.', score: 13850 },
  { rank: 3, name: 'Elena V.', score: 13100 },
  { rank: 4, name: 'David K.', score: 12410 },
  { rank: 5, name: 'Zoe F.', score: 12060 },
  { rank: 6, name: 'You', score: 11420, isYou: true },
  { rank: 7, name: 'Sam R.', score: 10800 },
]
</script>

<template>
  <div class="v2-shell">
    <CoachHubVersionSwitch />
    <main class="v2-canvas">
      <!-- HEADER ─────────────────────────────────────── -->
      <header class="v2-header">
        <div class="v2-header-left">
          <p class="eyebrow">
            <PhCalendarBlank :size="14" weight="regular" />
            <span>{{ dateLabel }}</span>
          </p>
          <h1 class="display-h1 font-display">
            {{ greeting }}, <span class="terracotta-accent">{{ studentName }}.</span>
          </h1>
        </div>
        <div class="v2-header-right">
          <button class="pill-btn" type="button">
            <PhSparkle :size="15" weight="fill" class="text-amber" />
            <span>Ask AI Tutor</span>
          </button>
          <button class="round-btn" type="button" aria-label="Notifications">
            <PhBell :size="16" weight="regular" />
            <span class="dot-badge" />
          </button>
        </div>
      </header>

      <!-- LOADING / ERROR -->
      <div v-if="loading" class="v2-skeleton-grid">
        <div class="skel skel-hero" />
        <div class="skel skel-quick" />
        <div class="skel skel-vitals" />
        <div class="skel skel-mastery" />
        <div class="skel skel-cases" />
      </div>
      <div v-else-if="error" class="v2-error">{{ error }}</div>

      <!-- BENTO GRID -->
      <div v-else class="bento-grid">
        <!-- Coach Directive (hero) -->
        <article class="card card-hero col-8">
          <div class="kente-stripe" />
          <div class="kente-watermark" aria-hidden="true" />
          <div class="card-hero-body">
            <div class="hero-meta-row">
              <span class="chip chip-quiet">
                <PhTarget :size="12" weight="fill" class="text-terracotta" />
                Coach Directive
              </span>
              <span v-if="nextAction?.estimated_minutes" class="chip chip-time">
                <PhClock :size="12" weight="regular" class="text-amber" />
                ~{{ nextAction.estimated_minutes }} min focus
              </span>
            </div>

            <h2 class="hero-title font-display">
              <template v-if="nextAction">
                {{ nextAction.title }}
              </template>
              <template v-else>
                You're all caught up — <span class="italic teal-accent">choose a fresh focus.</span>
              </template>
            </h2>
            <p class="hero-sub">
              {{
                nextAction?.subtitle ||
                'No pending coach directive. Open the Journey to plan your next move, or run a Diagnostic to surface a new priority.'
              }}
            </p>

            <div class="hero-actions">
              <button class="btn-primary" @click="startAction">
                <span>{{ nextAction ? 'Initiate deep dive' : 'Open journey' }}</span>
                <PhArrowRight :size="14" weight="bold" />
              </button>
              <button class="btn-ghost" @click="router.push('/student/journey')">
                Delay intervention
              </button>
            </div>
          </div>
        </article>

        <!-- Quick Check live card -->
        <article class="card card-quick col-4">
          <div class="quick-head">
            <span class="quick-live">
              <span class="live-dot" />
              Live sync
            </span>
            <span class="quick-combo">
              <PhFire :size="12" weight="fill" />
              3× combo
            </span>
          </div>

          <div class="quick-body">
            <p class="quick-eyebrow">{{ quickCheck?.topic || 'Real questions' }}</p>
            <h3 class="quick-prompt font-display">
              {{ quickCheck?.prompt || quickError || 'Loading real questions...' }}
            </h3>

            <div v-if="quickCheck" class="quick-options">
              <button
                v-for="opt in quickCheck.options"
                :key="opt.id"
                type="button"
                class="quick-option"
                :class="{
                  'quick-option--picked': selectedChoice === opt.id,
                  'quick-option--correct': selectedChoice && isQuickCorrectOption(opt),
                  'quick-option--wrong':
                    isQuickWrongOption(opt),
                }"
                :disabled="quickLocked"
                @click="pickChoice(opt.id)"
              >
                <span class="quick-letter">{{ opt.letter }}</span>
                <span class="quick-label">{{ opt.label }}</span>
              </button>
            </div>
          </div>

          <div class="quick-timer-track">
            <div class="quick-timer-fill" />
          </div>
        </article>

        <!-- Vital strip -->
        <section class="vital-strip col-12">
          <article class="vital-tile">
            <div>
              <p class="vital-label">Ember Streak</p>
              <p class="vital-num font-display">
                {{ vitalStats.streak }}
                <span class="vital-unit">days</span>
              </p>
            </div>
            <div class="vital-glyph glyph-terracotta">
              <PhFire :size="20" weight="fill" />
            </div>
          </article>
          <article class="vital-tile">
            <div>
              <p class="vital-label">Avg accuracy</p>
              <p class="vital-num font-display">
                {{ vitalStats.accuracy }}<span class="vital-unit">%</span>
              </p>
            </div>
            <div class="vital-glyph glyph-teal">
              <PhTarget :size="20" weight="fill" />
            </div>
          </article>
          <article class="vital-tile">
            <div>
              <p class="vital-label">Today's focus</p>
              <p class="vital-num font-display">
                {{ vitalStats.todayMin }}<span class="vital-unit">m</span>
              </p>
            </div>
            <div class="vital-glyph glyph-amber">
              <PhHourglassHigh :size="20" weight="fill" />
            </div>
          </article>
          <article class="vital-tile">
            <div>
              <p class="vital-label">Week output</p>
              <p class="vital-num font-display">
                {{ vitalStats.weekQuestions }}
                <span class="vital-unit">Qs</span>
              </p>
            </div>
            <div class="vital-glyph glyph-ink">
              <PhLightning :size="20" weight="fill" />
            </div>
          </article>
        </section>

        <!-- Mastery panel -->
        <article class="card card-mastery col-7">
          <header class="card-head">
            <div>
              <h3 class="card-title font-display">Cognitive mastery</h3>
              <p class="card-sub">Holistic readiness vs. syllabus targets.</p>
            </div>
            <button class="link-btn" @click="router.push('/student/progress')">
              Full analysis
              <PhCaretRight :size="12" weight="bold" />
            </button>
          </header>

          <div class="mastery-row">
            <div class="mastery-hero">
              <svg viewBox="0 0 100 100" class="ring-svg ring-bg">
                <circle cx="50" cy="50" :r="RING_R" stroke-width="8" />
              </svg>
              <svg viewBox="0 0 100 100" class="ring-svg ring-arc ring-arc--glow">
                <defs>
                  <linearGradient id="emberHigh" x1="0%" y1="100%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="#b45309" />
                    <stop offset="40%" stop-color="#0d9488" />
                    <stop offset="100%" stop-color="#0d9488" />
                  </linearGradient>
                  <linearGradient id="emberMid" x1="0%" y1="100%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="#9e958d" />
                    <stop offset="50%" stop-color="#b45309" />
                    <stop offset="100%" stop-color="#c2410c" />
                  </linearGradient>
                  <linearGradient id="emberLow" x1="0%" y1="100%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="#e2ddcd" />
                    <stop offset="100%" stop-color="#9e958d" />
                  </linearGradient>
                </defs>
                <circle
                  cx="50"
                  cy="50"
                  :r="RING_R"
                  stroke-width="8"
                  stroke-linecap="round"
                  stroke="url(#emberHigh)"
                  :stroke-dasharray="RING_C"
                  :stroke-dashoffset="ringOffset(RING_C, overallMastery)"
                />
              </svg>
              <div class="mastery-hero-text">
                <p class="mastery-hero-num font-display">
                  {{ overallMastery }}<span class="mastery-hero-pct">%</span>
                </p>
                <p class="mastery-hero-eyebrow">Overall</p>
              </div>
            </div>

            <div class="subject-grid">
              <button
                v-for="sub in subjectRings.slice(0, 4)"
                :key="sub.id"
                type="button"
                class="subject-tile"
                @click="router.push('/student/progress/mastery-map')"
              >
                <div class="subject-ring">
                  <svg viewBox="0 0 100 100" class="ring-svg ring-bg">
                    <circle cx="50" cy="50" :r="SUB_R" stroke-width="12" />
                  </svg>
                  <svg viewBox="0 0 100 100" class="ring-svg ring-arc">
                    <circle
                      cx="50"
                      cy="50"
                      :r="SUB_R"
                      stroke-width="12"
                      stroke-linecap="round"
                      :stroke="
                        sub.gradient === 'high'
                          ? 'url(#emberHigh)'
                          : sub.gradient === 'mid'
                          ? 'url(#emberMid)'
                          : 'url(#emberLow)'
                      "
                      :stroke-dasharray="SUB_C"
                      :stroke-dashoffset="ringOffset(SUB_C, sub.pct)"
                    />
                  </svg>
                </div>
                <div class="subject-meta">
                  <p class="subject-name">{{ sub.name }}</p>
                  <p class="subject-detail">
                    {{ sub.pct }}%
                    <span v-if="sub.weak > 0" class="subject-weak">
                      · {{ sub.weak }} weak {{ sub.weak === 1 ? 'area' : 'areas' }}
                    </span>
                    <span v-else class="subject-secure"> · secure</span>
                  </p>
                </div>
              </button>

              <p v-if="!subjectRings.length" class="subject-empty">
                No subjects yet — finish onboarding to populate mastery rings.
              </p>
            </div>
          </div>
        </article>

        <!-- Priority cases -->
        <article class="card card-cases col-5">
          <header class="card-head">
            <h3 class="card-title font-display">Priority cases</h3>
            <span class="chip chip-warm">
              {{ priorityCases.length }} interventions
            </span>
          </header>

          <ul class="case-list">
            <li
              v-for="c in priorityCases"
              :key="c.id"
              class="case-item"
              :class="`case-item--${c.tone}`"
            >
              <div class="case-row">
                <h4 class="case-title">{{ c.title }}</h4>
                <span class="case-dot" :class="`dot-${c.tone}`" />
              </div>
              <p class="case-detail">{{ c.detail }}</p>
              <div class="case-actions">
                <button class="case-primary" @click="router.push(c.route)">
                  {{ c.primary }}
                </button>
                <button v-if="c.secondary" class="case-secondary">
                  {{ c.secondary }}
                </button>
              </div>
            </li>
          </ul>
        </article>

        <!-- Activity feed -->
        <article class="card card-feed col-8">
          <header class="card-head">
            <h3 class="card-title font-display">Learning subroutines</h3>
          </header>

          <ol class="feed-list">
            <li v-for="item in feedItems" :key="item.id" class="feed-item">
              <span class="feed-dot" :class="`feed-dot--${item.tone}`">
                <component
                  :is="item.icon"
                  v-if="item.icon"
                  :size="10"
                  weight="fill"
                />
              </span>
              <div class="feed-body">
                <p class="feed-when">{{ item.when }}</p>
                <h4 class="feed-title">
                  {{ item.title }}
                  <PhCheckCircle
                    v-if="item.tone === 'teal'"
                    :size="14"
                    weight="fill"
                    class="text-teal"
                  />
                </h4>
                <p class="feed-detail">{{ item.detail }}</p>
                <button
                  v-if="item.cta"
                  class="feed-cta"
                  :class="`feed-cta--${item.tone}`"
                  @click="item.cta && router.push(item.cta.route)"
                >
                  {{ item.cta.label }}
                </button>
              </div>
            </li>
          </ol>
        </article>

        <!-- Leaderboard -->
        <article class="card card-board col-4">
          <header class="card-head">
            <h3 class="card-title font-display">Ember ranks</h3>
            <span class="card-sub-inline">Cohort Alpha</span>
          </header>

          <ul class="board-list">
            <li
              v-for="row in leaderboard"
              :key="row.name"
              class="board-row"
              :class="{ 'board-row--you': row.isYou }"
            >
              <div class="board-left">
                <span class="board-rank" :class="{ 'rank-gold': row.rank === 1 }">
                  {{ row.rank }}
                </span>
                <span class="board-name">{{ row.name }}{{ row.isYou ? ' (You)' : '' }}</span>
              </div>
              <span class="board-score font-display">
                {{ row.score.toLocaleString() }}
                <PhCaretUp v-if="row.isYou" :size="10" weight="bold" />
              </span>
            </li>
          </ul>
        </article>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* ════════════════════════════════════════════════════════
   v2 — Warm Intelligence editorial bento workspace
   Uses the existing CSS token palette so dark mode flips automatically.
   ════════════════════════════════════════════════════════ */

.v2-shell {
  height: 100%;
  overflow-y: auto;
  background: var(--paper);
  transition: background-color 240ms ease;
  position: relative;
}

.v2-canvas {
  max-width: 1500px;
  margin: 0 auto;
  padding: 36px 40px 80px;
}

/* HEADER ─────────────────────────────────────────────── */
.v2-header {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 28px;
}

.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 6px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.display-h1 {
  margin: 0;
  font-size: clamp(28px, 4vw, 44px);
  line-height: 1.05;
  letter-spacing: -0.02em;
  font-weight: 500;
  color: var(--ink);
}

.terracotta-accent { color: var(--warm); }
.teal-accent { color: var(--accent); font-style: italic; }

.v2-header-right {
  display: flex;
  gap: 10px;
  align-items: center;
}

.pill-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--border-soft);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
  box-shadow: var(--shadow-xs);
  transition: background 160ms ease, transform 160ms ease;
}
.pill-btn:hover { background: var(--paper-warm); transform: translateY(-1px); }

.round-btn {
  position: relative;
  width: 38px;
  height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--border-soft);
  color: var(--ink);
  box-shadow: var(--shadow-xs);
  transition: background 160ms ease;
}
.round-btn:hover { background: var(--paper-warm); }
.dot-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--warm);
  border: 2px solid var(--surface);
}

.text-amber { color: var(--gold); }
.text-teal { color: var(--accent); }
.text-terracotta { color: var(--warm); }

/* BENTO GRID ─────────────────────────────────────────── */
.bento-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 22px;
  align-items: start;
}

.col-12 { grid-column: span 12; }
.col-8  { grid-column: span 8; }
.col-7  { grid-column: span 7; }
.col-5  { grid-column: span 5; }
.col-4  { grid-column: span 4; }

@media (max-width: 1200px) {
  .col-8, .col-7 { grid-column: span 12; }
  .col-5, .col-4 { grid-column: span 12; }
}

/* CARD BASE ──────────────────────────────────────────── */
.card {
  position: relative;
  background: var(--surface);
  border: 1px solid color-mix(in srgb, var(--border-soft) 60%, transparent);
  border-radius: 28px;
  box-shadow:
    0 4px 20px -2px rgba(26, 22, 18, 0.04),
    0 1px 3px rgba(26, 22, 18, 0.02);
  transition: background 240ms ease, border-color 240ms ease;
  overflow: hidden;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 26px 28px 0;
}

.card-title {
  margin: 0;
  font-size: 20px;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.card-sub {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--ink-muted);
}

.card-sub-inline {
  font-size: 12px;
  color: var(--ink-muted);
  font-weight: 500;
}

.link-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  font-weight: 500;
  color: var(--accent);
  background: transparent;
  transition: color 160ms ease;
}
.link-btn:hover { color: var(--ink); }

.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 11px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
.chip-quiet {
  background: color-mix(in srgb, var(--ink) 6%, transparent);
  border: 1px solid color-mix(in srgb, var(--ink) 10%, transparent);
  color: var(--ink-muted);
}
.chip-time {
  background: var(--surface);
  border: 1px solid var(--border-soft);
  color: var(--ink-muted);
  text-transform: none;
  letter-spacing: 0.02em;
  font-weight: 500;
}
.chip-warm {
  padding: 4px 10px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--warm) 12%, transparent);
  color: var(--warm);
  font-size: 10px;
  letter-spacing: 0.14em;
}

/* HERO CARD ──────────────────────────────────────────── */
.card-hero {
  display: flex;
  flex-direction: column;
  min-height: 320px;
}

.kente-stripe {
  height: 6px;
  background: repeating-linear-gradient(
    90deg,
    var(--warm) 0 10px,
    var(--accent) 10px 20px,
    var(--gold) 20px 30px,
    var(--paper) 30px 40px
  );
  opacity: 0.7;
}

.kente-watermark {
  position: absolute;
  inset: 6px 0 0;
  pointer-events: none;
  z-index: 0;
  background-image: url("data:image/svg+xml,%3Csvg width='40' height='40' viewBox='0 0 40 40' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M0 0h20v20H0V0zm20 20h20v20H20V20zM20 0h4v20h-4V0zm4 0h16v4H24V0zM0 20h4v20H0V20zm4 0h16v4H4v-4z' fill='%23b45309' fill-opacity='0.03' fill-rule='evenodd'/%3E%3C/svg%3E");
}

.card-hero-body {
  position: relative;
  z-index: 1;
  padding: 36px 40px 36px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex: 1;
}

.hero-meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.hero-title {
  margin: 0;
  font-size: clamp(24px, 2.6vw, 34px);
  line-height: 1.2;
  letter-spacing: -0.015em;
  color: var(--ink);
  max-width: 32ch;
}
.hero-title .italic { font-style: italic; }

.hero-sub {
  margin: 0;
  max-width: 56ch;
  font-size: 15px;
  line-height: 1.6;
  color: var(--ink-muted);
}

.hero-actions {
  margin-top: auto;
  padding-top: 8px;
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 14px 26px;
  border-radius: 18px;
  background: var(--warm);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: -0.005em;
  box-shadow: 0 8px 20px -6px color-mix(in srgb, var(--warm) 60%, transparent);
  transition: transform 180ms ease, background 180ms ease;
}
.btn-primary:hover {
  transform: translateY(-1px);
  background: color-mix(in srgb, var(--warm) 88%, #000);
}

.btn-ghost {
  padding: 14px 22px;
  border-radius: 18px;
  font-size: 14px;
  font-weight: 500;
  color: var(--ink);
  background: transparent;
  transition: background 180ms ease;
}
.btn-ghost:hover { background: color-mix(in srgb, var(--ink) 5%, transparent); }

/* QUICK CHECK CARD ───────────────────────────────────── */
.card-quick {
  display: flex;
  flex-direction: column;
  min-height: 320px;
}

.quick-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 22px;
  background: var(--ink);
  color: var(--paper);
}

.quick-live {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}
.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--gold);
  animation: livePulse 1.6s ease-in-out infinite;
}
@keyframes livePulse {
  0%, 100% { opacity: 0.4; transform: scale(0.85); }
  50% { opacity: 1; transform: scale(1.1); }
}

.quick-combo {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 500;
  color: var(--warm);
}

.quick-body {
  flex: 1;
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.quick-eyebrow {
  margin: 0;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.quick-prompt {
  margin: 0;
  font-size: 17px;
  line-height: 1.4;
  color: var(--ink);
}

.quick-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: auto;
}

.quick-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 12px;
  border-radius: 14px;
  border: 1px solid var(--border-soft);
  background: var(--surface);
  text-align: left;
  transition: border-color 160ms ease, background 160ms ease, transform 160ms ease;
}
.quick-option:hover {
  border-color: color-mix(in srgb, var(--accent) 40%, var(--border-soft));
  background: color-mix(in srgb, var(--accent) 5%, transparent);
}
.quick-option--picked {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}
.quick-option--correct {
  border-color: var(--success);
  background: color-mix(in srgb, var(--success) 10%, transparent);
}
.quick-option--wrong {
  border-color: var(--danger);
  background: color-mix(in srgb, var(--danger) 10%, transparent);
}

.quick-letter {
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid var(--border-soft);
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-muted);
}
.quick-option--picked .quick-letter,
.quick-option--correct .quick-letter {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.quick-option--wrong .quick-letter {
  background: var(--danger);
  border-color: var(--danger);
  color: #fff;
}

.quick-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
}

.quick-timer-track {
  height: 6px;
  background: color-mix(in srgb, var(--gold) 18%, transparent);
}
.quick-timer-fill {
  height: 100%;
  width: 100%;
  background: var(--gold);
  transform-origin: left;
  animation: timerDeplete 20s linear forwards;
}
@keyframes timerDeplete {
  from { transform: scaleX(1); }
  to { transform: scaleX(0); }
}

/* VITAL STRIP ────────────────────────────────────────── */
.vital-strip {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}
@media (max-width: 800px) { .vital-strip { grid-template-columns: repeat(2, 1fr); } }

.vital-tile {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px;
  background: var(--surface);
  border: 1px solid color-mix(in srgb, var(--border-soft) 60%, transparent);
  border-radius: 18px;
  box-shadow: var(--shadow-xs);
  transition: transform 180ms ease, box-shadow 180ms ease;
}
.vital-tile:hover { transform: translateY(-2px); box-shadow: var(--shadow-sm); }

.vital-label {
  margin: 0 0 4px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.vital-num {
  margin: 0;
  font-size: 28px;
  letter-spacing: -0.02em;
  color: var(--ink);
  font-weight: 500;
}
.vital-unit {
  margin-left: 4px;
  font-size: 12px;
  font-family: var(--font-body);
  color: var(--ink-muted);
  font-weight: 400;
  letter-spacing: 0;
}

.vital-glyph {
  width: 44px;
  height: 44px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 180ms ease;
}
.vital-tile:hover .vital-glyph { transform: scale(1.08); }

.glyph-terracotta {
  background: color-mix(in srgb, var(--warm) 12%, transparent);
  color: var(--warm);
}
.glyph-teal {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
}
.glyph-amber {
  background: color-mix(in srgb, var(--gold) 14%, transparent);
  color: var(--gold);
}
.glyph-ink {
  background: color-mix(in srgb, var(--ink) 6%, transparent);
  color: var(--ink);
}

/* MASTERY ────────────────────────────────────────────── */
.card-mastery {
  padding-bottom: 26px;
  display: flex;
  flex-direction: column;
}

.mastery-row {
  display: flex;
  flex-wrap: wrap;
  gap: 28px;
  align-items: center;
  padding: 22px 28px 0;
}

.mastery-hero {
  position: relative;
  width: 184px;
  height: 184px;
  flex-shrink: 0;
}

.ring-svg {
  width: 100%;
  height: 100%;
  position: absolute;
  inset: 0;
  fill: none;
}
.ring-svg.ring-bg circle { stroke: color-mix(in srgb, var(--border-soft) 80%, transparent); }
.ring-arc {
  transform: rotate(-90deg);
  transform-origin: 50% 50%;
}
.ring-arc circle {
  transition: stroke-dashoffset 1.4s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
}
.ring-arc--glow circle {
  filter: drop-shadow(0 0 10px color-mix(in srgb, var(--accent) 35%, transparent));
}

.mastery-hero-text {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  pointer-events: none;
}
.mastery-hero-num {
  margin: 0;
  font-size: 46px;
  letter-spacing: -0.04em;
  color: var(--ink);
  font-weight: 500;
}
.mastery-hero-pct {
  font-size: 22px;
  color: var(--ink-muted);
  margin-left: 2px;
}
.mastery-hero-eyebrow {
  margin: 4px 0 0;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.subject-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  min-width: 240px;
}

.subject-tile {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: transparent;
  text-align: left;
  transition: background 160ms ease, border-color 160ms ease;
}
.subject-tile:hover {
  background: var(--paper-warm);
  border-color: color-mix(in srgb, var(--border-soft) 70%, transparent);
}

.subject-ring {
  position: relative;
  width: 46px;
  height: 46px;
  flex-shrink: 0;
}

.subject-name {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
}
.subject-detail {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--ink-muted);
}
.subject-weak { color: var(--warm); }
.subject-secure { color: var(--success); }
.subject-empty {
  grid-column: span 2;
  margin: 0;
  font-size: 13px;
  color: var(--ink-muted);
}

/* PRIORITY CASES ─────────────────────────────────────── */
.card-cases {
  padding-bottom: 26px;
  display: flex;
  flex-direction: column;
}
.case-list {
  list-style: none;
  margin: 18px 0 0;
  padding: 0 28px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.case-item {
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--border-soft) 70%, transparent);
  background: var(--surface);
  transition: background 160ms ease, border-color 160ms ease;
}
.case-item:hover { background: var(--paper-warm); }
.case-item--terracotta:hover { border-color: color-mix(in srgb, var(--warm) 30%, var(--border-soft)); }
.case-item--amber:hover { border-color: color-mix(in srgb, var(--gold) 30%, var(--border-soft)); }
.case-item--muted:hover { border-color: var(--border-strong); }

.case-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 6px;
}
.case-title { margin: 0; font-size: 14px; font-weight: 600; color: var(--ink); }
.case-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  margin-top: 6px;
}
.dot-terracotta {
  background: var(--warm);
  box-shadow: 0 0 8px color-mix(in srgb, var(--warm) 50%, transparent);
}
.dot-amber { background: var(--gold); }
.dot-muted { background: var(--ink-muted); }

.case-detail {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--ink-muted);
  line-height: 1.5;
}

.case-actions { display: flex; gap: 8px; }

.case-primary {
  padding: 7px 12px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--ink) 5%, transparent);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  transition: background 160ms ease, color 160ms ease;
}
.case-primary:hover { background: var(--ink); color: var(--paper); }

.case-secondary {
  padding: 7px 12px;
  border-radius: 10px;
  background: transparent;
  font-size: 12px;
  font-weight: 500;
  color: var(--ink-muted);
  transition: color 160ms ease;
}
.case-secondary:hover { color: var(--ink); }

/* FEED ─────────────────────────────────────────────── */
.card-feed { padding: 0 0 28px; }

.feed-list {
  list-style: none;
  margin: 18px 0 0;
  padding: 0 28px 0 50px;
  position: relative;
}
.feed-list::before {
  content: '';
  position: absolute;
  left: 32px;
  top: 8px;
  bottom: 8px;
  width: 2px;
  background: color-mix(in srgb, var(--border-soft) 80%, transparent);
}

.feed-item {
  position: relative;
  padding-bottom: 26px;
}
.feed-item:last-child { padding-bottom: 0; }

.feed-dot {
  position: absolute;
  left: -22px;
  top: 4px;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--surface);
  border: 2px solid var(--border-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-muted);
}
.feed-dot--teal { border-color: var(--accent); color: var(--accent); }
.feed-dot--terracotta { border-color: var(--warm); color: var(--warm); }
.feed-dot--teal::after,
.feed-dot--terracotta::after {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 999px;
}
.feed-dot--teal::after { background: var(--accent); }
.feed-dot--terracotta::after {
  background: var(--warm);
  animation: livePulse 2s ease-in-out infinite;
}

.feed-when {
  margin: 0 0 4px;
  font-size: 11px;
  font-weight: 500;
  color: var(--ink-muted);
}
.feed-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
}
.feed-detail {
  margin: 0 0 10px;
  font-size: 13px;
  color: var(--ink-muted);
  line-height: 1.55;
}

.feed-cta {
  padding: 7px 14px;
  border-radius: 12px;
  background: transparent;
  border: 1px solid;
  font-size: 12px;
  font-weight: 600;
  transition: background 160ms ease, color 160ms ease;
}
.feed-cta--teal {
  border-color: var(--accent);
  color: var(--accent);
}
.feed-cta--teal:hover { background: var(--accent); color: #fff; }
.feed-cta--terracotta {
  border-color: var(--warm);
  color: var(--warm);
}
.feed-cta--terracotta:hover { background: var(--warm); color: #fff; }
.feed-cta--muted {
  border-color: var(--border-strong);
  color: var(--ink-muted);
}
.feed-cta--muted:hover { color: var(--ink); }

/* LEADERBOARD ───────────────────────────────────────── */
.card-board { padding: 0 0 22px; display: flex; flex-direction: column; }
.board-list {
  list-style: none;
  margin: 18px 0 0;
  padding: 0 22px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.board-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 12px;
  transition: background 160ms ease;
}
.board-row:hover { background: var(--paper-warm); }

.board-row--you {
  background: color-mix(in srgb, var(--gold) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--gold) 22%, transparent);
  position: relative;
  overflow: hidden;
}
.board-row--you::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--gold);
}

.board-left { display: flex; align-items: center; gap: 12px; }
.board-rank {
  width: 22px;
  text-align: center;
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-muted);
}
.rank-gold { color: var(--gold); }

.board-name {
  font-size: 13px;
  color: var(--ink);
  font-weight: 500;
}
.board-row--you .board-name { font-weight: 700; }

.board-score {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--ink-muted);
  font-weight: 500;
}
.board-row--you .board-score { color: var(--gold); font-weight: 600; }

/* SKELETONS ─────────────────────────────────────────── */
.v2-skeleton-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 22px;
}
.skel {
  background: color-mix(in srgb, var(--border-soft) 60%, transparent);
  border-radius: 24px;
  animation: skelPulse 1.4s ease-in-out infinite;
}
.skel-hero    { grid-column: span 8; height: 280px; }
.skel-quick   { grid-column: span 4; height: 280px; }
.skel-vitals  { grid-column: span 12; height: 92px; }
.skel-mastery { grid-column: span 7; height: 240px; }
.skel-cases   { grid-column: span 5; height: 240px; }
@keyframes skelPulse {
  0%, 100% { opacity: 0.55; }
  50% { opacity: 1; }
}

.v2-error {
  padding: 60px 0;
  text-align: center;
  color: var(--ink-muted);
}
</style>
