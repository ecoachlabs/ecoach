<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  generateDailyClimbTarget,
  getBeatYesterdayDashboard,
  getPriorityTopics,
  listClimbTrend,
  listSubjects,
  type BeatYesterdayDashboardDto,
  type BeatYesterdayDailySummaryDto,
  type BeatYesterdayDailyTargetDto,
  type ClimbTrendPointDto,
} from '@/ipc/coach'
import ComparisonCard from '@/components/viz/ComparisonCard.vue'
import StreakCounter from '@/components/viz/StreakCounter.vue'
import MicroGainIndicator from '@/components/modes/beat-yesterday/MicroGainIndicator.vue'
import DailyClimbSession from '@/components/modes/beat-yesterday/DailyClimbSession.vue'
import GrowthBadges from '@/components/modes/beat-yesterday/GrowthBadges.vue'
import WeeklyTrends from '@/components/modes/beat-yesterday/WeeklyTrends.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const error = ref('')
const showTrends = ref(false)
const showBadges = ref(false)
const selectedSubjectId = ref<number | null>(null)
const selectedSubjectName = ref('Focused Subject')
const dashboard = ref<BeatYesterdayDashboardDto | null>(null)
const trend = ref<ClimbTrendPointDto[]>([])

const modeLabels: Record<string, string> = {
  volume: 'Volume Day',
  accuracy: 'Accuracy Day',
  speed: 'Speed Day',
  mixed: 'Mixed Day',
  recovery: 'Recovery Day',
}

const todayLabel = computed(() => localDateIso(new Date()))
const target = computed<BeatYesterdayDailyTargetDto | null>(() => dashboard.value?.target ?? null)
const growthMode = computed(() => target.value?.mode ?? dashboard.value?.profile.current_mode ?? 'mixed')
const streak = computed(() => dashboard.value?.profile.streak_days ?? 0)

const yesterday = computed(() => summarizeDaily(dashboard.value?.previous_summary))
const today = computed(() => {
  const latest = dashboard.value?.latest_summary
  if (!latest || latest.summary_date !== todayLabel.value) {
    return { attempted: 0, correct: 0, avgTime: 0 }
  }
  return summarizeDaily(latest)
})

const targets = computed(() => ({
  attempted: target.value?.target_attempts ?? 0,
  correct: target.value?.target_correct ?? 0,
  avgTime: secondsFromMs(target.value?.target_avg_response_time_ms),
}))

const blocks = computed(() => {
  const attemptSplit = splitAttempts(target.value?.target_attempts ?? 0)
  return [
    {
      name: 'Warm Start',
      duration: formatMinutes(target.value?.warm_start_minutes ?? 0),
      icon: 'W',
      questionCount: attemptSplit[0],
    },
    {
      name: 'Core Climb',
      duration: formatMinutes(target.value?.core_climb_minutes ?? 0),
      icon: 'C',
      questionCount: attemptSplit[1],
    },
    {
      name: 'Speed Burst',
      duration: formatMinutes(target.value?.speed_burst_minutes ?? 0),
      icon: 'S',
      questionCount: attemptSplit[2],
    },
    {
      name: 'Finish Strong',
      duration: formatMinutes(target.value?.finish_strong_minutes ?? 0),
      icon: 'F',
      questionCount: attemptSplit[3],
    },
  ]
})

const weekData = computed(() =>
  trend.value
    .slice()
    .sort((left, right) => left.summary_date.localeCompare(right.summary_date))
    .map(point => ({
      day: new Date(`${point.summary_date}T00:00:00`).toLocaleDateString('en-US', {
        weekday: 'short',
      }),
      attempted: point.actual_attempts,
      correct: point.actual_correct,
      avgTime: secondsFromMs(point.actual_avg_response_time_ms),
    })),
)

const weekBarScale = computed(() =>
  Math.max(1, ...weekData.value.map(day => day.correct), targets.value.correct, yesterday.value.correct),
)

const weeklyGains = computed(() => ({
  volume: today.value.attempted - yesterday.value.attempted,
  accuracy: accuracyPercent(today.value) - accuracyPercent(yesterday.value),
  speed:
    today.value.avgTime > 0 && yesterday.value.avgTime > 0
      ? yesterday.value.avgTime - today.value.avgTime
      : 0,
}))

