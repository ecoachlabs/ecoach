<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, type SubjectDto } from '@/ipc/coach'
import { getReadinessReport, type SubjectReadinessDto } from '@/ipc/readiness'
import { buildLearnerTopicTree, flattenLearnerTopics } from '@/utils/learnerTopics'
import { startPracticeSessionWithQuestions } from '@/utils/sessionQuestions'
import { getReadinessColor, getReadinessLabel, isPositiveReadinessBand } from '@/utils/readiness'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const subjects = ref<SubjectDto[]>([])
const readiness = ref<SubjectReadinessDto[]>([])
const loading = ref(true)
const starting = ref<number | null>(null)
const selectedSubjectId = ref<number | null>(null)
const error = ref('')
const searchQuery = ref('')

const subjectConfig: Record<string, { category: string; color: string; bg: string; symbolBig: string }> = {
  MATH: { category: 'Mathematics', color: 'var(--gold)',         bg: 'rgba(180,83,9,0.07)',    symbolBig: 'M'  },
  ENG:  { category: 'English',     color: 'var(--accent)',       bg: 'rgba(13,148,136,0.07)',  symbolBig: 'Aa' },
  SCI:  { category: 'Science',     color: '#0891b2',             bg: 'rgba(8,145,178,0.07)',   symbolBig: 'S'  },
  SS:   { category: 'Social Stud.', color: 'var(--warm)',        bg: 'rgba(194,65,12,0.07)',   symbolBig: 'SS'  },
  ICT:  { category: 'ICT',         color: '#7c3aed',             bg: 'rgba(124,58,237,0.07)', symbolBig: 'PC'  },
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
    const requestedTopicId = Number(route.query.topic)
    if (Number.isFinite(requestedTopicId) && requestedTopicId > 0) {
      await openTopic(requestedTopicId)
    }
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

const practiceSummary = computed(() => {
  return readiness.value.reduce((summary, subject) => {
    summary.trackedTopics += subject.total_topic_count
    summary.masteredTopics += subject.mastered_topic_count
    summary.weakTopics += Math.max(0, subject.total_topic_count - subject.mastered_topic_count)
    if (isPositiveReadinessBand(subject.readiness_band)) {
      summary.strongSubjects += 1
    }
    return summary
  }, {
    trackedTopics: 0,
    masteredTopics: 0,
    weakTopics: 0,
    strongSubjects: 0,
  })
})

const subjectReadinessRows = computed(() => {
  return subjects.value
    .map(subject => {
      const readinessState = readinessFor(subject.id)
      return {
        id: subject.id,
        name: subject.name,
        band: readinessState?.readiness_band ?? 'not_ready',
        weakTopics: readinessState
          ? Math.max(0, readinessState.total_topic_count - readinessState.mastered_topic_count)
          : 0,
        totalTopics: readinessState?.total_topic_count ?? 0,
        pct: completionPct(subject.id) ?? 0,
      }
    })
    .sort((left, right) => {
      if (left.totalTopics === 0 && right.totalTopics > 0) return 1
      if (right.totalTopics === 0 && left.totalTopics > 0) return -1
      if (left.pct !== right.pct) return left.pct - right.pct
      return right.weakTopics - left.weakTopics
    })
})

async function open(subjectId: number) {
  if (!auth.currentAccount || starting.value !== null) return
  starting.value = subjectId
  error.value = ''
  try {
    const topics = flattenLearnerTopics(buildLearnerTopicTree(await listTopics(subjectId)))
    const topicIds = Array.from(new Set(topics.slice(0, 5).flatMap(topic => topic.sourceTopicIds)))
    if (topicIds.length === 0) throw new Error('No topics available')
    const fallbackTopicSets = topicIds.length > 1 ? [[topicIds[0]]] : []
    const session = await startPracticeSessionWithQuestions({
      student_id: auth.currentAccount.id,
      subject_id: subjectId,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: false,
    }, fallbackTopicSets)
    router.push(`/student/session/${session.sessionId}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start'
    starting.value = null
  }
}

async function openTopic(topicId: number) {
  if (!auth.currentAccount || starting.value !== null) return

  error.value = ''
  const resolvedSubjectId = await resolveSubjectIdForTopic(topicId)
  if (resolvedSubjectId == null) {
    error.value = 'That topic is not available yet.'
    return
  }

  starting.value = resolvedSubjectId
  selectedSubjectId.value = resolvedSubjectId

  try {
    const session = await startPracticeSessionWithQuestions({
      student_id: auth.currentAccount.id,
      subject_id: resolvedSubjectId,
      topic_ids: [topicId],
      question_count: 10,
      is_timed: false,
    })
    router.push(`/student/session/${session.sessionId}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start topic practice'
    starting.value = null
  }
}

async function resolveSubjectIdForTopic(topicId: number): Promise<number | null> {
  for (const subject of subjects.value) {
    const topics = await listTopics(subject.id).catch(() => [])
    if (topics.some(topic => topic.id === topicId)) {
      return subject.id
    }
  }
  return null
}
</script>

<template>
  <div class="practice-shell">
    <header class="practice-header">
      <div class="practice-header__copy">
        <p class="eyebrow">Explore</p>
        <h1 class="practice-title">Courses</h1>
        <p class="practice-subtitle">
          {{ subjects.length }} real courses · {{ practiceSummary.trackedTopics }} tracked topics
        </p>
      </div>

      <div class="practice-actions">
        <button class="nav-pill nav-pill--secondary" @click="router.push('/student/practice/custom-test')">
          Custom Test
        </button>
        <button class="nav-pill nav-pill--primary" @click="router.push('/student/progress/mastery-map')">
          Mastery Map
        </button>
      </div>
    </header>

    <section class="search-panel">
      <label class="search-label" for="course-search">Course Search</label>
      <div class="search-field">
        <svg class="search-icon" viewBox="0 0 20 20" fill="none" aria-hidden="true">
          <circle cx="9" cy="9" r="6" stroke="currentColor" stroke-width="1.5" />
          <path d="M13.5 13.5L17 17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <input
          id="course-search"
          v-model="searchQuery"
          type="text"
          placeholder="Search courses..."
          class="search-input"
        />
      </div>
    </section>

    <p v-if="error" class="inline-status inline-status--error">[ ERROR ] {{ error }}</p>

    <div class="practice-content">
      <section class="courses-pane">
        <div v-if="loading" class="courses-loading">
          <p class="loading-tag">[ LOADING COURSES ]</p>
          <p class="loading-copy">Building your practice map.</p>
        </div>

        <div v-else-if="filtered.length" class="courses-grid">
          <button
            v-for="subject in filtered"
            :key="subject.id"
            class="course-card"
            :disabled="starting !== null"
            @click="open(subject.id)"
          >
            <div class="course-card__content">
              <div class="course-card__copy">
                <p class="category-label" :style="{ color: subjectCfg(subject.code).color }">
                  {{ subjectCfg(subject.code).category }}
                </p>
                <h3 class="course-title">{{ subject.name }}</h3>
              </div>

              <div class="course-card__footer">
                <div class="course-chip-row">
                  <span v-if="topicCount(subject.id)" class="stat-chip">
                    {{ topicCount(subject.id) }} topics
                  </span>
                  <span class="stat-chip">10 Q's</span>
                  <span v-if="starting === subject.id" class="stat-chip stat-chip--muted">Starting...</span>
                </div>

                <div v-if="completionPct(subject.id) !== null" class="course-readiness">
                  <span class="course-readiness__label">Readiness</span>
                  <span class="course-readiness__value">{{ completionPct(subject.id) }}%</span>
                </div>
                <div v-else class="course-readiness course-readiness--muted">
                  <span class="course-readiness__label">Readiness</span>
                  <span class="course-readiness__value">--</span>
                </div>
              </div>
            </div>

            <div class="card-visual" :style="{ '--subject-bg': subjectCfg(subject.code).bg }">
              <span class="symbol-watermark" :style="{ color: subjectCfg(subject.code).color }">
                {{ subjectCfg(subject.code).symbolBig }}
              </span>
              <div v-if="completionPct(subject.id) !== null" class="completion-badge">
                <span class="completion-badge__label">Track</span>
                <span class="completion-badge__value">{{ completionPct(subject.id) }}%</span>
              </div>
            </div>
          </button>
        </div>

        <div v-else class="courses-empty">
          <p class="loading-tag">[ NO COURSES ]</p>
          <p class="loading-copy">No courses match "{{ searchQuery }}".</p>
        </div>
      </section>

      <aside class="signals-panel">
        <div class="signals-panel__header">
          <p class="practice-section-label">Practice Signals</p>
          <p class="signals-panel__sub">{{ practiceSummary.trackedTopics }} topics tracked</p>
        </div>

        <div class="signals-panel__metrics">
          <div class="practice-metric-card">
            <span class="practice-metric-label">Mastered</span>
            <strong class="practice-metric-value">{{ practiceSummary.masteredTopics }}</strong>
          </div>
          <div class="practice-metric-card">
            <span class="practice-metric-label">Weak</span>
            <strong class="practice-metric-value">{{ practiceSummary.weakTopics }}</strong>
          </div>
          <div class="practice-metric-card">
            <span class="practice-metric-label">Strong</span>
            <strong class="practice-metric-value">{{ practiceSummary.strongSubjects }}</strong>
          </div>
          <div class="practice-metric-card">
            <span class="practice-metric-label">Courses</span>
            <strong class="practice-metric-value">{{ subjects.length }}</strong>
          </div>
        </div>

        <div class="signals-panel__body">
          <p class="practice-section-label">Subject readiness</p>
          <div v-if="subjectReadinessRows.length" class="signals-list">
            <div
              v-for="subject in subjectReadinessRows"
              :key="subject.id"
              class="practice-row"
            >
              <div class="practice-row__copy">
                <p class="practice-row-title">{{ subject.name }}</p>
                <p class="practice-row-sub" :style="{ color: getReadinessColor(subject.band) }">
                  {{ getReadinessLabel(subject.band) }} · {{ subject.weakTopics }} weak
                </p>
              </div>
              <span class="practice-row-pct" :style="{ color: getReadinessColor(subject.band) }">{{ subject.pct }}%</span>
            </div>
          </div>
          <p v-else class="practice-empty">[ NO SNAPSHOT ]</p>
        </div>

        <button class="panel-cta" @click="router.push('/student/practice/custom-test')">
          Custom Test
        </button>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.practice-shell {
  --nothing-bg: #f5f5f5;
  --nothing-surface: #ffffff;
  --nothing-surface-muted: #f0f0f0;
  --nothing-border: #e8e8e8;
  --nothing-border-strong: #cccccc;
  --nothing-text: #111111;
  --nothing-secondary: #666666;
  --nothing-disabled: #999999;
  --nothing-accent: #d71921;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 32px;
  background: var(--nothing-bg);
  color: var(--nothing-text);
  font-family: 'Space Grotesk', var(--font-body);
}

.practice-header {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 24px;
}

.practice-header__copy {
  display: grid;
  gap: 6px;
}

.eyebrow {
  margin: 0;
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.practice-title {
  margin: 0;
  font-size: 36px;
  line-height: 1.05;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: #000000;
}

.practice-subtitle {
  margin: 0;
  font-size: 14px;
  color: var(--nothing-secondary);
}

.practice-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.nav-pill {
  min-height: 44px;
  padding: 12px 18px;
  border-radius: 999px;
  border: 1px solid var(--nothing-border-strong);
  background: transparent;
  color: var(--nothing-text);
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  cursor: pointer;
  transition: border-color 160ms ease, background-color 160ms ease, color 160ms ease;
}
.nav-pill:hover { border-color: #000000; }

.nav-pill--primary {
  border-color: #000000;
  background: #000000;
  color: #ffffff;
}

.search-panel {
  display: grid;
  gap: 8px;
  max-width: 720px;
}

.search-label,
.practice-section-label,
.category-label,
.practice-metric-label,
.course-readiness__label,
.completion-badge__label,
.practice-row-sub,
.loading-tag {
  font-family: 'Space Mono', monospace;
}

.search-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.search-field {
  position: relative;
}

.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  width: 18px;
  height: 18px;
  transform: translateY(-50%);
  color: var(--nothing-secondary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  min-height: 52px;
  padding: 0 18px 0 44px;
  border-radius: 8px;
  border: 1px solid var(--nothing-border-strong);
  background: var(--nothing-surface);
  color: var(--nothing-text);
  font-family: 'Space Mono', monospace;
  font-size: 14px;
  outline: none;
  transition: border-color 160ms ease;
}
.search-input::placeholder { color: var(--nothing-disabled); }
.search-input:focus { border-color: #000000; }

.inline-status {
  margin: 0;
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.inline-status--error {
  color: var(--nothing-accent);
}

.practice-content {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: 24px;
  align-items: start;
}

.courses-pane {
  min-height: 0;
}

.courses-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.courses-loading,
.courses-empty {
  min-height: 280px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 8px;
  padding: 32px;
  border: 1px solid var(--nothing-border);
  border-radius: 12px;
  background: var(--nothing-surface);
}

.loading-tag {
  margin: 0;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.loading-copy {
  margin: 0;
  font-size: 14px;
  color: var(--nothing-secondary);
}

.course-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 116px;
  min-height: 172px;
  border-radius: 14px;
  border: 1px solid var(--nothing-border);
  background: var(--nothing-surface);
  cursor: pointer;
  overflow: hidden;
  text-align: left;
  transition: border-color 160ms ease, background-color 160ms ease;
}
.course-card:hover:not(:disabled) {
  border-color: #000000;
  background: #fcfcfc;
}
.course-card:disabled { opacity: 0.55; cursor: not-allowed; }

.course-card__content {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 20px;
  padding: 20px;
}

.course-card__copy {
  display: grid;
  gap: 10px;
}

.category-label {
  margin: 0;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.course-title {
  margin: 0;
  font-size: 20px;
  line-height: 1.15;
  font-weight: 500;
  color: #000000;
}

.course-card__footer {
  display: grid;
  gap: 14px;
}

.course-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.stat-chip {
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--nothing-border-strong);
  background: transparent;
  color: var(--nothing-secondary);
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.stat-chip--muted {
  color: var(--nothing-disabled);
}

.course-readiness {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding-top: 14px;
  border-top: 1px solid var(--nothing-border);
}

.course-readiness__label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.course-readiness__value {
  font-family: 'Space Mono', monospace;
  font-size: 18px;
  font-weight: 700;
  color: #000000;
}

.course-readiness--muted .course-readiness__value {
  color: var(--nothing-disabled);
}

.card-visual {
  display: flex;
  align-items: flex-end;
  justify-content: flex-start;
  position: relative;
  overflow: hidden;
  padding: 16px;
  border-left: 1px solid var(--nothing-border);
  background:
    linear-gradient(180deg, rgba(0, 0, 0, 0.03), transparent 60%),
    var(--subject-bg, var(--nothing-surface-muted));
}

.card-visual::before {
  content: '';
  position: absolute;
  inset: 0;
  opacity: 0.18;
  pointer-events: none;
  background-image: radial-gradient(circle at 1px 1px, rgba(0, 0, 0, 0.16) 1px, transparent 0);
  background-size: 14px 14px;
}

.symbol-watermark {
  position: relative;
  z-index: 1;
  font-family: 'Space Mono', monospace;
  font-size: 48px;
  font-weight: 700;
  opacity: 0.34;
  user-select: none;
  line-height: 1;
}

.completion-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 1;
  display: grid;
  gap: 2px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: rgba(255, 255, 255, 0.86);
}

.completion-badge__label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.completion-badge__value {
  font-family: 'Space Mono', monospace;
  font-size: 16px;
  font-weight: 700;
  color: #000000;
}

.signals-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 20px;
  border: 1px solid #000000;
  border-radius: 14px;
  background: var(--nothing-surface);
}

.signals-panel__header {
  display: grid;
  gap: 6px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--nothing-border);
}

.signals-panel__sub {
  margin: 0;
  font-size: 14px;
  color: var(--nothing-secondary);
}

.signals-panel__metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.practice-metric-card {
  padding: 12px;
  border-radius: 10px;
  background: var(--nothing-surface-muted);
  border: 1px solid var(--nothing-border);
}

.practice-metric-label {
  display: block;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.practice-metric-value {
  display: block;
  margin-top: 10px;
  font-family: 'Space Mono', monospace;
  font-size: 28px;
  line-height: 1;
  font-weight: 700;
  color: #000000;
}

.practice-section-label {
  margin: 0;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-secondary);
}

.signals-panel__body {
  display: grid;
  gap: 12px;
}

.practice-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--nothing-border);
}

.signals-list .practice-row:last-child {
  border-bottom: none;
}

.practice-row__copy {
  min-width: 0;
}

.practice-row-title {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--nothing-text);
}

.practice-row-sub {
  margin: 3px 0 0;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.practice-row-pct {
  flex-shrink: 0;
  font-family: 'Space Mono', monospace;
  font-size: 18px;
  font-weight: 700;
}

.practice-empty {
  margin: 0;
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--nothing-disabled);
}

.panel-cta {
  margin-top: auto;
  min-height: 44px;
  border-radius: 999px;
  border: 1px solid #000000;
  background: #000000;
  color: #ffffff;
  font-family: 'Space Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  cursor: pointer;
}

@media (max-width: 1120px) {
  .practice-content {
    grid-template-columns: 1fr;
  }

  .signals-panel {
    order: -1;
  }
}

@media (max-width: 720px) {
  .practice-shell {
    padding: 20px;
  }

  .practice-header {
    align-items: stretch;
    flex-direction: column;
  }

  .course-card {
    grid-template-columns: 1fr;
  }

  .card-visual {
    min-height: 112px;
    border-left: none;
    border-top: 1px solid var(--nothing-border);
  }
}
</style>

