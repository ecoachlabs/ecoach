<script setup lang="ts">
import {
  ref,
  shallowRef,
  markRaw,
  onMounted,
  computed,
  watch,
  nextTick,
  onBeforeUpdate,
  onRenderTriggered,
  onUpdated,
} from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import {
  listSubjects,
  listTopics,
  type SubjectDto,
  type TopicDto,
} from '@/ipc/coach'
import { getReadinessReport, type SubjectReadinessDto } from '@/ipc/readiness'
import {
  getActiveJourneyRoute,
  type JourneyRouteSnapshot,
} from '@/ipc/journey'
import { completeSessionWithPipeline } from '@/ipc/sessions'
import {
  submitAttempt,
  type SubmitAttemptInput,
  type AttemptResultDto,
  type SessionQuestionDto,
} from '@/ipc/questions'
import MathText from '@/components/question/MathText.vue'
import {
  buildLearnerTopicTree,
  type LearnerStrand as Strand,
  type LearnerSubStrand as SubStrand,
  type LearnerTopic as TopicNode,
} from '@/utils/learnerTopics'
import { buildQuestionSolution } from '@/utils/questionSolution'
import { startPracticeSessionWithQuestions } from '@/utils/sessionQuestions'
import { prewarmMathTexts } from '@/utils/mathCache'
import {
  advanceExploreTestBuffer,
  createExploreTestBuffer,
  createExploreTestBufferFromIndex,
  getExploreActiveStage,
  getExploreStandbyStage,
  type ExploreTestBuffer,
  type ExploreTestSlot,
} from '@/utils/exploreTestBuffer'
import {
  perfEnabled,
  perfFlags,
  recordActivePerfPoint,
  startPerfTrace,
} from '@/utils/perfTrace'

const auth = useAuthStore()
const ui = useUiStore()

const PERF_RENDER_TRIGGER_LIMIT = 48
let activeRenderTriggerBudget = 0

function nextAnimationFrame(): Promise<void> {
  return new Promise(resolve => requestAnimationFrame(() => resolve()))
}

function makeSingleStageBuffer(index: number): ExploreTestBuffer<SessionQuestionDto> | null {
  const question = testRawQuestions.value[index]
  if (!question) return null
  return {
    activeSlot: 'a',
    slotA: {
      slot: 'a',
      index,
      question,
    },
    slotB: null,
  }
}

function buildLocalAttemptResult(
  question: SessionQuestionDto,
  selectedOptionId: number,
  wasCorrect: boolean,
): AttemptResultDto {
  const selectedOption = question.options.find(option => option.id === selectedOptionId) ?? null
  const correctOption = question.options.find(option => option.is_correct) ?? null
  const answeredCount = testHistory.value.filter(Boolean).length
  return {
    attempt_id: -Date.now(),
    is_correct: wasCorrect,
    explanation: question.explanation_text ?? null,
    correct_option_text: correctOption?.text ?? null,
    selected_option_text: selectedOption?.text ?? null,
    misconception_info: null,
    error_type: null,
    diagnosis_summary: null,
    recommended_action: null,
    updated_mastery: 0,
    updated_gap: 0,
    session_answered: answeredCount,
    session_remaining: Math.max(0, testRawQuestions.value.length - answeredCount),
    session_complete: false,
    next_action_type: 'LocalOnly',
    next_action_title: 'Local-only perf trace',
    next_action_route: '',
  }
}

const loading = ref(true)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const readiness = ref<SubjectReadinessDto[]>([])
const journeyMap = ref<Record<number, JourneyRouteSnapshot | null>>({})
const searchQuery = ref('')

interface SubjectCfg {
  category: string
  color: string       // category text color
  iconBg: string      // icon box background
  iconColor: string   // icon symbol color
  visualBg: string    // frosted visual area bg
  blobColor: string   // blurred blob color
  symbol: string      // icon symbol
}

const cfgMap: Record<string, SubjectCfg> = {
  MATH: { category: 'MATHEMATICS',   color: '#ea580c', iconBg: 'rgba(234,88,12,0.1)',   iconColor: '#ea580c', visualBg: '#f7f4f0', blobColor: 'rgba(234,88,12,0.25)', symbol: 'M' },
  ENG:  { category: 'ENGLISH',       color: '#2563eb', iconBg: 'rgba(37,99,235,0.1)',   iconColor: '#2563eb', visualBg: '#f0f4f8', blobColor: 'rgba(37,99,235,0.2)',  symbol: 'Aa' },
  SCI:  { category: 'SCIENCE',       color: '#059669', iconBg: 'rgba(5,150,105,0.1)',   iconColor: '#059669', visualBg: '#f0f7f5', blobColor: 'rgba(5,150,105,0.2)',  symbol: 'S' },
  SS:   { category: 'SOCIAL STUDIES',color: '#ea580c', iconBg: 'rgba(234,88,12,0.1)',   iconColor: '#ea580c', visualBg: '#f7f4f0', blobColor: 'rgba(234,88,12,0.2)',  symbol: 'SS' },
  ICT:  { category: 'ICT',           color: '#7c3aed', iconBg: 'rgba(124,58,237,0.1)', iconColor: '#7c3aed', visualBg: '#f3f0f8', blobColor: 'rgba(124,58,237,0.2)', symbol: 'PC' },
  FR:   { category: 'FRENCH',        color: '#0891b2', iconBg: 'rgba(8,145,178,0.1)',   iconColor: '#0891b2', visualBg: '#f0f5f8', blobColor: 'rgba(8,145,178,0.2)',  symbol: 'Fr' },
  TWI:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: 'G' },
  RME:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: 'G' },
  BDT:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: 'G' },
  GA:   { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: 'G' },
}

function cfg(code: string): SubjectCfg {
  return cfgMap[code] ?? {
    category: 'GENERAL', color: '#64748b',
    iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b',
    visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)',
    symbol: 'G',
  }
}

// Inject Nothing Design fonts used by the topics + test views.
onMounted(() => {
  const id = 'nd-jh-fonts'
  if (document.getElementById(id)) return
  const pre1 = document.createElement('link')
  pre1.rel = 'preconnect'; pre1.href = 'https://fonts.googleapis.com'
  const pre2 = document.createElement('link')
  pre2.rel = 'preconnect'; pre2.href = 'https://fonts.gstatic.com'; pre2.crossOrigin = ''
  const css = document.createElement('link')
  css.id = id
  css.rel = 'stylesheet'
  css.href = 'https://fonts.googleapis.com/css2?' +
    'family=Space+Grotesk:wght@300;400;500;600;700&' +
    'family=Space+Mono:wght@400;700&display=swap'
  document.head.appendChild(pre1)
  document.head.appendChild(pre2)
  document.head.appendChild(css)
})

onMounted(async () => {
  if (!auth.currentAccount) return
  loadTestHistoryRecords()
  try {
    const [subs, rdns] = await Promise.all([
      listSubjects(1),
      getReadinessReport(auth.currentAccount.id),
    ])
    subjects.value = subs
    readiness.value = rdns.subjects
    await Promise.all(
      subs.map(async (s) => {
        try {
          journeyMap.value[s.id] = await getActiveJourneyRoute(auth.currentAccount!.id, s.id)
        } catch {
          journeyMap.value[s.id] = null
        }
      })
    )
  } catch {
    error.value = 'Failed to load courses'
  }
  loading.value = false
})

/**
 * Best-effort refetch of readiness + per-subject journey snapshots.
 * Called after a test completes so the Explore grid reflects the new
 * mastery counts instead of staying stuck on whatever mounted initially.
 */
async function refreshExploreStats(): Promise<void> {
  if (!auth.currentAccount) return
  try {
    const rdns = await getReadinessReport(auth.currentAccount.id)
    readiness.value = rdns.subjects
  } catch { /* silent — best effort */ }

  if (subjects.value.length === 0) return
  await Promise.all(
    subjects.value.map(async (s) => {
      try {
        journeyMap.value[s.id] = await getActiveJourneyRoute(auth.currentAccount!.id, s.id)
      } catch {
        journeyMap.value[s.id] = null
      }
    }),
  )
}

// ═══════════════════════════════════════════════════════════════════
// CLIENT-SIDE TEST HISTORY
// The backend tracks session_items by topic but doesn't yet expose a
// "session history" IPC. Until it does, we record completed tests in
// localStorage so the student gets immediate visible feedback ("3 tests
// taken on Mathematics · 71% avg") and a Recent Activity list on the
// courses grid. Cleared naturally when the browser profile is wiped.
// ═══════════════════════════════════════════════════════════════════

interface TestAttemptRecord {
  id: string
  timestamp: number     // ms since epoch
  studentId: number     // to filter across multiple profiles
  subjectId: number
  subjectName: string
  topicId: number
  topicName: string
  topicCode: string | null
  correct: number
  total: number
}

const TEST_HISTORY_KEY = 'ecoach.explore.testHistory.v1'
const TEST_HISTORY_MAX = 60
const testHistoryRecords = ref<TestAttemptRecord[]>([])

function loadTestHistoryRecords(): void {
  try {
    const raw = localStorage.getItem(TEST_HISTORY_KEY)
    if (!raw) return
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) {
      testHistoryRecords.value = parsed
        .filter((r: any) => r && typeof r === 'object' && typeof r.id === 'string')
        .slice(0, TEST_HISTORY_MAX)
    }
  } catch { /* noop */ }
}
function saveTestHistoryRecords(): void {
  try {
    localStorage.setItem(
      TEST_HISTORY_KEY,
      JSON.stringify(testHistoryRecords.value.slice(0, TEST_HISTORY_MAX)),
    )
  } catch { /* noop — quota / private mode */ }
}

function recordCompletedTest(): void {
  if (!auth.currentAccount) return
  const subject = selectedSubject.value
  const topic = selectedTopicNode.value
  if (!subject || !topic) return
  if (testTotal.value === 0) return

  const record: TestAttemptRecord = {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
    timestamp: Date.now(),
    studentId: auth.currentAccount.id,
    subjectId: subject.id,
    subjectName: subject.name,
    topicId: topic.id,
    topicName: topic.name,
    topicCode: topic.code ?? null,
    correct: testCorrectCount.value,
    total: testTotal.value,
  }
  testHistoryRecords.value = [record, ...testHistoryRecords.value].slice(0, TEST_HISTORY_MAX)
  saveTestHistoryRecords()
}

// Per-student filtered history (so switching profiles doesn't leak data)
const myTestHistory = computed<TestAttemptRecord[]>(() => {
  const sid = auth.currentAccount?.id
  if (!sid) return []
  return testHistoryRecords.value.filter(r => r.studentId === sid)
})

interface CourseHistoryStats {
  count: number
  avgPct: number | null
  lastLabel: string | null   // "7 / 10" of most recent attempt
}
function courseHistoryStats(subjectId: number): CourseHistoryStats {
  const rs = myTestHistory.value.filter(r => r.subjectId === subjectId)
  if (rs.length === 0) return { count: 0, avgPct: null, lastLabel: null }
  const correct = rs.reduce((s, r) => s + r.correct, 0)
  const total = rs.reduce((s, r) => s + r.total, 0)
  return {
    count: rs.length,
    avgPct: total > 0 ? Math.round((correct / total) * 100) : null,
    lastLabel: `${rs[0].correct} / ${rs[0].total}`,
  }
}

const recentActivity = computed(() => myTestHistory.value.slice(0, 6))

function formatRelativeTime(ts: number): string {
  const now = Date.now()
  const diff = Math.max(0, now - ts)
  const m = Math.floor(diff / 60_000)
  if (m < 1) return 'just now'
  if (m < 60) return `${m}m ago`
  const h = Math.floor(m / 60)
  if (h < 24) return `${h}h ago`
  const d = Math.floor(h / 24)
  if (d < 7) return `${d}d ago`
  return new Date(ts).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' })
}

// Watch for test completion → record attempt + refresh stats. Registered
// below after `testCompleted` has been declared so TS doesn't complain.

function readinessFor(id: number) {
  return readiness.value.find(r => r.subject_id === id)
}
function completionPct(id: number): number | null {
  const r = readinessFor(id)
  if (!r || r.total_topic_count === 0) return null
  return Math.round((r.mastered_topic_count / r.total_topic_count) * 100)
}
function masteredTopicCount(id: number): number {
  return readinessFor(id)?.mastered_topic_count ?? 0
}
function topicCount(id: number): number {
  return readinessFor(id)?.total_topic_count ?? 0
}
function routeFor(id: number): JourneyRouteSnapshot | null {
  return journeyMap.value[id] ?? null
}
function routeTotalStations(id: number): number {
  return routeFor(id)?.stations.length ?? 0
}
function completedStationCount(id: number): number {
  return routeFor(id)?.stations.filter(station => station.status === 'completed').length ?? 0
}
function currentStationTitle(id: number): string | null {
  const snapshot = routeFor(id)
  if (!snapshot) return null
  const current = snapshot.stations.find(station =>
    station.status === 'active' || station.station_code === snapshot.route.current_station_code,
  )
  return current?.title ?? null
}

const filtered = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return subjects.value
  return subjects.value.filter(s =>
    s.name.toLowerCase().includes(q) || s.code.toLowerCase().includes(q)
  )
})

