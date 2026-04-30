<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import { resetPin } from '@/ipc/identity'
import {
  getAvailabilityProfile,
  getGoalRecommendation,
  getHomeLearningStats,
  listAvailabilityWindows,
  listGoalProfiles,
  listSubjects,
  replaceAvailabilityWindows,
  saveLearnerGoal,
  upsertAvailabilityProfile,
  type AvailabilityProfileDto,
  type AvailabilityWindowDto,
  type GoalProfileDto,
  type GoalRecommendationDto,
  type HomeLearningStatsDto,
  type SubjectDto,
} from '@/ipc/coach'
import { PIN_LENGTH, isValidPin } from '@/utils/validation'

const auth = useAuthStore()
const ui = useUiStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const success = ref('')

const showPinForm = ref(false)
const newPin = ref('')
const confirmPin = ref('')
const pinSaving = ref(false)
const pinError = ref('')
const pinSuccess = ref('')

const subjects = ref<SubjectDto[]>([])
const selectedGoalSubjectId = ref<number | null>(null)
const recommendation = ref<GoalRecommendationDto | null>(null)
const goalProfiles = ref<GoalProfileDto[]>([])
const stats = ref<HomeLearningStatsDto | null>(null)
const goalType = ref('exam_readiness')
const examTarget = ref('')
const examDate = ref('')
const confidenceLevel = ref('steady')
const savingGoal = ref(false)

const availabilityProfile = ref<AvailabilityProfileDto>(defaultAvailabilityProfile(0))
const availabilityWindows = ref<AvailabilityWindowDto[]>(defaultAvailabilityWindows())
const savingAvailability = ref(false)

const soundEffects = ref(readBool('ecoach-pref-sound', true))
const studyReminders = ref(readBool('ecoach-pref-reminders', true))
const compactMode = ref(readBool('ecoach-pref-compact', false))

const goalOptions = [
  { key: 'foundation_building', label: 'Foundation' },
  { key: 'weakness_repair', label: 'Weakness Repair' },
  { key: 'revision_refresh', label: 'Revision Refresh' },
  { key: 'exam_readiness', label: 'Exam Readiness' },
  { key: 'score_improvement', label: 'Score Improvement' },
  { key: 'top_performance', label: 'Top Performance' },
  { key: 'confidence_recovery', label: 'Confidence Recovery' },
  { key: 'mastery_acceleration', label: 'Mastery Acceleration' },
] as const

const weekdayLabels = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

const activeGoals = computed(() =>
  goalProfiles.value
    .filter(goal => goal.goal_state === 'active')
    .slice(0, 6),
)

onMounted(() => {
  void loadSettings()
})

watch(selectedGoalSubjectId, () => {
  void loadRecommendation()
})

watch(soundEffects, value => writeBool('ecoach-pref-sound', value))
watch(studyReminders, value => writeBool('ecoach-pref-reminders', value))
watch(compactMode, value => writeBool('ecoach-pref-compact', value))

async function loadSettings() {
  const studentId = auth.currentAccount?.id
  if (!studentId) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    const [nextSubjects, nextGoals, nextStats, nextProfile, nextWindows] = await Promise.all([
      listSubjects(1),
      listGoalProfiles(studentId).catch(() => []),
      getHomeLearningStats(studentId).catch(() => null),
      getAvailabilityProfile(studentId).catch(() => null),
      listAvailabilityWindows(studentId).catch(() => []),
    ])

    subjects.value = nextSubjects
    goalProfiles.value = nextGoals
    stats.value = nextStats
    availabilityProfile.value = nextProfile ?? defaultAvailabilityProfile(studentId)
    availabilityWindows.value = nextWindows.length ? nextWindows : defaultAvailabilityWindows()

    selectedGoalSubjectId.value =
      nextGoals.find(goal => goal.subject_id != null)?.subject_id
      ?? nextSubjects[0]?.id
      ?? null
  } catch (cause: unknown) {
    error.value = toMessage(cause, 'Failed to load settings')
  } finally {
    loading.value = false
  }
}

