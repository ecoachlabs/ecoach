<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  getStudentCurriculumHome,
  getStudentSubjectCurriculumMap,
  type CurriculumStudentHomeSnapshot,
  type CurriculumStudentNodeState,
  type CurriculumStudentSubjectCardDto,
  type CurriculumStudentSubjectMap,
} from '@/ipc/curriculum'
import { useAuthStore } from '@/stores/auth'
import PageHeader from '@/components/layout/PageHeader.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'

type BadgeTone = 'accent' | 'warm' | 'gold' | 'success' | 'danger' | 'muted'
type ProgressTone = 'accent' | 'warm' | 'gold' | 'success' | 'danger'

interface BreakdownGroup {
  key: string
  label: string
  order: number
  total: number
  blocked: number
  reviewDue: number
  examReady: number
  nodes: CurriculumStudentNodeState[]
}

const auth = useAuthStore()
const router = useRouter()

const loadingHome = ref(false)
const loadingMap = ref(false)
const homeError = ref('')
const mapError = ref('')
const home = ref<CurriculumStudentHomeSnapshot | null>(null)
const subjectMap = ref<CurriculumStudentSubjectMap | null>(null)

const selectedSubjectTrackId = ref<number | null>(null)
const activeLevelKey = ref<string | null>(null)
const activeScope = ref<'stage' | 'all-stages'>('all-stages')
const activeStrandId = ref<number | null>(null)
const activeSubStrandId = ref<number | null>(null)
const selectedTopicId = ref<number | null>(null)
const searchQuery = ref('')

const studentId = computed(() => auth.currentAccount?.id ?? null)
const subjectCards = computed(() => home.value?.subject_cards ?? [])
const selectedSubjectCard = computed(
  () =>
    subjectCards.value.find((item) => item.subject_track_id === selectedSubjectTrackId.value) ??
    subjectCards.value[0] ??
    null,
)
const selectedSubjectOverview = computed(() => subjectMap.value?.overview ?? selectedSubjectCard.value)
const allNodeStates = computed(() => subjectMap.value?.nodes ?? [])
const recommendedTopics = computed(() => subjectMap.value?.recommended_topics ?? home.value?.recommended_topics ?? [])

const nodeById = computed(() => {
  const map = new Map<number, CurriculumStudentNodeState>()
  for (const state of allNodeStates.value) map.set(state.node.id, state)
  return map
})

const childLookup = computed(() => {
  const map = new Map<number | null, number[]>()
  for (const state of allNodeStates.value) {
    const parent = state.node.parent_node_id
    const existing = map.get(parent) ?? []
    existing.push(state.node.id)
    map.set(parent, existing)
  }
  for (const ids of map.values()) {
    ids.sort((leftId, rightId) => {
      const left = nodeById.value.get(leftId)
      const right = nodeById.value.get(rightId)
      if (!left || !right) return 0
      return sortNodeStates(left, right)
    })
  }
  return map
})

const levelGroups = computed(() =>
  buildGroups(
    allNodeStates.value,
    (item) => levelKey(item.node.level_id),
    (item) => levelLabel(item.node.level_id),
    (item) => item.node.level_id ?? 9999,
  ),
)

const levelMap = computed(() => {
  const map = new Map<string, BreakdownGroup>()
  for (const group of levelGroups.value) map.set(group.key, group)
  return map
})

const activeLevelNodes = computed(() => {
  if (!activeLevelKey.value) return []
  return levelMap.value.get(activeLevelKey.value)?.nodes ?? []
})

const scopedNodes = computed(() => {
  if (activeScope.value === 'stage') return activeLevelNodes.value
  return allNodeStates.value
})

const primaryNodes = computed(() => {
  const roots = scopedNodes.value.filter((item) => {
    const parentId = item.node.parent_node_id
    if (parentId == null) return true
    return !scopedNodes.value.some((candidate) => candidate.node.id === parentId)
  })
  return [...roots].sort(sortNodeStates)
})

const selectedPrimaryNode = computed(() => {
  if (activeStrandId.value == null) return null
  return nodeById.value.get(activeStrandId.value) ?? null
})

const secondaryNodes = computed(() => {
  if (!selectedPrimaryNode.value) return []
  const children = childLookup.value.get(selectedPrimaryNode.value.node.id) ?? []
  return children
    .map((id) => nodeById.value.get(id))
    .filter((item): item is CurriculumStudentNodeState => Boolean(item))
    .sort(sortNodeStates)
})

