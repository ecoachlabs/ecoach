<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useAuthStore } from '@/stores/auth'
import {
  addSubmissionBundleFile,
  applySubmissionBundleToCoach,
  createSubmissionBundle,
  getPersonalAcademicVault,
  getSubmissionBundleReport,
  getUploadedPaperReview,
  listSubmissionBundleInbox,
  reconstructSubmissionBundle,
  type BundleInboxItemDto,
  type BundleProcessReportDto,
  type PersonalAcademicVaultSnapshotDto,
  type UploadedPaperReviewSnapshotDto,
} from '@/ipc/intake'

const auth = useAuthStore()

const loading = ref(true)
const processing = ref(false)
const applying = ref(false)
const error = ref('')
const success = ref('')
const title = ref('')
const manualPaths = ref('')
const filePaths = ref<string[]>([])
const activeBundleId = ref<number | null>(null)
const report = ref<BundleProcessReportDto | null>(null)
const review = ref<UploadedPaperReviewSnapshotDto | null>(null)
const coachSummary = ref<string[]>([])
const inbox = ref<BundleInboxItemDto[]>([])
const vault = ref<PersonalAcademicVaultSnapshotDto | null>(null)

let unlistenDrop: (() => void) | null = null

const activeBundle = computed(() =>
  vault.value?.bundles.find(entry => entry.bundle.id === activeBundleId.value) ?? null,
)

onMounted(() => {
  void initialize()
})

onUnmounted(() => {
  if (unlistenDrop) {
    unlistenDrop()
    unlistenDrop = null
  }
})

async function initialize() {
  await refreshVault()

  try {
    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'drop') {
        addPaths(event.payload.paths)
      }
    })
  } catch {
    unlistenDrop = null
  }
}

async function refreshVault() {
  const studentId = auth.currentAccount?.id
  if (!studentId) {
    loading.value = false
    return
  }

  loading.value = true
  try {
    const [nextInbox, nextVault] = await Promise.all([
      listSubmissionBundleInbox(studentId, 6).catch(() => []),
      getPersonalAcademicVault(studentId, 16).catch(() => null),
    ])
    inbox.value = nextInbox
    vault.value = nextVault
  } finally {
    loading.value = false
  }
}

function addPaths(paths: string[]) {
  const normalized = paths
    .map(path => path.trim())
    .filter(Boolean)
    .filter(path => !filePaths.value.includes(path))

  if (!normalized.length) return

  filePaths.value = [...filePaths.value, ...normalized]
  if (!title.value.trim()) {
    title.value = baseName(normalized[0])
  }
}

function addManualPaths() {
  addPaths(
    manualPaths.value
      .split(/\r?\n/)
      .map(path => path.trim())
      .filter(Boolean),
  )
  manualPaths.value = ''
}

function removePath(path: string) {
  filePaths.value = filePaths.value.filter(candidate => candidate !== path)
}

async function processBundle() {
  const studentId = auth.currentAccount?.id
  if (!studentId || !filePaths.value.length || processing.value) return

  processing.value = true
  error.value = ''
  success.value = ''
  coachSummary.value = []

  try {
    const bundle = await createSubmissionBundle(
      studentId,
      title.value.trim() || `Evidence bundle (${filePaths.value.length} files)`,
    )

    activeBundleId.value = bundle.id

    for (const path of filePaths.value) {
      await addSubmissionBundleFile(bundle.id, baseName(path), path)
    }

    report.value = await reconstructSubmissionBundle(bundle.id)
    review.value = await getUploadedPaperReview(bundle.id).catch(() => null)
    success.value = 'Evidence bundle processed.'
    await refreshVault()
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to process evidence bundle'
  } finally {
    processing.value = false
  }
}

async function openBundle(bundleId: number) {
  activeBundleId.value = bundleId
  error.value = ''

  try {
    report.value = await getSubmissionBundleReport(bundleId)
    review.value = await getUploadedPaperReview(bundleId).catch(() => null)
    coachSummary.value = []
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to load this bundle'
  }
}

async function applyToCoach() {
  if (activeBundleId.value == null || applying.value) return

  applying.value = true
  error.value = ''
  success.value = ''

  try {
    const result = await applySubmissionBundleToCoach(activeBundleId.value)
    coachSummary.value = result.summary
    success.value = 'Coach systems updated from this evidence bundle.'
    await refreshVault()
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to apply this bundle to coach'
  } finally {
    applying.value = false
  }
}

