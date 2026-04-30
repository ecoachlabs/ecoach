<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watchEffect, watch, nextTick } from 'vue'
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
import MathText from '@/components/question/MathText.vue'
import QuestionFeedback from '@/components/question/QuestionFeedback.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import {
  PhArrowRight,
  PhBookOpen,
  PhClockCountdown,
  PhFlame,
  PhLightning,
  PhTarget,
  PhTrendUp,
  PhBrain,
  PhSword,
  PhGameController,
  PhWarning,
  PhChartLineUp,
  PhRepeat,
  PhCards,
  PhTimer,
  PhCheckCircle,
  PhCrown,
} from '@phosphor-icons/vue'

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
  questionPips: quickCheckPips,
  questionIndex: quickCheckIndex,
  currentQuestion: activeQuickCheck,
  selectedOptionId: selectedQuickOptionId,
  loading: arenaLoading,
  submitting: arenaSubmitting,
  error: arenaError,
  locked: quickAnswered,
  isCorrect: quickSelectionCorrect,
  feedbackPayload: quickFeedback,
  studyRoute: quickStudyRoute,
  weaknessTitle: arenaWeaknessTitle,
  weaknessDetail: arenaWeaknessDetail,
  loadArena,
  submitOption: submitQuickOption,
  markTimedOut: markQuickTimedOut,
  nextQuestion: advanceQuickQuestion,
  isCorrectOption: isQuickOptionCorrect,
  isWrongOption: isQuickOptionWrong,
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

const dateLabel = computed(() => {
  return new Date().toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' })
})

onMounted(async () => {
  if (!auth.currentAccount) { loading.value = false; return }
  try {
    await refreshHomeData()
    await loadArena()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  } finally {
    loading.value = false
    /* Trigger ring + count-up animations after layout settles */
    requestAnimationFrame(() => requestAnimationFrame(() => { ringsActive.value = true }))
  }
})

function startAction() {
  if (nextAction.value?.route) router.push(nextAction.value.route)
}

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
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

/* ────────────────────────────────────────────────────────────────
   Count-up animation — fires once after the dashboard data settles.
   ────────────────────────────────────────────────────────────── */
const ringsActive = ref(false)

function useCountUp(target: () => number, duration = 1100, delayMs = 0) {
  const value = ref(0)
  let raf = 0
  watchEffect(() => {
    const dest = target()
    if (!ringsActive.value) { value.value = 0; return }
    const start = value.value
    const t0 = performance.now() + delayMs
    cancelAnimationFrame(raf)
    const tick = (nowT: number) => {
      const t = Math.max(0, Math.min(1, (nowT - t0) / duration))
      const eased = 1 - Math.pow(1 - t, 3)
      value.value = Math.round(start + (dest - start) * eased)
      if (t < 1) raf = requestAnimationFrame(tick)
    }
    raf = requestAnimationFrame(tick)
  })
  return value
}

const animStreak    = useCountUp(() => vitalStats.value.streak,        950,  100)
const animAccuracy  = useCountUp(() => vitalStats.value.accuracy,     1100,  180)
const animToday     = useCountUp(() => vitalStats.value.todayMin,     1000,  260)
const animWeekQs    = useCountUp(() => vitalStats.value.weekQuestions,1300,  340)

/* ────────────────────────────────────────────────────────────────
   Mastery rings — overall + per-subject. Uses real dashboard data.
   ────────────────────────────────────────────────────────────── */
const RING_HERO_SIZE = 188
const RING_HERO_R = 84
const RING_HERO_W = 5
const RING_HERO_CIRC = 2 * Math.PI * RING_HERO_R

const RING_SUB_SIZE = 44
const RING_SUB_R = 18
const RING_SUB_W = 4
const RING_SUB_CIRC = 2 * Math.PI * RING_SUB_R

interface SubjectRingData {
  id: number
  name: string
  pct: number
  mastered: number
  total: number
  weak: number
  emberFrom: string
  emberTo: string
  glow: string
}

function emberPalette(pct: number): { from: string; to: string; glow: string } {
  if (pct < 13) return { from: '#d6d3d1', to: '#a8a29e', glow: 'rgba(168, 162, 158, 0.30)' }
  if (pct < 26) return { from: '#a8a29e', to: '#78716c', glow: 'rgba(168, 162, 158, 0.45)' }
  if (pct < 41) return { from: '#fde68a', to: '#fbbf24', glow: 'rgba(251, 191, 36, 0.55)' }
  if (pct < 56) return { from: '#fcd34d', to: '#f59e0b', glow: 'rgba(245, 158, 11, 0.60)' }
  if (pct < 71) return { from: '#fb923c', to: '#ea580c', glow: 'rgba(234, 88, 12, 0.62)' }
  if (pct < 84) return { from: '#4ade80', to: '#16a34a', glow: 'rgba(22, 163, 74, 0.55)' }
  if (pct < 94) return { from: '#34d399', to: '#059669', glow: 'rgba(5, 150, 105, 0.60)' }
  return            { from: '#5eead4', to: '#0d9488', glow: 'rgba(13, 148, 136, 0.65)' }
}

const subjectRings = computed<SubjectRingData[]>(() => {
  const subs = dashboard.value?.subjects ?? []
  return subs.map(s => {
    const pct = s.total_topic_count > 0
      ? Math.round((s.mastered_topic_count / s.total_topic_count) * 100)
      : 0
    const ember = emberPalette(pct)
    return {
      id: s.subject_id,
      name: s.subject_name,
      pct,
      mastered: s.mastered_topic_count,
      total: s.total_topic_count,
      weak: s.weak_topic_count,
      emberFrom: ember.from,
      emberTo: ember.to,
      glow: ember.glow,
    }
  })
})

const overallMasteryPct = computed(() => {
  const subs = subjectRings.value
  if (subs.length === 0) return 0
  return Math.round(subs.reduce((s, r) => s + r.pct, 0) / subs.length)
})
const animOverallMastery = useCountUp(() => overallMasteryPct.value, 1500, 200)

const overallReadinessLabel = computed(() => {
  const pct = overallMasteryPct.value
  if (pct < 25)  return 'Building foundation'
  if (pct < 50)  return 'Gaining ground'
  if (pct < 70)  return 'On track'
  if (pct < 85)  return 'Strong'
  return 'Exam-ready'
})

type HeroOutlineRing = SubjectRingData & {
  radius: number
  width: number
  circumference: number
}

const heroOutlineRings = computed<HeroOutlineRing[]>(() => {
  const radii = [66, 52, 38]
  const widths = [4, 4, 3.5]
  return subjectRings.value.slice(0, 3).map((sub, index) => ({
    ...sub,
    radius: radii[index] ?? 38,
    width: widths[index] ?? 3.5,
    circumference: 2 * Math.PI * (radii[index] ?? 38),
  }))
})

type HomeLaunchTile = {
  id: string
  label: string
  detail: string
  route: string
  icon: any
  tone: string
  eyebrow: string
  featured?: boolean
}

const homeLaunchTiles = computed<HomeLaunchTile[]>(() => {
  const topTopic = topicCases.value[0]?.topic_name
  const examTarget = dashboard.value?.exam_target
  const streakDays = vitalStats.value.streak

  return [
    {
      id: 'start-quiz',
      label: 'Start quiz',
      detail: topTopic
        ? `${topTopic} is ready for a real-question run.`
        : 'Warm up with a real-question practice run.',
      route: '/student/practice',
      icon: PhBookOpen,
      tone: '#16a34a',
      eyebrow: 'Practice',
      featured: true,
    },
    {
      id: 'marathon',
      label: 'Marathon',
      detail: streakDays > 0
        ? `${streakDays} day streak on the board. Stretch it further.`
        : 'Build a longer focus run and set a pace.',
      route: '/student/rise',
      icon: PhTrendUp,
      tone: '#ef4444',
      eyebrow: 'Stamina',
    },
    {
      id: 'mock-centre',
      label: 'Mock Centre',
      detail: examTarget
        ? `Run a timed paper against ${examTarget}.`
        : 'Simulate a full paper under timed pressure.',
      route: '/student/mock',
      icon: PhCards,
      tone: '#7c3aed',
      eyebrow: 'Exam',
    },
    {
      id: 'prepare-test',
      label: 'Prepare Test',
      detail: nextAction.value?.estimated_minutes
        ? `${nextAction.value.estimated_minutes} minute guided prep is ready.`
        : 'Set up a guided paper for the next push.',
      route: '/student/mock/setup',
      icon: PhClockCountdown,
      tone: '#ea580c',
      eyebrow: 'Prep',
    },
    {
      id: 'topic-clinic',
      label: 'Topic Clinic',
      detail: topTopic
        ? `Repair ${topTopic} before it becomes expensive.`
        : 'Repair weak topics and active misconceptions.',
      route: '/student/knowledge-gap',
      icon: PhSword,
      tone: '#06b6d4',
      eyebrow: 'Repair',
    },
    {
      id: 'games-hub',
      label: 'Games Hub',
      detail: 'Turn fast drills into repetitions without losing pressure.',
      route: '/student/games',
      icon: PhGameController,
      tone: '#2563eb',
      eyebrow: 'Arcade',
    },
  ]
})

function bandLabel(band: string): string {
  if (band === 'strong') return 'Strong'
  if (band === 'developing') return 'Developing'
  return 'Weak'
}

type FeedTone = 'accent' | 'warm' | 'gold' | 'muted'

type LiveFeedItem = {
  id: string
  tone: FeedTone
  icon: any
  title: string
  detail: string
  time: string
  action: string
  to: string
}

const quickCheckKey = ref(0) // bumped on question change to re-trigger entrance animation

const CORRECT_AUTO_ADVANCE_DELAY_MS = 2400
const SOLIDIFY_PROMPT_MS = 5200
type SolidifyPrompt = { title: string; detail: string; route: string }
const solidifyPrompt = ref<SolidifyPrompt | null>(null)
let arenaAutoAdvanceTimeout: number | null = null
let solidifyPromptTimeout: number | null = null

function clearArenaAutoAdvance() {
  if (arenaAutoAdvanceTimeout !== null) {
    window.clearTimeout(arenaAutoAdvanceTimeout)
    arenaAutoAdvanceTimeout = null
  }
}

