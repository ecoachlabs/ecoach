<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listTopics, type TopicDto } from '@/ipc/coach'
import {
  listPastPaperCourses,
  listPastPaperTopicCounts,
  listPastPapersForSubject,
  startPastPaperSection,
  startPastPaperTopicSession,
  type PastPaperCourseSummaryDto,
  type PastPaperTopicCountDto,
  type PastPaperTopicFormat,
  type PastPaperYearDto,
  type PastPaperSectionDto,
  type PastPaperSectionKind,
} from '@/ipc/pastPapers'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

// ── Fonts (Nothing design — Space Grotesk + Space Mono) ─────────────
onMounted(() => {
  const fontHref =
    'https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Space+Mono:wght@400;700&display=swap'
  if (!document.head.querySelector(`link[href="${fontHref}"]`)) {
    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = fontHref
    document.head.appendChild(link)
  }
})

// ── Load courses ────────────────────────────────────────────────────
const courses = ref<PastPaperCourseSummaryDto[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    courses.value = await listPastPaperCourses()
    void prefetchFilterData()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load courses'
  } finally {
    loading.value = false
  }

  // ?topic=ID query: pre-filter by topic and auto-expand the matching course.
  const topicId = Number(route.query.topic)
  if (Number.isFinite(topicId) && topicId > 0) {
    activeFilters.value.topic = new Set([topicId])
    activeDimension.value = 'topic'
    // Find a course that contains this topic by lazy-checking each —
    // cheap because the user usually arrives with one valid topic.
    for (const course of courses.value) {
      await ensurePapersLoaded(course.subject_id)
      const yearsForCourse = papersByCourse.value[course.subject_id] ?? []
      if (yearsForCourse.some(y => y.topic_ids.includes(topicId))) {
        expandedCourseId.value = course.subject_id
        break
      }
    }
  }
})

async function prefetchFilterData(): Promise<void> {
  await Promise.allSettled(
    courses.value.map(async (course) => {
      await ensurePapersLoaded(course.subject_id)
      await ensureTopicNamesLoaded(course.subject_id)
      await ensureTopicCountsLoaded(course.subject_id)
    }),
  )
}

// ── Expanded course state (one-at-a-time, mode: "confidence through emptiness") ──
const expandedCourseId = ref<number | null>(null)
const papersByCourse = ref<Record<number, PastPaperYearDto[]>>({})
const loadingPapers = ref<Set<number>>(new Set())

async function ensurePapersLoaded(subjectId: number): Promise<void> {
  if (papersByCourse.value[subjectId] !== undefined) return
  loadingPapers.value.add(subjectId)
  loadingPapers.value = new Set(loadingPapers.value)
  try {
    papersByCourse.value[subjectId] = await listPastPapersForSubject(subjectId)
  } catch {
    papersByCourse.value[subjectId] = []
  } finally {
    loadingPapers.value.delete(subjectId)
    loadingPapers.value = new Set(loadingPapers.value)
  }
}

async function toggleCourse(subjectId: number): Promise<void> {
  if (expandedCourseId.value === subjectId) {
    expandedCourseId.value = null
    return
  }
  await Promise.all([
    ensurePapersLoaded(subjectId),
    ensureTopicCountsLoaded(subjectId),
  ])
  expandedCourseId.value = subjectId
}

// ── Topic-count cache (for the "Topic" body view) ───────────────────
const topicCountsByCourse = ref<Record<number, PastPaperTopicCountDto[]>>({})
const loadingTopicCounts = ref<Set<number>>(new Set())

async function ensureTopicCountsLoaded(subjectId: number): Promise<void> {
  if (topicCountsByCourse.value[subjectId] !== undefined) return
  loadingTopicCounts.value.add(subjectId)
  loadingTopicCounts.value = new Set(loadingTopicCounts.value)
  try {
    topicCountsByCourse.value[subjectId] = await listPastPaperTopicCounts(subjectId)
  } catch {
    topicCountsByCourse.value[subjectId] = []
  } finally {
    loadingTopicCounts.value.delete(subjectId)
    loadingTopicCounts.value = new Set(loadingTopicCounts.value)
  }
}

// ── Topic name cache (for topic chips) ──────────────────────────────
const topicNames = ref<Record<number, string>>({})
async function ensureTopicNamesLoaded(subjectId: number): Promise<void> {
  const alreadyLoaded = (papersByCourse.value[subjectId] ?? []).every(y =>
    y.topic_ids.every(id => topicNames.value[id] !== undefined),
  )
  if (alreadyLoaded) return
  try {
    const topics = await listTopics(subjectId)
    const nextMap = { ...topicNames.value }
    for (const t of topics as TopicDto[]) nextMap[t.id] = t.name
    topicNames.value = nextMap
  } catch { /* silent — chips will fall back to "Topic {id}" */ }
}
watch(expandedCourseId, async (id) => {
  if (id != null) await ensureTopicNamesLoaded(id)
})

