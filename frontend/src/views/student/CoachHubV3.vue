<script setup lang="ts">
/**
 * CoachHub — v3 (Nothing Design · v1 outlay)
 *
 * Preserves v1's two-column layout:
 *   LEFT  column (flex): greeting → directive → vitals → mastery → priority
 *   RIGHT rail  (22rem): arena (live quick test) → drag divider → live feed
 *
 * Every surface restyled in Nothing aesthetic:
 *   · monochrome warm off-white / OLED dark
 *   · Space Grotesk (body) · Space Mono (labels/data) · Doto (one hero moment)
 *   · typography over color, dot-matrix + segmented forms, no shadows
 */

import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  watchEffect,
} from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import {
  getCoachNextAction,
  getHomeLearningStats,
  getStudentDashboard,
  getPriorityTopics,
  listActiveMisconceptions,
  listStudentActivityHistory,
  listSubjects,
  type CoachNextActionDto,
  type HomeLearningStatsDto,
  type LearnerMisconceptionSnapshotDto,
  type StudentActivityHistoryItemDto,
  type StudentDashboardDto,
  type SubjectDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import { useHomepageArena } from '@/composables/useHomepageArena'
import MathText from '@/components/question/MathText.vue'
import QuestionFeedback from '@/components/question/QuestionFeedback.vue'

// ─── Stores & routing ───────────────────────────────────────────────
const auth = useAuthStore()
const ui = useUiStore()
const router = useRouter()
const route = useRoute()

// ─── Remote state ───────────────────────────────────────────────────
const loading = ref(true)
const error = ref('')
const nextAction = ref<CoachNextActionDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topicCases = ref<TopicCaseDto[]>([])
const homeStats = ref<HomeLearningStatsDto | null>(null)
const subjectDirectory = ref<SubjectDto[]>([])
const activityHistory = ref<StudentActivityHistoryItemDto[]>([])
const activeMisconceptions = ref<LearnerMisconceptionSnapshotDto[]>([])

async function refreshHomeData() {
  if (!auth.currentAccount) return
  const sid = auth.currentAccount.id
  const [a, d, t, stats, subjects, history] = await Promise.all([
    getCoachNextAction(sid),
    getStudentDashboard(sid),
    getPriorityTopics(sid, 6),
    getHomeLearningStats(sid),
    listSubjects(1),
    listStudentActivityHistory(sid, 8),
  ])
  nextAction.value = a
  dashboard.value = d
  topicCases.value = t
  homeStats.value = stats
  subjectDirectory.value = subjects
  activityHistory.value = history

  const focusedSubjectId = t[0]?.subject_code
    ? (subjects.find(subject => subject.code === t[0].subject_code)?.id ?? null)
    : (d.subjects[0]?.subject_id ?? null)

  activeMisconceptions.value = focusedSubjectId
    ? await listActiveMisconceptions(sid, focusedSubjectId).catch(() => [])
    : []
}

const {
  questionPips: qtPips,
  questionIndex: qtIndex,
  currentQuestion: qtQuestion,
  selectedOptionId: qtChoice,
  loading: arenaLoading,
  submitting: arenaSubmitting,
  error: arenaError,
  locked: qtLocked,
  isCorrect: qtIsCorrect,
  feedbackPayload: qtFeedback,
  studyRoute: qtStudyRoute,
  weaknessTitle: arenaWeaknessTitle,
  weaknessDetail: arenaWeaknessDetail,
  loadArena,
  submitOption: submitQtOption,
  markTimedOut: markQtTimedOut,
  nextQuestion: advanceQtQuestion,
  isCorrectOption: isQtOptionCorrect,
  isWrongOption: isQtOptionWrong,
} = useHomepageArena({
  getStudentId: () => auth.currentAccount?.id,
  getTopicCases: () => topicCases.value,
  onRecorded: refreshHomeData,
  questionCount: 4,
  isTimed: true,
})

// ─── Live clock ─────────────────────────────────────────────────────
const now = ref(new Date())
let clockId: number | null = null
onMounted(() => {
  clockId = window.setInterval(() => { now.value = new Date() }, 15_000)
})
onBeforeUnmount(() => { if (clockId !== null) clearInterval(clockId) })

// ─── Font injection (Doto, Space Grotesk, Space Mono) ───────────────
onMounted(() => {
  const id = 'nd-v3-fonts'
  if (document.getElementById(id)) return
  const pre1 = document.createElement('link')
  pre1.rel = 'preconnect'; pre1.href = 'https://fonts.googleapis.com'
  const pre2 = document.createElement('link')
  pre2.rel = 'preconnect'; pre2.href = 'https://fonts.gstatic.com'; pre2.crossOrigin = ''
  const css = document.createElement('link')
  css.id = id
  css.rel = 'stylesheet'
  css.href = 'https://fonts.googleapis.com/css2?' +
    'family=Doto:wght@400;700;900&' +
    'family=Space+Grotesk:wght@300;400;500;600;700&' +
    'family=Space+Mono:wght@400;700&display=swap'
  document.head.appendChild(pre1)
  document.head.appendChild(pre2)
  document.head.appendChild(css)
})

// ─── Greeting + dates ───────────────────────────────────────────────
const greeting = computed(() => {
  const h = now.value.getHours()
  if (h < 5)  return 'Late night'
  if (h < 12) return 'Good morning'
  if (h < 17) return 'Good afternoon'
  return 'Good evening'
})
const dateLabel = computed(() =>
  now.value.toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' }),
)
const dateStamp = computed(() => {
  const d = now.value
  return `${d.getFullYear()}.${String(d.getMonth() + 1).padStart(2, '0')}.${String(d.getDate()).padStart(2, '0')}`
})
const timeStamp = computed(() => {
  const d = now.value
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
})
const weekdayStamp = computed(() =>
  now.value.toLocaleDateString('en-GB', { weekday: 'long' }).toUpperCase(),
)
const studentName = computed(() =>
  dashboard.value?.student_name || auth.currentAccount?.display_name || 'Student',
)
const firstName = computed(() => studentName.value.split(' ')[0])
const topMisconception = computed(() =>
  [...activeMisconceptions.value]
    .sort((left, right) => {
      if (right.risk_score !== left.risk_score) return right.risk_score - left.risk_score
      return right.times_detected - left.times_detected
    })[0] ?? null,
)

function parseOccurredAt(value: string): number | null {
  if (!value) return null
  const normalized = value.includes('T') ? value : `${value.replace(' ', 'T')}Z`
  const parsed = Date.parse(normalized)
  return Number.isNaN(parsed) ? null : parsed
}

function timeAgoShort(value: string): string {
  const timestamp = parseOccurredAt(value)
  if (timestamp == null) return 'NOW'

  const diffMinutes = Math.max(0, Math.round((Date.now() - timestamp) / 60_000))
  if (diffMinutes < 1) return 'NOW'
  if (diffMinutes < 60) return `${diffMinutes}M`

  const diffHours = Math.floor(diffMinutes / 60)
  if (diffHours < 24) return `${diffHours}H`

  return `${Math.floor(diffHours / 24)}D`
}

function routeForHistoryItem(item: StudentActivityHistoryItemDto): string {
  if (item.type_key === 'mock') return '/student/mock'
  if (item.type_key === 'diagnostic') return '/student/diagnostic'
  return '/student/practice'
}

function feedTagForHistoryItem(item: StudentActivityHistoryItemDto): FeedTag {
  if (item.type_key === 'mock') return 'MOCK'
  if (item.type_key === 'diagnostic') return 'COACH'
  return 'REVIEW'
}

function historySummary(item: StudentActivityHistoryItemDto): string {
  const attemptSummary = item.total_questions > 0
    ? `${item.answered_questions}/${item.total_questions} answered`
    : `${item.answered_questions} answered`

  return `${item.subject} · ${attemptSummary} · ${item.score}%`
}

// ─── Fetch ──────────────────────────────────────────────────────────
onMounted(async () => {
  if (!auth.currentAccount) { loading.value = false; return }
  try {
    await refreshHomeData()
    await loadArena()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message ?? 'Failed to load')
  } finally {
    loading.value = false
  }
})

// ─── Deterministic mocks (same seed contract as v1/v2) ──────────────
const vitalStats = computed(() => {
  const stats = homeStats.value
  return {
    streak: stats?.streak_days ?? 0,
    accuracy: stats?.accuracy_percent ?? 0,
    todayMin: stats?.today_minutes ?? 0,
    weekQuestions: stats?.week_questions ?? 0,
  }
})

// ─── Count-up helper ────────────────────────────────────────────────
function useCountUp(source: () => number, duration = 1100, delay = 0) {
  const out = ref(0)
  watchEffect((onCleanup) => {
    const dest = source()
    const start = out.value
    const t0 = performance.now() + delay
    let raf = 0
    const tick = (t: number) => {
      const k = Math.max(0, Math.min(1, (t - t0) / duration))
      const eased = 1 - Math.pow(1 - k, 3)
      out.value = Math.round(start + (dest - start) * eased)
      if (k < 1) raf = requestAnimationFrame(tick)
    }
    raf = requestAnimationFrame(tick)
    onCleanup(() => cancelAnimationFrame(raf))
  })
  return out
}
const heroGate = ref(0)
watch(loading, (v) => { if (!v) heroGate.value = 1 })

// ─── Subjects + mastery ─────────────────────────────────────────────
interface SubjectRing {
  id: number
  name: string
  pct: number
  mastered: number
  total: number
  weak: number
}
const subjectRings = computed<SubjectRing[]>(() => {
  const subs = dashboard.value?.subjects ?? []
  return subs.map(s => ({
    id: s.subject_id,
    name: s.subject_name,
    pct: s.total_topic_count > 0
      ? Math.round((s.mastered_topic_count / s.total_topic_count) * 100)
      : 0,
    mastered: s.mastered_topic_count,
    total: s.total_topic_count,
    weak: s.weak_topic_count,
  }))
})
const overallPct = computed(() => {
  const r = subjectRings.value
  if (!r.length) return 0
  return Math.round(r.reduce((s, x) => s + x.pct, 0) / r.length)
})
const readinessBand = computed(() => {
  const p = overallPct.value
  if (p < 25) return 'FOUNDATION'
  if (p < 50) return 'BUILDING'
  if (p < 70) return 'ON TRACK'
  if (p < 85) return 'STRONG'
  return 'EXAM READY'
})
const animOverall  = useCountUp(() => heroGate.value ? overallPct.value            : 0, 1400, 150)
const animStreak   = useCountUp(() => heroGate.value ? vitalStats.value.streak      : 0,  900, 100)
const animAccuracy = useCountUp(() => heroGate.value ? vitalStats.value.accuracy    : 0, 1100, 180)
const animToday    = useCountUp(() => heroGate.value ? vitalStats.value.todayMin    : 0, 1000, 260)
const animWeekQs   = useCountUp(() => heroGate.value ? vitalStats.value.weekQuestions : 0, 1300, 340)

