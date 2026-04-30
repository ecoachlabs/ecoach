<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildGlossaryCompareView,
  buildGlossaryHomeSnapshot,
  getGlossaryEntryDetail,
  type GlossaryComparisonViewDto,
  type GlossaryEntryFocusDto,
} from '@/ipc/library'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import GlossaryCompare from '@/components/glossary/GlossaryCompare.vue'

type CompareCandidate = {
  key: string
  leftId: number
  rightId: number
  title: string
  clue: string
}

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const candidates = ref<CompareCandidate[]>([])
const comparison = ref<GlossaryComparisonViewDto | null>(null)
const loading = ref(false)
const loadingCandidates = ref(false)
const error = ref<string | null>(null)

const studentId = computed(() => auth.currentAccount?.id ?? null)
const leftEntryId = computed(() => parseNumber(route.query.left))
const rightEntryId = computed(() => parseNumber(route.query.right))

const conceptA = computed(() => {
  if (!comparison.value) return null
  return {
    title: comparison.value.left_entry.title,
    type: comparison.value.left_entry.entry_type.replace(/_/g, ' '),
    meaning: bestText(comparison.value.left_entry),
    features: [
      comparison.value.left_entry.exam_text,
      comparison.value.left_entry.technical_text,
      comparison.value.left_entry.audio_available ? 'Audio explanation is available.' : null,
      comparison.value.left_entry.has_formula ? 'Formula support is available.' : null,
    ].filter((value): value is string => Boolean(value)),
  }
})

const conceptB = computed(() => {
  if (!comparison.value) return null
  return {
    title: comparison.value.right_entry.title,
    type: comparison.value.right_entry.entry_type.replace(/_/g, ' '),
    meaning: bestText(comparison.value.right_entry),
    features: [
      comparison.value.right_entry.exam_text,
      comparison.value.right_entry.technical_text,
      comparison.value.right_entry.audio_available ? 'Audio explanation is available.' : null,
      comparison.value.right_entry.has_formula ? 'Formula support is available.' : null,
    ].filter((value): value is string => Boolean(value)),
  }
})

const confusionPoints = computed(() => {
  if (!comparison.value) return []
  return [
    comparison.value.distinction_explanation,
    comparison.value.clue_to_distinguish,
  ].filter((value): value is string => Boolean(value))
})