const routePanelRows = computed(() => {
  return filtered.value.map(subject => {
    const totalStations = routeTotalStations(subject.id)
    const completedStations = completedStationCount(subject.id)
    const currentTitle = currentStationTitle(subject.id)

    return {
      id: subject.id,
      label: subject.name,
      badge: totalStations > 0 ? `${completedStations}/${totalStations}` : '--',
      meta: totalStations > 0
        ? (currentTitle ?? 'Route mapped and waiting')
        : 'Route not built from live learner data yet',
    }
  })
})


// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// INLINE FLOW â€” courses â†’ topics â†’ test, all on /student/journey
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Three explicit viewing zoom levels, as the user asked for:
//   general â†’ full course: strands + sub-strand names visible
//   strand  â†’ one strand drilled in: its sub-strands and all 57-style topics
//   topic   â†’ one specific topic: full outcome text + Start Test
type Mode = 'courses' | 'general' | 'strand' | 'topic' | 'test'
const mode = ref<Mode>('courses')
const selectedSubject = ref<SubjectDto | null>(null)
const subjectTopics = ref<TopicDto[]>([])          // raw â€” every node returned by listTopics
const topicsLoading = ref(false)
const topicsError = ref('')

// Currently focused strand / topic (when drilled in)
const selectedStrandId = ref<number | null>(null)
const selectedTopicId = ref<number | null>(null)

const strands = computed<Strand[]>(() => buildLearnerTopicTree(subjectTopics.value))

// Helpers used by the templates
const selectedStrand = computed(() =>
  strands.value.find(s => s.id === selectedStrandId.value) ?? null,
)
const selectedTopicNode = computed<TopicNode | null>(() => {
  if (!selectedTopicId.value) return null
  for (const s of strands.value) {
    for (const ss of s.subStrands) {
      const found = ss.topics.find(t => t.id === selectedTopicId.value)
      if (found) return found
    }
  }
  return null
})
const selectedTopicContext = computed(() => {
  const t = selectedTopicNode.value
  if (!t) return null
  const strand = strands.value.find(s => s.id === t.strandId) ?? null
  const subStrand = strand?.subStrands.find(ss => ss.id === t.subStrandId) ?? null
  return { strand, subStrand }
})
const selectedStrandIndex = computed<number>(() => {
  const s = selectedStrand.value
  if (!s) return -1
  return strands.value.findIndex(x => x.id === s.id)
})
const strandUnlockAfterSelected = computed<string | null>(() => {
  const i = selectedStrandIndex.value
  if (i < 0) return null
  const next = strands.value[i + 1]
  return next ? next.name : null
})
const totalTopicsInCourse = computed(() =>
  strands.value.reduce((s, x) => s + x.totalTopics, 0),
)

function globalTopicIndex(strand: Strand, subStrand: SubStrand, within: number): number {
  let offset = 0
  for (const ss of strand.subStrands) {
    if (ss.id === subStrand.id) return offset + within
    offset += ss.topics.length
  }
  return within
}

function strandUnlocks(si: number): string | null {
  const next = strands.value[si + 1]
  return next ? next.name : null
}

// Navigation helpers
function openStrand(strand: Strand) {
  selectedStrandId.value = strand.id
  selectedTopicId.value = null
  mode.value = 'strand'
}
function openTopicNode(topic: TopicNode) {
  selectedTopicId.value = topic.id
  selectedStrandId.value = topic.strandId
  mode.value = 'topic'
}
function backToGeneral() {
  selectedStrandId.value = null
  selectedTopicId.value = null
  mode.value = 'general'
}
function backToStrand() {
  selectedTopicId.value = null
  mode.value = 'strand'
}

// Inline test state (parallels useHomepageArena but scoped to a picked topic)
const TEST_QUESTION_COUNT = 10
const testSessionId = ref<number | null>(null)
// `shallowRef`: question data is loaded ONCE at session start and never
// structurally mutated. Making each nested option a reactive Proxy costs
// noticeable main-thread time during render (every `option.text`,
// `option.is_correct` access is a trap) and gives us nothing in return.
// Structural re-assignment (`testRawQuestions.value = [...]`) still
// triggers reactivity; only deep mutations are skipped — which is the
// whole point.
const testRawQuestions = shallowRef<SessionQuestionDto[]>([])
const testBuffer = shallowRef<ExploreTestBuffer<SessionQuestionDto> | null>(null)
const testIndex = ref(0)
const testSelectedOptionId = ref<number | null>(null)
const testLoading = ref(false)
const testError = ref('')
const testCompleted = ref(false)
const testCorrectCount = ref(0)
const testAdvancing = ref(false)
// Per-question history used by the instrument panel's progress dots.
type TestMark = 'correct' | 'wrong' | null
const testHistory = ref<TestMark[]>([])
const questionStartedAt = ref<number>(0)
const testScrollPane = ref<HTMLElement | null>(null)
const testFeedbackCard = ref<HTMLElement | null>(null)
const testPendingSubmissions = new Map<number, Promise<AttemptResultDto>>()
const testUnsavedInputs = new Map<number, SubmitAttemptInput>()
const testAttemptResult = ref<AttemptResultDto | null>(null)

onBeforeUpdate(() => {
  recordActivePerfPoint('journey.view.before-update')
})

onUpdated(() => {
  recordActivePerfPoint('journey.view.updated')
})

onRenderTriggered((event) => {
  if (!perfEnabled || activeRenderTriggerBudget <= 0) return
  activeRenderTriggerBudget -= 1
  recordActivePerfPoint('journey.render.triggered', {
    type: event.type,
    key: String(event.key),
    targetType: event.target?.constructor?.name ?? typeof event.target,
  })
})

// Persist the completed test AND refresh Explore stats the moment
// `testCompleted` flips true. Declared here so `testCompleted` is in scope.
watch(testCompleted, (val) => {
  if (!val) return
  recordCompletedTest()
  void refreshExploreStats()
})

interface ExplanationStep {
  label: string
  detail: string
}

interface ParsedExplanation {
  commentary: string
  steps: ExplanationStep[]
  hasAny: boolean
}

const currentTestStage = computed(() =>
  testBuffer.value ? getExploreActiveStage(testBuffer.value) : null,
)
const standbyTestStage = computed(() =>
  testBuffer.value ? getExploreStandbyStage(testBuffer.value) : null,
)
const currentTestQuestion = computed(() => currentTestStage.value?.question ?? null)
const testRenderableStages = computed(() => {
  const buffer = testBuffer.value
  if (!buffer) return []
  return [
    { slot: 'a' as const, stage: buffer.slotA, isActive: buffer.activeSlot === 'a' },
    { slot: 'b' as const, stage: buffer.slotB, isActive: buffer.activeSlot === 'b' },
  ]
})
const testSelectedOption = computed(() =>
  currentTestQuestion.value?.options.find(option => option.id === testSelectedOptionId.value) ?? null,
)
const testCorrectOption = computed(() =>
  currentTestQuestion.value?.options.find(option => option.is_correct) ?? null,
)
const testCorrectOptionId = computed(() => testCorrectOption.value?.id ?? null)
const testPendingCorrect = computed<boolean | null>(() => {
  if (testSelectedOptionId.value === null) return null
  return testSelectedOption.value?.is_correct ?? null
})
const testLocked = computed(() => testSelectedOptionId.value !== null)
const testTotal = computed(() => testRawQuestions.value.length)

const parsedExplanation = computed<ParsedExplanation>(() => {
  const question = currentTestQuestion.value
  if (!question || testSelectedOptionId.value === null) {
    return { commentary: '', steps: [], hasAny: false }
  }

  if (perfFlags.minimalRender) {
    const commentary = testAttemptResult.value?.explanation ?? question.explanation_text ?? ''
    return {
      commentary,
      steps: [],
      hasAny: commentary.trim().length > 0,
    }
  }

  const solution = buildQuestionSolution(
    question.stem,
    testAttemptResult.value?.explanation ?? question.explanation_text ?? null,
    testCorrectOption.value?.text ?? null,
  )
  const commentary = solution.intro
    || (testPendingCorrect.value
      ? 'Nice work - here is the reasoning behind the answer.'
      : 'Here is how to arrive at the correct answer.')

  const steps = solution.steps
    .map(step => ({
      label: step.note,
      detail: step.work || '',
    }))
    .filter(step => step.label.trim().length > 0 || step.detail.trim().length > 0)

  return {
    commentary,
    steps,
    hasAny: commentary.trim().length > 0 || steps.length > 0,
  }
})

watch(currentTestQuestion, (nextQuestion, previousQuestion) => {
  recordActivePerfPoint('journey.current-question.changed', {
    fromItemId: previousQuestion?.item_id ?? null,
    toItemId: nextQuestion?.item_id ?? null,
  })
})

watch(parsedExplanation, (value) => {
  recordActivePerfPoint('journey.parsed-explanation.changed', {
    commentaryChars: value.commentary.length,
    steps: value.steps.length,
    hasAny: value.hasAny,
  })
})

watch(testRenderableStages, (stages) => {
  recordActivePerfPoint('journey.renderable-stages.changed', {
    activeIndex: stages.find(stage => stage.isActive)?.stage?.index ?? null,
    standbyIndex: stages.find(stage => !stage.isActive)?.stage?.index ?? null,
  })
})

// ═══════════════════════════════════════════════════════════════════
// EXPLANATION PARSER
// Backend explanations arrive as one run-on paragraph with embedded
// "Label: detail." structure. We split into:
//   · commentary — a short contextual lead-in (synthesized)
//   · steps      — ordered list of { label, detail } pulled from prose
// ═══════════════════════════════════════════════════════════════════
function resetTest() {
  testPendingSubmissions.clear()
  testUnsavedInputs.clear()
  testAttemptResult.value = null
  testSessionId.value = null
  testRawQuestions.value = []
  testBuffer.value = null
  testIndex.value = 0
  testSelectedOptionId.value = null
  testLoading.value = false
  testError.value = ''
  testCompleted.value = false
  testCorrectCount.value = 0
  testAdvancing.value = false
  testHistory.value = []
}

async function flushTestSubmissions(): Promise<boolean> {
  const inFlight = Array.from(testPendingSubmissions.values())
  if (inFlight.length > 0) {
    await Promise.allSettled(inFlight)
  }

  if (testUnsavedInputs.size === 0) return true

  let failureMessage = 'Some answers could not be saved yet. Please try again.'
  for (const [itemId, input] of Array.from(testUnsavedInputs.entries())) {
    try {
      await submitAttempt(input)
      testUnsavedInputs.delete(itemId)
    } catch (e: any) {
      failureMessage = typeof e === 'string' ? e : e?.message ?? failureMessage
    }
  }

  if (testUnsavedInputs.size > 0) {
    testError.value = failureMessage
    return false
  }

  return true
}

function syncTestBufferFromIndex(index: number, activeSlot: ExploreTestSlot = 'a') {
  if (perfFlags.disableBuffer) {
    testBuffer.value = makeSingleStageBuffer(index)
    testIndex.value = index
    return
  }
  testBuffer.value = createExploreTestBufferFromIndex(testRawQuestions.value, index, activeSlot)
  testIndex.value = getExploreActiveStage(testBuffer.value)?.index ?? index
}

function ensureStandbyTestStage() {
  if (perfFlags.disableBuffer) return
  const buffer = testBuffer.value
  const nextIndex = testIndex.value + 1
  if (nextIndex >= testRawQuestions.value.length) return
  if (buffer) {
    const standby = getExploreStandbyStage(buffer)
    if (standby?.index === nextIndex) return
    syncTestBufferFromIndex(testIndex.value, buffer.activeSlot)
    return
  }
  syncTestBufferFromIndex(testIndex.value, 'a')
}

function advanceVisibleTestStage(): boolean {
  if (perfFlags.disableBuffer) return false
  const buffer = testBuffer.value
  if (!buffer) return false

  const standby = getExploreStandbyStage(buffer)
  if (!standby) return false

  testBuffer.value = advanceExploreTestBuffer(buffer, testRawQuestions.value)
  testIndex.value = getExploreActiveStage(testBuffer.value)?.index ?? standby.index
  return true
}

async function openCourse(subject: SubjectDto) {
  selectedSubject.value = subject
  mode.value = 'general'
  topicsError.value = ''
  subjectTopics.value = []
  selectedStrandId.value = null
  selectedTopicId.value = null
  topicsLoading.value = true
  try {
    subjectTopics.value = await listTopics(subject.id)
  } catch (e: any) {
    topicsError.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load topics'
  } finally {
    topicsLoading.value = false
  }
}

function backToCourses() {
  resetTest()
  selectedSubject.value = null
  selectedStrandId.value = null
  selectedTopicId.value = null
  subjectTopics.value = []
  mode.value = 'courses'
}

