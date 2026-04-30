<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildGlossaryHomeSnapshot,
  createGlossaryTestSession,
  recordGlossaryInteraction,
  searchCatalog,
  submitGlossaryTestAttempt,
  type GlossaryEntryFocusDto,
  type GlossaryHomeSnapshotDto,
  type GlossarySearchGroupDto,
  type GlossarySearchResultDto,
  type GlossaryTestAttemptResultDto,
  type GlossaryTestItemDto,
  type GlossaryTestSessionDetailDto,
} from '@/ipc/library'
import { listSubjects, type SubjectDto } from '@/ipc/curriculum'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'
import GlossaryEntryCard from '@/components/glossary/GlossaryEntryCard.vue'
import GlossaryTestLab from '@/components/glossary/GlossaryTestLab.vue'
import AudioRadioMode from '@/components/glossary/AudioRadioMode.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(false)
const loadError = ref<string | null>(null)
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const snapshot = ref<GlossaryHomeSnapshotDto | null>(null)

const searchQuery = ref('')
const searching = ref(false)
const searchError = ref<string | null>(null)
const searchGroups = ref<GlossarySearchGroupDto[]>([])

const showTestLab = ref(false)
const showRadio = ref(false)

const activeTestSession = ref<GlossaryTestSessionDetailDto | null>(null)
const activeTestLabel = ref('Quick drill')
const testAnswer = ref('')
const testError = ref<string | null>(null)
const testSubmitting = ref(false)
const testResult = ref<GlossaryTestAttemptResultDto | null>(null)

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

const activeFocusCount = computed(() => focusEntries.value.length)
const dailyDrillCount = computed(() => snapshot.value?.weak_entries.length ?? 0)
const activeTestItem = computed<GlossaryTestItemDto | null>(() => activeTestSession.value?.items[0] ?? null)

const tools = [
  { label: 'Audio Radio', desc: 'Start a live glossary queue', action: () => { showRadio.value = !showRadio.value } },
  { label: 'Test Lab', desc: 'Run a real glossary drill', action: () => { showTestLab.value = !showTestLab.value } },
  { label: 'Formula Lab', desc: 'Work from actual formula entries', action: () => router.push('/student/glossary/formula-lab') },
  { label: 'Compare', desc: 'Open a real confusion comparison', action: () => router.push('/student/glossary/compare') },
]

const summaryTiles = computed(() => [
  { key: 'weak', label: 'Weak Now', desc: 'Entries under pressure', count: snapshot.value?.weak_entries.length ?? 0 },
  { key: 'hotspots', label: 'Exam Hotspots', desc: 'Most urgent high-value ideas', count: snapshot.value?.exam_hotspots.length ?? 0 },
  { key: 'discover', label: 'Discover', desc: 'Fresh glossary anchors to open next', count: snapshot.value?.discover.length ?? 0 },
])

function subjectLabel(subjectId: number | null): string {
  if (subjectId == null) return 'All subjects'
  return subjects.value.find(subject => subject.id === subjectId)?.name ?? 'Selected subject'
}

function prettyEntryType(entryType: string | null | undefined): string {
  if (!entryType) return 'entry'
  return entryType.replace(/_/g, ' ')
}

function modeLabelForTest(mode: string): string {
  switch (mode) {
    case 'recall':
      return 'Definition recall'
    case 'which_one_does_not_belong':
    case 'intruder_mode':
      return 'Intruder drill'
    case 'confusion_duel':
      return 'Confusion duel'
    case 'context_recognition':
      return 'Context clue'
    case 'fill_gap':
      return 'Hidden term'
    case 'reverse_recall':
      return 'Reverse recall'
    case 'connection_map':
    case 'relationship_hunt':
      return 'Connection map'
    case 'audio_recall':
      return 'Audio recall'
    case 'formula_reverse_recall':
    case 'formula_builder':
      return 'Formula builder'
    case 'question_signal':
      return 'Application match'
    default:
      return 'Glossary drill'
  }
}

