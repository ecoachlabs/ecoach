<script setup lang="ts">
/**
 * Curriculum Journey — an interactive, zoomable path through the student's curriculum.
 *
 * Four altitudes (zoom levels) replace the old "dump-everything" dashboard:
 *   orbit  — subjects float as planets                       (fly in → stages)
 *   stages — stages ride a zig-zag trail                     (fly in → path)
 *   path   — topics sit on a Duolingo-style winding path     (fly in → node)
 *   node   — topic detail blooms; Teach / Practice launch here
 *
 * Keyboard: Esc/Backspace zoom out · ← → ↑ ↓ cycle siblings · Enter zoom in
 *           / focus search · r refresh.
 *
 * The IPC contract is unchanged: getStudentCurriculumHome + getStudentSubjectCurriculumMap.
 */

import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
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
import AppKbd from '@/components/ui/AppKbd.vue'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import { sanitizeLearnerSnippet, stripCurriculumCodes } from '@/utils/learnerCopy'

type Altitude = 'orbit' | 'stages' | 'path' | 'node'
type FlightDirection = 'in' | 'out'
type BadgeTone = 'accent' | 'warm' | 'gold' | 'success' | 'danger' | 'muted'
type NodeVibe = 'ready' | 'review' | 'progress' | 'fresh' | 'blocked'

interface StageSummary {
  key: string
  levelId: number | null
  label: string
  total: number
  ready: number
  review: number
  blocked: number
  entered: number
  readyPct: number
  topicCount: number
}

interface PathItemBanner {
  kind: 'banner'
  id: string
  y: number
  label: string
}
interface PathItemNode {
  kind: 'node'
  id: number
  y: number
  x: number
  state: CurriculumStudentNodeState
  vibe: NodeVibe
  index: number
}
type PathItem = PathItemBanner | PathItemNode

const auth = useAuthStore()
const router = useRouter()

// ── data ────────────────────────────────────────────────────────────────────
const loadingHome = ref(false)
const loadingMap = ref(false)
const homeError = ref('')
const mapError = ref('')
const home = ref<CurriculumStudentHomeSnapshot | null>(null)
const subjectMap = ref<CurriculumStudentSubjectMap | null>(null)

// ── journey state ───────────────────────────────────────────────────────────
const altitude = ref<Altitude>('orbit')
const flight = ref<FlightDirection>('in')
const selectedSubjectTrackId = ref<number | null>(null)
const selectedStageKey = ref<string | null>(null) // 'all' | level key
const selectedTopicId = ref<number | null>(null)
const hoveredSubjectId = ref<number | null>(null)
const hoveredStageKey = ref<string | null>(null)
const hoveredTopicId = ref<number | null>(null)
const searchQuery = ref('')
const pathViewportWidth = ref(960)
const reducedMotion = ref(false)

const pathContainer = ref<HTMLElement | null>(null)
const searchBox = ref<InstanceType<typeof AppSearchInput> | null>(null)

function learnerTitle(value: string | null | undefined): string {
  return stripCurriculumCodes(value)
}

function learnerSummary(value: string | null | undefined): string {
  return sanitizeLearnerSnippet(value, { dropSyllabusMeta: true })
}

const studentId = computed(() => auth.currentAccount?.id ?? null)
const subjectCards = computed(() => home.value?.subject_cards ?? [])
const selectedSubjectCard = computed(
  () =>
    subjectCards.value.find((item) => item.subject_track_id === selectedSubjectTrackId.value) ?? null,
)
const selectedSubjectOverview = computed(
  () => subjectMap.value?.overview ?? selectedSubjectCard.value,
)
const allNodeStates = computed(() => subjectMap.value?.nodes ?? [])
const recommendedTopics = computed(
  () => subjectMap.value?.recommended_topics ?? home.value?.recommended_topics ?? [],
)

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

// Every node that has no children and/or ties back to a legacy topic → it's a stop on the path.
const actionableNodes = computed(() => {
  return allNodeStates.value
    .filter((state) => {
      const children = childLookup.value.get(state.node.id) ?? []
      return state.node.legacy_topic_id != null || children.length === 0
    })
    .slice()
    .sort(sortNodeStates)
})

// ── stages grouping ─────────────────────────────────────────────────────────
const stages = computed<StageSummary[]>(() => {
  const map = new Map<string, StageSummary>()
  const push = (key: string, label: string, levelId: number | null) => {
    if (!map.has(key)) {
      map.set(key, {
        key,
        levelId,
        label,
        total: 0,
        ready: 0,
        review: 0,
        blocked: 0,
        entered: 0,
        readyPct: 0,
        topicCount: 0,
      })
    }
    return map.get(key)!
  }

  for (const state of allNodeStates.value) {
    const levelId = state.node.level_id
    const key = stageKey(levelId)
    const label = stageLabel(levelId)
    const row = push(key, label, levelId)
    row.total += 1
    if (state.exam_ready) row.ready += 1
    if (state.review_due) row.review += 1
    if (state.blocked) row.blocked += 1
    if (!state.blocked && !state.review_due) row.entered += 1
  }

  for (const state of actionableNodes.value) {
    const row = map.get(stageKey(state.node.level_id))
    if (row) row.topicCount += 1
  }

  const list = [...map.values()].sort((left, right) => {
    const l = left.levelId ?? 9999
    const r = right.levelId ?? 9999
    return l - r || left.label.localeCompare(right.label)
  })
  for (const stage of list) {
    stage.readyPct = stage.total ? Math.round((stage.ready / stage.total) * 100) : 0
  }
  return list
})

const activeStage = computed(() => {
  if (!selectedStageKey.value) return null
  return stages.value.find((s) => s.key === selectedStageKey.value) ?? null
})

// ── topics on the journey path ──────────────────────────────────────────────
const stageActionableNodes = computed(() => {
  if (!selectedStageKey.value) return []
  if (selectedStageKey.value === 'all') return actionableNodes.value
  return actionableNodes.value.filter(
    (state) => stageKey(state.node.level_id) === selectedStageKey.value,
  )
})