// Ring geometry
const RING_HERO_SIZE = 176
const RING_HERO_R    = 76
const RING_HERO_W    = 6
const RING_HERO_CIRC = 2 * Math.PI * RING_HERO_R

// Per-subject dot-row cap
const SUB_DOT_CAP = 16
interface DotState { kind: 'filled' | 'ring' | 'faint' }
function subjectDots(s: SubjectRing): DotState[] {
  const cap = Math.max(1, Math.min(SUB_DOT_CAP, s.total))
  const mastered = Math.round((s.mastered / Math.max(1, s.total)) * cap)
  const weak     = Math.min(cap - mastered, Math.round((s.weak / Math.max(1, s.total)) * cap))
  const faint    = cap - mastered - weak
  const dots: DotState[] = []
  for (let i = 0; i < mastered; i++) dots.push({ kind: 'filled' })
  for (let i = 0; i < weak;     i++) dots.push({ kind: 'ring' })
  for (let i = 0; i < faint;    i++) dots.push({ kind: 'faint' })
  return dots
}

function bp(bpVal: number): string { return (bpVal / 100).toFixed(0) + '%' }
function topicTagLabel(t: TopicCaseDto): string {
  if (t.intervention_urgency === 'high')   return 'CRITICAL'
  if (t.intervention_urgency === 'medium') return 'WATCH'
  return 'REVIEW'
}
function topicDotKind(state: string): 'filled' | 'ring' | 'faint' {
  if (state === 'mastered' || state === 'robust') return 'filled'
  if (state === 'weak' || state === 'fragile' || state === 'decaying') return 'ring'
  return 'faint'
}

// ═══════════════════════════════════════════════════════════════════
// ARENA — live quick-test (preserves v1's scoring contract)
// ═══════════════════════════════════════════════════════════════════

/*
const obsoleteArenaPlaceholder = [
  {
    id: 'q-gap',
    topic: 'GAP SCAN',
    prompt: '',
    options: [
      { id: 'A', label: '' },
      { id: 'B', label: '' },
      { id: 'C', label: 'Skip the topic until the next exam' },
    ],
    explain: 'Pattern review with focused drilling fixes root causes faster than random practice.',
    route: '/student/knowledge-gap',
  },
  {
    id: 'q-diag',
    topic: 'DIAGNOSTIC',
    prompt: 'A good diagnostic test mostly measures…',
    options: [
      { id: 'A', label: '' },
      { id: 'B', label: 'Your strongest topics only' },
      { id: 'C', label: '' },
    ],
    explain: 'Diagnostics earn their cost only when they surface true gaps.',
    route: '/student/diagnostic',
  },
  {
    id: 'q-mock',
    topic: 'PREPARE',
    prompt: '',
    options: [
      { id: 'A', label: '' },
      { id: 'B', label: '' },
      { id: 'C', label: 'Avoid all practice' },
    ],
    explain: '',
    route: '/student/mock',
  },
  {
    id: 'q-mem',
    topic: 'REVIEW',
    prompt: '',
    options: [
      { id: 'A', label: '' },
      { id: 'B', label: '' },
      { id: 'C', label: 'Never — once is enough' },
    ],
    explain: 'Spaced review protects memory strength before decay becomes expensive.',
    route: '/student/memory#reviews',
  },
]
*/

const TIMER_MAX = 20
const qtTimer   = ref(TIMER_MAX)
const qtScore   = ref(0)
const qtStreak  = ref(0)
const qtBest    = ref(0)
const qtAnswered = ref(0)
const qtGain    = ref(0)
const qtKey     = ref(0)
const qtFloatId = ref<number | null>(null)

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
    nextQuestion()
  }, delay)
}

function buildSolidifyPrompt(): SolidifyPrompt {
  if (topMisconception.value) {
    const misconception = topMisconception.value
    return {
      title: misconception.topic_name || misconception.title,
      detail: arenaWeaknessDetail.value
        || `${misconception.title} is still active in your evidence trail.`,
      route: qtStudyRoute.value || '/student/knowledge-gap',
    }
  }

  if (arenaWeaknessTitle.value || arenaWeaknessDetail.value) {
    return {
      title: arenaWeaknessTitle.value,
      detail: arenaWeaknessDetail.value || 'Focused practice is ready.',
      route: qtStudyRoute.value || '/student/knowledge-gap',
    }
  }

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
    route: qtStudyRoute.value,
  }
}

function clearSolidifyPromptTimeout() {
  if (solidifyPromptTimeout !== null) {
    window.clearTimeout(solidifyPromptTimeout)
    solidifyPromptTimeout = null
  }
}

function dismissSolidifyPrompt() {
  // Silent dismissal — used on unmount and manual advance. Does NOT trigger next-question.
  clearSolidifyPromptTimeout()
  solidifyPrompt.value = null
}

function userDismissSolidifyPrompt() {
  // User clicked NOT NOW. Hide the popup and keep the current review on screen.
  clearSolidifyPromptTimeout()
  solidifyPrompt.value = null
}

function showSolidifyPrompt() {
  solidifyPrompt.value = buildSolidifyPrompt()
  clearSolidifyPromptTimeout()
  solidifyPromptTimeout = window.setTimeout(() => {
    // Popup self-dismissed. Keep the current question review visible until the learner advances.
    solidifyPrompt.value = null
    solidifyPromptTimeout = null
  }, SOLIDIFY_PROMPT_MS)
}

function solidifyKnownWeakness() {
  // User chose to drill the weakness — leave the page, no auto-advance.
  const route = solidifyPrompt.value?.route
  clearArenaAutoAdvance()
  dismissSolidifyPrompt()
  if (route) router.push(route)
}

let qtInterval: number | null = null
function startTimer() {
  if (!qtQuestion.value || qtLocked.value || arenaLoading.value) return
  stopTimer()
  qtTimer.value = TIMER_MAX
  qtInterval = window.setInterval(() => {
    qtTimer.value = Math.max(0, +(qtTimer.value - 0.1).toFixed(1))
    if (qtTimer.value <= 0) {
      stopTimer()
      if (!qtLocked.value) commitQtTimeout()
    }
  }, 100)
}
function stopTimer() { if (qtInterval !== null) { clearInterval(qtInterval); qtInterval = null } }

const animScore  = useCountUp(() => qtScore.value, 620, 0)
const scorePulse = ref(false)

async function pickOption(id: string) {
  if (qtLocked.value || arenaSubmitting.value) return
  stopTimer()
  const result = await submitQtOption(id)
  if (!result) {
    startTimer()
    return
  }

  qtAnswered.value++
  if (result.is_correct) {
    const timeBonus = Math.floor(Math.max(0, qtTimer.value) * 2)
    const streakBonus = qtStreak.value * 5
    const pts = 50 + timeBonus + streakBonus
    qtGain.value = pts
    qtScore.value += pts
    qtStreak.value += 1
    if (qtStreak.value > qtBest.value) qtBest.value = qtStreak.value
    scorePulse.value = true
    window.setTimeout(() => { scorePulse.value = false }, 620)
    qtFloatId.value = Date.now()
    window.setTimeout(() => { qtFloatId.value = null }, 1400)
    // Correct answer — fast auto-advance, no popup to wait for.
    scheduleArenaAutoAdvance()
  } else {
    qtStreak.value = 0
    qtGain.value = 0
    // Wrong answer — popup shows; advance is scheduled only when popup dismisses.
    showSolidifyPrompt()
  }
}
async function nextQuestion() {
  clearArenaAutoAdvance()
  // If user manually advances while popup is still showing, clear it.
  dismissSolidifyPrompt()
  await advanceQtQuestion()
  qtKey.value++
  nextTick(() => startTimer())
}
function openTopic() {
  clearArenaAutoAdvance()
  router.push(qtStudyRoute.value)
}

function commitQtTimeout() {
  if (qtLocked.value) return
  stopTimer()
  markQtTimedOut()
  qtAnswered.value++
  qtStreak.value = 0
  qtGain.value = 0
  // Timeout shows the popup — advance will run once the popup dismisses.
  showSolidifyPrompt()
}
watch(qtIndex, () => startTimer())
watch(loading, (v) => { if (!v) nextTick(() => startTimer()) })
onBeforeUnmount(() => {
  stopTimer()
  clearArenaAutoAdvance()
  dismissSolidifyPrompt()
})

const timerCells = computed(() => {
  const lit = Math.ceil(qtTimer.value)
  return Array.from({ length: TIMER_MAX }, (_, i) => i < lit)
})
const STREAK_CAPS = 6
const streakDots = computed(() =>
  Array.from({ length: STREAK_CAPS }, (_, i) => i < Math.min(qtStreak.value, STREAK_CAPS)),
)

// ═══════════════════════════════════════════════════════════════════
// LIVE FEED — same composition as v1 (directive + topic + subject + 3 system events)
// ═══════════════════════════════════════════════════════════════════

