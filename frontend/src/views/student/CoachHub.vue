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

      <!-- Right: quick test + live feed -->
      <div class="w-[22rem] flex-shrink-0 flex flex-col gap-4 overflow-hidden px-4 py-5"
        :style="{ backgroundColor: 'var(--surface)' }">
        <div class="quick-test-card rounded-2xl p-4" :style="{ backgroundColor: 'var(--paper)' }">
          <div class="flex items-center justify-between gap-2">
            <p class="section-label">Quick Test</p>
            <span class="flex items-center gap-1 text-[10px] font-bold px-2 py-1 rounded-full"
              :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">
              <PhClockCountdown :size="10" weight="bold" />
              {{ quickStepLabel }}
            </span>
          </div>

          <div class="mt-3 flex items-start gap-3">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0"
              :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">
              <PhTarget :size="15" weight="duotone" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-[10px] uppercase tracking-[0.14em] font-bold" :style="{ color: 'var(--ink-muted)' }">
                {{ activeQuickCheck.topic }}
              </p>
              <p class="text-[13px] font-semibold leading-relaxed mt-1" :style="{ color: 'var(--ink)' }">
                {{ activeQuickCheck.prompt }}
              </p>
            </div>
          </div>

          <div class="mt-3 space-y-2">
            <button
              v-for="option in activeQuickCheck.options"
              :key="option.id"
              class="quick-option w-full text-left px-3 py-2 rounded-xl text-[12px] font-semibold"
              :style="{
                backgroundColor:
                  selectedQuickOptionId === option.id
                    ? (option.correct ? 'rgba(22,163,74,0.11)' : 'rgba(194,65,12,0.11)')
                    : (quickAnswered && option.correct ? 'rgba(22,163,74,0.08)' : 'var(--surface)'),
                color: selectedQuickOptionId === option.id && !option.correct ? 'var(--warm)' : 'var(--ink)',
                boxShadow:
                  selectedQuickOptionId === option.id
                    ? (option.correct ? 'inset 0 0 0 1px rgba(22,163,74,0.35)' : 'inset 0 0 0 1px rgba(194,65,12,0.35)')
                    : (quickAnswered && option.correct ? 'inset 0 0 0 1px rgba(22,163,74,0.25)' : 'none'),
                opacity: quickAnswered && selectedQuickOptionId !== option.id && !option.correct ? 0.8 : 1,
              }"
              :disabled="quickAnswered"
              @click="pickQuickOption(option.id)"
            >
              {{ option.label }}
            </button>
          </div>

          <div v-if="quickAnswered" class="mt-3 p-3 rounded-xl" :style="{ backgroundColor: 'var(--surface)' }">
            <div class="flex items-start gap-2">
              <PhBookOpen :size="14" weight="duotone" :style="{ color: quickSelectionCorrect ? '#16a34a' : 'var(--warm)' }" />
              <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">
                {{ activeQuickCheck.explanation }}
              </p>
            </div>
            <div class="mt-3 flex items-center gap-2">
              <button
                class="mini-btn px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider"
                :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }"
                @click="openQuickCheckTopic"
              >
                Review Topic
              </button>
              <button
                class="mini-btn px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider"
                :style="{ backgroundColor: 'var(--ink)', color: 'white' }"
                @click="nextQuickCheck"
              >
                Next Question
              </button>
            </div>
          </div>
        </div>

        <div class="feed-panel min-h-0 flex-1 rounded-2xl p-4 flex flex-col"
          :style="{ backgroundColor: 'var(--paper)' }">
          <div class="flex items-center justify-between">
            <p class="section-label">Recent Activity</p>
            <span class="text-[10px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ liveFeed.length }} updates</span>
          </div>

          <div class="feed-list mt-3 flex-1 overflow-y-auto pr-1 space-y-2.5">
            <button
              v-for="item in liveFeed"
              :key="item.id"
              class="feed-item w-full text-left rounded-xl p-3 flex items-start gap-3"
              :style="{ backgroundColor: 'var(--surface)' }"
              @click="router.push(item.to)"
            >
              <div class="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0"
                :style="{ backgroundColor: `${feedToneColor(item.tone)}1A`, color: feedToneColor(item.tone) }">
                <PhTrendUp v-if="item.tone === 'accent'" :size="14" weight="duotone" />
                <PhFlame v-else-if="item.tone === 'warm'" :size="14" weight="duotone" />
                <PhLightning v-else-if="item.tone === 'gold'" :size="14" weight="duotone" />
                <PhBookOpen v-else :size="14" weight="duotone" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[12px] font-semibold leading-tight" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
                <p class="text-[10px] mt-1 leading-relaxed" :style="{ color: 'var(--ink-muted)' }">{{ item.detail }}</p>
                <p class="text-[9px] mt-1.5 flex items-center gap-1" :style="{ color: 'var(--ink-muted)' }">
                  <PhClockCountdown :size="10" weight="bold" />
                  {{ item.time }}
                </p>
              </div>
              <span class="text-[9px] font-bold uppercase tracking-wider px-2 py-1 rounded-md flex-shrink-0"
                :style="{ backgroundColor: `${feedToneColor(item.tone)}14`, color: feedToneColor(item.tone) }">
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

.quick-test-card,
.feed-panel {
  box-shadow: 0 10px 28px rgba(15, 23, 42, 0.05);
}

.quick-option {
  cursor: pointer;
  transition: transform 110ms ease, background-color 110ms ease, box-shadow 110ms ease, opacity 110ms ease;
}
.quick-option:hover:enabled {
  transform: translateX(2px);
}
.quick-option:disabled {
  cursor: default;
}

.mini-btn {
  transition: transform 110ms ease, opacity 110ms ease;
}
.mini-btn:hover {
  transform: translateY(-1px);
  opacity: 0.9;
}

.feed-item {
  transition: transform 110ms ease, background-color 110ms ease;
}
.feed-item:hover {
  transform: translateX(2px);
  background-color: var(--paper) !important;
}

.feed-list {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.feed-list::-webkit-scrollbar {
  width: 0;
  height: 0;
}
</style>

