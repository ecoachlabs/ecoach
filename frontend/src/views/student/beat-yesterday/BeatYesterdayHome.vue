<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import ComparisonCard from '@/components/viz/ComparisonCard.vue'
import StreakCounter from '@/components/viz/StreakCounter.vue'
import MicroGainIndicator from '@/components/modes/beat-yesterday/MicroGainIndicator.vue'
import DailyClimbSession from '@/components/modes/beat-yesterday/DailyClimbSession.vue'
import GrowthBadges from '@/components/modes/beat-yesterday/GrowthBadges.vue'
import WeeklyTrends from '@/components/modes/beat-yesterday/WeeklyTrends.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const showTrends = ref(false)
const showBadges = ref(false)

const yesterday = ref({ attempted: 12, correct: 8, avgTime: 34 })
const targets = ref({ attempted: 14, correct: 10, avgTime: 31 })
const today = ref({ attempted: 0, correct: 0, avgTime: 0 })
const streak = ref(12)
const growthMode = ref('accuracy')

const blocks = [
  { name: 'Warm Start', duration: '2 min', icon: '☀', questionCount: 3 },
  { name: 'Core Climb', duration: '5 min', icon: '△', questionCount: 7 },
  { name: 'Speed Burst', duration: '1 min', icon: '⚡', questionCount: 5 },
  { name: 'Finish Strong', duration: '1 min', icon: '★', questionCount: 3 },
]

const weekData = [
  { day: 'Mon', attempted: 14, correct: 10, avgTime: 32 },
  { day: 'Tue', attempted: 12, correct: 9, avgTime: 28 },
  { day: 'Wed', attempted: 15, correct: 12, avgTime: 30 },
  { day: 'Thu', attempted: 13, correct: 10, avgTime: 29 },
  { day: 'Fri', attempted: 12, correct: 8, avgTime: 34 },
]

const badges = [
  { name: '7-Day Streak', icon: '🔥', earned: true, requirement: '7 days' },
  { name: 'Speed Demon', icon: '⚡', earned: true, requirement: 'Sub-25s avg' },
  { name: 'Perfect Day', icon: '★', earned: false, requirement: '100% accuracy' },
  { name: '30-Day Warrior', icon: '🏅', earned: false, requirement: '30 day streak' },
]

const modeLabels: Record<string, string> = {
  volume: 'Volume Day', accuracy: 'Accuracy Day',
  speed: 'Speed Day', mixed: 'Mixed Day', recovery: 'Recovery Day',
}

onMounted(async () => {
  if (!auth.currentAccount) return
  try { truth.value = await getLearnerTruth(auth.currentAccount.id) } catch {}
  loading.value = false
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Beat Yesterday</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Small gains. Big transformation.
        </h1>
      </div>
      <div class="flex items-center gap-4">
        <div class="px-3 py-1.5 rounded-lg text-xs font-semibold"
          :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-secondary)', border: '1px solid var(--border-soft)' }">
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
      <div class="flex-1 overflow-hidden flex">

        <!-- Left: comparison + session -->
        <div class="flex-1 overflow-y-auto p-6 space-y-5">
          <ComparisonCard
            left-label="Yesterday"
            right-label="Today's Target"
            :left-value="yesterday.correct + '/' + yesterday.attempted"
            :right-value="targets.correct + '/' + targets.attempted"
            :left-subtext="yesterday.avgTime + 's avg'"
            :right-subtext="targets.avgTime + 's avg'"
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
              @click="router.push('/student/session/beat-yesterday')"
            >Start Today's Climb →</button>
            <p class="text-xs text-center mt-2" :style="{ color: 'var(--ink-muted)' }">
              ~9 minutes · {{ targets.attempted }} questions
            </p>
          </div>
        </div>

        <!-- Right: weekly + badges -->
        <div
          class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
        >
          <!-- Weekly bars -->
          <div class="px-5 pt-5 pb-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
            <p class="section-label mb-4">This Week</p>
            <div class="flex items-end justify-between gap-2" style="height: 64px;">
              <div v-for="day in weekData" :key="day.day" class="flex-1 flex flex-col items-center gap-1">
                <div class="flex-1 w-full flex flex-col justify-end">
                  <div class="w-full rounded-t-sm"
                    :style="{ height: (day.correct / 15 * 100) + '%', minHeight: '4px', backgroundColor: 'var(--accent-glow)', border: '1px solid var(--accent)' }" />
                </div>
                <span class="text-[9px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ day.day }}</span>
              </div>
            </div>
          </div>

          <!-- Badges -->
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
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
        <WeeklyTrends v-if="showTrends" :week-data="weekData" :weekly-gains="{ volume: 3, accuracy: 2, speed: -2 }" class="mb-4" />
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
  border: 1px solid var(--border-soft);
  background: var(--paper);
}
.badge-card.earned {
  background: var(--surface);
  border-color: var(--border-soft);
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
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.toggle-btn.on {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
