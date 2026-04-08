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

      <!-- Right panel -->
      <div class="right-panel">

        <!-- ── TOP: Quick Test ── -->
        <div class="qt-panel">
          <!-- Progress dots + label -->
          <div class="qt-header">
            <span class="panel-label">Quick Test</span>
            <div class="qt-dots">
              <div
                v-for="(_, i) in quickChecks"
                :key="i"
                class="qt-dot"
                :class="{ 'qt-dot--active': i === quickCheckIndex }"
              />
            </div>
          </div>

          <!-- Topic pill -->
          <div class="qt-topic-pill">
            <PhTarget :size="11" weight="fill" />
            {{ activeQuickCheck.topic }}
          </div>

          <!-- Question -->
          <p class="qt-question">{{ activeQuickCheck.prompt }}</p>

          <!-- Options -->
          <div class="qt-options">
            <button
              v-for="option in activeQuickCheck.options"
              :key="option.id"
              class="qt-option"
              :class="{
                'qt-option--correct': quickAnswered && option.correct,
                'qt-option--wrong': quickAnswered && selectedQuickOptionId === option.id && !option.correct,
                'qt-option--dim': quickAnswered && selectedQuickOptionId !== option.id && !option.correct,
              }"
              :disabled="quickAnswered"
              @click="pickQuickOption(option.id)"
            >
              <span class="qt-letter">{{ option.id.toUpperCase() }}</span>
              <span class="qt-option-text">{{ option.label }}</span>
            </button>
          </div>

          <!-- Explanation -->
          <Transition name="explain">
            <div v-if="quickAnswered" class="qt-explain">
              <div class="qt-explain-icon">
                <PhBookOpen :size="13" weight="fill" :style="{ color: quickSelectionCorrect ? '#16a34a' : 'var(--warm)' }" />
              </div>
              <p class="qt-explain-text">{{ activeQuickCheck.explanation }}</p>
              <div class="qt-explain-actions">
                <button class="qt-btn qt-btn--secondary" @click="openQuickCheckTopic">Review Topic</button>
                <button class="qt-btn qt-btn--primary" @click="nextQuickCheck">Next →</button>
              </div>
            </div>
          </Transition>
        </div>

        <!-- ── BOTTOM: Recent Activity ── -->
        <div class="activity-panel">
          <div class="activity-header">
            <span class="panel-label">Recent Activity</span>
            <span class="activity-count">{{ liveFeed.length }}</span>
          </div>

          <!-- Hero 3D card — most recent -->
          <div
            v-if="liveFeed[0]"
            class="hero-card"
            :style="{ '--hc': feedToneColor(liveFeed[0].tone) }"
            @click="router.push(liveFeed[0].to)"
          >
            <div class="hero-card-shine" />
            <div class="hero-card-body">
              <span class="hero-card-now">Just now</span>
              <p class="hero-card-title">{{ liveFeed[0].title }}</p>
              <p class="hero-card-detail">{{ liveFeed[0].detail }}</p>
              <button class="hero-card-cta">
                {{ liveFeed[0].action }}
                <PhArrowRight :size="12" weight="bold" />
              </button>
            </div>
          </div>

          <!-- Compact feed list -->
          <div class="compact-feed">
            <button
              v-for="item in liveFeed.slice(1, 7)"
              :key="item.id"
              class="feed-row"
              @click="router.push(item.to)"
            >
              <span class="feed-row-dot" :style="{ background: feedToneColor(item.tone) }" />
              <div class="feed-row-body">
                <p class="feed-row-title">{{ item.title }}</p>
                <p class="feed-row-time">{{ item.time }}</p>
              </div>
              <span class="feed-row-action" :style="{ color: feedToneColor(item.tone) }">
                {{ item.action }}
              </span>
            </button>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Shared ────────────────────────────────────────────── */
.section-label, .panel-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.15em;
  color: var(--ink-muted);
}

