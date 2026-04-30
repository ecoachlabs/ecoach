<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import ContentInspectorPanel from '@/components/admin/cms/ContentInspectorPanel.vue'
import MathText from '@/components/question/MathText.vue'
import type { CmsMetricItem } from '@/components/admin/cms/types'
import {
  archiveAdminQuestion,
  bulkUpdateAdminQuestions,
  createQuestionGenerationRequest,
  getAdminQuestionBankStats,
  getQuestionIntelligence,
  listAdminQuestions,
  processQuestionGenerationRequest,
  reviewQuestionIntelligence,
  restoreAdminQuestion,
  type AdminQuestionBankStatsDto,
  type AdminQuestionListItemDto,
} from '@/ipc/admin'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/curriculum'

const router = useRouter()
const auth = useAuthStore()

const loading = ref(true)
const actionLoading = ref(false)
const error = ref('')
const success = ref('')
const stats = ref<AdminQuestionBankStatsDto | null>(null)
const questions = ref<AdminQuestionListItemDto[]>([])
const selected = ref<AdminQuestionListItemDto | null>(null)
const selectedIntelligence = ref<any | null>(null)
const selectedIds = ref<number[]>([])

const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])
const bulkTopics = ref<TopicDto[]>([])
const search = ref('')
const subjectId = ref('')
const topicId = ref('')
const bulkSubjectId = ref('')
const bulkTopicId = ref('')
const reviewStatus = ref('')
const activeStatus = ref<'active' | 'archived' | 'all'>('active')
const seedCount = ref(3)
const variantMode = ref('representation_shift')

const selectedAccuracy = computed(() => {
  if (!selected.value?.attempt_count) return null
  return Math.round((selected.value.correct_count / selected.value.attempt_count) * 100)
})

const bankMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Active Questions', value: stats.value?.active_questions ?? 0, caption: 'Visible inventory' },
  { label: 'Answers', value: stats.value?.total_options ?? 0, caption: 'Stored options' },
  { label: 'Attempts', value: stats.value?.total_attempts ?? 0, caption: 'Learner signal' },
  { label: 'Families', value: stats.value?.family_count ?? 0, caption: 'Variant groups' },
  { label: 'Pending Review', value: stats.value?.pending_review_count ?? 0, tone: 'review' },
  {
    label: 'Archived',
    value: Math.max(0, (stats.value?.total_questions ?? 0) - (stats.value?.active_questions ?? 0)),
    tone: 'neutral',
  },
])

const selectedQuestions = computed(() => {
  const selectedSet = new Set(selectedIds.value)
  return questions.value.filter(question => selectedSet.has(question.question_id))
})

const selectedActiveCount = computed(() => selectedQuestions.value.filter(question => question.is_active).length)
const selectedArchivedCount = computed(() => selectedQuestions.value.filter(question => !question.is_active).length)

function openEditor(question?: AdminQuestionListItemDto | null) {
  router.push({
    path: '/admin/content-editor',
    query: {
      type: 'question',
      ...(question ? { id: question.question_id } : {}),
    },
  })
}

function jumpToSeeding() {
  window.location.hash = 'seeding'
}

async function loadLookups() {
  subjects.value = await listSubjects(1)
  if (subjectId.value) {
    topics.value = await listTopics(Number(subjectId.value))
  }
}

async function loadQuestions() {
  loading.value = true
  error.value = ''
  try {
    const [statsResult, questionResult] = await Promise.all([
      getAdminQuestionBankStats(),
      listAdminQuestions({
        search: search.value || null,
        subject_id: subjectId.value ? Number(subjectId.value) : null,
        topic_id: topicId.value ? Number(topicId.value) : null,
        review_status: reviewStatus.value || null,
        source_type: null,
        active_status: activeStatus.value,
        limit: 80,
      }),
    ])
    stats.value = statsResult
    questions.value = questionResult
    const visibleIds = new Set(questionResult.map(question => question.question_id))
    selectedIds.value = selectedIds.value.filter(questionId => visibleIds.has(questionId))
    if (!selected.value || !questionResult.some(question => question.question_id === selected.value?.question_id)) {
      selected.value = questionResult[0] ?? null
    }
    await loadSelectedIntelligence()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load questions.'
  } finally {
    loading.value = false
  }
}