type FeedTag = 'COACH' | 'GAP' | 'REVIEW' | 'MOCK' | 'GAMES' | 'MEM'
interface FeedItem {
  id: string
  tag: FeedTag
  title: string
  sub: string
  time: string
  route: string
  critical?: boolean
}
const liveFeed = computed<FeedItem[]>(() => {
  const items: FeedItem[] = []

  if (nextAction.value) {
    items.push({
      id: 'f-coach', tag: 'COACH',
      title: nextAction.value.title, sub: nextAction.value.subtitle,
      time: 'NOW', route: nextAction.value.route, critical: true,
    })
  }

  if (topMisconception.value) {
    const misconception = topMisconception.value
    items.push({
      id: `f-mis-${misconception.misconception_id}`, tag: 'GAP',
      title: misconception.topic_name || misconception.title,
      sub: `${misconception.title} · risk ${Math.round(misconception.risk_score / 100)}%`,
      time: 'NOW', route: '/student/knowledge-gap',
      critical: misconception.risk_score >= 7000,
    })
  } else if (topicCases.value.length) {
    const t = topicCases.value[0]
    items.push({
      id: `f-gap-${t.topic_id}`, tag: 'GAP',
      title: `${t.topic_name} flagged`,
      sub: t.intervention_mode.replace(/_/g, ' '),
      time: 'NOW', route: '/student/knowledge-gap',
      critical: t.intervention_urgency === 'high',
    })
  }

  if (dashboard.value?.subjects?.length) {
    const weakest = [...dashboard.value.subjects].sort((a, b) => {
      const ap = a.total_topic_count ? a.mastered_topic_count / a.total_topic_count : 0
      const bp = b.total_topic_count ? b.mastered_topic_count / b.total_topic_count : 0
      return ap - bp
    })[0]
    const weakestPct = Math.round(weakest.mastered_topic_count / Math.max(weakest.total_topic_count, 1) * 100)
    items.push({
      id: `f-sub-${weakest.subject_id}`, tag: 'REVIEW',
      title: `${weakest.subject_name} needs attention`,
      sub: `${weakestPct}% mastered · ${weakest.weak_topic_count} weak`,
      time: 'NOW', route: '/student/progress',
    })
  }

  for (const item of activityHistory.value) {
    items.push({
      id: `f-history-${item.type_key}-${item.id}`,
      tag: feedTagForHistoryItem(item),
      title: item.label,
      sub: historySummary(item),
      time: timeAgoShort(item.occurred_at),
      route: routeForHistoryItem(item),
      critical: item.score < 50,
    })
  }

  return items.slice(0, 8)
})
const continueItem = computed(() => liveFeed.value[0])
const feedRest     = computed(() => liveFeed.value.slice(1, 7))

// ─── Routes ─────────────────────────────────────────────────────────
function startAction() { if (nextAction.value?.route) router.push(nextAction.value.route) }

// ═══════════════════════════════════════════════════════════════════
// DRAG-TO-RESIZE DIVIDER (arena vs. feed)
// ═══════════════════════════════════════════════════════════════════
const splitRatio = ref(0.55)
try {
  const saved = parseFloat(localStorage.getItem('ecoach.rp.split') ?? '0.55')
  if (!Number.isNaN(saved) && saved >= 0.22 && saved <= 0.78) splitRatio.value = saved
} catch { /* noop */ }

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
  try { localStorage.setItem('ecoach.rp.split', String(splitRatio.value)) } catch { /* noop */ }
}

// ─── Version switcher ───────────────────────────────────────────────
interface VersionChip { key: 'v1' | 'v2' | 'v3'; label: string; to: string }
const versions: VersionChip[] = [
  { key: 'v1', label: 'V1', to: '/student' },
  { key: 'v2', label: 'V2', to: '/student/v2' },
  { key: 'v3', label: 'V3', to: '/student/v3' },
]
const currentVersion = computed<VersionChip['key']>(() => {
  if (route.path.endsWith('/v3')) return 'v3'
  if (route.path.endsWith('/v2')) return 'v2'
  return 'v1'
})
function goVersion(v: VersionChip) {
  if (v.key !== currentVersion.value) router.push(v.to)
}
</script>

