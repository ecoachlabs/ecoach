<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import {
  getActiveJourneyRoute,
  buildOrRefreshJourneyRoute,
  type JourneyRouteSnapshot,
  type JourneyStation,
} from '@/ipc/journey'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const building = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const snapshot = ref<JourneyRouteSnapshot | null>(null)

const phases = ['Stabilize', 'Build', 'Strengthen', 'Condition', 'Ready']

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
      await loadRoute(subjects.value[0].id)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load journey'
  }
  loading.value = false
})

async function loadRoute(subjectId: number) {
  if (!auth.currentAccount) return
  try {
    snapshot.value = await getActiveJourneyRoute(auth.currentAccount.id, subjectId)
  } catch {}
}

async function changeSubject(id: number) {
  selectedSubjectId.value = id
  snapshot.value = null
  loading.value = true
  await loadRoute(id)
  loading.value = false
}

async function startJourney() {
  if (!auth.currentAccount || selectedSubjectId.value === null || building.value) return
  building.value = true
  error.value = ''
  try {
    snapshot.value = await buildOrRefreshJourneyRoute(
      auth.currentAccount.id,
      selectedSubjectId.value,
      'BECE',
    )
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to build journey'
  }
  building.value = false
}

const currentStations = computed(() => snapshot.value?.stations ?? [])

const completedCount = computed(() =>
  currentStations.value.filter(s => s.status === 'completed').length
)

const totalCount = computed(() => currentStations.value.length)

const currentStationCode = computed(() => snapshot.value?.route.current_station_code)

function stationStatusColor(status: string): string {
  if (status === 'completed') return 'success'
  if (status === 'active') return 'accent'
  if (status === 'locked') return 'muted'
  return 'muted'
}

function stationIcon(status: string): string {
  if (status === 'completed') return '✓'
  if (status === 'active') return '▶'
  return '○'
}

function phaseForStation(s: JourneyStation): string {
  // Map station type to phase name
  const typeMap: Record<string, string> = {
    foundation: 'Stabilize',
    build: 'Build',
    strengthen: 'Strengthen',
    conditioning: 'Condition',
    final: 'Ready',
  }
  return typeMap[s.station_type] ?? s.station_type
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Journey</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Your path from where you are to exam readiness.</p>
    </div>

    <!-- Error -->
    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm"
      :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">
      {{ error }}
    </div>

    <!-- Subject Selector -->
    <div class="flex gap-2 mb-6 flex-wrap">
      <button
        v-for="s in subjects"
        :key="s.id"
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
        :class="selectedSubjectId === s.id
          ? 'bg-[var(--accent)] text-white shadow-sm'
          : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)] hover:border-[var(--accent)]'"
        @click="changeSubject(s.id)"
      >
        {{ s.name }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <div class="h-24 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div v-for="i in 5" :key="i" class="h-16 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- No Journey Yet -->
    <div v-else-if="!snapshot">
      <AppCard padding="lg" glow="accent" class="text-center">
        <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-3xl"
          :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">◎</div>
        <h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--text)' }">
          Start Your Journey
        </h2>
        <p class="text-sm mb-6 max-w-sm mx-auto" :style="{ color: 'var(--text-2)' }">
          Your coach will build a personalized path from your current level to exam readiness,
          based on your diagnostic data and available time.
        </p>
        <AppButton variant="primary" size="lg" :loading="building" @click="startJourney">
          Build My Journey →
        </AppButton>
      </AppCard>
    </div>

    <!-- Journey Map -->
    <template v-else>
      <!-- Phase header strip -->
      <AppCard padding="md" glow="accent" class="mb-6">
        <div class="flex items-center justify-between flex-wrap gap-3">
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="phase in phases"
              :key="phase"
              class="px-2.5 py-1 text-[10px] font-semibold rounded-full uppercase tracking-wide"
              :style="snapshot.route.current_station_code?.toLowerCase().includes(phase.toLowerCase())
                ? { backgroundColor: 'var(--accent)', color: 'white' }
                : { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }"
            >
              {{ phase }}
            </span>
          </div>
          <div class="flex items-center gap-3">
            <span class="text-xs" :style="{ color: 'var(--text-3)' }">{{ completedCount }}/{{ totalCount }} stations</span>
            <AppProgress :value="completedCount" :max="totalCount || 1" size="sm" color="accent" class="w-24" />
          </div>
        </div>
      </AppCard>

      <!-- Station List -->
      <div class="space-y-2">
        <AppCard
          v-for="station in currentStations"
          :key="station.id"
          padding="md"
          :hover="station.status !== 'locked'"
          @click="station.status !== 'locked' ? router.push('/student/journey/station/' + station.id) : null"
        >
          <div class="flex items-center gap-4">
            <!-- Status icon -->
            <div
              class="w-10 h-10 rounded-xl flex items-center justify-center text-sm font-bold shrink-0"
              :style="{
                backgroundColor: station.status === 'completed' ? 'var(--success-light)'
                  : station.status === 'active' ? 'var(--accent-light)'
                  : 'var(--border-soft)',
                color: station.status === 'completed' ? 'var(--success)'
                  : station.status === 'active' ? 'var(--accent)'
                  : 'var(--text-3)',
              }"
            >
              {{ stationIcon(station.status) }}
            </div>

            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <p class="text-sm font-semibold truncate" :style="{ color: station.status === 'locked' ? 'var(--text-3)' : 'var(--text)' }">
                  {{ station.title }}
                </p>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-[10px] uppercase font-medium" :style="{ color: 'var(--text-3)' }">{{ phaseForStation(station) }}</span>
                <span class="text-[10px]" :style="{ color: 'var(--text-3)' }">#{{ station.sequence_no }}</span>
              </div>
            </div>

            <!-- Progress bar for active/in-progress stations -->
            <div v-if="station.status !== 'locked' && station.progress_score > 0" class="w-20 shrink-0">
              <AppProgress
                :value="station.progress_score"
                :max="10000"
                size="sm"
                :color="station.status === 'completed' ? 'success' : 'accent'"
              />
            </div>

            <AppBadge :color="(stationStatusColor(station.status) as any)" size="xs">
              {{ station.status }}
            </AppBadge>
          </div>
        </AppCard>
      </div>

      <!-- Refresh button -->
      <div class="mt-6 flex gap-3">
        <AppButton variant="secondary" size="sm" :loading="building" @click="startJourney">
          Refresh Journey
        </AppButton>
      </div>
    </template>
  </div>
</template>