.directive-card { transition: box-shadow 140ms ease, transform 140ms ease; }
.directive-card:hover { transform: translateY(-1px); box-shadow: 0 10px 24px rgba(15,23,42,0.07); }
.cta-btn { transition: opacity 120ms ease, transform 120ms ease; }
.cta-btn:hover { opacity: 0.88; transform: translateY(-1px); }
.topic-row { transition: background-color 100ms ease; }
.topic-row:hover { background-color: var(--paper) !important; }

/* ── Right panel ───────────────────────────────────────── */
.right-panel {
  width: 22rem;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface);
  border-left: 1px solid var(--border-soft);
}

/* ── Quick Test ────────────────────────────────────────── */
.qt-panel {
  flex-shrink: 0;
  padding: 18px 16px 16px;
  border-bottom: 1px solid var(--border-soft);
  background: var(--surface);
}

.qt-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.qt-dots {
  display: flex;
  gap: 4px;
  align-items: center;
}
.qt-dot {
  width: 5px;
  height: 5px;
  border-radius: 99px;
  background: var(--border-strong);
  transition: width 220ms ease, background 220ms ease;
}
.qt-dot--active {
  width: 18px;
  background: var(--accent);
}

.qt-topic-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 9px;
  border-radius: 99px;
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--accent);
  background: var(--accent-glow);
  margin-bottom: 10px;
}

.qt-question {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--ink);
  line-height: 1.5;
  margin-bottom: 12px;
}

.qt-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.qt-option {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 11px;
  border-radius: 10px;
  background: var(--paper);
  border: 1px solid var(--border-soft);
  cursor: pointer;
  text-align: left;
  transition: transform 100ms ease, background 120ms ease, border-color 120ms ease;
}
.qt-option:hover:not(:disabled) {
  transform: translateX(3px);
  background: var(--paper-warm);
  border-color: var(--border-strong);
}
.qt-option:disabled { cursor: default; }
.qt-option:active:not(:disabled) { transform: scale(0.98); }

.qt-option--correct {
  background: rgba(22,163,74,0.09) !important;
  border-color: rgba(22,163,74,0.35) !important;
}
.qt-option--wrong {
  background: rgba(194,65,12,0.09) !important;
  border-color: rgba(194,65,12,0.35) !important;
}
.qt-option--dim { opacity: 0.45; }

.qt-letter {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  background: var(--border-strong);
  color: var(--ink-secondary);
  font-size: 10px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 120ms ease, color 120ms ease;
}
.qt-option--correct .qt-letter { background: rgba(22,163,74,0.2); color: #15803d; }
.qt-option--wrong .qt-letter { background: rgba(194,65,12,0.2); color: var(--warm); }

.qt-option-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--ink);
  line-height: 1.3;
}

/* Explanation */
.qt-explain {
  margin-top: 10px;
  padding: 11px 12px;
  border-radius: 10px;
  background: var(--paper);
  border: 1px solid var(--border-soft);
}
.qt-explain-icon { margin-bottom: 5px; }
.qt-explain-text {
  font-size: 11px;
  line-height: 1.55;
  color: var(--ink-secondary);
  margin-bottom: 10px;
}
.qt-explain-actions { display: flex; gap: 6px; }
.qt-btn {
  padding: 5px 11px;
  border-radius: 7px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: transform 110ms ease, opacity 110ms ease;
}
.qt-btn:hover { transform: translateY(-1px); opacity: 0.88; }
.qt-btn--primary { background: var(--ink); color: #fff; }
.qt-btn--secondary { background: var(--accent-glow); color: var(--accent); }

/* Explain transition */
.explain-enter-active { transition: opacity 220ms ease, transform 220ms ease; }
.explain-leave-active { transition: opacity 150ms ease; }
.explain-enter-from { opacity: 0; transform: translateY(6px); }
.explain-leave-to { opacity: 0; }

/* ── Activity panel ─────────────────────────────────────── */
.activity-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 14px 16px 12px;
  overflow: hidden;
}

.activity-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.activity-count {
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-muted);
  background: var(--border-soft);
  padding: 1px 7px;
  border-radius: 99px;
}

