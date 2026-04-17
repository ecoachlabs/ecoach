<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { getReadinessReport, type SubjectReadinessDto } from '@/ipc/readiness'
import {
  getActiveJourneyRoute,
  buildOrRefreshJourneyRoute,
  type JourneyRouteSnapshot,
} from '@/ipc/journey'

const auth = useAuthStore()
const ui = useUiStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const readiness = ref<SubjectReadinessDto[]>([])
const journeyMap = ref<Record<number, JourneyRouteSnapshot | null>>({})
const searchQuery = ref('')
const activeLevel = ref('JHS 1')

const levels = [
  { label: 'Junior High JHS 1', key: 'JHS 1', count: 11 },
  { label: 'Junior High JHS 2', key: 'JHS 2', count: 10 },
  { label: 'Junior High JHS 3', key: 'JHS 3', count: 9 },
  { label: 'Primary 1',         key: 'P1',    count: 8  },
  { label: 'Primary 2',         key: 'P2',    count: 9  },
  { label: 'Primary 3',         key: 'P3',    count: 10 },
  { label: 'Primary 4',         key: 'P4',    count: 11 },
  { label: 'Primary 5',         key: 'P5',    count: 11 },
  { label: 'Primary 6',         key: 'P6',    count: 11 },
  { label: 'Senior High SHS 1', key: 'SHS 1', count: 9  },
  { label: 'Senior High SHS 2', key: 'SHS 2', count: 8  },
  { label: 'Senior High SHS 3', key: 'SHS 3', count: 7  },
  { label: 'BECE',              key: 'BECE',  count: 24 },
  { label: 'WASSCE SHS',        key: 'WASSCE',count: 30 },
  { label: 'BECE Mocks',        key: 'BECE Mocks', count: 7 },
]

interface SubjectCfg {
  category: string
  color: string       // category text color
  iconBg: string      // icon box background
  iconColor: string   // icon symbol color
  visualBg: string    // frosted visual area bg
  blobColor: string   // blurred blob color
  symbol: string      // icon symbol
}

const cfgMap: Record<string, SubjectCfg> = {
  MATH: { category: 'MATHEMATICS',   color: '#ea580c', iconBg: 'rgba(234,88,12,0.1)',   iconColor: '#ea580c', visualBg: '#f7f4f0', blobColor: 'rgba(234,88,12,0.25)', symbol: '∑' },
  ENG:  { category: 'ENGLISH',       color: '#2563eb', iconBg: 'rgba(37,99,235,0.1)',   iconColor: '#2563eb', visualBg: '#f0f4f8', blobColor: 'rgba(37,99,235,0.2)',  symbol: 'Aa' },
  SCI:  { category: 'SCIENCE',       color: '#059669', iconBg: 'rgba(5,150,105,0.1)',   iconColor: '#059669', visualBg: '#f0f7f5', blobColor: 'rgba(5,150,105,0.2)',  symbol: '⚗' },
  SS:   { category: 'SOCIAL STUDIES',color: '#ea580c', iconBg: 'rgba(234,88,12,0.1)',   iconColor: '#ea580c', visualBg: '#f7f4f0', blobColor: 'rgba(234,88,12,0.2)',  symbol: '⊕' },
  ICT:  { category: 'ICT',           color: '#7c3aed', iconBg: 'rgba(124,58,237,0.1)', iconColor: '#7c3aed', visualBg: '#f3f0f8', blobColor: 'rgba(124,58,237,0.2)', symbol: '⌘' },
  FR:   { category: 'FRENCH',        color: '#0891b2', iconBg: 'rgba(8,145,178,0.1)',   iconColor: '#0891b2', visualBg: '#f0f5f8', blobColor: 'rgba(8,145,178,0.2)',  symbol: 'Fr' },
  TWI:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: '◎' },
  RME:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: '◎' },
  BDT:  { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: '◎' },
  GA:   { category: 'GENERAL',       color: '#64748b', iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b', visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)', symbol: '◎' },
}

function cfg(code: string): SubjectCfg {
  return cfgMap[code] ?? {
    category: 'GENERAL', color: '#64748b',
    iconBg: 'rgba(100,116,139,0.1)', iconColor: '#64748b',
    visualBg: '#f5f5f3', blobColor: 'rgba(100,116,139,0.15)',
    symbol: '◎',
  }
}

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [subs, rdns] = await Promise.all([
      listSubjects(1),
      getReadinessReport(auth.currentAccount.id),
    ])
    subjects.value = subs
    readiness.value = rdns.subjects
    await Promise.all(
      subs.map(async (s) => {
        try {
          journeyMap.value[s.id] = await getActiveJourneyRoute(auth.currentAccount!.id, s.id)
        } catch {
          journeyMap.value[s.id] = null
        }
      })
    )
  } catch {
    error.value = 'Failed to load courses'
  }
  loading.value = false
})

