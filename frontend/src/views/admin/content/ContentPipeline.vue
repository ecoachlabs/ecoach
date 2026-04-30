<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import UploadDropzone from '@/components/upload/UploadDropzone.vue'
import {
  addCurriculumParseCandidate,
  finalizeCurriculumSource,
  getContentFoundrySourceReport,
  getContentHealthReadModel,
  getContentSourceDetail,
  getFoundryJobBoard,
  listContentSources,
  markCurriculumSourceReviewed,
  queueSourceFollowUpJobs,
  registerCurriculumSource,
  runFoundryJob,
  runNextFoundryJob,
  type ContentFoundrySourceReportDto,
  type ContentHealthReadModelDto,
  type ContentSourceDetailDto,
  type ContentSourceRegistryEntryDto,
  type FoundryJobBoardDto,
} from '@/ipc/admin'
import type { CmsMetricItem } from '@/components/admin/cms/types'

const router = useRouter()
const auth = useAuthStore()

const loading = ref(true)
const saving = ref(false)
const sourceDetailLoading = ref(false)
const runningJobId = ref<number | null>(null)
const sourceActionLoading = ref<'candidate' | 'finalize' | 'review' | 'queue' | ''>('')
const error = ref('')
const success = ref('')
const health = ref<ContentHealthReadModelDto | null>(null)
const sources = ref<ContentSourceRegistryEntryDto[]>([])
const board = ref<FoundryJobBoardDto | null>(null)
const selectedSourceId = ref<number | null>(null)
const selectedDetail = ref<ContentSourceDetailDto | null>(null)
const selectedReport = ref<ContentFoundrySourceReportDto | null>(null)

const sourceKind = ref('worksheet')
const subjectCode = ref('MTH')
const titlePrefix = ref('')
const candidateType = ref('topic')
const candidateRawLabel = ref('')
const candidateNormalizedLabel = ref('')
const candidateConfidence = ref('8100')
const candidatePayloadText = ref('{\n  "source": "manual_admin_extraction"\n}')

const selectedSource = computed(() => sources.value.find((source) => source.id === selectedSourceId.value) ?? null)

const pipelineCounts = computed<CmsMetricItem[]>(() => [
  { label: 'Sources', value: health.value?.source_count ?? 0, caption: 'Raw material' },
  { label: 'Stale', value: health.value?.stale_source_count ?? 0, tone: 'review' },
  { label: 'Review Due', value: health.value?.overdue_source_review_count ?? 0, tone: 'review' },
  { label: 'Missions', value: health.value?.active_mission_count ?? 0, caption: 'Foundry work' },
  { label: 'Blocked', value: health.value?.blocked_publish_count ?? 0, tone: 'danger' },
  { label: 'Preview', value: health.value?.preview_publish_count ?? 0, caption: 'Before publish' },
])

const selectedReportMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Candidates', value: selectedReport.value?.parse_candidates.length ?? 0, caption: 'Extracted labels' },
  { label: 'Approved', value: selectedReport.value?.approved_candidate_count ?? 0, tone: 'good' },
  { label: 'Low Trust', value: selectedReport.value?.low_confidence_candidate_count ?? 0, tone: 'review' },
  { label: 'Open Reviews', value: selectedReport.value?.unresolved_review_count ?? 0, tone: 'danger' },
])

function messageFromError(err: unknown, fallback: string) {
  return typeof err === 'string' ? err : (err as { message?: string })?.message ?? fallback
}

function formatBp(value: number | null | undefined) {
  return `${Math.round((value ?? 0) / 100)}%`
}

function candidateTone(status: string): 'success' | 'danger' | 'gold' | 'muted' {
  if (status === 'approved') return 'success'
  if (status === 'rejected') return 'danger'
  if (status === 'review_required') return 'gold'
  return 'muted'
}