function scheduleArenaAutoAdvance(delay: number = CORRECT_AUTO_ADVANCE_DELAY_MS) {
  clearArenaAutoAdvance()
  arenaAutoAdvanceTimeout = window.setTimeout(() => {
    arenaAutoAdvanceTimeout = null
    nextQuickCheck()
  }, delay)
}

function buildSolidifyPrompt(): SolidifyPrompt {
  const weakness = topicCases.value[0]
  if (weakness) {
    return {
      title: weakness.topic_name,
      detail: `${weakness.intervention_mode.replace(/_/g, ' ')} practice is ready.`,
      route: '/student/knowledge-gap',
    }
  }

  return {
    title: arenaWeaknessTitle.value,
    detail: arenaWeaknessDetail.value,
    route: quickStudyRoute.value,
  }
}

function clearSolidifyPromptTimeout() {
  if (solidifyPromptTimeout !== null) {
    window.clearTimeout(solidifyPromptTimeout)
    solidifyPromptTimeout = null
  }
}

function dismissSolidifyPrompt() {
  clearSolidifyPromptTimeout()
  solidifyPrompt.value = null
}

function showSolidifyPrompt() {
  solidifyPrompt.value = buildSolidifyPrompt()
  clearSolidifyPromptTimeout()
  solidifyPromptTimeout = window.setTimeout(() => {
    solidifyPrompt.value = null
    solidifyPromptTimeout = null
  }, SOLIDIFY_PROMPT_MS)
}

function solidifyKnownWeakness() {
  const route = solidifyPrompt.value?.route
  dismissSolidifyPrompt()
  if (route) router.push(route)
}

const liveFeed = computed<LiveFeedItem[]>(() => {
  const items: LiveFeedItem[] = []
  const stats = homeStats.value

  if (nextAction.value) {
    items.push({
      id: 'coach-directive',
      tone: 'accent',
      icon: PhBrain,
      title: nextAction.value.title,
      detail: nextAction.value.subtitle,
      time: 'current',
      action: 'Open',
      to: nextAction.value.route,
    })
  }

  if (topicCases.value.length) {
    const topTopic = topicCases.value[0]
    items.push({
      id: `topic-${topTopic.topic_id}`,
      tone: topTopic.intervention_urgency === 'high' ? 'warm' : 'gold',
      icon: PhWarning,
      title: `${topTopic.topic_name} flagged for recovery`,
      detail: `${topTopic.intervention_mode.replace(/_/g, ' ')} · real weakness signal`,
      time: 'priority',
      action: 'Practice',
      to: '/student/practice',
    })
  }

  if (topicCases.value.length > 1) {
    const followUpTopic = topicCases.value[1]
    items.push({
      id: `topic-follow-up-${followUpTopic.topic_id}`,
      tone: followUpTopic.intervention_urgency === 'high' ? 'warm' : 'gold',
      icon: PhRepeat,
      title: `${followUpTopic.topic_name} is next in line`,
      detail: followUpTopic.intervention_reason,
      time: 'next focus',
      action: 'Drill',
      to: '/student/practice',
    })
  }

  if (dashboard.value?.subjects?.length) {
    const weakest = [...dashboard.value.subjects].sort((a, b) => {
      const aPct = Math.round(a.mastered_topic_count / Math.max(a.total_topic_count, 1) * 100)
      const bPct = Math.round(b.mastered_topic_count / Math.max(b.total_topic_count, 1) * 100)
      return aPct - bPct
    })[0]

    const weakestPct = Math.round(weakest.mastered_topic_count / Math.max(weakest.total_topic_count, 1) * 100)

    items.push({
      id: `subject-${weakest.subject_id}`,
      tone: 'muted',
      icon: PhChartLineUp,
      title: `${weakest.subject_name} needs attention`,
      detail: `${weakestPct}% mastered · ${weakest.weak_topic_count} weak topics`,
      time: 'subject',
      action: 'Review',
      to: '/student/progress',
    })
  }

  if (stats) {
    if (stats.streak_days > 0) {
      items.push({
        id: 'stats-streak',
        tone: 'gold',
        icon: PhFlame,
        title: `${stats.streak_days}-day streak alive`,
        detail: 'Keep the run moving while momentum is still hot',
        time: 'momentum',
        action: 'Continue',
        to: '/student/practice',
      })
    }

    items.push({
      id: 'stats-accuracy',
      tone: stats.accuracy_percent >= 70 ? 'accent' : 'muted',
      icon: PhCheckCircle,
      title: `${stats.accuracy_percent}% answer accuracy`,
      detail: `${stats.correct_attempts} correct from ${stats.total_attempts} recorded attempts`,
      time: 'snapshot',
      action: 'Review',
      to: '/student/progress',
    })

    items.push({
      id: 'stats-volume',
      tone: 'gold',
      icon: PhLightning,
      title: `${stats.week_questions} questions answered this week`,
      detail: `${stats.today_minutes} minutes studied today`,
      time: 'today',
      action: 'Open',
      to: '/student/progress/history',
    })
  }

  if (!items.length) {
    items.push({
      id: 'feed-empty',
      tone: 'muted',
      icon: PhCards,
      title: 'No live signals yet',
      detail: 'Complete a real session to start building the feed.',
      time: 'waiting',
      action: 'Start',
      to: '/student/practice',
    })
  }

  return items.slice(0, 9)
})

async function pickQuickOption(optionId: string) {
  if (quickAnswered.value || arenaSubmitting.value) return
  stopArenaTimer()
  const result = await submitQuickOption(optionId)
  if (!result) {
    startArenaTimer()
    return
  }

  arenaAnswered.value++
  if (result.is_correct) {
    arenaCorrect.value++
    const timeBonus = Math.floor(Math.max(0, arenaTimer.value) * 2)
    const streakBonus = arenaStreak.value * 5
    const points = 50 + timeBonus + streakBonus
    lastPoints.value = points
    arenaScore.value += points
    arenaStreak.value++
    if (arenaStreak.value > arenaBest.value) arenaBest.value = arenaStreak.value
    triggerFloatUp(points)
    scorePulsing.value = true
    window.setTimeout(() => { scorePulsing.value = false }, 620)
    if (arenaStreak.value > 0 && arenaStreak.value % 3 === 0) {
      showArenaToast({
        message: `${arenaStreak.value} in a row!`,
        detail: `${arenaCorrect.value} correct · ${arenaScore.value} pts in this run`,
      })
    }
  } else {
    arenaStreak.value = 0
    lastPoints.value = 0
    showSolidifyPrompt()
    return
  }
  scheduleArenaAutoAdvance(CORRECT_AUTO_ADVANCE_DELAY_MS)
}

async function nextQuickCheck() {
  clearArenaAutoAdvance()
  await advanceQuickQuestion()
  quickCheckKey.value++
  nextTick(() => startArenaTimer())
}

function openQuickCheckTopic() {
  clearArenaAutoAdvance()
  router.push(quickStudyRoute.value)
}

function feedToneColor(tone: FeedTone): string {
  if (tone === 'warm') return 'var(--warm)'
  if (tone === 'gold') return 'var(--gold)'
  if (tone === 'accent') return 'var(--accent)'
  return 'var(--ink-muted)'
}

/* ═══════════════════════════════════════════════════════════════
   LIVE ARENA — real-time scoring, streak combos, run-status pop-ups
   ═══════════════════════════════════════════════════════════════ */

const arenaScore = ref(0)
const animArenaScore = useCountUp(() => arenaScore.value, 620, 0)
const arenaStreak = ref(0)
const arenaBest = ref(0)
const arenaAnswered = ref(0)
const arenaCorrect = ref(0)
const lastPoints = ref(0)

/* Floating "+points" element — key-bumped so each reward re-animates */
const floatAward = ref<{ points: number; id: number } | null>(null)
function triggerFloatUp(points: number) {
  floatAward.value = { points, id: Date.now() }
  window.setTimeout(() => { floatAward.value = null }, 1350)
}

/* Score-pulse flash — CSS class toggled on every point gain */
const scorePulsing = ref(false)

/* Per-question countdown — 20s, generous. Timeout = 0 points + streak break */
const ARENA_TIMER_SECONDS = 20
const arenaTimer = ref(ARENA_TIMER_SECONDS)
let arenaTimerInterval: number | null = null
function startArenaTimer() {
  if (!activeQuickCheck.value || quickAnswered.value || arenaLoading.value) return
  stopArenaTimer()
  arenaTimer.value = ARENA_TIMER_SECONDS
  arenaTimerInterval = window.setInterval(() => {
    arenaTimer.value = Math.max(0, +(arenaTimer.value - 0.1).toFixed(1))
    if (arenaTimer.value <= 0) {
      stopArenaTimer()
      if (!quickAnswered.value) {
        commitQuickTimeout()
      }
    }
  }, 100)
}
function stopArenaTimer() {
  if (arenaTimerInterval) {
    clearInterval(arenaTimerInterval)
    arenaTimerInterval = null
  }
}

/* Session toast payload — derived only from the learner's current run. */
const arenaToast = ref<null | { message: string; detail: string }>(null)
let toastTimeout: number | null = null
function showArenaToast(payload: { message: string; detail: string }) {
  arenaToast.value = payload
  if (toastTimeout) clearTimeout(toastTimeout)
  toastTimeout = window.setTimeout(() => { arenaToast.value = null }, 5200)
}
function dismissArenaToast() {
  arenaToast.value = null
  if (toastTimeout) { clearTimeout(toastTimeout); toastTimeout = null }
}

/* Periodic run pulse — every 55s, surface current run truth if idle. */
let periodicToastInterval: number | null = null
function startPeriodicToasts() {
  if (periodicToastInterval) clearInterval(periodicToastInterval)
  periodicToastInterval = window.setInterval(() => {
    if (!arenaToast.value && !quickAnswered.value) {
      showArenaToast({
        message: arenaAnswered.value === 0 ? 'Quick drill ready' : `${arenaAnswered.value} answered`,
        detail: arenaAnswered.value === 0
          ? 'A real question is live. Take the first shot.'
          : `${arenaCorrect.value} correct · ${arenaStreak.value} current streak`,
      })
    }
  }, 55000)
}

