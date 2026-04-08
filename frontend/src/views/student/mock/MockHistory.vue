<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listMockSessions, type MockSessionSummaryDto } from '@/ipc/mock'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const sessions = ref<MockSessionSummaryDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    sessions.value = await listMockSessions(auth.currentAccount.id, 50)
  } catch (e) {
    console.error('Failed to load mock history:', e)
  }
  loading.value = false
})

function gradeColor(grade: string | null): { bg: string; text: string } {
  if (!grade) return { bg: 'var(--paper)', text: 'var(--ink-muted)' }
  if (['A1', 'B2', 'B3'].includes(grade)) return { bg: 'rgba(13,148,136,0.1)', text: 'var(--accent)' }
  if (['C4', 'C5', 'C6'].includes(grade)) return { bg: 'rgba(180,83,9,0.1)', text: 'var(--gold)' }
  return { bg: 'rgba(194,65,12,0.1)', text: 'var(--warm)' }
}

const totalCompleted = computed(() => sessions.value.filter(s => s.status === 'completed').length)
const avgScore = computed(() => {
  const completed = sessions.value.filter(s => s.percentage != null)
  if (!completed.length) return 0
  return completed.reduce((acc, s) => acc + (s.percentage ?? 0), 0) / completed.length
})
const bestGrade = computed(() => {
  const gradeOrder = ['A1', 'B2', 'B3', 'C4', 'C5', 'C6', 'D7', 'E8', 'F9']
  const grades = sessions.value.map(s => s.grade).filter(Boolean) as string[]
  if (!grades.length) return '—'
  return grades.sort((a, b) => gradeOrder.indexOf(a) - gradeOrder.indexOf(b))[0]
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Mock History</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Battle Record</h1>
      </div>
      <button class="nav-pill" @click="router.push('/student/mock')">← Mock Centre</button>
    </div>

    <!-- Stats strip -->
    <div
      class="flex-shrink-0 grid grid-cols-3 divide-x border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--ink)' }">{{ totalCompleted }}</p>
        <p class="stat-lbl">Completed</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: avgScore >= 60 ? 'var(--accent)' : avgScore >= 45 ? 'var(--gold)' : 'var(--warm)' }">
          {{ avgScore.toFixed(0) }}%
        </p>
        <p class="stat-lbl">Avg Score</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="gradeColor(bestGrade)">{{ bestGrade }}</p>
        <p class="stat-lbl">Best Grade</p>
      </div>
    </div>

    <!-- Session list -->
    <div class="flex-1 overflow-y-auto">
      <div v-if="loading" class="p-6 space-y-3">
        <div v-for="i in 6" :key="i" class="h-16 rounded-xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else-if="!sessions.length" class="flex flex-col items-center justify-center h-full gap-4">
        <p class="text-sm font-semibold" :style="{ color: 'var(--ink-muted)' }">No mock sessions yet</p>
        <button
          class="px-5 py-2.5 rounded-xl font-semibold text-sm"
          :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
          @click="router.push('/student/mock/setup')"
        >Start Your First Mock</button>
      </div>

        <div v-else class="divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
        <button
          v-for="s in sessions"
          :key="s.id"
          class="session-row w-full text-left px-7 py-4 flex items-center gap-5"
          :disabled="s.status !== 'completed'"
          @click="s.status === 'completed' ? router.push(`/student/mock/review/${s.id}`) : null"
        >
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center text-sm font-black flex-shrink-0"
            :style="{ backgroundColor: gradeColor(s.grade).bg, color: gradeColor(s.grade).text }"
          >{{ s.grade ?? '—' }}</div>

          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold capitalize" :style="{ color: 'var(--ink)' }">
              {{ s.mock_type.replace(/_/g, ' ') }}
            </p>
            <div class="flex items-center gap-3 mt-0.5">
              <span v-if="s.percentage != null" class="text-[11px] font-semibold"
                :style="{ color: gradeColor(s.grade).text }">{{ s.percentage.toFixed(1) }}%</span>
              <span v-if="s.paper_year" class="text-[11px]"
                :style="{ color: 'var(--ink-muted)' }">{{ s.paper_year }}</span>
            </div>
          </div>

          <span
            class="text-[10px] font-bold px-3 py-1 rounded-full flex-shrink-0"
            :style="{
              background: s.status === 'completed' ? 'rgba(13,148,136,0.1)' : s.status === 'in_progress' ? 'rgba(180,83,9,0.1)' : 'var(--paper)',
              color: s.status === 'completed' ? 'var(--accent)' : s.status === 'in_progress' ? 'var(--gold)' : 'var(--ink-muted)',
            }"
          >{{ s.status.replace(/_/g, ' ') }}</span>

          <span v-if="s.status === 'completed'" class="text-sm" :style="{ color: 'var(--ink-muted)' }">→</span>
        </button>
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
  color: var(--gold);
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
  border: 1px solid transparent;
  transition: all 100ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.stat-cell {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.stat-big {
  font-size: 22px;
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

.session-row {
  transition: background-color 100ms;
}
.session-row:not(:disabled):hover { background-color: var(--surface); }
.session-row:disabled { cursor: default; opacity: 0.7; }
</style>