function candidateEntryIds(mode: string): number[] {
  const all = focusEntries.value
  const formulas = all.filter(entry => entry.entry_type === 'formula')
  if (mode === 'formula_reverse_recall' || mode === 'formula_builder') {
    return [...formulas, ...all].map(entry => entry.entry_id)
  }
  return all.map(entry => entry.entry_id)
}

function openEntry(entryId: number) {
  router.push(`/student/glossary/entry/${entryId}`)
}

function handleStationSelect(station: string) {
  router.push({
    path: '/student/glossary/audio',
    query: { station },
  })
}

function resetTestState() {
  testAnswer.value = ''
  testError.value = null
  testResult.value = null
}

function safeRecordInteraction(
  input: Parameters<typeof recordGlossaryInteraction>[0],
) {
  void recordGlossaryInteraction(input).catch(() => {})
}

async function loadSubjects() {
  subjects.value = await listSubjects(1).catch(() => [])
}

async function loadSnapshot() {
  if (studentId.value == null) return

  loading.value = true
  loadError.value = null

  try {
    snapshot.value = await buildGlossaryHomeSnapshot(studentId.value, selectedSubjectId.value, 10)
  } catch {
    snapshot.value = null
    loadError.value = 'The glossary snapshot could not be loaded yet.'
  } finally {
    loading.value = false
  }
}

async function search() {
  if (studentId.value == null || !searchQuery.value.trim()) {
    searchGroups.value = []
    searchError.value = null
    return
  }

  const query = searchQuery.value.trim()
  searching.value = true
  searchError.value = null

  try {
    const response = await searchCatalog(
      {
        query,
        student_id: studentId.value,
        subject_id: selectedSubjectId.value,
        topic_id: null,
        include_bundles: true,
        include_questions: true,
        include_confusions: true,
        include_audio_ready_only: false,
      },
      12,
    )
    searchGroups.value = response.groups.filter(group => group.results.length > 0)
    safeRecordInteraction({
      student_id: studentId.value,
      event_type: 'search_result',
      query_text: query,
      metadata: {
        source: 'glossary-home',
        subject_id: selectedSubjectId.value,
      },
    })
  } catch {
    searchGroups.value = []
    searchError.value = 'Search is not available right now.'
  } finally {
    searching.value = false
  }
}

function openSearchResult(result: GlossarySearchResultDto) {
  if (studentId.value != null) {
    safeRecordInteraction({
      student_id: studentId.value,
      entry_id: result.entry_id,
      bundle_id: result.bundle_id,
      question_id: result.question_id,
      event_type: 'search_result',
      query_text: searchQuery.value.trim() || null,
      metadata: {
        source: 'glossary-home',
        result_type: result.result_type,
        title: result.title,
      },
    })
  }

  if (result.entry_id != null) {
    router.push({
      path: `/student/glossary/entry/${result.entry_id}`,
      query: searchQuery.value.trim() ? { q: searchQuery.value.trim() } : {},
    })
    return
  }

  if (result.bundle_id != null) {
    router.push({ path: '/student/library', hash: '#revision-box' })
    return
  }

  if (result.question_id != null) {
    router.push(result.topic_id != null ? `/student/practice?topic=${result.topic_id}` : '/student/practice')
  }
}

async function startQuickDrill() {
  showTestLab.value = true
  await handleStartTest('recall')
}

async function handleStartTest(mode: string) {
  if (studentId.value == null) return

  activeTestLabel.value = modeLabelForTest(mode)
  activeTestSession.value = null
  resetTestState()

  const entryIds = candidateEntryIds(mode)
  if (!entryIds.length) {
    testError.value = 'No glossary entries are ready for a live drill yet.'
    return
  }

  let lastError: unknown = null
  for (const entryId of entryIds.slice(0, 8)) {
    try {
      activeTestSession.value = await createGlossaryTestSession(studentId.value, {
        test_mode: mode,
        topic_id: null,
        bundle_id: null,
        entry_ids: [entryId],
        entry_count: 1,
        duration_seconds: 90,
        difficulty_level: 5600,
      })
      return
    } catch (error) {
      lastError = error
    }
  }

  testError.value = lastError instanceof Error
    ? lastError.message
    : 'A compatible glossary drill could not be built from the live entry pool.'
}