async function startTopicTest(topic: TopicNode) {
  if (!auth.currentAccount || !selectedSubject.value) return
  selectedTopicId.value = topic.id
  selectedStrandId.value = topic.strandId
  resetTest()
  mode.value = 'test'
  testLoading.value = true
  try {
    const fallbackTopicSets = topic.sourceTopicIds.length > 1
      ? [[topic.sourceTopicIds[0]]]
      : []
    const session = await startPracticeSessionWithQuestions({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubject.value.id,
      topic_ids: topic.sourceTopicIds,
      question_count: TEST_QUESTION_COUNT,
      is_timed: false,
    }, fallbackTopicSets)
    testSessionId.value = session.sessionId
    // `markRaw` each question so Vue never wraps its nested options/fields
    // in a reactive Proxy. Render-time property access (stem, options[i].text,
    // is_correct, etc.) becomes plain JS reads instead of Proxy traps.
    testRawQuestions.value = session.questions.map(q => markRaw(q))
    if (perfFlags.disableBuffer) {
      testBuffer.value = makeSingleStageBuffer(0)
    } else {
      testBuffer.value = createExploreTestBuffer(testRawQuestions.value)
    }
    testHistory.value = Array.from({ length: testRawQuestions.value.length }, () => null)
    testIndex.value = testBuffer.value ? getExploreActiveStage(testBuffer.value)?.index ?? 0 : 0
    testSelectedOptionId.value = null
    questionStartedAt.value = performance.now()
    // Warm the KaTeX cache for EVERY question's stem, options, and
    // explanation in idle time. By the time the user clicks NEXT, the
    // next question's math has already been parsed — the swap is a
    // pure DOM update with no KaTeX work on the critical path.
    const mathTexts: string[] = []
    for (const q of testRawQuestions.value) {
      if (q.stem) mathTexts.push(q.stem)
      if (q.explanation_text) mathTexts.push(q.explanation_text)
      for (const o of q.options) {
        if (o.text) mathTexts.push(o.text)
      }
    }
    prewarmMathTexts(mathTexts)
  } catch (e: any) {
    testError.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start the test'
  } finally {
    testLoading.value = false
  }
}

async function submitTestAnswer(optionId: number) {
  if (testLocked.value) return
  const question = currentTestQuestion.value
  if (!question || !testSessionId.value || !auth.currentAccount) return
  const trace = startPerfTrace('journey.submit', {
    index: testIndex.value,
    itemId: question.item_id,
    questionId: question.question_id,
    optionId,
    localOnly: perfFlags.localOnly,
    noPersist: perfFlags.noPersist,
  })
  trace.point('click-start')

  const selectedOption = question.options.find(option => option.id === optionId) ?? null
  const wasCorrect = !!selectedOption?.is_correct
  trace.measureSync('local-feedback.state-update', () => {
    testSelectedOptionId.value = optionId
    testAttemptResult.value = null
    testError.value = ''
    if (wasCorrect) testCorrectCount.value += 1
    if (testHistory.value.length <= testIndex.value) {
      testHistory.value.length = testIndex.value + 1
    }
    testHistory.value[testIndex.value] = wasCorrect ? 'correct' : 'wrong'
  })

  const input: SubmitAttemptInput = {
    student_id: auth.currentAccount.id,
    session_id: testSessionId.value,
    session_item_id: question.item_id,
    question_id: question.question_id,
    selected_option_id: optionId,
    response_time_ms: Math.max(0, Math.round(performance.now() - questionStartedAt.value)),
    confidence_level: null,
    hint_count: 0,
    changed_answer_count: 0,
    was_timed: false,
    defer_coach_brain: true,
  }

  trace.measureSync('buffer.refill', () => {
    ensureStandbyTestStage()
  })

  if (perfFlags.localOnly || perfFlags.noPersist) {
    testAttemptResult.value = buildLocalAttemptResult(question, optionId, wasCorrect)
    trace.point('submission.skipped', {
      localOnly: perfFlags.localOnly,
      noPersist: perfFlags.noPersist,
    })
    trace.finish({
      isCorrect: wasCorrect,
      pendingSubmissions: testPendingSubmissions.size,
    })
    return
  }

  testUnsavedInputs.set(question.item_id, input)
  const submission = submitAttempt(input)
  testPendingSubmissions.set(question.item_id, submission)
  void trace.measureAsync('submission.await', async () => submission, {
    pendingSubmissions: testPendingSubmissions.size,
  })
    .then((result) => {
      if (currentTestQuestion.value?.item_id === question.item_id) {
        testAttemptResult.value = result
      }
      testUnsavedInputs.delete(question.item_id)
      trace.point('submission.resolved', {
        isCorrect: result.is_correct,
        answerId: result.attempt_id,
      })
    })
    .catch((submissionError: unknown) => {
      trace.point('submission.rejected', {
        message: submissionError instanceof Error ? submissionError.message : String(submissionError),
      })
      // Keep the unsaved input for the end-of-session flush.
    })
    .finally(() => {
      testPendingSubmissions.delete(question.item_id)
      trace.finish({
        pendingSubmissions: testPendingSubmissions.size,
      })
    })
}

async function nextTestQuestion() {
  // Gate EVERY path on testAdvancing, not just the atEnd branch. This
  // prevents rapid double-clicks from re-entering while the first call
  // is still running its (possibly async) state transitions.
  if (testSelectedOptionId.value === null || testAdvancing.value) return
  testAdvancing.value = true
  const trace = startPerfTrace('journey.next', {
    index: testIndex.value,
    itemId: currentTestQuestion.value?.item_id ?? null,
    pendingSubmissions: testPendingSubmissions.size,
    bufferDisabled: perfFlags.disableBuffer,
  })
  trace.activate()
  activeRenderTriggerBudget = PERF_RENDER_TRIGGER_LIMIT
  try {
    trace.point('click-start')
    const atEnd = testIndex.value >= testRawQuestions.value.length - 1
    if (atEnd) {
      if (!(await trace.measureAsync('flush-test-submissions', async () => flushTestSubmissions()))) return
      if (testSessionId.value) {
        try {
          await trace.measureAsync('complete-session', async () =>
            completeSessionWithPipeline(auth.currentAccount!.id, testSessionId.value!),
          )
        } catch (error: any) {
          testError.value = typeof error === 'string'
            ? error
            : error?.message ?? 'Your answers were saved, but session completion failed. Tap Next to retry.'
          return
        }
      }
      testCompleted.value = true
      trace.point('test-completed')
      return
    }

    trace.measureSync('explanation.reset', () => {
      testSelectedOptionId.value = null
      testAttemptResult.value = null
      testError.value = ''
    })
    const usedStandby = trace.measureSync('next-question.swap', () => advanceVisibleTestStage())
    if (!usedStandby) {
      trace.measureSync('next-question.sync-from-index', () => {
        syncTestBufferFromIndex(testIndex.value + 1, testBuffer.value?.activeSlot ?? 'a')
      })
    }
    questionStartedAt.value = performance.now()
    trace.measureSync('buffer.refill', () => {
      ensureStandbyTestStage()
    })
    await nextTick()
    trace.point('vue.nextTick.complete', {
      currentItemId: currentTestQuestion.value?.item_id ?? null,
      currentIndex: testIndex.value,
    })
    await nextAnimationFrame()
    await nextAnimationFrame()
    trace.point('paint.complete', {
      currentItemId: currentTestQuestion.value?.item_id ?? null,
      currentIndex: testIndex.value,
      pendingSubmissions: testPendingSubmissions.size,
    })
  } finally {
    activeRenderTriggerBudget = 0
    trace.deactivate()
    trace.finish({
      currentItemId: currentTestQuestion.value?.item_id ?? null,
      currentIndex: testIndex.value,
    })
    testAdvancing.value = false
  }
}

async function backFromTest() {
  if (!(await flushTestSubmissions())) return
  resetTest()
  // Pull fresh readiness/journey so any just-recorded progress shows.
  void refreshExploreStats()
  // Return to the topic focus the user started from, if available,
  // otherwise to the enclosing strand, otherwise to general.
  if (selectedTopicId.value) mode.value = 'topic'
  else if (selectedStrandId.value) mode.value = 'strand'
  else mode.value = 'general'
}

async function retakeTopicTest() {
  const t = selectedTopicNode.value
  if (!t) return
  if (!(await flushTestSubmissions())) return
  await startTopicTest(t)
}

function isPickedOption(optionId: number): boolean {
  return testSelectedOptionId.value === optionId
}
function isCorrectOption(optionId: number): boolean {
  if (testSelectedOptionId.value === null) return false
  return testCorrectOptionId.value === optionId
}
function isWrongPickedOption(optionId: number): boolean {
  return testSelectedOptionId.value === optionId && testPendingCorrect.value === false
}

function pickTestOptionFromStage(isActive: boolean, optionId: number) {
  if (!isActive) return
  void submitTestAnswer(optionId)
}

// If auth changes mid-session, reset the flow
watch(() => auth.currentAccount?.id, () => {
  backToCourses()
})

// Deliberately NOT scrolling on answer — the new 2-column layout keeps
// the feedback visible in the right panel without scrolling the viewport.
</script>

