<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { listStudentActivityHistory, type StudentActivityHistoryItemDto } from '@/ipc/coach'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const activeFilter = ref<'all' | 'practice' | 'mock' | 'diagnostic' | 'games' | 'elite'>('all')
const loading = ref(true)
const error = ref('')
const allSessions = ref<StudentActivityHistoryItemDto[]>([])

const filters = [
  { key: 'all', label: 'All' },
  { key: 'practice', label: 'Practice' },
  { key: 'mock', label: 'Mock' },
  { key: 'diagnostic', label: 'Diagnostic' },
  { key: 'games', label: 'Games' },
  { key: 'elite', label: 'Elite' },
] as const

const filtered = computed(() =>
  activeFilter.value === 'all'
    ? allSessions.value
    : allSessions.value.filter(session => session.type_key === activeFilter.value),
)

function scoreColor(score: number) {
  if (score >= 80) return 'var(--accent)'
  if (score >= 60) return 'var(--gold)'
  return 'var(--warm)'
}

function parseOccurredAt(value: string): number | null {
  if (!value) return null
  const normalized = value.includes('T') ? value : `${value.replace(' ', 'T')}Z`
  const parsed = Date.parse(normalized)
  return Number.isNaN(parsed) ? null : parsed
}

function timeAgo(occurredAt: string): string {
  const timestamp = parseOccurredAt(occurredAt)
  if (timestamp == null) return 'Unknown'

  const diffMinutes = Math.max(0, Math.round((Date.now() - timestamp) / 60000))
  if (diffMinutes < 60) return `${diffMinutes}m ago`
  const hours = Math.floor(diffMinutes / 60)
  if (hours < 24) return `${hours}h ago`
  return `${Math.floor(hours / 24)}d ago`
}

const stats = computed(() => {
  const sessions = filtered.value
  if (sessions.length === 0) {
    return { avg: 0, best: 0, total: 0, count: 0 }
  }

  const avg = Math.round(sessions.reduce((sum, session) => sum + session.score, 0) / sessions.length)
  const best = Math.max(...sessions.map(session => session.score))
  const total = sessions.reduce((sum, session) => sum + session.answered_questions, 0)
  return { avg, best, total, count: sessions.length }
})

async function loadHistory() {
  const studentId = auth.currentAccount?.id
  if (!studentId) {
    error.value = 'No active student account.'
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''
  try {
    allSessions.value = await listStudentActivityHistory(studentId, 48)
  } catch (err: any) {
    error.value = typeof err === 'string' ? err : err?.message ?? 'Failed to load activity history.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadHistory()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
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
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Mastery Map</button>
      </div>
    </div>

    <div class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6">
        <div class="flex gap-1.5 mb-5">
          <button
            v-for="filter in filters"
            :key="filter.key"
            class="filter-chip"
            :class="{ active: activeFilter === filter.key }"
            @click="activeFilter = filter.key"
          >{{ filter.label }}</button>
        </div>

        <div v-if="loading" class="py-12 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Loading real activity...</p>
        </div>

        <div v-else-if="error" class="py-12 text-center">
          <p class="text-sm mb-3" :style="{ color: 'var(--warm)' }">{{ error }}</p>
          <button class="nav-pill" @click="loadHistory">Retry</button>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="session in filtered"
            :key="`${session.type_key}-${session.id}`"
            class="session-row flex items-center gap-4 px-5 py-4 rounded-2xl border"
            :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
          >
            <div class="score-ring flex-shrink-0" :style="{ color: scoreColor(session.score) }">
              <svg width="44" height="44" viewBox="0 0 44 44">
                <circle cx="22" cy="22" r="18" fill="none" stroke="var(--border-soft)" stroke-width="3" />
                <circle
                  cx="22"
                  cy="22"
                  r="18"
                  fill="none"
                  :stroke="scoreColor(session.score)"
                  stroke-width="3"
                  :stroke-dasharray="`${2 * Math.PI * 18}`"
                  :stroke-dashoffset="`${2 * Math.PI * 18 * (1 - session.score / 100)}`"
                  stroke-linecap="round"
                  transform="rotate(-90 22 22)"
                />
                <text
                  x="22"
                  y="26"
                  text-anchor="middle"
                  font-size="9"
                  font-weight="700"
                  :fill="scoreColor(session.score)"
                >{{ session.score }}%</text>
              </svg>
            </div>

            <div class="flex-1 min-w-0">
              <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ session.label }}</p>
              <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                {{ session.subject }} · {{ session.answered_questions }}/{{ session.total_questions }} attempted
              </p>
            </div>

            <div class="text-right flex-shrink-0">
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ timeAgo(session.occurred_at) }}</p>
              <span class="type-badge mt-1" :class="session.type_key">{{ session.type_key }}</span>
            </div>
          </div>
        </div>

        <div v-if="!loading && !error && filtered.length === 0" class="py-12 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No real sessions found yet.</p>
        </div>
      </div>

      <div
        class="w-64 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Recent Summary</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ stats.count }}</p>
            <p class="stat-label">Last 48 Sessions</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: scoreColor(stats.avg) }">{{ stats.avg }}%</p>
            <p class="stat-label">Recent Avg</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ stats.best }}%</p>
            <p class="stat-label">Recent Best</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ stats.total }}</p>
            <p class="stat-label">Recent Questions</p>
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
  cursor: default;
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
.type-badge.games { background: rgba(15,23,42,0.08); color: var(--ink); }
.type-badge.elite { background: rgba(202,138,4,0.12); color: var(--gold); }

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
