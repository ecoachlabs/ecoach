<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  listStudentActivityHistory,
  getPriorityTopics,
  listActiveMisconceptions,
  listSubjects,
  listTopics,
  type StudentActivityHistoryItemDto,
  type LearnerMisconceptionSnapshotDto,
  type SubjectDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import {
  checkRiseModeTransition,
  enterRiseMode,
  getRiseModeProfile,
  type RiseModeProfileDto,
  type StageTransitionResultDto,
} from '@/ipc/rise'
import {
  buildLearnerTopicIndex,
  type LearnerTopic,
} from '@/utils/learnerTopics'
import AppProgress from '@/components/ui/AppProgress.vue'
import StageIndicator from '@/components/modes/rise/StageIndicator.vue'
import TransformationSession from '@/components/modes/rise/TransformationSession.vue'
import BeforeAfterProof from '@/components/modes/rise/BeforeAfterProof.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const starting = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const profile = ref<RiseModeProfileDto | null>(null)
interface LearnerWeakTopic extends TopicCaseDto {
  sourceTopicIds: number[]
}

const weakTopics = ref<LearnerWeakTopic[]>([])
const misconceptionSignals = ref<LearnerMisconceptionSnapshotDto[]>([])
const recentActivity = ref<StudentActivityHistoryItemDto[]>([])
const stageTransition = ref<StageTransitionResultDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    subjects.value = await listSubjects()
    const firstSubjectId = subjects.value[0]?.id ?? null
    selectedSubjectId.value = firstSubjectId

    if (firstSubjectId != null) {
      await loadProfile(firstSubjectId)
    }
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to load'
  } finally {
    loading.value = false
  }
})

async function loadProfile(subjectId: number) {
  if (!auth.currentAccount) return

  const studentId = auth.currentAccount.id
  const subjectName = subjects.value.find(subject => subject.id === subjectId)?.name ?? null
  const [initialProfile, subjectTopics, priorityTopics, signals, activity] = await Promise.all([
    getRiseModeProfile(studentId, subjectId),
    listTopics(subjectId).catch(() => []),
    getPriorityTopics(studentId, 40),
    listActiveMisconceptions(studentId, subjectId).catch(() => []),
    listStudentActivityHistory(studentId, 48).catch(() => []),
  ])

  stageTransition.value = await checkRiseModeTransition(studentId, subjectId).catch(() => null)
  const riseProfile = stageTransition.value
    ? await getRiseModeProfile(studentId, subjectId).then(value => value ?? initialProfile)
    : initialProfile

  const learnerIndex = buildLearnerTopicIndex(subjectTopics)
  const subjectTopicIds = new Set(subjectTopics.map(topic => topic.id))
  const filteredTopics = priorityTopics.filter(topic => subjectTopicIds.has(topic.topic_id))
  const groupedTopics = new Map<number, LearnerWeakTopic>()

  for (const topic of filteredTopics) {
    const learnerTopic = learnerIndex.bySourceTopicId.get(topic.topic_id)
    const groupId = learnerTopic?.id ?? topic.topic_id
    const sourceTopicIds = learnerTopic?.sourceTopicIds ?? [topic.topic_id]
    const existing = groupedTopics.get(groupId)

    if (!existing) {
      groupedTopics.set(groupId, {
        ...topic,
        topic_id: groupId,
        topic_name: learnerTopic?.name ?? topic.topic_name,
        sourceTopicIds: [...sourceTopicIds],
      })
      continue
    }

    existing.sourceTopicIds = Array.from(new Set([...existing.sourceTopicIds, ...sourceTopicIds]))
    existing.priority_score = Math.max(existing.priority_score, topic.priority_score)
    existing.gap_score = Math.max(existing.gap_score, topic.gap_score)
    existing.fragility_score = Math.max(existing.fragility_score, topic.fragility_score)
    existing.decay_risk = Math.max(existing.decay_risk, topic.decay_risk)
    existing.requires_probe = existing.requires_probe || topic.requires_probe

    if (topic.mastery_score <= existing.mastery_score) {
      existing.mastery_score = topic.mastery_score
      existing.mastery_state = topic.mastery_state
      existing.intervention_mode = topic.intervention_mode
      existing.intervention_urgency = topic.intervention_urgency
      existing.intervention_reason = topic.intervention_reason
    }
  }

  const learnerWeakTopics = Array.from(groupedTopics.values()).sort((left, right) =>
    right.priority_score - left.priority_score
    || left.mastery_score - right.mastery_score
    || right.gap_score - left.gap_score,
  )

  profile.value = riseProfile
  recentActivity.value = activity
    .filter(item => subjectName == null || item.subject === subjectName)
    .sort((left, right) => new Date(left.occurred_at).getTime() - new Date(right.occurred_at).getTime())
  weakTopics.value = learnerWeakTopics.filter(topic => topic.mastery_score < 5000).slice(0, 6)
  if (!weakTopics.value.length) {
    weakTopics.value = learnerWeakTopics.slice(0, 6)
  }
  misconceptionSignals.value = signals.sort((left, right) => right.risk_score - left.risk_score)
}