<template>
  <div
    class="journey-home h-full flex flex-col overflow-hidden"
    :class="{ 'journey-home--dark': ui.isDark }"
    :style="{ backgroundColor: 'var(--paper)' }"
  >

    <!-- Header â€” adapts to current mode (courses / topics / test) -->
    <div
      class="page-header"
      :style="{ backgroundColor: 'var(--paper)' }"
    >
      <!-- Breadcrumb (scoped to current zoom level) -->
      <div v-if="mode !== 'courses'" class="flex items-center gap-2 mb-2 flex-wrap">
        <button class="crumb-btn" @click="backToCourses">
          <span aria-hidden="true">Back</span>
          <span>All Courses</span>
        </button>
        <span v-if="selectedSubject" class="crumb-sep">/</span>
        <!-- Course link -->
        <button
          v-if="selectedSubject && mode !== 'general'"
          class="crumb-btn"
          @click="backToGeneral"
        >{{ selectedSubject.name }}</button>
        <span v-else-if="selectedSubject" class="crumb-current">
          {{ selectedSubject.name }}
        </span>
        <!-- Strand link (when in topic or test) -->
        <template v-if="selectedStrand && (mode === 'topic' || mode === 'test')">
          <span class="crumb-sep">/</span>
          <button
            v-if="mode === 'test'"
            class="crumb-btn"
            @click="backToStrand"
          >{{ selectedStrand.name }}</button>
          <span v-else class="crumb-current">{{ selectedStrand.name }}</span>
        </template>
        <!-- Topic name (when in test) -->
        <template v-if="mode === 'test' && selectedTopicNode">
          <span class="crumb-sep">/</span>
          <span class="crumb-current">{{ selectedTopicNode.name }}</span>
        </template>
      </div>

      <h1 class="page-title">
        <template v-if="mode === 'courses'">Explore Courses</template>
        <template v-else-if="mode === 'general'">{{ selectedSubject?.name ?? 'Course' }}</template>
        <template v-else-if="mode === 'strand'">{{ selectedStrand?.name ?? 'Strand' }}</template>
        <template v-else-if="mode === 'topic'">{{ selectedTopicNode?.name ?? 'Topic' }}</template>
        <template v-else-if="mode === 'test'">
          <span class="topic-code-badge">{{ selectedTopicNode?.code ?? 'Test' }}</span>
        </template>
        <template v-else>Test</template>
      </h1>
      <p class="page-sub">
        <template v-if="mode === 'courses'">
          {{ subjects.length }} courses in Junior High JHS 1
        </template>
        <template v-else-if="mode === 'general'">
          {{ strands.length }} {{ strands.length === 1 ? 'strand' : 'strands' }}
          - {{ totalTopicsInCourse }} topics - pick any strand to drill in
        </template>
        <template v-else-if="mode === 'strand' && selectedStrand">
          {{ selectedStrand.subStrands.length }}
          {{ selectedStrand.subStrands.length === 1 ? 'sub-strand' : 'sub-strands' }}
          - {{ selectedStrand.totalTopics }} topics
        </template>
        <template v-else-if="mode === 'topic' && selectedTopicContext">
          {{ selectedTopicContext.strand?.name }}
          <template v-if="selectedTopicContext.subStrand"> - {{ selectedTopicContext.subStrand.name }}</template>
        </template>
        <template v-else-if="mode === 'test' && testCompleted">
          Test complete · {{ testCorrectCount }}/{{ testTotal }} correct
        </template>
        <template v-else-if="mode === 'test' && testLoading">
          Preparing test…
        </template>
        <template v-else-if="mode === 'test'">
          <span class="test-head-dots" :aria-label="`Question ${testIndex + 1} of ${testTotal}`">
            <span
              v-for="i in testTotal" :key="i"
              class="test-head-dot"
              :class="{
                'test-head-dot--done':    !!testHistory[i - 1],
                'test-head-dot--current': i - 1 === testIndex && !testHistory[testIndex],
              }"
            />
          </span>
          <span class="test-head-count">QUESTION {{ testIndex + 1 }} / {{ testTotal }}</span>
        </template>
      </p>
    </div>

    <div v-if="error" class="page-error">
      <span class="page-error-tag">[ ERROR ]</span>
      <span class="page-error-msg">{{ error }}</span>
    </div>

    <!-- Body: scrollable content + floating levels panel -->
    <div class="flex-1 overflow-hidden relative">

      <!-- COURSES MODE â€” course grid + search (Nothing) -->
      <div
        v-if="mode === 'courses'"
        class="h-full overflow-y-auto topic-scroll courses-body"
      >
        <!-- Search â€” flat underline, no box -->
        <div class="relative search-wrap">
          <svg class="search-icon" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="9" cy="9" r="6" stroke="currentColor" stroke-width="1.5"/>
            <path d="M13.5 13.5L17 17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search courses"
            class="search-input"
          />
          <span class="search-count">{{ filtered.length }} / {{ subjects.length }}</span>
        </div>

        <!-- Skeleton â€” shimmer cards matching the real shape -->
        <div v-if="loading" class="courses-grid">
          <div v-for="i in 8" :key="i" class="course-skel">
            <div class="course-skel-top" />
            <div class="course-skel-name" />
            <div class="course-skel-stats" />
            <div class="course-skel-bar" />
          </div>
        </div>

        <!-- Course grid â€” flat monochrome cards -->
        <div v-else class="courses-grid">
          <button
            v-for="subject in filtered"
            :key="subject.id"
            class="course-card"
            @click="openCourse(subject)"
          >
            <div class="cc-top">
              <span class="cc-cat">{{ cfg(subject.code).category }}<template v-if="subject.code"> - {{ subject.code }}</template></span>
              <span class="cc-open-arrow" aria-hidden="true">-></span>
            </div>

            <h3 class="cc-name">{{ subject.name }}</h3>

            <p class="cc-stats">
              <span>{{ topicCount(subject.id) || '-' }} {{ (topicCount(subject.id) || 0) === 1 ? 'topic' : 'topics' }}</span>
              <span class="cc-sep">·</span>
              <span>{{ masteredTopicCount(subject.id) || 0 }} mastered</span>
              <template v-if="courseHistoryStats(subject.id).count > 0">
                <span class="cc-sep">·</span>
                <span>
                  {{ courseHistoryStats(subject.id).count }}
                  {{ courseHistoryStats(subject.id).count === 1 ? 'test' : 'tests' }} taken
                </span>
                <template v-if="courseHistoryStats(subject.id).avgPct !== null">
                  <span class="cc-sep">·</span>
                  <span>{{ courseHistoryStats(subject.id).avgPct }}% avg</span>
                </template>
              </template>
              <template v-if="routeTotalStations(subject.id) > 0">
                <span class="cc-sep">·</span>
                <span>{{ completedStationCount(subject.id) }}/{{ routeTotalStations(subject.id) }} stations</span>
              </template>
            </p>

            <!-- Footer: completion % + hairline progress track -->
            <div class="cc-foot">
              <template v-if="completionPct(subject.id) !== null">
                <span class="cc-pct">
                  <span class="cc-pct-num">{{ completionPct(subject.id) }}</span>
                  <span class="cc-pct-u">%</span>
                </span>
                <span class="cc-state">mastered</span>
              </template>
              <template v-else>
                <span class="cc-pct cc-pct--new">
                  <span class="cc-pct-num">-</span>
                  <span class="cc-pct-u">%</span>
                </span>
                <span class="cc-state">not started</span>
              </template>
            </div>

            <!-- Progress hairline (bottom edge of card) -->
            <div class="cc-progress">
              <div
                class="cc-progress-fill"
                :style="{ width: (completionPct(subject.id) ?? 0) + '%' }"
              />
            </div>
          </button>
        </div>

        <div v-if="!loading && filtered.length === 0" class="courses-empty">
          <p class="topic-empty-tag">[ NO COURSES ]</p>
          <p class="topic-empty-sub">No courses match "{{ searchQuery }}".</p>
        </div>

        <!-- ─── Recent activity ─── a local record of completed tests so
             the student sees immediate feedback even before backend
             mastery recomputes propagate. -->
        <section v-if="!loading && recentActivity.length > 0" class="recent-section">
          <header class="recent-head">
            <p class="recent-eyebrow">Recent activity</p>
            <p class="recent-sub">
              {{ myTestHistory.length }}
              {{ myTestHistory.length === 1 ? 'test' : 'tests' }} taken on this device
            </p>
          </header>
          <ol class="recent-list">
            <li
              v-for="r in recentActivity" :key="r.id"
              class="recent-row"
              :class="{
                'recent-row--ok':   r.total > 0 && r.correct / r.total >= 0.7,
                'recent-row--mid':  r.total > 0 && r.correct / r.total >= 0.4 && r.correct / r.total < 0.7,
                'recent-row--err':  r.total > 0 && r.correct / r.total < 0.4,
              }"
            >
              <span class="recent-time">{{ formatRelativeTime(r.timestamp) }}</span>
              <div class="recent-body">
                <p class="recent-topic">{{ r.topicName }}</p>
                <p class="recent-meta">
                  {{ r.subjectName }}
                  <template v-if="r.topicCode"> · {{ r.topicCode }}</template>
                </p>
              </div>
              <span class="recent-score">
                <span class="recent-score-num">{{ r.correct }}</span>
                <span class="recent-score-sep">/</span>
                <span class="recent-score-total">{{ r.total }}</span>
              </span>
            </li>
          </ol>
        </section>
      </div>

      <!-- â”€â”€â”€ GENERAL VIEW â”€â”€â”€ course overview: all strands, each card
           lists its sub-strand names (the clean short names). Clicking
           anywhere on a card drills into STRAND view. -->
      <div
        v-else-if="mode === 'general'"
        class="h-full overflow-y-auto topic-scroll"
      >
        <div class="topic-inner">
          <div v-if="topicsLoading" class="strand-grid">
            <div v-for="i in 4" :key="i" class="strand-skel">
              <div class="strand-skel-head" />
              <div class="strand-skel-path" />
              <div class="strand-skel-row" />
              <div class="strand-skel-row" />
              <div class="strand-skel-row" />
            </div>
          </div>

          <div v-else-if="topicsError" class="topic-empty">
            <p class="topic-empty-tag">[ ERROR ]</p>
            <p class="topic-empty-sub">{{ topicsError }}</p>
          </div>

          <div v-else-if="strands.length === 0" class="topic-empty">
            <p class="topic-empty-tag">[ NO TOPICS ]</p>
            <p class="topic-empty-sub">This course has no topics published yet.</p>
          </div>

          <div v-else class="strand-grid">
            <article
              v-for="(strand, si) in strands" :key="strand.id"
              class="strand-card strand-card--clickable"
              :style="{ animationDelay: (si * 80) + 'ms' }"
              role="button"
              tabindex="0"
              @click="openStrand(strand)"
              @keydown.enter.prevent="openStrand(strand)"
              @keydown.space.prevent="openStrand(strand)"
            >
              <header class="s-head">
                <span class="s-num">{{ String(si + 1).padStart(2, '0') }}</span>
                <div class="s-titles">
                  <h3 class="s-name">{{ strand.name }}</h3>
                  <p class="s-meta">
                    <span v-if="strand.code" class="s-code">{{ strand.code }}</span>
                    <span v-if="strand.code" class="s-sep">-</span>
                    <span>{{ strand.subStrands.length }} {{ strand.subStrands.length === 1 ? 'sub-strand' : 'sub-strands' }}</span>
                    <span class="s-sep">-</span>
                    <span>{{ strand.totalTopics }} topics</span>
                  </p>
                </div>
              </header>

              <!-- Sub-strand names as a compact list â€” these are the
                   short, readable labels the user wants to scan -->
              <ul class="substrand-list">
                <li
                  v-for="(ss, ssi) in strand.subStrands" :key="ss.id"
                  class="substrand-row"
                >
                  <span class="ss-n">{{ String(ssi + 1).padStart(2, '0') }}</span>
                  <span class="ss-name">{{ ss.name }}</span>
                  <span class="ss-count">{{ ss.topics.length }}</span>
                </li>
              </ul>

              <footer class="s-card-foot">
                <span class="open-strand">
                  Open strand
                  <span class="open-strand-arrow" aria-hidden="true">-></span>
                </span>
                <span v-if="strandUnlocks(si)" class="unlock-hint">
                  UNLOCKS - {{ strandUnlocks(si) }}
                </span>
                <span v-else class="unlock-hint unlock-hint--final">
                  COMPLETES - {{ selectedSubject?.name }}
                </span>
              </footer>
            </article>
          </div>
        </div>
      </div>

      <!-- â”€â”€â”€ STRAND VIEW â”€â”€â”€ one strand: sub-strands as sections,
           every topic listed (B7.x.x.x + full outcome text). -->
      <div
        v-else-if="mode === 'strand'"
        class="h-full overflow-y-auto topic-scroll"
      >
        <div class="topic-inner topic-inner--narrow">
          <div v-if="!selectedStrand" class="topic-empty">
            <p class="topic-empty-tag">[ STRAND NOT FOUND ]</p>
          </div>

          <div v-else class="strand-detail">
            <!-- Sticky strand header plate -->
            <div class="strand-plate">
              <div class="plate-left">
                <span class="plate-num">{{ String(selectedStrandIndex + 1).padStart(2, '0') }}</span>
                <div>
                  <h2 class="plate-name">{{ selectedStrand?.name }}</h2>
                  <p class="plate-meta">
                    <span v-if="selectedStrand?.code" class="plate-code">{{ selectedStrand.code }}</span>
                    <span v-if="selectedStrand?.code" class="s-sep">-</span>
                    <span>{{ selectedStrand?.totalTopics }} topics</span>
                  </p>
                </div>
              </div>
              <div class="plate-right">
                <span v-if="strandUnlockAfterSelected" class="unlock-hint">
                  UNLOCKS - {{ strandUnlockAfterSelected }}
                </span>
                <span v-else class="unlock-hint unlock-hint--final">
                  COMPLETES - {{ selectedSubject?.name }}
                </span>
              </div>
            </div>

            <!-- Each sub-strand = a section; topics listed under it -->
            <section
              v-for="(ss, ssi) in selectedStrand?.subStrands ?? []" :key="ss.id"
              class="ss-section"
            >
              <header class="ss-section-head">
                <span class="ss-section-n">{{ String(ssi + 1).padStart(2, '0') }}</span>
                <div>
                  <h3 class="ss-section-name">{{ ss.name }}</h3>
                  <p class="ss-section-meta">
                    <span v-if="ss.code" class="ss-section-code">{{ ss.code }}</span>
                    <span v-if="ss.code" class="s-sep">-</span>
                    <span>{{ ss.topics.length }} topics</span>
                  </p>
                </div>
              </header>

              <ol class="ss-topics">
                <li
                  v-for="(t, i) in ss.topics" :key="t.id"
                  class="ss-topic"
                  tabindex="0"
                  role="button"
                  @click="openTopicNode(t)"
                  @keydown.enter.prevent="openTopicNode(t)"
                  @keydown.space.prevent="openTopicNode(t)"
                >
                  <span class="sst-n">{{ String((selectedStrand ? globalTopicIndex(selectedStrand, ss, i) : i) + 1).padStart(2, '0') }}</span>
                  <div class="sst-body">
                    <p class="sst-name">{{ t.name }}</p>
                    <p v-if="t.code" class="sst-code">{{ t.code }}</p>
                  </div>
                  <span class="sst-arrow" aria-hidden="true">-></span>
                </li>
              </ol>
            </section>
          </div>
        </div>
      </div>

      <!-- â”€â”€â”€ TOPIC VIEW â”€â”€â”€ one topic detail with Start Test CTA -->
      <div
        v-else-if="mode === 'topic'"
        class="h-full overflow-y-auto topic-scroll"
      >
        <div class="topic-inner topic-inner--narrow">
          <div v-if="!selectedTopicNode" class="topic-empty">
            <p class="topic-empty-tag">[ TOPIC NOT FOUND ]</p>
          </div>

          <article v-else class="topic-detail">
            <p class="td-kicker">
              <span v-if="selectedTopicContext?.strand" class="td-kicker-strand">
                {{ selectedTopicContext.strand.name }}
              </span>
              <span v-if="selectedTopicContext?.subStrand" class="td-kicker-sep">/</span>
              <span v-if="selectedTopicContext?.subStrand" class="td-kicker-ss">
                {{ selectedTopicContext.subStrand.name }}
              </span>
            </p>

            <p class="td-statement">{{ selectedTopicNode.name }}</p>
            <div v-if="selectedTopicNode.goalDescriptions.length" class="td-goal-block">
              <p class="td-goal-label">
                {{ selectedTopicNode.goalDescriptions.length === 1 ? 'Goal / Description' : 'Goals / Description' }}
              </p>
              <ol v-if="selectedTopicNode.goalDescriptions.length > 1" class="td-goal-list">
                <li
                  v-for="(goal, index) in selectedTopicNode.goalDescriptions"
                  :key="`${selectedTopicNode.id}-${index}`"
                  class="td-goal-item"
                >
                  <span class="td-goal-step">{{ String(index + 1).padStart(2, '0') }}</span>
                  <span class="td-goal">{{ goal }}</span>
                </li>
              </ol>
              <p v-else class="td-goal">{{ selectedTopicNode.goalDescriptions[0] }}</p>
            </div>

            <div class="td-actions">
              <button class="btn-primary btn-lg" @click="startTopicTest(selectedTopicNode)">
                Start Test <span aria-hidden="true">-></span>
              </button>
              <button class="btn-secondary" @click="backToStrand">
                Back to strand
              </button>
            </div>

            <!-- Nearby topics in the same sub-strand for quick nav -->
            <aside
              v-if="selectedTopicContext?.subStrand && selectedTopicContext.subStrand.topics.length > 1"
              class="td-nearby"
            >
              <p class="td-nearby-head">OTHER TOPICS IN {{ selectedTopicContext.subStrand.name.toUpperCase() }}</p>
              <ol class="td-nearby-list">
                <li
                  v-for="t in selectedTopicContext.subStrand.topics.filter(x => x.id !== selectedTopicNode!.id)"
                  :key="t.id"
                  class="td-nearby-item"
                  tabindex="0"
                  role="button"
                  @click="openTopicNode(t)"
                  @keydown.enter.prevent="openTopicNode(t)"
                  @keydown.space.prevent="openTopicNode(t)"
                >
                  <span class="td-nearby-name">{{ t.name }}</span>
                </li>
              </ol>
            </aside>
          </article>
        </div>
      </div>

      <!-- TEST MODE â€” two-column no-scroll layout (Nothing) -->
      <div
        v-else-if="mode === 'test'"
        ref="testScrollPane"
        class="test-view"
      >
        <!-- Loading -->
        <div v-if="testLoading" class="test-status test-status--loading">
          <div class="test-spinner" aria-hidden="true">
            <span /><span /><span /><span /><span /><span />
          </div>
          <p class="test-status-tag">Preparing your test</p>
          <p v-if="selectedTopicNode?.name" class="test-status-msg">
            {{ selectedTopicNode.name }}
          </p>
          <p class="test-status-hint">
            Selecting {{ TEST_QUESTION_COUNT }} questions
            <span class="test-dots" aria-hidden="true">
              <span>.</span><span>.</span><span>.</span>
            </span>
          </p>
        </div>

        <!-- Error (fatal - no session / no questions) -->
        <div v-else-if="testError && testRawQuestions.length === 0" class="test-status">
          <p class="test-status-tag test-status-tag--err">[ ERROR ]</p>
          <p class="test-status-msg">{{ testError }}</p>
          <button class="btn-secondary" @click="backFromTest">Back</button>
        </div>

        <!-- Completion screen -->
        <div v-else-if="testCompleted" class="test-done">
          <p class="test-done-kicker">TEST COMPLETE</p>
          <p class="test-done-score">
            <span class="test-done-num">{{ testCorrectCount }}</span>
            <span class="test-done-div">/</span>
            <span class="test-done-total">{{ testTotal }}</span>
          </p>
          <p class="test-done-sub">{{ selectedTopicNode?.name }}</p>
          <div class="test-done-actions">
            <button class="btn-secondary" @click="backFromTest">Back to topic</button>
            <button class="btn-primary" @click="retakeTopicTest">Retake test</button>
          </div>
        </div>

        <!-- Active test — two-column layout per the sketch.
             LEFT:  question card (top) + options card (below)
             RIGHT: full-height explanation panel
             The page never scrolls. Long explanations scroll inside
             the right panel, which is tall enough to read comfortably. -->
        <div v-else-if="currentTestQuestion" class="test-stage">
          <div class="test-grid">

            <!-- LEFT COLUMN — question + options -->
            <div class="test-left">
              <div class="test-left-buffer">
                <section
                  v-for="pane in testRenderableStages"
                  v-show="pane.stage"
                  :key="pane.slot"
                  @vue:mounted="recordActivePerfPoint('journey.stage.mounted', { slot: pane.slot, index: pane.stage?.index ?? null })"
                  @vue:unmounted="recordActivePerfPoint('journey.stage.unmounted', { slot: pane.slot, index: pane.stage?.index ?? null })"
                  class="test-left-stage"
                  :class="{
                    'test-left-stage--active': pane.isActive,
                    'test-left-stage--standby': !pane.isActive,
                  }"
                  :aria-hidden="pane.isActive ? 'false' : 'true'"
                >
                  <template v-if="pane.stage">
                    <div class="q-card">
                      <p class="q-eyebrow">QUESTION</p>
                      <p
                        v-if="(pane.stage.question.stem ?? '').trim()"
                        class="test-question"
                      >
                        <template v-if="perfFlags.minimalRender">{{ pane.stage.question.stem }}</template>
                        <MathText v-else :text="pane.stage.question.stem" />
                      </p>
                      <p v-else class="test-inline-error">
                        Question text is missing for this item.
                      </p>
                    </div>

                    <div class="options-card">
                      <p class="q-eyebrow">CHOOSE AN ANSWER</p>
                      <ul v-if="pane.stage.question.options.length" class="test-options">
                        <li v-for="option in pane.stage.question.options" :key="option.id">
                          <button
                            class="test-option"
                            :class="{
                              'test-option--picked':  pane.isActive && isPickedOption(option.id),
                              'test-option--correct': pane.isActive && testSelectedOptionId !== null && isCorrectOption(option.id),
                              'test-option--wrong':   pane.isActive && isWrongPickedOption(option.id),
                              'test-option--faded':   pane.isActive && testSelectedOptionId !== null && !isCorrectOption(option.id) && !isPickedOption(option.id),
                            }"
                            :disabled="!pane.isActive || testLocked || testAdvancing"
                            @click="pickTestOptionFromStage(pane.isActive, option.id)"
                          >
                            <span class="test-option-letter">{{ option.label }}</span>
                            <span class="test-option-text">
                              <template v-if="perfFlags.minimalRender">{{ option.text }}</template>
                              <MathText v-else :text="option.text" />
                            </span>
                          </button>
                        </li>
                      </ul>
                      <p v-else class="test-inline-error">
                        Answer choices are missing for this item.
                      </p>

                      <p
                        v-if="pane.isActive && testError && testRawQuestions.length > 0"
                        class="test-inline-error"
                      >
                        {{ testError }}
                      </p>
                    </div>
                  </template>
                </section>
              </div>

              <div v-if="testSelectedOptionId !== null" class="next-row">
                <button
                  type="button"
                  class="fb-next"
                  :disabled="testAdvancing"
                  @click.prevent="nextTestQuestion"
                >
                  <span>
                    {{
                      testAdvancing
                        ? 'Finishing test'
                        : (testIndex >= testRawQuestions.length - 1 ? 'Finish test' : 'Next question')
                    }}
                  </span>
                  <span class="fb-next-arrow" aria-hidden="true">→</span>
                </button>
              </div>

            </div>

            <!-- RIGHT COLUMN — editorial answer-review panel -->
            <aside class="test-right">
              <!-- No `mode="out-in"`: the new state must mount in the SAME
                   frame as the click so Next feels instant. The two children
                   cross-fade (absolutely positioned) instead of sequencing. -->
              <Transition name="fb-in">
                <!-- ANSWERED -->
                <div
                  v-if="testSelectedOptionId !== null"
                  key="answered"
                  class="fb-panel"
                  :class="{
                    'fb-panel--ok':  testPendingCorrect === true,
                    'fb-panel--err': testPendingCorrect === false,
                  }"
                >
                  <header class="fb-top">
                    <p class="fb-eyebrow">Answer review</p>
                    <h2 class="fb-verdict">
                      {{ testPendingCorrect ? 'Correct' : 'Incorrect' }}
                    </h2>
                    <p v-if="testCorrectOption" class="fb-tagline">
                      <template v-if="testPendingCorrect">
                        You picked <strong>{{ testSelectedOption?.label ?? testCorrectOption.label }}</strong>.
                      </template>
                      <template v-else>
                        The correct answer is <strong>{{ testCorrectOption.label }}</strong>.
                      </template>
                    </p>
                  </header>

                  <hr class="fb-rule" aria-hidden="true" />

                  <section class="fb-explanation">
                    <p class="fb-eyebrow">Explanation</p>
                    <div class="fb-scroll">
                      <p
                        v-if="parsedExplanation.hasAny"
                        class="fb-commentary"
                      >
                        <template v-if="perfFlags.minimalRender">{{ parsedExplanation.commentary }}</template>
                        <MathText v-else :text="parsedExplanation.commentary" />
                      </p>

                      <ol
                        v-if="!perfFlags.minimalRender && parsedExplanation.steps.length > 0"
                        class="fb-steps"
                      >
                        <li
                          v-for="(step, i) in parsedExplanation.steps"
                          :key="i"
                          class="fb-step"
                        >
                          <span class="fb-step-num">{{ String(i + 1).padStart(2, '0') }}</span>
                          <div class="fb-step-body">
                            <p v-if="step.label" class="fb-step-label">
                              <MathText :text="step.label" />
                            </p>
                            <p class="fb-step-detail">
                              <MathText :text="step.detail" />
                            </p>
                          </div>
                        </li>
                      </ol>

                      <p v-else class="fb-empty">
                        No explanation available for this question.
                      </p>
                    </div>
                  </section>
                </div>

                <!-- IDLE -->
                <div v-else key="idle" class="fb-panel fb-panel--idle">
                  <header class="fb-top">
                    <p class="fb-eyebrow">Answer review</p>
                    <p class="fb-idle-prompt">
                      Pick an option on the left.<br>
                      Your result and a full explanation will appear here.
                    </p>
                  </header>
                </div>
              </Transition>
            </aside>

          </div>
        </div>

        <div v-else class="test-status">
          <p class="test-status-tag test-status-tag--err">[ NO QUESTIONS ]</p>
          <p class="test-status-msg">No question could be rendered for this topic yet.</p>
          <button class="btn-secondary" @click="backFromTest">Back</button>
        </div>
      </div>

      <!-- Floating LEVEL panel â€” only while browsing courses -->
      <div v-if="mode === 'courses'" class="levels-panel">
        <div class="levels-header">
          <p class="levels-title">ROUTES</p>
          <p class="levels-sub">{{ routePanelRows.length }} tracked</p>
        </div>
        <div class="levels-list">
          <article
            v-for="entry in routePanelRows"
            :key="entry.id"
            class="level-row level-row--summary"
          >
            <div class="level-copy">
              <span class="level-label">{{ entry.label }}</span>
              <span class="level-meta">{{ entry.meta }}</span>
            </div>
            <span class="level-badge">{{ entry.badge }}</span>
          </article>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   EXPLORE â€” Nothing Design across every zoom level.
   Flat surfaces, typography does the hierarchy, no colored icons,
   no shadows, no frosted blobs. One monochrome system.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */

