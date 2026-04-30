<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import MathText from '@/components/question/MathText.vue'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/curriculum'
import {
  archiveAdminQuestion,
  createQuestionGenerationRequest,
  getAdminQuestionEditorAny,
  listAdminQuestions,
  processQuestionGenerationRequest,
  restoreAdminQuestion,
  upsertAdminQuestion,
  type AdminQuestionEditorDto,
  type AdminQuestionListItemDto,
  type AdminQuestionOptionInput,
} from '@/ipc/admin'
import type { CmsMetricItem } from '@/components/admin/cms/types'

type RecordType = 'question' | 'lesson' | 'answer' | 'curriculum'

interface RecordTypeOption {
  id: RecordType
  label: string
  description: string
  available: boolean
}

type EditorQuestionOptionInput = Omit<AdminQuestionOptionInput, 'distractor_intent'> & {
  distractor_intent?: string
}

const route = useRoute()
const router = useRouter()

const loading = ref(false)
const browserLoading = ref(false)
const saving = ref(false)
const seeding = ref(false)
const error = ref('')
const success = ref('')
const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])
const editorQuestions = ref<AdminQuestionListItemDto[]>([])
const loadedQuestion = ref<AdminQuestionEditorDto | null>(null)

const recordTypes: RecordTypeOption[] = [
  { id: 'question', label: 'Question', description: 'Stem, answers, explanation, labels', available: true },
  { id: 'answer', label: 'Answer Text', description: 'Option copy and distractor intent', available: false },
  { id: 'lesson', label: 'Lesson Text', description: 'Extracted study content', available: false },
  { id: 'curriculum', label: 'Curriculum Node', description: 'Subject, topic, and objective text', available: false },
]

const recordType = ref<RecordType>(normalizeRecordType(route.query.type))
const questionId = computed(() => route.query.id ? Number(route.query.id) : null)
const isEditing = computed(() => questionId.value !== null)
const editorTitle = computed(() => isEditing.value ? `Question #${questionId.value}` : 'New question')
const isArchived = computed(() => loadedQuestion.value ? !loadedQuestion.value.is_active : false)

const subjectId = ref('')
const topicId = ref('')
const subtopicId = ref<number | null>(null)
const familyId = ref<number | null>(null)
const stem = ref('')
const questionFormat = ref('mcq')
const sourceType = ref('authored')
const sourceRef = ref('')
const examYear = ref('')
const difficultyLevel = ref(5000)
const estimatedTime = ref(45)
const marks = ref(1)
const explanation = ref('')
const knowledgeRole = ref('')
const cognitiveDemand = ref('')
const solvePattern = ref('')
const pedagogicFunction = ref('')
const contentGrain = ref('topic')
const options = ref<EditorQuestionOptionInput[]>(defaultOptions())
const editorSearch = ref('')
const editorVisibility = ref<'active' | 'archived' | 'all'>('active')
const seedCount = ref(3)
const variantMode = ref('representation_shift')

const editorMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Options', value: options.value.length, caption: 'Answer records' },
  { label: 'Correct', value: options.value.filter(option => option.is_correct).length, tone: 'good' },
  { label: 'Difficulty', value: difficultyLevel.value || '-', caption: 'Basis points' },
  { label: 'Time', value: `${estimatedTime.value || 0}s`, caption: 'Target solve' },
])

const selectedRecordType = computed(() => recordTypes.find(type => type.id === recordType.value) ?? recordTypes[0])

function normalizeRecordType(value: unknown): RecordType {
  return ['question', 'lesson', 'answer', 'curriculum'].includes(String(value)) ? String(value) as RecordType : 'question'
}

function defaultOptions(): EditorQuestionOptionInput[] {
  return [
    { option_label: 'A', option_text: '', is_correct: true, distractor_intent: '', position: 1 },
    { option_label: 'B', option_text: '', is_correct: false, distractor_intent: '', position: 2 },
    { option_label: 'C', option_text: '', is_correct: false, distractor_intent: '', position: 3 },
    { option_label: 'D', option_text: '', is_correct: false, distractor_intent: '', position: 4 },
  ]
}