const selectedSecondaryNode = computed(() => {
  if (activeSubStrandId.value == null) return null
  return nodeById.value.get(activeSubStrandId.value) ?? null
})
const actionableNodes = computed(() => {
  const nodeIds = new Set<number>()

  const addDescendants = (rootId: number) => {
    const stack = [rootId]
    while (stack.length) {
      const currentId = stack.pop() as number
      const state = nodeById.value.get(currentId)
      if (state) {
        const childCount = childLookup.value.get(currentId)?.length ?? 0
        const actionable = state.node.legacy_topic_id != null || childCount === 0
        if (actionable) nodeIds.add(currentId)
      }
      const children = childLookup.value.get(currentId) ?? []
      for (const childId of children) stack.push(childId)
    }
  }

  if (selectedSecondaryNode.value) {
    addDescendants(selectedSecondaryNode.value.node.id)
  } else if (selectedPrimaryNode.value) {
    addDescendants(selectedPrimaryNode.value.node.id)
  } else if (primaryNodes.value.length) {
    for (const node of primaryNodes.value) addDescendants(node.node.id)
  } else {
    for (const item of scopedNodes.value) addDescendants(item.node.id)
  }

  return [...nodeIds]
    .map((id) => nodeById.value.get(id))
    .filter((item): item is CurriculumStudentNodeState => Boolean(item))
    .sort(sortNodeStates)
})

const filteredActionableNodes = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return actionableNodes.value
  return actionableNodes.value.filter((item) => {
    const haystack = [
      item.node.public_title,
      item.node.canonical_title,
      item.node.public_summary ?? '',
      item.node.official_text ?? '',
      item.reason,
    ]
      .join(' ')
      .toLowerCase()
    return haystack.includes(query)
  })
})

const selectedTopicState = computed(() => {
  const current = selectedTopicId.value != null ? nodeById.value.get(selectedTopicId.value) : null
  if (current && filteredActionableNodes.value.some((item) => item.node.id === current.node.id)) return current
  return filteredActionableNodes.value[0] ?? null
})

const selectedTopicChildren = computed(() => {
  const selected = selectedTopicState.value
  if (!selected) return []
  const childIds = childLookup.value.get(selected.node.id) ?? []
  return childIds
    .map((id) => nodeById.value.get(id))
    .filter((item): item is CurriculumStudentNodeState => Boolean(item))
    .sort(sortNodeStates)
})

const selectedTopicPath = computed(() => {
  const selected = selectedTopicState.value
  if (!selected) return []
  const path: CurriculumStudentNodeState[] = []
  let cursor: CurriculumStudentNodeState | undefined = selected
  while (cursor) {
    path.unshift(cursor)
    const parentId = cursor.node.parent_node_id
    if (parentId == null) break
    cursor = nodeById.value.get(parentId)
  }
  return path
})

const selectedLaunchTopicId = computed(() => selectedTopicState.value?.node.legacy_topic_id ?? null)

const activeTypeBreakdown = computed(() =>
  buildGroups(
    activeLevelNodes.value,
    (item) => nodeTypeKey(item.node.node_type),
    (item) => groupLabelForType(item.node.node_type),
    () => 1,
  ),
)

watch(
  levelGroups,
  (groups) => {
    if (!groups.length) {
      activeLevelKey.value = null
      return
    }
    if (!activeLevelKey.value || !groups.some((item) => item.key === activeLevelKey.value)) {
      activeLevelKey.value = groups[0].key
    }
  },
  { immediate: true },
)

watch(
  primaryNodes,
  (items) => {
    if (!items.length) {
      activeStrandId.value = null
      return
    }
    if (activeStrandId.value == null || !items.some((item) => item.node.id === activeStrandId.value)) {
      activeStrandId.value = items[0].node.id
    }
  },
  { immediate: true },
)

watch(
  secondaryNodes,
  (items) => {
    if (!items.length) {
      activeSubStrandId.value = null
      return
    }
    if (activeSubStrandId.value == null || !items.some((item) => item.node.id === activeSubStrandId.value)) {
      activeSubStrandId.value = items[0].node.id
    }
  },
  { immediate: true },
)

watch(
  filteredActionableNodes,
  (items) => {
    if (!items.length) {
      selectedTopicId.value = null
      return
    }
    if (selectedTopicId.value == null || !items.some((item) => item.node.id === selectedTopicId.value)) {
      selectedTopicId.value = items[0].node.id
    }
  },
  { immediate: true },
)

onMounted(() => {
  void loadHome()
})

async function loadHome() {
  const sid = studentId.value
  if (sid == null) {
    homeError.value = 'No student account is active, so the curriculum cannot be loaded.'
    home.value = null
    subjectMap.value = null
    return
  }

  loadingHome.value = true
  homeError.value = ''

  try {
    const data = await getStudentCurriculumHome(sid, null)
    home.value = data
    const nextSubject = selectInitialSubject(data.subject_cards)
    if (nextSubject == null) {
      selectedSubjectTrackId.value = null
      subjectMap.value = null
      return
    }
    selectedSubjectTrackId.value = nextSubject
    await loadSubjectMap(nextSubject)
  } catch (error) {
    homeError.value = extractError(error, 'Failed to load curriculum overview.')
    home.value = null
    subjectMap.value = null
    selectedSubjectTrackId.value = null
  } finally {
    loadingHome.value = false
  }
}