const filteredActionableNodes = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return stageActionableNodes.value
  return stageActionableNodes.value.filter((item) => {
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

function ancestorAtDepth(state: CurriculumStudentNodeState, depth: number): CurriculumStudentNodeState {
  let cursor: CurriculumStudentNodeState | undefined = state
  while (cursor && cursor.node.depth > depth) {
    const parentId = cursor.node.parent_node_id
    if (parentId == null) break
    const next = nodeById.value.get(parentId)
    if (!next) break
    cursor = next
  }
  return cursor ?? state
}

function strandFor(state: CurriculumStudentNodeState): CurriculumStudentNodeState {
  // Strand = the second-highest ancestor under the subject root (depth 1 when root is 0).
  // Falls back to whatever the shallowest ancestor is.
  return ancestorAtDepth(state, Math.min(1, state.node.depth))
}

const pathLayout = computed(() => {
  const list = filteredActionableNodes.value
  const items: PathItem[] = []
  if (!list.length) return { items, totalHeight: 360 }

  const width = Math.max(480, pathViewportWidth.value)
  const amplitude = Math.min(width * 0.28, 180)
  const centerX = width / 2
  const SPACING = 128
  const BANNER = 68
  const TOP_PAD = 48

  let y = TOP_PAD
  let lastStrandId: number | null = null
  let nodeIdx = 0

  for (const state of list) {
    const strand = strandFor(state)
    if (strand.node.id !== lastStrandId) {
      items.push({
        kind: 'banner',
        id: `banner-${strand.node.id}-${nodeIdx}`,
        y,
        label: learnerTitle(strand.node.public_title),
      })
      y += BANNER
      lastStrandId = strand.node.id
    }
    const cx = centerX + Math.sin(nodeIdx * 0.9) * amplitude
    items.push({
      kind: 'node',
      id: state.node.id,
      y,
      x: cx,
      state,
      vibe: vibeFor(state),
      index: nodeIdx,
    })
    y += SPACING
    nodeIdx += 1
  }

  return { items, totalHeight: y + TOP_PAD }
})

const pathNodeItems = computed(() =>
  pathLayout.value.items.filter((item): item is PathItemNode => item.kind === 'node'),
)

const pathSvgD = computed(() => {
  const nodes = pathNodeItems.value
  if (nodes.length < 2) return ''
  let d = ''
  nodes.forEach((point, i) => {
    if (i === 0) {
      d += `M ${point.x.toFixed(2)} ${point.y.toFixed(2)}`
      return
    }
    const prev = nodes[i - 1]
    const midY = (prev.y + point.y) / 2
    d += ` C ${prev.x.toFixed(2)} ${midY.toFixed(2)} ${point.x.toFixed(2)} ${midY.toFixed(2)} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`
  })
  return d
})

// ── node detail ─────────────────────────────────────────────────────────────
const selectedTopicState = computed(() => {
  if (selectedTopicId.value == null) return null
  return nodeById.value.get(selectedTopicId.value) ?? null
})

const selectedTopicChildren = computed(() => {
  const state = selectedTopicState.value
  if (!state) return []
  const childIds = childLookup.value.get(state.node.id) ?? []
  return childIds
    .map((id) => nodeById.value.get(id))
    .filter((item): item is CurriculumStudentNodeState => Boolean(item))
    .sort(sortNodeStates)
})

const selectedTopicPath = computed(() => {
  const state = selectedTopicState.value
  if (!state) return []
  const path: CurriculumStudentNodeState[] = []
  let cursor: CurriculumStudentNodeState | undefined = state
  while (cursor) {
    path.unshift(cursor)
    const parentId = cursor.node.parent_node_id
    if (parentId == null) break
    cursor = nodeById.value.get(parentId)
  }
  return path
})

const selectedLaunchTopicId = computed(
  () => selectedTopicState.value?.node.legacy_topic_id ?? null,
)

// ── breadcrumb ──────────────────────────────────────────────────────────────
interface Crumb {
  key: string
  label: string
  target: Altitude
  active?: boolean
}
const crumbs = computed<Crumb[]>(() => {
  const trail: Crumb[] = [
    { key: 'root', label: 'Curriculum', target: 'orbit', active: altitude.value === 'orbit' },
  ]
  if (selectedSubjectOverview.value) {
    trail.push({
      key: 'subject',
      label: learnerTitle(selectedSubjectOverview.value.public_title),
      target: 'stages',
      active: altitude.value === 'stages',
    })
  }
  if (activeStage.value) {
    trail.push({
      key: 'stage',
      label: activeStage.value.label,
      target: 'path',
      active: altitude.value === 'path',
    })
  }
  if (selectedTopicState.value) {
    trail.push({
      key: 'topic',
      label: learnerTitle(selectedTopicState.value.node.public_title),
      target: 'node',
      active: altitude.value === 'node',
    })
  }
  return trail
})

// ── loaders ─────────────────────────────────────────────────────────────────
onMounted(() => {
  reducedMotion.value =
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  void loadHome()
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('resize', measurePath)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('resize', measurePath)
})

watch(altitude, () => {
  void nextTick(() => measurePath())
})

watch(pathNodeItems, (items) => {
  if (!items.length) return
  if (selectedTopicId.value == null) return
  if (!items.some((item) => item.id === selectedTopicId.value)) {
    selectedTopicId.value = items[0]?.id ?? null
  }
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
  } catch (error) {
    homeError.value = extractError(error, 'Failed to load curriculum overview.')
    home.value = null
  } finally {
    loadingHome.value = false
  }
}

async function loadSubjectMap(subjectTrackId: number) {
  const sid = studentId.value
  if (sid == null) return
  loadingMap.value = true
  mapError.value = ''
  try {
    subjectMap.value = await getStudentSubjectCurriculumMap(sid, subjectTrackId)
  } catch (error) {
    mapError.value = extractError(error, 'Failed to load the selected subject curriculum.')
    subjectMap.value = null
  } finally {
    loadingMap.value = false
  }
}

// ── camera / flight controls ────────────────────────────────────────────────
async function enterSubject(subjectTrackId: number) {
  flight.value = 'in'
  selectedSubjectTrackId.value = subjectTrackId
  selectedStageKey.value = null
  selectedTopicId.value = null
  subjectMap.value = null
  altitude.value = 'stages'
  await loadSubjectMap(subjectTrackId)
  // If only one stage available, auto-advance so the student isn't stuck.
  if (stages.value.length === 1) {
    enterStage(stages.value[0].key)
  }
}

function enterStage(stageKey: string) {
  flight.value = 'in'
  selectedStageKey.value = stageKey
  selectedTopicId.value = null
  altitude.value = 'path'
}

function enterNode(topicId: number) {
  flight.value = 'in'
  selectedTopicId.value = topicId
  altitude.value = 'node'
}

function zoomOut() {
  flight.value = 'out'
  if (altitude.value === 'node') {
    altitude.value = 'path'
  } else if (altitude.value === 'path') {
    selectedTopicId.value = null
    altitude.value = 'stages'
  } else if (altitude.value === 'stages') {
    selectedStageKey.value = null
    selectedSubjectTrackId.value = null
    subjectMap.value = null
    altitude.value = 'orbit'
  }
}

function flyToCrumb(crumb: Crumb) {
  if (crumb.target === altitude.value) return
  flight.value = order(crumb.target) < order(altitude.value) ? 'out' : 'in'
  if (crumb.target === 'orbit') {
    selectedSubjectTrackId.value = null
    selectedStageKey.value = null
    selectedTopicId.value = null
    subjectMap.value = null
  } else if (crumb.target === 'stages') {
    selectedStageKey.value = null
    selectedTopicId.value = null
  } else if (crumb.target === 'path') {
    selectedTopicId.value = null
  }
  altitude.value = crumb.target
}

function order(level: Altitude): number {
  return level === 'orbit' ? 0 : level === 'stages' ? 1 : level === 'path' ? 2 : 3
}

function refreshCurriculum() {
  if (selectedSubjectTrackId.value != null) {
    void loadSubjectMap(selectedSubjectTrackId.value)
  }
  void loadHome()
}

// ── keyboard ────────────────────────────────────────────────────────────────
function onKeydown(event: KeyboardEvent) {
  const tag = (event.target as HTMLElement | null)?.tagName?.toLowerCase()
  const typing = tag === 'input' || tag === 'textarea' || tag === 'select'
  if (typing && event.key !== 'Escape') return

  if (event.key === 'Escape' || event.key === 'Backspace') {
    if (altitude.value !== 'orbit') {
      event.preventDefault()
      zoomOut()
    }
    return
  }
  if (event.key === '/') {
    if (altitude.value === 'path') {
      event.preventDefault()
      focusSearch()
    }
    return
  }
  if (event.key.toLowerCase() === 'r') {
    event.preventDefault()
    refreshCurriculum()
    return
  }
  if (event.key === 'Enter') {
    if (altitude.value === 'orbit' && hoveredSubjectId.value != null) {
      void enterSubject(hoveredSubjectId.value)
    } else if (altitude.value === 'stages' && hoveredStageKey.value) {
      enterStage(hoveredStageKey.value)
    } else if (altitude.value === 'path' && hoveredTopicId.value != null) {
      enterNode(hoveredTopicId.value)
    }
    return
  }

  const horizontal = event.key === 'ArrowLeft' || event.key === 'ArrowRight'
  const vertical = event.key === 'ArrowUp' || event.key === 'ArrowDown'
  if (!horizontal && !vertical) return

  if (altitude.value === 'orbit') cycleSubjectHover(event.key)
  else if (altitude.value === 'stages') cycleStageHover(event.key)
  else if (altitude.value === 'path') cycleTopicHover(event.key)
}

function cycleSubjectHover(key: string) {
  const items = subjectCards.value
  if (!items.length) return
  const idx = items.findIndex((s) => s.subject_track_id === hoveredSubjectId.value)
  const step = key === 'ArrowRight' || key === 'ArrowDown' ? 1 : -1
  const next = (idx < 0 ? 0 : (idx + step + items.length) % items.length)
  hoveredSubjectId.value = items[next].subject_track_id
}

function cycleStageHover(key: string) {
  const items = stages.value
  if (!items.length) return
  const idx = items.findIndex((s) => s.key === hoveredStageKey.value)
  const step = key === 'ArrowRight' || key === 'ArrowDown' ? 1 : -1
  const next = (idx < 0 ? 0 : (idx + step + items.length) % items.length)
  hoveredStageKey.value = items[next].key
}

function cycleTopicHover(key: string) {
  const items = pathNodeItems.value
  if (!items.length) return
  const idx = items.findIndex((s) => s.id === hoveredTopicId.value)
  const step = key === 'ArrowRight' || key === 'ArrowDown' ? 1 : -1
  const next = (idx < 0 ? 0 : (idx + step + items.length) % items.length)
  hoveredTopicId.value = items[next].id
}

// ── launch into teach / practice ────────────────────────────────────────────
function openTeachMode() {
  if (selectedLaunchTopicId.value == null) return
  void router.push({ name: 'teach', params: { topicId: selectedLaunchTopicId.value } })
}
function openPracticeMode() {
  if (selectedLaunchTopicId.value == null) return
  void router.push({
    path: '/student/practice',
    query: { topicId: String(selectedLaunchTopicId.value), from: 'curriculum' },
  })
}

// ── helpers ─────────────────────────────────────────────────────────────────
function measurePath() {
  const el = pathContainer.value
  if (el) pathViewportWidth.value = el.clientWidth
}

function stageKey(levelId: number | null) {
  return levelId == null ? 'stage-unassigned' : `stage-${levelId}`
}
function stageLabel(levelId: number | null) {
  return levelId == null ? 'Unassigned stage' : `Stage ${levelId}`
}

function vibeFor(state: CurriculumStudentNodeState): NodeVibe {
  if (state.blocked) return 'blocked'
  if (state.exam_ready) return 'ready'
  if (state.review_due) return 'review'
  const label = state.status_label.toLowerCase()
  if (label.includes('not started') || label.includes('new')) return 'fresh'
  return 'progress'
}

function vibeGlyph(vibe: NodeVibe): string {
  switch (vibe) {
    case 'ready': return '✓'
    case 'review': return '↻'
    case 'progress': return '◆'
    case 'blocked': return '🔒'
    default: return '○'
  }
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

function sortNodeStates(left: CurriculumStudentNodeState, right: CurriculumStudentNodeState) {
  return (
    left.node.sequence_no - right.node.sequence_no ||
    left.node.public_title.localeCompare(right.node.public_title)
  )
}

function extractError(error: unknown, fallback: string) {
  if (typeof error === 'string') return error
  if (error && typeof error === 'object' && 'message' in error && typeof (error as { message?: unknown }).message === 'string') {
    return (error as { message: string }).message
  }
  return fallback
}

function clearSearch() {
  searchQuery.value = ''
}

function planetStyle(card: CurriculumStudentSubjectCardDto): Record<string, string> {
  const hue = (card.subject_track_id * 47) % 360
  return {
    background: `radial-gradient(circle at 30% 30%, hsl(${hue} 70% 92%) 0%, var(--card-bg) 70%)`,
    '--planet-accent': `hsl(${hue} 62% 48%)`,
  }
}

function ringStyle(ready: number): Record<string, string> {
  const pct = Math.max(0, Math.min(100, ready))
  return {
    background: `conic-gradient(var(--success) ${pct}%, color-mix(in srgb, var(--border-soft) 64%, transparent) ${pct}% 100%)`,
  }
}

function focusSearch() {
  const host = (searchBox.value as unknown as { $el?: HTMLElement } | null)?.$el
  const input = host?.querySelector?.('input') as HTMLInputElement | null | undefined
  input?.focus()
}
</script>

<template>
  <div
    class="curriculum-cosmos"
    :class="{ 'cosmos--reduced': reducedMotion, [`cosmos--${altitude}`]: true }"
    tabindex="-1"
  >
    <PageHeader
      title="Curriculum Journey"
      subtitle="Your path through the curriculum — zoom in to a subject, travel the stages, and open any node."
    >
      <template #actions>
        <div class="flex flex-wrap items-center gap-2">
          <AppButton
            v-if="altitude !== 'orbit'"
            variant="ghost"
            size="sm"
            @click="zoomOut"
            :aria-label="'Zoom out of ' + altitude"
          >
            ← Zoom out
          </AppButton>
          <AppButton
            variant="secondary"
            size="sm"
            :loading="loadingHome || loadingMap"
            @click="refreshCurriculum"
          >
            Refresh
          </AppButton>
          <AppBadge v-if="home?.curriculum_version" color="accent" size="sm">
            {{ home.curriculum_version.version_label }}
          </AppBadge>
        </div>
      </template>
    </PageHeader>

    <!-- breadcrumb / flight-path -->
    <nav class="flight-path" aria-label="Curriculum location">
      <template v-for="(crumb, i) in crumbs" :key="crumb.key">
        <button
          type="button"
          class="flight-chip"
          :class="{ 'flight-chip--active': crumb.active }"
          :aria-current="crumb.active ? 'page' : undefined"
          @click="flyToCrumb(crumb)"
        >
          {{ crumb.label }}
        </button>
        <span v-if="i < crumbs.length - 1" class="flight-sep" aria-hidden="true">›</span>
      </template>
    </nav>

    <div
      v-if="homeError"
      class="banner-error"
      role="alert"
    >{{ homeError }}</div>

    <!-- stage area -->
    <div class="stage-area">
      <Transition :name="reducedMotion ? 'fade' : flight === 'in' ? 'zoom-in' : 'zoom-out'" mode="out-in">
        <!-- ── ORBIT: subject selection ────────────────────────────────── -->
        <section v-if="altitude === 'orbit'" key="orbit" class="layer layer-orbit">
          <div v-if="loadingHome && !home" class="orbit-skeleton">
            <AppSkeleton v-for="i in 4" :key="`skeleton-${i}`" height="220px" />
          </div>
          <template v-else-if="subjectCards.length">
            <header class="layer-caption">
              <p class="caption-label">Altitude 1 · Subjects</p>
              <h2 class="caption-title">Pick a subject to fly into</h2>
              <p class="caption-hint">{{ subjectCards.length }} subject{{ subjectCards.length === 1 ? '' : 's' }} · hover or arrow keys to preview</p>
            </header>
            <div class="orbit-grid">
              <button
                v-for="card in subjectCards"
                :key="card.subject_track_id"
                class="planet"
                :class="{ 'planet--focus': hoveredSubjectId === card.subject_track_id }"
                :style="planetStyle(card)"
                :aria-pressed="selectedSubjectTrackId === card.subject_track_id"
                @mouseenter="hoveredSubjectId = card.subject_track_id"
                @focus="hoveredSubjectId = card.subject_track_id"
                @click="enterSubject(card.subject_track_id)"
              >
                <div class="planet-ring" :style="ringStyle(card.exam_ready_percent)" />
                <div class="planet-core">
                  <p class="planet-title">{{ learnerTitle(card.public_title) }}</p>
                  <p class="planet-ready">{{ card.exam_ready_percent }}<span class="pct">%</span></p>
                  <p class="planet-subtitle">exam ready</p>
                </div>
                <div class="planet-stats">
                  <AppBadge color="success" size="xs">Stable {{ card.stable_percent }}%</AppBadge>
                  <AppBadge color="warm" size="xs">Review {{ card.review_due_count }}</AppBadge>
                  <AppBadge color="danger" size="xs">Blocked {{ card.blocked_count }}</AppBadge>
                </div>
                <p class="planet-trend">{{ card.trend_label }}</p>
              </button>
            </div>
            <AppCard v-if="home?.position_statement" padding="md" class="orbit-note">
              <p class="note-copy">{{ home.position_statement }}</p>
            </AppCard>
          </template>
          <AppCard v-else padding="lg" class="orbit-empty">
            <p class="empty-copy">No subject curriculum is available yet for this account.</p>
          </AppCard>
        </section>

        <!-- ── STAGES: zig-zag trail ───────────────────────────────────── -->
        <section v-else-if="altitude === 'stages'" key="stages" class="layer layer-stages">
          <header class="layer-caption">
            <p class="caption-label">Altitude 2 · Stages</p>
            <h2 class="caption-title">
                <span v-if="selectedSubjectOverview">{{ learnerTitle(selectedSubjectOverview.public_title) }}</span>
              <span v-else>Loading subject…</span>
            </h2>
            <p class="caption-hint">Follow the trail — each checkpoint is a stage of this subject.</p>
          </header>

          <div v-if="loadingMap && !stages.length" class="stage-skeleton">
            <AppSkeleton v-for="i in 4" :key="`stage-skel-${i}`" height="132px" />
          </div>
          <div v-else-if="stages.length" class="zigzag">
            <button
              v-if="stages.length > 1"
              class="checkpoint checkpoint--all"
              :class="{ 'checkpoint--focus': hoveredStageKey === 'all' }"
              @mouseenter="hoveredStageKey = 'all'"
              @focus="hoveredStageKey = 'all'"
              @click="enterStage('all')"
            >
              <div class="checkpoint-dot">★</div>
              <div class="checkpoint-body">
                <p class="checkpoint-label">Whole journey</p>
                <p class="checkpoint-meta">All stages on one path</p>
              </div>
            </button>
            <button
              v-for="(stage, index) in stages"
              :key="stage.key"
              class="checkpoint"
              :class="[
                `checkpoint--${index % 2 === 0 ? 'left' : 'right'}`,
                { 'checkpoint--focus': hoveredStageKey === stage.key },
              ]"
              @mouseenter="hoveredStageKey = stage.key"
              @focus="hoveredStageKey = stage.key"
              @click="enterStage(stage.key)"
            >
              <div
                class="checkpoint-dot"
                :style="{ background: `conic-gradient(var(--success) ${stage.readyPct}%, var(--border-soft) ${stage.readyPct}% 100%)` }"
              >
                <span class="checkpoint-dot-inner">{{ stage.readyPct }}%</span>
              </div>
              <div class="checkpoint-body">
                <p class="checkpoint-label">{{ stage.label }}</p>
                <p class="checkpoint-meta">{{ stage.topicCount }} node{{ stage.topicCount === 1 ? '' : 's' }} to travel</p>
                <div class="checkpoint-chips">
                  <AppBadge color="success" size="xs">Ready {{ stage.ready }}</AppBadge>
                  <AppBadge color="warm" size="xs">Review {{ stage.review }}</AppBadge>
                  <AppBadge v-if="stage.blocked" color="danger" size="xs">Blocked {{ stage.blocked }}</AppBadge>
                </div>
              </div>
            </button>
          </div>
          <AppCard v-else-if="mapError" padding="lg" class="orbit-empty">
            <p class="empty-copy">{{ mapError }}</p>
          </AppCard>
          <AppCard v-else padding="lg" class="orbit-empty">
            <p class="empty-copy">No stages are defined for this subject yet.</p>
          </AppCard>
        </section>

        <!-- ── PATH: Duolingo-style winding journey ────────────────────── -->
        <section v-else-if="altitude === 'path'" key="path" class="layer layer-path">
          <header class="layer-caption">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p class="caption-label">Altitude 3 · Journey path</p>
                <h2 class="caption-title">{{ activeStage?.label ?? 'Whole journey' }}</h2>
                <p class="caption-hint">{{ filteredActionableNodes.length }} stop{{ filteredActionableNodes.length === 1 ? '' : 's' }} on the path — tap any node to open it.</p>
              </div>
              <div class="path-search">
                <AppSearchInput
                  ref="searchBox"
                  v-model="searchQuery"
                  placeholder="Filter nodes…"
                />
                <AppButton v-if="searchQuery" variant="ghost" size="sm" @click="clearSearch">Clear</AppButton>
              </div>
            </div>
            <div class="path-legend">
              <span class="legend-item legend-ready"><i /> Exam ready</span>
              <span class="legend-item legend-progress"><i /> In progress</span>
              <span class="legend-item legend-review"><i /> Review due</span>
              <span class="legend-item legend-fresh"><i /> Not started</span>
              <span class="legend-item legend-blocked"><i /> Blocked</span>
            </div>
          </header>

          <div ref="pathContainer" class="path-container">
            <div v-if="loadingMap" class="path-skeleton">
              <AppSkeleton v-for="i in 6" :key="`path-skel-${i}`" height="96px" />
            </div>
            <div
              v-else-if="pathLayout.items.length"
              class="path-canvas"
              :style="{ height: pathLayout.totalHeight + 'px' }"
            >
              <svg
                class="path-line"
                :width="pathViewportWidth"
                :height="pathLayout.totalHeight"
                :viewBox="`0 0 ${pathViewportWidth} ${pathLayout.totalHeight}`"
                aria-hidden="true"
              >
                <path
                  v-if="pathSvgD"
                  :d="pathSvgD"
                  fill="none"
                  stroke="url(#pathGradient)"
                  stroke-width="8"
                  stroke-linecap="round"
                  stroke-dasharray="2 14"
                />
                <defs>
                  <linearGradient id="pathGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.55" />
                    <stop offset="100%" stop-color="var(--gold)" stop-opacity="0.45" />
                  </linearGradient>
                </defs>
              </svg>

              <template v-for="item in pathLayout.items" :key="`${item.kind}-${(item as any).id}`">
                <div
                  v-if="item.kind === 'banner'"
                  class="strand-banner"
                  :style="{ top: item.y + 'px' }"
                >
                  <span class="strand-banner-line" aria-hidden="true" />
                  <span class="strand-banner-label">{{ item.label }}</span>
                  <span class="strand-banner-line" aria-hidden="true" />
                </div>
                <button
                  v-else
                  class="path-node"
                  :class="[
                    `path-node--${item.vibe}`,
                    { 'path-node--focus': hoveredTopicId === item.id },
                  ]"
                  :style="{ top: item.y + 'px', left: item.x + 'px' }"
                  :aria-label="`${learnerTitle(item.state.node.public_title)} — ${item.state.status_label}`"
                  @mouseenter="hoveredTopicId = item.id"
                  @focus="hoveredTopicId = item.id"
                  @click="enterNode(item.id)"
                >
                  <span class="path-node-glyph" aria-hidden="true">{{ vibeGlyph(item.vibe) }}</span>
                  <span class="path-node-title">{{ learnerTitle(item.state.node.public_title) }}</span>
                  <span class="path-node-meta">{{ formatNodeType(item.state.node.node_type) }}</span>
                </button>
              </template>
            </div>
            <AppCard v-else padding="lg" class="orbit-empty">
              <p class="empty-copy">No path nodes match the current filter.</p>
            </AppCard>
          </div>
        </section>

        <!-- ── NODE: detail bloom ──────────────────────────────────────── -->
        <section v-else-if="altitude === 'node'" key="node" class="layer layer-node">
          <div v-if="selectedTopicState" class="node-bloom">
            <header class="layer-caption">
              <p class="caption-label">Altitude 4 · Node</p>
              <div class="bloom-pill-row">
                <AppBadge color="accent" size="sm">{{ formatNodeType(selectedTopicState.node.node_type) }}</AppBadge>
                <AppBadge color="muted" size="sm">{{ stageLabel(selectedTopicState.node.level_id) }}</AppBadge>
                <AppBadge :color="badgeToneForStatus(selectedTopicState.status_label)" size="sm">
                  {{ selectedTopicState.status_label }}
                </AppBadge>
              </div>
              <h2 class="bloom-title">{{ learnerTitle(selectedTopicState.node.public_title) }}</h2>
              <p
                v-if="selectedTopicState.node.canonical_title !== selectedTopicState.node.public_title"
                class="bloom-canonical"
              >{{ learnerTitle(selectedTopicState.node.canonical_title) }}</p>
            </header>

            <div class="bloom-grid">
              <AppCard padding="lg" class="bloom-card bloom-card--summary">
                <p class="bloom-section-label">Goal / Description</p>
                <p class="bloom-summary">
                  {{ learnerSummary(
                    selectedTopicState.node.public_summary
                    || selectedTopicState.node.official_text
                    || selectedTopicState.reason,
                  ) }}
                </p>

                <div class="bloom-stats">
                  <div class="bloom-stat">
                    <span class="bloom-stat-label">Blocked</span>
                    <span class="bloom-stat-value">{{ selectedTopicState.blocked ? 'Yes' : 'No' }}</span>
                  </div>
                  <div class="bloom-stat">
                    <span class="bloom-stat-label">Review due</span>
                    <span class="bloom-stat-value">{{ selectedTopicState.review_due ? 'Yes' : 'No' }}</span>
                  </div>
                  <div class="bloom-stat">
                    <span class="bloom-stat-label">Exam ready</span>
                    <span class="bloom-stat-value">{{ selectedTopicState.exam_ready ? 'Yes' : 'No' }}</span>
                  </div>
                  <div class="bloom-stat">
                    <span class="bloom-stat-label">Under this node</span>
                    <span class="bloom-stat-value">{{ selectedTopicChildren.length }}</span>
                  </div>
                </div>
              </AppCard>

              <AppCard padding="lg" class="bloom-card bloom-card--path">
                <p class="bloom-section-label">Route you took</p>
                <div class="bloom-path">
                  <span
                    v-for="crumb in selectedTopicPath"
                    :key="crumb.node.id"
                    class="bloom-path-step"
                  >{{ learnerTitle(crumb.node.public_title) }}</span>
                </div>

                <p class="bloom-section-label mt-5">What is involved</p>
                <div v-if="selectedTopicChildren.length" class="bloom-chip-row">
                  <AppBadge
                    v-for="child in selectedTopicChildren"
                    :key="child.node.id"
                    color="muted"
                    size="sm"
                  >{{ learnerTitle(child.node.public_title) }}</AppBadge>
                </div>
                <div
                  v-else-if="selectedTopicState.downstream_titles.length"
                  class="bloom-chip-row"
                >
                  <AppBadge
                    v-for="title in selectedTopicState.downstream_titles.slice(0, 12)"
                    :key="title"
                    color="muted"
                    size="sm"
                  >{{ title }}</AppBadge>
                </div>
                <p v-else class="empty-copy">No lower-level nodes were recorded yet.</p>

                <template v-if="recommendedTopics.length">
                  <p class="bloom-section-label mt-5">Suggested next</p>
                  <div class="bloom-chip-row">
                    <AppBadge
                      v-for="topic in recommendedTopics.slice(0, 6)"
                      :key="topic.node_id"
                      color="gold"
                      size="sm"
                    >{{ learnerTitle(topic.public_title) }}</AppBadge>
                  </div>
                </template>
              </AppCard>
            </div>

            <div class="bloom-actions">
              <AppButton
                variant="primary"
                size="lg"
                :disabled="selectedLaunchTopicId == null"
                @click="openTeachMode"
              >Teach this node</AppButton>
              <AppButton
                variant="secondary"
                size="lg"
                :disabled="selectedLaunchTopicId == null"
                @click="openPracticeMode"
              >Practice this node</AppButton>
              <AppButton variant="ghost" size="lg" @click="zoomOut">Back to path</AppButton>
            </div>

            <AppCard v-if="selectedTopicState.blocked" padding="md" class="bloom-alert">
              <p class="bloom-alert-title">This node is blocked</p>
              <p class="bloom-alert-body">{{ learnerSummary(selectedTopicState.reason || 'Finish the prerequisite nodes first.') }}</p>
            </AppCard>
          </div>
          <AppCard v-else padding="lg" class="orbit-empty">
            <p class="empty-copy">No node is selected.</p>
          </AppCard>
        </section>
      </Transition>
    </div>

    <!-- keyboard hint rail -->
    <footer class="cosmos-rail" aria-hidden="true">
      <span><AppKbd :keys="['Esc']" /> zoom out</span>
      <span><AppKbd :keys="['←','→','↑','↓']" /> move</span>
      <span><AppKbd :keys="['Enter']" /> zoom in</span>
      <span v-if="altitude === 'path'"><AppKbd :keys="['/']" /> search</span>
      <span><AppKbd :keys="['R']" /> refresh</span>
    </footer>
  </div>
</template>

<style scoped>
/* ───── canvas ────────────────────────────────────────────── */
.curriculum-cosmos {
  position: relative;
  min-height: 100%;
  padding: 24px clamp(16px, 3vw, 48px) 96px;
  max-width: 1440px;
  margin: 0 auto;
}

.cosmos--orbit { background: radial-gradient(ellipse at 20% 0%, color-mix(in srgb, var(--accent) 6%, transparent), transparent 60%); }
.cosmos--stages { background: radial-gradient(ellipse at 80% 10%, color-mix(in srgb, var(--gold) 7%, transparent), transparent 55%); }
.cosmos--path { background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 3%, transparent), transparent 40%); }
.cosmos--node { background: radial-gradient(ellipse at 50% 0%, color-mix(in srgb, var(--gold) 10%, transparent), transparent 55%); }