const badges = computed(() => [
  {
    name: '7-Day Streak',
    icon: '7D',
    earned: streak.value >= 7,
    requirement: '7 day streak',
  },
  {
    name: 'Accuracy Gain',
    icon: 'ACC',
    earned: weeklyGains.value.accuracy > 0,
    requirement: 'Improve accuracy',
  },
  {
    name: 'Fast Recall',
    icon: 'SPD',
    earned:
      targets.value.avgTime > 0 &&
      today.value.avgTime > 0 &&
      today.value.avgTime <= targets.value.avgTime,
    requirement: 'Beat pace target',
  },
  {
    name: '30-Day Warrior',
    icon: '30D',
    earned: streak.value >= 30,
    requirement: '30 day streak',
  },
])

const targetMinutes = computed(
  () =>
    (target.value?.warm_start_minutes ?? 0) +
    (target.value?.core_climb_minutes ?? 0) +
    (target.value?.speed_burst_minutes ?? 0) +
    (target.value?.finish_strong_minutes ?? 0),
)

const targetNote = computed(() => extractRationaleLine(target.value?.rationale))

onMounted(() => {
  void loadBeatYesterday()
})

async function loadBeatYesterday() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    const studentId = auth.currentAccount.id
    const [priorityTopics, subjects] = await Promise.all([
      getPriorityTopics(studentId, 1),
      listSubjects(1),
    ])

    const focusedSubject =
      subjects.find(subject => subject.code === priorityTopics[0]?.subject_code) ??
      subjects[0] ??
      null

    if (!focusedSubject) {
      dashboard.value = null
      trend.value = []
      selectedSubjectId.value = null
      selectedSubjectName.value = 'No subject linked yet'
      return
    }

    selectedSubjectId.value = focusedSubject.id
    selectedSubjectName.value = focusedSubject.name

    let nextDashboard = await getBeatYesterdayDashboard(studentId, focusedSubject.id, todayLabel.value)
    if (!nextDashboard.target) {
      const generatedTarget = await generateDailyClimbTarget(studentId, focusedSubject.id, todayLabel.value)
      nextDashboard = {
        ...nextDashboard,
        target: generatedTarget,
      }
    }

    const nextTrend = await listClimbTrend(studentId, focusedSubject.id, 7)
    dashboard.value = nextDashboard
    trend.value = nextTrend
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load Beat Yesterday'
  } finally {
    loading.value = false
  }
}

function launchDailyClimb() {
  const query: Record<string, string> = { mode: 'beat-yesterday' }
  if (selectedSubjectId.value != null) {
    query.subjectId = String(selectedSubjectId.value)
  }
  if (target.value?.focus_topic_ids[0] != null) {
    query.topicId = String(target.value.focus_topic_ids[0])
  }
  if (target.value?.id != null) {
    query.targetId = String(target.value.id)
  }
  if (target.value?.target_attempts != null) {
    query.questionCount = String(target.value.target_attempts)
  }
  void router.push({ name: 'custom-test', query })
}

function summarizeDaily(summary?: BeatYesterdayDailySummaryDto | null) {
  return {
    attempted: summary?.actual_attempts ?? 0,
    correct: summary?.actual_correct ?? 0,
    avgTime: secondsFromMs(summary?.actual_avg_response_time_ms),
  }
}

function secondsFromMs(value?: number | null) {
  return value == null ? 0 : Math.round(value / 1000)
}

function formatMinutes(value: number) {
  return `${value} min`
}

