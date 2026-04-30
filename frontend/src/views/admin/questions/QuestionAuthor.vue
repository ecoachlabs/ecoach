<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/curriculum'
import {
  getAdminQuestionEditor,
  upsertAdminQuestion,
  type AdminQuestionOptionInput,
} from '@/ipc/admin'

const route = useRoute()
const router = useRouter()

const loading = ref(false)
const saving = ref(false)
const error = ref('')
const success = ref('')
const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])

const questionId = computed(() => route.query.id ? Number(route.query.id) : null)
const questionLabel = computed(() => questionId.value ? `Question #${questionId.value}` : 'Unsaved question')
const subjectId = ref('')
const topicId = ref('')
const subtopicId = ref<number | null>(null)
const familyId = ref<number | null>(null)
const stem = ref('')
const questionFormat = ref('mcq')
const sourceType = ref('authored')
const sourceRef = ref<string | null>(null)
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
const options = ref<AdminQuestionOptionInput[]>([
  { option_label: 'A', option_text: '', is_correct: true, position: 1 },
  { option_label: 'B', option_text: '', is_correct: false, position: 2 },
  { option_label: 'C', option_text: '', is_correct: false, position: 3 },
  { option_label: 'D', option_text: '', is_correct: false, position: 4 },
])

const isEditing = computed(() => questionId.value !== null)

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

async function loadLookups() {
  subjects.value = await listSubjects(1)
  if (!subjectId.value && subjects.value[0]) subjectId.value = String(subjects.value[0].id)
  if (subjectId.value) topics.value = await listTopics(Number(subjectId.value))
  if (!topicId.value && topics.value[0]) topicId.value = String(topics.value[0].id)
}

async function loadQuestion() {
  if (!questionId.value) return
  loading.value = true
  error.value = ''
  try {
    const snapshot = await getAdminQuestionEditor(questionId.value)
    subjectId.value = String(snapshot.subject_id)
    topics.value = await listTopics(snapshot.subject_id)
    topicId.value = String(snapshot.topic_id)
    subtopicId.value = snapshot.subtopic_id
    familyId.value = snapshot.family_id
    stem.value = snapshot.stem
    questionFormat.value = snapshot.question_format
    sourceType.value = snapshot.source_type
    sourceRef.value = snapshot.source_ref
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
    options.value = snapshot.options.map((option, index) => ({
      id: option.id,
      option_label: option.option_label,
      option_text: option.option_text,
      is_correct: option.is_correct,
      misconception_id: option.misconception_id,
      distractor_intent: option.distractor_intent,
      position: option.position ?? index + 1,
    }))
    if (!options.value.length) {
      options.value = [
        { option_label: 'A', option_text: '', is_correct: true, position: 1 },
        { option_label: 'B', option_text: '', is_correct: false, position: 2 },
      ]
    }
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
  if (!subjectId.value || !topicId.value) {
    error.value = 'Choose a subject and topic.'
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
      source_ref: isEditing.value ? sourceRef.value : 'admin-authored',
      exam_year: examYear.value ? Number(examYear.value) : null,
      primary_knowledge_role: knowledgeRole.value || null,
      primary_cognitive_demand: cognitiveDemand.value || null,
      primary_solve_pattern: solvePattern.value || null,
      primary_pedagogic_function: pedagogicFunction.value || null,
      primary_content_grain: contentGrain.value || null,
      cognitive_level: cognitiveDemand.value || null,
      options: options.value,
    })
    success.value = `Question #${result.question_id} saved.`
    if (!isEditing.value) {
      router.replace({ path: '/admin/questions/author', query: { id: result.question_id } })
    }
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not save question.'
  } finally {
    saving.value = false
  }
}

watch(subjectId, async value => {
  topics.value = value ? await listTopics(Number(value)) : []
  if (!topics.value.some(topic => String(topic.id) === topicId.value)) {
    topicId.value = topics.value[0] ? String(topics.value[0].id) : ''
  }
})

onMounted(async () => {
  await loadLookups()
  await loadQuestion()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">{{ isEditing ? 'Edit Question' : 'Author Question' }}</h1>
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Create or update questions, answers, and intelligence labels.</p>
      </div>
      <AppButton variant="secondary" size="sm" @click="router.push('/admin/questions')">Back to Workbench</AppButton>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <div v-if="loading" class="h-72 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--surface)' }" />

    <div v-else class="grid grid-cols-1 xl:grid-cols-[minmax(0,1fr)_380px] gap-5">
      <AppCard padding="md">
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

        <AppTextarea v-model="stem" label="Question Stem" placeholder="Enter the question text" :rows="5" class="mb-4" />

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

        <AppTextarea v-model="explanation" label="Explanation" placeholder="Why is the answer correct?" :rows="3" class="mb-5" />

        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Answer Options</h2>
          <AppButton variant="secondary" size="sm" @click="addOption">Add Option</AppButton>
        </div>

        <div class="space-y-2">
          <div v-for="(option, index) in options" :key="index" class="flex items-center gap-2">
            <button
              class="w-9 h-9 rounded-lg text-xs font-bold"
              :style="option.is_correct ? { backgroundColor: 'var(--accent)', color: 'white' } : { backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }"
              @click="markCorrect(index)"
            >{{ option.option_label }}</button>
            <AppInput v-model="option.option_text" class="flex-1" :placeholder="`Option ${option.option_label}`" />
            <AppButton variant="ghost" size="sm" :disabled="options.length <= 2" @click="removeOption(index)">Remove</AppButton>
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
          <div class="flex items-center gap-2 mb-3">
            <AppBadge :color="isEditing ? 'gold' : 'accent'" size="xs">{{ isEditing ? 'Editing' : 'New' }}</AppBadge>
            <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ questionLabel }}</span>
          </div>
          <AppButton variant="primary" class="w-full" :loading="saving" @click="saveQuestion">Save Question</AppButton>
        </AppCard>
      </div>
    </div>
  </div>
</template>