async function loadRecommendation() {
  const studentId = auth.currentAccount?.id
  if (!studentId || selectedGoalSubjectId.value == null) {
    recommendation.value = null
    return
  }

  try {
    recommendation.value = await getGoalRecommendation(studentId, selectedGoalSubjectId.value)
    goalType.value = recommendation.value.goal_type
  } catch {
    recommendation.value = null
  }
}

async function savePin() {
  pinError.value = ''
  pinSuccess.value = ''

  if (!isValidPin(newPin.value)) {
    pinError.value = `PIN must be exactly ${PIN_LENGTH} digits`
    return
  }

  if (newPin.value !== confirmPin.value) {
    pinError.value = 'PINs do not match'
    return
  }

  if (!auth.currentAccount) return

  pinSaving.value = true
  try {
    await resetPin(auth.currentAccount.id, newPin.value)
    pinSuccess.value = 'PIN updated successfully'
    newPin.value = ''
    confirmPin.value = ''
    showPinForm.value = false
  } catch (cause: unknown) {
    pinError.value = toMessage(cause, 'Failed to update PIN')
  } finally {
    pinSaving.value = false
  }
}

async function saveGoal() {
  const studentId = auth.currentAccount?.id
  if (!studentId || selectedGoalSubjectId.value == null) return

  savingGoal.value = true
  error.value = ''
  success.value = ''

  try {
    await saveLearnerGoal(
      studentId,
      selectedGoalSubjectId.value,
      goalType.value,
      examTarget.value.trim() || null,
      examDate.value || null,
      confidenceLevel.value || null,
    )
    success.value = 'Goal profile updated.'
    goalProfiles.value = await listGoalProfiles(studentId).catch(() => goalProfiles.value)
    await loadRecommendation()
  } catch (cause: unknown) {
    error.value = toMessage(cause, 'Failed to save goal')
  } finally {
    savingGoal.value = false
  }
}

async function saveAvailability() {
  const studentId = auth.currentAccount?.id
  if (!studentId) return

  savingAvailability.value = true
  error.value = ''
  success.value = ''

  try {
    availabilityProfile.value = await upsertAvailabilityProfile({
      ...availabilityProfile.value,
      student_id: studentId,
    })

    availabilityWindows.value = await replaceAvailabilityWindows(
      studentId,
      availabilityWindows.value.map(window => ({
        ...window,
        end_minute: Math.max(window.end_minute, window.start_minute + 30),
      })),
    )

    success.value = 'Study rhythm saved.'
  } catch (cause: unknown) {
    error.value = toMessage(cause, 'Failed to save study rhythm')
  } finally {
    savingAvailability.value = false
  }
}

function setWindowTime(index: number, field: 'start_minute' | 'end_minute', value: string) {
  const [hours, minutes] = value.split(':').map(Number)
  if (!Number.isFinite(hours) || !Number.isFinite(minutes)) return
  availabilityWindows.value[index][field] = hours * 60 + minutes
}