/* Page error banner */
.page-error {
  flex-shrink: 0;
  display: flex;
  gap: 12px;
  align-items: baseline;
  padding: 10px clamp(28px, 3vw, 56px);
  background: color-mix(in srgb, var(--warm, #c2410c) 8%, transparent);
}
.page-error-tag {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--warm, #c2410c);
}
.page-error-msg {
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-size: 12px;
  color: var(--ink-secondary);
}

/* Page header container */
.page-header {
  flex-shrink: 0;
  padding: 36px clamp(28px, 3vw, 56px) 28px;
  background: var(--paper);
}

/* Page-level title / subtitle (replaces .font-display + utility chain) */
.page-title {
  margin: 0 0 4px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 24px;
  letter-spacing: -0.014em;
  color: var(--ink);
  line-height: 1.15;
}
.page-sub {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  text-transform: uppercase;
}

/* Courses body = scrollable pane with right-side reserve for level panel */
.courses-body {
  padding: 8px clamp(28px, 3vw, 56px) 96px;
  padding-right: clamp(260px, 20vw, 300px); /* reserves room for .levels-panel */
}
@media (max-width: 960px) {
  .courses-body { padding-right: clamp(28px, 3vw, 56px); }
}

/* Search â€” flat underline, no box */
.search-wrap {
  position: relative;
  max-width: 720px;
  margin: 12px 0 32px;
}
.search-icon {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--ink-muted);
  pointer-events: none;
}
.search-input {
  width: 100%;
  padding: 12px 80px 12px 28px;
  border-radius: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 14px;
  font-weight: 400;
  letter-spacing: -0.005em;
  color: var(--ink);
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-soft);
  outline: none;
  transition: border-color 180ms ease;
}
.search-input:focus { border-bottom-color: var(--ink); }
.search-input::placeholder { color: var(--ink-muted); font-weight: 400; }
.search-count {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
}

/* Courses grid â€” responsive 1/2/3 cols, flat cards */
.courses-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(360px, 100%), 1fr));
  gap: 16px;
  align-items: start;
}

.course-card {
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 28px 30px 26px;
  border-radius: 16px;
  background: var(--surface);
  border: none;
  cursor: pointer;
  text-align: left;
  transition: transform 200ms cubic-bezier(0.16, 1, 0.3, 1), background 200ms ease;
  min-height: 180px;
  font-family: inherit;
}
.course-card:hover {
  transform: translateY(-2px);
  background: color-mix(in srgb, var(--ink) 2%, var(--surface));
}
.course-card:hover .cc-open-arrow {
  opacity: 1;
  transform: translateX(0);
  color: var(--ink);
}

