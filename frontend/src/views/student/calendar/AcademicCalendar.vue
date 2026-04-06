<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

const examDates = ref([
  { id: 1, name: 'Internal Mock', date: '2026-04-15', daysLeft: 10, subject: 'All Subjects', type: 'mock' },
  { id: 2, name: 'Mid-Term Exam', date: '2026-05-10', daysLeft: 35, subject: 'All Subjects', type: 'exam' },
  { id: 3, name: 'BECE Final', date: '2026-07-15', daysLeft: 101, subject: 'All Subjects', type: 'final' },
])

const weeklyPlan = ref([
  { day: 'Mon', subject: 'Mathematics', focus: 'Fractions repair', duration: '45 min', done: true },
  { day: 'Tue', subject: 'English', focus: 'Comprehension drill', duration: '30 min', done: true },
  { day: 'Wed', subject: 'Science', focus: 'Biology recall', duration: '40 min', done: false },
  { day: 'Thu', subject: 'Mathematics', focus: 'Algebra practice', duration: '45 min', done: false },
  { day: 'Fri', subject: 'All', focus: 'Mixed review', duration: '30 min', done: false },
])

const currentPhase = ref('Build')
const phases = ['Foundation', 'Build', 'Integration', 'Timed', 'Mock', 'Revision', 'Last-Mile']

function urgencyColor(daysLeft: number): string {
  if (daysLeft <= 30) return 'var(--warm)'
  if (daysLeft <= 60) return 'var(--gold)'
  return 'var(--accent)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Planning</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Academic Calendar
      </h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        Your exam timeline and preparation plan
      </p>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: main content -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">

        <!-- Exam countdown cards -->
        <div>
          <p class="section-label mb-4">Upcoming Exams</p>
          <div class="space-y-2">
            <div
              v-for="exam in examDates"
              :key="exam.id"
              class="exam-row flex items-center gap-4 px-5 py-4 rounded-2xl border"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
            >
              <div class="countdown-box flex-shrink-0"
                :style="{ borderColor: urgencyColor(exam.daysLeft), color: urgencyColor(exam.daysLeft) }">
                <span class="text-2xl font-black tabular-nums leading-none">{{ exam.daysLeft }}</span>
                <span class="text-[8px] font-bold uppercase tracking-wide">days</span>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ exam.name }}</p>
                <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ exam.date }} · {{ exam.subject }}</p>
              </div>
              <span class="type-chip" :class="exam.type">{{ exam.type }}</span>
            </div>
          </div>
        </div>

        <!-- Preparation phase -->
        <div>
          <p class="section-label mb-4">Preparation Phase</p>
          <div class="flex gap-1 rounded-2xl border p-1.5" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
            <div
              v-for="phase in phases"
              :key="phase"
              class="flex-1 py-2 text-center text-[10px] font-semibold rounded-xl transition-all"
              :class="phase === currentPhase ? 'active-phase' : 'inactive-phase'"
            >{{ phase }}</div>
          </div>
          <p class="text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">
            Currently in <strong :style="{ color: 'var(--ink)' }">{{ currentPhase }}</strong> phase — building topic mastery progressively.
          </p>
        </div>
      </div>

      <!-- Right: weekly plan -->
      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">This Week</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-2">
          <div
            v-for="day in weeklyPlan"
            :key="day.day"
            class="day-row flex items-center gap-3 px-4 py-3 rounded-xl border"
            :class="{ done: day.done }"
            :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }"
          >
            <div class="day-badge flex-shrink-0">{{ day.day }}</div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-bold truncate" :style="{ color: day.done ? 'var(--ink-muted)' : 'var(--ink)', textDecoration: day.done ? 'line-through' : 'none' }">
                {{ day.focus }}
              </p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ day.subject }}</p>
            </div>
            <span class="text-[10px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ day.duration }}</span>
          </div>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full plan-btn">Adjust Plan →</button>
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

.exam-row {
  transition: background-color 100ms;
}
.exam-row:hover { background-color: var(--paper) !important; }

.countdown-box {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  border: 2px solid;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.type-chip {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 3px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-soft);
  color: var(--ink-secondary);
  background: var(--paper);
}
.type-chip.final { color: var(--warm); border-color: rgba(194,65,12,0.3); background: rgba(194,65,12,0.06); }
.type-chip.exam { color: var(--gold); border-color: rgba(180,83,9,0.3); background: rgba(180,83,9,0.06); }
.type-chip.mock { color: var(--accent); border-color: rgba(13,148,136,0.3); background: rgba(13,148,136,0.06); }

.active-phase {
  background: var(--ink);
  color: var(--paper);
}
.inactive-phase {
  background: transparent;
  color: var(--ink-muted);
}

.day-row { transition: background-color 100ms; }
.day-row:hover { background-color: var(--surface) !important; }
.day-row.done { opacity: 0.55; }

.day-badge {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 800;
  background: var(--border-soft);
  color: var(--ink-secondary);
}

.plan-btn {
  padding: 9px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.plan-btn:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
</style>