function readinessFor(id: number) {
  return readiness.value.find(r => r.subject_id === id)
}
function completionPct(id: number): number | null {
  const r = readinessFor(id)
  if (!r || r.total_topic_count === 0) return null
  return Math.round((r.mastered_topic_count / r.total_topic_count) * 100)
}
function topicCount(id: number): number {
  return readinessFor(id)?.total_topic_count ?? 0
}
function sessionCount(id: number): number {
  const snap = journeyMap.value[id]
  return snap?.stations.filter(s => s.status === 'completed').length ?? 0
}
function attemptedCount(id: number): number {
  return sessionCount(id) * 10
}

const filtered = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return subjects.value
  return subjects.value.filter(s =>
    s.name.toLowerCase().includes(q) || s.code.toLowerCase().includes(q)
  )
})

const levelsPanelStyle = computed(() => {
  if (ui.isDark) {
    return {
      background: '#1e2130',
      borderColor: 'rgba(255,255,255,0.06)',
      boxShadow: '0 16px 48px rgba(0,0,0,0.5), 0 4px 12px rgba(0,0,0,0.3)',
    }
  }
  return {
    background: '#1e2130',
    borderColor: 'rgba(255,255,255,0.06)',
    boxShadow: '0 16px 48px rgba(0,0,0,0.5), 0 4px 12px rgba(0,0,0,0.3)',
  }
})

function openCourse(subject: SubjectDto) {
  const snap = journeyMap.value[subject.id]
  const active = snap?.stations.find(s => s.status === 'active')
  if (active) {
    router.push(`/student/journey/station/${active.id}`)
  } else {
    router.push('/student/practice/custom-test')
  }
}
</script>

<template>
  <div
    class="journey-home h-full flex flex-col overflow-hidden"
    :class="{ 'journey-home--dark': ui.isDark }"
    :style="{ backgroundColor: 'var(--paper)' }"
  >

    <!-- Header -->
    <div
      class="flex-shrink-0 px-8 pt-7 pb-5"
      :style="{ backgroundColor: 'var(--paper)' }"
    >
      <h1 class="font-display text-[22px] font-bold tracking-tight mb-0.5" :style="{ color: 'var(--ink)' }">
        Explore Courses
      </h1>
      <p class="text-[12px]" :style="{ color: 'var(--ink-muted)' }">
        {{ subjects.length }} courses in Junior High JHS 1
      </p>
    </div>

    <div v-if="error" class="px-8 py-2 text-xs flex-shrink-0"
      :style="{ background: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">{{ error }}</div>

    <!-- Body: scrollable content + floating levels panel -->
    <div class="flex-1 overflow-hidden relative">

      <!-- Scrollable course list -->
      <div class="h-full overflow-y-auto px-8 pb-8" style="padding-right: 248px">

        <!-- Search -->
        <div class="relative mb-5">
          <svg class="search-icon" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="9" cy="9" r="6" stroke="currentColor" stroke-width="1.5"/>
            <path d="M13.5 13.5L17 17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search courses..."
            class="search-input w-full"
          />
        </div>

        <!-- Skeleton -->
        <div v-if="loading" class="grid grid-cols-2 gap-3">
          <div v-for="i in 8" :key="i" class="h-28 rounded-2xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <!-- 2-column course cards -->
        <div v-else class="grid grid-cols-2 gap-3">
          <button
            v-for="subject in filtered"
            :key="subject.id"
            class="course-card"
            @click="openCourse(subject)"
          >
            <!-- LEFT: text content -->
            <div class="card-content">
              <!-- Subject icon -->
              <div class="subject-icon" :style="{ background: cfg(subject.code).iconBg }">
                <span :style="{ color: cfg(subject.code).iconColor }">{{ cfg(subject.code).symbol }}</span>
              </div>

              <!-- Info -->
              <div class="card-info">
                <p class="cat-label" :style="{ color: cfg(subject.code).color }">
                  {{ cfg(subject.code).category }}
                </p>
                <h3 class="course-name">{{ subject.name }}</h3>
                <div class="stats-row">
                  <span class="stat-item">
                    <span class="stat-icon">◫</span> {{ topicCount(subject.id) }} topics
                  </span>
                  <span class="stat-sep">·</span>
                  <span class="stat-item">
                    <span class="stat-icon">≡</span> {{ topicCount(subject.id) * 20 }} Q's
                  </span>
                  <template v-if="sessionCount(subject.id) > 0">
                    <span class="stat-sep">·</span>
                    <span class="stat-item">{{ sessionCount(subject.id) }} sessions</span>
                    <span class="stat-sep">·</span>
                    <span class="stat-item">{{ attemptedCount(subject.id) }} attempted</span>
                  </template>
                </div>
              </div>

              <!-- Completion % (large, prominent) -->
              <div v-if="completionPct(subject.id) !== null" class="completion-pct">
                {{ completionPct(subject.id) }}%
              </div>
            </div>

            <!-- RIGHT: frosted visual area -->
            <div
              class="card-visual"
              :style="{ backgroundColor: ui.isDark ? 'var(--paper)' : cfg(subject.code).visualBg }"
            >
              <div class="visual-blob" :style="{ background: cfg(subject.code).blobColor }" />
            </div>
          </button>
        </div>

        <div v-if="!loading && filtered.length === 0" class="py-16 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No courses found.</p>
        </div>
      </div>

      <!-- Floating LEVELS panel -->
      <div class="levels-panel" :style="levelsPanelStyle">
        <div class="levels-header">
          <p class="levels-title">LEVELS</p>
          <p class="levels-sub">{{ levels.length }} available</p>
        </div>
        <div class="levels-list">
          <button
            v-for="level in levels"
            :key="level.key"
            class="level-row"
            :class="{ active: activeLevel === level.key }"
            @click="activeLevel = level.key"
          >
            <span class="level-label">{{ level.label }}</span>
            <span class="level-badge">{{ level.count }}</span>
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* ── Search ── */
.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--ink-muted);
  pointer-events: none;
}