async function submitTestAnswer() {
  if (
    studentId.value == null ||
    activeTestSession.value == null ||
    activeTestItem.value == null ||
    !testAnswer.value.trim()
  ) {
    return
  }

  testSubmitting.value = true
  testError.value = null

  try {
    testResult.value = await submitGlossaryTestAttempt(studentId.value, activeTestSession.value.session_id, {
      entry_id: activeTestItem.value.entry_id,
      student_response: testAnswer.value.trim(),
      time_seconds: null,
    })
    await loadSnapshot()
  } catch {
    testError.value = 'That answer could not be submitted yet.'
  } finally {
    testSubmitting.value = false
  }
}

function submitOption(option: string) {
  testAnswer.value = option
  void submitTestAnswer()
}

watch(searchQuery, value => {
  if (!value.trim()) {
    searchGroups.value = []
    searchError.value = null
  }
})

watch(selectedSubjectId, () => {
  void loadSnapshot()
})

onMounted(async () => {
  await Promise.all([loadSubjects(), loadSnapshot()])
})
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pb-5 pt-6"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="mb-4 flex items-center justify-between gap-4">
        <div>
          <p class="eyebrow">Glossary Lab</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Definitions. Formulas. Concepts.
          </h1>
          <p class="mt-1 text-sm" :style="{ color: 'var(--ink-muted)' }">
            Real glossary focus lanes powered by the Rust knowledge graph.
          </p>
        </div>

        <div class="flex items-center gap-3 rounded-xl border px-4 py-2.5" :style="{ borderColor: 'transparent' }">
          <div>
            <p class="text-[10px] font-bold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Daily Drill</p>
            <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">
              {{ dailyDrillCount }} entry<span v-if="dailyDrillCount !== 1">ies</span> ready for repair
            </p>
          </div>
          <AppButton size="sm" @click="startQuickDrill">Start</AppButton>
        </div>
      </div>

      <AppSearchInput
        v-model="searchQuery"
        placeholder="Search any term, formula, or concept..."
        :loading="searching"
        @search="search"
      />

      <div class="mt-4 flex flex-wrap gap-2">
        <button
          class="subject-pill"
          :class="{ active: selectedSubjectId == null }"
          @click="selectedSubjectId = null"
        >
          All subjects
        </button>
        <button
          v-for="subject in subjects"
          :key="subject.id"
          class="subject-pill"
          :class="{ active: selectedSubjectId === subject.id }"
          @click="selectedSubjectId = subject.id"
        >
          {{ subject.name }}
        </button>
      </div>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6">
        <p v-if="loadError" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ loadError }}</p>
        <p v-if="searchError" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ searchError }}</p>

        <template v-if="searchGroups.length">
          <div class="space-y-5">
            <div v-for="group in searchGroups" :key="group.group_key">
              <p class="section-label mb-2">{{ group.title }}</p>
              <div class="space-y-2">
                <button
                  v-for="result in group.results"
                  :key="`${group.group_key}-${result.entry_id ?? result.bundle_id ?? result.question_id ?? result.title}`"
                  class="w-full text-left"
                  @click="openSearchResult(result)"
                >
                  <GlossaryEntryCard
                    v-if="result.entry_id != null"
                    :entry-id="result.entry_id"
                    :title="result.title"
                    :type="result.entry_type ?? result.result_type"
                    :topic="result.topic_name ?? undefined"
                    :quick-meaning="result.subtitle ?? result.match_reason"
                  />

                  <AppCard v-else hover padding="sm">
                    <div class="flex items-start justify-between gap-3">
                      <div>
                        <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ result.title }}</p>
                        <p class="mt-0.5 text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                          {{ result.subtitle ?? result.match_reason }}
                        </p>
                      </div>
                      <span class="rounded-full px-2 py-1 text-[10px] font-semibold uppercase tracking-wide" :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }">
                        {{ prettyEntryType(result.result_type) }}
                      </span>
                    </div>
                  </AppCard>
                </button>
              </div>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="mb-6">
            <p class="section-label mb-3">Live Focus Lanes</p>
            <div class="grid grid-cols-3 gap-4">
              <button v-for="tile in summaryTiles" :key="tile.key" class="section-tile">
                <div class="tile-symbol">{{ tile.count }}</div>
                <h3 class="mb-0.5 text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ tile.label }}</h3>
                <p class="mb-3 text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ tile.desc }}</p>
                <span class="count-badge">{{ subjectLabel(selectedSubjectId) }}</span>
              </button>
            </div>
          </div>

          <div v-if="snapshot?.weak_entries?.length" class="mb-6">
            <div class="mb-3 flex items-center justify-between">
              <p class="section-label">Weak Now</p>
              <span class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                {{ snapshot.weak_entries.length }} live
              </span>
            </div>
            <div class="space-y-2">
              <GlossaryEntryCard
                v-for="entry in snapshot.weak_entries"
                :key="entry.entry_id"
                :entry-id="entry.entry_id"
                :title="entry.title"
                :type="entry.entry_type"
                :topic="entry.topic_name ?? undefined"
                :quick-meaning="entry.match_reason"
                @click="openEntry(entry.entry_id)"
              />
            </div>
          </div>

          <div v-if="snapshot?.exam_hotspots?.length" class="mb-6">
            <div class="mb-3 flex items-center justify-between">
              <p class="section-label">Exam Hotspots</p>
              <span class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                high-value
              </span>
            </div>
            <div class="space-y-2">
              <GlossaryEntryCard
                v-for="entry in snapshot.exam_hotspots"
                :key="entry.entry_id"
                :entry-id="entry.entry_id"
                :title="entry.title"
                :type="entry.entry_type"
                :topic="entry.topic_name ?? undefined"
                :quick-meaning="entry.match_reason"
                @click="openEntry(entry.entry_id)"
              />
            </div>
          </div>

          <div v-if="snapshot?.discover?.length" class="mb-6">
            <div class="mb-3 flex items-center justify-between">
              <p class="section-label">Discover</p>
              <span class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                next anchors
              </span>
            </div>
            <div class="space-y-2">
              <GlossaryEntryCard
                v-for="entry in snapshot.discover"
                :key="entry.entry_id"
                :entry-id="entry.entry_id"
                :title="entry.title"
                :type="entry.entry_type"
                :topic="entry.topic_name ?? undefined"
                :quick-meaning="entry.match_reason"
                @click="openEntry(entry.entry_id)"
              />
            </div>
          </div>

          <div v-if="snapshot?.recommended_bundles?.length" class="mb-6">
            <p class="section-label mb-3">Bundle Routes</p>
            <div class="grid grid-cols-2 gap-3">
              <button
                v-for="bundle in snapshot.recommended_bundles"
                :key="bundle.bundle_id"
                class="w-full text-left"
                @click="router.push({ path: '/student/library', hash: '#revision-box' })"
              >
              <AppCard padding="md">
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ bundle.title }}</p>
                <p class="mt-1 text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ bundle.focus_reason }}</p>
                <p class="mt-2 text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                  {{ bundle.focus_entry_titles.join(', ') }}
                </p>
              </AppCard>
              </button>
            </div>
          </div>

          <div class="mb-6">
            <p class="section-label mb-3">Learning Tools</p>
            <div class="grid grid-cols-2 gap-3">
              <button
                v-for="tool in tools"
                :key="tool.label"
                class="tool-card"
                @click="tool.action()"
              >
                <div class="text-left">
                  <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ tool.label }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ tool.desc }}</p>
                </div>
              </button>
            </div>
          </div>

          <AudioRadioMode v-if="showRadio" class="mb-6" @select-station="handleStationSelect" />

          <div v-if="showTestLab" class="mb-6">
            <GlossaryTestLab @start-test="handleStartTest" />
          </div>

          <AppCard v-if="activeTestSession && activeTestItem" padding="lg" class="mb-6">
            <div class="mb-4 flex items-start justify-between gap-3">
              <div>
                <p class="section-label">{{ activeTestLabel }}</p>
                <h3 class="mt-1 text-lg font-semibold" :style="{ color: 'var(--ink)' }">
                  {{ activeTestItem.prompt_text }}
                </h3>
              </div>
              <button class="count-badge" @click="openEntry(activeTestItem.entry_id)">Open Entry</button>
            </div>

            <div v-if="activeTestItem.options.length" class="mb-4 flex flex-wrap gap-2">
              <button
                v-for="option in activeTestItem.options"
                :key="option"
                class="option-pill"
                @click="submitOption(option)"
              >
                {{ option }}
              </button>
            </div>

            <div class="mb-4">
              <AppInput
                v-model="testAnswer"
                label="Your answer"
                placeholder="Type the term or formula"
              />
            </div>

            <div class="flex items-center gap-3">
              <AppButton :loading="testSubmitting" @click="submitTestAnswer">Submit</AppButton>
              <AppButton variant="secondary" @click="handleStartTest(activeTestSession.test_mode)">New Entry</AppButton>
            </div>

            <p v-if="testError" class="mt-3 text-sm font-medium text-[var(--danger)]">{{ testError }}</p>

            <div v-if="testResult" class="mt-4 rounded-2xl border p-4" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }">
              <p class="text-sm font-semibold" :style="{ color: testResult.is_correct ? 'var(--success)' : 'var(--danger)' }">
                {{ testResult.is_correct ? 'Correct' : 'Needs repair' }}
              </p>
              <p class="mt-1 text-sm" :style="{ color: 'var(--ink-muted)' }">{{ testResult.feedback }}</p>
              <p class="mt-2 text-[11px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
                Mastery {{ Math.round(testResult.mastery_score / 100) }}%
                <span v-if="testResult.updated_mastery_state"> · {{ testResult.updated_mastery_state }}</span>
              </p>
            </div>
          </AppCard>
        </template>
      </div>

      <div
        class="flex w-60 flex-shrink-0 flex-col overflow-hidden border-l"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="flex-shrink-0 border-b px-5 py-4" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Access</p>
        </div>
        <div class="flex-1 space-y-1 overflow-y-auto p-4">
          <button class="quick-link w-full text-left" @click="router.push('/student/glossary/audio')">
            Audio Glossary
          </button>
          <button class="quick-link w-full text-left" @click="router.push('/student/library')">
            Full Library
          </button>
          <button class="quick-link w-full text-left" @click="router.push('/student/memory')">
            Memory Mode
          </button>
          <button class="quick-link w-full text-left" @click="router.push('/student/knowledge-gap')">
            Knowledge Gap
          </button>
        </div>

        <div class="border-t p-4" :style="{ borderColor: 'var(--border-soft)' }">
          <div class="text-center">
            <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ activeFocusCount }}</p>
            <p class="text-[10px] font-semibold uppercase" :style="{ color: 'var(--ink-muted)' }">Active Glossary Signals</p>
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
  color: var(--accent);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.subject-pill {
  border-radius: 999px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-muted);
  background: var(--paper);
  border: 1px solid transparent;
  transition: border-color 120ms ease, color 120ms ease, background-color 120ms ease;
}

.subject-pill.active {
  color: white;
  background: var(--accent);
}

.section-tile {
  position: relative;
  overflow: hidden;
  border-radius: 16px;
  padding: 20px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: default;
  background: var(--surface);
  border: 1px solid transparent;
}

.tile-symbol {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 900;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
  margin-bottom: 12px;
}

.count-badge,
.option-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

.tool-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: var(--surface);
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease;
}

.tool-card:hover,
.option-pill:hover,
.subject-pill:hover:not(.active) {
  border-color: var(--ink-muted);
}

.tool-card:hover {
  background: var(--paper);
}

.quick-link {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-secondary);
  cursor: pointer;
  transition: background-color 100ms ease;
}

.quick-link:hover {
  background-color: var(--paper);
  color: var(--ink);
}
</style>