async function loadSelectedIntelligence() {
  selectedIntelligence.value = null
  if (!selected.value) return
  try {
    selectedIntelligence.value = await getQuestionIntelligence(selected.value.question_id)
  } catch {
    selectedIntelligence.value = null
  }
}

async function selectQuestion(question: AdminQuestionListItemDto) {
  selected.value = question
  await loadSelectedIntelligence()
}

function isQuestionSelected(questionId: number) {
  return selectedIds.value.includes(questionId)
}

function toggleQuestionSelection(question: AdminQuestionListItemDto) {
  if (isQuestionSelected(question.question_id)) {
    selectedIds.value = selectedIds.value.filter(questionId => questionId !== question.question_id)
    return
  }
  selectedIds.value = [...selectedIds.value, question.question_id]
}

function selectVisibleQuestions() {
  selectedIds.value = questions.value.map(question => question.question_id)
}

function clearSelection() {
  selectedIds.value = []
}

async function approveSelected() {
  if (!selected.value) return
  actionLoading.value = true
  error.value = ''
  success.value = ''
  try {
    await reviewQuestionIntelligence(selected.value.question_id, {
      reviewer_id: `admin:${auth.currentAccount?.id ?? 'local'}`,
      action_code: 'approve',
      review_status: 'approved',
      note: 'Approved from admin question workbench.',
      primary_knowledge_role: selectedIntelligence.value?.knowledge_role ?? null,
      primary_cognitive_demand: selectedIntelligence.value?.cognitive_demand ?? null,
      primary_solve_pattern: selectedIntelligence.value?.solve_pattern ?? null,
      primary_pedagogic_function: selectedIntelligence.value?.pedagogic_function ?? null,
      primary_content_grain: selectedIntelligence.value?.content_grain ?? null,
      family_id: selected.value.family_id,
      misconception_codes: [],
      request_reclassification: false,
    })
    success.value = `Question #${selected.value.question_id} approved.`
    await loadQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not approve question.'
  } finally {
    actionLoading.value = false
  }
}

async function seedFromSelected() {
  if (!selected.value) return
  actionLoading.value = true
  error.value = ''
  success.value = ''
  try {
    const request = await createQuestionGenerationRequest({
      slot_spec: {
        subject_id: selected.value.subject_id,
        topic_id: selected.value.topic_id,
        target_cognitive_demand: selectedIntelligence.value?.cognitive_demand ?? null,
        target_question_format: selected.value.question_format,
        max_generated_share: 7000,
      },
      family_id: selected.value.family_id,
      source_question_id: selected.value.question_id,
      request_kind: 'variant',
      variant_mode: variantMode.value,
      requested_count: seedCount.value,
      rationale: 'Manual super admin seeding from question workbench.',
    })
    const drafts = await processQuestionGenerationRequest(request.id)
    success.value = `Seeded ${drafts.length} question${drafts.length === 1 ? '' : 's'} from #${selected.value.question_id}.`
    await loadQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not seed questions.'
  } finally {
    actionLoading.value = false
  }
}

async function archiveQuestion(question: AdminQuestionListItemDto | null) {
  if (!question || actionLoading.value) return
  const confirmed = window.confirm(`Archive question #${question.question_id} from the active bank?`)
  if (!confirmed) return

  actionLoading.value = true
  error.value = ''
  success.value = ''
  try {
    await archiveAdminQuestion(question.question_id)
    success.value = `Question #${question.question_id} archived.`
    selected.value = null
    selectedIntelligence.value = null
    await loadQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not archive question.'
  } finally {
    actionLoading.value = false
  }
}

async function restoreQuestion(question: AdminQuestionListItemDto | null) {
  if (!question || actionLoading.value) return

  actionLoading.value = true
  error.value = ''
  success.value = ''
  try {
    await restoreAdminQuestion(question.question_id)
    success.value = `Question #${question.question_id} restored to the active bank.`
    selected.value = null
    selectedIntelligence.value = null
    activeStatus.value = 'active'
    await loadQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not restore question.'
  } finally {
    actionLoading.value = false
  }
}