.cc-top {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 12px;
}
.cc-cat {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
  text-transform: uppercase;
}
.cc-open-arrow {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 14px;
  color: var(--ink-muted);
  opacity: 0;
  transform: translateX(-4px);
  transition: opacity 180ms ease, transform 180ms ease, color 180ms ease;
}

.cc-name {
  margin: 4px 0 auto;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 20px;
  letter-spacing: -0.01em;
  line-height: 1.25;
  color: var(--ink);
  text-wrap: balance;
}

.cc-stats {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
  display: inline-flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: baseline;
  text-transform: uppercase;
}
.cc-sep { color: rgba(26, 22, 18, 0.22); }

.cc-foot {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding-top: 6px;
}
.cc-pct {
  display: inline-flex;
  align-items: baseline;
  gap: 1px;
  font-variant-numeric: tabular-nums;
}
.cc-pct-num {
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-weight: 300;
  font-size: 36px;
  letter-spacing: -0.028em;
  line-height: 1;
  color: var(--ink);
}
.cc-pct-u {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}
.cc-pct--new .cc-pct-num { color: var(--ink-muted); }
.cc-state {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  text-transform: uppercase;
  margin-left: 4px;
}

/* Hairline progress track along the bottom of the card */
.cc-progress {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 2px;
  background: var(--border-soft);
  overflow: hidden;
}
.cc-progress-fill {
  height: 100%;
  background: var(--ink);
  transition: width 900ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* Course skeleton */
.course-skel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 28px 30px 26px;
  min-height: 180px;
  border-radius: 16px;
  background: var(--surface);
}
.course-skel-top,
.course-skel-name,
.course-skel-stats,
.course-skel-bar {
  border-radius: 3px;
  background: linear-gradient(
    90deg,
    var(--border-soft) 0%, var(--border-soft) 35%,
    rgba(26, 22, 18, 0.02) 50%,
    var(--border-soft) 65%, var(--border-soft) 100%);
  background-size: 200% 100%;
  animation: topic-skel-shimmer 1400ms ease-in-out infinite;
}
.course-skel-top   { height: 10px; width: 35%; }
.course-skel-name  { height: 22px; width: 70%; margin-top: 6px; }
.course-skel-stats { height: 10px; width: 85%; margin-top: auto; }
.course-skel-bar   { height: 34px; width: 50%; margin-top: 4px; }

.courses-empty {
  padding: 96px 24px;
  text-align: center;
}

/* ─── Recent activity ─── hairline-row list below the courses grid.
   Typography-only, no card — stays consistent with the rest of the
   Nothing aesthetic. */
.recent-section {
  margin-top: 72px;
  max-width: 880px;
}
.recent-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  padding-bottom: 14px;
  margin-bottom: 8px;
  border-bottom: 1px solid var(--border-soft, rgba(26,22,18,0.08));
  gap: 16px;
  flex-wrap: wrap;
}
.recent-eyebrow {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.26em;
  color: var(--ink, #1a1612);
  text-transform: uppercase;
}
.recent-sub {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.18em;
  color: var(--ink-muted, #9e958d);
  text-transform: uppercase;
}

.recent-list {
  list-style: none;
  padding: 0;
  margin: 0;
}
.recent-row {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr) auto;
  gap: 22px;
  align-items: baseline;
  padding: 16px 4px;
  border-bottom: 1px solid var(--border-soft, rgba(26,22,18,0.08));
}
.recent-row:last-child { border-bottom: none; }

.recent-time {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted, #9e958d);
  text-transform: uppercase;
}

.recent-body > p { margin: 0; }
.recent-topic {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
  font-weight: 500;
  font-size: 14px;
  line-height: 1.4;
  letter-spacing: -0.005em;
  color: var(--ink, #1a1612);
  text-wrap: pretty;
}
.recent-meta {
  margin-top: 4px !important;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.16em;
  color: var(--ink-muted, #9e958d);
  text-transform: uppercase;
}

.recent-score {
  display: inline-flex;
  align-items: baseline;
  gap: 2px;
  font-variant-numeric: tabular-nums;
}
.recent-score-num {
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-weight: 500;
  font-size: 22px;
  letter-spacing: -0.02em;
  color: var(--ink, #1a1612);
}
.recent-score-sep {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  color: var(--ink-muted, #9e958d);
  margin: 0 2px;
}
.recent-score-total {
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-weight: 300;
  font-size: 18px;
  letter-spacing: -0.02em;
  color: var(--ink-muted, #9e958d);
}

/* Color only on the score — encodes the outcome band */
.recent-row--ok  .recent-score-num { color: var(--success, #15803d); }
.recent-row--mid .recent-score-num { color: var(--warm, #c2410c); }
.recent-row--err .recent-score-num { color: var(--danger, #b91c1c); }

/* â”€â”€ LEVEL panel â€” flat surface, typographic â”€â”€ */
.levels-panel {
  position: absolute;
  top: 8px;
  right: 12px;
  bottom: 12px;
  width: 240px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 16px;
  background: var(--surface);
  border: none;
  box-shadow: none;
}
@media (max-width: 960px) { .levels-panel { display: none; } }

.levels-header {
  padding: 22px 22px 14px;
  flex-shrink: 0;
}
.levels-title {
  margin: 0 0 4px;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink);
  text-transform: uppercase;
}
.levels-sub {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  text-transform: uppercase;
}

.levels-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 10px 14px;
  box-sizing: border-box;
}
.levels-list::-webkit-scrollbar { width: 0; }

.level-row {
  width: 100%;
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: center;
  gap: 8px;
  padding: 11px 14px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 12.5px;
  font-weight: 400;
  color: var(--ink-secondary);
  background: transparent;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
  text-align: left;
  letter-spacing: -0.003em;
}
.level-row:hover {
  background: color-mix(in srgb, var(--ink) 4%, transparent);
  color: var(--ink);
}
.level-row.active {
  background: var(--ink);
  color: var(--paper);
}
.level-row.active::before { display: none; }
.level-row.active .level-badge {
  color: var(--paper);
  opacity: 0.66;
}
.level-row--summary {
  grid-template-columns: minmax(0, 1fr) auto;
  cursor: default;
}
.level-row--summary:hover {
  background: color-mix(in srgb, var(--ink) 4%, transparent);
  color: inherit;
}

.level-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.level-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.level-meta {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
  text-transform: uppercase;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.level-badge {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  background: transparent;
  color: var(--ink-muted);
  padding: 0 0 0 8px;
  margin: 0;
  font-variant-numeric: tabular-nums;
}

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   INLINE FLOW â€” breadcrumb, topic list, test arena
   Added to support: pick course â†’ pick topic â†’ take test, all on
   /student/journey without routing away.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */

/* Breadcrumb controls above the page title */
.crumb-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: none;
  border-radius: 999px;
  background: var(--surface);
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-secondary);
  cursor: pointer;
  transition: color 120ms ease, background 120ms ease;
}
.crumb-btn:hover {
  color: var(--ink);
  background: var(--paper-warm, var(--border-soft));
}
.crumb-sep {
  font-size: 12px;
  color: var(--ink-muted);
}
.crumb-current {
  font-size: 11px;
  font-weight: 600;
  color: var(--ink);
  padding: 4px 4px;
}

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   TOPICS VIEW â€” Nothing Ã— AIDesigner
   Strand cards in an auto-fitting grid. Each card shows its
   curriculum path as a row of connected dots (one per topic), with
   full topic rows below, and a footer that states what the strand
   unlocks or completes.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */

.topic-scroll { scrollbar-width: thin; }
.topic-scroll::-webkit-scrollbar { width: 6px; }
.topic-scroll::-webkit-scrollbar-thumb { background: var(--border-strong); border-radius: 99px; }
.topic-scroll::-webkit-scrollbar-thumb:hover { background: var(--ink-muted); }

.topic-inner {
  max-width: 1440px;
  margin: 0 auto;
  padding: 16px clamp(28px, 3vw, 56px) 96px;
}

/* Responsive grid: 1 col <960px, 2 cols up to 1440px, 3 cols beyond */
.strand-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(460px, 100%), 1fr));
  gap: 20px;
  align-items: start;
}

/* â”€â”€ Strand card â”€â”€ */
.strand-card {
  background: var(--surface);
  border-radius: 18px;
  padding: 32px 34px 26px;
  display: flex;
  flex-direction: column;
  gap: 28px;
  min-width: 0;
  animation: strand-in 620ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes strand-in {
  from { opacity: 0; transform: translateY(12px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* Header: big number + strand name + meta */
.s-head {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  column-gap: 22px;
  align-items: start;
  padding-bottom: 22px;
  border-bottom: 1px solid var(--border-soft);
}
.s-num {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-weight: 700;
  font-size: 38px;
  letter-spacing: -0.02em;
  color: var(--ink);
  line-height: 0.92;
  font-variant-numeric: tabular-nums;
}
.s-titles { min-width: 0; }
.s-name {
  margin: 0 0 8px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 22px;
  letter-spacing: -0.012em;
  line-height: 1.2;
  color: var(--ink);
  text-wrap: balance;
}
.s-meta {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  display: inline-flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: baseline;
  text-transform: uppercase;
}
.s-code {
  color: var(--ink-secondary);
  font-weight: 700;
}
.s-sep { color: rgba(26, 22, 18, 0.22); }

/* Clusters inside a strand */
.s-clusters {
  display: flex;
  flex-direction: column;
  gap: 28px;
}
.cluster {
  display: flex;
  flex-direction: column;
  gap: 16px;
  position: relative;
}
/* Connector between clusters: subtle vertical ascent from one to the next */
.cluster + .cluster::before {
  content: '';
  position: absolute;
  top: -20px;
  left: 6px;
  width: 1px;
  height: 12px;
  background: var(--ink-muted);
  opacity: 0.3;
}
.cluster-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 12px;
}
.cluster-name {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink);
  text-transform: uppercase;
}
.cluster-code {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  font-weight: 400;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
}

/* â”€â”€ Path visualization â”€â”€ */
/* Row of nodes connected by hairlines. 26px gap = 26px connector. */
.path {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 26px;
  padding: 6px 0 4px;
}
.node {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 160ms cubic-bezier(0.16, 1, 0.3, 1);
}
.node + .node::before {
  content: '';
  position: absolute;
  left: -26px;
  top: 6px;
  width: 26px;
  height: 1px;
  background: var(--ink-muted);
  opacity: 0.38;
  pointer-events: none;
}
.node:hover,
.node:focus-visible {
  outline: none;
  transform: translateY(-3px);
}
.node-dot {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: var(--ink);
  transition: box-shadow 180ms ease, transform 180ms ease;
}
.node:hover .node-dot,
.node:focus-visible .node-dot {
  transform: scale(1.12);
  box-shadow: 0 0 0 5px color-mix(in srgb, var(--ink) 14%, transparent);
}
.node-n {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
  transition: color 180ms ease;
}
.node:hover .node-n,
.node:focus-visible .node-n { color: var(--ink); }

/* â”€â”€ Topic rows inside a cluster â”€â”€ */
.cluster-topics {
  list-style: none;
  padding: 0;
  margin: 0;
}
.c-topic {
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr) auto;
  align-items: baseline;
  column-gap: 14px;
  padding: 14px 2px 14px 4px;
  border-top: 1px solid var(--border-soft);
  cursor: pointer;
  outline: none;
  transition: padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1), background 140ms ease;
}
.c-topic:first-child { border-top: none; padding-top: 10px; }
.c-topic:hover,
.c-topic:focus-visible {
  background: color-mix(in srgb, var(--ink) 3%, transparent);
  padding-left: 12px;
}
.c-topic-n {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
  padding-top: 2px;
}
.c-topic-name {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 13px;
  line-height: 1.45;
  color: var(--ink);
  letter-spacing: -0.003em;
  min-width: 0;
  text-wrap: balance;
}
.c-topic-code {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  font-weight: 400;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  white-space: nowrap;
  padding-top: 3px;
}

/* â”€â”€ Unlock footer: the linkage to what this strand enables â”€â”€ */
.s-unlock {
  margin-top: auto;
  padding-top: 20px;
  border-top: 1px solid var(--border-soft);
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 10px;
}
.u-tag {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink-muted);
  text-transform: uppercase;
}
.u-arrow {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
  color: var(--ink);
}
.u-name {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 13px;
  letter-spacing: -0.005em;
  color: var(--ink);
  text-transform: uppercase;
}
.s-unlock--final .u-arrow { color: var(--warm, #c2410c); }

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   GENERAL VIEW â€” strand cards are clickable, each one lists its
   sub-strand names (the short clean ones). No per-topic noise here.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
.strand-card--clickable {
  cursor: pointer;
  outline: none;
  transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1), background 200ms ease;
}
.strand-card--clickable:hover,
.strand-card--clickable:focus-visible {
  transform: translateY(-2px);
  background: color-mix(in srgb, var(--ink) 1.5%, var(--surface));
}

.substrand-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
}
.substrand-row {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr) auto;
  align-items: baseline;
  gap: 14px;
  padding: 12px 2px;
  border-top: 1px solid var(--border-soft);
}
.substrand-row:first-child {
  border-top: none;
  padding-top: 6px;
}
.ss-n {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
}
.ss-name {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 14px;
  color: var(--ink);
  letter-spacing: -0.005em;
  line-height: 1.4;
}
.ss-count {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
}

.s-card-foot {
  margin-top: auto;
  padding-top: 20px;
  border-top: 1px solid var(--border-soft);
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 14px;
  flex-wrap: wrap;
}
.open-strand {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  text-transform: uppercase;
}
.open-strand-arrow {
  transition: transform 200ms ease;
}
.strand-card--clickable:hover .open-strand-arrow,
.strand-card--clickable:focus-visible .open-strand-arrow {
  transform: translateX(4px);
}
.unlock-hint {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
  text-transform: uppercase;
}
.unlock-hint--final { color: var(--warm, #c2410c); }

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   STRAND VIEW â€” one strand, every sub-strand as a section, every
   topic listed under its sub-strand with code + full outcome text.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
.topic-inner--narrow {
  max-width: 960px;
  margin: 0 auto;
}

.strand-detail {
  display: flex;
  flex-direction: column;
  gap: 40px;
  padding-bottom: 40px;
}

.strand-plate {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 24px;
  padding: 22px 26px;
  background: var(--surface);
  border-radius: 18px;
  flex-wrap: wrap;
}
.plate-left {
  display: flex;
  align-items: baseline;
  gap: 22px;
  min-width: 0;
}
.plate-num {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-weight: 700;
  font-size: 44px;
  letter-spacing: -0.02em;
  color: var(--ink);
  line-height: 0.9;
  font-variant-numeric: tabular-nums;
}
.plate-name {
  margin: 0 0 6px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 28px;
  color: var(--ink);
  letter-spacing: -0.012em;
  line-height: 1.1;
}
.plate-meta {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  color: var(--ink-muted);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  display: inline-flex;
  gap: 8px;
  flex-wrap: wrap;
}
.plate-code { color: var(--ink-secondary); font-weight: 700; }

.ss-section { display: flex; flex-direction: column; gap: 18px; }

.ss-section-head {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 18px;
  align-items: baseline;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border-soft);
}
.ss-section-n {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--ink);
  line-height: 1;
  font-variant-numeric: tabular-nums;
}
.ss-section-name {
  margin: 0 0 4px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 20px;
  color: var(--ink);
  letter-spacing: -0.01em;
  line-height: 1.2;
}
.ss-section-meta {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  color: var(--ink-muted);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  display: inline-flex;
  gap: 8px;
  flex-wrap: wrap;
}
.ss-section-code { color: var(--ink-secondary); font-weight: 700; }

.ss-topics { list-style: none; padding: 0; margin: 0; }
.ss-topic {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr) 20px;
  align-items: start;
  gap: 18px;
  padding: 18px 4px;
  border-bottom: 1px solid var(--border-soft);
  cursor: pointer;
  outline: none;
  transition: padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1), background 160ms ease;
}
.ss-topic:last-child { border-bottom: none; }
.ss-topic:hover,
.ss-topic:focus-visible {
  background: color-mix(in srgb, var(--ink) 3%, transparent);
  padding-left: 14px;
}
.sst-n {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  font-variant-numeric: tabular-nums;
  padding-top: 4px;
}
.sst-body { min-width: 0; }
.sst-name {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 14px;
  line-height: 1.5;
  color: var(--ink);
  letter-spacing: -0.003em;
  text-wrap: balance;
}
.sst-code {
  margin: 4px 0 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  color: var(--ink-muted);
  letter-spacing: 0.18em;
}
.sst-arrow {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  color: var(--ink-muted);
  opacity: 0;
  transform: translateX(-4px);
  transition: opacity 160ms ease, transform 160ms ease, color 160ms ease;
  padding-top: 2px;
  justify-self: end;
}
.ss-topic:hover .sst-arrow,
.ss-topic:focus-visible .sst-arrow {
  opacity: 1;
  transform: translateX(0);
  color: var(--ink);
}

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   TOPIC VIEW â€” one topic's full statement + Start Test CTA.
   Editorial typographic treatment, lots of whitespace.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
.topic-detail {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 8px 8px 64px;
  max-width: 780px;
}

.td-kicker {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
  text-transform: uppercase;
  display: inline-flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: baseline;
}
.td-kicker-strand { color: var(--ink); }
.td-kicker-sep { color: rgba(26, 22, 18, 0.3); }

.td-statement {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 600;
  font-size: clamp(22px, 2.4vw, 28px);
  line-height: 1.25;
  letter-spacing: -0.01em;
  color: var(--ink);
  text-wrap: balance;
}

.td-goal-block {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 18px 20px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--panel) 82%, white 18%);
}