// ── Filter state ────────────────────────────────────────────────────
type Dimension = 'year' | 'topic' | 'keyword'
const activeDimension = ref<Dimension>('year')
const activeFilters = ref<{
  year: Set<number>
  topic: Set<number>
  keyword: Set<string>
}>({
  year: new Set(),
  topic: new Set(),
  keyword: new Set(),
})
const searchQuery = ref('')

function toggleYear(year: number): void {
  const next = new Set(activeFilters.value.year)
  next.has(year) ? next.delete(year) : next.add(year)
  activeFilters.value = { ...activeFilters.value, year: next }
}
function toggleTopic(topicId: number): void {
  const next = new Set(activeFilters.value.topic)
  next.has(topicId) ? next.delete(topicId) : next.add(topicId)
  activeFilters.value = { ...activeFilters.value, topic: next }
}
function toggleKeyword(kw: string): void {
  const next = new Set(activeFilters.value.keyword)
  next.has(kw) ? next.delete(kw) : next.add(kw)
  activeFilters.value = { ...activeFilters.value, keyword: next }
}
function clearAllFilters(): void {
  activeFilters.value = { year: new Set(), topic: new Set(), keyword: new Set() }
  searchQuery.value = ''
}

// ── Chip populations (drawn from loaded data; scoped to expanded course
//     when available, else global across all loaded courses) ─────────
const allLoadedYears = computed<PastPaperYearDto[]>(() => {
  if (expandedCourseId.value != null && papersByCourse.value[expandedCourseId.value]) {
    return papersByCourse.value[expandedCourseId.value]
  }
  return Object.values(papersByCourse.value).flat()
})

const yearChips = computed<number[]>(() => {
  const years = new Set<number>()
  for (const y of allLoadedYears.value) years.add(y.exam_year)
  // Fallback: if nothing loaded yet, derive year ranges from course summaries.
  if (years.size === 0) {
    for (const c of courses.value) {
      if (c.first_year != null && c.last_year != null) {
        for (let y = c.last_year; y >= c.first_year; y--) years.add(y)
      }
    }
  }
  return Array.from(years).sort((a, b) => b - a)
})

const topicChips = computed<Array<{ id: number; name: string }>>(() => {
  const ids = new Set<number>()
  for (const y of allLoadedYears.value) for (const id of y.topic_ids) ids.add(id)
  return Array.from(ids)
    .map(id => ({ id, name: topicNames.value[id] ?? `Topic ${id}` }))
    .sort((a, b) => a.name.localeCompare(b.name))
})

const keywordChips = computed<string[]>(() => {
  const kws = new Set<string>()
  for (const y of allLoadedYears.value) for (const k of y.keywords) kws.add(k)
  return Array.from(kws).sort((a, b) => a.localeCompare(b))
})

// ── Apply filters to the years list of one course ───────────────────
function yearPassesFilters(year: PastPaperYearDto): boolean {
  const f = activeFilters.value
  if (f.year.size > 0 && !f.year.has(year.exam_year)) return false
  if (f.topic.size > 0 && !year.topic_ids.some(id => f.topic.has(id))) return false
  if (f.keyword.size > 0 && !year.keywords.some(k => f.keyword.has(k))) return false
  if (searchQuery.value.trim().length > 0) {
    const q = searchQuery.value.trim().toLowerCase()
    const haystack = [
      String(year.exam_year),
      year.title.toLowerCase(),
      (year.paper_code ?? '').toLowerCase(),
      ...year.keywords.map(k => k.toLowerCase()),
    ].join(' ')
    if (!haystack.includes(q)) return false
  }
  return true
}

function filteredYearsForCourse(subjectId: number): PastPaperYearDto[] {
  const years = papersByCourse.value[subjectId] ?? []
  return years.filter(yearPassesFilters)
}

// ── Topic-list view: filtered + counted topics for the expanded course ──
// When the active filter dimension is "topic", the accordion body shows
// this list instead of per-year sections. Search text filters topic
// name; topic chips, when any are selected, narrow to those topics.
function filteredTopicsForCourse(subjectId: number): PastPaperTopicCountDto[] {
  const topics = topicCountsByCourse.value[subjectId] ?? []
  const topicFilter = activeFilters.value.topic
  const q = searchQuery.value.trim().toLowerCase()
  return topics.filter(t => {
    if (topicFilter.size > 0 && !topicFilter.has(t.topic_id)) return false
    if (q.length > 0 && !t.topic_name.toLowerCase().includes(q)) return false
    return true
  })
}