function baseName(path: string) {
  const segments = path.split(/[\\/]/)
  return segments[segments.length - 1] || path
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pt-6 pb-5"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Evidence Intake</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Upload Evidence
      </h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        Feed real schoolwork into your vault, review stack, and coach plan.
      </p>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="success" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(13,148,136,0.08)', color: 'var(--accent)' }">
      {{ success }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="index in 4" :key="index" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 flex overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="grid gap-4 md:grid-cols-4">
            <div class="stat-card">
              <span class="stat-label">Vault Bundles</span>
              <strong class="stat-value">{{ vault?.total_bundle_count ?? 0 }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Pending Review</span>
              <strong class="stat-value">{{ vault?.pending_review_count ?? 0 }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Coach Applied</span>
              <strong class="stat-value">{{ vault?.coach_applied_count ?? 0 }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Promoted</span>
              <strong class="stat-value">{{ vault?.promoted_bundle_count ?? 0 }}</strong>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-3">New Bundle</p>

          <div class="space-y-4">
            <div>
              <label class="mini-label mb-2 block">Bundle Title</label>
              <input
                v-model="title"
                type="text"
                class="text-input"
                placeholder="Midterm corrections, class test, mock paper"
              />
            </div>

            <div class="dropzone">
              <div class="drop-icon">EV</div>
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">
                Drop files into the app window
              </p>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
                Each dropped file is captured with its real desktop path.
              </p>
            </div>

            <div>
              <label class="mini-label mb-2 block">Paste File Paths</label>
              <textarea
                v-model="manualPaths"
                class="text-area"
                placeholder="C:\Users\surfaceSudio\Downloads\bece-math-paper.pdf"
              />
              <div class="mt-3 flex gap-2">
                <button class="secondary-btn" @click="addManualPaths">Add Paths</button>
                <button class="secondary-btn" @click="filePaths = []">Clear Queue</button>
              </div>
            </div>

            <div v-if="filePaths.length" class="space-y-2">
              <p class="mini-label">{{ filePaths.length }} files queued</p>
              <div class="space-y-2">
                <div v-for="path in filePaths" :key="path" class="path-row">
                  <div class="min-w-0">
                    <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ baseName(path) }}</p>
                    <p class="text-[11px] truncate" :style="{ color: 'var(--ink-muted)' }">{{ path }}</p>
                  </div>
                  <button class="remove-btn" @click="removePath(path)">Remove</button>
                </div>
              </div>
            </div>

            <button class="primary-btn" :disabled="!filePaths.length || processing" @click="processBundle">
              {{ processing ? 'Processing bundle...' : 'Process Evidence Bundle' }}
            </button>
          </div>
        </section>

        <section
          v-if="report"
          class="rounded-2xl border p-5"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <div class="flex items-center justify-between gap-4 mb-4">
            <div>
              <p class="section-label">Bundle Report</p>
              <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ report.bundle.title }}</p>
            </div>
            <span class="report-pill">{{ report.review_priority }}</span>
          </div>

          <div class="grid gap-3 md:grid-cols-4">
            <div class="stat-card">
              <span class="stat-label">Files</span>
              <strong class="stat-value">{{ report.files.length }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Question Blocks</span>
              <strong class="stat-value">{{ report.extracted_question_block_count }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Aligned Pairs</span>
              <strong class="stat-value">{{ report.aligned_question_pair_count }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Confidence</span>
              <strong class="stat-value">{{ Math.round(report.reconstruction_confidence_score / 100) }}%</strong>
            </div>
          </div>

          <div class="mt-5 grid gap-5 md:grid-cols-2">
            <div>
              <p class="mini-label mb-2">Detected Subjects</p>
              <div class="flex flex-wrap gap-2">
                <span v-for="subject in report.detected_subjects" :key="subject" class="signal-pill">{{ subject }}</span>
                <span v-if="!report.detected_subjects.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
                  No subject signal yet
                </span>
              </div>
            </div>

            <div>
              <p class="mini-label mb-2">Detected Topics</p>
              <div class="flex flex-wrap gap-2">
                <span v-for="topic in report.detected_topics" :key="topic" class="signal-pill">{{ topic }}</span>
                <span v-if="!report.detected_topics.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
                  No topic signal yet
                </span>
              </div>
            </div>
          </div>

          <div class="mt-5 grid gap-5 md:grid-cols-2">
            <div>
              <p class="mini-label mb-2">Weakness Signals</p>
              <ul class="space-y-2">
                <li v-for="signal in report.weakness_signals" :key="signal" class="detail-line">{{ signal }}</li>
                <li v-if="!report.weakness_signals.length" class="detail-line muted">No weakness signal yet</li>
              </ul>
            </div>

            <div>
              <p class="mini-label mb-2">Recommended Actions</p>
              <ul class="space-y-2">
                <li v-for="action in report.recommended_actions" :key="action" class="detail-line">{{ action }}</li>
                <li v-if="!report.recommended_actions.length" class="detail-line muted">No recommended action yet</li>
              </ul>
            </div>
          </div>
        </section>

        <section
          v-if="review?.items.length"
          class="rounded-2xl border p-5"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <p class="section-label mb-4">Smart Review</p>
          <div class="space-y-3">
            <div v-for="item in review.items.slice(0, 6)" :key="item.question_ref" class="review-row">
              <div>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ item.question_ref }}</p>
                <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">{{ item.coach_explanation }}</p>
              </div>
              <span class="review-chip">{{ item.alignment_confidence }}</span>
            </div>
          </div>
        </section>
      </div>

      <aside
        class="w-80 flex-shrink-0 border-l overflow-y-auto p-5 space-y-5"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Inbox</p>
          <div v-if="inbox.length" class="space-y-3">
            <div v-for="item in inbox" :key="item.bundle.id" class="inbox-row">
              <div class="min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ item.bundle.title }}</p>
                <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">
                  {{ item.review_priority }} · {{ item.coach_application_status }}
                </p>
              </div>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No review inbox items yet.</p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <div class="flex items-center justify-between mb-3">
            <p class="section-label">Vault</p>
            <button class="text-btn" @click="refreshVault">Refresh</button>
          </div>

          <div v-if="vault?.bundles.length" class="space-y-2">
            <button
              v-for="entry in vault.bundles.slice(0, 8)"
              :key="entry.bundle.id"
              class="vault-row"
              :class="{ active: activeBundleId === entry.bundle.id }"
              @click="openBundle(entry.bundle.id)"
            >
              <div class="min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ entry.bundle.title }}</p>
                <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">
                  {{ entry.file_count }} files · {{ entry.review_priority }}
                </p>
              </div>
            </button>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Your personal academic vault is empty.</p>
        </section>

        <section
          v-if="activeBundle"
          class="rounded-2xl border p-4"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }"
        >
          <p class="section-label mb-3">Coach Sync</p>
          <div class="space-y-3">
            <div class="detail-row">
              <span>Status</span>
              <strong>{{ activeBundle.coach_application_status }}</strong>
            </div>
            <div class="detail-row">
              <span>Confirmation</span>
              <strong>{{ activeBundle.confirmation_state }}</strong>
            </div>
            <button class="primary-btn" :disabled="applying" @click="applyToCoach">
              {{ applying ? 'Applying...' : 'Apply To Coach' }}
            </button>
            <ul v-if="coachSummary.length" class="space-y-2">
              <li v-for="line in coachSummary" :key="line" class="detail-line">{{ line }}</li>
            </ul>
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

.text-input,
.text-area {
  width: 100%;
  border-radius: 12px;
  border: 1px solid var(--border-soft);
  background: var(--paper);
  color: var(--ink);
  padding: 12px 14px;
  font-size: 14px;
  outline: none;
}

.text-area {
  min-height: 110px;
  resize: vertical;
}

.dropzone {
  border: 1px dashed var(--accent);
  border-radius: 16px;
  padding: 28px 20px;
  text-align: center;
  background: color-mix(in srgb, var(--accent-glow) 65%, var(--surface));
}

.drop-icon {
  width: 52px;
  height: 52px;
  margin: 0 auto 12px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--paper);
  color: var(--accent);
  font-size: 14px;
  font-weight: 900;
}

