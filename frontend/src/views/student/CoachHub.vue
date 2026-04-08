<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getCoachNextAction,
  getStudentDashboard,
  getPriorityTopics,
  type CoachNextActionDto,
  type StudentDashboardDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import {
  PhArrowRight,
  PhBookOpen,
  PhClockCountdown,
  PhFlame,
  PhLightning,
  PhTarget,
  PhTrendUp,
} from '@phosphor-icons/vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const error = ref('')

const nextAction = ref<CoachNextActionDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topicCases = ref<TopicCaseDto[]>([])

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
  if (!auth.currentAccount) return
  const sid = auth.currentAccount.id
  try {
    const [action, dash, topics] = await Promise.all([
      getCoachNextAction(sid),
      getStudentDashboard(sid),
      getPriorityTopics(sid, 6),
    ])
    nextAction.value = action
    dashboard.value = dash
    topicCases.value = topics
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  } finally {
    loading.value = false
  }
})

function startAction() {
  if (nextAction.value?.route) router.push(nextAction.value.route)
}

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}

function bandLabel(band: string): string {
  if (band === 'strong') return 'Strong'
  if (band === 'developing') return 'Developing'
  return 'Weak'
}

type QuickCheckOption = {
  id: string
  label: string
  correct?: boolean
}

type QuickCheck = {
  id: string
  topic: string
  prompt: string
  options: QuickCheckOption[]
  explanation: string
  to: string
}

type FeedTone = 'accent' | 'warm' | 'gold' | 'muted'

type LiveFeedItem = {
  id: string
  tone: FeedTone
  title: string
  detail: string
  time: string
  action: string
  to: string
}

const quickChecks: QuickCheck[] = [
  {
    id: 'quick-gap',
    topic: 'Gap Scan',
    prompt: 'What is the fastest way to fix repeated mistakes in one topic?',
    options: [
      { id: 'a', label: 'Do random mixed questions only' },
      { id: 'b', label: 'Review error pattern and run focused drills', correct: true },
      { id: 'c', label: 'Skip that topic for now' },
    ],
    explanation: 'Pattern review + focused drilling fixes root causes faster than random practice.',
    to: '/student/knowledge-gap',
  },
  {
    id: 'quick-diagnostic',
    topic: 'Diagnostic DNA',
    prompt: 'A diagnostic test should mostly measure:',
    options: [
      { id: 'a', label: 'Your speed under pressure' },
      { id: 'b', label: 'Your strongest topics only' },
      { id: 'c', label: 'Your real weak points and misconceptions', correct: true },
    ],
    explanation: 'Diagnostics are best when they surface true gaps, not just speed.',
    to: '/student/diagnostic',
  },
  {
    id: 'quick-mock',
    topic: 'Prepare Test',
    prompt: 'Best move the day before a mock exam?',
    options: [
      { id: 'a', label: 'Learn brand new topics' },
      { id: 'b', label: 'Do light revision + one timed set', correct: true },
      { id: 'c', label: 'Avoid all practice' },
    ],
    explanation: 'Light revision and a controlled timed set keeps recall sharp without burnout.',
    to: '/student/mock',
  },
  {
    id: 'quick-review',
    topic: 'Review Queue',
    prompt: 'When should you revisit a recently learned idea?',
    options: [
      { id: 'a', label: 'Only when exams are near' },
      { id: 'b', label: 'On spaced intervals before memory fades', correct: true },
      { id: 'c', label: 'Never, once is enough' },
    ],
    explanation: 'Spaced review protects memory strength before decay becomes expensive.',
    to: '/student/memory#reviews',
  },
]

const quickCheckIndex = ref(0)
const selectedQuickOptionId = ref<string | null>(null)

const activeQuickCheck = computed(() => quickChecks[quickCheckIndex.value % quickChecks.length])
const quickAnswered = computed(() => selectedQuickOptionId.value !== null)
const quickSelectionCorrect = computed(() => {
  if (!selectedQuickOptionId.value) return false
  const option = activeQuickCheck.value.options.find(opt => opt.id === selectedQuickOptionId.value)
  return !!option?.correct
})

const quickStepLabel = computed(() => `${quickCheckIndex.value + 1}/${quickChecks.length}`)