<template>
  <div class="nd" :data-nd-dark="ui.isDark ? '1' : '0'">

    <!-- ════════ TOP STATUS BAR ════════ -->
    <header class="nd-statusbar">
      <div class="nd-statusbar-left">
        <span class="nd-stat nd-stat--accent">● COACHHUB/03</span>
        <span class="nd-stat">{{ dateStamp }}</span>
        <span class="nd-stat nd-stat--dim">{{ weekdayStamp }}</span>
        <span class="nd-stat">{{ timeStamp }}</span>
      </div>
      <div class="nd-statusbar-right">
        <!-- v1/v2 temporarily disabled — keep switcher hidden until they are re-enabled.
        <div class="nd-version-switch">
          <button
            v-for="v in versions" :key="v.key"
            class="nd-version-btn"
            :class="{ 'nd-version-btn--on': currentVersion === v.key }"
            @click="goVersion(v)"
          >{{ v.label }}</button>
        </div>
        -->
        <span class="nd-stat nd-stat--dim">{{ ui.isDark ? 'SHELL/DARK' : 'SHELL/LIGHT' }}</span>
        <span class="nd-stat">GUIDED MODE</span>
      </div>
    </header>

    <!-- ════════ SHELL (v1 layout: left column + right rail) ════════ -->
    <div class="nd-shell">

      <!-- ─── LEFT COLUMN ─────────────────────────────────────────── -->
      <div class="nd-left">

        <!-- Greeting card (compact, top of column) -->
        <div class="nd-greet">
          <div class="nd-greet-l">
            <p class="nd-mono nd-mono--dim nd-greet-date">{{ dateLabel.toUpperCase() }}</p>
            <h1 class="nd-greet-title">
              {{ greeting }},
              <span class="nd-greet-name">{{ firstName }}</span>
            </h1>
          </div>
          <div v-if="subjectRings.length" class="nd-greet-subs">
            <div
              v-for="s in subjectRings.slice(0, 3)" :key="s.id"
              class="nd-greet-sub"
            >
              <p class="nd-greet-sub-pct">{{ s.pct }}<span class="nd-greet-sub-u">%</span></p>
              <p class="nd-mono nd-mono--dim nd-greet-sub-name">{{ s.name }}</p>
            </div>
          </div>
        </div>

        <!-- Loading / error -->
        <div v-if="loading" class="nd-left-loading">
          <span class="nd-spinner"><span /><span /><span /><span /><span /><span /></span>
          <p class="nd-mono">[ SYNCING COACH STATE… ]</p>
        </div>
        <div v-else-if="error" class="nd-left-error">
          <p class="nd-mono nd-mono--red">[ ERROR ]</p>
          <p class="nd-error-msg">{{ error }}</p>
        </div>

        <!-- Scrollable main content -->
        <div v-else class="nd-left-main">

          <!-- ─── 1. COACH DIRECTIVE ─── -->
          <section v-if="nextAction" class="nd-directive nd-reveal" @click="startAction">
            <div class="nd-dir-head">
              <span class="nd-tag nd-tag--pulse">DIRECTIVE</span>
              <span v-if="nextAction.estimated_minutes" class="nd-mono nd-mono--dim">
                ~{{ nextAction.estimated_minutes }} MIN
              </span>
            </div>
            <h2 class="nd-dir-title">{{ nextAction.title }}</h2>
            <p class="nd-dir-sub">{{ nextAction.subtitle }}</p>
            <div class="nd-dir-actions">
              <button class="nd-cta" @click.stop="startAction">
                <span>START NOW</span><span class="nd-cta-arrow">→</span>
              </button>
              <span class="nd-dir-type">
                {{ nextAction.action_type?.replace(/_/g, ' ').toUpperCase() }}
              </span>
            </div>
          </section>

          <section v-else class="nd-directive nd-directive--idle nd-reveal">
            <span class="nd-tag nd-tag--dim">ALL CAUGHT UP</span>
            <p class="nd-dir-title">Nothing pending.</p>
            <p class="nd-dir-sub">Open the journey to plan your next move.</p>
          </section>

          <!-- ─── 2. VITALS STRIP ─── -->
          <section class="nd-vitals nd-reveal">
            <div class="nd-vital">
              <p class="nd-mono nd-mono--dim">STREAK</p>
              <p class="nd-vital-num">{{ animStreak }}<span class="nd-vital-u">D</span></p>
              <p class="nd-mono nd-mono--dim nd-vital-cap">consecutive days</p>
            </div>
            <div class="nd-vital">
              <p class="nd-mono nd-mono--dim">ACCURACY</p>
              <p class="nd-vital-num">{{ animAccuracy }}<span class="nd-vital-u">%</span></p>
              <p class="nd-mono nd-mono--dim nd-vital-cap">last 7 days</p>
            </div>
            <div class="nd-vital">
              <p class="nd-mono nd-mono--dim">TODAY</p>
              <p class="nd-vital-num">{{ animToday }}<span class="nd-vital-u">M</span></p>
              <p class="nd-mono nd-mono--dim nd-vital-cap">minutes studied</p>
            </div>
            <div class="nd-vital">
              <p class="nd-mono nd-mono--dim">WEEK</p>
              <p class="nd-vital-num">{{ animWeekQs }}</p>
              <p class="nd-mono nd-mono--dim nd-vital-cap">questions answered</p>
            </div>
          </section>

          <!-- ─── 3. MASTERY PANEL ─── -->
          <section class="nd-mastery nd-reveal">
            <div class="nd-sec-head">
              <p class="nd-eyebrow nd-eyebrow--plain">MASTERY</p>
              <span v-if="dashboard?.exam_target" class="nd-mono nd-mono--dim">
                TARGET · {{ dashboard.exam_target.toUpperCase() }}
              </span>
            </div>

            <div class="nd-mastery-body">
              <!-- LEFT: big monochrome ring with Doto % in the center -->
              <div class="nd-ring-hero">
                <svg
                  class="nd-ring-svg"
                  :width="RING_HERO_SIZE" :height="RING_HERO_SIZE"
                  :viewBox="`0 0 ${RING_HERO_SIZE} ${RING_HERO_SIZE}`"
                  aria-hidden="true"
                >
                  <circle
                    :cx="RING_HERO_SIZE / 2" :cy="RING_HERO_SIZE / 2" :r="RING_HERO_R"
                    fill="none"
                    stroke="var(--nd-ink-4)"
                    :stroke-width="RING_HERO_W"
                  />
                  <circle
                    class="nd-ring-arc"
                    :cx="RING_HERO_SIZE / 2" :cy="RING_HERO_SIZE / 2" :r="RING_HERO_R"
                    fill="none"
                    stroke="var(--nd-ink)"
                    :stroke-width="RING_HERO_W"
                    stroke-linecap="round"
                    :stroke-dasharray="RING_HERO_CIRC"
                    :stroke-dashoffset="heroGate
                      ? RING_HERO_CIRC * (1 - Math.min(1, overallPct / 100))
                      : RING_HERO_CIRC"
                    :transform="`rotate(-90 ${RING_HERO_SIZE / 2} ${RING_HERO_SIZE / 2})`"
                  />
                </svg>
                <div class="nd-ring-center">
                  <p class="nd-ring-num">
                    {{ animOverall }}<span class="nd-ring-u">%</span>
                  </p>
                  <p class="nd-mono nd-mono--dim">READINESS</p>
                </div>
                <p class="nd-ring-band">[ {{ readinessBand }} ]</p>
              </div>

              <!-- RIGHT: per-subject dot-row progress -->
              <ul class="nd-sub-list">
                <li
                  v-for="sub in subjectRings.slice(0, 5)" :key="sub.id"
                  class="nd-sub-row"
                >
                  <div class="nd-sub-head">
                    <p class="nd-sub-name">{{ sub.name }}</p>
                    <p class="nd-sub-pct">
                      <span class="nd-sub-pct-num">{{ sub.pct }}</span>
                      <span class="nd-sub-pct-u">%</span>
                    </p>
                  </div>
                  <div class="nd-sub-dots">
                    <span
                      v-for="(d, i) in subjectDots(sub)" :key="i"
                      class="nd-dot"
                      :class="[`nd-dot--${d.kind}`]"
                      :style="{ animationDelay: (i * 22) + 'ms' }"
                    />
                  </div>
                  <p class="nd-mono nd-mono--dim">
                    <strong class="nd-sub-strong">{{ sub.mastered }}</strong> OF {{ sub.total }} MASTERED
                    <template v-if="sub.weak > 0"> · {{ sub.weak }} WEAK</template>
                  </p>
                </li>

                <li v-if="subjectRings.length === 0" class="nd-sub-empty">
                  <p class="nd-mono nd-mono--dim">— NO SUBJECTS INITIALISED —</p>
                  <p class="nd-sub-empty-sub">Finish onboarding to unlock mastery tracking.</p>
                </li>
              </ul>
            </div>
          </section>

          <!-- ─── 4. PRIORITY TOPICS ─── -->
          <section v-if="topicCases.length" class="nd-priority nd-reveal">
            <div class="nd-sec-head">
              <p class="nd-eyebrow nd-eyebrow--plain">PRIORITY TOPICS</p>
              <button class="nd-inline-link" @click="router.push('/student/practice')">
                PRACTICE <span>→</span>
              </button>
            </div>

            <ol class="nd-topic-list">
              <li
                v-for="(t, i) in topicCases" :key="t.topic_id"
                class="nd-topic-row"
                @click="router.push('/student/practice')"
              >
                <span class="nd-topic-index nd-mono">{{ String(i + 1).padStart(2, '0') }}</span>
                <span
                  class="nd-dot"
                  :class="[`nd-dot--${topicDotKind(t.mastery_state)}`]"
                  :title="t.mastery_state"
                />
                <div class="nd-topic-body">
                  <p class="nd-topic-name">{{ t.topic_name }}</p>
                  <p class="nd-mono nd-mono--dim">
                    {{ t.intervention_mode.replace(/_/g, ' ').toUpperCase() }} · GAP {{ bp(t.gap_score) }}
                  </p>
                </div>
                <span
                  class="nd-topic-tag nd-mono"
                  :class="[`nd-topic-tag--${t.intervention_urgency}`]"
                >[{{ topicTagLabel(t) }}]</span>
              </li>
            </ol>
          </section>

          <div class="nd-foot-spacer" />
        </div>
      </div>

      <!-- ─── RIGHT RAIL (arena + divider + feed) ──────────────────── -->
      <div v-if="!loading" class="nd-rail" ref="rpBodyRef">

        <!-- ══════ ARENA ══════ -->
        <section class="nd-arena" :style="{ flex: `${splitRatio} 1 0` }">

          <!-- Top strip -->
          <div class="nd-arena-top">
            <div class="nd-arena-top-l">
              <span class="nd-tag nd-tag--pulse">LIVE ARENA</span>
              <span class="nd-mono nd-mono--dim">DRILL · REAL-TIME</span>
            </div>
            <div class="nd-arena-score-head">
              <span class="nd-mono nd-mono--dim">SCORE</span>
              <span class="nd-arena-score-val" :class="{ 'nd-arena-score-val--pulse': scorePulse }">
                {{ animScore }}
              </span>
              <Transition name="nd-float">
                <span v-if="qtFloatId" :key="qtFloatId" class="nd-arena-float">+{{ qtGain }}</span>
              </Transition>
            </div>
          </div>

          <!-- Streak -->
          <div class="nd-arena-streak">
            <span class="nd-mono nd-mono--dim">STREAK</span>
            <div class="nd-streak-row">
              <span
                v-for="(on, i) in streakDots" :key="i"
                class="nd-dot"
                :class="on ? 'nd-dot--filled' : 'nd-dot--faint'"
              />
            </div>
            <span class="nd-mono nd-arena-streak-x">×{{ qtStreak }}</span>
            <span v-if="qtBest > 0" class="nd-mono nd-mono--dim">BEST {{ qtBest }}</span>
          </div>

          <!-- Timer cells -->
          <div
            class="nd-timer-cells"
            :class="{
              'nd-timer-cells--low':  qtTimer < 6 && !qtLocked,
              'nd-timer-cells--done': qtLocked,
            }"
          >
            <span
              v-for="(on, i) in timerCells" :key="i"
              class="nd-tcell"
              :class="{ 'nd-tcell--on': on }"
            />
          </div>

          <!-- Scroll body -->
          <div class="nd-arena-scroll">
            <div class="nd-arena-topmeta">
              <span class="nd-mono">{{ qtQuestion?.topic || 'REAL QUESTIONS' }}</span>
              <div class="nd-pips">
                <span
                  v-for="(_, i) in qtPips" :key="i"
                  class="nd-pip"
                  :class="{ 'nd-pip--on': i === qtIndex }"
                />
              </div>
            </div>

            <!-- Solidify popup — overlays the question area on wrong/timeout.
                 Holds the next question back until it dismisses. -->
            <Transition name="nd-solidify-pop">
              <div
                v-if="solidifyPrompt"
                class="nd-solidify-pop"
                role="dialog"
                aria-live="polite"
                aria-label="Known weakness"
              >
                <p class="nd-solidify-pop-kicker">KNOWN WEAKNESS</p>
                <h3 class="nd-solidify-pop-title">Solidify {{ solidifyPrompt.title }}?</h3>
                <p class="nd-solidify-pop-text">{{ solidifyPrompt.detail }}</p>
                <div class="nd-solidify-pop-actions">
                  <button class="nd-solidify-pop-secondary" @click="userDismissSolidifyPrompt">NOT NOW</button>
                  <button class="nd-solidify-pop-primary" @click="solidifyKnownWeakness">SOLIDIFY</button>
                </div>
                <!-- Countdown bar — depletes over SOLIDIFY_PROMPT_MS -->
                <div class="nd-solidify-pop-timer" aria-hidden="true">
                  <span class="nd-solidify-pop-timer-fill" />
                </div>
              </div>
            </Transition>

            <Transition name="nd-slide" mode="out-in">
              <div :key="qtKey" class="nd-arena-qbox">
                <p class="nd-arena-question">
                  <MathText
                    v-if="qtQuestion?.prompt"
                    :text="qtQuestion.prompt"
                  />
                  <span v-else>{{ arenaError || 'Loading real questions...' }}</span>
                </p>

                <ul v-if="qtQuestion" class="nd-arena-options">
                  <li v-for="opt in qtQuestion.options" :key="opt.id">
                    <button
                      class="nd-opt"
                      :class="{
                        'nd-opt--correct': qtLocked && isQtOptionCorrect(opt),
                        'nd-opt--wrong':   qtLocked && isQtOptionWrong(opt),
                        'nd-opt--faded':   qtLocked && !isQtOptionCorrect(opt) && qtChoice !== opt.id,
                        'nd-opt--shake':   qtLocked && isQtOptionWrong(opt),
                      }"
                      :disabled="qtLocked || arenaSubmitting"
                      @click="pickOption(opt.id)"
                    >
                      <span class="nd-opt-letter">{{ opt.letter }}</span>
                      <span class="nd-opt-sep" />
                      <span class="nd-opt-text"><MathText :text="opt.label" /></span>
                      <span class="nd-opt-result" aria-hidden="true">
                        <template v-if="qtLocked && isQtOptionCorrect(opt)">[✓]</template>
                        <template v-else-if="qtLocked && isQtOptionWrong(opt)">[✗]</template>
                      </span>
                      <span class="nd-opt-sweep" />
                    </button>
                  </li>
                </ul>

                <Transition name="nd-reveal">
                  <div
                    v-if="qtLocked && qtFeedback"
                    class="nd-arena-explain"
                    :class="{ 'nd-arena-explain--err': !qtIsCorrect }"
                  >
                    <p class="nd-mono nd-arena-explain-head">
                      <template v-if="qtIsCorrect">
                        + {{ qtGain }} PTS<template v-if="qtStreak > 1"> · {{ qtStreak }}× COMBO</template>
                      </template>
                      <template v-else-if="qtChoice === '__timeout__'">[ TIME UP ]</template>
                      <template v-else>[ STREAK BROKEN ]</template>
                    </p>
                    <QuestionFeedback
                      v-bind="qtFeedback"
                      :show-review-action="true"
                      review-label="Study Topic"
                      @review="openTopic"
                      @next="nextQuestion"
                    />
                    <div v-if="false" class="nd-arena-explain-actions">
                      <button class="nd-cta nd-cta--ghost nd-cta--sm" @click="openTopic">
                        STUDY <span class="nd-cta-arrow">→</span>
                      </button>
                      <button class="nd-cta nd-cta--sm" @click="nextQuestion">
                        NEXT <span class="nd-cta-arrow">→</span>
                      </button>
                    </div>
                  </div>
                </Transition>
              </div>
            </Transition>
          </div>

        </section>

        <!-- ══════ DIVIDER ══════ -->
        <div
          class="nd-divider"
          role="separator"
          aria-orientation="horizontal"
          title="Drag to resize"
          @pointerdown="onDividerPointerDown"
          @pointermove="onDividerPointerMove"
          @pointerup="onDividerPointerUp"
          @pointercancel="onDividerPointerUp"
        >
          <div class="nd-divider-grip"><span /><span /><span /></div>
        </div>

        <!-- ══════ LIVE FEED ══════ -->
        <section class="nd-feed" :style="{ flex: `${1 - splitRatio} 1 0` }">

          <div class="nd-feed-head">
            <div class="nd-feed-head-l">
              <span class="nd-eyebrow-dot nd-eyebrow-dot--sm" />
              <span class="nd-eyebrow nd-eyebrow--plain nd-feed-title-label">LIVE FEED</span>
            </div>
            <button class="nd-inline-link" @click="router.push('/student/progress')">
              ALL <span>→</span>
            </button>
          </div>

          <!-- CONTINUE (hero) card -->
          <button
            v-if="continueItem"
            class="nd-continue"
            @click="router.push(continueItem.route)"
          >
            <div class="nd-continue-top">
              <span class="nd-tag nd-tag--pulse nd-tag--inv">CONTINUE</span>
              <span class="nd-mono nd-mono--inv">{{ continueItem.time }}</span>
            </div>
            <p class="nd-continue-title">{{ continueItem.title }}</p>
            <p class="nd-continue-sub">{{ continueItem.sub }}</p>
            <span class="nd-continue-arrow">OPEN →</span>
          </button>

          <!-- TIMELINE -->
          <ol class="nd-feed-list">
            <li
              v-for="it in feedRest" :key="it.id"
              class="nd-feed-row"
              :class="{ 'nd-feed-row--critical': it.critical }"
              @click="router.push(it.route)"
            >
              <span class="nd-feed-time nd-mono">{{ it.time }}</span>
              <span
                class="nd-feed-tag nd-mono"
                :class="[`nd-feed-tag--${it.tag.toLowerCase()}`]"
              >[{{ it.tag }}]</span>
              <div class="nd-feed-body">
                <p class="nd-feed-title">{{ it.title }}</p>
                <p class="nd-mono nd-mono--dim">{{ it.sub }}</p>
              </div>
            </li>
          </ol>
        </section>
      </div>
    </div>

    <!-- ════════ BOTTOM SYSTEM BAR ════════ -->
    <footer class="nd-systembar">
      <span class="nd-sys">BUILD/03</span>
      <span class="nd-sys-sep">·</span>
      <span class="nd-sys">MODE/STUDENT</span>
      <span class="nd-sys-sep">·</span>
      <span class="nd-sys">READINESS/{{ overallPct }}%</span>
      <span class="nd-sys-sep">·</span>
      <span class="nd-sys">STREAK/{{ vitalStats.streak }}D</span>
      <span class="nd-sys-sep">·</span>
      <span class="nd-sys">SCORE/{{ qtScore }}</span>
      <span class="nd-sys-sep">·</span>
      <span class="nd-sys nd-sys--dim">UTC+0 · {{ timeStamp }}</span>
    </footer>
  </div>
