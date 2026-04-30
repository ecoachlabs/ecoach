<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  buildTeachActionPlan,
  getTeachLesson,
  recordGlossaryInteraction,
  type TeachActionPlanDto,
  type TeachLessonDto,
  type TeachMicroCheckDto,
} from '@/ipc/library'
import { listSubjects, listTopics } from '@/ipc/coach'

const props = defineProps<{
  topicId: string | number
}>()

interface ContentGroup {
  title: string | null
  lines: string[]
}

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const lessonLoading = ref(false)
const error = ref('')
const plan = ref<TeachActionPlanDto | null>(null)
const lesson = ref<TeachLessonDto | null>(null)
const subjectId = ref<number | null>(null)
const subjectName = ref<string | null>(null)
const selectedLevel = ref<'foundation' | 'core' | 'exam' | 'advanced'>('core')
const selectedAnswers = ref<Record<number, string>>({})
const revealedChecks = ref<Record<number, boolean>>({})
const optionCache = ref<Record<number, string[]>>({})

const levelOptions = [
  { key: 'foundation', label: 'Foundation' },
  { key: 'core', label: 'Core' },
  { key: 'exam', label: 'Exam' },
  { key: 'advanced', label: 'Advanced' },
] as const

const teachSections = computed(() => {
  const explanation = lesson.value?.explanation
  if (!explanation) return []

  return [
    { key: 'hero', title: 'Hero Summary', groups: toGroups(explanation.hero_summary) },
    { key: 'why', title: 'Why It Matters', groups: toGroups(explanation.why_it_matters) },
    { key: 'simple', title: 'Simple Explanation', groups: toGroups(explanation.simple_explanation) },
    { key: 'breakdown', title: 'Structured Breakdown', groups: toGroups(explanation.structured_breakdown) },
    { key: 'examples', title: 'Worked Examples', groups: toGroups(explanation.worked_examples) },
    { key: 'mistakes', title: 'Common Mistakes', groups: toGroups(explanation.common_mistakes) },
    { key: 'exam-notes', title: 'Exam Appearance', groups: toGroups(explanation.exam_appearance_notes) },
    { key: 'patterns', title: 'Pattern Recognition', groups: toGroups(explanation.pattern_recognition_tips) },
    { key: 'related', title: 'Related Concepts', groups: toGroups(explanation.related_concepts) },
  ].filter(section => section.groups.length > 0)
})

const masteryPercent = computed(() => {
  if (!plan.value) return 0
  return Math.max(0, Math.min(100, Math.round(plan.value.mastery_score / 100)))
})

const gapPercent = computed(() => {
  if (!plan.value) return 0
  return Math.max(0, Math.min(100, Math.round(plan.value.gap_score / 100)))
})

const linkedEntryIds = computed(() => plan.value?.linked_entry_ids.slice(0, 6) ?? [])

onMounted(() => {
  void loadTeachMode()
})

watch(selectedLevel, () => {
  if (!loading.value) {
    void loadLesson()
  }
})

async function loadTeachMode() {
  const studentId = auth.currentAccount?.id
  const topicId = numericTopicId()

  if (!studentId || topicId == null) {
    error.value = 'Teach mode needs an active student and a valid topic.'
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    const [actionPlan] = await Promise.all([
      buildTeachActionPlan(studentId, topicId, 8),
      resolveTopicSubject(topicId),
    ])
    plan.value = actionPlan
    await loadLesson()
  } catch (cause: unknown) {
    error.value = toMessage(cause, 'Failed to load teach mode')
  } finally {
    loading.value = false
  }
}

async function loadLesson() {
  const topicId = numericTopicId()
  if (topicId == null) return

  lessonLoading.value = true
  error.value = ''

  try {
    lesson.value = await getTeachLesson(topicId, selectedLevel.value, 4)
    selectedAnswers.value = {}
    revealedChecks.value = {}
    optionCache.value = Object.fromEntries(
      (lesson.value.micro_checks ?? []).map(check => [check.id, buildOptions(check)]),
    )
    safeRecordInteraction({
      student_id: auth.currentAccount?.id ?? null,
      event_type: 'teach_lesson_opened',
      metadata: {
        topic_id: topicId,
        level: selectedLevel.value,
        generated: lesson.value.generated,
        micro_check_count: lesson.value.micro_checks.length,
      },
    })
  } catch (cause: unknown) {
    error.value = toMessage(cause, 'Failed to load this lesson layer')
  } finally {
    lessonLoading.value = false
  }
}

