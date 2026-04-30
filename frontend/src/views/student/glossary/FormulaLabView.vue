<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildGlossaryHomeSnapshot,
  getFormulaLabView,
  searchCatalog,
  type FormulaLabViewDto,
  type GlossaryEntryFocusDto,
  type GlossarySearchResultDto,
} from '@/ipc/library'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormulaLab from '@/components/glossary/FormulaLab.vue'

type FormulaCandidate = {
  entryId: number
  title: string
  reason: string
}

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const snapshotEntries = ref<GlossaryEntryFocusDto[]>([])
const searchCandidates = ref<GlossarySearchResultDto[]>([])
const formulaView = ref<FormulaLabViewDto | null>(null)
const loading = ref(false)
const loadingCandidates = ref(false)
const error = ref<string | null>(null)

const studentId = computed(() => auth.currentAccount?.id ?? null)
const selectedEntryId = computed<number | null>(() => {
  const raw = Number(route.query.entry)
  return Number.isFinite(raw) ? raw : null
})

const formulaCandidates = computed<FormulaCandidate[]>(() => {
  const merged = new Map<number, FormulaCandidate>()

  for (const entry of snapshotEntries.value) {
    if (entry.entry_type !== 'formula') continue
    merged.set(entry.entry_id, {
      entryId: entry.entry_id,
      title: entry.title,
      reason: entry.match_reason,
    })
  }

  for (const result of searchCandidates.value) {
    if (result.entry_id == null || result.entry_type !== 'formula') continue
    if (!merged.has(result.entry_id)) {
      merged.set(result.entry_id, {
        entryId: result.entry_id,
        title: result.title,
        reason: result.match_reason,
      })
    }
  }

  return Array.from(merged.values())
})

const variableRows = computed(() => {
  const formulaMeta = formulaView.value?.formula_meta
  if (!formulaMeta) return []

  const variables = formulaMeta.variables
  const units = asRecord(formulaMeta.units)

  if (Array.isArray(variables)) {
    return variables
      .filter((value): value is Record<string, unknown> => Boolean(value) && typeof value === 'object')
      .map(value => ({
        symbol: String(value.symbol ?? value.key ?? ''),
        meaning: String(value.meaning ?? value.name ?? value.symbol ?? 'variable'),
        unit: String(value.unit ?? ''),
      }))
  }

  const variableMap = asRecord(variables)
  return Object.entries(variableMap).map(([meaning, symbol]) => ({
    symbol: String(symbol),
    meaning: startCase(meaning),
    unit: String(units[meaning] ?? units[String(symbol)] ?? ''),
  }))
})

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {}
}

function startCase(value: string): string {
  return value
    .replace(/_/g, ' ')
    .replace(/\b\w/g, char => char.toUpperCase())
}

async function loadCandidates() {
  if (studentId.value == null) return

  loadingCandidates.value = true
  try {
    const [homeSnapshot, formulaSearch] = await Promise.all([
      buildGlossaryHomeSnapshot(studentId.value, null, 12).catch(() => null),
      searchCatalog(
        {
          query: 'formula',
          student_id: studentId.value,
          subject_id: null,
          topic_id: null,
          include_bundles: false,
          include_questions: false,
          include_confusions: false,
          include_audio_ready_only: false,
        },
        10,
      ).catch(() => null),
    ])

    snapshotEntries.value = [
      ...(homeSnapshot?.discover ?? []),
      ...(homeSnapshot?.weak_entries ?? []),
      ...(homeSnapshot?.exam_hotspots ?? []),
    ]
    searchCandidates.value = formulaSearch?.groups.flatMap(group => group.results) ?? []

    if (selectedEntryId.value == null && formulaCandidates.value[0]) {
      router.replace({ query: { entry: String(formulaCandidates.value[0].entryId) } })
    }
  } finally {
    loadingCandidates.value = false
  }
}

async function loadFormula(entryId: number | null) {
  if (entryId == null) {
    formulaView.value = null
    return
  }

  loading.value = true
  error.value = null

  try {
    formulaView.value = await getFormulaLabView(entryId)
  } catch {
    formulaView.value = null
    error.value = 'This formula entry could not be loaded yet.'
  } finally {
    loading.value = false
  }
}

function selectFormula(entryId: number) {
  router.replace({ query: { entry: String(entryId) } })
}

watch(selectedEntryId, entryId => {
  void loadFormula(entryId)
}, { immediate: true })