.td-goal-label {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-muted);
}

.td-goal {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 15px;
  line-height: 1.6;
  color: var(--ink-secondary);
}

.td-goal-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.td-goal-item {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.td-goal-step {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
  padding-top: 4px;
}

.td-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 8px;
}
.btn-lg {
  padding: 14px 26px;
  font-size: 13px;
  letter-spacing: 0.06em;
}

.td-nearby {
  margin-top: 24px;
  padding-top: 28px;
  border-top: 1px solid var(--border-soft);
}
.td-nearby-head {
  margin: 0 0 16px;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink-muted);
}
.td-nearby-list { list-style: none; padding: 0; margin: 0; }
.td-nearby-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 18px;
  align-items: start;
  padding: 14px 4px;
  border-bottom: 1px solid var(--border-soft);
  cursor: pointer;
  outline: none;
  transition: padding-left 160ms ease, background 140ms ease;
}
.td-nearby-item:last-child { border-bottom: none; }
.td-nearby-item:hover,
.td-nearby-item:focus-visible {
  background: color-mix(in srgb, var(--ink) 3%, transparent);
  padding-left: 12px;
}
.td-nearby-name {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 13px;
  color: var(--ink-secondary);
  line-height: 1.5;
  letter-spacing: -0.003em;
  text-wrap: balance;
}

@media (max-width: 720px) {
  .strand-plate { padding: 18px 20px; }
  .plate-num { font-size: 34px; }
  .plate-name { font-size: 22px; }
  .ss-section-head { grid-template-columns: auto minmax(0, 1fr); gap: 14px; }
  .ss-section-n { font-size: 18px; }
  .ss-section-name { font-size: 17px; }
  .ss-topic { grid-template-columns: 36px minmax(0, 1fr); }
  .sst-arrow { display: none; }
  .td-nearby-item { grid-template-columns: minmax(0, 1fr); }
}

/* â”€â”€ Skeleton (matches real card shape) â”€â”€ */
.strand-skel {
  background: var(--surface);
  border-radius: 18px;
  padding: 32px 34px 26px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 280px;
}
.strand-skel-head {
  height: 42px;
  border-radius: 6px;
  background: linear-gradient(
    90deg, var(--border-soft) 0%, var(--border-soft) 35%,
    rgba(26, 22, 18, 0.02) 50%, var(--border-soft) 65%, var(--border-soft) 100%);
  background-size: 200% 100%;
  animation: topic-skel-shimmer 1400ms ease-in-out infinite;
}
.strand-skel-path {
  height: 14px;
  border-radius: 99px;
  background: linear-gradient(
    90deg, var(--border-soft) 0%, var(--border-soft) 35%,
    rgba(26, 22, 18, 0.02) 50%, var(--border-soft) 65%, var(--border-soft) 100%);
  background-size: 200% 100%;
  animation: topic-skel-shimmer 1400ms ease-in-out infinite;
  width: 80%;
}
.strand-skel-row {
  height: 18px;
  border-radius: 3px;
  background: linear-gradient(
    90deg, var(--border-soft) 0%, var(--border-soft) 35%,
    rgba(26, 22, 18, 0.02) 50%, var(--border-soft) 65%, var(--border-soft) 100%);
  background-size: 200% 100%;
  animation: topic-skel-shimmer 1400ms ease-in-out infinite;
}
.strand-skel-row:nth-child(odd) { width: 92%; }
.strand-skel-row:nth-child(even) { width: 78%; }
@keyframes topic-skel-shimmer {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* â”€â”€ Empty / error states â”€â”€ */
.topic-empty {
  padding: 96px 24px;
  text-align: center;
  max-width: 420px;
  margin: 0 auto;
}
.topic-empty-tag {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink-muted);
  margin: 0 0 10px;
}
.topic-empty-sub {
  font-size: 13px;
  color: var(--ink-secondary);
  margin: 0;
  line-height: 1.55;
}

@media (max-width: 960px) {
  .strand-card { padding: 26px 24px 22px; }
  .s-num { font-size: 32px; }
  .s-name { font-size: 19px; }
}
@media (max-width: 640px) {
  .topic-inner { padding: 12px 18px 64px; }
  .strand-card { padding: 22px 20px 20px; }
  .s-head { column-gap: 16px; padding-bottom: 18px; }
  .s-num { font-size: 28px; }
  .s-name { font-size: 17px; }
  .path { gap: 22px; }
  .node + .node::before { left: -22px; width: 22px; }
}

@media (prefers-reduced-motion: reduce) {
  .strand-card,
  .strand-skel-head,
  .strand-skel-path,
  .strand-skel-row { animation: none !important; }
  .c-topic,
  .node,
  .node-dot { transition: none !important; }
}

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   TEST VIEW â€” no-scroll two-column layout (Nothing)
   LEFT:  question + options (never pushed)
   RIGHT: instrument panel (progress / feedback swaps in place)
   No borders. Breathing room. Feedback never shifts the viewport.
   â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
.test-view {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: var(--ink, #1a1612);
}

/* ─────── Header progress dots (test mode) ─────── */
.test-head-dots {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  vertical-align: middle;
  margin-right: 14px;
}
.test-head-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--ink);
  opacity: 0.14;
  transition: transform 220ms cubic-bezier(0.16, 1, 0.3, 1), opacity 220ms ease, box-shadow 220ms ease;
}
.test-head-dot--done    { opacity: 1; }
.test-head-dot--current {
  opacity: 1;
  transform: scale(1.4);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--ink) 20%, transparent);
}
.test-head-count {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
  vertical-align: middle;
}

/* ─────── Stage (single column, editorial width) ─────── */
.test-stage-wrap {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ─────── TEST STAGE — two-column layout ─────── */
.test-stage {
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;
  padding: clamp(16px, 2vw, 28px) clamp(20px, 3vw, 40px) clamp(20px, 3vw, 40px);
}

.test-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(360px, 480px);
  grid-template-rows: minmax(0, 1fr);
  gap: clamp(18px, 2vw, 28px);
  height: 100%;
  max-width: 1440px;
  margin: 0 auto;
}

/* LEFT column — question card on top, options card below */
.test-left {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  scrollbar-width: thin;
  padding-right: 2px;
  position: relative;
}
.test-left::-webkit-scrollbar { width: 6px; }
.test-left::-webkit-scrollbar-thumb { background: var(--border-strong); border-radius: 99px; }

.test-left-buffer {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
}