/* ───── breadcrumb ────────────────────────────────────────── */
.flight-path {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
  padding: 4px 0 18px;
}
.flight-chip {
  border: none;
  background: color-mix(in srgb, var(--card-bg) 92%, var(--paper));
  color: var(--text-2);
  border-radius: 999px;
  padding: 6px 12px;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-soft) 64%, transparent);
  transition: background-color 140ms ease, color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
}
.flight-chip:hover { color: var(--accent); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 42%, transparent); transform: translateY(-1px); }
.flight-chip--active { background: color-mix(in srgb, var(--accent) 10%, var(--card-bg)); color: var(--accent); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 54%, transparent); }
.flight-sep { color: var(--text-3); font-size: 0.9rem; padding: 0 2px; }

.banner-error {
  margin: 0 0 14px;
  padding: 10px 14px;
  border-radius: var(--radius-lg);
  background: rgba(220, 38, 38, 0.08);
  color: var(--danger);
  font-size: 0.85rem;
}

/* ───── stage area / layers ───────────────────────────────── */
.stage-area {
  position: relative;
  min-height: 420px;
}
.layer { position: relative; width: 100%; }
.layer-caption { margin-bottom: 22px; }
.caption-label {
  text-transform: uppercase;
  letter-spacing: 0.14em;
  font-size: 0.68rem;
  font-weight: 700;
  color: var(--text-3);
}
.caption-title {
  color: var(--text);
  font-size: 1.5rem;
  line-height: 1.2;
  font-weight: 700;
  margin-top: 4px;
}
.caption-hint {
  color: var(--text-3);
  font-size: 0.85rem;
  margin-top: 6px;
}