async function loadSelectedSource() {
  const sourceId = selectedSourceId.value
  if (!sourceId) {
    selectedDetail.value = null
    selectedReport.value = null
    return
  }

  sourceDetailLoading.value = true
  try {
    const [detailResult, reportResult] = await Promise.all([
      getContentSourceDetail(sourceId),
      getContentFoundrySourceReport(sourceId),
    ])
    selectedDetail.value = detailResult
    selectedReport.value = reportResult
  } catch (err) {
    selectedDetail.value = null
    selectedReport.value = null
    error.value = messageFromError(err, 'Could not load source detail.')
  } finally {
    sourceDetailLoading.value = false
  }
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const previousSourceId = selectedSourceId.value
    const [healthResult, sourceResult, boardResult] = await Promise.all([
      getContentHealthReadModel(),
      listContentSources(null, null, 60),
      getFoundryJobBoard(null),
    ])
    health.value = healthResult
    sources.value = sourceResult
    board.value = boardResult

    if (previousSourceId && sourceResult.some((source) => source.id === previousSourceId)) {
      selectedSourceId.value = previousSourceId
    } else {
      selectedSourceId.value = sourceResult[0]?.id ?? null
    }
    await loadSelectedSource()
  } catch (err) {
    error.value = messageFromError(err, 'Could not load content pipeline.')
  } finally {
    loading.value = false
  }
}

function readableSize(file: File) {
  if (file.size < 1024) return `${file.size} B`
  if (file.size < 1024 * 1024) return `${Math.round(file.size / 1024)} KB`
  return `${(file.size / (1024 * 1024)).toFixed(1)} MB`
}

async function handleFiles(files: File[]) {
  const adminId = auth.currentAccount?.id
  if (!adminId) {
    error.value = 'Admin account is not loaded.'
    return
  }
  if (!files.length || saving.value) return

  saving.value = true
  error.value = ''
  success.value = ''
  try {
    let newestSourceId: number | null = null
    for (const file of files) {
      const registered = await registerCurriculumSource({
        uploader_account_id: adminId,
        source_kind: sourceKind.value,
        title: titlePrefix.value.trim() || file.name,
        source_path: file.name,
        subject_code: subjectCode.value.trim() || null,
        language_code: 'en',
        metadata: {
          file_name: file.name,
          file_type: file.type || 'unknown',
          file_size: file.size,
          readable_size: readableSize(file),
          last_modified: file.lastModified,
          intake_mode: 'manual_admin_upload',
        },
      })
      newestSourceId = registered.id
    }
    success.value = `${files.length} source${files.length === 1 ? '' : 's'} registered.`
    titlePrefix.value = ''
    selectedSourceId.value = newestSourceId
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not register source.')
  } finally {
    saving.value = false
  }
}

async function selectSource(source: ContentSourceRegistryEntryDto) {
  if (selectedSourceId.value === source.id && selectedReport.value) return
  selectedSourceId.value = source.id
  error.value = ''
  success.value = ''
  await loadSelectedSource()
}

function buildCandidatePayload() {
  const payloadText = candidatePayloadText.value.trim()
  if (!payloadText) return { source: 'manual_admin_extraction' }
  try {
    return JSON.parse(payloadText)
  } catch {
    error.value = 'Candidate payload must be valid JSON.'
    return null
  }
}

async function addCandidate() {
  const source = selectedSource.value
  if (!source || sourceActionLoading.value) return
  const rawLabel = candidateRawLabel.value.trim()
  if (!rawLabel) {
    error.value = 'Add the extracted label before saving a candidate.'
    return
  }
  const payload = buildCandidatePayload()
  if (payload === null) return

  const confidence = Math.max(0, Math.min(10_000, Number(candidateConfidence.value) || 0))
  sourceActionLoading.value = 'candidate'
  error.value = ''
  success.value = ''
  try {
    await addCurriculumParseCandidate(source.id, {
      candidate_type: candidateType.value,
      parent_candidate_id: null,
      raw_label: rawLabel,
      normalized_label: candidateNormalizedLabel.value.trim() || null,
      payload,
      confidence_score: confidence,
    })
    success.value = 'Extraction candidate added.'
    candidateRawLabel.value = ''
    candidateNormalizedLabel.value = ''
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not add extraction candidate.')
  } finally {
    sourceActionLoading.value = ''
  }
}

async function finalizeSelectedSource() {
  const source = selectedSource.value
  if (!source || sourceActionLoading.value) return
  if (!window.confirm(`Finalize extraction for "${source.title}"?`)) return

  sourceActionLoading.value = 'finalize'
  error.value = ''
  success.value = ''
  try {
    selectedReport.value = await finalizeCurriculumSource(source.id)
    success.value = 'Source extraction finalized.'
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not finalize source extraction.')
  } finally {
    sourceActionLoading.value = ''
  }
}