/* Central answer handler — fires once when an answer is committed. */
function commitQuickTimeout() {
  if (quickAnswered.value) return
  stopArenaTimer()
  markQuickTimedOut()
  arenaAnswered.value++

  arenaStreak.value = 0
  lastPoints.value = 0
  showSolidifyPrompt()
}

/* Question change → reset timer */
watch(quickCheckIndex, () => { startArenaTimer() })

/* First load → kick off timer + periodic toasts once data is ready */
watch(loading, (val) => {
  if (!val) {
    nextTick(() => {
      startArenaTimer()
      startPeriodicToasts()
    })
  }
})

onUnmounted(() => {
  stopArenaTimer()
  clearArenaAutoAdvance()
  dismissSolidifyPrompt()
  if (toastTimeout) clearTimeout(toastTimeout)
  if (periodicToastInterval) clearInterval(periodicToastInterval)
})

/* ═══════════════════════════════════════════════════════════════
   RESIZABLE DIVIDER — drag to resize Arena vs. Activity, persisted
   ═══════════════════════════════════════════════════════════════ */
const HOME_RAIL_SPLIT_KEY = 'ecoach.home.split.v2'
const HOME_RAIL_DEFAULT_SPLIT = 0.64
const splitRatio = ref(HOME_RAIL_DEFAULT_SPLIT)
try {
  const saved = parseFloat(localStorage.getItem(HOME_RAIL_SPLIT_KEY) ?? String(HOME_RAIL_DEFAULT_SPLIT))
  if (!Number.isNaN(saved) && saved >= 0.22 && saved <= 0.78) splitRatio.value = saved
} catch { /* localStorage unavailable */ }

const rpBodyRef = ref<HTMLElement | null>(null)
let dividerDragging = false

function onDividerPointerDown(e: PointerEvent) {
  dividerDragging = true
  document.body.style.cursor = 'row-resize'
  document.body.style.userSelect = 'none'
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  e.preventDefault()
}
function onDividerPointerMove(e: PointerEvent) {
  if (!dividerDragging || !rpBodyRef.value) return
  const rect = rpBodyRef.value.getBoundingClientRect()
  const y = e.clientY - rect.top
  const ratio = Math.max(0.22, Math.min(0.78, y / rect.height))
  splitRatio.value = ratio
}
function onDividerPointerUp() {
  if (!dividerDragging) return
  dividerDragging = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  try { localStorage.setItem(HOME_RAIL_SPLIT_KEY, String(splitRatio.value)) } catch { /* noop */ }
}
</script>

