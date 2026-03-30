<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import ComparisonCard from '@/components/viz/ComparisonCard.vue'
import StreakCounter from '@/components/viz/StreakCounter.vue'
import MicroGainIndicator from '@/components/modes/beat-yesterday/MicroGainIndicator.vue'
import DailyClimbSession from '@/components/modes/beat-yesterday/DailyClimbSession.vue'
import GrowthBadges from '@/components/modes/beat-yesterday/GrowthBadges.vue'
import WeeklyTrends from '@/components/modes/beat-yesterday/WeeklyTrends.vue'
import AppCard from '@/components/ui/AppCard.vue'

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
  volume: 'Volume Day — attempt more questions',
  accuracy: 'Accuracy Day — get more right',
  speed: 'Speed Day — answer faster',
  mixed: 'Mixed Day — all dimensions',
  recovery: 'Recovery Day — gentle rebuild',
}

onMounted(async () => {
  if (!auth.currentAccount) return
  try { truth.value = await getLearnerTruth(auth.currentAccount.id) } catch {}
  loading.value = false
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <PageHeader title="Beat Yesterday" subtitle="Small daily gains compound into transformation." back-to="/student">
      <template #actions>
        <StreakCounter :count="streak" label="Day Streak" animated />
      </template>
    </PageHeader>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-24 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else>
      <!-- Yesterday vs Today -->
      <div class="mb-6">
        <ComparisonCard left-label="Yesterday" right-label="Today's Target"
          :left-value="yesterday.correct + '/' + yesterday.attempted"
          :right-value="targets.correct + '/' + targets.attempted"
          :left-subtext="yesterday.avgTime + 's avg'" :right-subtext="targets.avgTime + 's avg'"
          highlight="right" />
      </div>

      <!-- Growth Mode -->
      <div class="text-center mb-6">
        <AppBadge color="gold" size="md">{{ modeLabels[growthMode] }}</AppBadge>
      </div>

      <!-- Micro Gains (if any today) -->
      <div v-if="today.attempted > 0" class="flex items-center justify-center gap-3 mb-4">
        <MicroGainIndicator type="volume" :value="today.attempted - yesterday.attempted" />
        <MicroGainIndicator type="accuracy" :value="today.correct - yesterday.correct" />
        <MicroGainIndicator type="speed" :value="yesterday.avgTime - today.avgTime" unit="s" />
      </div>

      <!-- Session Blocks -->
      <div class="mb-6">
        <DailyClimbSession :current-block="0" :blocks="blocks" />
      </div>

      <!-- CTA -->
      <div class="text-center mb-6">
        <AppButton variant="warm" size="lg" @click="router.push('/student/session/beat-yesterday')">Start Today's Climb →</AppButton>
        <p class="text-xs mt-2" :style="{color:'var(--text-3)'}">~9 minutes · {{ targets.attempted }} questions</p>
      </div>

      <!-- Toggle sections -->
      <div class="flex gap-2 mb-4">
        <AppButton variant="secondary" size="sm" @click="showTrends = !showTrends">{{ showTrends ? 'Hide' : 'Show' }} Weekly Trends</AppButton>
        <AppButton variant="secondary" size="sm" @click="showBadges = !showBadges">{{ showBadges ? 'Hide' : 'Show' }} Badges</AppButton>
      </div>

      <WeeklyTrends v-if="showTrends" :week-data="weekData" :weekly-gains="{ volume: 3, accuracy: 2, speed: -2 }" class="mb-6" />
      <GrowthBadges v-if="showBadges" :streak-days="streak" :badges="badges" />
    </template>
  </div>
</template>