const liveFeed = computed<LiveFeedItem[]>(() => {
  const items: LiveFeedItem[] = []

  if (nextAction.value) {
    items.push({
      id: 'coach-directive',
      tone: 'accent',
      title: nextAction.value.title,
      detail: nextAction.value.subtitle,
      time: 'now',
      action: 'Open',
      to: nextAction.value.route,
    })
  }

  if (topicCases.value.length) {
    const topTopic = topicCases.value[0]
    items.push({
      id: `topic-${topTopic.topic_id}`,
      tone: topTopic.intervention_urgency === 'high' ? 'warm' : 'gold',
      title: `${topTopic.topic_name} flagged for recovery`,
      detail: `${topTopic.intervention_mode.replace(/_/g, ' ')} recommended`,
      time: '2m ago',
      action: 'Practice',
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
      title: `${weakest.subject_name} is ${bandLabel(weakest.readiness_band)}`,
      detail: `${weakestPct}% mastered - ${weakest.weak_topic_count} weak topics`,
      time: '5m ago',
      action: 'Review',
      to: '/student/progress',
    })
  }

  items.push(
    {
      id: 'revision-box',
      tone: 'gold',
      title: 'Revision Box refreshed',
      detail: 'New revision pack suggestions are available',
      time: '12m ago',
      action: 'Open',
      to: '/student/library#revision-box',
    },
    {
      id: 'retry-zone',
      tone: 'warm',
      title: 'Retry Zone queued',
      detail: 'High-impact mistakes are ready for rematch',
      time: '17m ago',
      action: 'Retry',
      to: '/student/mistakes#retry-zone',
    },
    {
      id: 'games-hub',
      tone: 'accent',
      title: 'Games challenge available',
      detail: 'Train speed and focus in Games Hub',
      time: '26m ago',
      action: 'Play',
      to: '/student/games',
    },
  )

  return items.slice(0, 9)
})

function pickQuickOption(optionId: string) {
  if (selectedQuickOptionId.value) return
  selectedQuickOptionId.value = optionId
}

function nextQuickCheck() {
  quickCheckIndex.value = (quickCheckIndex.value + 1) % quickChecks.length
  selectedQuickOptionId.value = null
}

function openQuickCheckTopic() {
  router.push(activeQuickCheck.value.to)
}

