<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getJourneyStation,
  completeJourneyStation,
  type JourneyStationDetail,
} from '@/ipc/journey'
import { startPracticeSession } from '@/ipc/sessions'
import { listTopics, type TopicDto } from '@/ipc/coach'
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

onMounted(async () => {
  try {
    station.value = await getJourneyStation(stationId.value)
    // Load topics for the subject so we can start a focused session
    if (station.value?.subject_id) {
      topics.value = await listTopics(station.value.subject_id)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load station'
  }
  loading.value = false
})

const stationPhase = computed(() => {
  const type = station.value?.station_type ?? ''
  const phaseMap: Record<string, string> = {
    foundation: 'Phase 1 — Stabilize the Foundation',
    build: 'Phase 2 — Build the Core',
    strengthen: 'Phase 3 — Strengthen Weak Links',
    conditioning: 'Phase 4 — Exam Conditioning',
    final: 'Phase 5 — Final Readiness',
  }
  return phaseMap[type] ?? type
})

const statusColor = computed(() => {
  const s = station.value?.status ?? ''
  if (s === 'completed') return 'success'
  if (s === 'active') return 'accent'
  return 'muted'
})

async function launchSession() {
  if (!auth.currentAccount || !station.value || starting.value) return
  starting.value = true
  error.value = ''
  try {
    // Start a practice session for this station's subject, focused on first 3 topics
    const topicIds = topics.value.slice(0, 3).map(t => t.id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: station.value.subject_id,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
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
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <!-- Back -->
    <button
      class="flex items-center gap-1 text-xs mb-6 hover:underline"
      :style="{ color: 'var(--text-3)' }"
      @click="router.push('/student/journey')"
    >
      ← Back to Journey
    </button>

    <!-- Loading -->
    <div v-if="loading" class="space-y-4">
      <div class="h-32 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div class="h-48 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Error -->
    <div v-else-if="error && !station" class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--danger)' }">{{ error }}</p>
      <AppButton variant="secondary" @click="router.push('/student/journey')">Back</AppButton>
    </div>

    <template v-else-if="station">
      <!-- Station Header -->
      <AppCard padding="lg" glow="accent" class="mb-6">
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1">
            <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--text-3)' }">
              {{ stationPhase }}
            </p>
            <h1 class="font-display text-2xl font-bold mb-3" :style="{ color: 'var(--text)' }">
              {{ station.title }}
            </h1>
            <div class="flex items-center gap-3">
              <AppBadge :color="(statusColor as any)" size="sm">{{ station.status }}</AppBadge>
              <span class="text-xs" :style="{ color: 'var(--text-3)' }">Station #{{ station.sequence_no }}</span>
            </div>
          </div>
          <!-- Circular progress -->
          <div class="text-right">
            <p class="font-display text-3xl font-bold" :style="{ color: 'var(--accent)' }">
              {{ (station.progress_score / 100).toFixed(0) }}%
            </p>
            <p class="text-[10px] uppercase mt-0.5" :style="{ color: 'var(--text-3)' }">Progress</p>
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
        :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">
        {{ error }}
      </div>

      <!-- Station Description -->
      <AppCard padding="lg" class="mb-6">
        <h3 class="text-sm font-semibold mb-3" :style="{ color: 'var(--text)' }">What to do in this station</h3>
        <div class="space-y-2 text-sm" :style="{ color: 'var(--text-2)' }">
          <template v-if="station.station_type === 'foundation'">
            <p>• Work through core concepts and fix fundamental gaps.</p>
            <p>• Complete practice sessions until you reach a solid baseline.</p>
            <p>• Focus on understanding, not speed.</p>
          </template>
          <template v-else-if="station.station_type === 'build'">
            <p>• Build depth on the main syllabus topics.</p>
            <p>• Practice increasingly harder questions.</p>
            <p>• Connect concepts across topics.</p>
          </template>
          <template v-else-if="station.station_type === 'strengthen'">
            <p>• Attack your weak topics identified from earlier sessions.</p>
            <p>• Review recurring mistakes and understand the root cause.</p>
            <p>• Boost your confidence on tricky areas.</p>
          </template>
          <template v-else-if="station.station_type === 'conditioning'">
            <p>• Train under timed, exam-like conditions.</p>
            <p>• Practice mixed-topic question sets.</p>
            <p>• Develop speed and accuracy together.</p>
          </template>
          <template v-else>
            <p>• Final revision burst before exam day.</p>
            <p>• Focus on high-yield topics and exam strategy.</p>
            <p>• Build confidence and mental readiness.</p>
          </template>
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
          Start Practice Session →
        </AppButton>

        <AppButton
          v-if="station.status === 'active' && station.progress_score >= 7000"
          variant="secondary"
          :loading="starting"
          @click="markComplete"
        >
          Mark Station Complete ✓
        </AppButton>

        <AppButton
          v-if="station.status === 'completed'"
          variant="secondary"
          :loading="starting"
          @click="launchSession"
        >
          Practice Again
        </AppButton>

        <AppButton variant="ghost" size="sm" @click="router.push('/student/journey')">Back</AppButton>
      </div>
    </template>
  </div>
</template>