async function onSubjectChange(subjectId: number) {
  if (selectedSubjectId.value === subjectId) return
  selectedSubjectId.value = subjectId
  loading.value = true
  error.value = ''

  try {
    await loadProfile(subjectId)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to load profile'
  } finally {
    loading.value = false
  }
}

async function activateRiseMode() {
  if (!auth.currentAccount || !selectedSubjectId.value || starting.value) return

  starting.value = true
  error.value = ''
  try {
    await enterRiseMode(auth.currentAccount.id, selectedSubjectId.value)
    await loadProfile(selectedSubjectId.value)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to enter Rise Mode'
  } finally {
    starting.value = false
  }
}

async function launchSession() {
  if (!auth.currentAccount || !selectedSubjectId.value || starting.value) return

  starting.value = true
  error.value = ''
  try {
    const topicIds = Array.from(
      new Set(weakTopics.value.slice(0, 3).flatMap(topic => topic.sourceTopicIds)),
    )
    if (!topicIds.length) {
      throw new Error('No weak topics are available for this subject yet.')
    }

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: currentStage.value === 'accelerate' || currentStage.value === 'dominate',
    })

    router.push(`/student/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to start session'
    starting.value = false
  }
}

const currentStage = computed(() => profile.value?.current_stage ?? 'rescue')
const stageProgress = computed(() => {
  if (!profile.value) return 0
  return Math.round(profile.value.transformation_readiness_score / 100)
})

const selectedSubjectName = computed(() => {
  return subjects.value.find(subject => subject.id === selectedSubjectId.value)?.name ?? 'This subject'
})

const scoreDimensions = computed(() => {
  if (!profile.value) return []
  return [
    { label: 'Foundation', value: profile.value.foundation_score },
    { label: 'Recall', value: profile.value.recall_score },
    { label: 'Speed', value: profile.value.speed_score },
    { label: 'Accuracy', value: profile.value.accuracy_score },
    { label: 'Pressure', value: profile.value.pressure_stability_score },
    { label: 'Misconception', value: profile.value.misconception_density_score },
    { label: 'Momentum', value: profile.value.momentum_score },
    { label: 'Readiness', value: profile.value.transformation_readiness_score },
  ]
})

const topSignals = computed(() => misconceptionSignals.value.slice(0, 6))
const nextSessionNumber = computed(() => Math.max(recentActivity.value.length + 1, 1))

const proofCard = computed(() => {
  if (!weakTopics.value[0]) return null

  const firstRun = recentActivity.value[0]
  const latestRun = recentActivity.value[recentActivity.value.length - 1]
  const historyScoreToBp = (item: StudentActivityHistoryItemDto | undefined) => {
    if (!item) return null
    return item.score > 100 ? item.score : item.score * 100
  }
  const beforeScore = historyScoreToBp(firstRun) ?? weakTopics.value[0].gap_score
  const afterScore = historyScoreToBp(latestRun) ?? weakTopics.value[0].mastery_score
  const improvements: string[] = []

  if (latestRun && firstRun) {
    const delta = Math.round((afterScore - beforeScore) / 100)
    if (delta !== 0) {
      improvements.push(`Recent subject performance moved ${delta > 0 ? '+' : ''}${delta} points.`)
    }
  }
  if (stageTransition.value) {
    improvements.push(stageTransition.value.reason)
  }
  if (topSignals.value.length) {
    improvements.push(`${topSignals.value.length} active misconception pressure signal${topSignals.value.length === 1 ? '' : 's'} still being watched.`)
  }
  if (!improvements.length) {
    improvements.push(`Mastery on ${weakTopics.value[0].topic_name} is now ${Math.round(weakTopics.value[0].mastery_score / 100)}%.`)
  }

  return {
    topicName: weakTopics.value[0].topic_name,
    beforeScore,
    afterScore,
    beforeDate: firstRun ? new Date(firstRun.occurred_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric' }) : 'Gap',
    afterDate: latestRun ? new Date(latestRun.occurred_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric' }) : 'Now',
    improvements,
  }
})

function scoreColor(value: number): string {
  if (value >= 7000) return 'var(--accent)'
  if (value >= 4000) return 'var(--gold)'
  return 'var(--warm)'
}

function riskColor(score: number): string {
  if (score >= 7000) return 'var(--warm)'
  if (score >= 4500) return 'var(--gold)'
  return 'var(--ink)'
}

function formatPercent(bp: number): string {
  return `${Math.round(bp / 100)}%`
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Rise Mode</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Transform from struggling to dominating
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          {{ selectedSubjectName }} - real weakness map plus active misconception pressure
        </p>
      </div>
      <div class="flex items-center gap-3">
        <div v-if="profile" class="text-right">
          <p class="text-lg font-black tabular-nums" :style="{ color: scoreColor(profile.transformation_readiness_score) }">
            {{ stageProgress }}%
          </p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">
            {{ profile.stage_label }}
          </p>
        </div>
        <div class="flex gap-1.5">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="subject-tab"
            :class="{ active: selectedSubjectId === subject.id }"
            @click="onSubjectChange(subject.id)"
          >
            {{ subject.name }}
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="error"
      class="px-7 py-2 text-xs flex-shrink-0"
      :style="{ background: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }"
    >
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div
        v-for="i in 3"
        :key="i"
        class="h-20 rounded-lg animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }"
      />
    </div>

    <div v-else-if="!profile" class="flex-1 flex items-center justify-center p-8">
      <div class="max-w-2xl w-full">
        <div class="text-center mb-8">
          <h2 class="font-display text-2xl font-bold mb-3" :style="{ color: 'var(--ink)' }">Start Your Transformation</h2>
          <p class="text-sm max-w-sm mx-auto" :style="{ color: 'var(--ink-muted)' }">
            Rise Mode uses your recorded weakness map and misconception state to build a precision recovery path for this subject.
          </p>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-8">
          <div
            v-for="item in [
              { title: 'Deep Diagnosis', desc: 'Find root causes, not just scores' },
              { title: 'Foundation Repair', desc: 'Rebuild from missing building blocks' },
              { title: 'Speed Training', desc: 'Think faster under pressure' },
              { title: 'Dominate', desc: 'Push toward elite performance' },
            ]"
            :key="item.title"
            class="feature-card"
          >
            <p class="text-xs font-bold mb-1" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
            <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ item.desc }}</p>
          </div>
        </div>
        <div class="text-center">
          <button class="rise-cta" :disabled="starting" @click="activateRiseMode">
            {{ starting ? 'Activating...' : 'Begin Rise Mode ->' }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div>
          <StageIndicator :current-stage="currentStage" />
          <div class="mt-3 flex items-center gap-3">
            <div class="flex-1">
              <AppProgress :value="stageProgress" size="md" color="warm" :glow="true" />
            </div>
            <span class="text-sm font-bold tabular-nums" :style="{ color: scoreColor(profile.transformation_readiness_score) }">
              {{ stageProgress }}%
            </span>
          </div>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ profile.stage_purpose }}</p>
        </div>

        <TransformationSession
          :stage="currentStage as any"
          :topic-name="weakTopics[0]?.topic_name || 'Your weakest topic'"
          :session-number="nextSessionNumber"
          :progress="stageProgress"
          @start="launchSession"
        />

        <div
          v-if="stageTransition"
          class="rounded-lg px-4 py-3 text-xs"
          :style="{ backgroundColor: 'rgba(16,185,129,0.08)', color: 'var(--accent)' }"
        >
          {{ stageTransition.reason }}
        </div>

        <div v-if="weakTopics.length">
          <p class="section-label mb-3">Weakness Map</p>
          <div class="space-y-2">
            <div
              v-for="topic in weakTopics"
              :key="topic.topic_id"
              class="weakness-row flex items-center gap-3 px-4 py-3 rounded-lg border"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
            >
              <MasteryBadge :state="topic.mastery_state" size="sm" />
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ topic.intervention_reason }}</p>
              </div>
              <span class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }">
                {{ Math.round(topic.mastery_score / 100) }}%
              </span>
            </div>
          </div>
        </div>

        <div v-if="topSignals.length">
          <p class="section-label mb-3">Misconception Pressure</p>
          <div class="space-y-2">
            <div
              v-for="signal in topSignals"
              :key="`${signal.subject_id}-${signal.misconception_id}`"
              class="signal-row"
            >
              <div class="flex items-start gap-3">
                <div class="w-2 h-2 rounded-full flex-shrink-0 mt-1.5" :style="{ backgroundColor: riskColor(signal.risk_score) }" />
                <div class="flex-1 min-w-0">
                  <p class="text-[12px] font-semibold" :style="{ color: 'var(--ink)' }">{{ signal.title }}</p>
                  <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ signal.topic_name ?? selectedSubjectName }}</p>
                  <p class="text-[10px] mt-1 capitalize" :style="{ color: 'var(--ink-secondary)' }">
                    {{ signal.current_status.replace(/_/g, ' ') }} - detected {{ signal.times_detected }}x
                  </p>
                </div>
                <span class="text-[10px] font-bold flex-shrink-0" :style="{ color: riskColor(signal.risk_score) }">
                  {{ formatPercent(signal.risk_score) }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <BeforeAfterProof
          v-if="proofCard"
          :topic-name="proofCard.topicName"
          :before-score="proofCard.beforeScore"
          :after-score="proofCard.afterScore"
          :before-date="proofCard.beforeDate"
          :after-date="proofCard.afterDate"
          :improvements="proofCard.improvements"
        />
      </div>

      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Transformation Scores</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4">
          <div class="grid grid-cols-2 gap-3">
            <div v-for="dimension in scoreDimensions" :key="dimension.label" class="dimension-card text-center">
              <p class="text-2xl font-black tabular-nums" :style="{ color: scoreColor(dimension.value) }">
                {{ Math.round(dimension.value / 100) }}<span class="text-sm">%</span>
              </p>
              <p class="text-[9px] uppercase font-semibold mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ dimension.label }}</p>
              <div class="h-1 rounded-full mt-2 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full" :style="{ width: `${dimension.value / 100}%`, backgroundColor: scoreColor(dimension.value) }" />
              </div>
            </div>
          </div>
        </div>

        <div class="p-4 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="nav-btn w-full" @click="router.push('/student/elite')">Elite Mode -></button>
          <button class="nav-btn w-full" @click="router.push('/student/practice')">Practice -></button>
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

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.subject-tab {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}

.subject-tab.active,
.subject-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.feature-card,
.signal-row,
.dimension-card {
  border-radius: 8px;
  border: 1px solid transparent;
  background: var(--surface);
}

.feature-card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.signal-row {
  padding: 12px;
}

.rise-cta {
  padding: 13px 40px;
  border-radius: 8px;
  background: var(--accent);
  color: white;
  font-size: 15px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 140ms, transform 140ms;
  border: none;
}

.rise-cta:hover:not(:disabled) {
  opacity: 0.87;
  transform: translateY(-2px);
}

.rise-cta:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.weakness-row {
  transition: background-color 100ms;
}

.weakness-row:hover {
  background-color: var(--paper) !important;
}

.dimension-card {
  padding: 14px 10px;
  background: var(--paper);
}

.nav-btn {
  padding: 8px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}

.nav-btn:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