async function runBulkQuestionAction(action: 'archive' | 'restore' | 'move') {
  if (!selectedIds.value.length || actionLoading.value) return

  error.value = ''
  success.value = ''

  const targetQuestions =
    action === 'archive'
      ? selectedQuestions.value.filter(question => question.is_active)
      : action === 'restore'
        ? selectedQuestions.value.filter(question => !question.is_active)
        : selectedQuestions.value
  const targetIds = targetQuestions.map(question => question.question_id)

  if (!targetIds.length) {
    error.value = action === 'archive'
      ? 'Select at least one active question to archive.'
      : 'Select at least one archived question to restore.'
    return
  }

  if (action === 'move' && !bulkTopicId.value) {
    error.value = 'Choose a destination subject and topic before moving questions.'
    return
  }

  const actionLabel = action === 'archive' ? 'archive' : action === 'restore' ? 'restore' : 'move'
  const confirmed = window.confirm(`${actionLabel.charAt(0).toUpperCase()}${actionLabel.slice(1)} ${targetIds.length} selected question${targetIds.length === 1 ? '' : 's'}?`)
  if (!confirmed) return

  actionLoading.value = true
  try {
    const result = await bulkUpdateAdminQuestions({
      question_ids: targetIds,
      action,
      subject_id: bulkSubjectId.value ? Number(bulkSubjectId.value) : null,
      topic_id: bulkTopicId.value ? Number(bulkTopicId.value) : null,
    })
    success.value = `${actionLabel.charAt(0).toUpperCase()}${actionLabel.slice(1)}d ${result.updated_count} question${result.updated_count === 1 ? '' : 's'}.`
    selected.value = null
    selectedIntelligence.value = null
    selectedIds.value = []
    if (action === 'restore') activeStatus.value = 'active'
    await loadQuestions()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? `Could not ${actionLabel} selected questions.`
  } finally {
    actionLoading.value = false
  }
}

watch(subjectId, async value => {
  topicId.value = ''
  topics.value = value ? await listTopics(Number(value)) : []
})

watch(bulkSubjectId, async value => {
  bulkTopicId.value = ''
  bulkTopics.value = value ? await listTopics(Number(value)) : []
})

let searchTimer: number | undefined
watch([search, subjectId, topicId, reviewStatus, activeStatus], () => {
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(loadQuestions, 250)
})

