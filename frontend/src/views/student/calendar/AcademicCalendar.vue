<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildAcademicCalendarSnapshot,
  getPreparationIntensityProfile,
  getWeeklyPlanSnapshot,
  type AcademicCalendarSnapshotDto,
  type PreparationIntensityProfileDto,
  type WeeklyPlanSnapshotDto,
} from '@/ipc/coach'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const error = ref('')
const calendar = ref<AcademicCalendarSnapshotDto | null>(null)
const weeklyPlan = ref<WeeklyPlanSnapshotDto | null>(null)
const intensity = ref<PreparationIntensityProfileDto | null>(null)

const anchorDate = computed(() => localDateIso(new Date()))

const phaseSteps = [
  { key: 'long_range', label: 'Long Range' },
  { key: 'structured', label: 'Structured' },
  { key: 'focused', label: 'Focused' },
  { key: 'intensive', label: 'Intensive' },
  { key: 'last_mile', label: 'Last Mile' },
]

const currentPhase = computed(() => intensity.value?.phase ?? calendar.value?.intensity?.phase ?? '')

const upcomingEvents = computed(() =>
  (calendar.value?.events ?? [])
    .slice()
    .sort((left, right) => left.scheduled_date.localeCompare(right.scheduled_date)),
)

const weeklyRows = computed(() =>
  (weeklyPlan.value?.days ?? []).map(day => ({
    key: day.date,
    day: new Date(`${day.date}T00:00:00`).toLocaleDateString('en-US', { weekday: 'short' }),
    focus: day.blocks[0]?.rationale ?? 'Recovery block',
    subject: day.blocks[0]?.exam_support_scope ?? day.blocks[0]?.session_type ?? 'Study',
    duration: `${day.planned_minutes} min`,
    done: day.date < anchorDate.value,
  })),
)

onMounted(() => {
  void loadCalendar()
})

async function loadCalendar() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    const studentId = auth.currentAccount.id
    const [calendarSnapshot, weeklyPlanSnapshot, intensityProfile] = await Promise.all([
      buildAcademicCalendarSnapshot(studentId, anchorDate.value),
      getWeeklyPlanSnapshot(studentId, null, anchorDate.value),
      getPreparationIntensityProfile(studentId, anchorDate.value),
    ])

    calendar.value = calendarSnapshot
    weeklyPlan.value = weeklyPlanSnapshot
    intensity.value = intensityProfile ?? calendarSnapshot.intensity
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load academic calendar'
  } finally {
    loading.value = false
  }
}

function urgencyColor(daysLeft: number | null): string {
  if (daysLeft == null) return 'var(--accent)'
  if (daysLeft <= 30) return 'var(--warm)'
  if (daysLeft <= 60) return 'var(--gold)'
  return 'var(--accent)'
}

function eventTypeLabel(type: string) {
  return type.replace(/_/g, ' ')
}

function localDateIso(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Planning</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Academic Calendar
      </h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        {{ calendar?.strategy_message ?? 'Your exam timeline and preparation plan' }}
      </p>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 4" :key="i" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div>
          <p class="section-label mb-4">Upcoming Exams</p>
          <div v-if="upcomingEvents.length" class="space-y-2">
            <div
              v-for="exam in upcomingEvents"
              :key="exam.id"
              class="exam-row flex items-center gap-4 px-5 py-4 rounded-2xl border"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
            >
              <div class="countdown-box flex-shrink-0"
                :style="{ borderColor: urgencyColor(exam.days_to_event), color: urgencyColor(exam.days_to_event) }">
                <span class="text-2xl font-black tabular-nums leading-none">{{ exam.days_to_event ?? '--' }}</span>
                <span class="text-[8px] font-bold uppercase tracking-wide">days</span>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ exam.title }}</p>
                <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                  {{ exam.scheduled_date }} · {{ exam.subject_name ?? exam.scope }}
                </p>
              </div>
              <span class="type-chip" :class="exam.event_type">{{ eventTypeLabel(exam.event_type) }}</span>
            </div>
          </div>
          <div v-else class="rounded-2xl px-5 py-5 text-sm"
            :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
            No academic events are scheduled yet.
          </div>
        </div>

        <div>
          <p class="section-label mb-4">Preparation Phase</p>
          <div class="flex gap-1 rounded-2xl border p-1.5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
            <div
              v-for="phase in phaseSteps"
              :key="phase.key"
              class="flex-1 py-2 text-center text-[10px] font-semibold rounded-xl transition-all"
              :class="phase.key === currentPhase ? 'active-phase' : 'inactive-phase'"
            >{{ phase.label }}</div>
          </div>
          <p class="text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">
            <strong :style="{ color: 'var(--ink)' }">{{ intensity?.recommended_mode ?? 'adaptive' }}</strong>
            · {{ intensity?.revision_density ?? 'balanced' }} · {{ intensity?.tone ?? 'steady' }}
          </p>
        </div>
      </div>

      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">This Week</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-2">
          <div
            v-for="day in weeklyRows"
            :key="day.key"
            class="day-row flex items-center gap-3 px-4 py-3 rounded-xl border"
            :class="{ done: day.done }"
            :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }"
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
          <button class="w-full plan-btn" @click="router.push('/student/journey')">Open Study Route</button>
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

.exam-row:hover {
  background-color: var(--paper) !important;
}

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
  border: 1px solid transparent;
  color: var(--ink-secondary);
  background: var(--paper);
}

.type-chip.final {
  color: var(--warm);
  border-color: rgba(194,65,12,0.3);
  background: rgba(194,65,12,0.06);
}

.type-chip.exam {
  color: var(--gold);
  border-color: rgba(180,83,9,0.3);
  background: rgba(180,83,9,0.06);
}

.type-chip.mock {
  color: var(--accent);
  border-color: rgba(13,148,136,0.3);
  background: rgba(13,148,136,0.06);
}

.active-phase {
  background: var(--ink);
  color: var(--paper);
}

.inactive-phase {
  background: transparent;
  color: var(--ink-muted);
}

.day-row {
  transition: background-color 100ms;
}

.day-row:hover {
  background-color: var(--surface) !important;
}

.day-row.done {
  opacity: 0.55;
}

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
  border: 1px solid transparent;
  transition: all 120ms;
}

.plan-btn:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