async function markSelectedSourceReviewed() {
  const source = selectedSource.value
  if (!source || sourceActionLoading.value) return
  if (!window.confirm(`Mark "${source.title}" as reviewed?`)) return

  sourceActionLoading.value = 'review'
  error.value = ''
  success.value = ''
  try {
    selectedReport.value = await markCurriculumSourceReviewed(source.id)
    success.value = 'Source marked reviewed.'
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not mark source reviewed.')
  } finally {
    sourceActionLoading.value = ''
  }
}

async function queueSelectedSourceJobs() {
  const source = selectedSource.value
  if (!source || sourceActionLoading.value) return

  sourceActionLoading.value = 'queue'
  error.value = ''
  success.value = ''
  try {
    const jobs = await queueSourceFollowUpJobs(source.id)
    success.value = `${jobs.length} follow-up job${jobs.length === 1 ? '' : 's'} queued.`
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not queue follow-up jobs.')
  } finally {
    sourceActionLoading.value = ''
  }
}

async function runNext() {
  runningJobId.value = -1
  error.value = ''
  try {
    const result = await runNextFoundryJob(null)
    success.value = result ? `Ran job #${result.id}.` : 'No queued foundry job is ready.'
    await load()
  } catch (err) {
    error.value = messageFromError(err, 'Could not run next job.')
  } finally {
    runningJobId.value = null
  }
}

async function runJob(jobId: number) {
  runningJobId.value = jobId
  error.value = ''
  try {
    const result = await runFoundryJob(jobId)
    success.value = `Ran job #${result.id}.`
    await load()
  } catch (err) {
    error.value = messageFromError(err, `Could not run job #${jobId}.`)
  } finally {
    runningJobId.value = null
  }
}

