<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getGlossaryEntryDetail,
  recordGlossaryInteraction,
  type EntryContentBlockDto,
  type GlossaryEntryDetailDto,
  type KnowledgeEntryProfileDto,
} from '@/ipc/library'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const detail = ref<GlossaryEntryDetailDto | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const displayMode = ref<'quick' | 'simple' | 'exam' | 'deep' | 'technical' | 'audio'>('simple')

const studentId = computed(() => auth.currentAccount?.id ?? null)
const entryId = computed(() => {
  const parsed = Number(route.params.id)
  return Number.isFinite(parsed) ? parsed : null
})

const entry = computed(() => detail.value?.entry ?? null)
const masteryScore = computed(() => detail.value?.student_state?.mastery_score ?? 0)
const masteryState = computed(() => detail.value?.student_state?.mastery_state ?? detail.value?.student_state?.familiarity_state ?? 'unseen')

const primaryText = computed(() => {
  const currentEntry = entry.value
  if (!currentEntry) return ''

  switch (displayMode.value) {
    case 'quick':
      return detail.value?.definition_meta?.short_definition
        ?? currentEntry.short_text
        ?? currentEntry.simple_text
        ?? currentEntry.full_text
        ?? ''
    case 'simple':
      return detail.value?.definition_meta?.plain_english_definition
        ?? currentEntry.simple_text
        ?? currentEntry.short_text
        ?? currentEntry.full_text
        ?? ''
    case 'exam':
      return currentEntry.exam_text
        ?? detail.value?.definition_meta?.context_clues
        ?? currentEntry.short_text
        ?? currentEntry.full_text
        ?? ''
    case 'technical':
      return currentEntry.technical_text
        ?? detail.value?.formula_meta?.derivation_summary
        ?? currentEntry.full_text
        ?? ''
    case 'audio':
      return detail.value?.audio_segments.map(segment => segment.script_text).join(' ')
        || currentEntry.short_text
        || currentEntry.full_text
        || ''
    case 'deep':
    default:
      return currentEntry.full_text
        ?? detail.value?.concept_meta?.concept_explanation
        ?? currentEntry.technical_text
        ?? currentEntry.short_text
        ?? ''
  }
})

const contentNotes = computed(() => {
  return (detail.value?.content_blocks ?? [])
    .map(blockToText)
    .filter((value): value is string => Boolean(value))
    .slice(0, 4)
})

const modeButtons = [
  { key: 'quick', label: 'Quick' },
  { key: 'simple', label: 'Simple' },
  { key: 'exam', label: 'Exam' },
  { key: 'deep', label: 'Deep' },
  { key: 'technical', label: 'Technical' },
  { key: 'audio', label: 'Audio' },
] as const

function bestEntryText(currentEntry: KnowledgeEntryProfileDto): string {
  return currentEntry.short_text
    ?? currentEntry.simple_text
    ?? currentEntry.full_text
    ?? currentEntry.exam_text
    ?? currentEntry.technical_text
    ?? 'No explanation has been added for this entry yet.'
}

function blockToText(block: EntryContentBlockDto): string | null {
  if (typeof block.content === 'string') {
    return block.content
  }

  if (block.content && typeof block.content === 'object') {
    const record = block.content as Record<string, unknown>
    const text = record.text ?? record.body ?? record.summary
    if (typeof text === 'string') {
      return text
    }
    return JSON.stringify(block.content)
  }

  return null
}

function prettyEntryType(entryType: string): string {
  return entryType.replace(/_/g, ' ')
}

function humanizeRelation(relationType: string): string {
  return relationType.replace(/_/g, ' ')
}

function safeRecordInteraction() {
  if (studentId.value == null || entryId.value == null) return

  void recordGlossaryInteraction({
    student_id: studentId.value,
    entry_id: entryId.value,
    event_type: 'opened_entry',
    query_text: typeof route.query.q === 'string' ? route.query.q : null,
    metadata: {
      source: 'glossary-entry',
    },
  }).catch(() => {})
}

function openBundleRoute(bundleId: number) {
  if (studentId.value != null && entryId.value != null) {
    void recordGlossaryInteraction({
      student_id: studentId.value,
      entry_id: entryId.value,
      bundle_id: bundleId,
      event_type: 'opened_bundle_route',
      metadata: { source: 'glossary-entry' },
    }).catch(() => {})
  }
  void router.push({ path: '/student/library', hash: '#revision-box' })
}

function openQuestionRoute(questionId: number) {
  if (studentId.value != null && entryId.value != null) {
    void recordGlossaryInteraction({
      student_id: studentId.value,
      entry_id: entryId.value,
      question_id: questionId,
      event_type: 'opened_question_route',
      metadata: { source: 'glossary-entry' },
    }).catch(() => {})
  }
  void router.push('/student/practice')
}