function resetQuestionForm() {
  loadedQuestion.value = null
  subtopicId.value = null
  familyId.value = null
  stem.value = ''
  questionFormat.value = 'mcq'
  sourceType.value = 'authored'
  sourceRef.value = ''
  examYear.value = ''
  difficultyLevel.value = 5000
  estimatedTime.value = 45
  marks.value = 1
  explanation.value = ''
  knowledgeRole.value = ''
  cognitiveDemand.value = ''
  solvePattern.value = ''
  pedagogicFunction.value = ''
  contentGrain.value = 'topic'
  options.value = defaultOptions()
}

function normalizeOptionPositions() {
  options.value = options.value.map((option, index) => ({
    ...option,
    option_label: option.option_label || 'ABCDEFGH'[index] || String(index + 1),
    position: index + 1,
  }))
}

function addOption() {
  const index = options.value.length
  options.value.push({
    option_label: 'ABCDEFGH'[index] || String(index + 1),
    option_text: '',
    is_correct: false,
    distractor_intent: '',
    position: index + 1,
  })
}

function removeOption(index: number) {
  if (options.value.length <= 2) return
  options.value.splice(index, 1)
  if (!options.value.some(option => option.is_correct)) {
    options.value[0].is_correct = true
  }
  normalizeOptionPositions()
}

function markCorrect(index: number) {
  options.value = options.value.map((option, optionIndex) => ({
    ...option,
    is_correct: optionIndex === index,
  }))
}

function selectRecordType(type: RecordTypeOption) {
  recordType.value = type.id
  router.replace({
    path: '/admin/content-editor',
    query: { ...route.query, type: type.id },
  })
}

async function loadLookups() {
  subjects.value = await listSubjects(1)
  if (!subjectId.value && subjects.value[0]) subjectId.value = String(subjects.value[0].id)
  if (subjectId.value) topics.value = await listTopics(Number(subjectId.value))
  if (!topicId.value && topics.value[0]) topicId.value = String(topics.value[0].id)
}

async function loadEditorQuestions() {
  browserLoading.value = true
  try {
    editorQuestions.value = await listAdminQuestions({
      search: editorSearch.value || null,
      subject_id: null,
      topic_id: null,
      review_status: null,
      source_type: null,
      active_status: editorVisibility.value,
      limit: 30,
    })
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load editor records.'
  } finally {
    browserLoading.value = false
  }
}

function openQuestionRecord(question: AdminQuestionListItemDto) {
  router.push({
    path: '/admin/content-editor',
    query: { type: 'question', id: question.question_id },
  })
}

async function loadQuestion() {
  resetQuestionForm()
  if (!questionId.value) return
  loading.value = true
  error.value = ''
  try {
    const snapshot = await getAdminQuestionEditorAny(questionId.value)
    loadedQuestion.value = snapshot
    subjectId.value = String(snapshot.subject_id)
    topics.value = await listTopics(snapshot.subject_id)
    topicId.value = String(snapshot.topic_id)
    subtopicId.value = snapshot.subtopic_id
    familyId.value = snapshot.family_id
    stem.value = snapshot.stem
    questionFormat.value = snapshot.question_format
    sourceType.value = snapshot.source_type
    sourceRef.value = snapshot.source_ref ?? ''
    examYear.value = snapshot.exam_year ? String(snapshot.exam_year) : ''
    difficultyLevel.value = snapshot.difficulty_level
    estimatedTime.value = snapshot.estimated_time_seconds
    marks.value = snapshot.marks
    explanation.value = snapshot.explanation_text ?? ''
    knowledgeRole.value = snapshot.primary_knowledge_role ?? ''
    cognitiveDemand.value = snapshot.primary_cognitive_demand ?? ''
    solvePattern.value = snapshot.primary_solve_pattern ?? ''
    pedagogicFunction.value = snapshot.primary_pedagogic_function ?? ''
    contentGrain.value = snapshot.primary_content_grain ?? 'topic'
    options.value = snapshot.options.length
      ? snapshot.options.map((option, index) => ({
        id: option.id,
        option_label: option.option_label,
        option_text: option.option_text,
        is_correct: option.is_correct,
        misconception_id: option.misconception_id,
        distractor_intent: option.distractor_intent ?? '',
        position: option.position ?? index + 1,
      }))
      : defaultOptions().slice(0, 2)
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load question.'
  } finally {
    loading.value = false
  }
}

