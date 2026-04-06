<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import { listMockSessions, type MockSessionSummaryDto } from '@/ipc/mock'
import { PhClockCountdown, PhSquaresFour, PhTarget, PhLightning } from '@phosphor-icons/vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const history = ref<MockSessionSummaryDto[]>([])

const mockTypes = [
  {
    key: 'full',
    label: 'Full Mock',
    desc: 'Complete exam simulation with all sections and timing.',
    icon: PhSquaresFour,
    time: '2h 30m',
  },
  {
    key: 'topic',
    label: 'Topic Mock',
    desc: 'Focus on specific topics you need to strengthen.',
    icon: PhTarget,
    time: '30–45m',
  },
  {
    key: 'mini',
    label: 'Mini Mock',
    desc: 'Quick 20-question check across all topics.',
    icon: PhClockCountdown,
    time: '15–20m',
  },
  {
    key: 'pressure',
    label: 'Pressure Mock',
    desc: 'Tighter timing, harder questions, full exam pressure.',
    icon: PhLightning,
    time: '45m',
  },
]

const bandLabel: Record<string, string> = {
  strong: 'Exam Ready', developing: 'Developing', weak: 'Needs Work', critical: 'Critical',
}

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, sessions] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      listMockSessions(auth.currentAccount.id, 5),
    ])
    truth.value = t
    history.value = sessions
  } catch (e) {
    console.error('Failed to load mock home:', e)
  }
  loading.value = false
})

function goToSetup(mockType: string) {
  router.push(`/student/mock/setup?type=${mockType}`)
}

function gradeColor(grade: string | null): { bg: string; text: string } {
  if (!grade) return { bg: 'var(--paper)', text: 'var(--ink-muted)' }
  if (['A1', 'B2', 'B3'].includes(grade)) return { bg: 'rgba(13,148,136,0.1)', text: 'var(--accent)' }
  if (['C4', 'C5', 'C6'].includes(grade)) return { bg: 'rgba(180,83,9,0.1)', text: 'var(--gold)' }
  return { bg: 'rgba(194,65,12,0.1)', text: 'var(--warm)' }
}

function readinessColor() {
  const band = truth.value?.overall_readiness_band ?? 'developing'
  if (band === 'strong') return 'var(--accent)'
  if (band === 'developing') return 'var(--gold)'
  return 'var(--warm)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Mock Centre</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Simulate. Discover. Improve.
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Exam conditions. Real pressure. Honest scores.</p>
      </div>
      <div class="flex items-center gap-4">
        <div v-if="!loading && truth" class="text-right">
          <p class="text-lg font-black capitalize" :style="{ color: readinessColor() }">
            {{ bandLabel[truth.overall_readiness_band] ?? truth.overall_readiness_band }}
          </p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Current Band</p>
        </div>
        <button
          class="nav-pill"
          @click="router.push('/student/mock/history')"
        >View History</button>
      </div>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Mock type tiles -->
      <div class="flex-1 p-6 overflow-y-auto">
        <div v-if="loading" class="space-y-3">
          <div v-for="i in 4" :key="i" class="h-28 rounded-2xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div v-else class="space-y-3">
          <button
            v-for="mock in mockTypes"
            :key="mock.key"
            class="mock-row w-full text-left"
            @click="goToSetup(mock.key)"
          >
            <div class="mock-icon-box">
              <component :is="mock.icon" :size="20" weight="duotone" :style="{ color: 'var(--ink)' }" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-base font-bold" :style="{ color: 'var(--ink)' }">{{ mock.label }}</h3>
              <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ mock.desc }}</p>
            </div>
            <div class="flex-shrink-0 flex flex-col items-end gap-1">
              <span class="time-badge">{{ mock.time }}</span>
              <span class="go-label">Begin →</span>
            </div>
          </button>
        </div>

        <!-- Practice note -->
        <div class="mt-6 px-5 py-4 rounded-xl border" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="text-[11px] font-semibold mb-1" :style="{ color: 'var(--ink)' }">Before your mock</p>
          <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
            Run a diagnostic first to get accurate topic weights. Mocks adapt to your current readiness level.
          </p>
          <button class="mt-2 text-[11px] font-semibold" :style="{ color: 'var(--accent)' }"
            @click="router.push('/student/diagnostic')">Run Diagnostic →</button>
        </div>
      </div>

      <!-- Recent history panel -->
      <div
        class="w-72 flex-shrink-0 flex flex-col overflow-hidden border-l"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Recent Mocks</p>
        </div>
        <div class="flex-1 overflow-y-auto p-3 space-y-1">
          <div v-if="loading">
            <div v-for="i in 4" :key="i" class="h-14 rounded-xl animate-pulse mb-2"
              :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else-if="!history.length" class="py-10 text-center px-4">
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No mock sessions yet.<br>Start your first one!</p>
          </div>
          <button
            v-for="entry in history"
            :key="entry.id"
            class="hist-btn w-full text-left px-3 py-2.5 rounded-xl"
            :disabled="entry.status !== 'completed'"
            @click="entry.status === 'completed' ? router.push(`/student/mock/review/${entry.id}`) : null"
          >
            <div class="flex items-center gap-3">
              <div
                class="w-9 h-9 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
                :style="{ backgroundColor: gradeColor(entry.grade).bg, color: gradeColor(entry.grade).text }"
              >
                {{ entry.grade ?? (entry.status === 'in_progress' ? '…' : '—') }}
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[11px] font-semibold capitalize" :style="{ color: 'var(--ink)' }">
                  {{ entry.mock_type.replace(/_/g, ' ') }}
                </p>
                <p class="text-[9px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ entry.percentage != null ? entry.percentage.toFixed(0) + '%' : entry.status }}
                </p>
              </div>
              <span v-if="entry.status === 'completed'" :style="{ color: 'var(--ink-muted)' }" class="text-xs">→</span>
            </div>
          </button>
        </div>
        <div class="p-3 border-t flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full py-2.5 rounded-xl text-[11px] font-bold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="goToSetup('full')">
            Start Full Mock →
          </button>
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
  color: var(--accent);
  margin-bottom: 4px;
}

.nav-pill {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.mock-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px 20px;
  border-radius: 16px;
  border: 1px solid var(--border-soft);
  background-color: var(--surface);
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease, transform 120ms ease;
}
.mock-row:hover {
  border-color: var(--ink);
  transform: translateX(2px);
}

.mock-icon-box {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background-color: var(--paper);
  border: 1px solid var(--border-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.time-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 99px;
  background: var(--paper);
  color: var(--ink-muted);
  border: 1px solid var(--border-soft);
}

.go-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-muted);
  transition: color 120ms;
}
.mock-row:hover .go-label { color: var(--ink); }

.hist-btn {
  transition: background-color 100ms;
}
.hist-btn:not(:disabled):hover { background-color: var(--paper); }
.hist-btn:disabled { opacity: 0.6; cursor: default; }
</style>
