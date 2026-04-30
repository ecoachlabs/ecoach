<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { listExamHotspots, type ExamHotspotDto } from '@/ipc/library'
import { getQuestionFamilyHealth, type QuestionFamilyHealthDto } from '@/ipc/questions'
import { startPracticeSession } from '@/ipc/sessions'

const props = defineProps<{
  id: string | number
}>()

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const launching = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const hotspot = ref<ExamHotspotDto | null>(null)
const siblings = ref<ExamHotspotDto[]>([])
const health = ref<QuestionFamilyHealthDto | null>(null)

const familyId = computed(() => {
  const parsed = Number(props.id)
  return Number.isFinite(parsed) ? parsed : null
})

const accuracyPercent = computed(() => {
  if (hotspot.value?.student_accuracy_bp == null) return null
  return Math.round(hotspot.value.student_accuracy_bp / 100)
})

const healthMeters = computed(() => {
  if (!health.value) return []
  return [
    { label: 'Freshness', value: Math.round(health.value.freshness_score / 100), color: 'var(--accent)' },
    { label: 'Calibration', value: Math.round(health.value.calibration_score / 100), color: 'var(--gold)' },
    { label: 'Quality', value: Math.round(health.value.quality_score / 100), color: 'var(--warm)' },
  ]
})

const observedAccuracy = computed(() => {
  if (!health.value || health.value.recent_attempts === 0) return null
  return Math.round((health.value.recent_correct_attempts / health.value.recent_attempts) * 100)
})

onMounted(() => {
  void loadFamily()
})

async function loadFamily() {
  const studentId = auth.currentAccount?.id
  const currentFamilyId = familyId.value
  if (!studentId || currentFamilyId == null) {
    error.value = 'This family needs a valid student and family id.'
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    subjects.value = await listSubjects(1)

    const hotspotGroups = await Promise.all(
      subjects.value.map(async subject => ({
        subject,
        hotspots: await listExamHotspots(studentId, subject.id, 20).catch(() => []),
      })),
    )

    const matched = hotspotGroups.find(group =>
      group.hotspots.some(candidate => candidate.family_id === currentFamilyId),
    )

    const currentHotspot = matched?.hotspots.find(candidate => candidate.family_id === currentFamilyId) ?? null
    hotspot.value = currentHotspot
    siblings.value = matched?.hotspots.filter(candidate => candidate.family_id !== currentFamilyId).slice(0, 5) ?? []
    health.value = await getQuestionFamilyHealth(currentFamilyId)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to load family intelligence'
  } finally {
    loading.value = false
  }
}