/* ───── orbit (subjects) ──────────────────────────────────── */
.orbit-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 20px;
  padding: 8px 0;
}
.orbit-skeleton {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
}
.planet {
  position: relative;
  border: none;
  border-radius: 28px;
  padding: 28px 24px 24px;
  min-height: 232px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  cursor: pointer;
  box-shadow: 0 16px 40px -28px rgba(15, 23, 42, 0.55);
  text-align: left;
  overflow: hidden;
  transition: transform 220ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 220ms ease;
  animation: planet-float 9s ease-in-out infinite;
}
.planet:nth-child(2n) { animation-duration: 11s; animation-delay: -2s; }
.planet:nth-child(3n) { animation-duration: 13s; animation-delay: -5s; }
.planet:hover, .planet--focus {
  transform: translateY(-4px) scale(1.015);
  box-shadow: 0 22px 50px -26px rgba(15, 23, 42, 0.65), 0 0 0 2px color-mix(in srgb, var(--planet-accent, var(--accent)) 38%, transparent);
}
.planet-ring {
  position: absolute;
  top: -28px;
  right: -28px;
  width: 128px;
  height: 128px;
  border-radius: 50%;
  filter: blur(0.2px);
  opacity: 0.85;
}
.planet-core { position: relative; z-index: 1; }
.planet-title { font-size: 1.12rem; font-weight: 700; color: var(--text); line-height: 1.25; }
.planet-ready { font-size: 2.4rem; font-weight: 800; color: var(--planet-accent, var(--accent)); line-height: 1; margin-top: 6px; }
.planet-ready .pct { font-size: 1rem; font-weight: 600; color: var(--text-3); margin-left: 2px; }
.planet-subtitle { color: var(--text-3); font-size: 0.76rem; margin-top: 2px; }
.planet-stats { display: flex; flex-wrap: wrap; gap: 6px; }
.planet-trend { color: var(--text-3); font-size: 0.78rem; font-style: italic; }
.orbit-note { margin-top: 20px; }
.note-copy { font-size: 0.88rem; color: var(--text-2); line-height: 1.55; }
.orbit-empty .empty-copy { color: var(--text-3); font-size: 0.9rem; }

