<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildGlossaryHomeSnapshot,
  currentGlossaryAudioQueue,
  nextGlossaryAudioQueue,
  previousGlossaryAudioQueue,
  recordGlossaryInteraction,
  startGlossaryAudioQueue,
  updateGlossaryAudioQueue,
  type GlossaryAudioQueueSnapshotDto,
  type GlossaryEntryFocusDto,
  type GlossaryHomeSnapshotDto,
  type StartGlossaryAudioQueueInputDto,
} from '@/ipc/library'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AudioRadioMode from '@/components/glossary/AudioRadioMode.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const snapshot = ref<GlossaryHomeSnapshotDto | null>(null)
const queue = ref<GlossaryAudioQueueSnapshotDto | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const activeStation = ref<string>('all')
const speechSupported = typeof window !== 'undefined' && 'speechSynthesis' in window
let speechToken = 0

const studentId = computed(() => auth.currentAccount?.id ?? null)

const focusEntries = computed<GlossaryEntryFocusDto[]>(() => {
  const merged = new Map<number, GlossaryEntryFocusDto>()
  for (const entry of snapshot.value?.weak_entries ?? []) {
    merged.set(entry.entry_id, entry)
  }
  for (const entry of snapshot.value?.exam_hotspots ?? []) {
    if (!merged.has(entry.entry_id)) {
      merged.set(entry.entry_id, entry)
    }
  }
  for (const entry of snapshot.value?.discover ?? []) {
    if (!merged.has(entry.entry_id)) {
      merged.set(entry.entry_id, entry)
    }
  }
  return Array.from(merged.values())
})

const formulaEntry = computed(() => focusEntries.value.find(entry => entry.entry_type === 'formula') ?? null)
const totalSegments = computed(() => queue.value?.program?.segments.length ?? 0)
const progressPercent = computed(() => {
  if (!queue.value || totalSegments.value === 0) return 0
  return Math.round(((queue.value.current_position + 1) / totalSegments.value) * 100)
})

function guessStationFromQueue(snapshotValue: GlossaryAudioQueueSnapshotDto | null): string {
  const sourceType = snapshotValue?.program?.source_type
  switch (sourceType) {
    case 'weakness':
      return 'weak'
    case 'bundle':
      return 'custom'
    case 'entry':
      return 'formulas'
    case 'topic':
    default:
      return 'all'
  }
}

function safeRecordAudio(entryId: number | null | undefined) {
  if (studentId.value == null || entryId == null) return

  void recordGlossaryInteraction({
    student_id: studentId.value,
    entry_id: entryId,
    event_type: 'played_audio',
    metadata: {
      source: 'glossary-audio',
      station: activeStation.value,
    },
  }).catch(() => {})
}

async function applyQueue(nextQueue: GlossaryAudioQueueSnapshotDto) {
  queue.value = nextQueue
  safeRecordAudio(nextQueue.current_segment?.entry_id)
}

function startRequestForStation(station: string): StartGlossaryAudioQueueInputDto | null {
  switch (station) {
    case 'weak':
      return {
        source_type: 'weakness',
        source_id: 0,
        limit: 4,
        teaching_mode: 'repair',
        include_examples: true,
        include_misconceptions: true,
      }
    case 'today':
      return {
        source_type: 'weakness',
        source_id: 0,
        limit: 3,
        teaching_mode: 'reactivation',
        include_examples: true,
        include_misconceptions: true,
      }
    case 'hotspot': {
      const hotspot = snapshot.value?.exam_hotspots[0]
      if (hotspot?.topic_id != null) {
        return {
          source_type: 'topic',
          source_id: hotspot.topic_id,
          limit: 4,
          teaching_mode: 'repair',
          include_examples: true,
          include_misconceptions: true,
        }
      }
      if (hotspot) {
        return {
          source_type: 'entry',
          source_id: hotspot.entry_id,
          limit: 4,
          teaching_mode: 'repair',
          include_examples: true,
          include_misconceptions: true,
        }
      }
      return null
    }
    case 'formulas':
      if (!formulaEntry.value) return null
      return {
        source_type: 'entry',
        source_id: formulaEntry.value.entry_id,
        limit: 4,
        teaching_mode: 'standard',
        include_examples: true,
        include_misconceptions: true,
      }
    case 'custom':
      if (!snapshot.value?.recommended_bundles[0]) return null
      return {
        source_type: 'bundle',
        source_id: snapshot.value.recommended_bundles[0].bundle_id,
        limit: 4,
        teaching_mode: 'standard',
        include_examples: true,
        include_misconceptions: true,
      }
    case 'all':
    default: {
      const discover = snapshot.value?.discover[0]
      if (discover?.topic_id != null) {
        return {
          source_type: 'topic',
          source_id: discover.topic_id,
          limit: 4,
          teaching_mode: 'standard',
          include_examples: true,
          include_misconceptions: true,
        }
      }
      if (discover) {
        return {
          source_type: 'entry',
          source_id: discover.entry_id,
          limit: 4,
          teaching_mode: 'standard',
          include_examples: true,
          include_misconceptions: true,
        }
      }
      return null
    }
  }
}