async function resolveTopicSubject(topicId: number) {
  subjectId.value = null
  subjectName.value = null

  const subjects = await listSubjects(1)
  for (const subject of subjects) {
    const topics = await listTopics(subject.id)
    if (topics.some(topic => topic.id === topicId)) {
      subjectId.value = subject.id
      subjectName.value = subject.name
      return
    }
  }
}

function numericTopicId() {
  const value = Number(props.topicId)
  return Number.isFinite(value) ? value : null
}

function toGroups(value: unknown): ContentGroup[] {
  if (value == null) return []

  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    const text = String(value).trim()
    return text ? [{ title: null, lines: [text] }] : []
  }

  if (Array.isArray(value)) {
    const lines = value.flatMap(flattenLines).filter(Boolean)
    return lines.length ? [{ title: null, lines }] : []
  }

  if (typeof value === 'object') {
    return Object.entries(value as Record<string, unknown>)
      .map(([key, raw]) => ({
        title: humanize(key),
        lines: flattenLines(raw),
      }))
      .filter(group => group.lines.length > 0)
  }

  return []
}

function flattenLines(value: unknown): string[] {
  if (value == null) return []

  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    const text = String(value).trim()
    return text ? [text] : []
  }

  if (Array.isArray(value)) {
    return value.flatMap(flattenLines)
  }

  if (typeof value === 'object') {
    return Object.entries(value as Record<string, unknown>)
      .flatMap(([key, raw]) => {
        const leaf = flattenLines(raw)
        if (!leaf.length) return []
        if (leaf.length === 1) return [`${humanize(key)}: ${leaf[0]}`]
        return [humanize(key), ...leaf.map(line => `- ${line}`)]
      })
  }

  return []
}

function buildOptions(check: TeachMicroCheckDto) {
  const unique = Array.from(
    new Set([check.correct_answer, ...check.distractor_answers].map(text => text.trim()).filter(Boolean)),
  )
  if (unique.length <= 1) return unique
  const shift = check.position_index % unique.length
  return [...unique.slice(shift), ...unique.slice(0, shift)]
}

function revealCheck(check: TeachMicroCheckDto) {
  if (!selectedAnswers.value[check.id]) return
  revealedChecks.value[check.id] = true
  safeRecordInteraction({
    student_id: auth.currentAccount?.id ?? null,
    event_type: 'teach_micro_check_checked',
    metadata: {
      topic_id: numericTopicId(),
      level: selectedLevel.value,
      check_id: check.id,
      check_type: check.check_type,
      selected_answer: selectedAnswers.value[check.id],
      correct_answer: check.correct_answer,
      is_correct: isCorrect(check),
    },
  })
}

function isCorrect(check: TeachMicroCheckDto) {
  const answer = selectedAnswers.value[check.id] ?? ''
  return answer.trim().toLowerCase() === check.correct_answer.trim().toLowerCase()
}

function humanize(value: string) {
  return value.replace(/_/g, ' ').replace(/\b\w/g, letter => letter.toUpperCase())
}

function toMessage(cause: unknown, fallback: string) {
  return typeof cause === 'string'
    ? cause
    : cause instanceof Error
      ? cause.message
      : fallback
}