function toClock(minuteOfDay: number) {
  const hours = Math.floor(minuteOfDay / 60)
  const minutes = minuteOfDay % 60
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

function defaultAvailabilityProfile(studentId: number): AvailabilityProfileDto {
  return {
    student_id: studentId,
    timezone_name: Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC',
    preferred_daily_minutes: 90,
    ideal_session_minutes: 35,
    min_session_minutes: 20,
    max_session_minutes: 60,
    split_sessions_allowed: true,
    max_split_sessions: 2,
    min_break_minutes: 15,
    trigger_mode: 'coach_recommended',
    notification_lead_minutes: 20,
    weekday_capacity_weight_bp: 6500,
    weekend_capacity_weight_bp: 8200,
    schedule_buffer_ratio_bp: 1500,
    fatigue_start_minute: 1200,
    fatigue_end_minute: 1380,
    thinking_idle_grace_seconds: 60,
    idle_confirmation_seconds: 180,
    abandonment_seconds: 900,
  }
}

function defaultAvailabilityWindows(): AvailabilityWindowDto[] {
  return [
    { weekday: 1, start_minute: 16 * 60, end_minute: 19 * 60, is_preferred: true },
    { weekday: 2, start_minute: 16 * 60, end_minute: 19 * 60, is_preferred: true },
    { weekday: 3, start_minute: 16 * 60, end_minute: 19 * 60, is_preferred: true },
    { weekday: 4, start_minute: 16 * 60, end_minute: 19 * 60, is_preferred: true },
    { weekday: 5, start_minute: 16 * 60, end_minute: 18 * 60 + 30, is_preferred: true },
    { weekday: 6, start_minute: 9 * 60, end_minute: 12 * 60, is_preferred: true },
    { weekday: 7, start_minute: 14 * 60, end_minute: 17 * 60, is_preferred: false },
  ]
}

function readBool(key: string, fallback: boolean) {
  const stored = localStorage.getItem(key)
  return stored == null ? fallback : stored === 'true'
}

function writeBool(key: string, value: boolean) {
  localStorage.setItem(key, String(value))
}

function toMessage(cause: unknown, fallback: string) {
  return typeof cause === 'string'
    ? cause
    : cause instanceof Error
      ? cause.message
      : fallback
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pt-6 pb-5"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Settings</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Preferences</h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        Real study rhythm, real goal steering, and device preferences.
      </p>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="success" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(13,148,136,0.08)', color: 'var(--accent)' }">
      {{ success }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="index in 4" :key="index" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 flex overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="flex items-start justify-between gap-6">
            <div>
              <p class="section-label mb-2">Account</p>
              <h2 class="text-xl font-bold" :style="{ color: 'var(--ink)' }">{{ auth.currentAccount?.display_name }}</h2>
              <p class="text-sm mt-1" :style="{ color: 'var(--ink-muted)' }">Student account</p>
            </div>
            <div class="grid gap-3 sm:grid-cols-3">
              <div class="stat-card">
                <span class="stat-label">Streak</span>
                <strong class="stat-value">{{ stats?.streak_days ?? 0 }}</strong>
              </div>
              <div class="stat-card">
                <span class="stat-label">Accuracy</span>
                <strong class="stat-value">{{ stats?.accuracy_percent ?? 0 }}%</strong>
              </div>
              <div class="stat-card">
                <span class="stat-label">Attempts</span>
                <strong class="stat-value">{{ stats?.total_attempts ?? 0 }}</strong>
              </div>
            </div>
          </div>

          <div class="mt-5 flex flex-wrap gap-2">
            <button class="secondary-btn" @click="showPinForm = !showPinForm">
              {{ showPinForm ? 'Close PIN Form' : 'Change PIN' }}
            </button>
            <button class="secondary-btn" @click="auth.logout(); router.push('/')">Switch Account</button>
            <button class="secondary-btn" @click="auth.logout(); router.push('/')">Sign Out</button>
          </div>

          <div v-if="showPinForm" class="mt-5 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mini-label mb-2 block">New PIN</label>
              <input v-model="newPin" type="password" inputmode="numeric" maxlength="4" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Confirm PIN</label>
              <input v-model="confirmPin" type="password" inputmode="numeric" maxlength="4" class="text-input" />
            </div>
            <div class="md:col-span-2 flex items-center gap-3">
              <button class="primary-btn inline-btn" :disabled="pinSaving" @click="savePin">
                {{ pinSaving ? 'Saving...' : 'Save PIN' }}
              </button>
              <span v-if="pinError" class="text-sm" :style="{ color: 'var(--warm)' }">{{ pinError }}</span>
              <span v-if="pinSuccess" class="text-sm" :style="{ color: 'var(--accent)' }">{{ pinSuccess }}</span>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">Goal Steering</p>
              <p class="text-sm mt-1" :style="{ color: 'var(--ink-muted)' }">
                Save the goal profile that shapes your coach strategy.
              </p>
            </div>
            <button class="primary-btn inline-btn" :disabled="savingGoal || selectedGoalSubjectId == null" @click="saveGoal">
              {{ savingGoal ? 'Saving...' : 'Save Goal' }}
            </button>
          </div>

          <div class="flex flex-wrap gap-1.5 mb-4">
            <button
              v-for="subject in subjects"
              :key="subject.id"
              class="chip"
              :class="{ active: selectedGoalSubjectId === subject.id }"
              @click="selectedGoalSubjectId = subject.id"
            >
              {{ subject.name }}
            </button>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <div>
              <label class="mini-label mb-2 block">Goal Type</label>
              <select v-model="goalType" class="text-input">
                <option v-for="option in goalOptions" :key="option.key" :value="option.key">{{ option.label }}</option>
              </select>
            </div>
            <div>
              <label class="mini-label mb-2 block">Confidence</label>
              <select v-model="confidenceLevel" class="text-input">
                <option value="fragile">Fragile</option>
                <option value="steady">Steady</option>
                <option value="high">High</option>
              </select>
            </div>
            <div>
              <label class="mini-label mb-2 block">Target Exam</label>
              <input v-model="examTarget" type="text" class="text-input" placeholder="BECE, WASSCE, mock final" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Exam Date</label>
              <input v-model="examDate" type="date" class="text-input" />
            </div>
          </div>

          <div v-if="recommendation" class="mt-5 rounded-2xl border p-4" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }">
            <p class="mini-label">Coach Recommendation</p>
            <p class="text-sm mt-2" :style="{ color: 'var(--ink)' }">
              {{ recommendation.urgency_label }} · {{ recommendation.session_style }}
            </p>
            <div class="mt-3 flex flex-wrap gap-2">
              <span v-for="action in recommendation.recommended_actions" :key="action" class="signal-pill">{{ action }}</span>
            </div>
          </div>

          <div v-if="activeGoals.length" class="mt-5">
            <p class="mini-label mb-3">Active Goal Profiles</p>
            <div class="space-y-2">
              <div v-for="goal in activeGoals" :key="goal.id" class="goal-row">
                <div class="min-w-0">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ goal.title }}</p>
                  <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">
                    {{ goal.goal_type }} · {{ goal.goal_level }} · {{ goal.urgency_level }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">Study Rhythm</p>
              <p class="text-sm mt-1" :style="{ color: 'var(--ink-muted)' }">
                This feeds the real availability and reminder engine.
              </p>
            </div>
            <button class="primary-btn inline-btn" :disabled="savingAvailability" @click="saveAvailability">
              {{ savingAvailability ? 'Saving...' : 'Save Rhythm' }}
            </button>
          </div>

          <div class="grid gap-4 md:grid-cols-3">
            <div>
              <label class="mini-label mb-2 block">Daily Minutes</label>
              <input v-model.number="availabilityProfile.preferred_daily_minutes" type="number" min="15" step="5" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Ideal Session</label>
              <input v-model.number="availabilityProfile.ideal_session_minutes" type="number" min="10" step="5" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Reminder Lead</label>
              <input v-model.number="availabilityProfile.notification_lead_minutes" type="number" min="0" step="5" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Min Session</label>
              <input v-model.number="availabilityProfile.min_session_minutes" type="number" min="10" step="5" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Max Session</label>
              <input v-model.number="availabilityProfile.max_session_minutes" type="number" min="15" step="5" class="text-input" />
            </div>
            <div>
              <label class="mini-label mb-2 block">Split Sessions</label>
              <select v-model="availabilityProfile.trigger_mode" class="text-input">
                <option value="coach_recommended">Coach Recommended</option>
                <option value="always_on">Always On</option>
                <option value="manual">Manual</option>
              </select>
            </div>
          </div>

          <div class="mt-4 flex flex-wrap gap-6">
            <label class="toggle-row">
              <input v-model="availabilityProfile.split_sessions_allowed" type="checkbox" />
              <span>Allow split sessions</span>
            </label>
            <label class="toggle-row">
              <input v-model="studyReminders" type="checkbox" />
              <span>Enable study reminders on this device</span>
            </label>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-4">Weekly Windows</p>
          <div class="space-y-3">
            <div v-for="(window, index) in availabilityWindows" :key="window.weekday" class="window-row">
              <div class="window-day">{{ weekdayLabels[index] }}</div>
              <input :value="toClock(window.start_minute)" type="time" class="time-input" @change="setWindowTime(index, 'start_minute', ($event.target as HTMLInputElement).value)" />
              <input :value="toClock(window.end_minute)" type="time" class="time-input" @change="setWindowTime(index, 'end_minute', ($event.target as HTMLInputElement).value)" />
              <label class="toggle-row compact">
                <input v-model="window.is_preferred" type="checkbox" />
                <span>Preferred</span>
              </label>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-4">Device Preferences</p>
          <div class="space-y-3">
            <label class="pref-row">
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Dark Mode</p>
                <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Applied across the app theme.</p>
              </div>
              <input :checked="ui.isDark" type="checkbox" @change="ui.toggleDark()" />
            </label>
            <label class="pref-row">
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Sound Effects</p>
                <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Saved on this device.</p>
              </div>
              <input v-model="soundEffects" type="checkbox" />
            </label>
            <label class="pref-row">
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Compact Layout</p>
                <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Saved on this device.</p>
              </div>
              <input v-model="compactMode" type="checkbox" />
            </label>
          </div>
        </section>
      </div>

      <aside
        class="w-80 flex-shrink-0 border-l overflow-y-auto p-5 space-y-5"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Signals</p>
          <div class="space-y-3 text-sm">
            <div class="detail-row">
              <span>Daily target</span>
              <strong>{{ availabilityProfile.preferred_daily_minutes }} min</strong>
            </div>
            <div class="detail-row">
              <span>Ideal block</span>
              <strong>{{ availabilityProfile.ideal_session_minutes }} min</strong>
            </div>
            <div class="detail-row">
              <span>Trigger mode</span>
              <strong>{{ availabilityProfile.trigger_mode }}</strong>
            </div>
            <div class="detail-row">
              <span>Goal subject</span>
              <strong>{{ subjects.find(subject => subject.id === selectedGoalSubjectId)?.name ?? '--' }}</strong>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Quick Links</p>
          <div class="space-y-2">
            <button class="secondary-btn full-btn" @click="router.push('/student/calendar')">Open Calendar</button>
            <button class="secondary-btn full-btn" @click="router.push('/student/memory')">Open Memory</button>
            <button class="secondary-btn full-btn" @click="router.push('/student/journey')">Open Journey</button>
          </div>
        </section>
      </aside>
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
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.mini-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink);
}

