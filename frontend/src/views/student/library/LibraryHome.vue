<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects } from '@/ipc/coach'
import {
  buildRevisionPack,
  getLibrarySnapshot,
  listLibraryItems,
  listRevisionPacks,
  searchLibrary,
  type LibraryHomeSnapshotDto,
  type LibrarySearchResultDto,
  type RevisionPackSummaryDto,
} from '@/ipc/library'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'

type ShelfTile = {
  key: string
  label: string
  symbol: string
  count: number
  desc: string
  to: string
}

type DisplayRow = {
  key: string
  title: string
  type: string
  subject: string
  route: string
  badge: string
}

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const loading = ref(true)
const buildingPack = ref(false)
const searchLoading = ref(false)
const error = ref('')
const searchQuery = ref('')
const snapshot = ref<LibraryHomeSnapshotDto | null>(null)
const trackedItemCount = ref(0)
const revisionPacks = ref<RevisionPackSummaryDto[]>([])
const searchResults = ref<LibrarySearchResultDto[]>([])
const subjectDirectory = ref<Record<number, string>>({})
let searchTimer: ReturnType<typeof setTimeout> | null = null

const continueCard = computed(() => snapshot.value?.continue_card ?? null)

const shelves = computed<ShelfTile[]>(() => {
  const currentSnapshot = snapshot.value
  if (!currentSnapshot) return []

  const generatedTiles = currentSnapshot.generated_shelves.slice(0, 5).map(shelf => ({
    key: `generated-${shelf.shelf_type}`,
    label: shelf.title,
    symbol: symbolForShelf(shelf.icon_hint ?? shelf.title),
    count: shelf.items.length,
    desc: shelf.description ?? 'Assembled from your current learning state',
    to: routeForShelf(shelf.shelf_type),
  }))

  return [
    {
      key: 'due-now',
      label: 'Due Now',
      symbol: 'DN',
      count: currentSnapshot.due_now_count,
      desc: 'Memory items ready right now',
      to: '/student/memory#reviews',
    },
    {
      key: 'pending-review',
      label: 'Pending Review',
      symbol: 'PR',
      count: currentSnapshot.pending_review_count,
      desc: 'Items waiting for another pass',
      to: '/student/memory',
    },
    {
      key: 'fading',
      label: 'Fading Concepts',
      symbol: 'FC',
      count: currentSnapshot.fading_concept_count,
      desc: 'Concepts slipping out of reach',
      to: '/student/memory',
    },
    {
      key: 'revision',
      label: 'Revision Packs',
      symbol: 'RP',
      count: revisionPacks.value.length,
      desc: 'Generated rescue packs ready to open',
      to: '/student/library#revision-box',
    },
    ...generatedTiles,
  ]
})

const recentRows = computed<DisplayRow[]>(() => {
  const rows: DisplayRow[] = []
  const seen = new Set<string>()

  for (const question of snapshot.value?.saved_questions ?? []) {
    const key = `saved-question-${question.library_item_id}`
    if (seen.has(key)) continue
    seen.add(key)
    rows.push({
      key,
      title: question.stem,
      type: 'question',
      subject: question.topic_name,
      route: practiceRouteForTopic(question.topic_id),
      badge: question.state,
    })
  }

  for (const shelf of snapshot.value?.generated_shelves ?? []) {
    for (const item of shelf.items) {
      const key = `shelf-item-${shelf.shelf_type}-${item.item_type}-${item.item_ref_id ?? item.title}`
      if (seen.has(key)) continue
      seen.add(key)
      rows.push({
        key,
        title: item.title,
        type: item.item_type,
        subject: item.subtitle ?? shelf.title,
        route: routeForLibraryItemType(item.item_type),
        badge: shelf.title,
      })
      if (rows.length >= 8) return rows
    }
  }

  for (const pack of revisionPacks.value) {
    const key = `pack-${pack.pack_id}`
    if (seen.has(key)) continue
    seen.add(key)
    rows.push({
      key,
      title: pack.title,
      type: 'revision_pack',
      subject: pack.difficulty_profile ?? 'Revision pack',
      route: '/student/library#revision-box',
      badge: pack.status ?? 'ready',
    })
    if (rows.length >= 8) return rows
  }

  return rows
})

const displayRows = computed<DisplayRow[]>(() => {
  const query = searchQuery.value.trim()
  if (!query) return recentRows.value

  return searchResults.value.map(result => ({
    key: `search-${result.item_type}-${result.item_ref_id ?? result.library_item_id ?? result.title}`,
    title: result.title,
    type: result.item_type,
    subject:
      result.subject_name ??
      result.topic_name ??
      result.subtitle ??
      subjectLabelForResult(result) ??
      'Library result',
    route: routeForLibrarySearchResult(result),
    badge: result.state ?? 'match',
  }))
})