onMounted(load)
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Sources & Ingestion</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          Register raw material, inspect extracted structure, and send source work into review before content editing.
        </p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
        <AppButton variant="secondary" size="sm" @click="router.push('/admin/content-editor')">Content Editor</AppButton>
        <AppButton variant="primary" size="sm" :loading="runningJobId === -1" @click="runNext">Run Next Job</AppButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <CmsMetricStrip :items="pipelineCounts" class="mb-6" />

    <div class="grid grid-cols-1 2xl:grid-cols-[320px_minmax(0,1fr)_430px] gap-5">
      <div class="space-y-4">
        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Register Raw Source</h2>
          <div class="grid grid-cols-2 gap-3 mb-3">
            <AppSelect
              v-model="sourceKind"
              label="Source Type"
              :options="[
                { value: 'worksheet', label: 'Worksheet' },
                { value: 'past_question', label: 'Past Question' },
                { value: 'curriculum', label: 'Curriculum' },
                { value: 'syllabus', label: 'Syllabus' },
                { value: 'guide', label: 'Guide' },
                { value: 'textbook', label: 'Textbook' },
                { value: 'web_source', label: 'Web Source' },
              ]"
            />
            <AppInput v-model="subjectCode" label="Subject Code" placeholder="MTH" />
          </div>
          <AppInput v-model="titlePrefix" label="Title Override" placeholder="Leave blank to use file name" class="mb-3" />
          <UploadDropzone @files="handleFiles" />
          <p class="text-[11px] mt-3" :style="{ color: 'var(--ink-muted)' }">
            PDFs, images, Word documents, text files, CSVs, and JSON start here. The source stays inspectable before any generated content reaches the editor.
          </p>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Selected Source</h2>
          <div v-if="selectedSource" class="space-y-2">
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ selectedSource.title }}</p>
            <div class="flex flex-wrap gap-2">
              <AppBadge color="muted" size="xs">{{ selectedSource.source_kind }}</AppBadge>
              <CmsStatusBadge :status="selectedSource.source_status" />
              <AppBadge :color="selectedSource.stale_flag ? 'gold' : 'success'" size="xs">
                {{ selectedSource.stale_flag ? 'Stale' : 'Fresh' }}
              </AppBadge>
            </div>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
              Trust {{ formatBp(selectedSource.trust_score_bp) }} / freshness {{ formatBp(selectedSource.freshness_score_bp) }}
            </p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
              {{ selectedSource.source_path ?? 'No local path' }}
            </p>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Select a source to inspect extraction data.</p>
        </AppCard>
      </div>

      <div class="space-y-4">
        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Source Registry</h2>
            <AppBadge color="muted" size="xs">{{ sources.length }} shown</AppBadge>
          </div>
          <div v-if="loading" class="space-y-2">
            <div v-for="i in 5" :key="i" class="h-12 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else class="space-y-2 max-h-[410px] overflow-y-auto pr-1">
            <button
              v-for="source in sources"
              :key="source.id"
              type="button"
              class="w-full flex items-center gap-3 rounded-lg px-3 py-2 text-left transition"
              :class="selectedSourceId === source.id ? 'ring-1 ring-[var(--primary)]' : ''"
              :style="{ backgroundColor: selectedSourceId === source.id ? 'var(--primary-light)' : 'var(--paper)' }"
              @click="selectSource(source)"
            >
              <AppBadge color="muted" size="xs">{{ source.source_kind }}</AppBadge>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium truncate" :style="{ color: 'var(--ink)' }">{{ source.title }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ source.subject_code ?? 'No subject' }} / trust {{ formatBp(source.trust_score_bp) }} / {{ source.parse_status_detail ?? 'uploaded' }}
                </p>
              </div>
              <CmsStatusBadge :status="source.source_status" />
            </button>
            <p v-if="!sources.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No sources registered yet.</p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Foundry Jobs</h2>
            <div class="flex gap-2">
              <AppBadge color="gold" size="xs">{{ board?.queued_count ?? 0 }} queued</AppBadge>
              <AppBadge color="danger" size="xs">{{ board?.failed_count ?? 0 }} failed</AppBadge>
            </div>
          </div>
          <div class="space-y-2 max-h-[290px] overflow-y-auto pr-1">
            <div v-for="job in board?.jobs ?? []" :key="job.id" class="flex items-center gap-3 rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
              <AppBadge :color="job.status === 'completed' ? 'success' : job.status === 'failed' ? 'danger' : 'gold'" size="xs">#{{ job.id }}</AppBadge>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium truncate" :style="{ color: 'var(--ink)' }">{{ job.job_type }} / {{ job.target_type }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ job.trigger_type }} / priority {{ job.priority }}</p>
              </div>
              <AppButton
                variant="secondary"
                size="sm"
                :loading="runningJobId === job.id"
                :disabled="job.status === 'completed'"
                @click="runJob(job.id)"
              >
                Run
              </AppButton>
            </div>
            <p v-if="!board?.jobs?.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No foundry jobs yet.</p>
          </div>
        </AppCard>
      </div>

      <div class="space-y-4">
        <AppCard padding="md">
          <div class="flex items-start justify-between gap-3 mb-3">
            <div>
              <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Extraction Workbench</h2>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                Review candidates before content editor work begins.
              </p>
            </div>
            <AppBadge color="accent" size="xs">{{ selectedReport?.publish_readiness_score ?? 0 }} bp ready</AppBadge>
          </div>

          <div v-if="!selectedSource" class="rounded-lg px-3 py-4 text-sm" :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }">
            Select a source to see extraction candidates, review tasks, segments, and governance history.
          </div>

          <div v-else-if="sourceDetailLoading" class="space-y-2">
            <div v-for="i in 4" :key="i" class="h-12 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>

          <div v-else class="space-y-4">
            <CmsMetricStrip :items="selectedReportMetrics" />

            <div class="grid grid-cols-2 gap-2">
              <div class="rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Segments</p>
                <p class="text-lg font-bold" :style="{ color: 'var(--ink)' }">{{ selectedDetail?.segments.length ?? 0 }}</p>
              </div>
              <div class="rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
                <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Evidence</p>
                <p class="text-lg font-bold" :style="{ color: 'var(--ink)' }">{{ selectedDetail?.evidence_records.length ?? 0 }}</p>
              </div>
            </div>

            <div v-if="selectedReport?.recommended_actions.length" class="rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-[10px] uppercase mb-1" :style="{ color: 'var(--ink-muted)' }">Recommended Actions</p>
              <ul class="space-y-1">
                <li v-for="action in selectedReport.recommended_actions" :key="action" class="text-xs" :style="{ color: 'var(--ink)' }">
                  {{ action }}
                </li>
              </ul>
            </div>

            <div class="space-y-3 rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <div class="grid grid-cols-2 gap-3">
                <AppSelect
                  v-model="candidateType"
                  label="Candidate Type"
                  :options="[
                    { value: 'topic', label: 'Topic' },
                    { value: 'subtopic', label: 'Subtopic' },
                    { value: 'outcome', label: 'Outcome' },
                    { value: 'skill', label: 'Skill' },
                    { value: 'question_family', label: 'Question Family' },
                  ]"
                />
                <AppInput v-model="candidateConfidence" label="Confidence BP" type="number" placeholder="8100" />
              </div>
              <AppInput v-model="candidateRawLabel" label="Raw Label" placeholder="Text exactly as extracted" />
              <AppInput v-model="candidateNormalizedLabel" label="Normalized Label" placeholder="Optional cleaned label" />
              <AppTextarea v-model="candidatePayloadText" label="Payload JSON" :rows="5" />
              <AppButton
                variant="primary"
                size="sm"
                class="w-full"
                :loading="sourceActionLoading === 'candidate'"
                :disabled="sourceActionLoading !== ''"
                @click="addCandidate"
              >
                Add Candidate
              </AppButton>
            </div>

            <div class="flex flex-wrap gap-2">
              <AppButton
                variant="secondary"
                size="sm"
                :loading="sourceActionLoading === 'finalize'"
                :disabled="sourceActionLoading !== ''"
                @click="finalizeSelectedSource"
              >
                Finalize Parse
              </AppButton>
              <AppButton
                variant="secondary"
                size="sm"
                :loading="sourceActionLoading === 'queue'"
                :disabled="sourceActionLoading !== ''"
                @click="queueSelectedSourceJobs"
              >
                Queue Jobs
              </AppButton>
              <AppButton
                variant="primary"
                size="sm"
                :loading="sourceActionLoading === 'review'"
                :disabled="sourceActionLoading !== '' || !selectedReport?.can_mark_reviewed"
                @click="markSelectedSourceReviewed"
              >
                Mark Reviewed
              </AppButton>
            </div>
          </div>
        </AppCard>

        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Candidate List</h2>
            <AppBadge color="muted" size="xs">{{ selectedReport?.candidate_counts.length ?? 0 }} groups</AppBadge>
          </div>
          <div class="space-y-2 max-h-[360px] overflow-y-auto pr-1">
            <div
              v-for="candidate in selectedReport?.parse_candidates ?? []"
              :key="candidate.id"
              class="rounded-lg px-3 py-2"
              :style="{ backgroundColor: 'var(--paper)' }"
            >
              <div class="flex items-center justify-between gap-2 mb-1">
                <p class="text-sm font-medium truncate" :style="{ color: 'var(--ink)' }">{{ candidate.raw_label }}</p>
                <AppBadge :color="candidateTone(candidate.review_status)" size="xs">{{ candidate.review_status }}</AppBadge>
              </div>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                {{ candidate.candidate_type }} / {{ candidate.normalized_label ?? 'not normalized' }} / {{ formatBp(candidate.confidence_score) }}
              </p>
            </div>
            <p v-if="!selectedReport?.parse_candidates.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
              No extraction candidates yet.
            </p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Review & Governance</h2>
            <AppBadge color="gold" size="xs">{{ selectedReport?.review_tasks.length ?? 0 }} tasks</AppBadge>
          </div>
          <div class="space-y-2">
            <div v-for="task in selectedReport?.review_tasks ?? []" :key="task.id" class="rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
              <div class="flex items-center justify-between gap-2 mb-1">
                <p class="text-sm font-medium" :style="{ color: 'var(--ink)' }">{{ task.task_type }}</p>
                <AppBadge :color="task.status === 'resolved' ? 'success' : 'gold'" size="xs">{{ task.severity }}</AppBadge>
              </div>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ task.notes ?? task.status }}</p>
            </div>
            <div v-for="event in selectedDetail?.governance_events ?? []" :key="`event-${event.id}`" class="rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-sm font-medium" :style="{ color: 'var(--ink)' }">{{ event.source_status }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ event.note ?? event.created_at }}</p>
            </div>
            <p v-if="!selectedReport?.review_tasks.length && !selectedDetail?.governance_events.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
              No review tasks or governance events yet.
            </p>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