function feedToneColor(tone: FeedTone): string {
  if (tone === 'warm') return 'var(--warm)'
  if (tone === 'gold') return 'var(--gold)'
  if (tone === 'accent') return 'var(--accent)'
  return 'var(--ink-muted)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Greeting strip -->
    <div
      class="flex-shrink-0 px-7 py-4 border-b flex items-center justify-between"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="text-[11px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ dateLabel }}</p>
        <h1 class="font-display text-xl font-bold mt-0.5" :style="{ color: 'var(--ink)' }">
          {{ greeting }}, {{ dashboard?.student_name || auth.currentAccount?.display_name || 'Student' }}
        </h1>
      </div>
      <div v-if="dashboard?.subjects?.length" class="flex items-center gap-5">
        <div
          v-for="s in dashboard.subjects.slice(0, 3)"
          :key="s.subject_id"
          class="text-center"
        >
          <p class="text-base font-black tabular-nums" :style="{ color: 'var(--ink)' }">
            {{ Math.round(s.mastered_topic_count / Math.max(s.total_topic_count, 1) * 100) }}%
          </p>
          <p class="text-[9px] font-bold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">{{ s.subject_name }}</p>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-7 grid grid-cols-12 gap-5">
      <div class="col-span-7 space-y-4">
        <div class="h-48 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        <div class="h-40 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
      <div class="col-span-5 space-y-3">
        <div v-for="i in 4" :key="i" class="h-16 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="flex-1 flex items-center justify-center">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">{{ error }}</p>
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: directive + topics -->
      <div class="flex-1 overflow-y-auto p-7 space-y-5">

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
                    Coach Directive
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
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Your coach has no pending actions. Keep practising!</p>
        </div>

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
      </div>

      <!-- Right panel — dark premium -->
      <div class="rp">

        <!-- Quick Test -->
        <div class="rp-qt">
          <div class="rp-qt-head">
            <span class="rp-label">Quick Test</span>
            <div class="rp-steps">
              <span
                v-for="(_, i) in quickChecks" :key="i"
                class="rp-step" :class="{ 'rp-step--on': i === quickCheckIndex }"
              />
            </div>
          </div>

          <div class="rp-topic-badge">{{ activeQuickCheck.topic }}</div>

          <p class="rp-question">{{ activeQuickCheck.prompt }}</p>

          <div class="rp-options">
            <button
              v-for="option in activeQuickCheck.options" :key="option.id"
              class="rp-option"
              :class="{
                'rp-option--correct': quickAnswered && option.correct,
                'rp-option--wrong':   quickAnswered && selectedQuickOptionId === option.id && !option.correct,
                'rp-option--faded':   quickAnswered && !option.correct && selectedQuickOptionId !== option.id,
              }"
              :disabled="quickAnswered"
              @click="pickQuickOption(option.id)"
            >
              <span class="rp-letter">{{ option.id.toUpperCase() }}</span>
              <span class="rp-option-text">{{ option.label }}</span>
            </button>
          </div>

          <Transition name="rp-reveal">
            <div v-if="quickAnswered" class="rp-explain"
              :class="quickSelectionCorrect ? 'rp-explain--ok' : 'rp-explain--err'">
              <p class="rp-explain-text">{{ activeQuickCheck.explanation }}</p>
              <div class="rp-explain-actions">
                <button class="rp-ghost" @click="openQuickCheckTopic">Review topic</button>
                <button class="rp-solid" @click="nextQuickCheck">Next →</button>
              </div>
            </div>
          </Transition>
        </div>

        <!-- Recent Activity -->
        <div class="rp-activity">
          <span class="rp-label" style="display:block;margin-bottom:14px">Recent Activity</span>

          <!-- 3D hero card — most recent -->
          <div
            v-if="liveFeed[0]"
            class="rp-hero"
            :style="{ '--hc': feedToneColor(liveFeed[0].tone) }"
            @click="router.push(liveFeed[0].to)"
          >
            <div class="rp-hero-body">
              <span class="rp-hero-eyebrow">Just now</span>
              <p class="rp-hero-title">{{ liveFeed[0].title }}</p>
              <p class="rp-hero-sub">{{ liveFeed[0].detail }}</p>
              <button class="rp-hero-cta">
                {{ liveFeed[0].action }}
                <PhArrowRight :size="11" weight="bold" />
              </button>
            </div>
          </div>

          <!-- Borderless feed list -->
          <div class="rp-feed">
            <button
              v-for="item in liveFeed.slice(1, 7)" :key="item.id"
              class="rp-feed-row"
              @click="router.push(item.to)"
            >
              <span class="rp-feed-dot" :style="{ background: feedToneColor(item.tone) }" />
              <div class="rp-feed-info">
                <p class="rp-feed-title">{{ item.title }}</p>
                <p class="rp-feed-time">{{ item.time }}</p>
              </div>
              <span class="rp-feed-action" :style="{ color: feedToneColor(item.tone) }">{{ item.action }}</span>
            </button>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
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

/* ═══════════════════════════════════════════
   Dark right panel
═══════════════════════════════════════════ */
.rp {
  width: 22rem;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #1c1814;
}

/* ── Quick Test ───────────────────────────── */
.rp-qt {
  flex-shrink: 0;
  padding: 22px 20px 20px;
}

.rp-qt-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.rp-label {
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.20em;
  color: rgba(255,255,255,0.28);
}

.rp-steps {
  display: flex;
  gap: 5px;
  align-items: center;
}
.rp-step {
  height: 3px;
  width: 14px;
  border-radius: 99px;
  background: rgba(255,255,255,0.12);
  transition: width 300ms ease, background 300ms ease;
}
.rp-step--on {
  width: 26px;
  background: var(--accent);
}

.rp-topic-badge {
  display: inline-block;
  padding: 4px 11px;
  border-radius: 99px;
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.13em;
  background: rgba(13,148,136,0.18);
  color: #2dd4bf;
  margin-bottom: 13px;
}