async function loadPage() {
  if (studentId.value == null) return

  loading.value = true
  error.value = null

  try {
    const [homeSnapshot, currentQueue] = await Promise.all([
      buildGlossaryHomeSnapshot(studentId.value, null, 10),
      currentGlossaryAudioQueue(studentId.value),
    ])
    snapshot.value = homeSnapshot
    queue.value = currentQueue
    activeStation.value = guessStationFromQueue(currentQueue)
  } catch {
    error.value = 'The audio glossary could not be loaded yet.'
  } finally {
    loading.value = false
  }
}

async function startStation(station: string) {
  if (studentId.value == null) return

  const request = startRequestForStation(station)
  if (!request) {
    error.value = 'There is not enough glossary data to start that station yet.'
    return
  }

  loading.value = true
  error.value = null
  activeStation.value = station

  try {
    const nextQueue = await startGlossaryAudioQueue(studentId.value, request)
    await applyQueue(nextQueue)
  } catch {
    error.value = 'That audio station could not be started yet.'
  } finally {
    loading.value = false
  }
}

async function goNext() {
  if (studentId.value == null) return
  try {
    await applyQueue(await nextGlossaryAudioQueue(studentId.value))
  } catch {
    error.value = 'The audio queue could not advance.'
  }
}

async function goPrevious() {
  if (studentId.value == null) return
  try {
    await applyQueue(await previousGlossaryAudioQueue(studentId.value))
  } catch {
    error.value = 'The audio queue could not rewind.'
  }
}

async function togglePlayback() {
  if (studentId.value == null || !queue.value) return
  try {
    await applyQueue(await updateGlossaryAudioQueue(studentId.value, {
      is_playing: !queue.value.is_playing,
    }))
  } catch {
    error.value = 'Playback could not be updated.'
  }
}

async function setSpeed(speed: number) {
  if (studentId.value == null) return
  try {
    await applyQueue(await updateGlossaryAudioQueue(studentId.value, {
      playback_speed: speed,
    }))
  } catch {
    error.value = 'Playback speed could not be updated.'
  }
}

function stopSpeechPlayback() {
  if (!speechSupported) return
  speechToken += 1
  window.speechSynthesis.cancel()
}

async function setPlaybackState(isPlaying: boolean) {
  if (studentId.value == null) return
  try {
    await applyQueue(await updateGlossaryAudioQueue(studentId.value, {
      is_playing: isPlaying,
    }))
  } catch {
    error.value = 'Playback could not be updated.'
  }
}

function speakCurrentSegment() {
  if (!speechSupported || !queue.value?.is_playing) return

  const script = queue.value.current_segment?.script_text?.trim()
  if (!script) return

  stopSpeechPlayback()
  const token = speechToken
  const utterance = new SpeechSynthesisUtterance(script)
  utterance.rate = queue.value.playback_speed
  utterance.onend = () => {
    if (token !== speechToken || !queue.value?.is_playing) return
    if ((queue.value.current_position + 1) >= totalSegments.value) {
      void setPlaybackState(false)
      return
    }
    void goNext()
  }
  utterance.onerror = () => {
    if (token === speechToken) {
      error.value = 'Speech playback is unavailable in this webview right now.'
    }
  }
  window.speechSynthesis.speak(utterance)
}

function openBundleRoute(bundleId: number) {
  if (studentId.value != null) {
    void recordGlossaryInteraction({
      student_id: studentId.value,
      entry_id: queue.value?.current_segment?.entry_id ?? null,
      bundle_id: bundleId,
      event_type: 'opened_bundle_route',
      metadata: {
        source: 'glossary-audio',
        station: activeStation.value,
      },
    }).catch(() => {})
  }

  void router.push({ path: '/student/library', hash: '#revision-box' })
}

watch(() => route.query.station, station => {
  if (typeof station === 'string' && snapshot.value) {
    void startStation(station)
  }
})

watch(
  () => [
    queue.value?.is_playing,
    queue.value?.current_segment?.script_text,
    queue.value?.playback_speed,
    queue.value?.current_position,
  ],
  () => {
    if (!speechSupported) return
    if (queue.value?.is_playing && queue.value.current_segment?.script_text) {
      speakCurrentSegment()
      return
    }
    stopSpeechPlayback()
  },
)

onMounted(async () => {
  await loadPage()
  if (typeof route.query.station === 'string') {
    await startStation(route.query.station)
  }
})