<template>
  <div class="h-full flex overflow-hidden relative" :style="{ backgroundColor: 'var(--paper)' }">
    <!-- LEFT COLUMN — greeting + scrollable main content.
         Greeting now lives inside this column so it no longer stretches across
         the whole viewport; the right panel (Arena) extends up to the same top. -->
    <div class="flex-1 flex flex-col overflow-hidden min-w-0">

      <!-- Greeting card — compact rounded card, not full-width -->
      <div class="greeting-card">
        <div class="greeting-main">
          <p class="greeting-date">{{ dateLabel }}</p>
          <h1 class="greeting-title font-display">
            {{ greeting }}, {{ dashboard?.student_name || auth.currentAccount?.display_name || 'Student' }}
          </h1>
        </div>
        <div v-if="dashboard?.subjects?.length" class="greeting-subjects">
          <div
            v-for="s in dashboard.subjects.slice(0, 3)"
            :key="s.subject_id"
            class="greeting-subject"
          >
            <p class="greeting-subject-pct">
              {{ Math.round(s.mastered_topic_count / Math.max(s.total_topic_count, 1) * 100) }}%
            </p>
            <p class="greeting-subject-name">{{ s.subject_name }}</p>
          </div>
        </div>
      </div>

      <!-- Loading skeleton — left column only (rp renders independently) -->
      <div v-if="loading" class="flex-1 p-7 space-y-4 overflow-hidden">
        <div class="h-48 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        <div class="h-32 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        <div class="h-44 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

    <!-- Error -->
    <div v-else-if="error" class="flex-1 flex items-center justify-center">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">{{ error }}</p>
    </div>

      <!-- Main content (left column only) — directive + vitals + mastery + topics -->
      <div v-else class="flex-1 overflow-y-auto p-7 space-y-5">

        <!-- Coach directive -->
        <div v-if="nextAction"
          class="directive-card rounded-2xl border cursor-pointer"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
          @click="startAction"
        >
          <div class="directive-inner p-6">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-3">
                  <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider"
                    :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">
                    <PhLightning :size="9" weight="fill" />
                    Today's Focus
                  </div>
                </div>
                <h2 class="font-display text-2xl font-bold leading-snug mb-2" :style="{ color: 'var(--ink)' }">
                  {{ nextAction.title }}
                </h2>
                <p class="text-sm leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">
                  {{ nextAction.subtitle }}
                </p>
              </div>
              <div v-if="nextAction.estimated_minutes"
                class="flex-shrink-0 text-center px-3 py-2 rounded-xl border"
                :style="{ borderColor: 'transparent' }">
                <p class="text-lg font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ nextAction.estimated_minutes }}</p>
                <p class="text-[9px] uppercase font-bold tracking-wide" :style="{ color: 'var(--ink-muted)' }">min</p>
              </div>
            </div>
            <div class="mt-5 flex items-center gap-3">
              <button
                class="cta-btn flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-bold"
                :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
                @click.stop="startAction"
              >
                Start Now
                <PhArrowRight :size="14" weight="bold" />
              </button>
              <span class="text-xs capitalize px-3 py-1.5 rounded-lg"
                :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }">
                {{ nextAction.action_type?.replace(/_/g, ' ') }}
              </span>
            </div>
          </div>
        </div>

        <!-- Empty state if no directive -->
        <div v-else
          class="rounded-2xl border p-8 text-center"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <p class="font-display text-lg font-bold mb-1" :style="{ color: 'var(--ink)' }">All caught up</p>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No urgent focus right now. Keep practising or open Coach Hub for a deeper plan.</p>
        </div>

        <!-- ─────────────── Vitals strip — 4 stat tiles ─────────────── -->
        <div class="vitals-grid">
          <article class="vital-tile" style="--tone: #ea580c;">
            <div class="vital-glyph"><PhFlame :size="14" weight="fill" /></div>
            <div class="vital-body">
              <p class="vital-num font-display">{{ animStreak }}<span class="vital-unit">d</span></p>
              <p class="vital-label">Streak</p>
            </div>
            <p class="vital-foot">consecutive days practised</p>
          </article>

          <article class="vital-tile" style="--tone: #0d9488;">
            <div class="vital-glyph"><PhCheckCircle :size="14" weight="fill" /></div>
            <div class="vital-body">
              <p class="vital-num font-display">{{ animAccuracy }}<span class="vital-unit">%</span></p>
              <p class="vital-label">Accuracy</p>
            </div>
            <p class="vital-foot">last 7 days · all attempts</p>
          </article>

          <article class="vital-tile" style="--tone: #b45309;">
            <div class="vital-glyph"><PhTimer :size="14" weight="fill" /></div>
            <div class="vital-body">
              <p class="vital-num font-display">{{ animToday }}<span class="vital-unit">m</span></p>
              <p class="vital-label">Today</p>
            </div>
            <p class="vital-foot">minutes studied so far</p>
          </article>

          <article class="vital-tile" style="--tone: #7c3aed;">
            <div class="vital-glyph"><PhTarget :size="14" weight="fill" /></div>
            <div class="vital-body">
              <p class="vital-num font-display">{{ animWeekQs }}</p>
              <p class="vital-label">This week</p>
            </div>
            <p class="vital-foot">questions answered</p>
          </article>
        </div>

        <!-- ─────────────── Mastery panel — big ring + per-subject mini-rings ─────────────── -->
        <section
          class="mastery-panel rounded-2xl"
          :style="{ backgroundColor: 'var(--surface)' }"
        >
          <div class="mastery-head">
            <p class="section-label">Mastery</p>
            <span v-if="dashboard?.exam_target" class="mastery-target">
              Target: <strong>{{ dashboard.exam_target }}</strong>
            </span>
          </div>

            <div class="mastery-body">
            <!-- LEFT: concentric outline rings -->
            <div class="mastery-hero">
              <svg
                class="mastery-hero-svg"
                :width="RING_HERO_SIZE"
                :height="RING_HERO_SIZE"
                :viewBox="`0 0 ${RING_HERO_SIZE} ${RING_HERO_SIZE}`"
                aria-hidden="true"
              >
                <defs>
                  <linearGradient id="mastery-hero-grad" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0%"   stop-color="#5eead4" />
                    <stop offset="100%" stop-color="#0d9488" />
                  </linearGradient>
                  <template v-for="ring in heroOutlineRings" :key="`hero-grad-${ring.id}`">
                    <linearGradient :id="`hero-grad-${ring.id}`" x1="0" y1="0" x2="1" y2="1">
                      <stop offset="0%" :stop-color="ring.emberFrom" />
                      <stop offset="100%" :stop-color="ring.emberTo" />
                    </linearGradient>
                  </template>
                </defs>
                <circle
                  :cx="RING_HERO_SIZE / 2"
                  :cy="RING_HERO_SIZE / 2"
                  :r="RING_HERO_R"
                  fill="none"
                  stroke="var(--border-soft)"
                  :stroke-width="RING_HERO_W"
                />
                <circle
                  class="ring-arc ring-arc--hero"
                  :cx="RING_HERO_SIZE / 2"
                  :cy="RING_HERO_SIZE / 2"
                  :r="RING_HERO_R"
                  fill="none"
                  stroke="url(#mastery-hero-grad)"
                  :stroke-width="RING_HERO_W"
                  stroke-linecap="round"
                  :stroke-dasharray="RING_HERO_CIRC"
                  :stroke-dashoffset="ringsActive
                    ? RING_HERO_CIRC * (1 - Math.min(1, overallMasteryPct / 100))
                    : RING_HERO_CIRC"
                  :transform="`rotate(-90 ${RING_HERO_SIZE / 2} ${RING_HERO_SIZE / 2})`"
                />
                <template v-for="ring in heroOutlineRings" :key="ring.id">
                  <circle
                    class="mastery-nest-track"
                    :cx="RING_HERO_SIZE / 2"
                    :cy="RING_HERO_SIZE / 2"
                    :r="ring.radius"
                    fill="none"
                    stroke="var(--border-soft)"
                    :stroke-width="ring.width"
                  />
                  <circle
                    class="ring-arc ring-arc--nest"
                    :cx="RING_HERO_SIZE / 2"
                    :cy="RING_HERO_SIZE / 2"
                    :r="ring.radius"
                    fill="none"
                    :stroke="`url(#hero-grad-${ring.id})`"
                    :stroke-width="ring.width"
                    stroke-linecap="round"
                    :stroke-dasharray="ring.circumference"
                    :stroke-dashoffset="ringsActive
                      ? ring.circumference * (1 - Math.min(1, ring.pct / 100))
                      : ring.circumference"
                    :transform="`rotate(-90 ${RING_HERO_SIZE / 2} ${RING_HERO_SIZE / 2})`"
                    :style="{ '--glow': ring.glow }"
                  />
                </template>
              </svg>
              <div class="mastery-hero-center">
                <p class="mastery-hero-num font-display">
                  {{ animOverallMastery }}<span class="mastery-hero-pct">%</span>
                </p>
                <p class="mastery-hero-eyebrow">Overall</p>
              </div>
              <p class="mastery-hero-band">{{ overallReadinessLabel }}</p>
              <div v-if="heroOutlineRings.length" class="mastery-hero-legend">
                <div
                  v-for="ring in heroOutlineRings"
                  :key="`hero-legend-${ring.id}`"
                  class="mastery-legend-chip"
                >
                  <span
                    class="mastery-legend-swatch"
                    :style="{ '--sw-from': ring.emberFrom, '--sw-to': ring.emberTo, boxShadow: `0 0 10px ${ring.glow}` }"
                  />
                  <span class="mastery-legend-name">{{ ring.name }}</span>
                  <span class="mastery-legend-pct">{{ ring.pct }}%</span>
                </div>
              </div>
            </div>

            <!-- RIGHT: per-subject mini-rings list -->
            <div class="mastery-subjects">
              <template v-if="subjectRings.length > 0">
                <div
                  v-for="sub in subjectRings.slice(0, 5)"
                  :key="sub.id"
                  class="subject-row"
                >
                  <div class="subject-ring-wrap">
                    <svg
                      class="subject-ring-svg"
                      :width="RING_SUB_SIZE"
                      :height="RING_SUB_SIZE"
                      :viewBox="`0 0 ${RING_SUB_SIZE} ${RING_SUB_SIZE}`"
                      aria-hidden="true"
                    >
                      <defs>
                        <linearGradient :id="`sub-grad-${sub.id}`" x1="0" y1="0" x2="1" y2="1">
                          <stop offset="0%"   :stop-color="sub.emberFrom" />
                          <stop offset="100%" :stop-color="sub.emberTo" />
                        </linearGradient>
                      </defs>
                      <circle
                        :cx="RING_SUB_SIZE / 2"
                        :cy="RING_SUB_SIZE / 2"
                        :r="RING_SUB_R"
                        fill="none"
                        stroke="var(--border-soft)"
                        :stroke-width="RING_SUB_W"
                      />
                      <circle
                        class="ring-arc ring-arc--sub"
                        :cx="RING_SUB_SIZE / 2"
                        :cy="RING_SUB_SIZE / 2"
                        :r="RING_SUB_R"
                        fill="none"
                        :stroke="`url(#sub-grad-${sub.id})`"
                        :stroke-width="RING_SUB_W"
                        stroke-linecap="round"
                        :stroke-dasharray="RING_SUB_CIRC"
                        :stroke-dashoffset="ringsActive
                          ? RING_SUB_CIRC * (1 - Math.min(1, sub.pct / 100))
                          : RING_SUB_CIRC"
                        :transform="`rotate(-90 ${RING_SUB_SIZE / 2} ${RING_SUB_SIZE / 2})`"
                        :style="{ '--glow': sub.glow }"
                      />
                    </svg>
                    <span class="subject-ring-pct font-display">{{ sub.pct }}</span>
                  </div>
                  <div class="subject-meta">
                    <p class="subject-name">{{ sub.name }}</p>
                    <p class="subject-counts">
                      <strong>{{ sub.mastered }}</strong> of {{ sub.total }} mastered
                      <span v-if="sub.weak > 0" class="subject-weak"> · {{ sub.weak }} weak</span>
                    </p>
                  </div>
                </div>
              </template>
              <div v-else class="subjects-empty">
                <p class="subjects-empty-title">No subjects tracked yet</p>
                <p class="subjects-empty-sub">Finish onboarding to unlock mastery tracking across your subjects.</p>
              </div>
            </div>
          </div>
        </section>

        <!-- Priority topics -->
        <div v-if="topicCases.length">
          <p class="section-label mb-3">Priority Topics</p>
          <div class="space-y-1.5">
            <div
              v-for="topic in topicCases"
              :key="topic.topic_id"
              class="topic-row flex items-center gap-3 px-4 py-3 rounded-xl border cursor-pointer"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              @click="router.push('/student/practice')"
            >
              <MasteryBadge :state="topic.mastery_state" size="sm" />
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                <p class="text-[10px] capitalize" :style="{ color: 'var(--ink-muted)' }">
                  {{ topic.intervention_mode.replace(/_/g, ' ') }} · gap {{ formatBp(topic.gap_score) }}
                </p>
              </div>
              <span
                class="text-[9px] font-bold uppercase px-2 py-0.5 rounded-full"
                :style="{
                  backgroundColor: topic.intervention_urgency === 'high' ? 'rgba(194,65,12,0.1)' : 'var(--paper)',
                  color: topic.intervention_urgency === 'high' ? 'var(--warm)' : 'var(--ink-muted)',
                }"
              >{{ topic.intervention_urgency }}</span>
            </div>
          </div>
        </div>

        <section class="launch-panel">
          <div class="launch-head">
            <p class="section-label">Jump In</p>
            <button class="launch-inline-link" @click="router.push('/student/coach')">
              Coach Hub <PhArrowRight :size="12" weight="bold" />
            </button>
          </div>
          <div class="launch-grid">
            <button
              v-for="tile in homeLaunchTiles"
              :key="tile.id"
              class="launch-tile"
              :class="{ 'launch-tile--wide': tile.featured }"
              :style="{ '--launch-tone': tile.tone }"
              @click="router.push(tile.route)"
            >
              <div class="launch-tile-head">
                <span class="launch-tile-kicker">{{ tile.eyebrow }}</span>
                <span class="launch-tile-icon">
                  <component :is="tile.icon" :size="16" weight="fill" />
                </span>
              </div>
              <div class="launch-tile-body">
                <h3 class="launch-tile-title">{{ tile.label }}</h3>
                <p class="launch-tile-detail">{{ tile.detail }}</p>
              </div>
              <div class="launch-tile-cta">
                <span>Open</span>
                <PhArrowRight :size="12" weight="bold" />
              </div>
            </button>
          </div>
        </section>
      </div>

    </div>  <!-- /LEFT COLUMN -->

    <!-- RIGHT PANEL — Arena + Divider + Activity (full height, sibling of left col) -->
    <div class="rp" ref="rpBodyRef">

        <!-- ══════════ ARENA — dark, live scoring ══════════ -->
        <section class="arena" :style="{ flex: `${splitRatio} 1 0` }">

          <!-- Top strip: label + rank chip -->
          <div class="arena-head">
            <div class="arena-head-left">
              <span class="arena-label">Live Arena</span>
              <span class="arena-sublabel">Quick drills · real-time scoring</span>
            </div>
            <div class="arena-rank" title="Questions answered in this run">
              <PhTarget :size="10" weight="fill" />
              <span>{{ arenaAnswered }}</span>
            </div>
          </div>

          <!-- Score + streak -->
          <div class="arena-score-strip">
            <div class="arena-score">
              <span
                class="arena-score-num font-display"
                :class="{ 'arena-score-num--pulse': scorePulsing }"
              >{{ animArenaScore }}</span>
              <span class="arena-score-label">pts</span>
            </div>

            <div class="arena-streak">
              <div class="arena-streak-meta">
                <PhFlame :size="11" weight="fill" class="arena-streak-flame"
                  :class="{ 'arena-streak-flame--hot': arenaStreak >= 3 }" />
                <span class="arena-streak-num">{{ arenaStreak }}</span>
                <span class="arena-streak-word">streak</span>
                <span v-if="arenaBest > 0" class="arena-streak-best">best {{ arenaBest }}</span>
              </div>
              <div class="arena-streak-bar">
                <div class="arena-streak-fill"
                  :style="{ width: Math.min(100, arenaStreak * 18) + '%' }" />
              </div>
            </div>
          </div>

          <!-- Timer bar -->
          <div class="arena-timer">
            <div class="arena-timer-fill"
              :class="{ 'arena-timer-fill--low': arenaTimer < 6, 'arena-timer-fill--done': quickAnswered }"
              :style="{ width: (arenaTimer / ARENA_TIMER_SECONDS) * 100 + '%' }" />
          </div>

          <!-- Scrollable question area (accommodates longer content) -->
          <div class="arena-scroll">
            <div class="arena-topmeta">
              <span class="arena-topic-chip">{{ activeQuickCheck?.topic || 'Real questions' }}</span>
              <div class="rp-steps">
                <span v-for="(_, i) in quickCheckPips" :key="i"
                  class="rp-step" :class="{ 'rp-step--on': i === quickCheckIndex }" />
              </div>
            </div>

            <p class="arena-question">
              <MathText
                v-if="activeQuickCheck?.prompt"
                :text="activeQuickCheck.prompt"
              />
              <span v-else>{{ arenaError || 'Loading real questions...' }}</span>
            </p>

            <div v-if="activeQuickCheck" class="arena-options">
              <button
                v-for="option in activeQuickCheck.options" :key="option.id"
                class="arena-option"
                :class="{
                  'arena-option--correct': quickAnswered && isQuickOptionCorrect(option),
                  'arena-option--wrong':   quickAnswered && isQuickOptionWrong(option),
                  'arena-option--faded':   quickAnswered && !isQuickOptionCorrect(option) && selectedQuickOptionId !== option.id,
                }"
                :disabled="quickAnswered || arenaSubmitting"
                @click="pickQuickOption(option.id)"
              >
                <span class="arena-letter">{{ option.letter }}</span>
                <span class="arena-option-text"><MathText :text="option.label" /></span>
              </button>
            </div>

            <Transition name="arena-reveal">
              <div v-if="quickAnswered && quickFeedback" class="arena-explain"
                :class="quickSelectionCorrect ? 'arena-explain--ok' : 'arena-explain--err'">
                <span class="arena-explain-badge">
                  <template v-if="quickSelectionCorrect">
                    +{{ lastPoints }}<template v-if="arenaStreak > 1"> · {{ arenaStreak }}× combo</template>
                  </template>
                  <template v-else-if="selectedQuickOptionId === '__timeout__'">
                    Time up
                  </template>
                  <template v-else>
                    Streak broken
                  </template>
                </span>
                <QuestionFeedback
                  v-bind="quickFeedback"
                  :show-review-action="true"
                  review-label="Study Topic"
                  @review="openQuickCheckTopic"
                  @next="nextQuickCheck"
                />
                <div v-if="false" class="arena-explain-actions">
                  <button class="arena-btn-ghost" @click="openQuickCheckTopic">Study topic</button>
                  <button class="arena-btn-solid" @click="nextQuickCheck">Next →</button>
                </div>
              </div>
            </Transition>
          </div>

          <!-- Floating "+pts" reward -->
          <Transition>
            <div
              v-if="floatAward"
              :key="floatAward.id"
              class="arena-floatup font-display"
            >+{{ floatAward.points }}</div>
          </Transition>

          <!-- Run-status toast (absolute inside arena) -->
          <Transition name="lb-toast">
            <div v-if="arenaToast" class="lb-toast" @click="dismissArenaToast">
              <div class="lb-toast-head">
                <PhCrown :size="11" weight="fill" />
                <span class="lb-toast-title">{{ arenaToast.message }}</span>
                <span class="lb-toast-close" aria-label="Dismiss">✕</span>
              </div>
              <div class="lb-toast-body">
                <p class="lb-toast-copy">{{ arenaToast.detail }}</p>
              </div>
            </div>
          </Transition>

          <Transition name="solidify-pop">
            <div
              v-if="solidifyPrompt"
              class="solidify-pop"
              role="dialog"
              aria-live="polite"
              aria-label="Known weakness"
            >
              <p class="solidify-pop-kicker">Known weakness</p>
              <h3 class="solidify-pop-title">Solidify {{ solidifyPrompt.title }}?</h3>
              <p class="solidify-pop-text">{{ solidifyPrompt.detail }}</p>
              <div class="solidify-pop-actions">
                <button class="solidify-pop-secondary" @click="dismissSolidifyPrompt">Not now</button>
                <button class="solidify-pop-primary" @click="solidifyKnownWeakness">Solidify</button>
              </div>
            </div>
          </Transition>
        </section>

        <!-- ══════════ DIVIDER — drag to resize ══════════ -->
        <div
          class="rp-divider"
          role="separator"
          aria-orientation="horizontal"
          title="Drag to resize"
          @pointerdown="onDividerPointerDown"
          @pointermove="onDividerPointerMove"
          @pointerup="onDividerPointerUp"
          @pointercancel="onDividerPointerUp"
        >
          <div class="rp-divider-grip"><span /><span /><span /></div>
        </div>

        <!-- ══════════ RECENT ACTIVITY ══════════ -->
        <div class="rp-activity" :style="{ flex: `${1 - splitRatio} 1 0` }">

          <!-- Section header -->
          <div class="rp-act-header">
            <div class="rp-act-header-left">
              <span class="rp-pulse-ring" />
              <p class="rp-section-label">Live Feed</p>
            </div>
            <button class="rp-act-all" @click="router.push('/student/progress')">All <PhArrowRight :size="9" weight="bold" /></button>
          </div>

          <!-- Continue card (hero) -->
          <div
            v-if="liveFeed[0]"
            class="rp-continue"
            @click="router.push(liveFeed[0].to)"
          >
            <div class="rp-continue-top">
              <span class="rp-continue-label">Continue</span>
              <span class="rp-continue-time">{{ liveFeed[0].time }}</span>
            </div>
            <div class="rp-continue-body">
              <span class="rp-continue-icon-wrap" :style="{ background: feedToneColor(liveFeed[0].tone) + '28' }">
                <component :is="liveFeed[0].icon" :size="16" weight="fill" :style="{ color: feedToneColor(liveFeed[0].tone) }" />
              </span>
              <div class="rp-continue-text">
                <p class="rp-continue-title">{{ liveFeed[0].title }}</p>
                <p class="rp-continue-sub">{{ liveFeed[0].detail }}</p>
              </div>
            </div>
            <button class="rp-continue-cta" :style="{ color: feedToneColor(liveFeed[0].tone) }">
              {{ liveFeed[0].action }} <PhArrowRight :size="11" weight="bold" />
            </button>
          </div>

          <!-- Timeline feed -->
          <div class="rp-feed">
            <div
              v-for="item in liveFeed.slice(1, 8)" :key="item.id"
              class="rp-feed-item"
              :style="{ '--fc': feedToneColor(item.tone) }"
              @click="router.push(item.to)"
            >
              <!-- Timeline dot -->
              <div class="rp-feed-track">
                <span class="rp-feed-dot" :style="{ background: feedToneColor(item.tone) }" />
                <span class="rp-feed-line" />
              </div>
              <!-- Content -->
              <div class="rp-feed-body">
                <div class="rp-feed-title-row">
                  <component :is="item.icon" :size="12" weight="fill" :style="{ color: feedToneColor(item.tone), flexShrink: 0 }" />
                  <span class="rp-feed-title">{{ item.title }}</span>
                </div>
                <p class="rp-feed-detail">{{ item.detail }}</p>
                <div class="rp-feed-footer">
                  <span class="rp-feed-time">{{ item.time }}</span>
                  <button class="rp-feed-cta" :style="{ color: feedToneColor(item.tone) }">
                    {{ item.action }} →
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

    </div>  <!-- /.rp -->
  </div>  <!-- /outer -->