</template>

<style scoped>
/* ══════════════════════════════════════════════════════════════════
   TOKENS (page-local — doesn't touch app theme)
   ══════════════════════════════════════════════════════════════════ */
.nd {
  --nd-canvas:   #ecebe6;
  --nd-surface:  #f4f3ee;
  --nd-border:   rgba(20, 17, 14, 0.09);
  --nd-border-2: rgba(20, 17, 14, 0.16);
  --nd-ink:      #14110e;
  --nd-ink-1:    rgba(20, 17, 14, 0.92);
  --nd-ink-2:    rgba(20, 17, 14, 0.58);
  --nd-ink-3:    rgba(20, 17, 14, 0.36);
  --nd-ink-4:    rgba(20, 17, 14, 0.14);
  --nd-red:      #d71921;
  --nd-amber:    #d97706;
  --nd-green:    #15803d;

  --nd-sans: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --nd-mono: 'Space Mono', 'SF Mono', 'JetBrains Mono', Consolas, ui-monospace, monospace;
  --nd-doto: 'Doto', 'Space Mono', ui-monospace, monospace;

  color: var(--nd-ink-1);
  background: var(--nd-canvas);
  font-family: var(--nd-sans);
  font-feature-settings: 'tnum';
  letter-spacing: -0.005em;

  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.nd[data-nd-dark='1'] {
  --nd-canvas:   #0a0908;
  --nd-surface:  #121110;
  --nd-border:   rgba(245, 243, 238, 0.08);
  --nd-border-2: rgba(245, 243, 238, 0.16);
  --nd-ink:      #f5f3ee;
  --nd-ink-1:    rgba(245, 243, 238, 0.92);
  --nd-ink-2:    rgba(245, 243, 238, 0.58);
  --nd-ink-3:    rgba(245, 243, 238, 0.38);
  --nd-ink-4:    rgba(245, 243, 238, 0.14);
  --nd-red:      #ff4a4f;
  --nd-amber:    #fbbf24;
  --nd-green:    #22c55e;
}
.nd *, .nd *::before, .nd *::after { box-sizing: border-box; }

/* ══════════════════════════════════════════════════════════════════
   STATUS + SYSTEM BARS
   ══════════════════════════════════════════════════════════════════ */
.nd-statusbar,
.nd-systembar {
  flex: 0 0 auto;
  display: flex; align-items: center;
  padding: 14px 36px; gap: 20px;
  background: var(--nd-canvas);
}
.nd-statusbar { justify-content: space-between; border-bottom: 1px solid var(--nd-border); }
.nd-systembar { border-top: 1px solid var(--nd-border); overflow-x: auto; white-space: nowrap; scrollbar-width: none; }
.nd-systembar::-webkit-scrollbar { display: none; }
.nd-statusbar-left, .nd-statusbar-right { display: flex; align-items: center; gap: 16px; }
.nd-stat, .nd-sys {
  font-family: var(--nd-mono); font-size: 10px;
  letter-spacing: 0.14em; color: var(--nd-ink-2);
}
.nd-stat--dim, .nd-sys--dim { color: var(--nd-ink-3); }
.nd-stat--accent { color: var(--nd-ink); font-weight: 700; }
.nd-sys-sep { color: var(--nd-ink-4); font-family: var(--nd-mono); font-size: 10px; }

.nd-version-switch {
  display: flex;
  border: 1px solid var(--nd-border-2);
  border-radius: 999px; padding: 2px; gap: 1px;
}
.nd-version-btn {
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700; letter-spacing: 0.18em;
  color: var(--nd-ink-2); background: transparent;
  border: none; padding: 4px 10px; cursor: pointer;
  border-radius: 999px;
  transition: background 140ms ease, color 140ms ease;
}
.nd-version-btn:hover { color: var(--nd-ink); }
.nd-version-btn--on { background: var(--nd-ink); color: var(--nd-canvas); }

.nd-toggle {
  background: transparent; border: none; cursor: pointer;
  display: inline-flex; align-items: center; gap: 8px; padding: 2px 4px;
}
.nd-toggle-track {
  width: 30px; height: 16px; border-radius: 999px;
  border: 1px solid var(--nd-border-2); position: relative;
}
.nd-toggle-thumb {
  position: absolute; top: 1px; left: 1px;
  width: 12px; height: 12px;
  background: var(--nd-ink); border-radius: 50%;
  transition: left 160ms cubic-bezier(0, 0, 0.2, 1);
}
.nd-toggle-track[data-on='1'] .nd-toggle-thumb { left: 15px; }
.nd-toggle-label {
  font-family: var(--nd-mono); font-size: 10px;
  letter-spacing: 0.16em; color: var(--nd-ink-2);
}

/* ══════════════════════════════════════════════════════════════════
   SHELL — v1 layout: left flex column + right rail
   ══════════════════════════════════════════════════════════════════ */
.nd-shell {
  flex: 1 1 0;
  display: flex;
  overflow: hidden;
  background: var(--nd-canvas);
}

/* ══════════════════════════════════════════════════════════════════
   LEFT COLUMN
   ══════════════════════════════════════════════════════════════════ */
.nd-left {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

/* ── Greeting (compact, doesn't stretch) ── */
.nd-greet {
  align-self: flex-start;
  max-width: calc(100% - 48px);
  margin: 32px 36px 0;
  padding: 22px 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 56px;
  border-radius: 14px;
  background: var(--nd-surface);
  flex-shrink: 0;
}
.nd-greet-l { min-width: 0; }
.nd-greet-date { margin: 0 0 4px; font-size: 9px; }
.nd-greet-title {
  font-family: var(--nd-sans);
  font-weight: 500;
  font-size: 22px;
  letter-spacing: -0.01em;
  color: var(--nd-ink);
  margin: 0;
  text-wrap: balance;
}
.nd-greet-name {
  font-weight: 700;
  color: var(--nd-ink);
}
.nd-greet-subs {
  display: flex;
  gap: 28px;
  flex-shrink: 0;
}
.nd-greet-sub { text-align: center; }
.nd-greet-sub-pct {
  margin: 0;
  font-family: var(--nd-sans);
  font-weight: 300;
  font-size: 22px;
  letter-spacing: -0.02em;
  color: var(--nd-ink);
  font-variant-numeric: tabular-nums;
  line-height: 1;
}
.nd-greet-sub-u {
  font-family: var(--nd-mono);
  font-size: 11px;
  color: var(--nd-ink-3);
  letter-spacing: 0.14em;
  margin-left: 1px;
}
.nd-greet-sub-name {
  margin: 4px 0 0;
  font-size: 9px;
}

/* ── Left loading / error ── */
.nd-left-loading, .nd-left-error {
  flex: 1 1 0;
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  gap: 12px; padding: 48px;
}
.nd-error-msg { font-size: 14px; color: var(--nd-ink-2); }

/* ── Scrollable main ── */
.nd-left-main {
  flex: 1 1 0;
  overflow-y: auto;
  padding: 48px clamp(28px, 4vw, 52px) 0;
  display: flex;
  flex-direction: column;
  gap: 72px;
}
.nd-left-main::-webkit-scrollbar { width: 6px; }
.nd-left-main::-webkit-scrollbar-thumb { background: var(--nd-ink-4); border-radius: 99px; }
.nd-left-main::-webkit-scrollbar-thumb:hover { background: var(--nd-ink-3); }

.nd-reveal { animation: nd-enter 640ms cubic-bezier(0.16, 1, 0.3, 1) both; }
.nd-reveal:nth-child(2) { animation-delay:  80ms; }
.nd-reveal:nth-child(3) { animation-delay: 160ms; }
.nd-reveal:nth-child(4) { animation-delay: 240ms; }
.nd-reveal:nth-child(5) { animation-delay: 320ms; }
@keyframes nd-enter {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0);    }
}