// ── Apply filters + search to the course list ───────────────────────
const filteredCourses = computed<PastPaperCourseSummaryDto[]>(() => {
  const q = searchQuery.value.trim().toLowerCase()
  const hasFacetFilters =
    activeFilters.value.year.size > 0 ||
    activeFilters.value.topic.size > 0 ||
    activeFilters.value.keyword.size > 0

  return courses.value.filter(course => {
    if (!q && !hasFacetFilters) return true

    const courseMatch =
      q.length > 0 &&
      (
        course.subject_name.toLowerCase().includes(q) ||
        course.subject_code.toLowerCase().includes(q)
      )

    const years = papersByCourse.value[course.subject_id]
    if (years === undefined) return courseMatch || (!q && !hasFacetFilters)

    return courseMatch || years.some(yearPassesFilters)
  })
})

// ── Global stats for the hero subtitle ──────────────────────────────
const globalStats = computed(() => {
  const totalCourses = courses.value.length
  const totalPapers = courses.value.reduce((sum, c) => sum + c.paper_count, 0)
  let minYear: number | null = null
  let maxYear: number | null = null
  for (const c of courses.value) {
    if (c.first_year != null && (minYear == null || c.first_year < minYear)) minYear = c.first_year
    if (c.last_year != null && (maxYear == null || c.last_year > maxYear)) maxYear = c.last_year
  }
  return { totalCourses, totalPapers, minYear, maxYear }
})

// ── Start a section ─────────────────────────────────────────────────
const startingKey = ref<string | null>(null)
function keyFor(paperId: number, sectionLabel: string): string {
  return `${paperId}:${sectionLabel}`
}

// ── Start a topic test. Objective and essay are kept strictly
//     separate: each topic row offers two independent launchers.
const startingTopicKey = ref<string | null>(null)
function topicKey(subjectId: number, topicId: number, format: PastPaperTopicFormat): string {
  return `${subjectId}:${topicId}:${format}`
}