function parseNumber(value: unknown): number | null {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

function bestText(entry: GlossaryComparisonViewDto['left_entry']): string {
  return entry.short_text
    ?? entry.simple_text
    ?? entry.full_text
    ?? entry.technical_text
    ?? 'No summary is available for this glossary entry yet.'
}

async function loadCandidates() {
  if (studentId.value == null) return

  loadingCandidates.value = true
  try {
    const home = await buildGlossaryHomeSnapshot(studentId.value, null, 8)
    const seedEntries = dedupeEntries([
      ...home.weak_entries,
      ...home.exam_hotspots,
      ...home.discover,
    ]).slice(0, 6)

    const details = await Promise.all(
      seedEntries.map(entry =>
        getGlossaryEntryDetail(studentId.value, entry.entry_id, 4, 2).catch(() => null),
      ),
    )

    const nextCandidates: CompareCandidate[] = []
    const seen = new Set<string>()

    for (const detail of details) {
      if (!detail) continue
      for (const pair of detail.confusion_pairs) {
        const key = [detail.entry.id, pair.paired_entry_id].sort((left, right) => left - right).join(':')
        if (seen.has(key)) continue
        seen.add(key)
        nextCandidates.push({
          key,
          leftId: detail.entry.id,
          rightId: pair.paired_entry_id,
          title: `${detail.entry.title} vs ${pair.paired_entry_title}`,
          clue: pair.clue_to_distinguish ?? pair.distinction_explanation,
        })
      }
    }

    candidates.value = nextCandidates

    if (leftEntryId.value == null || rightEntryId.value == null) {
      const fallback = nextCandidates[0]
      if (fallback) {
        router.replace({ query: { left: String(fallback.leftId), right: String(fallback.rightId) } })
      }
    }
  } finally {
    loadingCandidates.value = false
  }
}

function dedupeEntries(entries: GlossaryEntryFocusDto[]): GlossaryEntryFocusDto[] {
  const merged = new Map<number, GlossaryEntryFocusDto>()
  for (const entry of entries) {
    if (!merged.has(entry.entry_id)) {
      merged.set(entry.entry_id, entry)
    }
  }
  return Array.from(merged.values())
}

async function loadComparison() {
  if (leftEntryId.value == null || rightEntryId.value == null) {
    comparison.value = null
    return
  }

  loading.value = true
  error.value = null

  try {
    comparison.value = await buildGlossaryCompareView(leftEntryId.value, rightEntryId.value)
  } catch {
    comparison.value = null
    error.value = 'That comparison could not be loaded yet.'
  } finally {
    loading.value = false
  }
}

function selectCandidate(candidate: CompareCandidate) {
  router.replace({ query: { left: String(candidate.leftId), right: String(candidate.rightId) } })
}

function openBundleRoute() {
  router.push({ path: '/student/library', hash: '#revision-box' })
}

watch(() => [leftEntryId.value, rightEntryId.value], () => {
  void loadComparison()
}, { immediate: true })

onMounted(async () => {
  await loadCandidates()
})
</script>

<template>
  <div class="mx-auto max-w-6xl p-6 lg:p-8">
    <div class="mb-6">
      <p class="mb-2 text-[10px] font-bold uppercase tracking-[0.16em]" style="color: #10B981;">Glossary Tool</p>
      <h1 class="mb-2 font-display text-2xl font-black tracking-tight" :style="{ color: 'var(--ink)' }">Compare Terms</h1>
      <p class="max-w-2xl text-sm" :style="{ color: 'var(--ink-muted)' }">
        Pull live confusion pairs from the glossary graph and lay them side by side.
      </p>
    </div>

    <p v-if="loadingCandidates" class="mb-4 text-sm" :style="{ color: 'var(--ink-muted)' }">Loading comparison pairs...</p>
    <p v-if="error" class="mb-4 text-sm font-medium text-[var(--danger)]">{{ error }}</p>

    <div class="grid gap-6 lg:grid-cols-[260px_minmax(0,1fr)]">
      <AppCard padding="md">
        <p class="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">Comparison Sets</p>
        <div v-if="candidates.length" class="space-y-2">
          <button
            v-for="candidate in candidates"
            :key="candidate.key"
            class="w-full rounded-2xl border px-4 py-3 text-left transition-colors"
            :style="{
              borderColor: leftEntryId === candidate.leftId && rightEntryId === candidate.rightId ? '#10B981' : 'var(--border-soft)',
              backgroundColor: leftEntryId === candidate.leftId && rightEntryId === candidate.rightId ? 'rgba(16,185,129,0.08)' : 'var(--surface)',
            }"
            @click="selectCandidate(candidate)"
          >
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ candidate.title }}</p>
            <p class="mt-1 text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ candidate.clue }}</p>
          </button>
        </div>
        <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No live confusion pairs are available yet.</p>
      </AppCard>

      <div class="space-y-6">
        <AppCard v-if="comparison && conceptA && conceptB" padding="lg">
          <GlossaryCompare
            :concept-a="conceptA"
            :concept-b="conceptB"
            :shared-features="comparison.shared_relation_types.map(relation => relation.replace(/_/g, ' '))"
            :confusion-points="confusionPoints"
          />

          <div class="mt-5 flex flex-wrap items-center gap-3">
            <AppButton variant="secondary" @click="router.push('/student/glossary')">Back to Glossary</AppButton>
            <AppButton variant="secondary" @click="router.push(`/student/glossary/entry/${comparison.left_entry.id}`)">Open {{ comparison.left_entry.title }}</AppButton>
            <AppButton variant="secondary" @click="router.push(`/student/glossary/entry/${comparison.right_entry.id}`)">Open {{ comparison.right_entry.title }}</AppButton>
          </div>
        </AppCard>

        <div v-if="comparison" class="grid gap-4 lg:grid-cols-2">
          <AppCard v-if="comparison.shared_bundles.length" padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Shared Bundles</p>
            <div class="space-y-2">
              <button
                v-for="bundle in comparison.shared_bundles"
                :key="bundle.bundle_id"
                class="w-full rounded-xl border px-4 py-3 text-left"
                :style="{ borderColor: 'var(--border-soft)' }"
                @click="openBundleRoute"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ bundle.title }}</p>
                <p class="mt-1 text-xs" :style="{ color: 'var(--ink-muted)' }">{{ bundle.bundle_type }}</p>
              </button>
            </div>
          </AppCard>

          <AppCard padding="md">
            <p class="mb-2 text-xs font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Question Reach</p>
            <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
              {{ comparison.linked_question_ids.length }} linked question<span v-if="comparison.linked_question_ids.length !== 1">s</span>
              reference this contrast.
            </p>
          </AppCard>
        </div>

        <AppCard v-else-if="!loading" padding="lg">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Choose a live confusion pair to compare.</p>
        </AppCard>
      </div>
    </div>
  </div>
</template>
