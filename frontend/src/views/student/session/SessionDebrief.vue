<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { completeSession } from '@/ipc/sessions'
import { PhCheckCircle, PhXCircle, PhArrowRight, PhHouse, PhPencilSimple, PhMagnifyingGlass, PhChartBar } from '@phosphor-icons/vue'

const route = useRoute()
const router = useRouter()
const sessionId = computed(() => Number(route.params.id))
const loading = ref(true)
const summary = ref<any>(null)

onMounted(async () => {
  try {
    summary.value = await completeSession(sessionId.value)
  } catch (e) {
    console.error('Failed to load summary:', e)
  }
  loading.value = false
})

const accuracyPct = computed(() => {
  if (!summary.value) return 0
  return Math.round((summary.value.accuracy_score ?? 0) / 100)
})

const wrong = computed(() =>
  (summary.value?.answered_questions ?? 0) - (summary.value?.correct_questions ?? 0)
)

const grade = computed(() => {
  const pct = accuracyPct.value
  if (pct >= 90) return { label: 'Excellent', color: 'var(--accent)' }
  if (pct >= 75) return { label: 'Good', color: 'var(--accent)' }
  if (pct >= 60) return { label: 'Pass', color: 'var(--gold)' }
  if (pct >= 45) return { label: 'Borderline', color: 'var(--warm)' }
  return { label: 'Needs Work', color: 'var(--warm)' }
})

const scoreColor = computed(() => {
  const v = summary.value?.accuracy_score ?? 0
  if (v >= 7000) return 'var(--accent)'
  if (v >= 4000) return 'var(--gold)'
  return 'var(--warm)'
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="w-8 h-8 border-2 rounded-full animate-spin"
        :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
    </div>

    <!-- Results -->
    <template v-else-if="summary">

      <!-- Score hero -->
      <div
        class="flex-shrink-0 flex flex-col items-center justify-center py-14 px-8 border-b"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <p class="eyebrow mb-8">Session Complete</p>

        <!-- Big score ring (CSS only, no dependency) -->
        <div class="score-ring-wrap">
          <svg width="140" height="140" viewBox="0 0 140 140">
            <circle cx="70" cy="70" r="60" fill="none" stroke="var(--border-soft)" stroke-width="8" />
            <circle
              cx="70" cy="70" r="60" fill="none"
              :stroke="scoreColor"
              stroke-width="8"
              stroke-linecap="round"
              :stroke-dasharray="`${2 * Math.PI * 60}`"
              :stroke-dashoffset="`${2 * Math.PI * 60 * (1 - accuracyPct / 100)}`"
              transform="rotate(-90 70 70)"
              style="transition: stroke-dashoffset 800ms ease"
            />
          </svg>
          <div class="score-ring-center">
            <p class="font-display text-4xl font-black" :style="{ color: scoreColor }">{{ accuracyPct }}%</p>
            <p class="text-xs font-bold" :style="{ color: grade.color }">{{ grade.label }}</p>
          </div>
        </div>

        <p class="text-sm mt-5" :style="{ color: 'var(--ink-secondary)' }">
          {{ summary.correct_questions }}/{{ summary.answered_questions }} correct
        </p>
      </div>

      <!-- Stat strip -->
      <div
        class="flex-shrink-0 grid grid-cols-3 divide-x border-b"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ summary.answered_questions }}</p>
          <p class="stat-lbl">Answered</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ summary.correct_questions }}</p>
          <p class="stat-lbl">Correct</p>
        </div>
        <div class="stat-cell">
          <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ wrong }}</p>
          <p class="stat-lbl">Wrong</p>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex-1 flex flex-col items-center justify-center gap-4 p-8">
        <p class="text-sm mb-2" :style="{ color: 'var(--ink-muted)' }">What would you like to do next?</p>
        <div class="grid grid-cols-2 gap-3 w-full max-w-sm">
          <button class="action-tile primary" @click="router.push('/student')">
            <PhHouse :size="20" weight="duotone" />
            <span>Home</span>
          </button>
          <button class="action-tile" @click="router.push('/student/practice')">
            <PhPencilSimple :size="20" weight="duotone" />
            <span>Practice Again</span>
          </button>
          <button v-if="wrong > 0" class="action-tile" @click="router.push('/student/mistakes')">
            <PhMagnifyingGlass :size="20" weight="duotone" />
            <span>Review Mistakes</span>
          </button>
          <button class="action-tile" @click="router.push('/student/progress')">
            <PhChartBar :size="20" weight="duotone" />
            <span>My Progress</span>
          </button>
        </div>
      </div>
    </template>

    <!-- Error -->
    <div v-else class="flex-1 flex flex-col items-center justify-center gap-4 p-8">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Could not load session results.</p>
      <button class="px-5 py-2.5 rounded-xl font-semibold text-sm"
        :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
        @click="router.push('/student')">Back to Home</button>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
}

.score-ring-wrap {
  position: relative;
  width: 140px;
  height: 140px;
}
.score-ring-center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
}

.stat-cell {
  padding: 18px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.stat-big {
  font-size: 26px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}
.stat-lbl {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-muted);
}

.action-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px 16px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  border: 1px solid var(--border-soft);
  background: var(--surface);
  color: var(--ink);
  transition: border-color 120ms ease, background-color 120ms ease, transform 120ms ease;
}
.action-tile:hover {
  border-color: var(--ink);
  transform: translateY(-2px);
}
.action-tile.primary {
  background: var(--ink);
  color: var(--paper);
  border-color: transparent;
}
.action-tile.primary:hover { opacity: 0.88; }
</style>
