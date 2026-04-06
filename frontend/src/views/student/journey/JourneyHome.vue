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
    snapshot.value = await buildOrRefreshJourneyRoute(auth.currentAccount.id, selectedSubjectId.value, 'BECE')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to build journey'
  }
  building.value = false
}

const currentStations = computed(() => snapshot.value?.stations ?? [])
const completedCount = computed(() => currentStations.value.filter(s => s.status === 'completed').length)
const totalCount = computed(() => currentStations.value.length)

function stationIcon(status: string): string {
  if (status === 'completed') return '✓'
  if (status === 'active') return '▶'
  return '○'
}

function phaseForStation(s: JourneyStation): string {
  const typeMap: Record<string, string> = {
    foundation: 'Stabilize', build: 'Build', strengthen: 'Strengthen',
    conditioning: 'Condition', final: 'Ready',
  }
  return typeMap[s.station_type] ?? s.station_type
}

const activeStation = computed(() =>
  currentStations.value.find(s => s.status === 'active') ?? currentStations.value[0] ?? null
)

const currentPhase = computed(() => {
  if (!snapshot.value?.route.current_station_code) return phases[0]
  const code = snapshot.value.route.current_station_code.toLowerCase()
  return phases.find(p => code.includes(p.toLowerCase())) ?? phases[0]
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Journey</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Your path to readiness
        </h1>
      </div>
      <div class="flex items-center gap-3">
        <div class="flex gap-1.5">
          <button
            v-for="s in subjects"
            :key="s.id"
            class="subj-tab"
            :class="{ active: selectedSubjectId === s.id }"
            @click="changeSubject(s.id)"
          >{{ s.name }}</button>
        </div>
        <button v-if="snapshot" class="nav-pill"
          :disabled="building" @click="startJourney">
          {{ building ? '…' : '↺ Refresh' }}
        </button>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div class="h-20 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div v-for="i in 5" :key="i" class="h-14 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- No journey yet -->
    <div v-else-if="!snapshot" class="flex-1 flex items-center justify-center p-8">
      <div class="text-center max-w-md">
        <div class="w-16 h-16 rounded-2xl mx-auto mb-6 flex items-center justify-center text-3xl font-black"
          :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--ink)' }">◎</div>
        <h2 class="font-display text-xl font-bold mb-3" :style="{ color: 'var(--ink)' }">Start Your Journey</h2>
        <p class="text-sm mb-8" :style="{ color: 'var(--ink-muted)' }">
          Your coach will build a personalized path from your current level to exam readiness,
          based on your diagnostic data.
        </p>
        <button
          class="build-btn"
          :disabled="building"
          @click="startJourney"
        >{{ building ? 'Building your journey…' : 'Build My Journey →' }}</button>
      </div>
    </div>

    <!-- Journey content -->
    <template v-else>
      <!-- Phase strip + progress -->
      <div
        class="flex-shrink-0 px-7 py-3 border-b flex items-center gap-6"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="flex gap-1">
          <span
            v-for="phase in phases"
            :key="phase"
            class="phase-chip"
            :class="{ active: phase === currentPhase }"
          >{{ phase }}</span>
        </div>
        <div class="flex items-center gap-3 ml-auto">
          <span class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ completedCount }}/{{ totalCount }} stations</span>
          <AppProgress :value="completedCount" :max="totalCount || 1" size="sm" color="accent" class="w-28" />
        </div>
      </div>

      <!-- Body: station list + detail panel -->
      <div class="flex-1 overflow-hidden flex">

        <!-- Station list -->
        <div class="flex-1 overflow-y-auto divide-y" :style="{ borderColor: 'var(--border-soft)' }">
          <div
            v-for="(station, idx) in currentStations"
            :key="station.id"
            class="station-row"
            :class="[station.status, { clickable: station.status !== 'locked' }]"
            @click="station.status !== 'locked' ? router.push('/student/journey/station/' + station.id) : null"
          >
            <div class="flex items-center gap-4 px-7 py-4">
              <div
                class="station-node"
                :style="{
                  backgroundColor: station.status === 'completed' ? 'rgba(13,148,136,0.1)' : station.status === 'active' ? 'rgba(13,148,136,0.15)' : 'var(--paper)',
                  color: station.status === 'completed' ? 'var(--accent)' : station.status === 'active' ? 'var(--accent)' : 'var(--ink-muted)',
                  border: station.status === 'active' ? '2px solid var(--accent)' : '2px solid var(--border-soft)',
                }"
              >{{ stationIcon(station.status) }}</div>

              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate"
                  :style="{ color: station.status === 'locked' ? 'var(--ink-muted)' : 'var(--ink)' }">
                  {{ station.title }}
                </p>
                <p class="text-[10px] uppercase font-medium mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                  {{ phaseForStation(station) }} · Station {{ station.sequence_no }}
                </p>
              </div>

              <div v-if="station.status !== 'locked' && station.progress_score > 0" class="w-24 shrink-0">
                <AppProgress
                  :value="station.progress_score"
                  :max="10000"
                  size="sm"
                  color="accent"
                />
              </div>

              <span
                class="station-badge"
                :style="{
                  background: station.status === 'completed' ? 'rgba(13,148,136,0.1)' : station.status === 'active' ? 'rgba(13,148,136,0.1)' : 'var(--paper)',
                  color: station.status === 'completed' ? 'var(--accent)' : station.status === 'active' ? 'var(--accent)' : 'var(--ink-muted)',
                  border: '1px solid var(--border-soft)',
                }"
              >{{ station.status }}</span>
            </div>
          </div>
        </div>

        <!-- Active station detail -->
        <div
          v-if="activeStation"
          class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
        >
          <div class="px-5 pt-5 pb-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
            <p class="eyebrow mb-1">Active Station</p>
            <h3 class="font-display text-base font-bold" :style="{ color: 'var(--ink)' }">{{ activeStation.title }}</h3>
            <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
              {{ phaseForStation(activeStation) }} · Station {{ activeStation.sequence_no }}
            </p>
          </div>
          <div class="p-5 flex-1">
            <div class="mb-5">
              <div class="flex justify-between text-xs mb-1.5">
                <span :style="{ color: 'var(--ink-muted)' }">Progress</span>
                <span :style="{ color: 'var(--accent)', fontWeight: 600 }">{{ (activeStation.progress_score / 100).toFixed(0) }}%</span>
              </div>
              <AppProgress :value="activeStation.progress_score" :max="10000" size="md" color="accent" />
            </div>
            <button
              class="go-btn w-full"
              @click="router.push('/student/journey/station/' + activeStation.id)"
            >Continue Station →</button>
          </div>
        </div>
      </div>
    </template>
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

.subj-tab {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 120ms;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
}
.subj-tab.active,
.subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.nav-pill {
  padding: 5px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: opacity 120ms;
}
.nav-pill:hover:not(:disabled) { opacity: 0.7; }
.nav-pill:disabled { opacity: 0.5; cursor: not-allowed; }

.phase-chip {
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  background: var(--paper);
  color: var(--ink-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.phase-chip.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.station-row {
  border-bottom: 1px solid var(--border-soft);
  transition: background-color 100ms;
}
.station-row.clickable { cursor: pointer; }
.station-row.clickable:hover { background-color: var(--paper); }
.station-row.locked { opacity: 0.5; }

.station-node {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

.station-badge {
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  flex-shrink: 0;
}

.build-btn {
  padding: 12px 28px;
  border-radius: 14px;
  background: var(--accent);
  color: white;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: opacity 140ms, transform 140ms;
}
.build-btn:hover:not(:disabled) { opacity: 0.88; transform: translateY(-1px); }
.build-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.go-btn {
  padding: 10px;
  border-radius: 12px;
  background: var(--accent-glow);
  color: var(--accent);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  border: 1px solid var(--accent);
  transition: background-color 120ms;
}
.go-btn:hover { background: var(--accent); color: white; }
</style>