</template>

<style scoped>
.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

/* ── Greeting — compact rounded card, doesn't span full width ── */
.greeting-card {
  align-self: flex-start;       /* don't stretch across the column */
  margin: 16px 20px 0;
  padding: 12px 22px;
  border-radius: 18px;
  background: var(--surface);
  box-shadow:
    0 1px 2px rgba(26, 22, 18, 0.04),
    0 4px 12px rgba(26, 22, 18, 0.04);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 36px;
  flex-shrink: 0;
  max-width: calc(100% - 40px);
}
.greeting-main { min-width: 0; }
.greeting-date {
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-muted);
  letter-spacing: 0.02em;
  margin: 0;
}
.greeting-title {
  font-size: 22px;
  font-weight: 400;
  color: var(--ink);
  letter-spacing: -0.01em;
  line-height: 1.15;
  margin: 2px 0 0;
  text-wrap: balance;
}
.greeting-subjects {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-shrink: 0;
}
.greeting-subject { text-align: center; }
.greeting-subject-pct {
  font-size: 16px;
  font-weight: 700;
  color: var(--ink);
  font-feature-settings: 'tnum';
  margin: 0;
  letter-spacing: -0.01em;
}
.greeting-subject-name {
  font-size: 9px;
  font-weight: 700;
  color: var(--ink-muted);
  text-transform: uppercase;
  letter-spacing: 0.14em;
  margin: 2px 0 0;
}

.directive-card {
  transition: box-shadow 140ms ease, transform 140ms ease;
}
.directive-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06);
}

.cta-btn {
  transition: opacity 120ms ease, transform 120ms ease;
}
.cta-btn:hover { opacity: 0.88; transform: translateY(-1px); }

.topic-row {
  transition: background-color 100ms ease;
}
.topic-row:hover {
  background-color: var(--paper) !important;
}

/* ── Right panel — floating rounded card that holds Arena + Divider + Activity ── */
.rp {
  width: 22rem;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin: 14px 14px 14px 0;
  border-radius: 20px;
  background: var(--surface);
  box-shadow:
    0 2px 6px rgba(26, 22, 18, 0.04),
    0 14px 36px rgba(26, 22, 18, 0.06);
}

/* ── Shared labels / step dots (used in Arena + Activity) ── */
.rp-section-label {
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
}
.rp-steps {
  display: flex;
  gap: 4px;
  align-items: center;
}
.rp-step {
  height: 3px;
  width: 14px;
  border-radius: 99px;
  background: rgba(245, 243, 240, 0.14);
  transition: width 280ms ease, background 280ms ease;
}
.rp-step--on { width: 22px; background: #fbbf24; box-shadow: 0 0 8px rgba(251, 191, 36, 0.5); }

/* ═════════════════════════════════════════════════════════════
   ARENA — light practice zone, integrated with the design system
   (was a black/gold game panel; now a calm warm surface that
    belongs in the same room as the rest of eCoach)
   ═════════════════════════════════════════════════════════════ */
.arena {
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 220px;
  background: linear-gradient(180deg, var(--surface) 0%, var(--paper-warm) 100%);
  color: var(--ink);
}
.arena::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 88% 4%, rgba(194, 65, 12, 0.06), transparent 42%),
    radial-gradient(circle at 2% 96%, rgba(13, 148, 136, 0.04), transparent 40%);
  pointer-events: none;
  z-index: 0;
}