async function saveQuestion() {
  if (saving.value) return
  error.value = ''
  success.value = ''
  if (recordType.value !== 'question') {
    error.value = `${selectedRecordType.value.label} editing is not connected yet.`
    return
  }
  if (!subjectId.value || !topicId.value) {
    error.value = 'Choose a subject and topic.'
    return
  }
  if (!stem.value.trim()) {
    error.value = 'Add the question stem before saving.'
    return
  }
  if (isArchived.value) {
    error.value = 'Restore this question before saving edits.'
    return
  }

  saving.value = true
  normalizeOptionPositions()
  try {
    const result = await upsertAdminQuestion({
      question_id: questionId.value,
      subject_id: Number(subjectId.value),
      topic_id: Number(topicId.value),
      subtopic_id: subtopicId.value,
      family_id: familyId.value,
      stem: stem.value,
      question_format: questionFormat.value,
      explanation_text: explanation.value || null,
      difficulty_level: Number(difficultyLevel.value),
      estimated_time_seconds: Number(estimatedTime.value),
      marks: Number(marks.value),
      source_type: sourceType.value,
      source_ref: sourceRef.value || (isEditing.value ? null : 'admin-authored'),
      exam_year: examYear.value ? Number(examYear.value) : null,
      primary_knowledge_role: knowledgeRole.value || null,
      primary_cognitive_demand: cognitiveDemand.value || null,
      primary_solve_pattern: solvePattern.value || null,
      primary_pedagogic_function: pedagogicFunction.value || null,
      primary_content_grain: contentGrain.value || null,
      cognitive_level: cognitiveDemand.value || null,
      options: options.value.map(option => ({
        ...option,
        distractor_intent: option.distractor_intent || null,
      })),
    })
    success.value = `Question #${result.question_id} saved.`
    await router.replace({ path: '/admin/content-editor', query: { type: 'question', id: result.question_id } })
    await loadQuestion()
    await loadEditorQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not save question.'
  } finally {
    saving.value = false
  }
}

async function archiveCurrentQuestion() {
  if (!questionId.value || saving.value) return
  const confirmed = window.confirm(`Archive question #${questionId.value} from the active bank?`)
  if (!confirmed) return

  saving.value = true
  error.value = ''
  success.value = ''
  try {
    await archiveAdminQuestion(questionId.value)
    success.value = `Question #${questionId.value} archived.`
    editorVisibility.value = 'archived'
    await loadQuestion()
    await loadEditorQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not archive question.'
  } finally {
    saving.value = false
  }
}

async function restoreCurrentQuestion() {
  if (!questionId.value || saving.value) return

  saving.value = true
  error.value = ''
  success.value = ''
  try {
    await restoreAdminQuestion(questionId.value)
    success.value = `Question #${questionId.value} restored to the active bank.`
    editorVisibility.value = 'active'
    await loadQuestion()
    await loadEditorQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not restore question.'
  } finally {
    saving.value = false
  }
}

async function seedFromCurrentQuestion() {
  if (!loadedQuestion.value || !questionId.value || seeding.value) return
  if (isArchived.value) {
    error.value = 'Restore this question before generating variants from it.'
    return
  }

  seeding.value = true
  error.value = ''
  success.value = ''
  try {
    const request = await createQuestionGenerationRequest({
      slot_spec: {
        subject_id: loadedQuestion.value.subject_id,
        topic_id: loadedQuestion.value.topic_id,
        target_cognitive_demand: cognitiveDemand.value || loadedQuestion.value.primary_cognitive_demand,
        target_question_format: loadedQuestion.value.question_format,
        max_generated_share: 7000,
      },
      family_id: loadedQuestion.value.family_id,
      source_question_id: questionId.value,
      request_kind: 'variant',
      variant_mode: variantMode.value,
      requested_count: Number(seedCount.value),
      rationale: 'Manual super admin seeding from content editor.',
    })
    const drafts = await processQuestionGenerationRequest(request.id)
    success.value = `Seeded ${drafts.length} question${drafts.length === 1 ? '' : 's'} from #${questionId.value}.`
    await loadEditorQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not seed from this question.'
  } finally {
    seeding.value = false
  }
}

