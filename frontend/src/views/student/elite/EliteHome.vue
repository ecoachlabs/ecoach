<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, listSubjects, type LearnerTruthDto, type SubjectDto } from '@/ipc/coach'
import { getEliteProfile, listEliteTopicDomination, buildEliteSessionBlueprint, type EliteProfileDto, type EliteTopicProfileDto } from '@/ipc/elite'
import { startPracticeSession } from '@/ipc/sessions'
import AppProgress from '@/components/ui/AppProgress.vue'
import EliteIdentityPanel from '@/components/modes/elite/EliteIdentityPanel.vue'
import EliteSessionArena from '@/components/modes/elite/EliteSessionArena.vue'
import EliteTierProgress from '@/components/modes/elite/EliteTierProgress.vue'
import TopicDominationBoard from '@/components/modes/elite/TopicDominationBoard.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const starting = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const selectedSession = ref<string | undefined>(undefined)
const truth = ref<LearnerTruthDto | null>(null)
const profile = ref<EliteProfileDto | null>(null)
const topicDomination = ref<EliteTopicProfileDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, subs] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      listSubjects(),
    ])
    truth.value = t
    subjects.value = subs
    if (subs.length > 0) {
      selectedSubjectId.value = subs[0].id
      await loadSubjectData(subs[0].id)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load Elite Mode'
  }
  loading.value = false
})

async function loadSubjectData(subjectId: number) {
  if (!auth.currentAccount) return
  const studentId = auth.currentAccount.id
  const [p, topics] = await Promise.all([
    getEliteProfile(studentId, subjectId),
    listEliteTopicDomination(studentId, subjectId, 8),
  ])
  profile.value = p
  topicDomination.value = topics
}

async function onSubjectChange(subjectId: number) {
  selectedSubjectId.value = subjectId
  loading.value = true
  try {
    await loadSubjectData(subjectId)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load subject data'
  }
  loading.value = false
}

async function startSession() {
  if (!auth.currentAccount || !selectedSubjectId.value || !selectedSession.value || starting.value) return
  starting.value = true
  error.value = ''
  try {
    const blueprint = await buildEliteSessionBlueprint(auth.currentAccount.id, selectedSubjectId.value)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: blueprint.target_topic_ids as number[],
      question_count: blueprint.target_question_count,
      is_timed: selectedSession.value === 'sprint' || selectedSession.value === 'endurance' || selectedSession.value === 'apex',
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
    starting.value = false
  }
}

const identityDimensions = computed(() => {
  if (!profile.value) return []
  return [
    { label: 'Precision', score: profile.value.precision_score, icon: '◎' },
    { label: 'Speed', score: profile.value.speed_score, icon: '⚡' },
    { label: 'Depth', score: profile.value.depth_score, icon: '◈' },
    { label: 'Composure', score: profile.value.composure_score, icon: '∞' },
    { label: 'EPS', score: profile.value.eps_score, icon: '★' },
  ]
})

const tier = computed(() => profile.value?.tier ?? 'Foundation')
const progressToNext = computed(() => {
  if (!profile.value) return 0
  return Math.round(profile.value.eps_score / 100)
})
const nextTierLabel = computed(() => {
  const order = ['Foundation', 'Core', 'Prime', 'Apex', 'Master', 'Legend']
  const idx = order.indexOf(tier.value)
  return idx >= 0 && idx < order.length - 1 ? order[idx + 1] : 'Legend'
})

const dominationTopics = computed(() =>
  topicDomination.value.map(t => ({
    name: t.topic_name,
    status: t.status,
    dominationScore: t.domination_score,
    accuracy: t.precision_score,
    speed: t.speed_score,
    trapResistance: t.trap_resistance_score,
  })),
)
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" data-mode="elite" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center gap-6 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex-1">
        <p class="eyebrow">Elite Mode</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Push your performance limits
        </h1>
      </div>

      <!-- Tier display -->
      <div v-if="profile" class="text-right">
        <p class="text-lg font-black" :style="{ color: 'var(--gold)' }">{{ tier }}</p>
        <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Current Tier · EPS {{ Math.round(profile.eps_score / 100) }}%</p>
      </div>

      <!-- Subject tabs -->
      <div class="flex gap-1.5">
        <button
          v-for="subj in subjects"
          :key="subj.id"
          class="subj-tab"
          :class="{ active: selectedSubjectId === subj.id }"
          @click="onSubjectChange(subj.id)"
        >{{ subj.name }}</button>
      </div>

      <!-- Nav -->
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/elite/records')">Records</button>
        <button class="nav-pill" @click="router.push('/student/elite/insights')">Insights</button>
        <button class="nav-pill" @click="router.push('/student/elite/arena')">Arena</button>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: identity + session arena -->
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div v-if="!profile"
          class="px-5 py-4 rounded-2xl border"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
          <p class="text-sm font-semibold mb-1" :style="{ color: 'var(--gold)' }">Elite Mode activates automatically</p>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
            Start a session below — Elite Mode unlocks when you meet the performance bar.
          </p>
        </div>

        <div v-if="identityDimensions.length">
          <EliteIdentityPanel :dimensions="identityDimensions" />
        </div>

        <div v-if="profile">
          <EliteTierProgress
            :current-tier="tier.toLowerCase()"
            :next-tier="nextTierLabel"
            :progress-to-next="progressToNext"
            :eps="profile.eps_score"
          />
        </div>

        <EliteSessionArena
          :selected-session="selectedSession"
          @select="selectedSession = $event"
          @start="startSession"
        />
        <div v-if="starting" class="text-xs text-center" :style="{ color: 'var(--ink-muted)' }">
          Building your Elite session…
        </div>
      </div>

      <!-- Right: topic domination board -->
      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0 flex items-center justify-between"
          :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Topic Domination</p>
          <span class="text-[10px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ dominationTopics.length }} topics</span>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-if="dominationTopics.length">
            <TopicDominationBoard
              :topics="dominationTopics"
              @select-topic="router.push('/student/elite/insights')"
            />
          </div>
          <div v-else class="py-12 text-center px-5">
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
              No domination data yet.<br>Start a session to begin tracking.
            </p>
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
  color: var(--gold);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
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

.nav-pill {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.nav-pill:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