function safeRecordInteraction(
  input: Parameters<typeof recordGlossaryInteraction>[0],
) {
  void recordGlossaryInteraction(input).catch(() => {})
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pt-6 pb-5"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-start justify-between gap-6">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <button class="back-btn" @click="router.push('/student/curriculum')">Back</button>
            <p class="eyebrow">Teach Mode</p>
          </div>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            {{ plan?.topic_name ?? lesson?.topic_name ?? 'Topic Lesson' }}
          </h1>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
            {{ subjectName ?? 'Curriculum topic' }} · {{ plan?.readiness_band ?? 'loading readiness' }} ·
            {{ plan?.support_intensity ?? 'adaptive support' }}
          </p>
        </div>

        <div class="flex flex-wrap justify-end gap-1.5">
          <button
            v-for="option in levelOptions"
            :key="option.key"
            class="level-chip"
            :class="{ active: selectedLevel === option.key }"
            @click="selectedLevel = option.key"
          >
            {{ option.label }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="index in 4" :key="index" class="h-28 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 flex overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div class="grid gap-4 md:grid-cols-[1.2fr_0.8fr]">
          <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
            <p class="section-label mb-3">Teaching Prompt</p>
            <p class="text-sm leading-6" :style="{ color: 'var(--ink)' }">
              {{ plan?.primary_prompt ?? 'The coach is preparing this topic now.' }}
            </p>

            <div class="mt-5 grid gap-4 md:grid-cols-2">
              <div>
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] font-semibold uppercase tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">
                    Mastery
                  </span>
                  <strong class="text-sm" :style="{ color: 'var(--accent)' }">{{ masteryPercent }}%</strong>
                </div>
                <div class="meter-track">
                  <div class="meter-fill accent" :style="{ width: `${masteryPercent}%` }" />
                </div>
              </div>

              <div>
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] font-semibold uppercase tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">
                    Gap Pressure
                  </span>
                  <strong class="text-sm" :style="{ color: 'var(--warm)' }">{{ gapPercent }}%</strong>
                </div>
                <div class="meter-track">
                  <div class="meter-fill warm" :style="{ width: `${gapPercent}%` }" />
                </div>
              </div>
            </div>
          </section>

          <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
            <p class="section-label mb-3">Signals</p>
            <div class="space-y-3">
              <div>
                <p class="mini-label">Diagnostic Focus</p>
                <div class="flex flex-wrap gap-2 mt-2">
                  <span v-for="focus in plan?.diagnostic_focuses ?? []" :key="focus" class="signal-pill">
                    {{ focus }}
                  </span>
                </div>
              </div>
              <div v-if="plan?.recent_diagnoses.length">
                <p class="mini-label">Recent Diagnoses</p>
                <ul class="mt-2 space-y-2">
                  <li v-for="diagnosis in plan.recent_diagnoses" :key="diagnosis" class="signal-line">
                    {{ diagnosis }}
                  </li>
                </ul>
              </div>
            </div>
          </section>
        </div>

        <section
          v-for="section in teachSections"
          :key="section.key"
          class="rounded-2xl border p-5"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">{{ section.title }}</p>
              <p v-if="lesson?.node_title" class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
                {{ lesson.node_title }}
              </p>
            </div>
            <span v-if="lessonLoading" class="text-[11px] font-semibold" :style="{ color: 'var(--ink-muted)' }">
              Refreshing layer...
            </span>
          </div>

          <div class="space-y-4">
            <div
              v-for="(group, index) in section.groups"
              :key="`${section.key}-${group.title ?? index}`"
              class="space-y-2"
            >
              <p v-if="group.title" class="mini-label">{{ group.title }}</p>
              <ul class="space-y-2">
                <li
                  v-for="line in group.lines"
                  :key="`${section.key}-${line}`"
                  class="content-line"
                >
                  {{ line }}
                </li>
              </ul>
            </div>
          </div>
        </section>

        <section
          v-if="lesson?.micro_checks.length"
          class="rounded-2xl border p-5"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">Micro Checks</p>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
                Short checks pulled from the live lesson.
              </p>
            </div>
          </div>

          <div class="space-y-5">
            <div
              v-for="check in lesson.micro_checks"
              :key="check.id"
              class="rounded-2xl border p-4"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }"
            >
              <p class="text-sm font-semibold leading-6" :style="{ color: 'var(--ink)' }">{{ check.prompt }}</p>

              <div class="mt-3 space-y-2">
                <label
                  v-for="option in optionCache[check.id] ?? []"
                  :key="`${check.id}-${option}`"
                  class="option-row"
                >
                  <input v-model="selectedAnswers[check.id]" :value="option" type="radio" class="sr-only" />
                  <span class="option-dot" :class="{ active: selectedAnswers[check.id] === option }" />
                  <span class="text-sm" :style="{ color: 'var(--ink)' }">{{ option }}</span>
                </label>
              </div>

              <div class="mt-4 flex items-center gap-3">
                <button class="primary-btn" :disabled="!selectedAnswers[check.id]" @click="revealCheck(check)">
                  Check Answer
                </button>
                <span class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ humanize(check.check_type) }}
                </span>
              </div>

              <div
                v-if="revealedChecks[check.id]"
                class="mt-4 rounded-xl px-4 py-3 text-sm"
                :style="isCorrect(check)
                  ? { backgroundColor: 'rgba(13,148,136,0.08)', color: 'var(--accent)' }
                  : { backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }"
              >
                <p class="font-semibold">
                  {{ isCorrect(check) ? 'Correct' : `Not quite. Correct answer: ${check.correct_answer}` }}
                </p>
                <p v-if="!isCorrect(check) && check.explanation_if_wrong" class="mt-1 text-[13px] leading-6">
                  {{ check.explanation_if_wrong }}
                </p>
              </div>
            </div>
          </div>
        </section>
      </div>

      <aside
        class="w-[340px] flex-shrink-0 border-l overflow-y-auto p-5 space-y-5"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Recommended Sequence</p>
          <div v-if="plan?.recommended_sequence.length" class="space-y-3">
            <div v-for="step in plan.recommended_sequence" :key="step.sequence_no" class="sequence-row">
              <div class="sequence-no">{{ step.sequence_no }}</div>
              <div class="min-w-0">
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ step.title }}</p>
                <p class="text-[11px] mt-1 leading-5" :style="{ color: 'var(--ink-muted)' }">{{ step.prompt }}</p>
              </div>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">
            The coach has not added a sequence for this topic yet.
          </p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Relationship Hints</p>
          <div v-if="plan?.relationship_hints.length" class="space-y-3">
            <div v-for="hint in plan.relationship_hints" :key="`${hint.from_title}-${hint.to_title}`">
              <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">
                {{ hint.from_title }} → {{ hint.to_title }}
              </p>
              <p class="text-[11px] mt-1 leading-5" :style="{ color: 'var(--ink-muted)' }">
                {{ hint.explanation }}
              </p>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">
            No relationship cues yet for this topic.
          </p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Linked Entries</p>
          <div v-if="linkedEntryIds.length" class="space-y-2">
            <button
              v-for="entryId in linkedEntryIds"
              :key="entryId"
              class="link-row"
              @click="router.push(`/student/glossary/entry/${entryId}`)"
            >
              <span class="font-semibold" :style="{ color: 'var(--ink)' }">Entry {{ entryId }}</span>
              <span :style="{ color: 'var(--accent)' }">Open</span>
            </button>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">
            No glossary entries are linked yet.
          </p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Next Move</p>
          <div class="space-y-2">
            <button class="sidebar-btn" @click="router.push('/student/practice')">Open Practice</button>
            <button class="sidebar-btn" @click="router.push('/student/library')">Open Library</button>
            <button v-if="subjectId" class="sidebar-btn" @click="router.push('/student/exam-intel')">Open Exam Intel</button>
          </div>
        </section>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.mini-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink);
}