@keyframes planet-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

/* ───── stages (zig-zag) ──────────────────────────────────── */
.zigzag {
  display: grid;
  gap: 18px;
  padding: 8px 0 40px;
  position: relative;
}
.stage-skeleton { display: grid; gap: 16px; padding: 8px 0; }
.checkpoint {
  position: relative;
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 20px;
  border: none;
  border-radius: 24px;
  background: color-mix(in srgb, var(--card-bg) 94%, var(--paper));
  box-shadow: 0 14px 30px -26px rgba(15, 23, 42, 0.55);
  cursor: pointer;
  text-align: left;
  width: min(640px, 100%);
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 180ms ease;
}
.checkpoint--right { margin-left: auto; }
.checkpoint--left { margin-right: auto; }
.checkpoint--all { margin: 0 auto; background: color-mix(in srgb, var(--gold) 8%, var(--card-bg)); }
.checkpoint:hover, .checkpoint--focus {
  transform: translateY(-2px);
  box-shadow: 0 20px 44px -24px rgba(15, 23, 42, 0.65), 0 0 0 2px color-mix(in srgb, var(--accent) 36%, transparent);
}
.checkpoint-dot {
  flex-shrink: 0;
  width: 72px;
  height: 72px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--text);
  background: var(--accent-light);
  box-shadow: inset 0 0 0 6px var(--card-bg);
  position: relative;
}
.checkpoint-dot-inner {
  background: var(--card-bg);
  border-radius: 50%;
  width: 52px;
  height: 52px;
  display: grid;
  place-items: center;
  color: var(--text);
  font-size: 0.82rem;
  font-weight: 700;
}
.checkpoint-body { display: grid; gap: 6px; }
.checkpoint-label { font-size: 1.02rem; font-weight: 700; color: var(--text); }
.checkpoint-meta { font-size: 0.8rem; color: var(--text-3); }
.checkpoint-chips { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 2px; }