onMounted(async () => {
  await loadLookups()
  await loadQuestions()
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Question Bank</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          View the structured inventory, inspect what each question tests, and send content changes into the editor.
        </p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="loadQuestions">Refresh</AppButton>
        <AppButton variant="secondary" size="sm" @click="jumpToSeeding">Seeding Engine</AppButton>
        <AppButton variant="primary" size="sm" @click="openEditor(null)">New Structured Question</AppButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <CmsMetricStrip :items="bankMetrics" class="mb-6" />

    <div class="grid grid-cols-1 xl:grid-cols-[minmax(0,1.05fr)_minmax(420px,0.95fr)] gap-5">
      <AppCard padding="md">
        <div class="grid grid-cols-1 md:grid-cols-5 gap-3 mb-4">
          <AppInput v-model="search" label="Search" placeholder="Stem, topic, family" />
          <AppSelect
            v-model="subjectId"
            label="Subject"
            :options="[{ value: '', label: 'All subjects' }, ...subjects.map(subject => ({ value: String(subject.id), label: subject.name }))]"
          />
          <AppSelect
            v-model="topicId"
            label="Topic"
            :disabled="!subjectId"
            :options="[{ value: '', label: 'All topics' }, ...topics.map(topic => ({ value: String(topic.id), label: topic.name }))]"
          />
          <AppSelect
            v-model="reviewStatus"
            label="Review"
            :options="[
              { value: '', label: 'All review states' },
              { value: 'pending', label: 'Pending' },
              { value: 'needs_review', label: 'Needs Review' },
              { value: 'approved', label: 'Approved' },
              { value: 'unclassified', label: 'Unclassified' },
            ]"
          />
          <AppSelect
            v-model="activeStatus"
            label="Visibility"
            :options="[
              { value: 'active', label: 'Active Bank' },
              { value: 'archived', label: 'Archived' },
              { value: 'all', label: 'All Records' },
            ]"
          />
        </div>

        <div
          class="mb-4 rounded-lg border p-3"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
        >
          <div class="flex flex-wrap items-center gap-2">
            <div class="mr-auto">
              <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">Bulk Organizer</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                {{ selectedIds.length }} selected / {{ selectedActiveCount }} active / {{ selectedArchivedCount }} archived
              </p>
            </div>
            <AppButton variant="secondary" size="sm" :disabled="!questions.length || actionLoading" @click="selectVisibleQuestions">Select Visible</AppButton>
            <AppButton variant="secondary" size="sm" :disabled="!selectedIds.length || actionLoading" @click="clearSelection">Clear</AppButton>
            <AppButton
              variant="secondary"
              size="sm"
              :disabled="!selectedActiveCount || actionLoading"
              :loading="actionLoading && !!selectedActiveCount"
              @click="runBulkQuestionAction('archive')"
            >
              Archive Active
            </AppButton>
            <AppButton
              variant="secondary"
              size="sm"
              :disabled="!selectedArchivedCount || actionLoading"
              :loading="actionLoading && !!selectedArchivedCount"
              @click="runBulkQuestionAction('restore')"
            >
              Restore Archived
            </AppButton>
          </div>
          <div class="grid grid-cols-1 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] gap-3 mt-3">
            <AppSelect
              v-model="bulkSubjectId"
              label="Move Subject"
              :options="[{ value: '', label: 'Choose subject' }, ...subjects.map(subject => ({ value: String(subject.id), label: subject.name }))]"
            />
            <AppSelect
              v-model="bulkTopicId"
              label="Move Topic"
              :disabled="!bulkSubjectId"
              :options="[{ value: '', label: 'Choose topic' }, ...bulkTopics.map(topic => ({ value: String(topic.id), label: topic.name }))]"
            />
            <div class="flex items-end">
              <AppButton
                variant="primary"
                size="sm"
                class="w-full"
                :disabled="!selectedIds.length || !bulkTopicId || actionLoading"
                :loading="actionLoading && !!bulkTopicId"
                @click="runBulkQuestionAction('move')"
              >
                Move Selected
              </AppButton>
            </div>
          </div>
        </div>

        <div v-if="loading" class="space-y-2">
          <div v-for="i in 8" :key="i" class="h-16 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="question in questions"
            :key="question.question_id"
            class="w-full rounded-lg px-3 py-3 transition-colors"
            :style="{
              backgroundColor: selected?.question_id === question.question_id
                ? 'var(--surface)'
                : isQuestionSelected(question.question_id)
                  ? 'var(--border-soft)'
                  : 'var(--paper)',
            }"
          >
            <div class="flex items-start gap-3">
              <button
                type="button"
                class="h-7 w-7 shrink-0 rounded-md grid place-items-center"
                :aria-label="isQuestionSelected(question.question_id) ? `Deselect question ${question.question_id}` : `Select question ${question.question_id}`"
                :style="{ border: `1px solid ${isQuestionSelected(question.question_id) ? 'var(--accent)' : 'var(--border)'}` }"
                @click="toggleQuestionSelection(question)"
              >
                <span
                  class="block h-3 w-3 rounded-sm"
                  :style="{ backgroundColor: isQuestionSelected(question.question_id) ? 'var(--accent)' : 'transparent' }"
                />
              </button>
              <button type="button" class="flex-1 min-w-0 text-left" @click="selectQuestion(question)">
                <div class="flex items-start gap-3">
                  <AppBadge color="muted" size="xs">#{{ question.question_id }}</AppBadge>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium line-clamp-2" :style="{ color: 'var(--ink)' }">
                      <MathText :text="question.stem" size="sm" />
                    </p>
                    <p class="text-[10px] mt-1" :style="{ color: 'var(--ink-muted)' }">
                      {{ question.subject_name }} / {{ question.topic_name }} / {{ question.question_format }} / {{ question.option_count }} answers
                    </p>
                  </div>
                  <div class="text-right shrink-0">
                    <CmsStatusBadge :status="question.is_active ? question.review_status : 'archived'" />
                    <p class="text-[10px] mt-1 tabular-nums" :style="{ color: 'var(--ink-muted)' }">{{ question.attempt_count }} attempts</p>
                  </div>
                </div>
              </button>
            </div>
          </div>
          <p v-if="!questions.length" class="text-sm py-8 text-center" :style="{ color: 'var(--ink-muted)' }">No questions match this view.</p>
        </div>
      </AppCard>

      <div class="space-y-4">
        <ContentInspectorPanel
          :question="selected"
          :accuracy="selectedAccuracy"
          @edit="openEditor"
          @seed="() => seedFromSelected()"
          @archive="archiveQuestion"
          @restore="restoreQuestion"
        />

        <AppCard padding="md">
          <div v-if="selected">
            <div class="flex items-start justify-between gap-3 mb-4">
              <div>
                <p class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Testing Intelligence</p>
                <h2 class="text-base font-bold mt-1" :style="{ color: 'var(--ink)' }">What this question is measuring</h2>
              </div>
              <AppButton variant="primary" size="sm" :loading="actionLoading" @click="approveSelected">Approve</AppButton>
            </div>

            <p class="text-sm leading-relaxed mb-4" :style="{ color: 'var(--ink)' }">
              <MathText :text="selected.stem" size="sm" />
            </p>

            <div class="grid grid-cols-2 gap-3 mb-4">
              <div class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-lg font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ selected.attempt_count }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Attempts gathered</p>
              </div>
              <div class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-lg font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ selectedAccuracy ?? '-' }}%</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Accuracy</p>
              </div>
              <div class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-lg font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ selected.average_response_time_ms ?? '-' }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Avg response ms</p>
              </div>
              <div class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-lg font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ selected.machine_confidence_score }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Machine confidence</p>
              </div>
            </div>

            <div class="space-y-2 text-sm">
              <div class="flex justify-between gap-3">
                <span :style="{ color: 'var(--ink-muted)' }">Knowledge role</span>
                <span :style="{ color: 'var(--ink)' }">{{ selectedIntelligence?.knowledge_role ?? '-' }}</span>
              </div>
              <div class="flex justify-between gap-3">
                <span :style="{ color: 'var(--ink-muted)' }">Cognitive demand</span>
                <span :style="{ color: 'var(--ink)' }">{{ selectedIntelligence?.cognitive_demand ?? '-' }}</span>
              </div>
              <div class="flex justify-between gap-3">
                <span :style="{ color: 'var(--ink-muted)' }">Solve pattern</span>
                <span :style="{ color: 'var(--ink)' }">{{ selectedIntelligence?.solve_pattern ?? '-' }}</span>
              </div>
              <div class="flex justify-between gap-3">
                <span :style="{ color: 'var(--ink-muted)' }">Pedagogic function</span>
                <span :style="{ color: 'var(--ink)' }">{{ selectedIntelligence?.pedagogic_function ?? '-' }}</span>
              </div>
              <div class="flex justify-between gap-3">
                <span :style="{ color: 'var(--ink-muted)' }">Family</span>
                <span class="text-right" :style="{ color: 'var(--ink)' }">{{ selected.family_name ?? selectedIntelligence?.family?.family_name ?? '-' }}</span>
              </div>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Select a question to inspect it.</p>
        </AppCard>

        <AppCard id="seeding" padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Seeding Engine</h2>
          <p class="text-xs mb-4" :style="{ color: 'var(--ink-muted)' }">
            Generate more questions from a selected family or source question, then review the drafts before publishing.
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
          <AppButton variant="primary" :disabled="!selected" :loading="actionLoading" @click="seedFromSelected">Seed From Selected</AppButton>
        </AppCard>
      </div>
    </div>
  </div>
</template>