function localDateIso(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function accuracyPercent(metrics: { attempted: number; correct: number }) {
  if (metrics.attempted <= 0) return 0
  return Math.round((metrics.correct / metrics.attempted) * 100)
}

function splitAttempts(total: number) {
  if (total <= 0) return [0, 0, 0, 0]
  const weights = [0.2, 0.45, 0.2, 0.15]
  const buckets = weights.map(weight => Math.floor(total * weight))
  let assigned = buckets.reduce((sum, value) => sum + value, 0)
  let cursor = 0
  while (assigned < total) {
    buckets[cursor % buckets.length] += 1
    assigned += 1
    cursor += 1
  }
  return buckets
}

function extractRationaleLine(value: unknown) {
  if (Array.isArray(value)) {
    const firstLine = value.find(entry => typeof entry === 'string' && entry.trim().length > 0)
    return typeof firstLine === 'string' ? firstLine : 'Built from your latest performance pattern.'
  }

  if (value && typeof value === 'object') {
    const rationale = (value as Record<string, unknown>).rationale
    if (Array.isArray(rationale)) {
      const firstLine = rationale.find(entry => typeof entry === 'string' && entry.trim().length > 0)
      if (typeof firstLine === 'string') return firstLine
    }
  }

  return 'Built from your latest performance pattern.'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Beat Yesterday</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Small gains. Big transformation.
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          {{ selectedSubjectName }}
        </p>
      </div>
      <div class="flex items-center gap-4">
        <div class="px-3 py-1.5 rounded-lg text-xs font-semibold"
          :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-secondary)', border: '1px solid transparent' }">
          {{ modeLabels[growthMode] }}
        </div>
        <StreakCounter :count="streak" label="Day Streak" animated />
      </div>
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 3" :key="i" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else>
      <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
        :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
        {{ error }}
      </div>

      <div class="flex-1 overflow-hidden flex">
        <div class="flex-1 overflow-y-auto p-6 space-y-5">
          <ComparisonCard
            left-label="Yesterday"
            right-label="Today's Target"
            :left-value="`${yesterday.correct}/${yesterday.attempted}`"
            :right-value="`${targets.correct}/${targets.attempted}`"
            :left-subtext="`${yesterday.avgTime}s avg`"
            :right-subtext="`${targets.avgTime}s avg`"
            highlight="right"
          />

          <div v-if="today.attempted > 0" class="flex items-center gap-4">
            <MicroGainIndicator type="volume" :value="today.attempted - yesterday.attempted" />
            <MicroGainIndicator type="accuracy" :value="today.correct - yesterday.correct" />
            <MicroGainIndicator type="speed" :value="yesterday.avgTime - today.avgTime" unit="s" />
          </div>

          <DailyClimbSession :current-block="0" :blocks="blocks" />

          <div class="pt-2">
            <button
              class="w-full py-3.5 rounded-2xl font-bold text-sm"
              :style="{ backgroundColor: 'var(--ink)', color: 'var(--paper)' }"
              @click="launchDailyClimb"
            >Start Today's Climb</button>
            <p class="text-xs text-center mt-2" :style="{ color: 'var(--ink-muted)' }">
              {{ targetMinutes }} minutes · {{ targets.attempted }} questions
            </p>
            <p class="text-[11px] text-center mt-1" :style="{ color: 'var(--ink-muted)' }">
              {{ targetNote }}
            </p>
          </div>
        </div>

        <div
          class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <div class="px-5 pt-5 pb-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
            <p class="section-label mb-4">This Week</p>
            <div class="flex items-end justify-between gap-2" style="height: 64px;">
              <div v-for="day in weekData" :key="day.day" class="flex-1 flex flex-col items-center gap-1">
                <div class="flex-1 w-full flex flex-col justify-end">
                  <div class="w-full rounded-t-sm"
                    :style="{ height: `${(day.correct / weekBarScale) * 100}%`, minHeight: '4px', backgroundColor: 'var(--accent-glow)', border: '1px solid var(--accent)' }" />
                </div>
                <span class="text-[9px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ day.day }}</span>
              </div>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto p-4">
            <p class="section-label mb-3">Badges</p>
            <div class="grid grid-cols-2 gap-2">
              <div
                v-for="badge in badges"
                :key="badge.name"
                class="badge-card"
                :class="badge.earned ? 'earned' : 'locked'"
              >
                <span class="text-xl mb-1">{{ badge.icon }}</span>
                <span class="text-[10px] font-bold" :style="{ color: 'var(--ink)' }">{{ badge.name }}</span>
                <span class="text-[9px]" :style="{ color: 'var(--ink-muted)' }">{{ badge.requirement }}</span>
              </div>
            </div>
          </div>

          <div class="p-4 border-t flex gap-2" :style="{ borderColor: 'var(--border-soft)' }">
            <button class="toggle-btn flex-1" :class="{ on: showTrends }"
              @click="showTrends = !showTrends">Trends</button>
            <button class="toggle-btn flex-1" :class="{ on: showBadges }"
              @click="showBadges = !showBadges">Badges</button>
          </div>
        </div>
      </div>

      <div v-if="showTrends || showBadges"
        class="flex-shrink-0 border-t p-6"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
        <WeeklyTrends v-if="showTrends" :week-data="weekData" :weekly-gains="weeklyGains" class="mb-4" />
        <GrowthBadges v-if="showBadges" :streak-days="streak" :badges="badges" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.badge-card {
  border-radius: 12px;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 2px;
  border: 1px solid transparent;
  background: var(--paper);
}

.badge-card.earned {
  background: var(--surface);
  border-color: transparent;
}

.badge-card.locked {
  opacity: 0.4;
}

.toggle-btn {
  padding: 5px 8px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}

.toggle-btn.on {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