.test-left-stage {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.test-left-stage--standby {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  visibility: hidden;
  pointer-events: none;
}

.q-eyebrow {
  margin: 0 0 14px;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.26em;
  color: var(--ink-muted, #9e958d);
  text-transform: uppercase;
}

.q-card {
  padding: clamp(24px, 3vw, 36px) clamp(26px, 3vw, 40px);
  background: var(--surface, #ffffff);
  border-radius: 18px;
  flex-shrink: 0;
}

.options-card {
  padding: clamp(20px, 2.5vw, 28px) clamp(20px, 2.5vw, 28px) clamp(16px, 2vw, 22px);
  background: var(--surface, #ffffff);
  border-radius: 18px;
  flex-shrink: 0;
}

.test-question {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: clamp(20px, 2vw, 26px);
  color: var(--ink, #1a1612);
  line-height: 1.4;
  letter-spacing: -0.01em;
  margin: 0;
  text-wrap: pretty;
}
.test-question :deep(.math-text),
.test-option-text :deep(.math-text) {
  color: inherit;
}
.test-question :deep(.math-renderer),
.test-option-text :deep(.math-renderer),
.test-question :deep(.katex),
.test-option-text :deep(.katex) {
  color: inherit !important;
  opacity: 1;
}
.test-question :deep(.katex-display) {
  margin: 0.4em 0 0.2em;
  text-align: left;
}

.test-options {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.test-option {
  width: 100%;
  text-align: left;
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  align-items: baseline;
  gap: 16px;
  padding: 16px 20px;
  background: var(--paper, #faf8f5);
  border: none;
  border-radius: 12px;
  color: var(--ink, #1a1612);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 15px;
  line-height: 1.5;
  cursor: pointer;
  transition:
    background 200ms ease,
    transform 180ms cubic-bezier(0.16, 1, 0.3, 1),
    opacity 200ms ease;
}
.test-option:hover:not(:disabled) {
  background: color-mix(in srgb, var(--ink) 5%, var(--paper, #faf8f5));
  transform: translateX(4px);
}
.test-option:disabled { cursor: default; transform: none; }

.test-option-letter {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-secondary, #5c5650);
  letter-spacing: 0.14em;
  padding: 0;
  border: none;
  background: transparent;
  display: inline-block;
  min-height: auto;
}
.test-option-text {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 15px;
  line-height: 1.5;
  color: var(--ink, #1a1612);
  letter-spacing: -0.005em;
}
.test-option--picked:not(.test-option--correct):not(.test-option--wrong) {
  background: color-mix(in srgb, var(--ink) 8%, var(--paper, #faf8f5));
}
.test-option--correct {
  background: color-mix(in srgb, var(--success, #15803d) 12%, var(--paper, #faf8f5));
}
.test-option--correct .test-option-letter {
  color: var(--success, #15803d);
  font-weight: 700;
}
.test-option--wrong {
  background: color-mix(in srgb, var(--danger, #b91c1c) 12%, var(--paper, #faf8f5));
}
.test-option--wrong .test-option-letter {
  color: var(--danger, #b91c1c);
  font-weight: 700;
}
.test-option--faded { opacity: 0.4; }

.test-inline-error {
  margin: 4px 0 0;
  font-size: 12px;
  font-family: 'Space Mono', ui-monospace, monospace;
  letter-spacing: 0.14em;
  color: var(--warm, #c2410c);
}

/* ─────── Question-to-question transition ─────── */
.q-swap-enter-from  { opacity: 0; transform: translateY(8px); }
.q-swap-enter-active { transition: opacity 280ms ease, transform 320ms cubic-bezier(0.16, 1, 0.3, 1); }
.q-swap-leave-active { transition: opacity 180ms ease, transform 180ms ease; }
.q-swap-leave-to    { opacity: 0; transform: translateY(-4px); }

/* ─────── RIGHT COLUMN — full-height explanation panel ─────── */
/* `position: relative` anchors the absolutely-positioned .fb-panel
   children during their cross-fade (idle ↔ answered). Without it, both
   children would briefly stack vertically when `mode="out-in"` is off. */
.test-right {
  position: relative;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

/* ─────── RIGHT PANEL ───────
   Editorial answer-review layout. Generous padding (44px inner),
   three typographic tiers, scroll only inside the explanation body.
   Hierarchy is carried by SIZE and COLOR — weight stays restrained. */

.fb-panel {
  /* Absolutely positioned so the idle and answered children can
     overlap during the cross-fade. Parent .test-right is relative. */
  position: absolute;
  inset: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: clamp(36px, 3.5vw, 48px) clamp(32px, 3.5vw, 44px);
  /* Idle state: no card — just typography on the page.
     When a verdict lands, paint a radial wash that's strong near the
     verdict word and fades to transparent at the edges. No hard
     rectangle, no visible boundary — just a colored haze.
     No background transition: the <Transition> handles cross-fade via
     opacity, and each child is a fresh element — a 320ms background
     tween would just delay paint on NEXT with no visual benefit. */
  background: transparent;
  border-radius: 20px;
}

.fb-panel--ok {
  background:
    radial-gradient(
      ellipse 55% 40% at 22% 14%,
      color-mix(in srgb, var(--success, #15803d)  9%, transparent) 0%,
      color-mix(in srgb, var(--success, #15803d)  4%, transparent) 45%,
      transparent 80%
    );
}
.fb-panel--err {
  background:
    radial-gradient(
      ellipse 55% 40% at 22% 14%,
      color-mix(in srgb, var(--danger, #b91c1c)  9%, transparent) 0%,
      color-mix(in srgb, var(--danger, #b91c1c)  4%, transparent) 45%,
      transparent 80%
    );
}

/* ── Top block (verdict + tagline) — always visible, never scrolls ── */
.fb-top {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* Tertiary: eyebrow labels. Mono, ALL CAPS, wide tracking, dim. */
.fb-eyebrow {
  margin: 0;
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.28em;
  color: var(--ink-muted, #9e958d);
  text-transform: uppercase;
}

/* Primary: the verdict. SIZE and COLOR carry it — weight stays at 400. */
.fb-verdict {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: clamp(34px, 4vw, 44px);
  line-height: 1.05;
  letter-spacing: -0.028em;
  color: var(--success, #15803d);
}
.fb-panel--err .fb-verdict {
  color: var(--danger, #b91c1c);
}

/* Secondary: tagline. Regular weight, sentence case, answer letter emphasized. */
.fb-tagline {
  margin: 4px 0 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 15px;
  line-height: 1.55;
  color: var(--ink-secondary, #5c5650);
  letter-spacing: -0.003em;
  text-wrap: pretty;
}
.fb-tagline strong {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-weight: 700;
  font-size: 14px;
  letter-spacing: 0.12em;
  color: var(--success, #15803d);
  padding: 0 3px;
}
.fb-panel--err .fb-tagline strong {
  color: var(--danger, #b91c1c);
}

/* Hairline separator — the lightest container tool */
.fb-rule {
  flex-shrink: 0;
  border: none;
  height: 1px;
  background: var(--border-soft, rgba(26,22,18,0.10));
  margin: clamp(32px, 3vw, 44px) 0 clamp(26px, 2.5vw, 36px);
}

/* ── Explanation section — takes all remaining vertical space ── */
.fb-explanation {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.fb-scroll {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: thin;
  padding-right: 14px;
  margin-right: -6px;
}
.fb-scroll::-webkit-scrollbar { width: 6px; }
.fb-scroll::-webkit-scrollbar-thumb {
  background: rgba(26, 22, 18, 0.16);
  border-radius: 99px;
}
.fb-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(26, 22, 18, 0.32);
}

.fb-body {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 15px;
  line-height: 1.8;
  color: var(--ink, #1a1612);
  letter-spacing: -0.003em;
  text-wrap: pretty;
  max-width: 56ch;
}

.fb-empty {
  margin: 0;
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-weight: 400;
  font-size: 14px;
  line-height: 1.6;
  color: var(--ink-muted, #9e958d);
  font-style: italic;
}

/* ── Commentary lead-in (short intro paragraph) ── */
.fb-commentary {
  margin: 0 0 28px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 15px;
  line-height: 1.7;
  color: var(--ink, #1a1612);
  letter-spacing: -0.003em;
  text-wrap: pretty;
  max-width: 56ch;
}

/* ── Step-by-step list ── */
.fb-steps {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 56ch;
}
.fb-step {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 16px;
  align-items: baseline;
}
.fb-step-num {
  font-family: 'Space Mono', 'SF Mono', ui-monospace, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--ink-muted, #9e958d);
  padding-top: 3px;
  font-variant-numeric: tabular-nums;
}
.fb-step-body { min-width: 0; }
.fb-step-label {
  margin: 0 0 4px;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  font-size: 14px;
  line-height: 1.4;
  color: var(--ink, #1a1612);
  letter-spacing: -0.005em;
  text-wrap: pretty;
}
.fb-step-detail {
  margin: 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 14.5px;
  line-height: 1.75;
  color: var(--ink-secondary, #5c5650);
  letter-spacing: -0.003em;
  text-wrap: pretty;
}
/* When a step has no label, the detail IS the line — give it full primary weight */
.fb-step-body > .fb-step-detail:only-child {
  color: var(--ink, #1a1612);
}

/* ── NEXT row — below the options card on the LEFT column ── */
.next-row {
  display: flex;
  justify-content: flex-end;
  padding-top: 4px;
}
.fb-next {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding: 16px 30px;
  border: none;
  border-radius: 999px;
  background: var(--ink, #1a1612);
  color: var(--paper, #faf8f5);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.24em;
  text-transform: uppercase;
  cursor: pointer;
  transition: transform 140ms ease;
}
.fb-next:hover { transform: translateY(-1px); }
.fb-next-arrow { transition: transform 180ms ease; }
.fb-next:hover .fb-next-arrow { transform: translateX(4px); }
.fb-next:disabled {
  opacity: 0.58;
  cursor: wait;
  transform: none;
}
.fb-next:disabled .fb-next-arrow {
  transform: none;
}

/* ── Idle panel (before an answer is picked) — reuses .fb-top + .fb-eyebrow ── */
.fb-panel--idle .fb-top { gap: 12px; }
.fb-idle-prompt {
  margin: 6px 0 0;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-weight: 400;
  font-size: 15px;
  line-height: 1.7;
  color: var(--ink-muted, #9e958d);
  letter-spacing: -0.003em;
  text-wrap: pretty;
  max-width: 32ch;
}

/* Cross-fade between idle and answered states. Kept short (90ms) and
   simultaneous — the new panel paints in the SAME frame as the click,
   so NEXT feels instant rather than sequential. */
.fb-in-enter-from  { opacity: 0; }
.fb-in-enter-active { transition: opacity 90ms ease; }
.fb-in-leave-active { transition: opacity 90ms ease; }
.fb-in-leave-to    { opacity: 0; }

/* ─────── Narrow viewports — stack the two columns ─────── */
@media (max-width: 960px) {
  .test-stage { padding: 16px 16px 24px; overflow-y: auto; }
  .test-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
    height: auto;
  }
  .test-left { overflow: visible; }
  .fb-panel { min-height: 380px; max-height: 64vh; height: auto; }
}
@media (max-width: 640px) {
  .q-card, .options-card { padding: 20px 18px; }
  .fb-panel { padding: 28px 22px; }
  .fb-verdict { font-size: 30px; }
  .test-question { font-size: 18px; }
  .test-option { padding: 14px 16px; gap: 14px; }
}

/* Generic test loading / error status */
.test-status {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 48px 24px;
  text-align: center;
}
.test-status-tag {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink-muted);
  margin: 0;
  text-transform: uppercase;
}
.test-status-tag--err { color: var(--warm, #c2410c); }
.test-status-msg {
  margin: 0;
  font-size: 13px;
  color: var(--ink-secondary);
  line-height: 1.55;
  max-width: 420px;
}

/* ── Loading state: spinner + animated dots ── */
.test-status--loading { gap: 18px; }
.test-status--loading .test-status-tag {
  color: var(--ink, #1a1612);
  letter-spacing: 0.08em;
  text-transform: none;
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-size: 15px;
  font-weight: 500;
  margin-top: 6px;
}
.test-status--loading .test-status-msg {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--ink-muted);
  margin-top: 2px;
}
.test-status-hint {
  margin: 4px 0 0;
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-size: 13px;
  color: var(--ink-muted);
  letter-spacing: -0.003em;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.test-spinner {
  display: inline-flex;
  gap: 3px;
  align-items: flex-end;
  height: 18px;
}
.test-spinner > span {
  width: 3px;
  height: 100%;
  background: var(--ink, #1a1612);
  border-radius: 1px;
  animation: test-spinner-bar 900ms ease-in-out infinite;
  transform-origin: bottom;
}
.test-spinner > span:nth-child(1) { animation-delay:    0ms; }
.test-spinner > span:nth-child(2) { animation-delay:   90ms; }
.test-spinner > span:nth-child(3) { animation-delay:  180ms; }
.test-spinner > span:nth-child(4) { animation-delay:  270ms; }
.test-spinner > span:nth-child(5) { animation-delay:  360ms; }
.test-spinner > span:nth-child(6) { animation-delay:  450ms; }
@keyframes test-spinner-bar {
  0%, 100% { transform: scaleY(0.25); opacity: 0.35; }
  50%      { transform: scaleY(1);    opacity: 1;    }
}

.test-dots {
  display: inline-flex;
  gap: 1px;
}
.test-dots > span {
  animation: test-dots-pulse 1200ms ease-in-out infinite;
  opacity: 0.2;
}
.test-dots > span:nth-child(2) { animation-delay: 200ms; }
.test-dots > span:nth-child(3) { animation-delay: 400ms; }
@keyframes test-dots-pulse {
  0%, 80%, 100% { opacity: 0.2; }
  40%           { opacity: 1;   }
}

/* â”€â”€ Buttons used by the test arena â”€â”€ */
.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  border-radius: 999px;
  background: var(--ink);
  color: var(--paper);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: transform 120ms ease;
}
.btn-primary:hover { transform: translateY(-1px); }
.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: transparent;
  color: var(--ink);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}
.btn-secondary:hover { background: var(--surface); }

/* â”€â”€ Completion screen â”€â”€ */
.test-done {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  max-width: 620px;
  margin: 0 auto;
  padding: 48px 24px;
  text-align: center;
}
.test-done-kicker {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.24em;
  color: var(--ink-muted);
  margin: 0 0 8px;
  text-transform: uppercase;
}
.test-done-score {
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: clamp(72px, 10vw, 120px);
  font-weight: 300;
  color: var(--ink);
  letter-spacing: -0.04em;
  line-height: 0.9;
  margin: 0 0 14px;
  font-variant-numeric: tabular-nums;
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
}
.test-done-num { font-weight: 500; }
.test-done-div {
  color: var(--ink-muted);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 0.4em;
  font-weight: 300;
  letter-spacing: 0.08em;
}
.test-done-total { color: var(--ink-muted); font-weight: 300; }
.test-done-sub {
  font-family: 'Space Grotesk', system-ui, sans-serif;
  font-size: 14px;
  color: var(--ink-secondary);
  margin: 0 0 24px;
  max-width: 420px;
  line-height: 1.55;
  text-wrap: balance;
}
.test-done-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}
</style>