.arena-head {
  padding: 14px 18px 2px;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 10px;
  position: relative;
  z-index: 2;
}
.arena-head-left { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.arena-label {
  font-size: 9.5px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.22em;
  color: var(--warm);
}
.arena-sublabel {
  font-size: 9.5px;
  color: var(--ink-muted);
  letter-spacing: 0.02em;
}

.arena-rank {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 5px 11px;
  border-radius: 99px;
  background: var(--warm-glow);
  color: var(--warm);
  font-size: 10.5px; font-weight: 800;
  font-feature-settings: 'tnum';
  cursor: pointer;
  border: none;
  transition: background 140ms, transform 140ms;
  flex-shrink: 0;
}
.arena-rank:hover { background: rgba(194, 65, 12, 0.14); transform: translateY(-1px); }

/* Score + streak strip */
.arena-score-strip {
  padding: 4px 18px 12px;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 14px;
  position: relative;
  z-index: 2;
}
.arena-score { display: flex; align-items: baseline; gap: 6px; min-width: 0; }
.arena-score-num {
  font-size: 32px;
  font-weight: 400;
  letter-spacing: -0.02em;
  line-height: 1;
  color: var(--warm);
  font-feature-settings: 'tnum';
  transition: color 220ms, transform 220ms;
}
.arena-score-num--pulse { animation: arena-score-pulse 620ms ease-out; }
@keyframes arena-score-pulse {
  0%   { transform: scale(1.08); color: var(--gold); }
  100% { transform: scale(1);    color: var(--warm); }
}
.arena-score-label {
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  padding-bottom: 5px;
}

.arena-streak {
  display: flex;
  flex-direction: column;
  gap: 5px;
  flex: 1;
  max-width: 154px;
  min-width: 112px;
}
.arena-streak-meta {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-secondary);
  font-feature-settings: 'tnum';
}
.arena-streak-flame {
  color: var(--ink-muted);
  transition: color 260ms;
}
.arena-streak-flame--hot {
  color: var(--warm);
  animation: flame-flicker 1.8s ease-in-out infinite;
}
@keyframes flame-flicker {
  0%, 100% { transform: scale(1) rotate(-2deg); }
  50%      { transform: scale(1.08) rotate(2deg); }
}
.arena-streak-num { font-size: 12px; font-weight: 800; color: var(--ink); }
.arena-streak-word {
  text-transform: uppercase;
  letter-spacing: 0.10em;
  font-size: 9px;
  color: var(--ink-muted);
}
.arena-streak-best {
  margin-left: auto;
  font-size: 9.5px;
  font-weight: 600;
  color: var(--ink-muted);
  letter-spacing: 0.04em;
}

.arena-streak-bar {
  height: 3px;
  border-radius: 99px;
  background: var(--border-soft);
  overflow: hidden;
}
.arena-streak-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--ember-warming) 0%, var(--ember-kindling) 100%);
  transition: width 520ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* Timer bar */
.arena-timer {
  height: 2px;
  background: var(--border-soft);
  position: relative;
  z-index: 2;
  overflow: hidden;
}
.arena-timer-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--warm), var(--gold));
  transition: width 100ms linear, background 260ms;
}
.arena-timer-fill--low {
  background: linear-gradient(90deg, #fb923c, var(--danger));
  animation: timer-warn 1.0s ease-in-out infinite;
}
@keyframes timer-warn {
  0%, 100% { opacity: 0.85; }
  50%      { opacity: 1.0; }
}
.arena-timer-fill--done { opacity: 0.35; animation: none; }

/* Scrollable body */
.arena-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px 18px 16px;
  position: relative;
  z-index: 2;
  scrollbar-width: thin;
  scrollbar-color: var(--border-strong) transparent;
}
.arena-scroll::-webkit-scrollbar { width: 5px; }
.arena-scroll::-webkit-scrollbar-track { background: transparent; }
.arena-scroll::-webkit-scrollbar-thumb {
  background: var(--border-strong);
  border-radius: 99px;
}

.arena-topmeta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 11px;
}
.arena-topic-chip {
  display: inline-block;
  padding: 4px 11px;
  border-radius: 99px;
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  background: var(--accent-glow);
  color: var(--accent);
}

.arena-question {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: 400;
  line-height: 1.38;
  color: var(--ink);
  margin-bottom: 14px;
  letter-spacing: -0.005em;
  text-wrap: balance;
}

.arena-options { display: flex; flex-direction: column; gap: 7px; }

.arena-option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 11px 14px;
  border-radius: 12px;
  background: var(--surface);
  border: none;
  box-shadow:
    0 1px 2px rgba(26, 22, 18, 0.05),
    0 2px 7px rgba(26, 22, 18, 0.05);
  cursor: pointer;
  text-align: left;
  color: var(--ink);
  transition:
    transform 160ms ease,
    background 160ms ease,
    box-shadow 160ms ease,
    opacity 220ms ease;
}
.arena-option:hover:not(:disabled) {
  transform: translateX(4px);
  box-shadow:
    0 5px 16px rgba(26, 22, 18, 0.10),
    0 1px 3px rgba(26, 22, 18, 0.06);
}
.arena-option:active:not(:disabled) { transform: translateX(2px) scale(0.985); }
.arena-option:disabled { cursor: default; }