/* Hero 3D card */
.hero-card {
  --hc: var(--accent);
  position: relative;
  border-radius: 16px;
  padding: 16px;
  margin-bottom: 12px;
  cursor: pointer;
  overflow: hidden;
  flex-shrink: 0;

  /* Gradient from the item's colour */
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--hc) 90%, #000),
    color-mix(in srgb, var(--hc) 60%, #000)
  );

  /* 3D depth */
  transform: perspective(700px) rotateX(1.8deg);
  transform-origin: top center;
  transition: transform 200ms ease, box-shadow 200ms ease;
  box-shadow:
    0 8px 0 -2px color-mix(in srgb, var(--hc) 40%, #000),
    0 16px 0 -4px color-mix(in srgb, var(--hc) 22%, #000),
    0 20px 32px color-mix(in srgb, var(--hc) 28%, transparent),
    0 4px 12px rgba(0,0,0,0.18);
}
.hero-card:hover {
  transform: perspective(700px) rotateX(0deg) translateY(-2px);
  box-shadow:
    0 12px 0 -2px color-mix(in srgb, var(--hc) 40%, #000),
    0 22px 0 -4px color-mix(in srgb, var(--hc) 22%, #000),
    0 28px 40px color-mix(in srgb, var(--hc) 32%, transparent),
    0 6px 16px rgba(0,0,0,0.22);
}
.hero-card:active {
  transform: perspective(700px) rotateX(3deg) translateY(1px);
}

/* Glass shine overlay */
.hero-card-shine {
  position: absolute;
  inset: 0;
  border-radius: 16px;
  pointer-events: none;
  background:
    linear-gradient(
      148deg,
      rgba(255,255,255,0.22) 0%,
      rgba(255,255,255,0.06) 38%,
      transparent 65%
    ),
    radial-gradient(
      80% 50% at 18% 14%,
      rgba(255,255,255,0.18) 0%,
      transparent 70%
    );
}
/* Bottom edge highlight */
.hero-card-shine::after {
  content: '';
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 1px;
  background: rgba(255,255,255,0.15);
  border-radius: 0 0 16px 16px;
}

.hero-card-body { position: relative; z-index: 1; }

.hero-card-now {
  display: inline-block;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: rgba(255,255,255,0.65);
  margin-bottom: 6px;
}
.hero-card-title {
  font-size: 14px;
  font-weight: 700;
  color: #fff;
  line-height: 1.3;
  margin-bottom: 4px;
}
.hero-card-detail {
  font-size: 11px;
  color: rgba(255,255,255,0.72);
  line-height: 1.4;
  margin-bottom: 12px;
}
.hero-card-cta {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 8px;
  background: rgba(255,255,255,0.18);
  border: 1px solid rgba(255,255,255,0.28);
  color: #fff;
  font-size: 10.5px;
  font-weight: 700;
  cursor: pointer;
  backdrop-filter: blur(8px);
  transition: background 130ms ease;
}
.hero-card-cta:hover { background: rgba(255,255,255,0.28); }

/* Compact feed list */
.compact-feed {
  flex: 1;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.compact-feed::-webkit-scrollbar { display: none; }

.feed-row {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 8px 6px;
  border-radius: 8px;
  text-align: left;
  cursor: pointer;
  border-bottom: 1px solid var(--border-soft);
  transition: background 110ms ease, transform 110ms ease;
}
.feed-row:last-child { border-bottom: none; }
.feed-row:hover { background: var(--paper); transform: translateX(2px); }

.feed-row-dot {
  width: 7px;
  height: 7px;
  border-radius: 99px;
  flex-shrink: 0;
}
.feed-row-body { flex: 1; min-width: 0; }
.feed-row-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.feed-row-time {
  font-size: 10px;
  color: var(--ink-muted);
  margin-top: 1px;
}
.feed-row-action {
  font-size: 9.5px;
  font-weight: 700;
  flex-shrink: 0;
}
</style>