.primary-btn,
.secondary-btn,
.text-btn,
.remove-btn,
.vault-row {
  border: 1px solid transparent;
  transition: all 120ms;
}

.primary-btn,
.secondary-btn {
  padding: 10px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
}

.primary-btn {
  width: 100%;
  background: var(--ink);
  color: var(--paper);
}

.primary-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.secondary-btn,
.text-btn,
.remove-btn,
.vault-row {
  background: var(--surface);
  color: var(--ink-secondary);
}

.secondary-btn:hover,
.text-btn:hover,
.remove-btn:hover,
.vault-row:hover,
.vault-row.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.text-btn,
.remove-btn {
  border-radius: 999px;
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 700;
}

.path-row,
.review-row,
.inbox-row,
.vault-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 12px 14px;
  border-radius: 14px;
  background: var(--paper);
}

.stat-card {
  padding: 16px;
  border-radius: 14px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stat-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
}

.stat-value {
  font-size: 22px;
  font-weight: 900;
  color: var(--ink);
}

.signal-pill,
.report-pill,
.review-chip {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  font-size: 11px;
  font-weight: 700;
}

.detail-line {
  font-size: 13px;
  line-height: 1.55;
  color: var(--ink);
}

.detail-line.muted {
  color: var(--ink-muted);
}

.detail-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: var(--ink-muted);
}

.detail-row strong {
  color: var(--ink);
}
</style>