.text-input {
  width: 100%;
  border-radius: 12px;
  border: 1px solid var(--border-soft);
  background: var(--paper);
  color: var(--ink);
  padding: 12px 14px;
  font-size: 14px;
  outline: none;
}

.primary-btn,
.secondary-btn,
.chip {
  border: 1px solid transparent;
  transition: all 120ms;
}

.primary-btn,
.secondary-btn {
  padding: 10px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
}

.primary-btn {
  background: var(--ink);
  color: var(--paper);
}

.secondary-btn {
  background: var(--paper);
  color: var(--ink-secondary);
}

.secondary-btn:hover,
.chip:hover,
.chip.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.inline-btn {
  width: auto;
}

.full-btn {
  width: 100%;
}

.chip {
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  background: var(--paper);
  color: var(--ink-secondary);
}

.stat-card {
  padding: 16px;
  border-radius: 14px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stat-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
}

.stat-value {
  font-size: 22px;
  font-weight: 900;
  color: var(--ink);
}

.goal-row,
.pref-row,
.window-row {
  display: flex;
  align-items: center;
  gap: 14px;
  justify-content: space-between;
  padding: 12px 14px;
  border-radius: 14px;
  background: var(--paper);
}

.window-day {
  width: 34px;
  font-size: 12px;
  font-weight: 700;
  color: var(--ink);
}

.time-input {
  border-radius: 10px;
  border: 1px solid var(--border-soft);
  background: var(--surface);
  color: var(--ink);
  padding: 8px 10px;
  font-size: 12px;
}

.toggle-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--ink);
  font-size: 13px;
}

.toggle-row.compact {
  font-size: 12px;
}

.signal-pill {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--surface);
  color: var(--ink-secondary);
  font-size: 11px;
  font-weight: 700;
}

.detail-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: var(--ink-muted);
}

.detail-row strong {
  color: var(--ink);
}
</style>