async function loadDetail() {
  if (entryId.value == null) {
    error.value = 'This glossary entry is missing its identifier.'
    detail.value = null
    return
  }

  loading.value = true
  error.value = null

  try {
    detail.value = await getGlossaryEntryDetail(studentId.value, entryId.value, 8, 4)
    safeRecordInteraction()
  } catch {
    detail.value = null
    error.value = 'The glossary entry could not be loaded yet.'
  } finally {
    loading.value = false
  }
}

watch(entryId, () => {
  void loadDetail()
}, { immediate: true })
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <p v-if="loading" class="mb-4 text-sm" :style="{ color: 'var(--ink-muted)' }">Loading glossary entry...</p>
    <p v-if="error" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ error }}</p>

    <template v-if="detail && entry">
      <div class="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <p class="mb-2 text-[10px] font-bold uppercase tracking-[0.16em]" style="color: var(--accent);">Glossary Entry</p>
          <div class="flex flex-wrap items-center gap-2">
            <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">{{ entry.title }}</h1>
            <AppBadge size="xs" color="accent">{{ prettyEntryType(entry.entry_type) }}</AppBadge>
            <AppBadge v-if="entry.has_formula" size="xs" color="warm">formula</AppBadge>
            <AppBadge v-if="entry.audio_available" size="xs" color="gold">audio</AppBadge>
            <AppBadge v-if="detail.student_state?.at_risk_flag" size="xs" color="danger">at risk</AppBadge>
          </div>
          <p class="mt-2 max-w-3xl text-sm" :style="{ color: 'var(--ink-muted)' }">{{ bestEntryText(entry) }}</p>
          <p v-if="entry.phonetic_text" class="mt-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
            Pronunciation: {{ entry.phonetic_text }}
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <AppButton variant="secondary" @click="router.push('/student/glossary')">Back</AppButton>
          <AppButton
            v-if="entry.has_formula"
            variant="secondary"
            @click="router.push({ path: '/student/glossary/formula-lab', query: { entry: String(entry.id) } })"
          >
            Formula Lab
          </AppButton>
          <AppButton
            v-if="detail.confusion_pairs[0]"
            variant="secondary"
            @click="router.push({ path: '/student/glossary/compare', query: { left: String(entry.id), right: String(detail.confusion_pairs[0].paired_entry_id) } })"
          >
            Compare
          </AppButton>
          <AppButton
            v-if="entry.audio_available"
            @click="router.push({ path: '/student/glossary/audio', query: { station: 'all' } })"
          >
            Listen
          </AppButton>
        </div>
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
        <AppCard padding="lg">
          <div class="mb-4 flex flex-wrap gap-2">
            <AppButton
              v-for="mode in modeButtons"
              :key="mode.key"
              size="sm"
              variant="secondary"
              :class="displayMode === mode.key ? '!bg-[var(--accent)] !text-white' : ''"
              @click="displayMode = mode.key"
            >
              {{ mode.label }}
            </AppButton>
          </div>

          <h2 class="mb-3 text-lg font-semibold" :style="{ color: 'var(--ink)' }">Explanation</h2>
          <p class="text-sm leading-7 whitespace-pre-line" :style="{ color: 'var(--ink-secondary)' }">
            {{ primaryText || 'A full explanation has not been added yet.' }}
          </p>

          <div v-if="contentNotes.length" class="mt-4 space-y-2">
            <div
              v-for="note in contentNotes"
              :key="note"
              class="rounded-xl border px-4 py-3 text-sm"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)', color: 'var(--ink-secondary)' }"
            >
              {{ note }}
            </div>
          </div>
        </AppCard>

        <AppCard padding="lg">
          <h2 class="mb-2 text-sm font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Student State</h2>
          <p class="mb-2 text-2xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ Math.round(masteryScore / 100) }}%</p>
          <AppProgress :value="masteryScore" :max="10000" size="md" />
          <p class="mt-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">
            {{ masteryState }}
          </p>

          <div v-if="detail.student_state" class="mt-4 space-y-3 text-xs" :style="{ color: 'var(--ink-secondary)' }">
            <p>Recognition {{ Math.round(detail.student_state.recognition_score / 100) }}%</p>
            <p>Connection {{ Math.round(detail.student_state.connection_score / 100) }}%</p>
            <p>Application {{ Math.round(detail.student_state.application_score / 100) }}%</p>
            <p>Retention {{ Math.round(detail.student_state.retention_score / 100) }}%</p>
            <p v-if="detail.student_state.review_due_at">Review due {{ detail.student_state.review_due_at }}</p>
          </div>
        </AppCard>
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-2">
        <AppCard v-if="detail.aliases.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Aliases</h3>
          <div class="flex flex-wrap gap-2">
            <span
              v-for="alias in detail.aliases"
              :key="`${alias.alias_type}-${alias.alias_text}`"
              class="rounded-full px-3 py-1 text-xs font-semibold"
              :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink)' }"
            >
              {{ alias.alias_text }}
            </span>
          </div>
        </AppCard>

        <AppCard v-if="detail.definition_meta || detail.concept_meta || detail.formula_meta" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Learning Notes</h3>
          <div class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <p v-if="detail.definition_meta?.real_world_meaning">{{ detail.definition_meta.real_world_meaning }}</p>
            <p v-if="detail.concept_meta?.intuition_summary">{{ detail.concept_meta.intuition_summary }}</p>
            <p v-if="detail.formula_meta?.when_to_use">{{ detail.formula_meta.when_to_use }}</p>
            <p v-if="detail.formula_meta?.when_not_to_use" class="text-[var(--danger)]">{{ detail.formula_meta.when_not_to_use }}</p>
          </div>
        </AppCard>
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-2">
        <AppCard v-if="detail.examples.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Examples</h3>
          <div class="space-y-3">
            <div v-for="example in detail.examples" :key="example.id" class="rounded-xl border px-4 py-3" :style="{ borderColor: 'var(--border-soft)' }">
              <p class="text-sm" :style="{ color: 'var(--ink)' }">{{ example.example_text }}</p>
              <p v-if="example.worked_solution_text" class="mt-2 text-xs" :style="{ color: 'var(--ink-muted)' }">
                {{ example.worked_solution_text }}
              </p>
            </div>
          </div>
        </AppCard>

        <AppCard v-if="detail.misconceptions.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Known Misconceptions</h3>
          <div class="space-y-3">
            <div v-for="misconception in detail.misconceptions" :key="misconception.id" class="rounded-xl border px-4 py-3" :style="{ borderColor: 'var(--border-soft)' }">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ misconception.misconception_text }}</p>
              <p v-if="misconception.cause_explanation" class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ misconception.cause_explanation }}</p>
              <p v-if="misconception.correction_explanation" class="mt-2 text-xs font-medium" :style="{ color: 'var(--success)' }">{{ misconception.correction_explanation }}</p>
            </div>
          </div>
        </AppCard>
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-2">
        <AppCard v-if="detail.relations.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Connected Concepts</h3>
          <div class="space-y-2">
            <button
              v-for="relation in detail.relations"
              :key="relation.relation_id"
              class="w-full rounded-xl border px-4 py-3 text-left"
              :style="{ borderColor: 'var(--border-soft)' }"
              @click="router.push(`/student/glossary/entry/${relation.to_entry_id}`)"
            >
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ relation.related_entry_title }}</p>
              <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">
                {{ humanizeRelation(relation.relation_type) }}
                <span v-if="relation.explanation"> · {{ relation.explanation }}</span>
              </p>
            </button>
          </div>
        </AppCard>

        <AppCard v-if="detail.confusion_pairs.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Compare Against</h3>
          <div class="space-y-2">
            <button
              v-for="pair in detail.confusion_pairs"
              :key="pair.paired_entry_id"
              class="w-full rounded-xl border px-4 py-3 text-left"
              :style="{ borderColor: 'var(--border-soft)' }"
              @click="router.push({ path: '/student/glossary/compare', query: { left: String(entry.id), right: String(pair.paired_entry_id) } })"
            >
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ pair.paired_entry_title }}</p>
              <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">
                {{ pair.clue_to_distinguish ?? pair.distinction_explanation }}
              </p>
            </button>
          </div>
        </AppCard>
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-2">
        <AppCard v-if="detail.audio_segments.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Audio Segments</h3>
          <div class="space-y-2">
            <div
              v-for="segment in detail.audio_segments"
              :key="`${segment.sequence_no}-${segment.segment_type}`"
              class="rounded-xl border px-4 py-3"
              :style="{ borderColor: 'var(--border-soft)' }"
            >
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ segment.title }}</p>
              <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ segment.script_text }}</p>
            </div>
          </div>
        </AppCard>

        <AppCard v-if="detail.bundles.length || detail.linked_questions.length" padding="md">
          <h3 class="mb-3 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Linked Routes</h3>
          <div v-if="detail.bundles.length" class="mb-3">
            <p class="mb-2 text-[11px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Bundles</p>
            <div class="space-y-2">
              <button
                v-for="bundle in detail.bundles"
                :key="bundle.bundle_id"
                class="w-full rounded-xl border px-4 py-3 text-left"
                :style="{ borderColor: 'var(--border-soft)' }"
                @click="openBundleRoute(bundle.bundle_id)"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ bundle.title }}</p>
                <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ bundle.description ?? bundle.bundle_type }}</p>
              </button>
            </div>
          </div>

          <div v-if="detail.linked_questions.length">
            <p class="mb-2 text-[11px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Question Links</p>
            <div class="space-y-2">
              <button
                v-for="question in detail.linked_questions"
                :key="question.question_id"
                class="w-full rounded-xl border px-4 py-3 text-left"
                :style="{ borderColor: 'var(--border-soft)' }"
                @click="openQuestionRoute(question.question_id)"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Question {{ question.question_id }}</p>
                <p v-if="question.stem" class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ question.stem }}</p>
              </button>
            </div>
          </div>
        </AppCard>
      </div>
    </template>
  </div>
</template>