const totalItems = computed(() => trackedItemCount.value + revisionPacks.value.length)

onMounted(() => {
  if (route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    })
  }

  void loadLibraryHome()
})

watch(() => route.hash, hash => {
  if (!hash) return
  nextTick(() => {
    document.getElementById(hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  })
})

watch(searchQuery, value => {
  if (searchTimer) clearTimeout(searchTimer)

  const query = value.trim()
  if (query.length < 2) {
    searchLoading.value = false
    searchResults.value = []
    return
  }

  searchLoading.value = true
  searchTimer = setTimeout(() => {
    void runSearch(query)
  }, 250)
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
})

async function loadLibraryHome() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    const studentId = auth.currentAccount.id
    const [nextSnapshot, nextItems, nextRevisionPacks, subjects] = await Promise.all([
      getLibrarySnapshot(studentId),
      listLibraryItems(studentId),
      listRevisionPacks(studentId, 6),
      listSubjects(1),
    ])

    snapshot.value = nextSnapshot
    trackedItemCount.value = nextItems.length
    revisionPacks.value = nextRevisionPacks
    subjectDirectory.value = Object.fromEntries(subjects.map(subject => [subject.id, subject.name]))
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load library'
  } finally {
    loading.value = false
  }
}

async function runSearch(query: string) {
  if (!auth.currentAccount) return

  try {
    const results = await searchLibrary(
      auth.currentAccount.id,
      {
        query,
        subject_id: null,
        topic_id: null,
        item_types: [],
        states: [],
        tags: [],
        only_wrong: false,
        only_near_mastery: false,
        only_untouched: false,
        high_frequency_only: false,
        due_only: false,
        downloaded_only: false,
      },
      10,
    )

    if (searchQuery.value.trim() === query) {
      searchResults.value = results
    }
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Library search failed'
  } finally {
    if (searchQuery.value.trim() === query) {
      searchLoading.value = false
    }
  }
}

async function handleBuildRevisionPack() {
  if (!auth.currentAccount || buildingPack.value) return

  buildingPack.value = true
  error.value = ''

  try {
    const title = `Recovery Pack ${new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}`
    await buildRevisionPack(auth.currentAccount.id, title, 12)
    await loadLibraryHome()
    void router.replace({ path: route.path, hash: '#revision-box' })
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Could not build revision pack'
  } finally {
    buildingPack.value = false
  }
}

function openRoute(target: string) {
  if (target.startsWith('/student/library#')) {
    const hash = target.slice('/student/library'.length)
    void router.push({ path: '/student/library', hash })
    return
  }

  void router.push(target)
}

function routeForShelf(shelfType: string) {
  const normalized = shelfType.toLowerCase()
  if (normalized.includes('memory') || normalized.includes('review') || normalized.includes('due')) {
    return '/student/memory'
  }
  if (normalized.includes('mistake') || normalized.includes('wrong')) {
    return '/student/mistakes'
  }
  if (normalized.includes('glossary') || normalized.includes('formula') || normalized.includes('knowledge')) {
    return '/student/glossary'
  }
  if (normalized.includes('feed') || normalized.includes('path')) {
    return '/student/journey'
  }
  return '/student/library'
}

function routeForLibraryItemType(itemType: string) {
  const normalized = itemType.toLowerCase()
  if (normalized.includes('question')) return '/student/practice'
  if (normalized.includes('memory')) return '/student/memory'
  if (normalized.includes('pack')) return '/student/library#revision-box'
  if (
    normalized.includes('glossary') ||
    normalized.includes('entry') ||
    normalized.includes('formula') ||
    normalized.includes('bundle')
  ) {
    return '/student/glossary'
  }
  return '/student/library'
}

function routeForLibrarySearchResult(result: LibrarySearchResultDto) {
  if (result.item_type.toLowerCase().includes('pack')) {
    return '/student/library#revision-box'
  }
  if (result.item_type.toLowerCase().includes('entry') && result.item_ref_id != null) {
    return `/student/glossary/entry/${result.item_ref_id}`
  }
  if (result.item_type.toLowerCase().includes('question') && result.topic_id != null) {
    return practiceRouteForTopic(result.topic_id)
  }
  return routeForLibraryItemType(result.item_type)
}

function subjectLabelForResult(result: LibrarySearchResultDto) {
  if (result.subject_id != null) {
    return subjectDirectory.value[result.subject_id] ?? null
  }
  return null
}

function symbolForShelf(label: string) {
  const cleaned = label.replace(/[^A-Za-z0-9 ]+/g, ' ').trim()
  const words = cleaned.split(/\s+/).filter(Boolean)
  if (words.length === 0) return 'LB'
  if (words.length === 1) return words[0].slice(0, 2).toUpperCase()
  return `${words[0][0]}${words[1][0]}`.toUpperCase()
}