.rp-question {
  font-family: var(--font-display);
  font-size: 15.5px;
  font-weight: 700;
  line-height: 1.55;
  color: rgba(255,255,255,0.90);
  margin-bottom: 16px;
}

/* Options */
.rp-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rp-option {
  display: flex;
  align-items: center;
  gap: 11px;
  width: 100%;
  padding: 11px 14px;
  border-radius: 13px;
  background: rgba(255,255,255,0.06);
  cursor: pointer;
  text-align: left;
  box-shadow: 0 2px 8px rgba(0,0,0,0.20);
  transition:
    transform   160ms ease,
    background  180ms ease,
    box-shadow  180ms ease,
    opacity     240ms ease;
}
.rp-option:hover:not(:disabled) {
  background: rgba(255,255,255,0.11);
  transform: translateX(5px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.28);
}
.rp-option:active:not(:disabled) {
  transform: translateX(3px) scale(0.985);
}
.rp-option:disabled { cursor: default; }

.rp-option--correct {
  background: linear-gradient(135deg, #166534, #16a34a) !important;
  box-shadow: 0 10px 32px rgba(22,163,74,0.50), 0 4px 10px rgba(0,0,0,0.22) !important;
  transform: translateX(5px) !important;
}
.rp-option--wrong {
  background: linear-gradient(135deg, #991b1b, #dc2626) !important;
  box-shadow: 0 10px 32px rgba(220,38,38,0.50), 0 4px 10px rgba(0,0,0,0.22) !important;
  transform: translateX(5px) !important;
}
.rp-option--faded { opacity: 0.25; }

.rp-letter {
  width: 23px;
  height: 23px;
  border-radius: 7px;
  background: rgba(255,255,255,0.09);
  color: rgba(255,255,255,0.45);
  font-size: 10px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 180ms ease, color 180ms ease;
}
.rp-option--correct .rp-letter,
.rp-option--wrong .rp-letter {
  background: rgba(255,255,255,0.20);
  color: #fff;
}
.rp-option:hover:not(:disabled) .rp-letter {
  background: rgba(255,255,255,0.14);
  color: rgba(255,255,255,0.80);
}
.rp-option-text {
  font-size: 12.5px;
  font-weight: 500;
  color: rgba(255,255,255,0.72);
  line-height: 1.35;
  transition: color 180ms ease;
}
.rp-option--correct .rp-option-text,
.rp-option--wrong .rp-option-text { color: #fff; font-weight: 600; }

/* Explanation */
.rp-explain {
  margin-top: 14px;
  padding-left: 14px;
  position: relative;
}
.rp-explain::before {
  content: '';
  position: absolute;
  left: 0; top: 3px; bottom: 3px;
  width: 3px;
  border-radius: 99px;
}
.rp-explain--ok::before  { background: #16a34a; }
.rp-explain--err::before { background: #dc2626; }
.rp-explain-text {
  font-size: 12px;
  line-height: 1.65;
  color: rgba(255,255,255,0.52);
  margin-bottom: 13px;
}
.rp-explain-actions { display: flex; gap: 8px; }
.rp-ghost {
  padding: 6px 13px;
  border-radius: 9px;
  background: rgba(255,255,255,0.08);
  color: rgba(255,255,255,0.58);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: background 130ms ease;
}
.rp-ghost:hover { background: rgba(255,255,255,0.14); }
.rp-solid {
  padding: 6px 13px;
  border-radius: 9px;
  background: var(--accent);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 130ms ease, transform 130ms ease;
}
.rp-solid:hover { opacity: 0.85; transform: translateY(-1px); }

/* Reveal transition */
.rp-reveal-enter-active { transition: opacity 260ms ease, transform 260ms ease; }
.rp-reveal-leave-active { transition: opacity 160ms ease; }
.rp-reveal-enter-from   { opacity: 0; transform: translateY(10px); }
.rp-reveal-leave-to     { opacity: 0; }

/* ── Activity ─────────────────────────────── */
.rp-activity {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 18px 20px 16px;
  background: rgba(0,0,0,0.18);
  overflow: hidden;
}

/* Hero 3D card */
.rp-hero {
  --hc: var(--accent);
  position: relative;
  border-radius: 18px;
  padding: 20px;
  margin-bottom: 16px;
  flex-shrink: 0;
  cursor: pointer;
  overflow: hidden;

  background: linear-gradient(
    145deg,
    color-mix(in srgb, var(--hc) 80%, #fff),
    color-mix(in srgb, var(--hc) 65%, #000)
  );

  /* Tilted toward viewer from bottom */
  transform: perspective(900px) rotateX(3deg);
  transform-origin: bottom center;
  transition:
    transform  320ms cubic-bezier(0.34, 1.2, 0.64, 1),
    box-shadow 320ms ease;

  /* Stacked bottom shadows = physical depth */
  box-shadow:
    0  2px 0  color-mix(in srgb, var(--hc) 55%, #000),
    0  5px 0  color-mix(in srgb, var(--hc) 34%, #000),
    0  9px 0  color-mix(in srgb, var(--hc) 18%, #000),
    0 22px 40px color-mix(in srgb, var(--hc) 45%, transparent),
    0  8px 18px rgba(0,0,0,0.36);
}

/* Glass light sheen */
.rp-hero::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 18px;
  background:
    linear-gradient(
      to bottom,
      rgba(255,255,255,0.26) 0%,
      rgba(255,255,255,0.07) 42%,
      transparent 72%
    ),
    radial-gradient(
      ellipse 70% 55% at 20% 16%,
      rgba(255,255,255,0.22) 0%,
      transparent 66%
    );
  pointer-events: none;
}

.rp-hero:hover {
  transform: perspective(900px) rotateX(0deg) translateY(-6px);
  box-shadow:
    0  4px 0  color-mix(in srgb, var(--hc) 55%, #000),
    0  9px 0  color-mix(in srgb, var(--hc) 34%, #000),
    0 15px 0  color-mix(in srgb, var(--hc) 18%, #000),
    0 38px 56px color-mix(in srgb, var(--hc) 50%, transparent),
    0 14px 28px rgba(0,0,0,0.42);
}
.rp-hero:active {
  transform: perspective(900px) rotateX(5deg) translateY(2px);
  box-shadow:
    0  1px 0  color-mix(in srgb, var(--hc) 55%, #000),
    0  3px 0  color-mix(in srgb, var(--hc) 34%, #000),
    0  6px 0  color-mix(in srgb, var(--hc) 18%, #000),
    0 14px 22px color-mix(in srgb, var(--hc) 30%, transparent),
    0  4px 10px rgba(0,0,0,0.26);
}

.rp-hero-body { position: relative; z-index: 1; }

.rp-hero-eyebrow {
  display: block;
  font-size: 8.5px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.18em;
  color: rgba(255,255,255,0.55);
  margin-bottom: 7px;
}
.rp-hero-title {
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 700;
  color: #fff;
  line-height: 1.28;
  margin-bottom: 5px;
}
.rp-hero-sub {
  font-size: 11.5px;
  color: rgba(255,255,255,0.68);
  line-height: 1.48;
  margin-bottom: 16px;
}
.rp-hero-cta {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 14px;
  border-radius: 10px;
  background: rgba(255,255,255,0.18);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  transition: background 140ms ease, transform 140ms ease;
}
.rp-hero-cta:hover {
  background: rgba(255,255,255,0.28);
  transform: translateY(-1px);
}

/* Feed rows — clean, no borders */
.rp-feed {
  flex: 1;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.rp-feed::-webkit-scrollbar { display: none; }

.rp-feed-row {
  display: flex;
  align-items: center;
  gap: 11px;
  width: 100%;
  padding: 9px 6px;
  border-radius: 11px;
  text-align: left;
  cursor: pointer;
  transition: background 120ms ease, transform 120ms ease;
}
.rp-feed-row:hover {
  background: rgba(255,255,255,0.055);
  transform: translateX(4px);
}
.rp-feed-dot {
  width: 8px;
  height: 8px;
  border-radius: 99px;
  flex-shrink: 0;
  box-shadow: 0 0 6px currentColor;
}
.rp-feed-info { flex: 1; min-width: 0; }
.rp-feed-title {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255,255,255,0.80);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.rp-feed-time {
  font-size: 9.5px;
  color: rgba(255,255,255,0.28);
  margin-top: 1px;
}
.rp-feed-action {
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
}
</style>