async function openTopicTest(
  subjectId: number,
  topicId: number,
  format: PastPaperTopicFormat,
): Promise<void> {
  if (!auth.currentAccount || startingTopicKey.value !== null) return
  startingTopicKey.value = topicKey(subjectId, topicId, format)
  error.value = ''
  try {
    const snap = await startPastPaperTopicSession(
      auth.currentAccount.id,
      subjectId,
      topicId,
      format,
      false,
    )
    router.push(`/student/session/${snap.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start topic test'
    startingTopicKey.value = null
  }
}

async function openSection(paperId: number, section: PastPaperSectionDto): Promise<void> {
  if (!auth.currentAccount || startingKey.value !== null) return
  if (!sectionIsPlayable(section)) {
    error.value = 'Objective sections can be opened right now. Essay sections are listed for browsing, and answer capture is still on the way.'
    return
  }
  startingKey.value = keyFor(paperId, section.section_label)
  error.value = ''
  try {
    const snap = await startPastPaperSection(
      auth.currentAccount.id,
      paperId,
      section.section_label,
      false,
    )
    router.push(`/student/session/${snap.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to open section'
    startingKey.value = null
  }
}

// ── UI helpers ──────────────────────────────────────────────────────
function sectionKindLabel(kind: PastPaperSectionKind): string {
  if (kind === 'objective') return 'Objectives'
  if (kind === 'essay') return 'Essay'
  return 'Mixed'
}

function sectionIsPlayable(section: PastPaperSectionDto): boolean {
  return section.section_kind === 'objective'
}

function yearRange(first: number | null, last: number | null): string {
  if (first == null || last == null) return '—'
  if (first === last) return String(first)
  return `${first}–${last}`
}

function pluralise(n: number, singular: string, plural: string): string {
  return `${n} ${n === 1 ? singular : plural}`
}

function courseCode(code: string): string {
  // Take the first 3–4 letters uppercase for the eyebrow.
  return code.slice(0, 4).toUpperCase()
}
</script>

<template>
  <div class="pq-shell">

    <!-- ── HEADER (hero + stats) ────────────────────────────────── -->
    <header class="pq-hero">
      <p class="pq-eyebrow">PAST QUESTIONS</p>
      <h1 class="pq-title">Past Questions</h1>
      <p class="pq-sub">
        <span class="pq-stat">{{ pluralise(globalStats.totalCourses, 'course', 'courses') }}</span>
        <span class="pq-dot" aria-hidden="true">·</span>
        <span class="pq-stat">{{ pluralise(globalStats.totalPapers, 'paper', 'papers') }}</span>
        <template v-if="globalStats.minYear != null && globalStats.maxYear != null">
          <span class="pq-dot" aria-hidden="true">·</span>
          <span class="pq-stat">{{ yearRange(globalStats.minYear, globalStats.maxYear) }}</span>
        </template>
      </p>
    </header>

    <!-- ── SEARCH + TAG RAIL ───────────────────────────────────── -->
    <section class="pq-controls">
      <div class="pq-search">
        <label class="pq-search-label" for="pq-search-input">SEARCH</label>
        <input
          id="pq-search-input"
          v-model="searchQuery"
          type="text"
          class="pq-search-input"
          placeholder="Type to filter courses, years, keywords…"
          autocomplete="off"
        />
      </div>

      <div class="pq-tags">
        <div class="pq-tags-head">
          <p class="pq-tags-eyebrow">FILTER BY</p>
          <div class="pq-dim-switch" role="tablist" aria-label="Filter dimension">
            <button
              v-for="dim in (['year', 'topic', 'keyword'] as Dimension[])"
              :key="dim"
              type="button"
              role="tab"
              :aria-selected="activeDimension === dim"
              class="pq-dim"
              :class="{ 'pq-dim--on': activeDimension === dim }"
              @click="activeDimension = dim"
            >
              {{ dim.toUpperCase() }}
            </button>
          </div>
          <button
            v-if="activeFilters.year.size + activeFilters.topic.size + activeFilters.keyword.size + (searchQuery ? 1 : 0) > 0"
            type="button"
            class="pq-clear"
            @click="clearAllFilters"
          >
            CLEAR
          </button>
        </div>

        <!-- In TOPIC mode the chip rail disappears entirely — the full
             topic list (with Q tallies) is rendered inside the expanded
             course body, so a chip row would be redundant and visually
             noisy. YEAR and KEYWORD modes keep their chips. -->
        <div v-if="activeDimension !== 'topic'" class="pq-chip-row">
          <!-- YEAR chips -->
          <template v-if="activeDimension === 'year'">
            <button
              v-for="y in yearChips"
              :key="'y' + y"
              type="button"
              class="pq-chip pq-chip--mono"
              :class="{ 'pq-chip--on': activeFilters.year.has(y) }"
              @click="toggleYear(y)"
            >
              {{ y }}
            </button>
            <p v-if="yearChips.length === 0" class="pq-empty-chips">No years loaded yet. Expand a course.</p>
          </template>

          <!-- KEYWORD chips -->
          <template v-else>
            <button
              v-for="k in keywordChips"
              :key="'k' + k"
              type="button"
              class="pq-chip"
              :class="{ 'pq-chip--on': activeFilters.keyword.has(k) }"
              @click="toggleKeyword(k)"
            >
              {{ k }}
            </button>
            <p v-if="keywordChips.length === 0" class="pq-empty-chips">Expand a course to see keywords.</p>
          </template>
        </div>
      </div>
    </section>

    <!-- ── ERROR / LOADING ─────────────────────────────────────── -->
    <p v-if="error" class="pq-error">[ ERROR ] {{ error }}</p>

    <div v-if="loading" class="pq-loading">
      <p class="pq-mono-tag">[ LOADING... ]</p>
    </div>

    <div v-else-if="filteredCourses.length === 0" class="pq-loading">
      <p class="pq-mono-tag">[ NO COURSES ]</p>
      <p class="pq-empty-copy">No past-paper courses match "{{ searchQuery }}".</p>
    </div>

    <!-- ── COURSE ACCORDION ────────────────────────────────────── -->
    <ol v-else class="pq-list" role="list">
      <li
        v-for="course in filteredCourses"
        :key="course.subject_id"
        class="pq-row"
        :class="{ 'pq-row--open': expandedCourseId === course.subject_id }"
      >
        <!-- COURSE HEADER ROW (click to expand/collapse) -->
        <button
          type="button"
          class="pq-row-head"
          :aria-expanded="expandedCourseId === course.subject_id"
          @click="toggleCourse(course.subject_id)"
        >
          <div class="pq-row-copy">
            <p class="pq-row-code">{{ courseCode(course.subject_code) }}</p>
            <h2 class="pq-row-name">{{ course.subject_name }}</h2>
          </div>
          <div class="pq-row-meta">
            <p class="pq-row-stat">
              <span class="pq-row-stat-num">{{ course.paper_count }}</span>
              <span class="pq-row-stat-unit">PAPERS</span>
            </p>
            <p class="pq-row-years">{{ yearRange(course.first_year, course.last_year) }}</p>
          </div>
          <span class="pq-row-caret" aria-hidden="true">
            {{ expandedCourseId === course.subject_id ? '−' : '+' }}
          </span>
        </button>

        <!-- EXPANDED: years + sections (YEAR mode) or topics + tallies (TOPIC mode) -->
        <div v-if="expandedCourseId === course.subject_id" class="pq-row-body">

          <!-- ── TOPIC MODE ─────────────────────────────────────── -->
          <template v-if="activeDimension === 'topic'">
            <p v-if="loadingTopicCounts.has(course.subject_id)" class="pq-mono-tag pq-mono-tag--inset">[ LOADING TOPICS... ]</p>

            <ul
              v-else-if="filteredTopicsForCourse(course.subject_id).length > 0"
              class="pq-topics"
              role="list"
            >
              <li
                v-for="topic in filteredTopicsForCourse(course.subject_id)"
                :key="'pt' + topic.topic_id"
                class="pq-topic"
              >
                <div class="pq-topic-head">
                  <span class="pq-topic-name">{{ topic.topic_name }}</span>
                  <span class="pq-topic-counts">
                    <span
                      v-if="topic.objective_count > 0"
                      class="pq-topic-count"
                    >
                      <span class="pq-topic-count-num">{{ topic.objective_count }}</span>
                      <span class="pq-topic-count-unit">OBJECTIVES</span>
                    </span>
                    <span
                      v-if="topic.essay_count > 0"
                      class="pq-topic-count pq-topic-count--muted"
                    >
                      <span class="pq-topic-count-num">{{ topic.essay_count }}</span>
                      <span class="pq-topic-count-unit">ESSAY</span>
                    </span>
                  </span>
                </div>

                <div class="pq-topic-actions">
                  <button
                    v-if="topic.objective_count > 0"
                    type="button"
                    class="pq-topic-action pq-topic-action--primary"
                    :class="{ 'pq-topic-action--busy': startingTopicKey === topicKey(course.subject_id, topic.topic_id, 'objective') }"
                    :disabled="startingTopicKey !== null"
                    @click="openTopicTest(course.subject_id, topic.topic_id, 'objective')"
                  >
                    {{
                      startingTopicKey === topicKey(course.subject_id, topic.topic_id, 'objective')
                        ? 'STARTING…'
                        : 'OBJECTIVES →'
                    }}
                  </button>
                  <button
                    v-if="topic.essay_count > 0"
                    type="button"
                    class="pq-topic-action"
                    :class="{ 'pq-topic-action--busy': startingTopicKey === topicKey(course.subject_id, topic.topic_id, 'essay') }"
                    :disabled="startingTopicKey !== null"
                    @click="openTopicTest(course.subject_id, topic.topic_id, 'essay')"
                  >
                    {{
                      startingTopicKey === topicKey(course.subject_id, topic.topic_id, 'essay')
                        ? 'STARTING…'
                        : 'ESSAY →'
                    }}
                  </button>
                </div>
              </li>
            </ul>

            <p v-else class="pq-mono-tag pq-mono-tag--inset">[ NO TOPICS MATCH FILTERS ]</p>
          </template>

          <!-- ── YEAR / KEYWORD MODE ────────────────────────────── -->
          <template v-else>
            <p v-if="loadingPapers.has(course.subject_id)" class="pq-mono-tag pq-mono-tag--inset">[ LOADING PAPERS... ]</p>

            <ol v-else-if="filteredYearsForCourse(course.subject_id).length > 0" class="pq-years" role="list">
              <li
                v-for="year in filteredYearsForCourse(course.subject_id)"
                :key="year.paper_id"
                class="pq-year"
              >
                <!-- YEAR HEAD -->
                <div class="pq-year-head">
                  <p class="pq-year-num">{{ year.exam_year }}</p>
                  <div class="pq-year-copy">
                    <p class="pq-year-title">{{ year.title }}</p>
                    <p v-if="year.paper_code" class="pq-year-code">{{ year.paper_code }}</p>
                  </div>
                </div>

                <!-- SECTIONS -->
                <ul class="pq-sections" role="list">
                  <li v-for="section in year.sections" :key="section.section_label" class="pq-section-item">
                    <button
                      type="button"
                      class="pq-section"
                      :class="{
                        'pq-section--essay':     section.section_kind === 'essay',
                        'pq-section--objective': section.section_kind === 'objective',
                        'pq-section--mixed':     section.section_kind === 'mixed',
                        'pq-section--busy':      startingKey === keyFor(year.paper_id, section.section_label),
                      }"
                      :disabled="startingKey !== null || !sectionIsPlayable(section)"
                      @click="openSection(year.paper_id, section)"
                    >
                      <span class="pq-section-head">
                        <span class="pq-section-label">SECTION {{ section.section_label || '—' }}</span>
                        <span class="pq-section-kind">{{ sectionKindLabel(section.section_kind) }}</span>
                      </span>
                      <span class="pq-section-count">
                        <span class="pq-section-count-num">{{ section.question_count }}</span>
                        <span class="pq-section-count-unit">{{ section.question_count === 1 ? "Q" : "Q'S" }}</span>
                      </span>
                      <span class="pq-section-arrow" aria-hidden="true">
                        {{ startingKey === keyFor(year.paper_id, section.section_label) ? '…' : sectionIsPlayable(section) ? '→' : 'SOON' }}
                      </span>
                    </button>
                  </li>
                </ul>
              </li>
            </ol>

            <p v-else class="pq-mono-tag pq-mono-tag--inset">[ NO PAPERS MATCH FILTERS ]</p>
          </template>

        </div>
      </li>
    </ol>

  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   Nothing-design tokens — local so this view is self-contained and
   consistent across light/dark.
   ═══════════════════════════════════════════════════════════════════ */
.pq-shell {
  --paper:            #faf8f5;
  --paper-dim:        #f2efe9;
  --ink:              #1a1612;
  --ink-primary:      rgba(26, 22, 18, 0.90);
  --ink-secondary:    rgba(26, 22, 18, 0.60);
  --ink-muted:        rgba(26, 22, 18, 0.40);
  --rule:             rgba(26, 22, 18, 0.12);
  --rule-strong:      rgba(26, 22, 18, 0.28);
  --success:          #15803d;
  --warm:             #c2410c;
  --danger:           #b91c1c;

  min-height: 100%;
  padding: 48px clamp(24px, 5vw, 72px) 96px;
  background: var(--paper);
  color: var(--ink);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-feature-settings: 'ss01', 'ss02';
  letter-spacing: -0.005em;
}

/* ─────── HERO ─────── */
.pq-hero { margin-bottom: 56px; }
.pq-eyebrow {
  margin: 0 0 18px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-title {
  margin: 0 0 14px;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 300;
  font-size: clamp(48px, 7vw, 88px);
  line-height: 0.98;
  letter-spacing: -0.03em;
  color: var(--ink);
}
.pq-sub {
  margin: 0;
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 12px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 0.02em;
  color: var(--ink-secondary);
}
.pq-stat { color: var(--ink-primary); }
.pq-dot { color: var(--ink-muted); }

/* ─────── CONTROLS (search + tag rail) ─────── */
.pq-controls {
  display: grid;
  grid-template-columns: minmax(280px, 1fr) minmax(0, 2fr);
  gap: 48px;
  padding-bottom: 28px;
  margin-bottom: 40px;
  border-bottom: 1px solid var(--rule);
}

.pq-search { display: grid; gap: 10px; align-content: start; }
.pq-search-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-search-input {
  width: 100%;
  padding: 14px 0;
  border: none;
  border-bottom: 1px solid var(--rule-strong);
  background: transparent;
  color: var(--ink);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 15px;
  letter-spacing: -0.005em;
  outline: none;
  transition: border-color 160ms ease;
}
.pq-search-input::placeholder { color: var(--ink-muted); }
.pq-search-input:focus { border-color: var(--ink); }

.pq-tags { display: grid; gap: 16px; align-content: start; }
.pq-tags-head {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}
.pq-tags-eyebrow {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}

/* Segmented "dimension" switcher — three mutually-exclusive pills */
.pq-dim-switch {
  display: inline-flex;
  gap: 4px;
  padding: 2px;
  border: 1px solid var(--rule-strong);
  border-radius: 999px;
}
.pq-dim {
  padding: 7px 14px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--ink-secondary);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.20em;
  cursor: pointer;
  transition: background 160ms ease, color 160ms ease;
}
.pq-dim:hover:not(.pq-dim--on) { color: var(--ink); }
.pq-dim--on {
  background: var(--ink);
  color: var(--paper);
}

.pq-clear {
  margin-left: auto;
  padding: 6px 12px;
  border: none;
  background: transparent;
  color: var(--ink-muted);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: color 160ms ease;
}
.pq-clear:hover { color: var(--ink); }

/* Chip row */
.pq-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 40px;
}
.pq-chip {
  padding: 8px 14px;
  border: 1px solid var(--rule-strong);
  border-radius: 999px;
  background: transparent;
  color: var(--ink-secondary);
  font-family: 'Space Grotesk', sans-serif;
  font-size: 13px;
  font-weight: 500;
  letter-spacing: -0.005em;
  cursor: pointer;
  transition: border-color 140ms ease, color 140ms ease, background 140ms ease;
}
.pq-chip:hover:not(.pq-chip--on) {
  border-color: var(--ink);
  color: var(--ink);
}
.pq-chip--mono {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
}
.pq-chip--on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.pq-empty-chips {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
}