/* ───── path (journey) ────────────────────────────────────── */
.path-search { display: flex; align-items: center; gap: 8px; min-width: 260px; }
.path-legend {
  margin-top: 14px;
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  font-size: 0.72rem;
  color: var(--text-3);
}
.legend-item { display: inline-flex; align-items: center; gap: 6px; }
.legend-item i {
  width: 10px; height: 10px; border-radius: 50%; display: inline-block;
}
.legend-ready i { background: var(--success); box-shadow: 0 0 8px var(--success); }
.legend-progress i { background: var(--accent); }
.legend-review i { background: var(--warm); }
.legend-fresh i { background: var(--border-soft); box-shadow: inset 0 0 0 1px var(--text-3); }
.legend-blocked i { background: var(--danger); opacity: 0.6; }

.path-container {
  position: relative;
  padding: 8px 0 56px;
}
.path-skeleton { display: grid; gap: 12px; }
.path-canvas {
  position: relative;
  width: 100%;
}
.path-line {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}
.strand-banner {
  position: absolute;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 24px;
  transform: translateY(-50%);
}
.strand-banner-line {
  flex: 1;
  height: 1px;
  background: color-mix(in srgb, var(--border-soft) 72%, transparent);
}
.strand-banner-label {
  text-transform: uppercase;
  letter-spacing: 0.16em;
  font-size: 0.72rem;
  font-weight: 700;
  color: var(--text-3);
  padding: 4px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--card-bg) 94%, var(--paper));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-soft) 70%, transparent);
}
.path-node {
  position: absolute;
  transform: translate(-50%, -50%);
  width: 168px;
  border: none;
  border-radius: 22px;
  padding: 14px 14px 16px;
  background: var(--card-bg);
  box-shadow: 0 16px 32px -22px rgba(15, 23, 42, 0.6);
  cursor: pointer;
  text-align: center;
  display: grid;
  gap: 4px;
  justify-items: center;
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 180ms ease, background-color 180ms ease;
}
.path-node:hover, .path-node--focus {
  transform: translate(-50%, -50%) scale(1.06);
  z-index: 2;
}
.path-node-glyph {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 1.2rem;
  font-weight: 700;
  color: var(--card-bg);
  background: var(--text-3);
  margin-top: -28px;
  box-shadow: 0 6px 14px -8px rgba(15, 23, 42, 0.55), inset 0 -4px 0 rgba(0, 0, 0, 0.12);
}
.path-node-title { font-size: 0.85rem; font-weight: 650; color: var(--text); line-height: 1.25; }
.path-node-meta { font-size: 0.66rem; letter-spacing: 0.12em; text-transform: uppercase; color: var(--text-3); }