function practiceRouteForTopic(topicId: number | null) {
  return topicId != null ? `/student/practice?topic=${topicId}` : '/student/practice'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between mb-4">
        <div>
          <p class="eyebrow">Library</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Your Academic Hub
          </h1>
        </div>
        <button class="build-btn" :disabled="buildingPack" @click="handleBuildRevisionPack">
          {{ buildingPack ? 'Building...' : 'Build Revision Pack' }}
        </button>
      </div>
      <AppSearchInput
        v-model="searchQuery"
        :loading="searchLoading"
        placeholder="Search topics, formulas, past papers..."
      />
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 4" :key="i" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div>
          <p class="section-label mb-4">Shelves</p>
          <div class="grid grid-cols-4 gap-3">
            <button
              v-for="shelf in shelves"
              :key="shelf.key"
              :id="shelf.key === 'revision' ? 'revision-box' : undefined"
              class="shelf-tile"
              @click="openRoute(shelf.to)"
            >
              <div class="shelf-symbol">{{ shelf.symbol }}</div>
              <h3 class="text-xs font-bold mb-0.5" :style="{ color: 'var(--ink)' }">{{ shelf.label }}</h3>
              <p class="text-[10px] mb-2" :style="{ color: 'var(--ink-muted)' }">{{ shelf.desc }}</p>
              <span v-if="shelf.count" class="count-badge">{{ shelf.count }}</span>
            </button>
          </div>
        </div>

        <div>
          <p class="section-label mb-3">{{ searchQuery.trim() ? 'Search Results' : 'Recently Surfaced' }}</p>
          <div v-if="displayRows.length" class="space-y-2">
            <button
              v-for="item in displayRows"
              :key="item.key"
              class="recent-row flex items-center gap-4 px-4 py-3 rounded-xl border w-full text-left"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
              @click="openRoute(item.route)"
            >
              <div class="item-type-box flex-shrink-0">{{ item.type.charAt(0).toUpperCase() }}</div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ item.subject }}</p>
              </div>
              <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full border"
                :style="{ borderColor: 'transparent', color: 'var(--ink-secondary)' }">
                {{ item.badge }}
              </span>
            </button>
          </div>
          <div v-else class="rounded-xl px-4 py-5 text-sm"
            :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
            No library items match this view yet.
          </div>
        </div>
      </div>

      <div
        class="w-60 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Access</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div v-if="continueCard" class="rounded-xl px-4 py-4"
            :style="{ backgroundColor: 'var(--paper)', border: '1px solid transparent' }">
            <p class="text-[10px] uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">Continue</p>
            <p class="text-sm font-semibold mb-1" :style="{ color: 'var(--ink)' }">{{ continueCard.title }}</p>
            <p class="text-[10px] leading-relaxed mb-3" :style="{ color: 'var(--ink-muted)' }">
              {{ continueCard.reason ?? continueCard.topic_name ?? 'Resume where your learning path left off.' }}
            </p>
            <button class="build-btn w-full" @click="openRoute(continueCard.route)">Resume</button>
          </div>

          <div class="space-y-1">
            <button class="quick-link w-full text-left" @click="openRoute('/student/mistakes')">Mistake Bank</button>
            <button class="quick-link w-full text-left" @click="openRoute('/student/memory')">Memory Mode</button>
            <button class="quick-link w-full text-left" @click="openRoute('/student/glossary')">Glossary Lab</button>
            <button class="quick-link w-full text-left" @click="openRoute('/student/exam-intel')">Exam Intelligence</button>
          </div>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <div class="text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">
              {{ totalItems }}
            </p>
            <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Tracked Items</p>
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
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.build-btn {
  padding: 8px 18px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  background: var(--ink);
  color: var(--paper);
  border: none;
  transition: opacity 120ms;
}

.build-btn:disabled {
  opacity: 0.6;
  cursor: wait;
}

.build-btn:hover {
  opacity: 0.85;
}

.shelf-tile {
  position: relative;
  overflow: hidden;
  border-radius: 16px;
  padding: 18px 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid transparent;
  transition: border-color 130ms ease, transform 130ms ease;
}

.shelf-tile:hover {
  transform: translateY(-2px);
  border-color: var(--ink-muted);
}

.shelf-symbol {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 900;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
  margin-bottom: 10px;
}

.count-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

.recent-row {
  transition: background-color 100ms, transform 100ms;
  cursor: pointer;
}

.recent-row:hover {
  background-color: var(--paper) !important;
  transform: translateX(2px);
}

.item-type-box {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 800;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
}

.quick-link {
  display: flex;
  align-items: center;
  padding: 9px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-secondary);
  cursor: pointer;
  transition: background-color 100ms;
}

.quick-link:hover {
  background-color: var(--paper);
  color: var(--ink);
}
</style>