onBeforeUnmount(() => {
  stopSpeechPlayback()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-8">
      <p class="mb-2 text-[10px] font-bold uppercase tracking-[0.16em]" style="color: var(--accent);">Glossary Audio</p>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Audio Radio</h1>
          <p class="mt-2 text-sm" :style="{ color: 'var(--ink-muted)' }">
            Live queue state, real segment stepping, and spoken playback from the current glossary script queue.
          </p>
        </div>
        <div class="flex gap-2">
          <AppButton variant="secondary" @click="router.push('/student/glossary')">Back</AppButton>
          <AppButton variant="secondary" @click="startStation(activeStation || 'all')">Refresh Station</AppButton>
        </div>
      </div>
    </div>

    <p v-if="loading" class="mb-4 text-sm" :style="{ color: 'var(--ink-muted)' }">Loading audio queue...</p>
    <p v-if="error" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ error }}</p>

    <div class="mb-6">
      <AudioRadioMode :active-station="activeStation" @select-station="startStation" />
      <p v-if="!speechSupported" class="mt-3 text-xs" :style="{ color: 'var(--ink-muted)' }">
        Script playback is available through the live queue, but this webview does not expose speech synthesis right now.
      </p>
    </div>

    <div v-if="queue?.program" class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_300px]">
      <AppCard padding="lg">
        <div class="mb-4 flex flex-wrap items-center gap-2">
          <AppBadge size="xs" color="accent">{{ queue.program.source_type }}</AppBadge>
          <AppBadge size="xs" color="warm">{{ queue.program.teaching_mode }}</AppBadge>
          <AppBadge size="xs" color="gold">
            {{ queue.current_position + 1 }} / {{ totalSegments }}
          </AppBadge>
        </div>

        <h2 class="text-xl font-semibold" :style="{ color: 'var(--ink)' }">{{ queue.program.program_title }}</h2>
        <p class="mt-2 text-sm" :style="{ color: 'var(--ink-muted)' }">
          {{ queue.current_segment?.title ?? 'No active segment yet.' }}
        </p>

        <div class="mt-4">
          <AppProgress :value="progressPercent" />
        </div>

        <div class="mt-5 rounded-2xl border p-5" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }">
          <p class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
            Current Segment
          </p>
          <p class="mt-2 text-base font-semibold" :style="{ color: 'var(--ink)' }">
            {{ queue.current_segment?.title ?? 'Pick a station to begin.' }}
          </p>
          <p class="mt-3 text-sm leading-7 whitespace-pre-line" :style="{ color: 'var(--ink-secondary)' }">
            {{ queue.current_segment?.script_text ?? 'The audio script will appear here.' }}
          </p>
          <p v-if="queue.current_segment?.prompt_text" class="mt-3 text-xs font-medium" :style="{ color: 'var(--accent)' }">
            {{ queue.current_segment.prompt_text }}
          </p>
        </div>

        <div class="mt-5 flex flex-wrap items-center gap-3">
          <AppButton variant="secondary" @click="goPrevious">Previous</AppButton>
          <AppButton @click="togglePlayback">{{ queue.is_playing ? 'Pause' : 'Play' }}</AppButton>
          <AppButton variant="secondary" @click="goNext">Next</AppButton>
        </div>

        <div class="mt-4 flex flex-wrap gap-2">
          <button
            v-for="speed in [0.75, 1, 1.25, 1.5]"
            :key="speed"
            class="rounded-full px-3 py-1 text-xs font-semibold"
            :style="{
              backgroundColor: queue.playback_speed === speed ? 'var(--accent)' : 'var(--paper)',
              color: queue.playback_speed === speed ? 'white' : 'var(--ink)',
            }"
            @click="setSpeed(speed)"
          >
            {{ speed }}x
          </button>
        </div>
      </AppCard>

      <div class="space-y-4">
        <AppCard v-if="queue.program.listener_signals.length" padding="md">
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Learner Signals</p>
          <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <li v-for="signal in queue.program.listener_signals" :key="signal">{{ signal }}</li>
          </ul>
        </AppCard>

        <AppCard v-if="queue.program.contrast_titles.length" padding="md">
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Contrast Traps</p>
          <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <li v-for="title in queue.program.contrast_titles" :key="title">{{ title }}</li>
          </ul>
        </AppCard>

        <AppCard v-if="queue.program.review_entry_titles.length" padding="md">
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Review Loop</p>
          <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <li v-for="title in queue.program.review_entry_titles" :key="title">{{ title }}</li>
          </ul>
        </AppCard>

        <AppCard v-if="queue.program.recommended_bundles.length" padding="md">
          <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Suggested Bundles</p>
          <div class="space-y-2">
            <button
              v-for="bundle in queue.program.recommended_bundles"
              :key="bundle.bundle_id"
              class="w-full rounded-xl border px-3 py-2 text-left"
              :style="{ borderColor: 'var(--border-soft)' }"
              @click="openBundleRoute(bundle.bundle_id)"
            >
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ bundle.title }}</p>
              <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ bundle.focus_reason }}</p>
            </button>
          </div>
        </AppCard>
      </div>
    </div>

    <AppCard v-else padding="lg">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
        Choose a station to start a real glossary audio queue.
      </p>
    </AppCard>
  </div>
</template>
