<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import { listSubjects, listTopics, type SubjectDto } from '@/ipc/coach'
import { getReadinessReport, type SubjectReadinessDto } from '@/ipc/readiness'
import { startPracticeSession } from '@/ipc/sessions'

const auth = useAuthStore()
const ui = useUiStore()
const router = useRouter()

const subjects = ref<SubjectDto[]>([])
const readiness = ref<SubjectReadinessDto[]>([])
const loading = ref(true)
const starting = ref<number | null>(null)
const error = ref('')
const searchQuery = ref('')
const activeLevel = ref('JHS 1')

const levels = [
  { label: 'Junior High JHS 1', key: 'JHS 1', count: 11 },
  { label: 'Junior High JHS 2', key: 'JHS 2', count: 10 },
  { label: 'Junior High JHS 3', key: 'JHS 3', count: 9 },
  { label: 'Primary 1', key: 'P1', count: 8 },
  { label: 'Primary 2', key: 'P2', count: 9 },
  { label: 'Primary 3', key: 'P3', count: 10 },
  { label: 'Primary 4', key: 'P4', count: 11 },
  { label: 'Primary 5', key: 'P5', count: 11 },
  { label: 'Primary 6', key: 'P6', count: 11 },
  { label: 'Senior High SHS 1', key: 'SHS 1', count: 9 },
  { label: 'Senior High SHS 2', key: 'SHS 2', count: 8 },
  { label: 'Senior High SHS 3', key: 'SHS 3', count: 7 },
  { label: 'BECE', key: 'BECE', count: 24 },
  { label: 'WASSCE SHS', key: 'WASSCE', count: 30 },
  { label: 'BECE Mocks', key: 'BECE Mocks', count: 7 },
]

const subjectConfig: Record<string, { category: string; color: string; bg: string; symbolBig: string }> = {
  MATH: { category: 'Mathematics', color: 'var(--gold)',         bg: 'rgba(180,83,9,0.07)',    symbolBig: 'Σ'  },
  ENG:  { category: 'English',     color: 'var(--accent)',       bg: 'rgba(13,148,136,0.07)',  symbolBig: 'Aa' },
  SCI:  { category: 'Science',     color: '#0891b2',             bg: 'rgba(8,145,178,0.07)',   symbolBig: '⚛'  },
  SS:   { category: 'Social Stud.', color: 'var(--warm)',        bg: 'rgba(194,65,12,0.07)',   symbolBig: '⊕'  },
  ICT:  { category: 'ICT',         color: '#7c3aed',             bg: 'rgba(124,58,237,0.07)', symbolBig: '⌘'  },
  FR:   { category: 'French',      color: '#be185d',             bg: 'rgba(190,24,93,0.07)',   symbolBig: 'Fr' },
  TWI:  { category: 'General',     color: 'var(--ink-secondary)', bg: 'rgba(92,86,80,0.07)',  symbolBig: 'Tw' },
  RME:  { category: 'General',     color: 'var(--ink-secondary)', bg: 'rgba(92,86,80,0.07)',  symbolBig: 'Re' },
  BDT:  { category: 'General',     color: 'var(--ink-secondary)', bg: 'rgba(92,86,80,0.07)',  symbolBig: 'Bd' },
  GA:   { category: 'General',     color: 'var(--ink-secondary)', bg: 'rgba(92,86,80,0.07)',  symbolBig: 'Ga' },
}

function subjectCfg(code: string) {
  return subjectConfig[code] ?? {
    category: 'General',
    color: 'var(--ink-secondary)',
    bg: 'rgba(92,86,80,0.07)',
    symbolBig: code.slice(0, 2),
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
  } catch {
    error.value = 'Failed to load courses'
  }
  loading.value = false
})

function readinessFor(subjectId: number): SubjectReadinessDto | undefined {
  return readiness.value.find(r => r.subject_id === subjectId)
}

function completionPct(subjectId: number): number | null {
  const r = readinessFor(subjectId)
  if (!r || r.total_topic_count === 0) return null
  return Math.round((r.mastered_topic_count / r.total_topic_count) * 100)
}

function topicCount(subjectId: number): number {
  return readinessFor(subjectId)?.total_topic_count ?? 0
}

const filtered = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return subjects.value
  return subjects.value.filter(s =>
    s.name.toLowerCase().includes(q) || s.code.toLowerCase().includes(q)
  )
})

const levelsPanelStyle = computed(() => {
  return {
    background: '#1e2130',
    borderColor: 'rgba(255,255,255,0.06)',
    boxShadow: '0 16px 48px rgba(0,0,0,0.5), 0 4px 12px rgba(0,0,0,0.3)',
  }
})

