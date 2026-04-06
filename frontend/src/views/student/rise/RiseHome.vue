<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getPriorityTopics, listSubjects, type TopicCaseDto, type SubjectDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import { enterRiseMode, getRiseModeProfile, type RiseModeProfileDto } from '@/ipc/rise'
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
const weakTopics = ref<TopicCaseDto[]>([])
const sessionCount = ref(1)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    subjects.value = await listSubjects()
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
      await loadProfile()
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  }
  loading.value = false
})

async function loadProfile() {
  if (!auth.currentAccount || !selectedSubjectId.value) return
  const studentId = auth.currentAccount.id
  const subjectId = selectedSubjectId.value
  profile.value = await getRiseModeProfile(studentId, subjectId)
  const topics = await getPriorityTopics(studentId, 20)
  weakTopics.value = topics.filter(t => t.mastery_score < 5000).slice(0, 6)
}

async function onSubjectChange(subjectId: number) {
  selectedSubjectId.value = subjectId
  loading.value = true
  error.value = ''
  try { await loadProfile() } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load profile'
  }
  loading.value = false
}

async function activateRiseMode() {
  if (!auth.currentAccount || !selectedSubjectId.value || starting.value) return
  starting.value = true
  error.value = ''
  try {
    profile.value = await enterRiseMode(auth.currentAccount.id, selectedSubjectId.value)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to enter Rise Mode'
  }
  starting.value = false
}

async function launchSession() {
  if (!auth.currentAccount || !selectedSubjectId.value || starting.value) return
  starting.value = true
  error.value = ''
  try {
    const topicIds = weakTopics.value.slice(0, 3).map(t => t.topic_id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: currentStage.value === 'accelerate' || currentStage.value === 'dominate',
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
    starting.value = false
  }
}

const currentStage = computed(() => profile.value?.current_stage ?? 'rescue')
const stageProgress = computed(() => {
  if (!profile.value) return 0
  return Math.round(profile.value.transformation_readiness_score / 100)
})

const scoreDimensions = computed(() => {
  if (!profile.value) return []
  return [
    { label: 'Foundation', value: profile.value.foundation_score },
    { label: 'Recall', value: profile.value.recall_score },
    { label: 'Speed', value: profile.value.speed_score },
    { label: 'Accuracy', value: profile.value.accuracy_score },
    { label: 'Pressure', value: profile.value.pressure_stability_score },
    { label: 'Confidence', value: profile.value.confidence_score },
    { label: 'Momentum', value: profile.value.momentum_score },
    { label: 'Readiness', value: profile.value.transformation_readiness_score },
  ]
})

function scoreColor(v: number): string {
  if (v >= 7000) return 'var(--accent)'
  if (v >= 4000) return 'var(--gold)'
  return 'var(--warm)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Rise Mode</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Transform from struggling to dominating
        </h1>
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
            v-for="subj in subjects"
            :key="subj.id"
            class="subj-tab"
            :class="{ active: selectedSubjectId === subj.id }"
            @click="onSubjectChange(subj.id)"
          >{{ subj.name }}</button>
        </div>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Activation CTA -->
    <div v-else-if="!profile" class="flex-1 flex items-center justify-center p-8">
      <div class="max-w-2xl w-full">
        <div class="text-center mb-8">
          <h2 class="font-display text-2xl font-bold mb-3" :style="{ color: 'var(--ink)' }">Start Your Transformation</h2>
          <p class="text-sm max-w-sm mx-auto" :style="{ color: 'var(--ink-muted)' }">
            Rise Mode diagnoses your exact weaknesses and builds a precision recovery plan — from struggling to the top.
          </p>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-8">
          <div v-for="item in [
            { title: 'Deep Diagnosis', desc: 'Find root causes, not just scores' },
            { title: 'Foundation Repair', desc: 'Rebuild from missing building blocks' },
            { title: 'Speed Training', desc: 'Think faster under pressure' },
            { title: 'Dominate', desc: 'Outperform the top students' },
          ]" :key="item.title"
            class="feature-card"
          >
            <p class="text-xs font-bold mb-1" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
            <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ item.desc }}</p>
          </div>
        </div>
        <div class="text-center">
          <button class="rise-cta" :disabled="starting" @click="activateRiseMode">
            {{ starting ? 'Activating…' : 'Begin Rise Mode →' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Active profile -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: stage + session + weakness -->
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
          :stage="(currentStage as any)"
          :topic-name="weakTopics[0]?.topic_name || 'Your weakest topic'"
          :session-number="sessionCount"
          :progress="stageProgress"
          @start="launchSession"
        />

        <div v-if="weakTopics.length">
          <p class="section-label mb-3">Weakness Map</p>
          <div class="space-y-2">
            <div v-for="topic in weakTopics" :key="topic.topic_id"
              class="weakness-row flex items-center gap-3 px-4 py-3 rounded-xl border"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
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

        <BeforeAfterProof
          v-if="weakTopics[0]"
          :topic-name="weakTopics[0].topic_name"
          :before-score="weakTopics[0].gap_score"
          :after-score="weakTopics[0].mastery_score"
          before-date="Gap"
          after-date="Now"
          :improvements="['Accuracy improving', 'Fewer careless errors', 'Speed increasing']"
        />
      </div>

      <!-- Right: 8 dimension scores -->
      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Transformation Scores</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4">
          <div class="grid grid-cols-2 gap-3">
            <div v-for="dim in scoreDimensions" :key="dim.label" class="dim-card text-center">
              <p class="text-2xl font-black tabular-nums" :style="{ color: scoreColor(dim.value) }">
                {{ Math.round(dim.value / 100) }}<span class="text-sm">%</span>
              </p>
              <p class="text-[9px] uppercase font-semibold mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ dim.label }}</p>
              <div class="h-1 rounded-full mt-2 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full"
                  :style="{ width: (dim.value / 100) + '%', backgroundColor: scoreColor(dim.value) }" />
              </div>
            </div>
          </div>
        </div>

        <div class="p-4 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="nav-btn w-full" @click="router.push('/student/elite')">Elite Mode →</button>
          <button class="nav-btn w-full" @click="router.push('/student/practice')">Practice →</button>
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

.subj-tab {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--border-soft);
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}
.subj-tab.active, .subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.feature-card {
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  background: var(--surface);
  border: 1px solid var(--border-soft);
}

.rise-cta {
  padding: 13px 40px;
  border-radius: 14px;
  background: var(--accent);
  color: white;
  font-size: 15px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 140ms, transform 140ms;
}
.rise-cta:hover:not(:disabled) { opacity: 0.87; transform: translateY(-2px); }
.rise-cta:disabled { opacity: 0.5; cursor: not-allowed; }

.weakness-row { transition: background-color 100ms; }
.weakness-row:hover { background-color: var(--paper) !important; }

.dim-card {
  padding: 14px 10px;
  border-radius: 12px;
  border: 1px solid var(--border-soft);
  background: var(--paper);
}

.nav-btn {
  padding: 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.nav-btn:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
</style>