onMounted(async () => {
  await loadCandidates()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-6">
      <p class="mb-2 text-[10px] font-bold uppercase tracking-[0.16em]" style="color: #3B82F6;">Glossary Tool</p>
      <h1 class="mb-2 font-display text-2xl font-black tracking-tight" :style="{ color: 'var(--ink)' }">Formula Lab</h1>
      <p class="max-w-2xl text-sm" :style="{ color: 'var(--ink-muted)' }">
        Work from real glossary formula entries, not fixed presets.
      </p>
    </div>

    <p v-if="loadingCandidates" class="mb-4 text-sm" :style="{ color: 'var(--ink-muted)' }">Loading formula entries...</p>
    <p v-if="error" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ error }}</p>

    <div class="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
      <AppCard padding="md">
        <p class="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">Formula Entries</p>
        <div v-if="formulaCandidates.length" class="space-y-2">
          <button
            v-for="candidate in formulaCandidates"
            :key="candidate.entryId"
            class="w-full rounded-2xl border px-4 py-3 text-left transition-colors"
            :style="{
              borderColor: selectedEntryId === candidate.entryId ? '#3B82F6' : 'var(--border-soft)',
              backgroundColor: selectedEntryId === candidate.entryId ? 'rgba(59,130,246,0.08)' : 'var(--surface)',
            }"
            @click="selectFormula(candidate.entryId)"
          >
            <p class="mb-1 text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ candidate.title }}</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ candidate.reason }}</p>
          </button>
        </div>
        <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No formula entries are active in the live glossary yet.</p>
      </AppCard>

      <div class="space-y-6">
        <AppCard v-if="formulaView" padding="lg">
          <FormulaLab
            :formula-name="formulaView.entry.title"
            :formula="formulaView.formula_meta.formula_latex ?? formulaView.formula_meta.formula_expression"
            :variables="variableRows"
          />

          <div class="mt-5 flex flex-wrap items-center gap-3">
            <AppButton variant="secondary" @click="router.push('/student/glossary')">Back to Glossary</AppButton>
            <AppButton variant="secondary" @click="router.push(`/student/glossary/entry/${formulaView.entry.id}`)">Open Entry</AppButton>
          </div>
        </AppCard>

        <div v-if="formulaView" class="grid gap-4 lg:grid-cols-2">
          <AppCard padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">When To Use</p>
            <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
              {{ formulaView.formula_meta.when_to_use ?? 'Use this formula when the variables in the setup match the expression.' }}
            </p>
            <p v-if="formulaView.formula_meta.when_not_to_use" class="mt-3 text-sm font-medium text-[var(--danger)]">
              {{ formulaView.formula_meta.when_not_to_use }}
            </p>
          </AppCard>

          <AppCard padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Rearrangements</p>
            <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
              <li v-for="rearrangement in formulaView.formula_meta.rearrangements" :key="rearrangement">{{ rearrangement }}</li>
            </ul>
          </AppCard>

          <AppCard v-if="formulaView.formula_meta.common_errors.length" padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Common Errors</p>
            <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
              <li v-for="errorItem in formulaView.formula_meta.common_errors" :key="errorItem">{{ errorItem }}</li>
            </ul>
          </AppCard>

          <AppCard v-if="formulaView.examples.length" padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Worked Examples</p>
            <div class="space-y-3">
              <div v-for="example in formulaView.examples" :key="example.id" class="rounded-xl border px-4 py-3" :style="{ borderColor: 'var(--border-soft)' }">
                <p class="text-sm" :style="{ color: 'var(--ink)' }">{{ example.example_text }}</p>
                <p v-if="example.worked_solution_text" class="mt-2 text-xs" :style="{ color: 'var(--ink-muted)' }">
                  {{ example.worked_solution_text }}
                </p>
              </div>
            </div>
          </AppCard>

          <AppCard v-if="formulaView.related_entries.length" padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Related Entries</p>
            <div class="space-y-2">
              <button
                v-for="relation in formulaView.related_entries"
                :key="relation.relation_id"
                class="w-full rounded-xl border px-4 py-3 text-left"
                :style="{ borderColor: 'var(--border-soft)' }"
                @click="router.push(`/student/glossary/entry/${relation.to_entry_id}`)"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ relation.related_entry_title }}</p>
                <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ relation.relation_type.replace(/_/g, ' ') }}</p>
              </button>
            </div>
          </AppCard>

          <AppCard v-if="formulaView.misconceptions.length" padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Misconceptions</p>
            <ul class="space-y-2 text-sm" :style="{ color: 'var(--ink-secondary)' }">
              <li v-for="misconception in formulaView.misconceptions" :key="misconception.id">
                {{ misconception.misconception_text }}
              </li>
            </ul>
          </AppCard>
        </div>

        <AppCard v-else-if="!loading" padding="lg">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Choose a live formula entry to begin.</p>
        </AppCard>
      </div>
    </div>
  </div>
</template>