.back-btn,
.level-chip,
.primary-btn,
.sidebar-btn,
.link-row {
  border: 1px solid transparent;
  transition: all 120ms;
}

.back-btn {
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  background: var(--paper);
  color: var(--ink-secondary);
}

.back-btn:hover,
.level-chip:hover,
.sidebar-btn:hover,
.link-row:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.level-chip {
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  background: transparent;
  color: var(--ink-secondary);
}

.level-chip.active {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

.meter-track {
  height: 7px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--border-soft);
}

.meter-fill {
  height: 100%;
  border-radius: 999px;
}

.meter-fill.accent {
  background: var(--accent);
}

.meter-fill.warm {
  background: var(--warm);
}

.signal-pill {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  font-size: 11px;
  font-weight: 600;
}

.signal-line,
.content-line {
  font-size: 14px;
  line-height: 1.65;
  color: var(--ink);
}

.option-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 12px;
  cursor: pointer;
  background: var(--surface);
}

.option-dot {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: 2px solid var(--ink-muted);
  flex-shrink: 0;
}

.option-dot.active {
  border-color: var(--accent);
  background: var(--accent);
}

.primary-btn,
.sidebar-btn {
  width: 100%;
  padding: 10px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  background: var(--surface);
  color: var(--ink);
}

.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sequence-row {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.sequence-no {
  width: 28px;
  height: 28px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface);
  color: var(--ink);
  font-size: 12px;
  font-weight: 800;
}

.link-row {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--surface);
  font-size: 12px;
}
</style>