.path-node--ready .path-node-glyph {
  background: var(--success);
  box-shadow: 0 0 20px color-mix(in srgb, var(--success) 55%, transparent), inset 0 -4px 0 rgba(0, 0, 0, 0.12);
  animation: ready-pulse 2.4s ease-in-out infinite;
}
.path-node--progress .path-node-glyph { background: var(--accent); box-shadow: 0 0 18px color-mix(in srgb, var(--accent) 50%, transparent), inset 0 -4px 0 rgba(0, 0, 0, 0.12); }
.path-node--review .path-node-glyph {
  background: var(--warm);
  animation: review-halo 2.2s ease-in-out infinite;
}
.path-node--blocked {
  opacity: 0.7;
}
.path-node--blocked .path-node-glyph { background: var(--text-3); color: var(--card-bg); }
.path-node--fresh .path-node-glyph {
  background: var(--card-bg);
  color: var(--text-3);
  box-shadow: inset 0 0 0 2px var(--border-soft), 0 6px 14px -8px rgba(15, 23, 42, 0.45);
}

@keyframes ready-pulse {
  0%, 100% { box-shadow: 0 0 20px color-mix(in srgb, var(--success) 55%, transparent), inset 0 -4px 0 rgba(0, 0, 0, 0.12); }
  50% { box-shadow: 0 0 30px color-mix(in srgb, var(--success) 75%, transparent), inset 0 -4px 0 rgba(0, 0, 0, 0.12); }
}
@keyframes review-halo {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--warm) 35%, transparent); }
  50% { box-shadow: 0 0 0 10px color-mix(in srgb, var(--warm) 0%, transparent); }
}