async function loadSubjectMap(subjectTrackId: number) {
  const sid = studentId.value
  if (sid == null) return

  loadingMap.value = true
  mapError.value = ''
  selectedTopicId.value = null
  activeStrandId.value = null
  activeSubStrandId.value = null
  searchQuery.value = ''

  try {
    subjectMap.value = await getStudentSubjectCurriculumMap(sid, subjectTrackId)
  } catch (error) {
    mapError.value = extractError(error, 'Failed to load the selected subject curriculum.')
    subjectMap.value = null
    selectedTopicId.value = null
  } finally {
    loadingMap.value = false
  }
}
function selectSubject(subjectTrackId: number) {
  if (selectedSubjectTrackId.value === subjectTrackId && subjectMap.value) return
  selectedSubjectTrackId.value = subjectTrackId
  void loadSubjectMap(subjectTrackId)
}

function selectLevel(levelKeyValue: string) {
  if (activeLevelKey.value === levelKeyValue) return
  activeLevelKey.value = levelKeyValue
  activeScope.value = 'stage'
  activeStrandId.value = null
  activeSubStrandId.value = null
  selectedTopicId.value = null
}

function selectScope(scope: 'stage' | 'all-stages') {
  if (activeScope.value === scope) return
  activeScope.value = scope
  activeStrandId.value = null
  activeSubStrandId.value = null
  selectedTopicId.value = null
}

function selectStrand(strandId: number) {
  if (activeStrandId.value === strandId) return
  activeStrandId.value = strandId
  activeSubStrandId.value = null
  selectedTopicId.value = null
}

function selectSubStrand(subStrandId: number | null) {
  activeSubStrandId.value = subStrandId
  selectedTopicId.value = null
}

function selectTopic(topicId: number) {
  selectedTopicId.value = topicId
}

function refreshCurriculum() {
  void loadHome()
}

function resetFocus() {
  activeScope.value = 'all-stages'
  activeStrandId.value = null
  activeSubStrandId.value = null
  selectedTopicId.value = null
  searchQuery.value = ''
}

function clearSearch() {
  searchQuery.value = ''
}

function openTeachMode() {
  if (selectedLaunchTopicId.value == null) return
  void router.push({ name: 'teach', params: { topicId: selectedLaunchTopicId.value } })
}

function openPracticeMode() {
  if (selectedLaunchTopicId.value == null) return
  void router.push({ path: '/student/practice', query: { topicId: String(selectedLaunchTopicId.value), from: 'curriculum' } })
}

function progressTone(value: number): ProgressTone {
  if (value >= 75) return 'success'
  if (value >= 50) return 'accent'
  if (value >= 30) return 'warm'
  return 'danger'
}

function badgeToneForStatus(status: string): BadgeTone {
  const normalized = status.toLowerCase()
  if (normalized.includes('exam')) return 'success'
  if (normalized.includes('block')) return 'danger'
  if (normalized.includes('review') || normalized.includes('fragile') || normalized.includes('slip')) return 'warm'
  if (normalized.includes('stable') || normalized.includes('steady') || normalized.includes('strong')) return 'gold'
  if (normalized.includes('not started') || normalized.includes('new')) return 'muted'
  return 'accent'
}

function extractError(error: unknown, fallback: string) {
  if (typeof error === 'string') return error
  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message
  }
  return fallback
}

function selectInitialSubject(cards: CurriculumStudentSubjectCardDto[]) {
  return cards[0]?.subject_track_id ?? null
}

function levelKey(levelId: number | null) {
  return levelId == null ? 'unassigned' : String(levelId)
}

function levelLabel(levelId: number | null) {
  return levelId == null ? 'Unassigned stage' : `Stage ${levelId}`
}

function nodeTypeKey(nodeType: string) {
  return nodeType.trim().toLowerCase().replace(/\s+/g, '_')
}

