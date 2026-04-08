<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const activeFilter = ref<'all' | 'practice' | 'mock' | 'diagnostic'>('all')

const filters = [
  { key: 'all', label: 'All' },
  { key: 'practice', label: 'Practice' },
  { key: 'mock', label: 'Mock' },
  { key: 'diagnostic', label: 'Diagnostic' },
] as const

// Placeholder history data — would be replaced with real IPC call
const allSessions = [
  { id: 1, type: 'practice', label: 'Practice Session', subject: 'Mathematics', score: 82, questions: 15, minutesAgo: 60 * 2 },
  { id: 2, type: 'mock', label: 'Mini Mock', subject: 'English', score: 74, questions: 20, minutesAgo: 60 * 26 },
  { id: 3, type: 'practice', label: 'Practice Session', subject: 'Science', score: 91, questions: 12, minutesAgo: 60 * 50 },
  { id: 4, type: 'diagnostic', label: 'Diagnostic', subject: 'Mathematics', score: 68, questions: 30, minutesAgo: 60 * 75 },
  { id: 5, type: 'mock', label: 'Full Mock', subject: 'All Subjects', score: 79, questions: 40, minutesAgo: 60 * 100 },
  { id: 6, type: 'practice', label: 'Practice Session', subject: 'English', score: 88, questions: 10, minutesAgo: 60 * 148 },
  { id: 7, type: 'practice', label: 'Practice Session', subject: 'Mathematics', score: 55, questions: 15, minutesAgo: 60 * 172 },
  { id: 8, type: 'mock', label: 'Topic Mock', subject: 'Science', score: 83, questions: 20, minutesAgo: 60 * 220 },
]

const filtered = computed(() =>
  activeFilter.value === 'all' ? allSessions : allSessions.filter(s => s.type === activeFilter.value)
)

function scoreColor(score: number) {
  if (score >= 80) return 'var(--accent)'
  if (score >= 60) return 'var(--gold)'
  return 'var(--warm)'
}

function timeAgo(minutesAgo: number): string {
  if (minutesAgo < 60) return `${minutesAgo}m ago`
  const h = Math.floor(minutesAgo / 60)
  if (h < 24) return `${h}h ago`
  return `${Math.floor(h / 24)}d ago`
}

const stats = computed(() => {
  const sessions = allSessions
  const avg = Math.round(sessions.reduce((a, s) => a + s.score, 0) / sessions.length)
  const best = Math.max(...sessions.map(s => s.score))
  const total = sessions.reduce((a, s) => a + s.questions, 0)
  return { avg, best, total, count: sessions.length }
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Progress</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Activity History
        </h1>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress')">Overview</button>
        <button class="nav-pill" @click="router.push('/student/progress/analytics')">Analytics</button>
        <button class="nav-pill" @click="router.push('/student/progress/mastery')">Mastery Map</button>
      </div>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: session list -->
      <div class="flex-1 overflow-y-auto p-6">

        <!-- Filters -->
        <div class="flex gap-1.5 mb-5">
          <button
            v-for="f in filters"
            :key="f.key"
            class="filter-chip"
            :class="{ active: activeFilter === f.key }"
            @click="activeFilter = f.key"
          >{{ f.label }}</button>
        </div>

        <!-- Session rows -->
        <div class="space-y-2">
          <div
            v-for="session in filtered"
            :key="session.id"
            class="session-row flex items-center gap-4 px-5 py-4 rounded-2xl border"
            :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
          >
            <!-- Score ring -->
            <div class="score-ring flex-shrink-0" :style="{ color: scoreColor(session.score) }">
              <svg width="44" height="44" viewBox="0 0 44 44">
                <circle cx="22" cy="22" r="18" fill="none" stroke="var(--border-soft)" stroke-width="3" />
                <circle cx="22" cy="22" r="18" fill="none"
                  :stroke="scoreColor(session.score)"
                  stroke-width="3"
                  :stroke-dasharray="`${2 * Math.PI * 18}`"
                  :stroke-dashoffset="`${2 * Math.PI * 18 * (1 - session.score / 100)}`"
                  stroke-linecap="round"
                  transform="rotate(-90 22 22)" />
                <text x="22" y="26" text-anchor="middle" font-size="9" font-weight="700" :fill="scoreColor(session.score)">{{ session.score }}%</text>
              </svg>
            </div>

            <div class="flex-1 min-w-0">
              <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ session.label }}</p>
              <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                {{ session.subject }} · {{ session.questions }} questions
              </p>
            </div>

            <div class="text-right flex-shrink-0">
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ timeAgo(session.minutesAgo) }}</p>
              <span class="type-badge mt-1" :class="session.type">{{ session.type }}</span>
            </div>
          </div>
        </div>

        <div v-if="filtered.length === 0" class="py-12 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No sessions found.</p>
        </div>
      </div>

      <!-- Right: stats -->
      <div
        class="w-64 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Summary</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ stats.count }}</p>
            <p class="stat-label">Sessions</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: scoreColor(stats.avg) }">{{ stats.avg }}%</p>
            <p class="stat-label">Average Score</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ stats.best }}%</p>
            <p class="stat-label">Personal Best</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ stats.total }}</p>
            <p class="stat-label">Questions Answered</p>
          </div>
        </div>
      </div>
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
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.nav-pill {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.filter-chip {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}
.filter-chip:hover { color: var(--ink); border-color: var(--ink-muted); }
.filter-chip.active { background: var(--ink); color: var(--paper); border-color: var(--ink); }

.session-row {
  transition: background-color 100ms, transform 100ms;
  cursor: pointer;
}
.session-row:hover { background-color: var(--paper) !important; transform: translateX(2px); }

.type-badge {
  display: inline-block;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--border-soft);
  color: var(--ink-secondary);
}
.type-badge.mock { background: rgba(180,83,9,0.1); color: var(--gold); }
.type-badge.diagnostic { background: rgba(194,65,12,0.08); color: var(--warm); }
.type-badge.practice { background: rgba(13,148,136,0.08); color: var(--accent); }

.stat-card {
  padding: 16px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: var(--paper);
}
.stat-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
  margin-top: 4px;
}
</style>