.search-input {
  padding: 9px 16px 9px 38px;
  border-radius: 10px;
  font-size: 13px;
  border: 1px solid transparent;
  background: var(--surface);
  color: var(--ink);
  outline: none;
}
.search-input::placeholder { color: var(--ink-muted); }

/* ── Course card ── */
.course-card {
  display: flex;
  height: 105px;
  border-radius: 14px;
  background: white;
  border: 1px solid var(--border-soft);
  overflow: hidden;
  cursor: pointer;
  text-align: left;
  transition: box-shadow 140ms ease, transform 140ms ease;
}
.course-card:hover {
  box-shadow: 0 4px 20px rgba(0,0,0,0.09);
  transform: translateY(-1px);
}

/* Left: content area */
.card-content {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 18px;
  min-width: 0;
}

.subject-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 900;
  flex-shrink: 0;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.cat-label {
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  margin-bottom: 2px;
}

.course-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--ink);
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stats-row {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: nowrap;
}

.stat-item {
  font-size: 10px;
  color: var(--ink-muted);
  display: flex;
  align-items: center;
  gap: 3px;
  white-space: nowrap;
}

.stat-icon {
  font-size: 9px;
  opacity: 0.6;
}

.stat-sep {
  font-size: 10px;
  color: var(--border-soft);
}

/* Completion % */
.completion-pct {
  font-size: 22px;
  font-weight: 800;
  color: #ea580c;
  flex-shrink: 0;
  line-height: 1;
}

/* Right: visual */
.card-visual {
  width: 120px;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
}

.visual-blob {
  position: absolute;
  width: 90px;
  height: 90px;
  border-radius: 50%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  filter: blur(22px);
}

/* ── Floating LEVELS panel ── */
.levels-panel {
  position: absolute;
  top: 12px;
  right: 12px;
  bottom: 12px;
  width: 210px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 14px;
  border: 1px solid transparent;
}

.levels-header {
  padding: 18px 20px 14px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
  flex-shrink: 0;
}

.levels-title {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.16em;
  color: rgba(255,255,255,0.9);
  text-transform: uppercase;
}

.levels-sub {
  font-size: 11px;
  color: rgba(255,255,255,0.4);
  margin-top: 2px;
}

.levels-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
  box-sizing: border-box;
}

/* Hide scrollbar for levels list */
.levels-list::-webkit-scrollbar { width: 0; }

.level-row {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 16px 9px 20px;
  font-size: 12px;
  font-weight: 500;
  color: rgba(255,255,255,0.55);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background-color 120ms, color 120ms;
  text-align: left;
  position: relative;
}
.level-row:hover {
  background: rgba(255,255,255,0.05);
  color: rgba(255,255,255,0.85);
}
.level-row.active {
  background: rgba(96,165,250,0.06);
  color: #60a5fa;
  border-radius: 8px;
  margin: 0 8px;
  width: calc(100% - 16px);
  padding-left: 14px;
}
.level-row.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 20%;
  bottom: 20%;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: #60a5fa;
}
.level-row.active .level-badge {
  background: rgba(96,165,250,0.12);
  color: #60a5fa;
}

.level-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.level-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: 999px;
  background: rgba(255,255,255,0.1);
  color: rgba(255,255,255,0.45);
  flex-shrink: 0;
  margin-left: 8px;
}
</style>