/* Common pieces */
.nd-mono {
  font-family: var(--nd-mono); font-size: 10px;
  letter-spacing: 0.18em; color: var(--nd-ink-2);
  text-transform: uppercase;
}
.nd-mono--dim { color: var(--nd-ink-3); }
.nd-mono--red { color: var(--nd-red); }
.nd-mono--inv { color: var(--nd-canvas); }
.nd-eyebrow {
  display: inline-flex; align-items: center; gap: 10px;
  font-family: var(--nd-mono); font-size: 10px;
  letter-spacing: 0.22em; color: var(--nd-ink-2);
  text-transform: uppercase; margin: 0;
}
.nd-eyebrow--plain {
  color: var(--nd-ink-3); font-weight: 700;
}
.nd-eyebrow-dot {
  width: 7px; height: 7px; background: var(--nd-red); border-radius: 50%;
  box-shadow: 0 0 0 3px rgba(215, 25, 33, 0.12);
  animation: nd-blink 1.8s ease-in-out infinite;
}
.nd-eyebrow-dot--sm { width: 6px; height: 6px; box-shadow: 0 0 0 2px rgba(215, 25, 33, 0.10); }
@keyframes nd-blink { 0%, 100% { opacity: 1; } 50% { opacity: 0.25; } }

.nd-tag {
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700; letter-spacing: 0.22em;
  color: var(--nd-ink);
  display: inline-flex; align-items: center; gap: 8px;
}
.nd-tag::before {
  content: ''; width: 6px; height: 6px;
  background: var(--nd-ink); border-radius: 50%;
}
.nd-tag--pulse::before {
  background: var(--nd-red);
  box-shadow: 0 0 0 3px rgba(215, 25, 33, 0.12);
  animation: nd-blink 1.8s ease-in-out infinite;
}
.nd-tag--dim { color: var(--nd-ink-3); }
.nd-tag--dim::before { background: var(--nd-ink-3); }
.nd-tag--inv { color: var(--nd-canvas); }
.nd-tag--inv::before { background: var(--nd-red); }

.nd-sec-head {
  display: flex; justify-content: space-between; align-items: center;
  padding-bottom: 16px; margin-bottom: 32px;
  border-bottom: 1px solid var(--nd-border);
}
.nd-inline-link {
  background: transparent; border: none; cursor: pointer; padding: 0;
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700; letter-spacing: 0.22em;
  color: var(--nd-ink);
  display: inline-flex; align-items: center; gap: 8px;
}
.nd-inline-link > span { transition: transform 160ms ease; }
.nd-inline-link:hover > span { transform: translateX(3px); }

.nd-cta {
  display: inline-flex; align-items: center; gap: 10px;
  padding: 10px 18px; border-radius: 999px;
  background: var(--nd-ink); color: var(--nd-canvas);
  border: 1px solid var(--nd-ink);
  font-family: var(--nd-mono);
  font-size: 11px; font-weight: 700; letter-spacing: 0.22em;
  cursor: pointer;
  transition: transform 120ms ease;
}
.nd-cta:hover { transform: translateY(-1px); }
.nd-cta-arrow { transition: transform 160ms ease; }
.nd-cta:hover .nd-cta-arrow { transform: translateX(3px); }
.nd-cta--ghost { background: transparent; color: var(--nd-ink); border-color: var(--nd-border-2); }
.nd-cta--ghost:hover { border-color: var(--nd-ink); }
.nd-cta--sm { padding: 7px 13px; font-size: 10px; letter-spacing: 0.2em; }

/* ══════════════════════════════════════════════════════════════════
   1. DIRECTIVE
   ══════════════════════════════════════════════════════════════════ */
.nd-directive {
  border-radius: 14px;
  padding: 32px 36px;
  cursor: pointer;
  background: var(--nd-surface);
  transition: background 140ms ease, transform 140ms ease;
  position: relative;
}
.nd-directive:hover {
  background: color-mix(in srgb, var(--nd-ink) 4%, var(--nd-surface));
  transform: translateY(-1px);
}
.nd-directive--idle { cursor: default; }
.nd-directive--idle:hover { transform: none; background: var(--nd-surface); }
.nd-dir-head {
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 18px;
}
.nd-dir-title {
  font-family: var(--nd-sans);
  font-weight: 400;
  font-size: 22px;
  letter-spacing: -0.012em;
  line-height: 1.3;
  color: var(--nd-ink);
  margin: 0 0 12px;
  text-wrap: balance;
}
.nd-dir-sub {
  font-size: 13px;
  color: var(--nd-ink-2);
  line-height: 1.55;
  margin: 0 0 28px;
  text-wrap: balance;
}
.nd-dir-actions {
  display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
}
.nd-dir-type {
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700; letter-spacing: 0.2em;
  color: var(--nd-ink-3);
  padding: 5px 12px;
  border: 1px solid var(--nd-border);
  border-radius: 999px;
}

/* ══════════════════════════════════════════════════════════════════
   2. VITALS (4 tiles, no boxes — typography + vertical dividers)
   ══════════════════════════════════════════════════════════════════ */
.nd-vitals {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0;
}
.nd-vital {
  padding: 12px 32px;
  border-left: 1px solid var(--nd-border);
  display: flex; flex-direction: column; gap: 12px;
}
.nd-vital:first-child { padding-left: 0; border-left: none; }
.nd-vital p { margin: 0; }
.nd-vital-num {
  font-family: var(--nd-sans);
  font-weight: 300;
  font-size: clamp(36px, 4vw, 52px);
  letter-spacing: -0.03em;
  color: var(--nd-ink);
  line-height: 0.95;
  font-variant-numeric: tabular-nums;
  display: inline-flex; align-items: baseline; gap: 2px;
}
.nd-vital-u {
  font-family: var(--nd-mono);
  font-size: 13px;
  color: var(--nd-ink-3);
  letter-spacing: 0.14em;
  font-weight: 400;
}
.nd-vital-cap { font-size: 9px !important; }

/* ══════════════════════════════════════════════════════════════════
   3. MASTERY
   ══════════════════════════════════════════════════════════════════ */
.nd-mastery-body {
  display: grid;
  grid-template-columns: minmax(200px, auto) minmax(0, 1fr);
  gap: 72px;
  align-items: start;
}

.nd-ring-hero {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 4px 8px;
}
.nd-ring-svg { display: block; }
.nd-ring-arc {
  transition: stroke-dashoffset 1400ms cubic-bezier(0.16, 1, 0.3, 1);
}
.nd-ring-center {
  position: absolute;
  top: 76px;
  left: 0; right: 0;
  text-align: center;
}
.nd-ring-num {
  margin: 0;
  font-family: var(--nd-doto);
  font-weight: 900;
  font-size: 56px;
  letter-spacing: -0.04em;
  color: var(--nd-ink);
  line-height: 0.9;
  font-variant-numeric: tabular-nums;
}
.nd-ring-u {
  font-family: var(--nd-sans);
  font-weight: 300;
  font-size: 20px;
  color: var(--nd-ink-3);
  margin-left: 1px;
}
.nd-ring-center p:last-child {
  margin-top: 4px;
  font-size: 9px;
}
.nd-ring-band {
  margin: 0;
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--nd-ink);
}

/* Per-subject rows (dot row + name + %) */
.nd-sub-list {
  list-style: none;
  padding: 0; margin: 0;
  display: flex; flex-direction: column;
}
.nd-sub-row {
  padding: 22px 0;
  border-bottom: 1px solid var(--nd-border);
  display: flex; flex-direction: column; gap: 12px;
}
.nd-sub-row:first-child { padding-top: 0; }
.nd-sub-row:last-child { border-bottom: none; padding-bottom: 0; }

.nd-sub-head {
  display: flex; justify-content: space-between; align-items: baseline;
}
.nd-sub-name {
  margin: 0;
  font-family: var(--nd-sans);
  font-weight: 500;
  font-size: 14px;
  letter-spacing: -0.005em;
  color: var(--nd-ink);
}
.nd-sub-pct {
  margin: 0;
  display: inline-flex; align-items: baseline; gap: 2px;
  font-variant-numeric: tabular-nums;
}
.nd-sub-pct-num {
  font-family: var(--nd-sans);
  font-weight: 300;
  font-size: 20px;
  letter-spacing: -0.02em;
  color: var(--nd-ink);
  line-height: 1;
}
.nd-sub-pct-u {
  font-family: var(--nd-mono);
  font-size: 9px;
  color: var(--nd-ink-3);
  letter-spacing: 0.18em;
}
.nd-sub-dots {
  display: flex; flex-wrap: wrap; gap: 5px; align-items: center;
}
.nd-sub-strong { color: var(--nd-ink); font-weight: 700; }

.nd-sub-empty {
  padding: 28px 0;
  text-align: center;
  border: 1px dashed var(--nd-border-2);
  border-radius: 8px;
}
.nd-sub-empty-sub { color: var(--nd-ink-2); font-size: 13px; margin: 6px 0 0; }