watch(subjectId, async value => {
  topics.value = value ? await listTopics(Number(value)) : []
  if (!topics.value.some(topic => String(topic.id) === topicId.value)) {
    topicId.value = topics.value[0] ? String(topics.value[0].id) : ''
  }
})

watch(() => route.query.type, value => {
  recordType.value = normalizeRecordType(value)
})

watch(questionId, async () => {
  await loadQuestion()
})

let editorSearchTimer: number | undefined
watch([editorSearch, editorVisibility], () => {
  window.clearTimeout(editorSearchTimer)
  editorSearchTimer = window.setTimeout(loadEditorQuestions, 250)
})

onMounted(async () => {
  await loadLookups()
  await loadQuestion()
  await loadEditorQuestions()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
          Structured CMS Records
        </p>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Content Editor</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          Edit the structured text that powers the question bank. Question records are connected first; other record types can plug into this same workspace.
        </p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="router.push('/admin/questions')">Question Bank</AppButton>
        <AppButton v-if="isEditing && !isArchived" variant="danger" size="sm" :loading="saving" @click="archiveCurrentQuestion">Archive</AppButton>
        <AppButton v-if="isArchived" variant="primary" size="sm" :loading="saving" @click="restoreCurrentQuestion">Restore</AppButton>
        <AppButton variant="primary" size="sm" :disabled="isArchived" :loading="saving" @click="saveQuestion">Save</AppButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <div class="grid grid-cols-1 xl:grid-cols-[260px_minmax(0,1fr)_360px] gap-5">
      <div class="space-y-3">
        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Record Type</h2>
          <div class="space-y-2">
            <button
              v-for="type in recordTypes"
              :key="type.id"
              class="w-full text-left rounded-lg p-3 transition-colors"
              :style="{ backgroundColor: recordType === type.id ? 'var(--surface)' : 'var(--paper)' }"
              @click="selectRecordType(type)"
            >
              <div class="flex items-center justify-between gap-2">
                <span class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ type.label }}</span>
                <CmsStatusBadge :status="type.available ? 'active' : 'queued'" />
              </div>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ type.description }}</p>
            </button>
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Find Question</h2>
          <div class="space-y-3 mb-3">
            <AppInput v-model="editorSearch" label="Search Records" placeholder="Stem, topic, family" />
            <AppSelect
              v-model="editorVisibility"
              label="Visibility"
              :options="[
                { value: 'active', label: 'Active Bank' },
                { value: 'archived', label: 'Archived' },
                { value: 'all', label: 'All Records' },
              ]"
            />
          </div>
          <div v-if="browserLoading" class="space-y-2">
            <div v-for="i in 4" :key="i" class="h-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else class="space-y-2 max-h-80 overflow-y-auto">
            <button
              v-for="question in editorQuestions"
              :key="question.question_id"
              class="w-full text-left rounded-lg p-3 transition-colors"
              :style="{ backgroundColor: questionId === question.question_id ? 'var(--surface)' : 'var(--paper)' }"
              @click="openQuestionRecord(question)"
            >
              <div class="flex items-start justify-between gap-2 mb-2">
                <span class="text-[10px] font-bold tabular-nums" :style="{ color: 'var(--ink-muted)' }">#{{ question.question_id }}</span>
                <CmsStatusBadge :status="question.is_active ? question.review_status : 'archived'" />
              </div>
              <p class="text-xs font-semibold line-clamp-2" :style="{ color: 'var(--ink)' }">{{ question.stem }}</p>
              <p class="text-[10px] mt-1 truncate" :style="{ color: 'var(--ink-muted)' }">{{ question.subject_name }} / {{ question.topic_name }}</p>
            </button>
            <p v-if="!editorQuestions.length" class="text-xs py-5 text-center" :style="{ color: 'var(--ink-muted)' }">No question records match this view.</p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-2" :style="{ color: 'var(--ink)' }">Editor State</h2>
          <div class="flex items-center gap-2 mb-3">
            <CmsStatusBadge :status="isArchived ? 'archived' : loadedQuestion?.review_status ?? (isEditing ? 'loading' : 'draft')" />
            <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ editorTitle }}</span>
          </div>
          <CmsMetricStrip :items="editorMetrics" />
        </AppCard>
      </div>

      <div v-if="loading" class="h-72 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--surface)' }" />

      <AppCard v-else padding="md">
        <div v-if="recordType !== 'question'" class="min-h-80 flex flex-col justify-center">
          <h2 class="text-lg font-bold mb-2" :style="{ color: 'var(--ink)' }">{{ selectedRecordType.label }}</h2>
          <p class="text-sm max-w-xl" :style="{ color: 'var(--ink-muted)' }">
            This record family is part of the CMS shape, but the current write path is connected to questions first.
          </p>
        </div>

        <div v-else>
          <div
            v-if="isArchived"
            class="rounded-lg border p-3 mb-4 text-sm"
            :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }"
          >
            This question is archived. Restore it before saving edits or generating variants.
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
            <AppSelect
              v-model="subjectId"
              label="Subject"
              :options="subjects.map(subject => ({ value: String(subject.id), label: subject.name }))"
            />
            <AppSelect
              v-model="topicId"
              label="Topic"
              :options="topics.map(topic => ({ value: String(topic.id), label: topic.name }))"
            />
          </div>

          <AppTextarea v-model="stem" label="Question Stem" placeholder="Enter the exact question text" :rows="6" class="mb-4" />

          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <AppSelect
              v-model="questionFormat"
              label="Format"
              :options="[
                { value: 'mcq', label: 'Multiple Choice' },
                { value: 'short_answer', label: 'Short Answer' },
                { value: 'numeric', label: 'Numeric' },
                { value: 'true_false', label: 'True/False' },
                { value: 'matching', label: 'Matching' },
                { value: 'ordering', label: 'Ordering' },
              ]"
            />
            <AppSelect
              v-model="sourceType"
              label="Source"
              :options="[
                { value: 'authored', label: 'Authored' },
                { value: 'past_question', label: 'Past Question' },
                { value: 'generated', label: 'Generated' },
                { value: 'teacher_upload', label: 'Teacher Upload' },
              ]"
            />
            <AppInput v-model.number="difficultyLevel" label="Difficulty BP" type="number" />
            <AppInput v-model.number="marks" label="Marks" type="number" />
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
            <AppInput v-model.number="estimatedTime" label="Estimated Seconds" type="number" />
            <AppInput v-model="examYear" label="Exam Year" type="number" />
          </div>

          <AppInput v-model="sourceRef" label="Source Reference" placeholder="Document, import, or authoring note" class="mb-4" />
          <AppTextarea v-model="explanation" label="Explanation" placeholder="Why is the answer correct?" :rows="3" class="mb-5" />

          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Answer Options</h2>
            <AppButton variant="secondary" size="sm" @click="addOption">Add Option</AppButton>
          </div>

          <div class="space-y-2">
            <div
              v-for="(option, index) in options"
              :key="index"
              class="rounded-lg p-3"
              :style="{ backgroundColor: 'var(--paper)' }"
            >
              <div class="grid grid-cols-[80px_minmax(0,1fr)_auto_auto] gap-2 items-end">
                <AppInput v-model="option.option_label" label="Label" />
                <AppInput v-model="option.option_text" label="Answer Text" :placeholder="`Option ${option.option_label || index + 1}`" />
                <button
                  type="button"
                  class="h-9 rounded-lg px-3 text-xs font-bold"
                  :style="option.is_correct ? { backgroundColor: 'var(--accent)', color: 'white' } : { backgroundColor: 'var(--surface)', color: 'var(--ink-muted)' }"
                  @click="markCorrect(index)"
                >{{ option.is_correct ? 'Correct' : 'Mark Correct' }}</button>
                <AppButton variant="ghost" size="sm" :disabled="options.length <= 2" @click="removeOption(index)">Remove</AppButton>
              </div>
              <AppInput
                v-if="!option.is_correct"
                v-model="option.distractor_intent"
                label="Distractor Intent"
                placeholder="What misconception or mistake does this answer reveal?"
                class="mt-3"
              />
            </div>
          </div>
        </div>
      </AppCard>

      <div class="space-y-4">
        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">What This Tests</h2>
          <div class="space-y-3">
            <AppSelect
              v-model="knowledgeRole"
              label="Knowledge Role"
              :disabled="recordType !== 'question'"
              :options="[
                { value: '', label: 'Let engine infer' },
                { value: 'definition', label: 'Definition' },
                { value: 'procedure', label: 'Procedure' },
                { value: 'application', label: 'Application' },
                { value: 'comparison', label: 'Comparison' },
              ]"
            />
            <AppSelect
              v-model="cognitiveDemand"
              label="Cognitive Demand"
              :disabled="recordType !== 'question'"
              :options="[
                { value: '', label: 'Let engine infer' },
                { value: 'recognition', label: 'Recognition' },
                { value: 'recall', label: 'Recall' },
                { value: 'application', label: 'Application' },
                { value: 'analysis', label: 'Analysis' },
                { value: 'inference', label: 'Inference' },
              ]"
            />
            <AppSelect
              v-model="solvePattern"
              label="Solve Pattern"
              :disabled="recordType !== 'question'"
              :options="[
                { value: '', label: 'Let engine infer' },
                { value: 'direct_retrieval', label: 'Direct Retrieval' },
                { value: 'substitute_and_solve', label: 'Substitute and Solve' },
                { value: 'pattern_spotting', label: 'Pattern Spotting' },
                { value: 'multi_step_reasoning', label: 'Multi-step Reasoning' },
              ]"
            />
            <AppSelect
              v-model="pedagogicFunction"
              label="Pedagogic Function"
              :disabled="recordType !== 'question'"
              :options="[
                { value: '', label: 'Let engine infer' },
                { value: 'foundation_check', label: 'Foundation Check' },
                { value: 'misconception_diagnosis', label: 'Misconception Diagnosis' },
                { value: 'transfer_check', label: 'Transfer Check' },
                { value: 'speed_build', label: 'Speed Build' },
              ]"
            />
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Generate More</h2>
          <p class="text-xs mb-4" :style="{ color: 'var(--ink-muted)' }">
            Create related drafts from the current question, then inspect and edit them before use.
          </p>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <AppSelect
              v-model="variantMode"
              label="Variant Mode"
              :options="[
                { value: 'isomorphic', label: 'Isomorphic' },
                { value: 'representation_shift', label: 'Representation Shift' },
                { value: 'misconception_probe', label: 'Misconception Probe' },
                { value: 'rescue', label: 'Rescue' },
                { value: 'stretch', label: 'Stretch' },
              ]"
            />
            <AppInput v-model.number="seedCount" label="Count" type="number" />
          </div>
          <AppButton
            variant="primary"
            class="w-full"
            :disabled="!loadedQuestion || isArchived"
            :loading="seeding"
            @click="seedFromCurrentQuestion"
          >
            Seed From This Question
          </AppButton>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Preview</h2>
          <div class="rounded-lg p-3 min-h-28" :style="{ backgroundColor: 'var(--paper)' }">
            <p v-if="stem" class="text-sm leading-relaxed" :style="{ color: 'var(--ink)' }">
              <MathText :text="stem" size="sm" />
            </p>
            <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Question preview appears here.</p>
          </div>
          <div class="grid grid-cols-1 gap-2 mt-4">
            <AppButton variant="primary" class="w-full" :disabled="isArchived" :loading="saving" @click="saveQuestion">Save {{ selectedRecordType.label }}</AppButton>
            <AppButton v-if="isEditing && !isArchived" variant="danger" class="w-full" :loading="saving" @click="archiveCurrentQuestion">Archive Question</AppButton>
            <AppButton v-if="isArchived" variant="primary" class="w-full" :loading="saving" @click="restoreCurrentQuestion">Restore Question</AppButton>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