.arena-option--correct {
  background: linear-gradient(135deg, #dcfce7, #bbf7d0) !important;
  color: #14532d !important;
  box-shadow: 0 6px 20px rgba(22, 163, 74, 0.22) !important;
  transform: translateX(4px) !important;
  animation: arena-correct-pop 460ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.arena-option--wrong {
  background: linear-gradient(135deg, #fee2e2, #fecaca) !important;
  color: #7f1d1d !important;
  box-shadow: 0 6px 20px rgba(220, 38, 38, 0.20) !important;
  animation: arena-wrong-shake 320ms ease-in-out;
}
.arena-option--faded { opacity: 0.42; }

@keyframes arena-correct-pop {
  0%   { transform: translateX(4px) scale(0.97); }
  50%  { transform: translateX(4px) scale(1.02); }
  100% { transform: translateX(4px) scale(1); }
}
@keyframes arena-wrong-shake {
  0%, 100% { transform: translateX(0); }
  25%      { transform: translateX(-3px); }
  50%      { transform: translateX(3px); }
  75%      { transform: translateX(-2px); }
}

.arena-letter {
  width: 22px; height: 22px;
  border-radius: 7px;
  background: var(--paper-warm);
  color: var(--ink-muted);
  font-size: 10px; font-weight: 800;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  transition: background 180ms, color 180ms;
}
.arena-option--correct .arena-letter {
  background: rgba(21, 128, 61, 0.15);
  color: #15803d;
}
.arena-option--wrong .arena-letter {
  background: rgba(185, 28, 28, 0.15);
  color: #b91c1c;
}

.arena-option-text {
  font-size: 12.5px;
  font-weight: 500;
  line-height: 1.38;
  flex: 1;
}

/* Explanation panel — no borders, status conveyed by background tint + badge */
.arena-explain {
  margin-top: 12px;
  padding: 12px 14px 13px;
  border-radius: 12px;
  background: var(--paper-warm);
}
.arena-explain--ok  { background: rgba(22, 163, 74, 0.10); }
.arena-explain--err { background: rgba(220, 38, 38, 0.08); }

.arena-explain-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.06em;
  margin-bottom: 8px;
  font-feature-settings: 'tnum';
}
.arena-explain--ok .arena-explain-badge {
  background: rgba(21, 128, 61, 0.14);
  color: #15803d;
}
.arena-explain--err .arena-explain-badge {
  background: rgba(185, 28, 28, 0.12);
  color: #b91c1c;
}

.arena-explain :deep(.qf-shell) {
  padding: 0;
  border: none;
  background: transparent;
  gap: 14px;
}

.arena-explain :deep(.qf-title) {
  font-size: 20px;
}

.arena-explain :deep(.qf-summary),
.arena-explain :deep(.qf-intro),
.arena-explain :deep(.qf-note-line),
.arena-explain :deep(.qf-bullet),
.arena-explain :deep(.qf-option-line),
.arena-explain :deep(.qf-question),
.arena-explain :deep(.qf-answer-text),
.arena-explain :deep(.qf-step-note),
.arena-explain :deep(.qf-step-work),
.arena-explain :deep(.qf-option-choice) {
  font-size: 12px;
}

.arena-explain :deep(.qf-btn--primary) {
  background: var(--accent);
}

.arena-explain-text {
  font-size: 11.5px;
  color: var(--ink-secondary);
  line-height: 1.58;
  margin-bottom: 10px;
}

.arena-explain-actions { display: flex; gap: 7px; }
.arena-btn-ghost {
  padding: 7px 13px; border-radius: 8px;
  background: var(--surface);
  color: var(--ink-secondary);
  font-size: 10.5px; font-weight: 600;
  cursor: pointer;
  border: none;
  box-shadow: 0 1px 2px rgba(26, 22, 18, 0.04), 0 1px 3px rgba(26, 22, 18, 0.04);
  transition: background 130ms, box-shadow 130ms;
}
.arena-btn-ghost:hover {
  background: var(--paper-warm);
  box-shadow: 0 2px 6px rgba(26, 22, 18, 0.08);
}

.arena-btn-solid {
  padding: 6px 14px; border-radius: 8px;
  background: var(--accent);
  color: #fff;
  font-size: 10.5px; font-weight: 700;
  cursor: pointer;
  border: none;
  transition: opacity 130ms, transform 130ms, box-shadow 130ms;
}
.arena-btn-solid:hover {
  opacity: 0.92;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(13, 148, 136, 0.22);
}
.arena-btn-solid:active { transform: translateY(0) scale(0.97); }

/* Reveal transition for the explanation block */
.arena-reveal-enter-active { transition: opacity 260ms ease, transform 320ms cubic-bezier(0.16, 1, 0.3, 1); }
.arena-reveal-leave-active { transition: opacity 140ms ease; }
.arena-reveal-enter-from   { opacity: 0; transform: translateY(10px); }
.arena-reveal-leave-to     { opacity: 0; }

/* Floating "+points" reward — subtle warm drift, not a neon jackpot */
.arena-floatup {
  position: absolute;
  left: 98px;
  top: 46px;
  font-size: 22px;
  font-weight: 500;
  color: var(--warm);
  pointer-events: none;
  z-index: 5;
  letter-spacing: -0.02em;
  animation: arena-floatup 1.2s cubic-bezier(0.16, 1, 0.3, 1);
}
@keyframes arena-floatup {
  0%   { opacity: 0; transform: translateY(18px) scale(0.65); }
  22%  { opacity: 1; transform: translateY(0)    scale(1.10); }
  50%  { opacity: 1; transform: translateY(-8px) scale(1.0); }
  100% { opacity: 0; transform: translateY(-42px) scale(0.9); }
}

/* ═════════════════════════════════════════════════════════════
   RUN TOAST — light glass card matching design system
   ═════════════════════════════════════════════════════════════ */
.lb-toast {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 50;
  width: 214px;
  padding: 11px 13px 12px;
  border-radius: 14px;
  background: var(--surface);
  box-shadow:
    0 16px 40px rgba(26, 22, 18, 0.14),
    0 5px 14px rgba(26, 22, 18, 0.08);
  cursor: pointer;
  color: var(--ink);
}
.lb-toast-head {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--warm);
  margin-bottom: 8px;
}
.lb-toast-title { flex: 1; min-width: 0; }
.lb-toast-close {
  color: var(--ink-muted);
  font-size: 10.5px;
  font-weight: 700;
  padding: 0 3px;
  cursor: pointer;
  transition: color 130ms;
}
.lb-toast-close:hover { color: var(--ink); }

.lb-toast-body { display: flex; flex-direction: column; gap: 2px; }
.lb-toast-copy {
  margin: 0;
  font-size: 11px;
  line-height: 1.55;
  color: var(--ink-secondary);
}
.lb-toast-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  padding: 3px 6px;
  border-radius: 6px;
  transition: background 120ms;
}
.lb-toast-row--you {
  background: var(--accent-glow);
  color: var(--accent);
  font-weight: 700;
}
.lb-toast-rank {
  font-weight: 700;
  font-size: 10px;
  color: var(--ink-muted);
  width: 22px;
  font-feature-settings: 'tnum';
}
.lb-toast-row--you .lb-toast-rank { color: var(--accent); }
.lb-toast-name {
  flex: 1; min-width: 0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.lb-toast-score {
  font-feature-settings: 'tnum';
  font-weight: 700;
  font-size: 10.5px;
  color: var(--ink-secondary);
}
.lb-toast-row--you .lb-toast-score { color: var(--accent); }

.lb-toast-delta {
  margin-top: 8px;
  padding: 4px 8px;
  border-radius: 6px;
  background: rgba(22, 163, 74, 0.12);
  color: #15803d;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-align: center;
}

.lb-toast-enter-active,
.lb-toast-leave-active {
  transition: transform 340ms cubic-bezier(0.16, 1, 0.3, 1), opacity 240ms ease;
}
.lb-toast-enter-from { opacity: 0; transform: translateX(18px) translateY(-4px); }
.lb-toast-leave-to   { opacity: 0; transform: translateX(18px); }

.solidify-pop {
  position: absolute;
  right: 12px;
  bottom: 12px;
  z-index: 55;
  width: min(268px, calc(100% - 24px));
  padding: 12px;
  border-radius: 8px;
  background: var(--surface);
  box-shadow:
    0 18px 42px rgba(26, 22, 18, 0.16),
    0 5px 14px rgba(26, 22, 18, 0.08);
  color: var(--ink);
}
.solidify-pop-kicker {
  margin: 0 0 5px;
  color: var(--warm);
  font-size: 9.5px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
.solidify-pop-title {
  margin: 0;
  color: var(--ink);
  font-size: 14px;
  font-weight: 800;
  line-height: 1.2;
}
.solidify-pop-text {
  margin: 7px 0 11px;
  color: var(--ink-secondary);
  font-size: 11.5px;
  line-height: 1.45;
}
.solidify-pop-actions {
  display: flex;
  gap: 7px;
  justify-content: flex-end;
}
.solidify-pop-primary,
.solidify-pop-secondary {
  border: 0;
  border-radius: 8px;
  cursor: pointer;
  font-size: 10.5px;
  font-weight: 800;
  padding: 7px 10px;
  transition: transform 130ms, opacity 130ms, background 130ms;
}
.solidify-pop-primary {
  background: var(--accent);
  color: #fff;
}
.solidify-pop-secondary {
  background: var(--paper-warm);
  color: var(--ink-secondary);
}
.solidify-pop-primary:hover,
.solidify-pop-secondary:hover { transform: translateY(-1px); }
.solidify-pop-primary:active,
.solidify-pop-secondary:active { transform: translateY(0) scale(0.98); }
.solidify-pop-enter-active,
.solidify-pop-leave-active {
  transition: transform 260ms cubic-bezier(0.16, 1, 0.3, 1), opacity 180ms ease;
}
.solidify-pop-enter-from,
.solidify-pop-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

/* ═════════════════════════════════════════════════════════════
   RESIZABLE DIVIDER
   ═════════════════════════════════════════════════════════════ */
.rp-divider {
  flex-shrink: 0;
  height: 10px;
  background: transparent;
  cursor: row-resize;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: background 140ms;
  touch-action: none;
  user-select: none;
}
.rp-divider::before {
  content: '';
  position: absolute;
  inset: -3px 0;          /* wider hit zone than the visible bar */
}
.rp-divider:hover {
  background: var(--paper-warm);
}
.rp-divider:hover .rp-divider-grip span {
  background: var(--warm);
  transform: scaleY(1.4);
}
.rp-divider-grip {
  display: flex;
  gap: 3px;
  pointer-events: none;
}
.rp-divider-grip span {
  width: 3px;
  height: 3px;
  border-radius: 99px;
  background: var(--border-strong);
  transition: background 140ms, transform 140ms;
}

/* ── Activity ─────────────────────────────── */
.rp-activity {
  flex: 1; min-height: 0;
  display: flex; flex-direction: column;
  padding: 14px 16px 12px;
  background: var(--paper);
  overflow: hidden;
}

/* Section header */
.rp-act-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 13px;
}
.rp-act-header-left { display: flex; align-items: center; gap: 8px; }
.rp-pulse-ring {
  width: 8px; height: 8px; border-radius: 99px;
  background: var(--accent);
  position: relative; flex-shrink: 0;
  animation: pulse-ring 2.4s ease infinite;
}
@keyframes pulse-ring {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--accent) 60%, transparent); }
  50%       { box-shadow: 0 0 0 5px color-mix(in srgb, var(--accent) 0%, transparent); }
}
.rp-act-all {
  display: inline-flex; align-items: center; gap: 3px;
  font-size: 9.5px; font-weight: 700;
  color: var(--accent); cursor: pointer;
  transition: opacity 130ms; opacity: 0.7;
}
.rp-act-all:hover { opacity: 1; }

/* Continue card — light, warm-accent, lives inside the right panel card */
.rp-continue {
  border-radius: 14px;
  padding: 14px;
  margin-bottom: 14px;
  flex-shrink: 0;
  cursor: pointer;
  background: var(--paper-warm);
  box-shadow:
    0 1px 2px rgba(26, 22, 18, 0.04),
    0 4px 12px rgba(26, 22, 18, 0.05);
  transition: transform 220ms ease, box-shadow 220ms ease;
  position: relative;
  overflow: hidden;
}
.rp-continue::before {
  /* Warm accent corner — not an outline, a painted tint */
  content: '';
  position: absolute;
  top: 0; right: 0;
  width: 60%; height: 60%;
  background: radial-gradient(ellipse at top right, rgba(194, 65, 12, 0.10), transparent 60%);
  pointer-events: none;
}
.rp-continue:hover {
  transform: translateY(-2px);
  box-shadow:
    0 8px 22px rgba(26, 22, 18, 0.10),
    0 3px 8px rgba(26, 22, 18, 0.06);
}
.rp-continue:active { transform: translateY(0) scale(0.985); }

.rp-continue-top {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 12px;
  position: relative;
}
.rp-continue-label {
  font-size: 8.5px; font-weight: 800;
  text-transform: uppercase; letter-spacing: 0.20em;
  color: var(--warm);
}
.rp-continue-time {
  font-size: 9px; font-weight: 600;
  color: var(--ink-muted);
}

.rp-continue-body { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 14px; position: relative; }
.rp-continue-icon-wrap {
  width: 32px; height: 32px; border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.rp-continue-text { flex: 1; min-width: 0; }
.rp-continue-title {
  font-family: var(--font-display);
  font-size: 15px; font-weight: 400;
  color: var(--ink); line-height: 1.25; margin-bottom: 3px;
  letter-spacing: -0.005em;
}
.rp-continue-sub {
  font-size: 11px; color: var(--ink-secondary); line-height: 1.45;
}

.rp-continue-cta {
  display: flex; align-items: center; justify-content: center; gap: 5px;
  width: 100%; padding: 8px 0; border-radius: 9px;
  background: var(--surface);
  font-size: 11.5px; font-weight: 700; cursor: pointer;
  border: none;
  box-shadow: 0 1px 3px rgba(26, 22, 18, 0.05);
  transition: background 130ms, box-shadow 130ms;
  position: relative;
}
.rp-continue-cta:hover {
  background: var(--paper);
  box-shadow: 0 2px 6px rgba(26, 22, 18, 0.08);
}

/* Timeline feed */
.rp-feed { flex: 1; overflow-y: auto; scrollbar-width: none; }
.rp-feed::-webkit-scrollbar { display: none; }

.rp-feed-item {
  display: flex; gap: 11px;
  padding: 0 4px 0 2px;
  cursor: pointer;
  border-radius: 10px;
  transition: background 130ms;
}
.rp-feed-item:hover { background: color-mix(in srgb, var(--fc, var(--accent)) 5%, var(--surface)); }
.rp-feed-item:last-child .rp-feed-line { display: none; }

/* Left track: dot + vertical line */
.rp-feed-track {
  display: flex; flex-direction: column; align-items: center;
  padding-top: 13px; flex-shrink: 0;
}
.rp-feed-dot {
  width: 10px; height: 10px; border-radius: 99px;
  flex-shrink: 0;
  box-shadow: 0 0 0 3px var(--paper);
}
.rp-feed-line {
  width: 1px; flex: 1; min-height: 14px;
  background: var(--border-soft);
  margin-top: 5px; margin-bottom: 0;
}

/* Feed content */
.rp-feed-body {
  flex: 1; min-width: 0;
  padding: 10px 0 14px;
}
.rp-feed-title-row {
  display: flex; align-items: center; gap: 5px; margin-bottom: 3px;
}
.rp-feed-title {
  font-size: 12.5px; font-weight: 700; color: var(--ink);
  line-height: 1.3; flex: 1; min-width: 0;
}
.rp-feed-detail {
  font-size: 11px; color: var(--ink-muted);
  line-height: 1.5; margin-bottom: 7px;
}
.rp-feed-footer {
  display: flex; align-items: center; justify-content: space-between;
}
.rp-feed-time {
  font-size: 9.5px; color: var(--ink-muted); font-weight: 500;
}
.rp-feed-cta {
  font-size: 10.5px; font-weight: 700;
  cursor: pointer; transition: opacity 130ms;
  opacity: 0.85;
}
.rp-feed-cta:hover { opacity: 1; }

/* ══════════════════════════════════════════════════════════════
   VITALS STRIP — 4 stat tiles
   ══════════════════════════════════════════════════════════════ */
.vitals-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.vital-tile {
  position: relative;
  padding: 14px 16px 12px;
  border-radius: 16px;
  background: var(--surface);
  overflow: hidden;
  transition: transform 220ms var(--ease-spring),
              box-shadow 220ms var(--ease-smooth);
  opacity: 0;
  transform: translateY(8px);
  animation: vital-in 0.6s var(--ease-out) forwards;
}
.vitals-grid > .vital-tile:nth-child(1) { animation-delay: 60ms; }
.vitals-grid > .vital-tile:nth-child(2) { animation-delay: 130ms; }
.vitals-grid > .vital-tile:nth-child(3) { animation-delay: 200ms; }
.vitals-grid > .vital-tile:nth-child(4) { animation-delay: 270ms; }
@keyframes vital-in { to { opacity: 1; transform: translateY(0); } }

.vital-tile::before {
  content: '';
  position: absolute;
  left: 0; top: 0; bottom: 0;
  width: 3px;
  background: var(--tone, var(--accent));
  opacity: 0.85;
  border-radius: 99px 0 0 99px;
}
.vital-tile:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 24px -10px rgba(26, 22, 18, 0.12);
}