async function startFamilyDrill() {
  if (!auth.currentAccount || !hotspot.value || hotspot.value.topic_id == null) return

  launching.value = true
  error.value = ''

  try {
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: hotspot.value.subject_id,
      topic_ids: [hotspot.value.topic_id],
      family_ids: [hotspot.value.family_id],
      question_count: 10,
      is_timed: false,
    })
    void router.push(`/student/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to start family drill'
    launching.value = false
  }
}

function percent(bp: number | null | undefined) {
  if (bp == null) return null
  return Math.round(bp / 100)
}

function familySymbol(name: string | undefined) {
  const words = (name ?? '').split(/\s+/).filter(Boolean)
  if (words.length === 0) return 'QF'
  if (words.length === 1) return words[0].slice(0, 2).toUpperCase()
  return `${words[0][0]}${words[1][0]}`.toUpperCase()
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pt-6 pb-5"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-start justify-between gap-6">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <button class="back-btn" @click="router.push('/student/exam-intel')">Back</button>
            <p class="eyebrow">Exam Intelligence</p>
          </div>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            {{ hotspot?.family_name ?? `Family ${familyId ?? ''}` }}
          </h1>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
            {{ hotspot?.topic_name ?? 'Whole-subject pattern' }} ·
            {{ subjects.find(subject => subject.id === hotspot?.subject_id)?.name ?? 'Subject lookup in progress' }}
          </p>
        </div>

        <div class="symbol-lockup">
          {{ familySymbol(hotspot?.family_name) }}
        </div>
      </div>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="index in 4" :key="index" class="h-28 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 flex overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-3">Family Profile</p>
          <p class="text-sm leading-6" :style="{ color: 'var(--ink)' }">
            {{ hotspot?.reason ?? 'No family profile has been surfaced yet for this pattern.' }}
          </p>

          <div class="mt-5 grid gap-3 md:grid-cols-4">
            <div class="metric-card">
              <span class="metric-label">Recurs</span>
              <strong class="metric-value">{{ percent(hotspot?.recurrence_rate_bp) ?? '--' }}%</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Relevance</span>
              <strong class="metric-value">{{ percent(hotspot?.current_relevance_bp) ?? '--' }}%</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Your Accuracy</span>
              <strong class="metric-value">{{ accuracyPercent ?? '--' }}%</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Last Seen</span>
              <strong class="metric-value">{{ hotspot?.last_appearance_year ?? '--' }}</strong>
            </div>
          </div>
        </section>

        <section v-if="health" class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">Content Health</p>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ health.health_status }}</p>
            </div>
            <div class="health-pill">{{ observedAccuracy ?? '--' }}% observed accuracy</div>
          </div>

          <div class="grid gap-4 md:grid-cols-3">
            <div v-for="meter in healthMeters" :key="meter.label" class="space-y-2">
              <div class="flex items-center justify-between text-[11px] font-semibold">
                <span :style="{ color: 'var(--ink-muted)' }">{{ meter.label }}</span>
                <span :style="{ color: meter.color }">{{ meter.value }}%</span>
              </div>
              <div class="meter-track">
                <div class="meter-fill" :style="{ width: `${meter.value}%`, backgroundColor: meter.color }" />
              </div>
            </div>
          </div>

          <div class="mt-5 grid gap-3 md:grid-cols-4">
            <div class="metric-card">
              <span class="metric-label">Active Variants</span>
              <strong class="metric-value">{{ health.active_instances }}</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Observed Attempts</span>
              <strong class="metric-value">{{ health.recent_attempts }}</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Avg Response</span>
              <strong class="metric-value">{{ Math.round(health.avg_response_time_ms / 1000) }}s</strong>
            </div>
            <div class="metric-card">
              <span class="metric-label">Observed Hits</span>
              <strong class="metric-value">{{ health.misconception_hit_count }}</strong>
            </div>
          </div>
        </section>

        <section v-if="siblings.length" class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-4">Nearby Families</p>
          <div class="space-y-2">
            <button
              v-for="sibling in siblings"
              :key="sibling.family_id"
              class="sibling-row"
              @click="router.push(`/student/exam-intel/family/${sibling.family_id}`)"
            >
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ sibling.family_name }}</p>
                <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">{{ sibling.reason }}</p>
              </div>
              <strong class="text-sm" :style="{ color: 'var(--accent)' }">{{ percent(sibling.recurrence_rate_bp) ?? '--' }}%</strong>
            </button>
          </div>
        </section>
      </div>

      <aside
        class="w-80 flex-shrink-0 border-l overflow-y-auto p-5 space-y-5"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Action</p>
          <div class="space-y-2">
            <button
              class="primary-btn"
              :disabled="!hotspot || hotspot.topic_id == null || launching"
              @click="startFamilyDrill"
            >
              {{
                !hotspot || hotspot.topic_id == null
                  ? 'Topic Needed'
                  : launching
                    ? 'Starting...'
                    : 'Start Family Drill'
              }}
            </button>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              This filters the real question pool to this family inside its linked topic.
            </p>
            <button
              v-if="hotspot?.topic_id"
              class="sidebar-btn"
              @click="router.push(`/student/teach/${hotspot.topic_id}`)"
            >
              Open Teach Mode
            </button>
            <button class="sidebar-btn" @click="router.push('/student/library')">Open Library</button>
          </div>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Pattern Window</p>
          <div class="space-y-3 text-sm">
            <div class="detail-row">
              <span>First appearance</span>
              <strong>{{ hotspot?.first_appearance_year ?? '--' }}</strong>
            </div>
            <div class="detail-row">
              <span>Last appearance</span>
              <strong>{{ hotspot?.last_appearance_year ?? '--' }}</strong>
            </div>
            <div class="detail-row">
              <span>Persistence</span>
              <strong>{{ percent(hotspot?.persistence_score_bp) ?? '--' }}%</strong>
            </div>
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

.back-btn,
.primary-btn,
.sidebar-btn,
.sibling-row {
  border: 1px solid transparent;
  transition: all 120ms;
}

.back-btn {
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  background: var(--paper);
  color: var(--ink-secondary);
}

.back-btn:hover,
.sidebar-btn:hover,
.sibling-row:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.symbol-lockup {
  width: 72px;
  height: 72px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--paper);
  color: var(--ink);
  font-size: 20px;
  font-weight: 900;
}

.metric-card {
  padding: 14px;
  border-radius: 14px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.metric-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
}

.metric-value {
  font-size: 20px;
  font-weight: 900;
  color: var(--ink);
}

.meter-track {
  height: 7px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--border-soft);
}

.meter-fill {
  height: 100%;
  border-radius: 999px;
}

.health-pill {
  padding: 7px 12px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink);
  font-size: 11px;
  font-weight: 700;
}

.sibling-row {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 14px;
  border-radius: 14px;
  background: var(--paper);
  text-align: left;
}

.primary-btn,
.sidebar-btn {
  width: 100%;
  padding: 10px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  background: var(--surface);
  color: var(--ink);
}

.primary-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
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