/* ───── node bloom (detail) ───────────────────────────────── */
.node-bloom {
  display: grid;
  gap: 20px;
}
.bloom-pill-row { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 6px; }
.bloom-title {
  font-size: clamp(1.6rem, 3vw, 2.2rem);
  font-weight: 800;
  color: var(--text);
  line-height: 1.15;
  margin-top: 12px;
}
.bloom-canonical { color: var(--text-3); font-size: 0.9rem; margin-top: 4px; }
.bloom-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: 1fr;
}
@media (min-width: 960px) {
  .bloom-grid { grid-template-columns: 1.2fr 1fr; }
}
.bloom-card { border: none; box-shadow: 0 16px 32px -26px rgba(15, 23, 42, 0.65); }
.bloom-section-label {
  text-transform: uppercase;
  letter-spacing: 0.14em;
  font-size: 0.68rem;
  font-weight: 700;
  color: var(--text-3);
  margin-bottom: 8px;
}
.mt-5 { margin-top: 20px; }
.bloom-summary { font-size: 0.96rem; line-height: 1.6; color: var(--text-2); }
.bloom-stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin-top: 18px;
}
.bloom-stat {
  padding: 12px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--card-bg) 90%, var(--paper));
  display: grid;
  gap: 4px;
}
.bloom-stat-label { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.12em; color: var(--text-3); font-weight: 700; }
.bloom-stat-value { color: var(--text); font-size: 1.02rem; font-weight: 700; }
.bloom-path { display: flex; flex-wrap: wrap; gap: 6px; }
.bloom-path-step {
  padding: 4px 10px;
  font-size: 0.72rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--border-soft) 64%, transparent);
  color: var(--text-3);
}
.bloom-chip-row { display: flex; flex-wrap: wrap; gap: 6px; }
.bloom-actions { display: flex; flex-wrap: wrap; gap: 10px; }
.bloom-alert { background: color-mix(in srgb, var(--danger) 8%, var(--card-bg)); border: 1px solid color-mix(in srgb, var(--danger) 22%, transparent); }
.bloom-alert-title { font-weight: 700; color: var(--danger); font-size: 0.9rem; }
.bloom-alert-body { color: var(--text-2); font-size: 0.85rem; margin-top: 2px; }

/* ───── keyboard rail ─────────────────────────────────────── */
.cosmos-rail {
  position: fixed;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: center;
  padding: 8px 14px;
  background: color-mix(in srgb, var(--card-bg) 94%, var(--paper));
  border-radius: 999px;
  box-shadow: 0 10px 22px -18px rgba(15, 23, 42, 0.6);
  font-size: 0.72rem;
  color: var(--text-3);
  z-index: 20;
}
.cosmos-rail span { display: inline-flex; align-items: center; gap: 6px; }

/* ───── zoom transitions ──────────────────────────────────── */
.zoom-in-enter-active, .zoom-in-leave-active,
.zoom-out-enter-active, .zoom-out-leave-active {
  transition: transform 340ms cubic-bezier(0.22, 1, 0.36, 1), opacity 280ms cubic-bezier(0.22, 1, 0.36, 1), filter 280ms ease;
}
.zoom-in-enter-from { opacity: 0; transform: scale(0.72); filter: blur(6px); }
.zoom-in-leave-to   { opacity: 0; transform: scale(1.4);  filter: blur(6px); }
.zoom-out-enter-from { opacity: 0; transform: scale(1.35); filter: blur(4px); }
.zoom-out-leave-to   { opacity: 0; transform: scale(0.72); filter: blur(4px); }
.fade-enter-active, .fade-leave-active { transition: opacity 160ms ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* ───── reduced motion ────────────────────────────────────── */
.cosmos--reduced .planet,
.cosmos--reduced .path-node {
  animation: none !important;
  transition-duration: 120ms !important;
}
@media (prefers-reduced-motion: reduce) {
  .planet, .path-node--ready .path-node-glyph, .path-node--review .path-node-glyph {
    animation: none !important;
  }
  .zoom-in-enter-active, .zoom-in-leave-active,
  .zoom-out-enter-active, .zoom-out-leave-active {
    transition-duration: 140ms !important;
  }
  .zoom-in-enter-from, .zoom-out-enter-from, .zoom-in-leave-to, .zoom-out-leave-to {
    transform: none !important; filter: none !important;
  }
}

/* ───── responsive tightening ─────────────────────────────── */
@media (max-width: 720px) {
  .curriculum-cosmos { padding: 18px 14px 120px; }
  .bloom-stats { grid-template-columns: 1fr; }
  .checkpoint { width: 100%; }
  .path-node { width: 144px; }
}
/* end of curriculum journey styles */
</style>