.vital-glyph {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  background: color-mix(in srgb, var(--tone) 12%, transparent);
  color: var(--tone, var(--accent));
}

.vital-body { display: flex; flex-direction: column; gap: 0; }
.vital-num {
  font-size: 28px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.025em;
  color: var(--ink);
  font-feature-settings: 'tnum';
  margin: 0;
}
.vital-unit {
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-muted);
  margin-left: 1px;
}
.vital-label {
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-secondary);
  margin: 6px 0 0;
}
.vital-foot {
  font-size: 9.5px;
  color: var(--ink-muted);
  margin: 8px 0 0;
  letter-spacing: 0.02em;
  line-height: 1.3;
}

/* Narrow viewport: 2 columns */
@media (max-width: 1100px) {
  .vitals-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}

/* ══════════════════════════════════════════════════════════════
   MASTERY PANEL
   ══════════════════════════════════════════════════════════════ */
.mastery-panel {
  padding: 18px 20px 20px;
  opacity: 0;
  transform: translateY(8px);
  animation: vital-in 0.7s 0.35s var(--ease-out) forwards;
}

.mastery-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 14px;
}
.mastery-target {
  font-size: 10.5px;
  color: var(--ink-muted);
  letter-spacing: 0.02em;
}
.mastery-target strong {
  color: var(--ink-secondary);
  font-weight: 700;
}

.mastery-body {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 24px;
  align-items: center;
}

@media (max-width: 880px) {
  .mastery-body { grid-template-columns: 1fr; gap: 16px; justify-items: center; }
  .mastery-subjects { width: 100%; }
}

/* HERO ring */
.mastery-hero {
  position: relative;
  width: 188px;
  min-height: 260px;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.mastery-hero-svg {
  display: block;
  overflow: visible;
}
.mastery-hero-center {
  position: absolute;
  top: 0;
  left: 0;
  width: 188px;
  height: 188px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}
.mastery-hero-num {
  font-size: 44px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.04em;
  color: var(--ink);
  margin: 0;
  font-feature-settings: 'tnum';
}
.mastery-hero-pct {
  font-size: 22px;
  font-weight: 600;
  color: var(--ink-muted);
  margin-left: 2px;
}
.mastery-hero-eyebrow {
  font-size: 9.5px;
  font-weight: 800;
  letter-spacing: 0.20em;
  text-transform: uppercase;
  color: var(--ink-muted);
  margin: 4px 0 0;
}
.mastery-hero-band {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--accent);
  margin: 14px 0 0;
}

/* Ring arc — shared, with drop-shadow glow */
.ring-arc {
  transition: stroke-dashoffset 1.6s cubic-bezier(0.16, 1, 0.3, 1);
}
.ring-arc--hero {
  filter: drop-shadow(0 0 6px rgba(13, 148, 136, 0.45));
  animation: arc-pulse-hero 3.6s ease-in-out infinite;
}
@keyframes arc-pulse-hero {
  0%, 100% { filter: drop-shadow(0 0 6px rgba(13, 148, 136, 0.40)); }
  50%      { filter: drop-shadow(0 0 14px rgba(13, 148, 136, 0.70)); }
}
.ring-arc--sub {
  filter: drop-shadow(0 0 3px var(--glow, rgba(13, 148, 136, 0.40)));
}
.ring-arc--nest {
  filter: drop-shadow(0 0 6px var(--glow, rgba(13, 148, 136, 0.34)));
}
.mastery-nest-track {
  opacity: 0.62;
}

.mastery-hero-legend {
  margin-top: 12px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mastery-legend-chip {
  display: grid;
  grid-template-columns: 14px 1fr auto;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 86%, var(--paper));
}
.mastery-legend-swatch {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  background: linear-gradient(135deg, var(--sw-from), var(--sw-to));
}
.mastery-legend-name {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-secondary);
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mastery-legend-pct {
  font-size: 10.5px;
  font-weight: 800;
  color: var(--ink);
  font-feature-settings: 'tnum';
}

/* SUBJECTS list */
.mastery-subjects {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.subject-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 8px 6px;
  border-radius: 10px;
  transition: background 140ms ease;
}
.subject-row:hover { background: var(--paper); }

.subject-ring-wrap {
  position: relative;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
}
.subject-ring-svg {
  display: block;
  overflow: visible;
}
.subject-ring-pct {
  position: absolute;
  inset: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: var(--ink);
  font-feature-settings: 'tnum';
  letter-spacing: -0.02em;
}

.subject-meta { flex: 1; min-width: 0; }
.subject-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--ink);
  margin: 0;
  line-height: 1.2;
  letter-spacing: -0.01em;
}
.subject-counts {
  font-size: 11px;
  color: var(--ink-muted);
  margin: 3px 0 0;
  font-feature-settings: 'tnum';
}
.subject-counts strong { color: var(--ink-secondary); font-weight: 700; }
.subject-weak { color: var(--warm); font-weight: 600; }

/* Empty state for subjects */
.subjects-empty {
  padding: 14px 4px;
}
.subjects-empty-title {
  font-family: var(--font-display);
  font-size: 14px;
  font-weight: 700;
  color: var(--ink-secondary);
  margin: 0 0 4px;
}
.subjects-empty-sub {
  font-size: 11.5px;
  color: var(--ink-muted);
  margin: 0;
  line-height: 1.5;
}

/* ══════════════════════════════════════════════════════════════
   JUMP IN PANEL
   ══════════════════════════════════════════════════════════════ */
.launch-panel {
  padding-top: 2px;
}
.launch-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}
.launch-inline-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--ink-secondary);
  font-size: 11px;
  font-weight: 700;
  transition: color 140ms ease, transform 140ms ease;
}
.launch-inline-link:hover {
  color: var(--accent);
  transform: translateX(1px);
}
.launch-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}
.launch-tile {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 118px;
  padding: 16px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--launch-tone) 16%, transparent);
  background: linear-gradient(180deg, color-mix(in srgb, var(--launch-tone) 9%, var(--surface)) 0%, var(--surface) 100%);
  box-shadow:
    0 1px 2px rgba(26, 22, 18, 0.04),
    0 6px 18px rgba(26, 22, 18, 0.06);
  text-align: left;
  overflow: hidden;
  cursor: pointer;
  transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1), box-shadow 180ms ease, border-color 180ms ease;
}
.launch-tile::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, color-mix(in srgb, var(--launch-tone) 14%, transparent) 0%, transparent 62%);
  pointer-events: none;
}
.launch-tile:hover {
  transform: translateY(-2px);
  border-color: color-mix(in srgb, var(--launch-tone) 26%, transparent);
  box-shadow:
    0 12px 24px rgba(26, 22, 18, 0.10),
    0 3px 8px rgba(26, 22, 18, 0.06);
}
.launch-tile:active {
  transform: translateY(0) scale(0.985);
}
.launch-tile--wide {
  grid-column: 1 / -1;
  min-height: 130px;
}
.launch-tile-head,
.launch-tile-body,
.launch-tile-cta {
  position: relative;
  z-index: 1;
}
.launch-tile-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.launch-tile-kicker {
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: color-mix(in srgb, var(--launch-tone) 86%, var(--ink-secondary));
}
.launch-tile-icon {
  width: 30px;
  height: 30px;
  border-radius: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--launch-tone) 12%, transparent);
  color: var(--launch-tone);
  flex-shrink: 0;
}
.launch-tile-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 20px;
}
.launch-tile-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  line-height: 1.04;
  letter-spacing: -0.03em;
  color: var(--ink);
}
.launch-tile-detail {
  margin: 0;
  max-width: 30ch;
  font-size: 12px;
  line-height: 1.45;
  color: var(--ink-secondary);
}
.launch-tile-cta {
  margin-top: 18px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 800;
  color: color-mix(in srgb, var(--launch-tone) 88%, var(--ink-secondary));
}

@media (max-width: 880px) {
  .launch-grid {
    grid-template-columns: 1fr;
  }
  .launch-tile--wide {
    grid-column: auto;
  }
}
</style>