/* ─────── ERRORS / LOADING ─────── */
.pq-error {
  margin: 0 0 24px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--danger);
}
.pq-loading {
  padding: 96px 0;
  display: grid;
  gap: 10px;
}
.pq-mono-tag {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-mono-tag--inset { padding: 20px 0 12px; }
.pq-empty-copy {
  margin: 0;
  font-size: 15px;
  color: var(--ink-muted);
}

/* ═══════════════════════════════════════════════════════════════════
   ACCORDION — the heart of the page.
   ═══════════════════════════════════════════════════════════════════ */
.pq-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}
.pq-row {
  border-bottom: 1px solid var(--rule);
}
.pq-row:first-child { border-top: 1px solid var(--rule); }

/* COURSE HEAD — the clickable row */
.pq-row-head {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto 40px;
  align-items: center;
  gap: 32px;
  width: 100%;
  padding: 28px 4px;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  color: inherit;
  transition: background 120ms ease, padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.pq-row-head:hover {
  background: var(--paper-dim);
  padding-left: 16px;
}
.pq-row--open .pq-row-head {
  background: var(--paper-dim);
  padding-left: 16px;
}

.pq-row-copy { display: grid; gap: 4px; min-width: 0; }
.pq-row-code {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-row-name {
  margin: 0;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 400;
  font-size: clamp(22px, 2.2vw, 28px);
  line-height: 1.15;
  letter-spacing: -0.015em;
  color: var(--ink);
}

.pq-row-meta {
  display: flex;
  align-items: baseline;
  gap: 24px;
  white-space: nowrap;
}
.pq-row-stat {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
}
.pq-row-stat-num {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 22px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.01em;
}
.pq-row-stat-unit {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-row-years {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
}

.pq-row-caret {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 999px;
  border: 1px solid var(--rule-strong);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 16px;
  font-weight: 400;
  color: var(--ink);
  transition: transform 180ms ease, background 180ms ease;
}
.pq-row--open .pq-row-caret {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

/* ─────── EXPANDED BODY ─────── */
.pq-row-body {
  padding: 4px 16px 32px;
  display: grid;
  gap: 24px;
}
.pq-years {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 32px;
}
.pq-year {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: 32px;
  align-items: start;
}

.pq-year-head {
  display: grid;
  gap: 4px;
  align-content: start;
  padding-top: 12px;
}
.pq-year-num {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 36px;
  font-weight: 700;
  line-height: 1;
  color: var(--ink);
  letter-spacing: -0.02em;
}
.pq-year-copy { display: none; }
/* On wider viewports, show the title under the year */
@media (min-width: 900px) {
  .pq-year-copy { display: grid; gap: 2px; margin-top: 6px; }
  .pq-year-title {
    margin: 0;
    font-family: 'Space Grotesk', sans-serif;
    font-size: 13px;
    font-weight: 500;
    color: var(--ink-secondary);
    letter-spacing: -0.005em;
  }
  .pq-year-code {
    margin: 0;
    font-family: 'Space Mono', ui-monospace, monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.18em;
    color: var(--ink-muted);
  }
}

/* ─────── TOPIC LIST (Topic-mode body) ───────
   Each <li> row carries a head (name + counts) and an actions strip
   (objectives pill + essay pill). Objectives and essays are NEVER
   mixed — each pill launches its own session. Both pills hide at rest
   and fade in on hover to keep the row quiet. */
.pq-topics {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0;
  border-top: 1px solid var(--rule);
}

.pq-topic {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 20px;
  padding: 18px 4px;
  border-bottom: 1px solid var(--rule);
  transition: background 120ms ease, padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.pq-topic:hover { background: var(--paper-dim); padding-left: 16px; }

/* ── head: name + counts ───────────────────────────────────────── */
.pq-topic-head {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: baseline;
  gap: 20px;
  min-width: 0;
}
.pq-topic-name {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 18px;
  font-weight: 500;
  letter-spacing: -0.01em;
  color: var(--ink);
  min-width: 0;
  overflow-wrap: anywhere;
}
.pq-topic-counts {
  display: inline-flex;
  align-items: baseline;
  gap: 18px;
  white-space: nowrap;
}
.pq-topic-count {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
}
.pq-topic-count-num {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 20px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.01em;
}
.pq-topic-count-unit {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
}
.pq-topic-count--muted .pq-topic-count-num {
  color: var(--ink-secondary);
  font-size: 16px;
}

/* ── actions: revealed on hover, always visible when a launch is in
   flight so the busy state remains legible. Side-by-side pills, with
   the objectives pill reading as the primary (inverted ink) and the
   essay pill as secondary (outlined). ───────────────────────────── */
.pq-topic-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  visibility: hidden;
  opacity: 0;
  transform: translateX(-4px);
  transition: opacity 140ms ease, transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.pq-topic:hover .pq-topic-actions,
.pq-topic:focus-within .pq-topic-actions {
  visibility: visible;
  opacity: 1;
  transform: translateX(0);
}
.pq-topic-action--busy,
.pq-topic-actions:has(.pq-topic-action--busy) {
  visibility: visible;
  opacity: 1;
  transform: translateX(0);
}

.pq-topic-action {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  padding: 8px 14px;
  border: 1px solid var(--ink);
  border-radius: 999px;
  background: transparent;
  color: var(--ink);
  white-space: nowrap;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.pq-topic-action:disabled { cursor: not-allowed; opacity: 0.5; }
.pq-topic-action:hover:not(:disabled) { transform: translateX(2px); }

.pq-topic-action--primary {
  background: var(--ink);
  color: var(--paper);
}
.pq-topic-action--primary:hover:not(:disabled) {
  background: var(--ink);
}

.pq-topic-action--busy,
.pq-topic-action--primary.pq-topic-action--busy {
  background: var(--ink);
  color: var(--paper);
  cursor: wait;
}

.pq-sections {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}
.pq-section-item { display: block; }
.pq-section {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto 24px;
  align-items: center;
  gap: 20px;
  width: 100%;
  padding: 18px 20px;
  border: 1px solid var(--rule);
  border-radius: 12px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 140ms ease,
    background 140ms ease,
    transform 180ms cubic-bezier(0.16, 1, 0.3, 1),
    padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.pq-section:hover:not(:disabled) {
  border-color: var(--ink);
  padding-left: 28px;
}
.pq-section:disabled { opacity: 0.6; }
.pq-section--busy { border-color: var(--ink); cursor: wait; }
.pq-section:not(.pq-section--busy):disabled { cursor: not-allowed; }

.pq-section-head {
  display: inline-flex;
  align-items: baseline;
  gap: 14px;
  min-width: 0;
}
.pq-section-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pq-section-kind {
  font-family: 'Space Grotesk', sans-serif;
  font-size: 18px;
  font-weight: 500;
  letter-spacing: -0.01em;
  color: var(--ink);
}
/* Objective vs Essay: differentiate by OPACITY + a subtle kind underline,
   not by colored card backgrounds. Matches Nothing's data-over-color rule. */
.pq-section--essay .pq-section-kind {
  font-weight: 400;
  font-style: italic;
}
.pq-section--mixed .pq-section-kind {
  color: var(--ink-secondary);
}

.pq-section-count {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  white-space: nowrap;
}
.pq-section-count-num {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 18px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: -0.01em;
}
.pq-section-count-unit {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
}

.pq-section-arrow {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 16px;
  font-weight: 400;
  color: var(--ink-secondary);
  justify-self: end;
  transition: transform 180ms ease, color 140ms ease;
}
.pq-section:hover:not(:disabled) .pq-section-arrow {
  color: var(--ink);
  transform: translateX(4px);
}

/* ─────── Narrow viewports ─────── */
@media (max-width: 760px) {
  .pq-shell { padding: 32px 20px 72px; }
  .pq-controls { grid-template-columns: 1fr; gap: 28px; }
  .pq-row-head {
    grid-template-columns: minmax(0, 1fr) 28px;
    gap: 18px;
  }
  .pq-row-meta { display: none; }
  .pq-year { grid-template-columns: 1fr; gap: 14px; }
  .pq-year-head { padding-top: 0; }
  .pq-year-num { font-size: 26px; }
  .pq-section { grid-template-columns: minmax(0, 1fr) auto; }
  .pq-section-arrow { display: none; }
}

/* ─────── Dark-mode support ─────── */
@media (prefers-color-scheme: dark) {
  .pq-shell {
    --paper: #0a0906;
    --paper-dim: #15130f;
    --ink: #f3ede2;
    --ink-primary: rgba(243, 237, 226, 0.92);
    --ink-secondary: rgba(243, 237, 226, 0.60);
    --ink-muted: rgba(243, 237, 226, 0.38);
    --rule: rgba(243, 237, 226, 0.12);
    --rule-strong: rgba(243, 237, 226, 0.28);
  }
}
</style>