function formatNodeType(nodeType: string) {
  return nodeType
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
    .split(' ')
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

function groupLabelForType(nodeType: string) {
  return formatNodeType(nodeType)
}

function buildGroups(
  nodes: CurriculumStudentNodeState[],
  keyFor: (item: CurriculumStudentNodeState) => string,
  labelFor: (item: CurriculumStudentNodeState) => string,
  orderFor: (item: CurriculumStudentNodeState) => number,
) {
  const groups = new Map<string, BreakdownGroup>()
  for (const item of nodes) {
    const key = keyFor(item)
    const current = groups.get(key) ?? { key, label: labelFor(item), order: orderFor(item), total: 0, blocked: 0, reviewDue: 0, examReady: 0, nodes: [] }
    current.total += 1
    current.blocked += item.blocked ? 1 : 0
    current.reviewDue += item.review_due ? 1 : 0
    current.examReady += item.exam_ready ? 1 : 0
    current.nodes.push(item)
    groups.set(key, current)
  }
  return [...groups.values()].sort((left, right) => left.order - right.order || left.label.localeCompare(right.label))
}

function sortNodeStates(left: CurriculumStudentNodeState, right: CurriculumStudentNodeState) {
  return left.node.sequence_no - right.node.sequence_no || left.node.public_title.localeCompare(right.node.public_title)
}
</script>

<template>
  <div class="curriculum-shell mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
    <PageHeader title="Curriculum Explorer" subtitle="Move from subject to stage and structure, then drill into learning nodes in one guided flow.">
      <template #actions>
        <div class="flex items-center gap-2">
          <AppButton variant="secondary" size="sm" :loading="loadingHome || loadingMap" @click="refreshCurriculum">Refresh</AppButton>
          <AppBadge v-if="home?.curriculum_version" color="accent" size="sm">{{ home.curriculum_version.version_label }}</AppBadge>
        </div>
      </template>
    </PageHeader>

    <div v-if="homeError" class="mb-5 rounded-[var(--radius-lg)] border px-4 py-3 text-sm" :style="{ backgroundColor: 'rgba(220, 38, 38, 0.08)', borderColor: 'rgba(220, 38, 38, 0.16)', color: 'var(--danger)' }">
      {{ homeError }}
    </div>

    <div v-if="loadingHome && !home" class="space-y-5">
      <div class="grid gap-4 md:grid-cols-3"><AppSkeleton v-for="i in 3" :key="`hero-${i}`" height="120px" /></div>
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3"><AppSkeleton v-for="i in 3" :key="`subject-skeleton-${i}`" height="188px" /></div>
      <div class="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]"><AppSkeleton height="520px" /><AppSkeleton height="520px" /></div>
    </div>

    <template v-else>
      <AppCard padding="lg" class="hero-card mb-6">
        <div class="hero-grid">
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-2">
              <AppBadge v-if="home?.curriculum_version" color="accent" size="sm">{{ home.curriculum_version.name }}</AppBadge>
              <AppBadge v-if="home?.curriculum_version.country" color="muted" size="sm">{{ home.curriculum_version.country }}</AppBadge>
              <AppBadge v-if="home?.curriculum_version.education_stage" color="gold" size="sm">{{ home.curriculum_version.education_stage }}</AppBadge>
            </div>
            <h2 class="hero-title">Follow the curriculum in clear steps</h2>
            <p class="hero-copy">{{ home?.position_statement || 'Choose a subject, pick your scope, and drill into the curriculum graph.' }}</p>
          </div>
          <div class="hero-metrics">
            <div class="metric-box"><p class="metric-label">Entered</p><p class="metric-value">{{ home?.entered_percent ?? 0 }}%</p><AppProgress :value="home?.entered_percent ?? 0" :max="100" size="sm" color="accent" /></div>
            <div class="metric-box"><p class="metric-label">Stable</p><p class="metric-value">{{ home?.stable_percent ?? 0 }}%</p><AppProgress :value="home?.stable_percent ?? 0" :max="100" size="sm" color="success" /></div>
            <div class="metric-box"><p class="metric-label">Exam Ready</p><p class="metric-value">{{ home?.exam_readiness_percent ?? 0 }}%</p><AppProgress :value="home?.exam_readiness_percent ?? 0" :max="100" size="sm" :color="progressTone(home?.exam_readiness_percent ?? 0)" /></div>
          </div>
        </div>
      </AppCard>

      <section class="mb-6">
        <div class="mb-3 flex items-center justify-between gap-3">
          <p class="section-label">Step 1. Pick a subject</p>
          <p class="section-meta">{{ subjectCards.length }} subject{{ subjectCards.length === 1 ? '' : 's' }}</p>
        </div>
        <div v-if="subjectCards.length" class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <button v-for="card in subjectCards" :key="card.subject_track_id" class="subject-card" :class="{ 'subject-card--active': selectedSubjectTrackId === card.subject_track_id }" :aria-pressed="selectedSubjectTrackId === card.subject_track_id" @click="selectSubject(card.subject_track_id)">
            <div class="flex items-start justify-between gap-3">
              <div class="text-left"><p class="subject-title">{{ card.public_title }}</p><p class="subject-subtitle">{{ card.trend_label }}</p></div>
              <AppBadge :color="badgeToneForStatus(card.trend_label)" size="xs">{{ card.exam_ready_percent }}% ready</AppBadge>
            </div>
            <div class="mt-3 space-y-2">
              <div class="subject-progress"><span>Entered</span><span>{{ card.entered_percent }}%</span></div>
              <AppProgress :value="card.entered_percent" :max="100" size="sm" color="accent" />
              <div class="subject-progress"><span>Stable</span><span>{{ card.stable_percent }}%</span></div>
              <AppProgress :value="card.stable_percent" :max="100" size="sm" color="success" />
            </div>
            <div class="mt-3 flex flex-wrap gap-2"><AppBadge color="warm" size="xs">Weak {{ card.weak_area_count }}</AppBadge><AppBadge color="danger" size="xs">Blocked {{ card.blocked_count }}</AppBadge><AppBadge color="gold" size="xs">Review {{ card.review_due_count }}</AppBadge></div>
          </button>
        </div>
        <AppCard v-else padding="lg"><p class="empty-copy">No subject curriculum is available yet for this account.</p></AppCard>
      </section>

      <Transition name="panel-swap" mode="out-in">
        <section v-if="selectedSubjectOverview" :key="selectedSubjectTrackId ?? 'overview'" class="grid gap-5 xl:grid-cols-[1.15fr_0.85fr]">
          <div class="space-y-5">
            <AppCard padding="md">
              <div class="step-grid">
                <div class="step-card"><p class="step-label">1 Subject</p><p class="step-value">{{ selectedSubjectOverview.public_title }}</p></div>
                <div class="step-card"><p class="step-label">2 Scope</p><p class="step-value">{{ activeScope === 'all-stages' ? 'All stages' : levelMap.get(activeLevelKey || '')?.label ?? 'Selected stage' }}</p></div>
                <div class="step-card"><p class="step-label">3 Structure</p><p class="step-value">{{ selectedSecondaryNode?.node.public_title || selectedPrimaryNode?.node.public_title || 'Choose a branch' }}</p></div>
                <div class="step-card"><p class="step-label">4 Learning node</p><p class="step-value">{{ selectedTopicState?.node.public_title || 'Choose a node' }}</p></div>
              </div>
            </AppCard>

            <AppCard padding="lg">
              <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
                <div><p class="section-label">Step 2. Scope and stage distribution</p><h3 class="section-title">Explore all stages or focus on one stage</h3></div>
                <AppButton variant="ghost" size="sm" @click="resetFocus">Reset drilldown</AppButton>
              </div>
              <div class="mb-4 flex flex-wrap gap-2">
                <button class="substrand-chip" :class="{ 'substrand-chip--active': activeScope === 'all-stages' }" @click="selectScope('all-stages')">All stages</button>
                <button class="substrand-chip" :class="{ 'substrand-chip--active': activeScope === 'stage' }" :disabled="!activeLevelKey" @click="selectScope('stage')">Selected stage only</button>
              </div>
              <div v-if="levelGroups.length" class="level-grid">
                <button v-for="level in levelGroups" :key="level.key" class="level-card" :class="{ 'level-card--active': activeLevelKey === level.key && activeScope === 'stage' }" :aria-pressed="activeLevelKey === level.key && activeScope === 'stage'" @click="selectLevel(level.key)">
                  <div class="flex items-center justify-between gap-3"><p class="font-semibold">{{ level.label }}</p><AppBadge color="muted" size="xs">{{ level.total }} nodes</AppBadge></div>
                  <div class="mt-2"><AppProgress :value="level.examReady" :max="Math.max(level.total, 1)" size="sm" color="success" /></div>
                  <div class="mt-3 flex flex-wrap gap-2"><AppBadge color="danger" size="xs">Blocked {{ level.blocked }}</AppBadge><AppBadge color="warm" size="xs">Review {{ level.reviewDue }}</AppBadge><AppBadge color="success" size="xs">Ready {{ level.examReady }}</AppBadge></div>
                </button>
              </div>
              <p v-else class="empty-copy">No stage data was returned for this subject.</p>
            </AppCard>

            <AppCard padding="lg">
              <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
                <div><p class="section-label">Step 3. Structure navigation</p><h3 class="section-title">Choose categories and subcategories</h3></div>
                <AppBadge color="accent" size="sm">{{ activeTypeBreakdown.map((item) => `${item.label}: ${item.total}`).join(' | ') || 'No structure yet' }}</AppBadge>
              </div>
              <div v-if="primaryNodes.length" class="space-y-3">
                <div class="strand-grid">
                  <button v-for="node in primaryNodes" :key="node.node.id" class="strand-card" :class="{ 'strand-card--active': activeStrandId === node.node.id }" :aria-pressed="activeStrandId === node.node.id" @click="selectStrand(node.node.id)">
                    <div class="flex items-center justify-between gap-3"><p class="strand-title">{{ node.node.public_title }}</p><AppBadge :color="badgeToneForStatus(node.status_label)" size="xs">{{ node.status_label }}</AppBadge></div>
                    <p class="strand-reason">{{ node.reason }}</p>
                  </button>
                </div>
                <div v-if="selectedPrimaryNode" class="rounded-[var(--radius-lg)] p-3" :style="{ background: 'color-mix(in srgb, var(--card-bg) 90%, var(--paper))' }">
                  <div class="mb-2 flex items-center justify-between gap-3"><p class="section-label">Subcategories under {{ selectedPrimaryNode.node.public_title }}</p><AppButton variant="ghost" size="sm" :disabled="activeSubStrandId == null" @click="selectSubStrand(null)">Show full branch</AppButton></div>
                  <div v-if="secondaryNodes.length" class="substrand-row">
                    <button v-for="sub in secondaryNodes" :key="sub.node.id" class="substrand-chip" :class="{ 'substrand-chip--active': activeSubStrandId === sub.node.id }" :aria-pressed="activeSubStrandId === sub.node.id" @click="selectSubStrand(sub.node.id)">{{ sub.node.public_title }}</button>
                  </div>
                  <p v-else class="empty-copy">No explicit subcategories were defined under this branch.</p>
                </div>
              </div>
              <p v-else class="empty-copy">No structure hierarchy is available for this scope.</p>
            </AppCard>

            <AppCard padding="lg">
              <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
                <div><p class="section-label">Step 4. Learning nodes</p><h3 class="section-title">Select a node to see what it includes</h3></div>
                <AppBadge color="muted" size="sm">{{ filteredActionableNodes.length }} / {{ actionableNodes.length }} nodes</AppBadge>
              </div>
              <div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-end"><div class="flex-1"><AppSearchInput v-model="searchQuery" placeholder="Search node title, summary, or rationale..." /></div><AppButton variant="secondary" size="sm" :disabled="!searchQuery" @click="clearSearch">Clear search</AppButton></div>
              <div v-if="mapError" class="mb-4 rounded-[var(--radius-lg)] border px-4 py-3 text-sm" :style="{ backgroundColor: 'rgba(245, 158, 11, 0.08)', borderColor: 'rgba(245, 158, 11, 0.16)', color: 'var(--warm)' }">{{ mapError }}</div>
              <div v-if="loadingMap" class="space-y-3"><AppSkeleton v-for="i in 4" :key="`topic-skeleton-${i}`" height="86px" /></div>
              <div v-else-if="!filteredActionableNodes.length" class="empty-panel"><p class="empty-copy">No learning nodes match your current structure and search.</p></div>
              <TransitionGroup v-else name="topic-list" tag="div" class="topic-list">
                <button v-for="topic in filteredActionableNodes" :key="topic.node.id" class="topic-card" :class="{ 'topic-card--active': selectedTopicState?.node.id === topic.node.id }" :aria-selected="selectedTopicState?.node.id === topic.node.id" @click="selectTopic(topic.node.id)">
                  <div class="flex items-start justify-between gap-3">
                    <div class="text-left"><div class="mb-1 flex flex-wrap items-center gap-2"><p class="topic-title">{{ topic.node.public_title }}</p><AppBadge color="muted" size="xs">{{ formatNodeType(topic.node.node_type) }}</AppBadge><AppBadge color="muted" size="xs">{{ levelLabel(topic.node.level_id) }}</AppBadge></div><p class="topic-reason">{{ topic.reason }}</p></div>
                    <AppBadge :color="badgeToneForStatus(topic.status_label)" size="xs">{{ topic.status_label }}</AppBadge>
                  </div>
                  <div class="mt-3 flex flex-wrap gap-2"><AppBadge v-if="topic.blocked" color="danger" size="xs">Blocked</AppBadge><AppBadge v-if="topic.review_due" color="warm" size="xs">Review due</AppBadge><AppBadge v-if="topic.exam_ready" color="success" size="xs">Exam ready</AppBadge></div>
                </button>
              </TransitionGroup>
            </AppCard>
          </div>
          <div class="detail-column">
            <AppCard padding="lg" class="detail-card">
              <Transition name="panel-swap" mode="out-in">
                <div v-if="!loadingMap && selectedTopicState" :key="selectedTopicState.node.id" class="space-y-4">
                  <div class="flex flex-wrap items-center gap-2"><AppBadge color="accent" size="sm">{{ formatNodeType(selectedTopicState.node.node_type) }}</AppBadge><AppBadge color="muted" size="sm">{{ levelLabel(selectedTopicState.node.level_id) }}</AppBadge><AppBadge :color="badgeToneForStatus(selectedTopicState.status_label)" size="sm">{{ selectedTopicState.status_label }}</AppBadge></div>
                  <div class="path-row"><span v-for="pathNode in selectedTopicPath" :key="pathNode.node.id" class="path-chip">{{ pathNode.node.public_title }}</span></div>
                  <div><h3 class="detail-title">{{ selectedTopicState.node.public_title }}</h3><p class="detail-subtitle">{{ selectedTopicState.node.canonical_title }}</p></div>
                  <div class="detail-summary"><p>{{ selectedTopicState.node.public_summary || selectedTopicState.node.official_text || selectedTopicState.reason }}</p></div>
                  <div class="detail-stats">
                    <div class="stat-item"><p class="stat-label">Blocked</p><p class="stat-value">{{ selectedTopicState.blocked ? 'Yes' : 'No' }}</p></div>
                    <div class="stat-item"><p class="stat-label">Review due</p><p class="stat-value">{{ selectedTopicState.review_due ? 'Yes' : 'No' }}</p></div>
                    <div class="stat-item"><p class="stat-label">Exam ready</p><p class="stat-value">{{ selectedTopicState.exam_ready ? 'Yes' : 'No' }}</p></div>
                    <div class="stat-item"><p class="stat-label">Items under node</p><p class="stat-value">{{ selectedTopicChildren.length }}</p></div>
                  </div>
                  <div>
                    <p class="section-label">What is involved under this node</p>
                    <div v-if="selectedTopicChildren.length" class="mt-2 flex flex-wrap gap-2"><AppBadge v-for="child in selectedTopicChildren" :key="child.node.id" color="muted" size="sm">{{ child.node.public_title }}</AppBadge></div>
                    <div v-else-if="selectedTopicState.downstream_titles.length" class="mt-2 flex flex-wrap gap-2"><AppBadge v-for="item in selectedTopicState.downstream_titles.slice(0, 10)" :key="item" color="muted" size="sm">{{ item }}</AppBadge></div>
                    <p v-else class="empty-copy mt-2">No lower-level nodes were recorded under this node yet.</p>
                  </div>
                  <div v-if="recommendedTopics.length" class="space-y-2"><p class="section-label">Suggested next nodes</p><div class="flex flex-wrap gap-2"><AppBadge v-for="topic in recommendedTopics.slice(0, 6)" :key="topic.node_id" color="gold" size="sm">{{ topic.public_title }}</AppBadge></div></div>
                  <div class="flex flex-wrap gap-2"><AppButton size="sm" :disabled="selectedLaunchTopicId == null" @click="openTeachMode">Teach this node</AppButton><AppButton variant="secondary" size="sm" :disabled="selectedLaunchTopicId == null" @click="openPracticeMode">Practice this node</AppButton></div>
                </div>
                <div v-else-if="loadingMap" key="detail-loading" class="space-y-3"><AppSkeleton height="24px" /><AppSkeleton height="90px" /><AppSkeleton height="220px" /></div>
                <div v-else key="detail-empty" class="empty-panel"><p class="empty-copy">Choose a learning node from the list to inspect what is covered under it.</p></div>
              </Transition>
            </AppCard>
          </div>
        </section>
      </Transition>
    </template>
  </div>
</template>

<style scoped>
.curriculum-shell { min-height: 100%; }
.hero-card { position: relative; overflow: hidden; background: color-mix(in srgb, var(--card-bg) 90%, var(--paper)); box-shadow: 0 14px 30px -24px rgba(15, 23, 42, 0.55); }
.hero-card > * { position: relative; z-index: 1; }
.hero-grid { display: grid; grid-template-columns: 1.15fr 0.85fr; gap: 20px; }
.hero-title { color: var(--text); font-size: 1.6rem; line-height: 1.2; font-weight: 700; }
.hero-copy { color: var(--text-3); font-size: 0.92rem; line-height: 1.55; max-width: 48ch; }
.hero-metrics { display: grid; gap: 10px; }
.metric-box, .step-card, .level-card, .strand-card, .topic-card, .detail-summary, .stat-item { border-radius: var(--radius-lg); background: color-mix(in srgb, var(--card-bg) 92%, var(--paper)); box-shadow: 0 12px 26px -26px rgba(15, 23, 42, 0.72); }
.metric-box { padding: 12px; }
.metric-label, .section-label, .step-label, .stat-label { text-transform: uppercase; letter-spacing: 0.12em; font-size: 0.66rem; font-weight: 700; color: var(--text-3); }
.metric-value { margin-top: 6px; margin-bottom: 7px; color: var(--text); font-size: 1.25rem; line-height: 1; font-weight: 700; }
.section-meta { color: var(--text-3); font-size: 0.75rem; }
.section-title { color: var(--text); font-size: 1.2rem; line-height: 1.25; font-weight: 650; }
.subject-card { width: 100%; text-align: left; border-radius: var(--radius-lg); padding: 14px; border: none; background: color-mix(in srgb, var(--card-bg) 92%, var(--paper)); box-shadow: 0 12px 24px -24px rgba(15, 23, 42, 0.74); transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 180ms cubic-bezier(0.22, 1, 0.36, 1); }
.subject-card:hover { transform: translateY(-1px); }
.subject-card:focus-visible { outline: none; box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 38%, transparent); }
.subject-card--active { box-shadow: var(--shadow-glow-accent); background: color-mix(in srgb, var(--accent) 7%, var(--card-bg)); }
.subject-title { color: var(--text); font-size: 0.98rem; line-height: 1.3; font-weight: 620; }
.subject-subtitle { color: var(--text-3); font-size: 0.75rem; margin-top: 4px; }
.subject-progress { display: flex; justify-content: space-between; gap: 8px; font-size: 0.74rem; color: var(--text-3); }
.step-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
.step-card { padding: 12px; }
.step-value { color: var(--text); margin-top: 7px; font-size: 0.85rem; line-height: 1.35; font-weight: 600; }
.level-grid, .strand-grid { display: grid; gap: 10px; }
.level-grid { grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); }
.strand-grid { grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); }
.level-card, .strand-card, .topic-card { width: 100%; text-align: left; padding: 12px; border: none; transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 180ms ease, background-color 180ms ease; }
.level-card:hover, .strand-card:hover, .topic-card:hover { transform: translateY(-1px); }
.level-card:focus-visible, .strand-card:focus-visible, .topic-card:focus-visible { outline: none; box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 34%, transparent); }
.level-card--active, .strand-card--active, .topic-card--active { box-shadow: var(--shadow-glow-accent); background: color-mix(in srgb, var(--accent) 8%, var(--card-bg)); }
.strand-title, .topic-title { color: var(--text); font-weight: 620; line-height: 1.35; }
.strand-reason, .topic-reason { margin-top: 6px; color: var(--text-3); font-size: 0.8rem; line-height: 1.45; }
.substrand-row { display: flex; flex-wrap: wrap; gap: 8px; }
.substrand-chip { border-radius: 999px; border: none; padding: 7px 11px; font-size: 0.75rem; color: var(--text-2); background: color-mix(in srgb, var(--card-bg) 90%, var(--paper)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-soft) 64%, transparent); transition: background-color 120ms ease, color 120ms ease, box-shadow 120ms ease; }
.substrand-chip:hover { box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 40%, transparent); }
.substrand-chip:focus-visible { outline: none; box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 34%, transparent); }
.substrand-chip--active { color: var(--accent); background: color-mix(in srgb, var(--accent) 9%, var(--card-bg)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 48%, transparent); }
.topic-list { display: grid; gap: 10px; }
.detail-column { position: sticky; top: 14px; align-self: start; }
.detail-card { border: none; box-shadow: 0 14px 28px -26px rgba(15, 23, 42, 0.7); }
.path-row { display: flex; flex-wrap: wrap; gap: 6px; }
.path-chip { border-radius: 999px; padding: 3px 8px; font-size: 0.68rem; line-height: 1.2; color: var(--text-3); background: color-mix(in srgb, var(--border-soft) 64%, transparent); }
.detail-title { color: var(--text); font-size: 1.26rem; line-height: 1.2; font-weight: 700; }
.detail-subtitle { color: var(--text-3); font-size: 0.84rem; margin-top: 4px; }
.detail-summary { padding: 12px; color: var(--text-2); font-size: 0.86rem; line-height: 1.55; }
.detail-stats { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.stat-item { padding: 10px; }
.stat-value { color: var(--text); margin-top: 5px; font-size: 0.92rem; font-weight: 640; }
.empty-panel { border-radius: var(--radius-lg); padding: 22px 14px; text-align: center; background: color-mix(in srgb, var(--card-bg) 84%, var(--paper)); }
.empty-copy { color: var(--text-3); font-size: 0.85rem; line-height: 1.45; }
.panel-swap-enter-active, .panel-swap-leave-active { transition: opacity 180ms cubic-bezier(0.22, 1, 0.36, 1), transform 180ms cubic-bezier(0.22, 1, 0.36, 1); }
.panel-swap-enter-from, .panel-swap-leave-to { opacity: 0; transform: translateY(6px); }
.topic-list-enter-active, .topic-list-leave-active { transition: opacity 180ms cubic-bezier(0.22, 1, 0.36, 1), transform 180ms cubic-bezier(0.22, 1, 0.36, 1); }
.topic-list-enter-from, .topic-list-leave-to { opacity: 0; transform: translateY(8px); }
@media (max-width: 1279px) { .detail-column { position: static; } }
@media (max-width: 1023px) { .hero-grid { grid-template-columns: 1fr; } .step-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } .detail-stats { grid-template-columns: 1fr; } }
@media (max-width: 640px) { .step-grid { grid-template-columns: 1fr; } .hero-title { font-size: 1.4rem; } }
@media (prefers-reduced-motion: reduce) { .subject-card, .level-card, .strand-card, .topic-card, .substrand-chip, .panel-swap-enter-active, .panel-swap-leave-active, .topic-list-enter-active, .topic-list-leave-active { transition-duration: 80ms !important; transform: none !important; } }
</style>
