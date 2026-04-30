<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getJourneyStation,
  getActiveJourneyRoute,
  completeJourneyStation,
  type JourneyStationDetail,
  type JourneyRouteSnapshot,
} from '@/ipc/journey'
import { listTopics, type TopicDto } from '@/ipc/coach'
import { buildLearnerTopicIndex } from '@/utils/learnerTopics'
import { startPracticeSessionWithQuestions } from '@/utils/sessionQuestions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const stationId = computed(() => Number(route.params.id))
const loading = ref(true)
const starting = ref(false)
const error = ref('')
const station = ref<JourneyStationDetail | null>(null)
const topics = ref<TopicDto[]>([])
const routeSnapshot = ref<JourneyRouteSnapshot | null>(null)

onMounted(async () => {
  try {
    station.value = await getJourneyStation(stationId.value)
    if (station.value?.subject_id && station.value.student_id) {
      const [loadedTopics, snapshot] = await Promise.all([
        listTopics(station.value.subject_id),
        getActiveJourneyRoute(station.value.student_id, station.value.subject_id).catch(() => null),
      ])
      topics.value = loadedTopics
      routeSnapshot.value = snapshot
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load station'
  }
  loading.value = false
})

const learnerTopicIndex = computed(() => buildLearnerTopicIndex(topics.value))
const stationRouteEntry = computed(() =>
  routeSnapshot.value?.stations.find(entry => entry.id === stationId.value) ?? null,
)

function dedupeTopicIds(values: Array<number | null | undefined>): number[] {
  return Array.from(new Set(values.filter((value): value is number => Number.isFinite(value))))
}

const stationFocusTopicIds = computed(() => {
  const topicId = stationRouteEntry.value?.topic_id
  if (topicId == null) return []
  const learnerTopic = learnerTopicIndex.value.bySourceTopicId.get(topicId)
  if (learnerTopic) return learnerTopic.sourceTopicIds
  return topics.value.some(topic => topic.id === topicId) ? [topicId] : []
})

const routeTopicIds = computed(() =>
  dedupeTopicIds(routeSnapshot.value?.stations.map(entry => entry.topic_id) ?? []),
)

const launchTopicIds = computed(() => {
  if (stationFocusTopicIds.value.length > 0) return stationFocusTopicIds.value
  return routeTopicIds.value
})

const stationScopeLabel = computed(() => {
  const topicId = stationRouteEntry.value?.topic_id
  if (topicId != null) {
    const learnerTopic = learnerTopicIndex.value.bySourceTopicId.get(topicId)
    const topicName = learnerTopic?.name ?? topics.value.find(topic => topic.id === topicId)?.name ?? null
    return topicName
      ? `Focus topic: ${topicName}`
      : 'This station is linked to a focused topic practice set.'
  }

  if (routeTopicIds.value.length > 0) {
    return `Mixed route practice across ${routeTopicIds.value.length} linked topics.`
  }

  return 'This station will open the closest available route practice set.'
})

const launchButtonLabel = computed(() =>
  stationFocusTopicIds.value.length > 0 ? 'Start Focused Practice ->' : 'Start Route Practice ->',
)

const stationPhase = computed(() => {
  const type = station.value?.station_type ?? ''
  const phaseMap: Record<string, string> = {
    review: 'Review and Verify',
    foundation: 'Foundation Repair',
    repair: 'Targeted Repair',
    checkpoint: 'Checkpoint',
    performance: 'Performance Push',
    reactivation: 'Reactivation',
    readiness_gate: 'Readiness Gate',
    build: 'Build the Core',
    strengthen: 'Strengthen Weak Links',
    conditioning: 'Exam Conditioning',
    final: 'Final Readiness',
  }
  return phaseMap[type] ?? type
})

const statusColor = computed(() => {
  const s = station.value?.status ?? ''
  if (s === 'completed') return 'success'
  if (s === 'active' || s === 'passed') return 'accent'
  return 'muted'
})

async function launchSession() {
  if (!auth.currentAccount || !station.value || starting.value) return
  starting.value = true
  error.value = ''
  try {
    const topicIds = launchTopicIds.value
    if (topicIds.length === 0) {
      throw new Error('No linked topics are ready for this station yet.')
    }
    const fallbackTopicSets = topicIds.length > 1 ? [[topicIds[0]]] : []
    const session = await startPracticeSessionWithQuestions({
      student_id: auth.currentAccount.id,
      subject_id: station.value.subject_id,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: false,
    }, fallbackTopicSets)
    router.push(`/student/session/${session.sessionId}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
    starting.value = false
  }
}

async function markComplete() {
  if (!station.value || starting.value) return
  starting.value = true
  try {
    await completeJourneyStation(stationId.value, {})
    router.push('/student/journey')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to complete station'
    starting.value = false
  }
}
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <!-- Back -->
    <button
      class="flex items-center gap-1 text-xs mb-6 hover:underline"
      :style="{ color: 'var(--ink-muted)' }"
      @click="router.push('/student/journey')"
    >
      Back to Journey
    </button>

    <!-- Loading -->
    <div v-if="loading" class="space-y-4">
      <div class="h-32 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div class="h-48 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Error -->
    <div v-else-if="error && !station" class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--warm)' }">{{ error }}</p>
      <AppButton variant="secondary" @click="router.push('/student/journey')">Back</AppButton>
    </div>

    <template v-else-if="station">
      <!-- Station Header -->
      <AppCard padding="lg" glow="accent" class="mb-6">
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1">
            <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--ink-muted)' }">
              {{ stationPhase }}
            </p>
            <h1 class="font-display text-2xl font-bold mb-3" :style="{ color: 'var(--ink)' }">
              {{ station.title }}
            </h1>
            <div class="flex items-center gap-3">
              <AppBadge :color="(statusColor as any)" size="sm">{{ station.status }}</AppBadge>
              <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">Station #{{ station.sequence_no }}</span>
            </div>
          </div>
          <!-- Circular progress -->
          <div class="text-right">
            <p class="font-display text-3xl font-bold" :style="{ color: 'var(--accent)' }">
              {{ (station.progress_score / 100).toFixed(0) }}%
            </p>
            <p class="text-[10px] uppercase mt-0.5" :style="{ color: 'var(--ink-muted)' }">Progress</p>
          </div>
        </div>
        <div class="mt-4">
          <AppProgress
            :value="station.progress_score"
            :max="10000"
            size="md"
            :color="station.status === 'completed' ? 'success' : 'accent'"
          />
        </div>
      </AppCard>

      <!-- Error message -->
      <div v-if="error" class="mb-4 p-3 rounded-lg text-sm"
        :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
        {{ error }}
      </div>

      <!-- Station Description -->
      <AppCard padding="lg" class="mb-6">
        <h3 class="text-sm font-semibold mb-3" :style="{ color: 'var(--ink)' }">What to do in this station</h3>
        <div class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
          <template v-if="station.station_type === 'foundation'">
            <p>- Work through core concepts and fix fundamental gaps.</p>
            <p>- Complete practice sessions until you reach a solid baseline.</p>
            <p>- Focus on understanding, not speed.</p>
          </template>
          <template v-else-if="station.station_type === 'review'">
            <p>- Revisit the route evidence already gathered for this subject.</p>
            <p>- Confirm what has held up and what still needs another pass.</p>
            <p>- Use the linked practice as a verification round, not a fresh sprint.</p>
          </template>
          <template v-else-if="station.station_type === 'repair'">
            <p>- Attack the exact weaknesses the route has surfaced.</p>
            <p>- Stay inside the linked topic until the pattern feels stable.</p>
            <p>- Prioritize clean understanding over breadth.</p>
          </template>
          <template v-else-if="station.station_type === 'checkpoint'">
            <p>- Take a clean measurement of the route so far.</p>
            <p>- Treat this as a confidence check, not a cram session.</p>
            <p>- Let the result decide whether the route advances or loops back.</p>
          </template>
          <template v-else-if="station.station_type === 'performance'">
            <p>- Work under sharper exam pressure.</p>
            <p>- Push for accuracy and pace together.</p>
            <p>- Use this station to prove the route can hold up in performance conditions.</p>
          </template>
          <template v-else-if="station.station_type === 'reactivation'">
            <p>- Wake up ideas that have gone quiet since the last strong run.</p>
            <p>- Use this station to restore speed of recall and clean execution.</p>
            <p>- Keep the session short, deliberate, and accurate.</p>
          </template>
          <template v-else-if="station.station_type === 'readiness_gate'">
            <p>- This gate checks whether the route is ready to move on.</p>
            <p>- Treat the linked practice as a final proof round.</p>
            <p>- A clear result matters more than volume here.</p>
          </template>
          <template v-else-if="station.station_type === 'build'">
            <p>- Build depth on the main syllabus topics.</p>
            <p>- Practice increasingly harder questions.</p>
            <p>- Connect concepts across topics.</p>
          </template>
          <template v-else-if="station.station_type === 'strengthen'">
            <p>- Attack your weak topics identified from earlier sessions.</p>
            <p>- Review recurring mistakes and understand the root cause.</p>
            <p>- Boost your confidence on tricky areas.</p>
          </template>
          <template v-else-if="station.station_type === 'conditioning'">
            <p>- Train under timed, exam-like conditions.</p>
            <p>- Practice mixed-topic question sets.</p>
            <p>- Develop speed and accuracy together.</p>
          </template>
          <template v-else>
            <p>- Final revision burst before exam day.</p>
            <p>- Focus on high-yield topics and exam strategy.</p>
            <p>- Build confidence and mental readiness.</p>
          </template>
        </div>
        <div class="mt-4 pt-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="text-[10px] font-semibold uppercase tracking-[0.18em] mb-2" :style="{ color: 'var(--ink-muted)' }">
            Practice Scope
          </p>
          <p class="text-sm" :style="{ color: 'var(--ink)' }">{{ stationScopeLabel }}</p>
        </div>
      </AppCard>

      <!-- Actions -->
      <div class="flex flex-wrap items-center gap-3">
        <AppButton
          v-if="station.status !== 'completed'"
          variant="primary"
          size="lg"
          :loading="starting"
          @click="launchSession"
        >
          {{ launchButtonLabel }}
        </AppButton>

        <AppButton
          v-if="station.status === 'passed'"
          variant="secondary"
          :loading="starting"
          @click="markComplete"
        >
          Advance Station
        </AppButton>

        <AppButton
          v-if="station.status === 'completed'"
          variant="secondary"
          :loading="starting"
          @click="launchSession"
        >
          {{ stationFocusTopicIds.length > 0 ? 'Practice Focus Topic Again' : 'Practice Route Again' }}
        </AppButton>

        <AppButton variant="ghost" size="sm" @click="router.push('/student/journey')">Back</AppButton>
      </div>
    </template>
  </div>
</template>