async function open(subjectId: number) {
  if (!auth.currentAccount || starting.value !== null) return
  starting.value = subjectId
  error.value = ''
  try {
    const topics = await listTopics(subjectId)
    const topicIds = topics.slice(0, 5).map(t => t.id)
    if (topicIds.length === 0) throw new Error('No topics available')
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: subjectId,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start'
    starting.value = null
  }
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between mb-4">
        <div>
          <p class="eyebrow">Explore</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Courses
          </h1>
          <p class="text-xs mt-0.5" :style="{ color: 'var(--ink-muted)' }">
            {{ subjects.length }} courses in Junior High JHS 1
          </p>
        </div>
        <div class="flex gap-2">
          <button class="nav-pill" @click="router.push('/student/practice/custom-test')">Custom Test</button>
          <button class="nav-pill" @click="router.push('/student/progress/mastery')">Mastery Map</button>
        </div>
      </div>

      <!-- Search -->
      <div class="relative">
        <span class="search-icon">⌕</span>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search courses…"
          class="search-input w-full"
        />
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      :style="{ background: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Course grid -->
      <div class="flex-1 overflow-y-auto p-6">

        <!-- Skeleton -->
        <div v-if="loading" class="grid grid-cols-2 gap-4">
          <div v-for="i in 8" :key="i" class="h-32 rounded-2xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <!-- Cards -->
        <div v-else class="grid grid-cols-2 gap-4">
          <button
            v-for="subject in filtered"
            :key="subject.id"
            class="course-card text-left"
            :disabled="starting !== null"
            @click="open(subject.id)"
          >
            <!-- Left: text -->
            <div class="card-text flex-1 flex flex-col justify-between py-5 pl-5 pr-3">
              <div>
                <p class="category-label mb-1.5"
                  :style="{ color: subjectCfg(subject.code).color }">
                  {{ subjectCfg(subject.code).category }}
                </p>
                <h3 class="text-[13px] font-bold leading-snug mb-3"
                  :style="{ color: 'var(--ink)' }">
                  {{ subject.name }}
                </h3>
              </div>
              <div class="flex items-center gap-2 flex-wrap">
                <span v-if="topicCount(subject.id)" class="stat-chip">
                  {{ topicCount(subject.id) }} topics
                </span>
                <span class="stat-chip">10 Q's</span>
                <span v-if="starting === subject.id"
                  class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                  Starting…
                </span>
              </div>
            </div>

            <!-- Right: visual -->
            <div class="card-visual" :style="{ background: subjectCfg(subject.code).bg }">
              <span class="symbol-watermark"
                :style="{ color: subjectCfg(subject.code).color }">
                {{ subjectCfg(subject.code).symbolBig }}
              </span>
              <div v-if="completionPct(subject.id) !== null" class="completion-badge">
                {{ completionPct(subject.id) }}%
              </div>
            </div>
          </button>
        </div>

        <div v-if="!loading && filtered.length === 0" class="py-16 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No courses match your search.</p>
        </div>
      </div>

      <!-- Right: dark Levels sidebar -->
      <div
        class="levels-panel flex-shrink-0 w-52 flex flex-col overflow-hidden"
        :style="levelsPanelStyle"
      >
        <div class="px-5 py-4 flex-shrink-0" style="border-bottom: 1px solid rgba(255,255,255,0.07)">
          <p class="text-[10px] font-bold uppercase tracking-widest"
            style="color: rgba(255,255,255,0.35)">Levels</p>
          <p class="text-[11px] font-semibold mt-0.5"
            style="color: rgba(255,255,255,0.5)">{{ levels.length }} available</p>
        </div>
        <div class="flex-1 overflow-y-auto py-1.5">
          <button
            v-for="level in levels"
            :key="level.key"
            class="level-item w-full text-left"
            :class="{ active: activeLevel === level.key }"
            @click="activeLevel = level.key"
          >
            <span class="flex-1 truncate">{{ level.label }}</span>
            <span class="level-count">{{ level.count }}</span>
          </button>
        </div>
        <div class="px-4 py-4 flex-shrink-0" style="border-top: 1px solid rgba(255,255,255,0.07)">
          <button
            class="w-full py-2.5 rounded-xl text-[11px] font-bold"
            style="background: var(--accent); color: white; border: none; cursor: pointer;"
            @click="router.push('/student/practice/custom-test')"
          >
            Custom Test →
          </button>
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

.nav-pill {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 15px;
  color: var(--ink-muted);
  pointer-events: none;
}

.search-input {
  padding: 10px 16px 10px 38px;
  border-radius: 12px;
  font-size: 13px;
  border: 1px solid transparent;
  background: var(--paper);
  color: var(--ink);
  outline: none;
  transition: border-color 120ms;
}
.search-input::placeholder { color: var(--ink-muted); }
.search-input:focus { border-color: var(--accent); }

/* Course card: horizontal split */
.course-card {
  display: flex;
  border-radius: 18px;
  border: 1px solid transparent;
  background: var(--surface);
  cursor: pointer;
  min-height: 120px;
  overflow: hidden;
  transition: border-color 130ms ease, transform 130ms ease, box-shadow 130ms ease;
}
.course-card:hover:not(:disabled) {
  border-color: var(--ink-muted);
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}
.course-card:disabled { opacity: 0.55; cursor: not-allowed; }

.category-label {
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.14em;
}

.stat-chip {
  font-size: 9px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

/* Right visual panel of card */
.card-visual {
  width: 110px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.symbol-watermark {
  font-size: 56px;
  font-weight: 900;
  opacity: 0.16;
  user-select: none;
  line-height: 1;
}

.completion-badge {
  position: absolute;
  bottom: 10px;
  right: 10px;
  font-size: 12px;
  font-weight: 800;
  color: var(--gold);
  background: rgba(180,83,9,0.12);
  border-radius: 8px;
  padding: 3px 8px;
  border: 1px solid rgba(180,83,9,0.2);
}

/* Dark levels sidebar */
.levels-panel {
  border: 1px solid transparent;
  box-shadow: none;
}

.level-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  font-size: 11px;
  font-weight: 500;
  color: rgba(255,255,255,0.45);
  cursor: pointer;
  transition: all 100ms;
  border: none;
  background: transparent;
}
.level-item:hover {
  background: rgba(255,255,255,0.05);
  color: rgba(255,255,255,0.8);
}
.level-item.active {
  background: rgba(13,148,136,0.14);
  color: var(--accent);
}
.level-item.active .level-count {
  background: rgba(13,148,136,0.25);
  color: var(--accent);
}

.level-count {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(255,255,255,0.07);
  color: rgba(255,255,255,0.35);
  flex-shrink: 0;
}
</style>