/* Dot primitive (shared) */
.nd-dot {
  width: 10px; height: 10px;
  border-radius: 50%;
  display: inline-block;
  background: transparent;
  animation: nd-dot-in 420ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes nd-dot-in {
  from { opacity: 0; transform: scale(0.4); }
  to   { opacity: 1; transform: scale(1); }
}
.nd-dot--filled { background: var(--nd-ink); }
.nd-dot--ring   { background: transparent; border: 1.5px solid var(--nd-ink); }
.nd-dot--faint  { background: var(--nd-ink-4); }

/* ══════════════════════════════════════════════════════════════════
   4. PRIORITY TOPICS
   ══════════════════════════════════════════════════════════════════ */
.nd-topic-list {
  list-style: none; padding: 0; margin: 0;
  display: flex; flex-direction: column;
}
.nd-topic-row {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  gap: 20px; align-items: center;
  padding: 22px 4px;
  border-bottom: 1px solid var(--nd-border);
  cursor: pointer;
  transition: background 120ms ease, padding-left 140ms ease;
}
.nd-topic-row:last-child { border-bottom: none; }
.nd-topic-row:hover { background: var(--nd-surface); padding-left: 10px; }
.nd-topic-index {
  font-weight: 700; color: var(--nd-ink-3);
  font-variant-numeric: tabular-nums;
}
.nd-topic-body > p { margin: 0; }
.nd-topic-name {
  font-family: var(--nd-sans);
  font-size: 14px;
  font-weight: 500;
  color: var(--nd-ink);
  letter-spacing: -0.005em;
  line-height: 1.3;
}
.nd-topic-body > p:last-child {
  margin-top: 3px;
  font-size: 9px;
}
.nd-topic-tag {
  font-weight: 700; letter-spacing: 0.18em;
  font-size: 10px; color: var(--nd-ink-2);
}
.nd-topic-tag--high { color: var(--nd-red); }
.nd-topic-tag--medium { color: var(--nd-amber); }
.nd-topic-tag--low { color: var(--nd-ink-3); }

.nd-foot-spacer { height: 96px; flex-shrink: 0; }

/* ══════════════════════════════════════════════════════════════════
   RIGHT RAIL — arena + divider + feed
   ══════════════════════════════════════════════════════════════════ */
.nd-rail {
  width: 26rem;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin: 32px 28px 32px 0;
  border-radius: 16px;
  background: var(--nd-surface);
}

/* ══════ ARENA ══════ */
.nd-arena {
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 24px 26px 20px;
  gap: 18px;
  overflow: hidden;
}

.nd-arena-top {
  display: flex; justify-content: space-between; align-items: center;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--nd-border);
}
.nd-arena-top-l { display: inline-flex; align-items: center; gap: 10px; min-width: 0; }
.nd-arena-top-l .nd-mono--dim { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.nd-arena-score-head {
  display: inline-flex; align-items: baseline; gap: 8px;
  position: relative;
}
.nd-arena-score-val {
  font-family: var(--nd-doto);
  font-weight: 700;
  font-size: 28px;
  letter-spacing: -0.02em;
  color: var(--nd-ink);
  font-variant-numeric: tabular-nums;
  line-height: 1;
  transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
}
.nd-arena-score-val--pulse { transform: scale(1.08); }
.nd-arena-float {
  position: absolute;
  top: -16px; right: 0;
  font-family: var(--nd-mono);
  font-weight: 700; font-size: 11px;
  color: var(--nd-green);
  pointer-events: none;
}
.nd-float-enter-from { opacity: 0; transform: translateY(4px) scale(0.9); }
.nd-float-enter-active { transition: opacity 260ms ease, transform 260ms ease; }
.nd-float-leave-active { transition: opacity 900ms ease, transform 900ms cubic-bezier(0.16, 1, 0.3, 1); }
.nd-float-leave-to { opacity: 0; transform: translateY(-22px) scale(1); }

/* streak line */
.nd-arena-streak {
  display: flex; align-items: center; gap: 10px;
  flex-wrap: wrap;
}
.nd-streak-row {
  display: inline-flex; align-items: center; gap: 5px;
}
.nd-arena-streak-x { color: var(--nd-ink); font-weight: 700; }

/* timer cells */
.nd-timer-cells {
  display: grid;
  grid-template-columns: repeat(20, 1fr);
  gap: 2px;
}
.nd-tcell {
  height: 10px;
  background: var(--nd-ink-4);
  border-radius: 1px;
  transition: background 120ms ease;
}
.nd-tcell--on { background: var(--nd-ink); }
.nd-timer-cells--low .nd-tcell--on {
  background: var(--nd-red);
  animation: nd-blink 0.9s ease-in-out infinite;
}
.nd-timer-cells--done .nd-tcell--on { background: var(--nd-ink-2); animation: none; }

/* Arena scroll body */
.nd-arena-scroll {
  position: relative;
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  padding-top: 8px;
  display: flex; flex-direction: column; gap: 20px;
}
.nd-arena-scroll::-webkit-scrollbar { width: 5px; }
.nd-arena-scroll::-webkit-scrollbar-thumb { background: var(--nd-ink-4); border-radius: 99px; }

.nd-arena-topmeta {
  display: flex; justify-content: space-between; align-items: center;
  gap: 8px;
}
.nd-pips { display: inline-flex; gap: 4px; }
.nd-pip {
  width: 14px; height: 3px;
  background: var(--nd-ink-4);
  border-radius: 99px;
  transition: background 280ms ease, width 280ms ease;
}
.nd-pip--on { width: 22px; background: var(--nd-ink); }

.nd-arena-question {
  font-family: var(--nd-sans);
  font-weight: 400;
  font-size: 15px;
  line-height: 1.35;
  letter-spacing: -0.008em;
  color: var(--nd-ink);
  margin: 0;
  text-wrap: balance;
}

.nd-arena-options {
  list-style: none; padding: 0; margin: 0;
  display: flex; flex-direction: column; gap: 10px;
}
.nd-opt {
  position: relative; overflow: hidden;
  width: 100%;
  display: grid;
  grid-template-columns: 28px 1px minmax(0, 1fr) auto;
  align-items: center; gap: 10px;
  text-align: left;
  padding: 14px 16px;
  background: transparent;
  border: 1px solid var(--nd-border-2);
  border-radius: 8px;
  color: inherit;
  cursor: pointer;
  font-family: var(--nd-sans);
  font-size: 13px;
  line-height: 1.35;
  transition:
    background 180ms ease,
    border-color 180ms ease,
    transform 140ms ease,
    opacity 180ms ease;
}
.nd-opt:hover:not(:disabled) { border-color: var(--nd-ink); transform: translateX(2px); }
.nd-opt:disabled { cursor: default; transform: none; }
.nd-opt-letter {
  font-family: var(--nd-mono);
  font-size: 11px; font-weight: 700; letter-spacing: 0.14em;
  color: var(--nd-ink); text-align: center;
}
.nd-opt-sep { width: 1px; height: 16px; background: var(--nd-border-2); }
.nd-opt-text { color: var(--nd-ink-1); }
.nd-opt-result {
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700; letter-spacing: 0.16em;
  min-width: 24px; text-align: right;
}
.nd-opt-sweep {
  position: absolute; inset: 0; pointer-events: none;
  background: linear-gradient(
    90deg,
    transparent 0%,
    color-mix(in srgb, var(--nd-green) 22%, transparent) 50%,
    transparent 100%
  );
  transform: translateX(-100%);
  transition: transform 700ms cubic-bezier(0.16, 1, 0.3, 1);
}
.nd-opt--correct {
  border-color: var(--nd-green);
  background: color-mix(in srgb, var(--nd-green) 8%, transparent);
}
.nd-opt--correct .nd-opt-letter,
.nd-opt--correct .nd-opt-result { color: var(--nd-green); }
.nd-opt--correct .nd-opt-sweep { transform: translateX(100%); }
.nd-opt--wrong {
  border-color: var(--nd-red);
  background: color-mix(in srgb, var(--nd-red) 8%, transparent);
}
.nd-opt--wrong .nd-opt-letter,
.nd-opt--wrong .nd-opt-result { color: var(--nd-red); }
.nd-opt--faded { opacity: 0.4; }
.nd-opt--shake { animation: nd-shake 360ms cubic-bezier(0.36, 0.07, 0.19, 0.97) 1; }
@keyframes nd-shake {
  10%, 90% { transform: translateX(-1px); }
  20%, 80% { transform: translateX(2px); }
  30%, 50%, 70% { transform: translateX(-3px); }
  40%, 60% { transform: translateX(3px); }
}

.nd-slide-enter-from { opacity: 0; transform: translateY(8px); }
.nd-slide-enter-active { transition: opacity 380ms ease, transform 380ms cubic-bezier(0.16, 1, 0.3, 1); }
.nd-slide-leave-active { transition: opacity 180ms ease, transform 180ms ease; }
.nd-slide-leave-to { opacity: 0; transform: translateY(-6px); }

/* Post-answer */
.nd-arena-explain {
  margin-top: 6px;
  padding: 12px 14px;
  border-left: 2px solid var(--nd-green);
  background: color-mix(in srgb, var(--nd-green) 6%, transparent);
  border-radius: 0 7px 7px 0;
}
.nd-arena-explain--err {
  border-left-color: var(--nd-red);
  background: color-mix(in srgb, var(--nd-red) 6%, transparent);
}
.nd-arena-explain-head { margin: 0 0 6px; color: var(--nd-ink); font-size: 10px; }
.nd-arena-explain :deep(.qf-shell) {
  padding: 0;
  border: none;
  background: transparent;
  gap: 14px;
}

.nd-arena-explain :deep(.qf-title) {
  font-size: 18px;
  font-family: var(--nd-sans);
}

.nd-arena-explain :deep(.qf-summary),
.nd-arena-explain :deep(.qf-intro),
.nd-arena-explain :deep(.qf-note-line),
.nd-arena-explain :deep(.qf-bullet),
.nd-arena-explain :deep(.qf-option-line),
.nd-arena-explain :deep(.qf-question),
.nd-arena-explain :deep(.qf-answer-text),
.nd-arena-explain :deep(.qf-step-note),
.nd-arena-explain :deep(.qf-step-work),
.nd-arena-explain :deep(.qf-option-choice) {
  font-size: 12px;
}

.nd-arena-explain :deep(.qf-btn--primary) {
  background: var(--nd-ink);
}
.nd-arena-explain-text {
  font-size: 12px; line-height: 1.5;
  color: var(--nd-ink-1);
  margin: 0 0 12px;
  text-wrap: balance;
}
.nd-arena-explain-actions { display: flex; gap: 6px; flex-wrap: wrap; }

.nd-reveal-enter-from { opacity: 0; transform: translateY(-6px); }
.nd-reveal-enter-active { transition: opacity 240ms ease, transform 240ms ease; }
.nd-reveal-leave-active { transition: opacity 140ms ease; }
.nd-reveal-leave-to { opacity: 0; }

/* ══════ SOLIDIFY POPUP — black card, floats beside the question ══════ */
.nd-solidify-pop {
  position: absolute;
  left: 8px;
  right: 8px;
  top: 28px;
  z-index: 30;
  padding: 16px 16px 14px;
  border-radius: 12px;
  background: #0a0908;
  color: #f5f3ee;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.28);
}
.nd-solidify-pop-kicker {
  margin: 0 0 8px;
  color: var(--nd-red);
  font-family: var(--nd-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.nd-solidify-pop-kicker::before {
  content: '';
  width: 6px; height: 6px;
  background: var(--nd-red);
  border-radius: 50%;
  box-shadow: 0 0 0 3px rgba(215, 25, 33, 0.18);
  animation: nd-blink 1.6s ease-in-out infinite;
}
.nd-solidify-pop-title {
  margin: 0;
  color: #f5f3ee;
  font-family: var(--nd-sans);
  font-size: 15px;
  font-weight: 500;
  letter-spacing: -0.01em;
  line-height: 1.25;
  text-wrap: balance;
}
.nd-solidify-pop-text {
  margin: 8px 0 14px;
  color: rgba(245, 243, 238, 0.62);
  font-size: 12px;
  line-height: 1.45;
  text-wrap: balance;
}
.nd-solidify-pop-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}
.nd-solidify-pop-primary,
.nd-solidify-pop-secondary {
  border-radius: 999px;
  cursor: pointer;
  font-family: var(--nd-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  padding: 7px 14px;
  border: none;
  transition: transform 130ms ease, background 130ms ease, color 130ms ease;
}
.nd-solidify-pop-primary {
  background: #f5f3ee;
  color: #0a0908;
}
.nd-solidify-pop-primary:hover { transform: translateY(-1px); }
.nd-solidify-pop-secondary {
  background: transparent;
  color: rgba(245, 243, 238, 0.58);
}
.nd-solidify-pop-secondary:hover { color: #f5f3ee; }
.nd-solidify-pop-primary:active,
.nd-solidify-pop-secondary:active { transform: translateY(0) scale(0.98); }

/* Countdown bar — depletes over SOLIDIFY_PROMPT_MS (5200ms) */
.nd-solidify-pop-timer {
  margin-top: 16px;
  height: 2px;
  width: 100%;
  background: rgba(245, 243, 238, 0.10);
  border-radius: 99px;
  overflow: hidden;
}
.nd-solidify-pop-timer-fill {
  display: block;
  height: 100%;
  width: 100%;
  background: rgba(245, 243, 238, 0.72);
  transform-origin: left center;
  animation: nd-solidify-countdown 5200ms linear forwards;
}
@keyframes nd-solidify-countdown {
  from { transform: scaleX(1); }
  to   { transform: scaleX(0); }
}

.nd-solidify-pop-enter-active,
.nd-solidify-pop-leave-active {
  transition: opacity 220ms ease, transform 300ms cubic-bezier(0.16, 1, 0.3, 1);
}
.nd-solidify-pop-enter-from {
  opacity: 0;
  transform: translateY(-8px) scale(0.97);
}
.nd-solidify-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}

/* ══════ DIVIDER ══════ */
.nd-divider {
  flex-shrink: 0;
  height: 14px;
  position: relative;
  cursor: row-resize;
  background: var(--nd-canvas);
  display: flex; align-items: center; justify-content: center;
}
.nd-divider-grip {
  display: inline-flex; gap: 3px; align-items: center;
}
.nd-divider-grip span {
  width: 3px; height: 3px;
  background: var(--nd-ink-3);
  border-radius: 50%;
  transition: background 140ms ease;
}
.nd-divider:hover .nd-divider-grip span { background: var(--nd-ink); }

/* ══════ FEED ══════ */
.nd-feed {
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 22px 26px 24px;
  gap: 20px;
  overflow: hidden;
}
.nd-feed-head {
  display: flex; justify-content: space-between; align-items: center;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--nd-border);
}
.nd-feed-head-l {
  display: inline-flex; align-items: center; gap: 8px;
}
.nd-feed-title-label { margin: 0; }

/* CONTINUE (inverted hero) */
.nd-continue {
  text-align: left;
  display: flex; flex-direction: column; gap: 4px;
  padding: 20px 22px;
  border-radius: 12px;
  border: 1px solid var(--nd-ink);
  background: var(--nd-ink);
  color: var(--nd-canvas);
  cursor: pointer;
  transition: transform 120ms ease;
}
.nd-continue:hover { transform: translateY(-1px); }
.nd-continue-top {
  display: flex; justify-content: space-between; align-items: center;
}
.nd-continue-title {
  font-family: var(--nd-sans);
  font-weight: 500;
  font-size: 14px;
  letter-spacing: -0.005em;
  line-height: 1.3;
  color: var(--nd-canvas);
  margin: 4px 0 0;
  text-wrap: balance;
}
.nd-continue-sub {
  font-size: 11px;
  color: var(--nd-canvas);
  opacity: 0.72;
  margin: 2px 0 0;
  line-height: 1.4;
}
.nd-continue-arrow {
  margin: 8px 0 0;
  font-family: var(--nd-mono);
  font-size: 10px; font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--nd-canvas); opacity: 0.86;
}

/* TIMELINE */
.nd-feed-list {
  list-style: none; padding: 0; margin: 0;
  display: flex; flex-direction: column;
  overflow-y: auto;
  flex: 1 1 0;
}
.nd-feed-list::-webkit-scrollbar { width: 5px; }
.nd-feed-list::-webkit-scrollbar-thumb { background: var(--nd-ink-4); border-radius: 99px; }
.nd-feed-row {
  display: grid;
  grid-template-columns: 36px auto minmax(0, 1fr);
  gap: 14px; align-items: start;
  padding: 16px 0;
  border-bottom: 1px dashed var(--nd-border);
  cursor: pointer;
  transition: background 120ms ease, padding-left 140ms ease;
}
.nd-feed-row:last-child { border-bottom: none; }
.nd-feed-row:hover { background: var(--nd-canvas); padding-left: 8px; }
.nd-feed-row--critical .nd-feed-title { font-weight: 700; }
.nd-feed-time {
  font-weight: 700; color: var(--nd-ink-3);
  font-variant-numeric: tabular-nums; padding-top: 1px;
}
.nd-feed-tag {
  font-weight: 700; letter-spacing: 0.16em;
  font-size: 9px; padding-top: 1px;
  color: var(--nd-ink-2); white-space: nowrap;
}
.nd-feed-tag--coach  { color: var(--nd-red); }
.nd-feed-tag--gap    { color: var(--nd-amber); }
.nd-feed-tag--review { color: var(--nd-ink-2); }
.nd-feed-tag--mock   { color: var(--nd-ink); }
.nd-feed-tag--games  { color: var(--nd-green); }
.nd-feed-tag--mem    { color: var(--nd-ink-2); }
.nd-feed-body > p { margin: 0; }
.nd-feed-title {
  font-family: var(--nd-sans); font-size: 12.5px;
  color: var(--nd-ink); letter-spacing: -0.005em; line-height: 1.3;
}
.nd-feed-body > p:last-child { margin-top: 2px; font-size: 9px; }

/* ══════════════════════════════════════════════════════════════════
   LOADING SPINNER
   ══════════════════════════════════════════════════════════════════ */
.nd-spinner { display: inline-flex; gap: 3px; }
.nd-spinner > span {
  width: 5px; height: 16px;
  background: var(--nd-ink);
  animation: nd-bar 1s ease-in-out infinite;
}
.nd-spinner > span:nth-child(2) { animation-delay: 0.08s; }
.nd-spinner > span:nth-child(3) { animation-delay: 0.16s; }
.nd-spinner > span:nth-child(4) { animation-delay: 0.24s; }
.nd-spinner > span:nth-child(5) { animation-delay: 0.32s; }
.nd-spinner > span:nth-child(6) { animation-delay: 0.40s; }
@keyframes nd-bar {
  0%, 100% { transform: scaleY(0.3); opacity: 0.4; }
  50%      { transform: scaleY(1);   opacity: 1;   }
}

/* ══════════════════════════════════════════════════════════════════
   RESPONSIVE
   ══════════════════════════════════════════════════════════════════ */
@media (max-width: 1180px) {
  .nd-mastery-body {
    grid-template-columns: 1fr;
    gap: 28px;
    justify-items: center;
  }
  .nd-sub-list { width: 100%; }
  .nd-vitals { grid-template-columns: repeat(2, 1fr); row-gap: 24px; }
  .nd-vital:nth-child(3) { border-left: none; padding-left: 0; }
  .nd-rail { width: 20rem; }
}
@media (max-width: 900px) {
  .nd-shell { flex-direction: column; overflow-y: auto; overflow-x: hidden; }
  .nd-rail {
    width: auto;
    min-height: 520px;
    margin: 0 18px 18px;
  }
}
@media (max-width: 640px) {
  .nd-statusbar, .nd-systembar { padding: 10px 14px; flex-wrap: wrap; gap: 10px; }
  .nd-greet { margin: 14px 14px 0; padding: 12px 16px; flex-direction: column; align-items: flex-start; gap: 14px; }
  .nd-greet-subs { gap: 18px; }
  .nd-left-main { padding: 22px 14px 0; gap: 32px; }
  .nd-directive { padding: 18px 18px; }
  .nd-dir-title { font-size: 18px; }
  .nd-vitals { grid-template-columns: 1fr; row-gap: 18px; }
  .nd-vital { border-left: none; padding-left: 0; }
  .nd-rail { margin: 0 14px 14px; }
  .nd-timer-cells { grid-template-columns: repeat(10, 1fr); }
}

@media (prefers-reduced-motion: reduce) {
  .nd-reveal,
  .nd-eyebrow-dot,
  .nd-tag--pulse::before,
  .nd-spinner > span,
  .nd-dot,
  .nd-opt--shake,
  .nd-arena-score-val--pulse,
  .nd-ring-arc { animation: none !important; transition: none !important; }
}
</style>
