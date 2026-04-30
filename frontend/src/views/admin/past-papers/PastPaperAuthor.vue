<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/curriculum'
import {
  adminSavePastPaper,
  adminGetPastPaper,
  adminExtractPastPaperText,
  adminAttachQuestionAsset,
  adminDeleteQuestionAsset,
  fetchQuestionAssetObjectUrl,
  type AdminPastPaperSaveInput,
  type AdminPastPaperQuestionInput,
  type AdminPastPaperOptionInput,
  type QuestionAssetMetaDto,
  type QuestionAssetScope,
  type RecoveredTextDto,
} from '@/ipc/pastPaperAdmin'
import { onBeforeUnmount } from 'vue'
import { segmentPastPaperText, type DraftPaper } from '@/utils/pastPaperSegmenter'
import {
  parsePastPaperCsv,
  generatePastPaperCsvTemplate,
  type CsvImportResult,
} from '@/utils/pastPaperCsvParser'

const route = useRoute()
const router = useRouter()

// ── Route state ─────────────────────────────────────────────────────
const paperIdParam = computed(() => {
  const raw = route.params.id
  if (!raw || raw === 'new') return null
  const n = Number(raw)
  return Number.isFinite(n) && n > 0 ? n : null
})

// ── Paper metadata ──────────────────────────────────────────────────
const subjectId = ref<number | null>(null)
const examYear = ref<number>(new Date().getFullYear())
const paperCode = ref<string>('')
const title = ref<string>('')

// ── Questions working copy ──────────────────────────────────────────
type EditableQuestion = AdminPastPaperQuestionInput & {
  _local_id: string
  /** Visible-to-admin question category. Maps to { question_format,
   *  primary_pedagogic_function } when we save. See uiTypeToDbFormat /
   *  dbFormatToUiType. */
  _ui_type: UiType
  /** Segmenter confidence — rendered as a red dot when true. Not persisted. */
  _is_ambiguous: boolean
  /** Image attachments already on disk (present once the question has a
   *  DB id). Bytes are fetched lazily into `_asset_urls` for <img>. */
  _assets: QuestionAssetMetaDto[]
}

type UiType =
  | 'mcq'         // multiple choice, SINGLE correct
  | 'mcq_multi'   // multiple choice, MULTIPLE correct
  | 'short_answer'
  | 'numeric'
  | 'true_false'
  | 'essay'
  | 'fill_blank'

/** Map UI-facing type onto what the DB will store. Essay and fill-blank
 *  both persist as `short_answer`; we distinguish them in
 *  `primary_pedagogic_function` because the `questions.question_format`
 *  CHECK constraint was locked down in migration 003 and widening it
 *  would need a risky table rebuild. */
function uiTypeToDbFormat(ui: UiType): { question_format: string; pedagogic: string | null } {
  switch (ui) {
    case 'mcq':          return { question_format: 'mcq', pedagogic: 'mcq_single' }
    case 'mcq_multi':    return { question_format: 'mcq', pedagogic: 'mcq_multi' }
    case 'short_answer': return { question_format: 'short_answer', pedagogic: 'short_response' }
    case 'numeric':      return { question_format: 'numeric', pedagogic: null }
    case 'true_false':   return { question_format: 'true_false', pedagogic: null }
    case 'essay':        return { question_format: 'short_answer', pedagogic: 'essay_response' }
    case 'fill_blank':   return { question_format: 'short_answer', pedagogic: 'fill_blank' }
  }
}

/** Reverse the mapping when we load an existing paper. MCQ uses option
 *  count to distinguish single/multi so historic rows still work. */
function dbFormatToUiType(
  question_format: string,
  pedagogic: string | null,
  options: readonly { is_correct: boolean }[],
): UiType {
  if (question_format === 'mcq') {
    if (pedagogic === 'mcq_multi') return 'mcq_multi'
    if (pedagogic === 'mcq_single') return 'mcq'
    return options.filter(o => o.is_correct).length > 1 ? 'mcq_multi' : 'mcq'
  }
  if (question_format === 'true_false') return 'true_false'
  if (question_format === 'numeric') return 'numeric'
  if (question_format === 'short_answer') {
    if (pedagogic === 'essay_response') return 'essay'
    if (pedagogic === 'fill_blank') return 'fill_blank'
    return 'short_answer'
  }
  return 'short_answer'
}

const questions = ref<EditableQuestion[]>([])

function nextLocalId(): string {
  return `q_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
}

function blankQuestion(overrides: Partial<EditableQuestion> = {}): EditableQuestion {
  return {
    _local_id: nextLocalId(),
    _ui_type: 'mcq',
    _is_ambiguous: false,
    _assets: [],
    question_id: null,
    section_label: 'A',
    question_number: null,
    topic_id: topics.value[0]?.id ?? 0,
    subtopic_id: null,
    stem: '',
    question_format: 'mcq',
    primary_pedagogic_function: 'mcq_single',
    explanation_text: null,
    difficulty_level: 5000,
    marks: 1,
    options: [
      { option_id: null, option_label: 'A', option_text: '', is_correct: true },
      { option_id: null, option_label: 'B', option_text: '', is_correct: false },
      { option_id: null, option_label: 'C', option_text: '', is_correct: false },
      { option_id: null, option_label: 'D', option_text: '', is_correct: false },
    ],
    ...overrides,
  }
}

function addQuestion(): void {
  questions.value.push(blankQuestion({ section_label: lastSectionLabel.value }))
}

function removeQuestion(localId: string): void {
  questions.value = questions.value.filter(q => q._local_id !== localId)
}

function duplicateQuestion(localId: string): void {
  const idx = questions.value.findIndex(q => q._local_id === localId)
  if (idx < 0) return
  const src = questions.value[idx]
  const copy: EditableQuestion = {
    ...src,
    _local_id: nextLocalId(),
    question_id: null,
    options: src.options.map(o => ({ ...o })),
  }
  questions.value.splice(idx + 1, 0, copy)
}

function moveQuestion(localId: string, direction: -1 | 1): void {
  const idx = questions.value.findIndex(q => q._local_id === localId)
  const target = idx + direction
  if (idx < 0 || target < 0 || target >= questions.value.length) return
  const arr = [...questions.value]
  ;[arr[idx], arr[target]] = [arr[target], arr[idx]]
  questions.value = arr
}

const lastSectionLabel = computed(() => {
  const last = questions.value[questions.value.length - 1]
  return last?.section_label ?? 'A'
})

// ── Option helpers (MCQ — single AND multi) ─────────────────────────
function addOption(q: EditableQuestion): void {
  const label = 'ABCDEFGH'[q.options.length] ?? String(q.options.length + 1)
  q.options = [...q.options, { option_id: null, option_label: label, option_text: '', is_correct: false }]
}
function removeOption(q: EditableQuestion, idx: number): void {
  if (q.options.length <= 2) return
  const removed = q.options[idx]
  q.options = q.options.filter((_, i) => i !== idx)
  if (removed.is_correct && !q.options.some(o => o.is_correct)) {
    q.options[0].is_correct = true
  }
  q.options = q.options.map((o, i) => ({
    ...o,
    option_label: 'ABCDEFGH'[i] ?? String(i + 1),
  }))
}

/** Single-correct MCQ: one option on, rest off. */
function setCorrectOption(q: EditableQuestion, idx: number): void {
  q.options = q.options.map((o, i) => ({ ...o, is_correct: i === idx }))
}

/** Multi-correct MCQ: toggle this one, leave the others alone. Guards
 *  against zero-correct state by keeping one on if the admin tries to
 *  un-toggle the only correct answer. */
function toggleCorrectOption(q: EditableQuestion, idx: number): void {
  const next = q.options.map((o, i) => i === idx ? { ...o, is_correct: !o.is_correct } : o)
  if (!next.some(o => o.is_correct)) next[idx].is_correct = true
  q.options = next
}

// ── UI-type switcher ────────────────────────────────────────────────
/** Regenerate default options/stem when the admin flips the question
 *  category. Preserves existing stem + topic; replaces the answer area. */
function setUiType(q: EditableQuestion, next: UiType): void {
  if (q._ui_type === next) return
  q._ui_type = next
  const { question_format, pedagogic } = uiTypeToDbFormat(next)
  q.question_format = question_format
  q.primary_pedagogic_function = pedagogic

  if (next === 'mcq' || next === 'mcq_multi') {
    if (q.options.length < 2) {
      q.options = [
        { option_id: null, option_label: 'A', option_text: '', is_correct: true },
        { option_id: null, option_label: 'B', option_text: '', is_correct: false },
        { option_id: null, option_label: 'C', option_text: '', is_correct: false },
        { option_id: null, option_label: 'D', option_text: '', is_correct: false },
      ]
    }
    // Enforce single-correct if the admin switched AWAY from multi.
    if (next === 'mcq' && q.options.filter(o => o.is_correct).length > 1) {
      let first = true
      q.options = q.options.map(o => {
        if (o.is_correct && first) { first = false; return o }
        return { ...o, is_correct: false }
      })
    }
  } else if (next === 'true_false') {
    q.options = [
      { option_id: null, option_label: 'TRUE', option_text: 'True', is_correct: true },
      { option_id: null, option_label: 'FALSE', option_text: 'False', is_correct: false },
    ]
  } else if (next === 'fill_blank') {
    // One blank seed. Admin adds more with the toolbar.
    q.options = [{ option_id: null, option_label: '1', option_text: '', is_correct: true }]
    if (!/\[\[\d+\]\]/.test(q.stem)) q.stem = (q.stem + ' [[1]]').trim()
  } else {
    // short_answer / numeric / essay: answer/guide lives in explanation_text,
    // no option rows.
    q.options = []
  }

  // Marks defaults per type — admin can override.
  if (next === 'essay') q.marks = Math.max(q.marks ?? 0, 6)
  if (next === 'short_answer' || next === 'numeric') q.marks = Math.min(q.marks ?? 2, 3)
  if (next === 'mcq' || next === 'mcq_multi' || next === 'true_false') q.marks = 1
}

// ── Fill-in-the-blank tooling ───────────────────────────────────────
function nextBlankIndex(q: EditableQuestion): number {
  const numbers = Array.from(q.stem.matchAll(/\[\[(\d+)\]\]/g)).map(m => Number(m[1]))
  return numbers.length === 0 ? 1 : Math.max(...numbers) + 1
}

function insertBlank(q: EditableQuestion): void {
  const n = nextBlankIndex(q)
  q.stem = (q.stem.endsWith(' ') || q.stem.length === 0 ? q.stem : q.stem + ' ') + `[[${n}]]`
  q.options = [
    ...q.options,
    { option_id: null, option_label: String(n), option_text: '', is_correct: true },
  ]
}

/** Blanks grouped by index — multiple rows per position = alternatives. */
function blankGroups(q: EditableQuestion): Array<{ index: number; alternatives: number[] }> {
  const byIndex = new Map<number, number[]>()
  q.options.forEach((o, i) => {
    const n = Number(o.option_label) || 0
    if (!byIndex.has(n)) byIndex.set(n, [])
    byIndex.get(n)!.push(i)
  })
  return Array.from(byIndex.entries())
    .map(([index, alternatives]) => ({ index, alternatives }))
    .sort((a, b) => a.index - b.index)
}

function addAlternativeToBlank(q: EditableQuestion, blankIndex: number): void {
  q.options = [
    ...q.options,
    { option_id: null, option_label: String(blankIndex), option_text: '', is_correct: true },
  ]
}

function removeBlank(q: EditableQuestion, blankIndex: number): void {
  q.stem = q.stem.replace(new RegExp(`\\s*\\[\\[${blankIndex}\\]\\]`, 'g'), '')
  q.options = q.options.filter(o => Number(o.option_label) !== blankIndex)
}

// ── Image attachments ───────────────────────────────────────────────
//
// The object-URL cache lives at module scope (per component instance)
// so thumbnails persist across rerenders. We revoke all URLs on unmount
// to avoid leaking memory when the admin bounces between papers.
const assetUrlCache = ref<Record<number, string>>({})
const assetBusy = ref<Set<number>>(new Set())

async function ensureAssetUrl(assetId: number): Promise<string> {
  const existing = assetUrlCache.value[assetId]
  if (existing) return existing
  try {
    const url = await fetchQuestionAssetObjectUrl(assetId)
    assetUrlCache.value = { ...assetUrlCache.value, [assetId]: url }
    return url
  } catch {
    return ''
  }
}

function mimeLabel(mime: string): string {
  if (mime.startsWith('image/')) return mime.slice(6).toUpperCase()
  return mime
}

function scopeAssets(
  q: EditableQuestion,
  scope: QuestionAssetScope,
  scopeRef: number | null = null,
): QuestionAssetMetaDto[] {
  return q._assets.filter(a => {
    if (a.scope !== scope) return false
    if (scope === 'option') return a.scope_ref === scopeRef
    return true
  })
}

/** Open a hidden file input, read bytes, POST to the backend, and
 *  append the returned metadata to the question's asset list.
 *  Guarded by question_id !== null — assets live on DB rows, and we
 *  can't attach until the question has been persisted. */
async function attachAssetToScope(
  q: EditableQuestion,
  scope: QuestionAssetScope,
  scopeRef: number | null,
): Promise<void> {
  if (q.question_id == null) {
    error.value = 'Save the paper first — images attach to stored questions.'
    return
  }
  if (assetBusy.value.has(q.question_id)) return

  const picker = document.createElement('input')
  picker.type = 'file'
  picker.accept = 'image/png, image/jpeg, image/webp, image/gif, image/svg+xml'

  const file: File | null = await new Promise(resolve => {
    picker.addEventListener('change', () => resolve(picker.files?.[0] ?? null))
    picker.click()
  })
  if (!file) return

  assetBusy.value = new Set(assetBusy.value).add(q.question_id)
  error.value = ''
  try {
    const bytes = new Uint8Array(await file.arrayBuffer())
    const meta = await adminAttachQuestionAsset(
      q.question_id,
      scope,
      scopeRef,
      file.type || 'image/png',
      bytes,
      file.name || null,
    )
    q._assets = [...q._assets, meta]
    // Warm the object-URL cache straight away so the thumbnail renders
    // without a perceptible flash.
    void ensureAssetUrl(meta.asset_id)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Asset upload failed'
  } finally {
    const next = new Set(assetBusy.value)
    next.delete(q.question_id!)
    assetBusy.value = next
  }
}

async function removeAsset(q: EditableQuestion, assetId: number): Promise<void> {
  error.value = ''
  try {
    await adminDeleteQuestionAsset(assetId)
    q._assets = q._assets.filter(a => a.asset_id !== assetId)
    const url = assetUrlCache.value[assetId]
    if (url) {
      URL.revokeObjectURL(url)
      const next = { ...assetUrlCache.value }
      delete next[assetId]
      assetUrlCache.value = next
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Asset delete failed'
  }
}

onBeforeUnmount(() => {
  for (const url of Object.values(assetUrlCache.value)) URL.revokeObjectURL(url)
})

// ── Subjects / topics ───────────────────────────────────────────────
const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])
const loadingTopics = ref(false)

async function loadTopicsForSubject(sid: number): Promise<void> {
  loadingTopics.value = true
  try {
    topics.value = await listTopics(sid)
  } catch {
    topics.value = []
  } finally {
    loadingTopics.value = false
  }
}

watch(subjectId, async (sid) => {
  if (sid == null) return
  await loadTopicsForSubject(sid)
  // For any question without a valid topic_id, default to the first topic.
  const firstTopicId = topics.value[0]?.id ?? 0
  questions.value = questions.value.map(q => ({
    ...q,
    topic_id: q.topic_id && topics.value.some(t => t.id === q.topic_id) ? q.topic_id : firstTopicId,
  }))
})

// ── Initial load ────────────────────────────────────────────────────
const loading = ref(false)
const error = ref('')
const success = ref('')

onMounted(async () => {
  try {
    subjects.value = await listSubjects()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load subjects'
  }

  if (paperIdParam.value != null) {
    loading.value = true
    try {
      const paper = await adminGetPastPaper(paperIdParam.value)
      subjectId.value = paper.subject_id
      examYear.value = paper.exam_year
      paperCode.value = paper.paper_code ?? ''
      title.value = paper.title
      await loadTopicsForSubject(paper.subject_id)
      questions.value = paper.questions.map(q => ({
        _local_id: nextLocalId(),
        _ui_type: dbFormatToUiType(q.question_format, q.primary_pedagogic_function, q.options),
        _is_ambiguous: false,
        _assets: q.assets ?? [],
        question_id: q.question_id,
        section_label: q.section_label || 'A',
        question_number: q.question_number,
        topic_id: q.topic_id,
        subtopic_id: q.subtopic_id,
        stem: q.stem,
        question_format: q.question_format,
        primary_pedagogic_function: q.primary_pedagogic_function,
        explanation_text: q.explanation_text,
        difficulty_level: q.difficulty_level,
        marks: q.marks,
        options: q.options.map(o => ({
          option_id: o.option_id,
          option_label: o.option_label,
          option_text: o.option_text,
          is_correct: o.is_correct,
        })),
      }))
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load paper'
    } finally {
      loading.value = false
    }
  }
})

// ── IMPORT tab: PDF / Word / image → draft ──────────────────────────
const sourceTab = ref<'manual' | 'import' | 'csv'>('manual')
const importing = ref(false)
const importWarnings = ref<string[]>([])
const importInfo = ref<string>('')
const fileInput = ref<HTMLInputElement | null>(null)
const csvFileInput = ref<HTMLInputElement | null>(null)

async function handleFilePicked(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  importing.value = true
  importWarnings.value = []
  importInfo.value = ''
  try {
    const bytes = new Uint8Array(await file.arrayBuffer())
    const recovered: RecoveredTextDto = await adminExtractPastPaperText(
      file.name,
      file.type || null,
      bytes,
    )
    importInfo.value =
      `Extracted from ${file.name} · ${recovered.pages.length} page(s) · source: ${recovered.source}`
    if (recovered.source.includes('requires_ocr') || recovered.text.trim().length === 0) {
      importWarnings.value.push(
        'The file needed OCR but produced little or no text. Make sure `pdftotext` / `tesseract` are installed, or paste text in the Manual tab.',
      )
      return
    }

    const draft: DraftPaper = segmentPastPaperText(recovered.text)
    importWarnings.value = [...draft.warnings]

    // Build questions from the draft, ONE row per draft question, in
    // section order. The admin confirms & corrects in place.
    const nextQuestions: EditableQuestion[] = []
    for (const section of draft.sections) {
      for (const q of section.questions) {
        const uiType: UiType = q.question_format === 'essay'
          ? 'essay'
          : q.question_format === 'mcq'
            ? (q.options.filter(o => o.is_correct).length > 1 ? 'mcq_multi' : 'mcq')
            : q.question_format
        const mapping = uiTypeToDbFormat(uiType)
        nextQuestions.push({
          _local_id: nextLocalId(),
          _ui_type: uiType,
          _is_ambiguous: q.is_ambiguous,
          _assets: [],
          question_id: null,
          section_label: section.label || 'A',
          question_number: q.number,
          topic_id: topics.value[0]?.id ?? 0,
          subtopic_id: null,
          stem: q.stem,
          question_format: mapping.question_format,
          primary_pedagogic_function: mapping.pedagogic,
          explanation_text: null,
          difficulty_level: 5000,
          marks: uiType === 'mcq' || uiType === 'mcq_multi' ? 1 : 4,
          options: q.options.map(o => ({
            option_id: null,
            option_label: o.label,
            option_text: o.text,
            is_correct: o.is_correct,
          })),
        })
      }
    }
    questions.value = nextQuestions

    importInfo.value +=
      ` · Parsed ${nextQuestions.length} draft question${nextQuestions.length === 1 ? '' : 's'} — please review every row.`
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'File extraction failed'
  } finally {
    importing.value = false
    if (input) input.value = ''
  }
}

// ── CSV tab: structured bulk upload ─────────────────────────────────
async function handleCsvPicked(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  if (subjectId.value == null) {
    error.value = 'Pick a subject before importing CSV so topic names can resolve.'
    if (input) input.value = ''
    return
  }

  importing.value = true
  importWarnings.value = []
  importInfo.value = ''
  try {
    const text = await file.text()
    const result: CsvImportResult = parsePastPaperCsv(text)
    importWarnings.value = [...result.warnings]

    if (result.sections.every(s => s.questions.length === 0)) {
      error.value = 'CSV parsed but no valid question rows were found. Check the header row and type values.'
      return
    }

    // Resolve topic names to ids using the subject's topic list.
    const topicByLoweredName = new Map<string, number>()
    for (const t of topics.value) topicByLoweredName.set(t.name.toLowerCase(), t.id)
    const fallbackTopicId = topics.value[0]?.id ?? 0

    let hintIndex = 0
    const nextQuestions: EditableQuestion[] = []
    for (const section of result.sections) {
      for (const q of section.questions) {
        const topicHint = result.topicHints[hintIndex] ?? null
        const answerHint = result.answerHints[hintIndex] ?? null
        hintIndex += 1

        let topicId = fallbackTopicId
        if (topicHint) {
          const resolved = topicByLoweredName.get(topicHint.toLowerCase())
          if (resolved != null) {
            topicId = resolved
          } else {
            importWarnings.value.push(
              `Topic "${topicHint}" was not found in this subject — assigned "${topics.value[0]?.name ?? 'fallback'}" instead.`,
            )
          }
        }

        // Promote CSV's flattened type back to the richer UI set using
        // option-shape heuristics (segmenter can't distinguish mcq_multi
        // from mcq alone).
        const correctCount = q.options.filter(o => o.is_correct).length
        let uiType: UiType
        if (q.question_format === 'essay') uiType = 'essay'
        else if (q.options.length === 2
                 && q.options.some(o => /^true$/i.test(o.text))
                 && q.options.some(o => /^false$/i.test(o.text))) uiType = 'true_false'
        else if (q.question_format === 'mcq') uiType = correctCount > 1 ? 'mcq_multi' : 'mcq'
        else uiType = 'short_answer'
        const mapping = uiTypeToDbFormat(uiType)

        nextQuestions.push({
          _local_id: nextLocalId(),
          _ui_type: uiType,
          _is_ambiguous: false,
          _assets: [],
          question_id: null,
          section_label: section.label || 'A',
          question_number: q.number,
          topic_id: topicId,
          subtopic_id: null,
          stem: q.stem,
          question_format: mapping.question_format,
          primary_pedagogic_function: mapping.pedagogic,
          // For non-MCQ types, CSV's "answer" column lands in the
          // explanation textarea (which doubles as model-answer /
          // marking guide in the current schema).
          explanation_text: answerHint ?? null,
          difficulty_level: 5000,
          marks: uiType === 'essay' ? 6
               : uiType === 'mcq' || uiType === 'mcq_multi' || uiType === 'true_false' ? 1
               : 2,
          options: q.options.map(o => ({
            option_id: null,
            option_label: o.label,
            option_text: o.text,
            is_correct: o.is_correct,
          })),
        })
      }
    }
    questions.value = nextQuestions
    importInfo.value =
      `Parsed ${nextQuestions.length} question${nextQuestions.length === 1 ? '' : 's'} from ${file.name}.`
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'CSV parse failed'
  } finally {
    importing.value = false
    if (input) input.value = ''
  }
}

function downloadCsvTemplate(): void {
  const csv = generatePastPaperCsvTemplate()
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'past-paper-template.csv'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  // Revoke after a tick so the download actually starts.
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}

// ── Save ────────────────────────────────────────────────────────────
const saving = ref(false)

async function onSave(): Promise<void> {
  error.value = ''
  success.value = ''

  if (subjectId.value == null) {
    error.value = 'Pick a subject.'
    return
  }
  if (title.value.trim().length === 0) {
    error.value = 'Paper title is required.'
    return
  }
  if (questions.value.length === 0) {
    error.value = 'Add at least one question before saving.'
    return
  }

  const input: AdminPastPaperSaveInput = {
    paper_id: paperIdParam.value,
    subject_id: subjectId.value,
    exam_year: examYear.value,
    paper_code: paperCode.value.trim() ? paperCode.value.trim() : null,
    title: title.value.trim(),
    questions: questions.value.map(q => {
      // Always re-derive the DB mapping from the UI type at save time.
      // This catches rows whose type was changed after creation but
      // whose question_format / pedagogic flag didn't stay in sync.
      const mapping = uiTypeToDbFormat(q._ui_type)
      return {
        question_id: q.question_id,
        section_label: q.section_label || 'A',
        question_number: q.question_number,
        topic_id: q.topic_id,
        subtopic_id: q.subtopic_id,
        stem: q.stem,
        question_format: mapping.question_format,
        primary_pedagogic_function: mapping.pedagogic,
        explanation_text: q.explanation_text,
        difficulty_level: q.difficulty_level,
        marks: q.marks,
        options: q.options.map(o => ({
          option_id: o.option_id,
          option_label: o.option_label,
          option_text: o.option_text,
          is_correct: o.is_correct,
        } satisfies AdminPastPaperOptionInput)),
      }
    }),
  }

  saving.value = true
  try {
    const result = await adminSavePastPaper(input)
    success.value = `Saved paper · ${result.question_count} question(s) persisted.`

    // Re-hydrate the editor from the persisted state so every row has
    // its fresh DB id — both question_id and option_id. This is what
    // makes option-scope image attachments point at stable rows across
    // re-saves; without it a subsequent save would orphan assets.
    try {
      const refreshed = await adminGetPastPaper(result.paper_id)
      questions.value = refreshed.questions.map(q => ({
        _local_id: nextLocalId(),
        _ui_type: dbFormatToUiType(q.question_format, q.primary_pedagogic_function, q.options),
        _is_ambiguous: false,
        _assets: q.assets ?? [],
        question_id: q.question_id,
        section_label: q.section_label || 'A',
        question_number: q.question_number,
        topic_id: q.topic_id,
        subtopic_id: q.subtopic_id,
        stem: q.stem,
        question_format: q.question_format,
        primary_pedagogic_function: q.primary_pedagogic_function,
        explanation_text: q.explanation_text,
        difficulty_level: q.difficulty_level,
        marks: q.marks,
        options: q.options.map(o => ({
          option_id: o.option_id,
          option_label: o.option_label,
          option_text: o.option_text,
          is_correct: o.is_correct,
        })),
      }))
    } catch { /* best-effort — local state is still valid */ }

    // After first save, route to the paper-edit URL so subsequent saves
    // update in place rather than creating duplicates.
    if (paperIdParam.value == null) {
      router.replace(`/admin/past-papers/${result.paper_id}`)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Save failed'
  } finally {
    saving.value = false
  }
}

// ── Section labels (freeform but suggest A/B) ──────────────────────
const sectionOptions = ['A', 'B', 'C', 'D']
</script>

<template>
  <div class="pa-shell">
    <!-- HEADER -->
    <header class="pa-head">
      <div class="pa-crumbs">
        <button class="pa-link" @click="router.push('/admin/past-papers')">← Past Papers</button>
      </div>
      <h1 class="pa-title">
        {{ paperIdParam == null ? 'New past paper' : `Edit paper #${paperIdParam}` }}
      </h1>
      <p class="pa-sub">
        Author questions once — they appear under the student "Past Questions" page, grouped by year and section.
      </p>
    </header>

    <p v-if="error" class="pa-banner pa-banner--err">[ ERROR ] {{ error }}</p>
    <p v-if="success" class="pa-banner pa-banner--ok">[ SAVED ] {{ success }}</p>

    <div v-if="loading" class="pa-loading">[ LOADING... ]</div>

    <div v-else class="pa-grid">
      <!-- ── LEFT: metadata + source ── -->
      <aside class="pa-left">
        <section class="pa-card">
          <p class="pa-eyebrow">Paper</p>

          <label class="pa-field">
            <span class="pa-field-label">Subject</span>
            <select v-model.number="subjectId" class="pa-input">
              <option :value="null" disabled>Pick a subject</option>
              <option v-for="s in subjects" :key="s.id" :value="s.id">{{ s.name }}</option>
            </select>
          </label>

          <div class="pa-row">
            <label class="pa-field">
              <span class="pa-field-label">Exam year</span>
              <input v-model.number="examYear" type="number" class="pa-input" min="1980" max="2100" />
            </label>
            <label class="pa-field">
              <span class="pa-field-label">Paper code</span>
              <input
                v-model="paperCode"
                type="text"
                class="pa-input"
                placeholder="e.g. 1/2"
                maxlength="40"
              />
            </label>
          </div>

          <label class="pa-field">
            <span class="pa-field-label">Title</span>
            <input
              v-model="title"
              type="text"
              class="pa-input"
              placeholder="e.g. BECE Mathematics Paper 1"
              maxlength="200"
            />
          </label>
        </section>

        <section class="pa-card">
          <p class="pa-eyebrow">Source</p>
          <div class="pa-tabs" role="tablist">
            <button
              class="pa-tab"
              :class="{ 'pa-tab--on': sourceTab === 'manual' }"
              role="tab"
              :aria-selected="sourceTab === 'manual'"
              @click="sourceTab = 'manual'"
            >MANUAL</button>
            <button
              class="pa-tab"
              :class="{ 'pa-tab--on': sourceTab === 'import' }"
              role="tab"
              :aria-selected="sourceTab === 'import'"
              @click="sourceTab = 'import'"
            >PDF / WORD / IMAGE</button>
            <button
              class="pa-tab"
              :class="{ 'pa-tab--on': sourceTab === 'csv' }"
              role="tab"
              :aria-selected="sourceTab === 'csv'"
              @click="sourceTab = 'csv'"
            >CSV</button>
          </div>

          <div v-if="sourceTab === 'manual'" class="pa-source-body">
            <p class="pa-hint">
              Add questions on the right. Switch a row between Multiple Choice and Essay per question.
            </p>
          </div>

          <div v-else-if="sourceTab === 'import'" class="pa-source-body">
            <p class="pa-hint">
              Upload a PDF, Word (.docx) or scanned image. Text is extracted with the local OCR pipeline
              ({{ ' ' }}<code>pdftotext</code> / <code>tesseract</code>) and segmented into draft questions
              — each row must still be reviewed.
            </p>
            <input
              ref="fileInput"
              type="file"
              accept=".pdf,.docx,.doc,.png,.jpg,.jpeg,.webp,.txt"
              class="pa-file"
              :disabled="importing"
              @change="handleFilePicked"
            />
            <p v-if="importing" class="pa-mono-tag">[ EXTRACTING... ]</p>
            <p v-if="importInfo" class="pa-mono-tag">[ OK ] {{ importInfo }}</p>
            <p
              v-for="(warn, i) in importWarnings"
              :key="i"
              class="pa-mono-tag pa-mono-tag--warn"
            >
              [ WARN ] {{ warn }}
            </p>
          </div>

          <div v-else class="pa-source-body">
            <p class="pa-hint">
              Bulk-upload a structured <code>.csv</code> — one row per question.
              Supported types: <code>mcq</code>, <code>mcq_multi</code>, <code>true_false</code>,
              <code>short_answer</code>, <code>numeric</code>, <code>essay</code>.
              Fill-in-the-blank needs the Manual tab.
            </p>
            <div class="pa-csv-actions">
              <button
                class="pa-btn pa-btn--ghost"
                type="button"
                @click="downloadCsvTemplate"
              >↓ Download template</button>
              <input
                ref="csvFileInput"
                type="file"
                accept=".csv,text/csv"
                class="pa-file"
                :disabled="importing || subjectId == null"
                @change="handleCsvPicked"
              />
            </div>
            <p v-if="subjectId == null" class="pa-mono-tag pa-mono-tag--warn">
              [ NEED SUBJECT ] Pick a subject above — topic names resolve against it.
            </p>
            <p v-if="importing" class="pa-mono-tag">[ PARSING CSV... ]</p>
            <p v-if="importInfo" class="pa-mono-tag">[ OK ] {{ importInfo }}</p>
            <p
              v-for="(warn, i) in importWarnings"
              :key="i"
              class="pa-mono-tag pa-mono-tag--warn"
            >
              [ WARN ] {{ warn }}
            </p>
          </div>
        </section>

        <button
          class="pa-save"
          :disabled="saving || loading"
          @click="onSave"
        >
          <span>{{ saving ? 'SAVING…' : 'SAVE PAPER' }}</span>
          <span aria-hidden="true">→</span>
        </button>
      </aside>

      <!-- ── RIGHT: question list ── -->
      <main class="pa-right">
        <header class="pa-right-head">
          <p class="pa-eyebrow">Questions · {{ questions.length }}</p>
          <button class="pa-btn" :disabled="subjectId == null" @click="addQuestion">+ Add question</button>
        </header>

        <p v-if="questions.length === 0" class="pa-empty">
          No questions yet. Add one manually, or import a file on the left.
        </p>

        <ol class="pa-qlist" role="list">
          <li
            v-for="(q, index) in questions"
            :key="q._local_id"
            class="pa-qrow"
            :class="{ 'pa-qrow--ambiguous': q._is_ambiguous }"
          >
            <header class="pa-qhead">
              <div class="pa-qhead-left">
                <span class="pa-qn">Q{{ index + 1 }}</span>
                <span
                  v-if="q._is_ambiguous"
                  class="pa-ambiguous-dot"
                  title="The importer wasn't confident about this row — review the stem, options, and correct answer."
                  aria-label="Needs review"
                >●</span>
                <label class="pa-inline">
                  <span class="pa-inline-label">SECTION</span>
                  <select v-model="q.section_label" class="pa-inline-input">
                    <option v-for="opt in sectionOptions" :key="opt" :value="opt">{{ opt }}</option>
                  </select>
                </label>
                <label class="pa-inline">
                  <span class="pa-inline-label">#</span>
                  <input
                    v-model="q.question_number"
                    type="text"
                    class="pa-inline-input pa-inline-input--narrow"
                    placeholder="1"
                    maxlength="8"
                  />
                </label>
                <label class="pa-inline">
                  <span class="pa-inline-label">TYPE</span>
                  <select
                    :value="q._ui_type"
                    class="pa-inline-input pa-inline-input--wide"
                    @change="setUiType(q, ($event.target as HTMLSelectElement).value as UiType)"
                  >
                    <option value="mcq">Multiple Choice · single correct</option>
                    <option value="mcq_multi">Multiple Choice · multi correct</option>
                    <option value="true_false">True / False</option>
                    <option value="short_answer">Short Answer</option>
                    <option value="numeric">Numeric</option>
                    <option value="fill_blank">Fill in the Blank</option>
                    <option value="essay">Essay</option>
                  </select>
                </label>
              </div>
              <div class="pa-qhead-right">
                <button class="pa-icon-btn" title="Move up" @click="moveQuestion(q._local_id, -1)">↑</button>
                <button class="pa-icon-btn" title="Move down" @click="moveQuestion(q._local_id, 1)">↓</button>
                <button class="pa-icon-btn" title="Duplicate" @click="duplicateQuestion(q._local_id)">⎘</button>
                <button class="pa-icon-btn pa-icon-btn--danger" title="Remove" @click="removeQuestion(q._local_id)">✕</button>
              </div>
            </header>

            <label class="pa-field pa-field--dense">
              <span class="pa-field-label">Topic</span>
              <select v-model.number="q.topic_id" class="pa-input">
                <option :value="0" disabled>Pick a topic</option>
                <option v-for="t in topics" :key="t.id" :value="t.id">{{ t.name }}</option>
              </select>
            </label>

            <label class="pa-field pa-field--dense">
              <span class="pa-field-label">
                Stem
                <span v-if="q._ui_type === 'fill_blank'" class="pa-field-hint">
                  · use [[1]], [[2]] markers for blanks
                </span>
              </span>
              <textarea
                v-model="q.stem"
                class="pa-input pa-textarea"
                rows="3"
                placeholder="Question text…"
              ></textarea>
              <button
                v-if="q._ui_type === 'fill_blank'"
                class="pa-btn pa-btn--ghost"
                type="button"
                @click="insertBlank(q)"
              >+ Insert blank</button>
            </label>

            <!-- Stem-scope image attachments -->
            <div class="pa-assets">
              <div class="pa-asset-strip">
                <figure
                  v-for="asset in scopeAssets(q, 'stem')"
                  :key="asset.asset_id"
                  class="pa-asset-thumb"
                >
                  <img
                    v-if="assetUrlCache[asset.asset_id]"
                    :src="assetUrlCache[asset.asset_id]"
                    :alt="asset.alt_text ?? ''"
                    class="pa-asset-img"
                  />
                  <span
                    v-else
                    class="pa-asset-placeholder"
                    :data-loading="ensureAssetUrl(asset.asset_id)"
                  >···</span>
                  <figcaption class="pa-asset-meta">
                    <span class="pa-asset-mime">{{ mimeLabel(asset.mime_type) }}</span>
                    <span class="pa-asset-size">{{ Math.round(asset.byte_size / 1024) }}KB</span>
                    <button
                      class="pa-icon-btn pa-icon-btn--danger pa-asset-remove"
                      type="button"
                      title="Remove image"
                      @click="removeAsset(q, asset.asset_id)"
                    >✕</button>
                  </figcaption>
                </figure>
              </div>
              <button
                class="pa-btn pa-btn--ghost"
                type="button"
                :disabled="q.question_id == null || assetBusy.has(q.question_id ?? -1)"
                @click="attachAssetToScope(q, 'stem', null)"
              >
                {{ q.question_id == null
                    ? '🔒 Save paper first to attach images'
                    : assetBusy.has(q.question_id ?? -1) ? 'UPLOADING…' : '+ Attach image to stem' }}
              </button>
            </div>

            <!-- MCQ single-correct -->
            <div v-if="q._ui_type === 'mcq' || q._ui_type === 'true_false'" class="pa-options">
              <p class="pa-eyebrow pa-eyebrow--inset">
                {{ q._ui_type === 'true_false' ? 'Answer' : 'Options · click the letter to mark correct' }}
              </p>
              <ul role="list" class="pa-option-list">
                <li
                  v-for="(opt, i) in q.options"
                  :key="i"
                  class="pa-option-wrap"
                >
                  <div
                    class="pa-option"
                    :class="{ 'pa-option--correct': opt.is_correct }"
                  >
                    <button
                      type="button"
                      class="pa-option-radio"
                      :class="{ 'pa-option-radio--on': opt.is_correct }"
                      :aria-pressed="opt.is_correct"
                      title="Mark correct"
                      @click="setCorrectOption(q, i)"
                    >
                      {{ opt.option_label }}
                    </button>
                    <input
                      v-model="opt.option_text"
                      type="text"
                      class="pa-input pa-input--bare"
                      placeholder="Option text…"
                    />
                    <button
                      type="button"
                      class="pa-icon-btn"
                      :class="{ 'pa-icon-btn--disabled': opt.option_id == null }"
                      :title="opt.option_id == null ? 'Save paper first to attach option image' : 'Attach image to this option'"
                      :disabled="opt.option_id == null || assetBusy.has(q.question_id ?? -1)"
                      @click="attachAssetToScope(q, 'option', opt.option_id)"
                    >+📎</button>
                    <button
                      type="button"
                      class="pa-icon-btn pa-icon-btn--danger"
                      title="Remove option"
                      :disabled="q.options.length <= 2 || q._ui_type === 'true_false'"
                      @click="removeOption(q, i)"
                    >✕</button>
                  </div>
                  <div v-if="opt.option_id != null && scopeAssets(q, 'option', opt.option_id).length > 0" class="pa-option-assets">
                    <figure
                      v-for="asset in scopeAssets(q, 'option', opt.option_id)"
                      :key="asset.asset_id"
                      class="pa-asset-thumb pa-asset-thumb--compact"
                    >
                      <img
                        v-if="assetUrlCache[asset.asset_id]"
                        :src="assetUrlCache[asset.asset_id]"
                        :alt="asset.alt_text ?? ''"
                        class="pa-asset-img"
                      />
                      <span
                        v-else
                        class="pa-asset-placeholder"
                        :data-loading="ensureAssetUrl(asset.asset_id)"
                      >···</span>
                      <figcaption class="pa-asset-meta">
                        <span class="pa-asset-size">{{ Math.round(asset.byte_size / 1024) }}KB</span>
                        <button
                          class="pa-icon-btn pa-icon-btn--danger pa-asset-remove"
                          type="button"
                          title="Remove image"
                          @click="removeAsset(q, asset.asset_id)"
                        >✕</button>
                      </figcaption>
                    </figure>
                  </div>
                </li>
              </ul>
              <button
                v-if="q._ui_type !== 'true_false'"
                class="pa-btn pa-btn--ghost"
                type="button"
                @click="addOption(q)"
              >+ Add option</button>
            </div>

            <!-- MCQ multi-correct — same layout, but the letter button TOGGLES
                 rather than being radio-exclusive. Square corners signal the
                 distinction visually. -->
            <div v-else-if="q._ui_type === 'mcq_multi'" class="pa-options">
              <p class="pa-eyebrow pa-eyebrow--inset">
                Options · click each letter that's a correct answer
              </p>
              <ul role="list" class="pa-option-list">
                <li
                  v-for="(opt, i) in q.options"
                  :key="i"
                  class="pa-option-wrap"
                >
                  <div
                    class="pa-option"
                    :class="{ 'pa-option--correct': opt.is_correct }"
                  >
                    <button
                      type="button"
                      class="pa-option-radio pa-option-radio--check"
                      :class="{ 'pa-option-radio--on': opt.is_correct }"
                      :aria-pressed="opt.is_correct"
                      title="Toggle correct"
                      @click="toggleCorrectOption(q, i)"
                    >
                      {{ opt.option_label }}
                    </button>
                    <input
                      v-model="opt.option_text"
                      type="text"
                      class="pa-input pa-input--bare"
                      placeholder="Option text…"
                    />
                    <button
                      type="button"
                      class="pa-icon-btn"
                      :class="{ 'pa-icon-btn--disabled': opt.option_id == null }"
                      :title="opt.option_id == null ? 'Save paper first to attach option image' : 'Attach image to this option'"
                      :disabled="opt.option_id == null || assetBusy.has(q.question_id ?? -1)"
                      @click="attachAssetToScope(q, 'option', opt.option_id)"
                    >+📎</button>
                    <button
                      type="button"
                      class="pa-icon-btn pa-icon-btn--danger"
                      title="Remove option"
                      :disabled="q.options.length <= 2"
                      @click="removeOption(q, i)"
                    >✕</button>
                  </div>
                  <div v-if="opt.option_id != null && scopeAssets(q, 'option', opt.option_id).length > 0" class="pa-option-assets">
                    <figure
                      v-for="asset in scopeAssets(q, 'option', opt.option_id)"
                      :key="asset.asset_id"
                      class="pa-asset-thumb pa-asset-thumb--compact"
                    >
                      <img
                        v-if="assetUrlCache[asset.asset_id]"
                        :src="assetUrlCache[asset.asset_id]"
                        :alt="asset.alt_text ?? ''"
                        class="pa-asset-img"
                      />
                      <span
                        v-else
                        class="pa-asset-placeholder"
                        :data-loading="ensureAssetUrl(asset.asset_id)"
                      >···</span>
                      <figcaption class="pa-asset-meta">
                        <span class="pa-asset-size">{{ Math.round(asset.byte_size / 1024) }}KB</span>
                        <button
                          class="pa-icon-btn pa-icon-btn--danger pa-asset-remove"
                          type="button"
                          title="Remove image"
                          @click="removeAsset(q, asset.asset_id)"
                        >✕</button>
                      </figcaption>
                    </figure>
                  </div>
                </li>
              </ul>
              <button class="pa-btn pa-btn--ghost" type="button" @click="addOption(q)">+ Add option</button>
            </div>

            <!-- Fill-in-the-blank: per-blank acceptable-answer lists. -->
            <div v-else-if="q._ui_type === 'fill_blank'" class="pa-options">
              <p class="pa-eyebrow pa-eyebrow--inset">
                Acceptable answers · one blank per group, add alternatives within a group
              </p>
              <div
                v-for="group in blankGroups(q)"
                :key="group.index"
                class="pa-blank-group"
              >
                <div class="pa-blank-head">
                  <span class="pa-blank-label">[[{{ group.index }}]]</span>
                  <button
                    class="pa-icon-btn pa-icon-btn--danger"
                    type="button"
                    title="Remove this blank from stem and answers"
                    @click="removeBlank(q, group.index)"
                  >✕</button>
                </div>
                <ul role="list" class="pa-option-list pa-option-list--nested">
                  <li
                    v-for="optIdx in group.alternatives"
                    :key="optIdx"
                    class="pa-option"
                  >
                    <span class="pa-option-radio pa-option-radio--static" title="Acceptable answer">≡</span>
                    <input
                      v-model="q.options[optIdx].option_text"
                      type="text"
                      class="pa-input pa-input--bare"
                      placeholder="Accepted answer (e.g. Accra)"
                    />
                    <button
                      class="pa-icon-btn pa-icon-btn--danger"
                      type="button"
                      title="Remove alternative"
                      :disabled="group.alternatives.length <= 1"
                      @click="removeOption(q, optIdx)"
                    >✕</button>
                  </li>
                </ul>
                <button
                  class="pa-btn pa-btn--ghost"
                  type="button"
                  @click="addAlternativeToBlank(q, group.index)"
                >+ Add alternative for blank {{ group.index }}</button>
              </div>
              <p v-if="blankGroups(q).length === 0" class="pa-hint">
                Use the "Insert blank" button above to place a blank in the stem.
              </p>
            </div>

            <!-- Essay / short_answer / numeric: marking guide textarea. -->
            <label v-else class="pa-field pa-field--dense">
              <span class="pa-field-label">
                {{ q._ui_type === 'essay'
                    ? 'Model answer / marking guide'
                    : q._ui_type === 'numeric'
                      ? 'Correct numeric answer (or acceptable range)'
                      : 'Accepted answer(s) / marking hint' }}
              </span>
              <textarea
                v-model="q.explanation_text"
                class="pa-input pa-textarea"
                :rows="q._ui_type === 'essay' ? 6 : 3"
                :placeholder="q._ui_type === 'essay'
                  ? 'The expected answer, marking points, and common pitfalls…'
                  : 'What answer(s) should be marked correct?'"
              ></textarea>
            </label>

            <!-- Explanation-scope image attachments (figures for essays, working diagrams, etc.) -->
            <div v-if="q._ui_type !== 'mcq' && q._ui_type !== 'mcq_multi' && q._ui_type !== 'true_false' && q._ui_type !== 'fill_blank'" class="pa-assets">
              <div class="pa-asset-strip">
                <figure
                  v-for="asset in scopeAssets(q, 'explanation')"
                  :key="asset.asset_id"
                  class="pa-asset-thumb"
                >
                  <img
                    v-if="assetUrlCache[asset.asset_id]"
                    :src="assetUrlCache[asset.asset_id]"
                    :alt="asset.alt_text ?? ''"
                    class="pa-asset-img"
                  />
                  <span
                    v-else
                    class="pa-asset-placeholder"
                    :data-loading="ensureAssetUrl(asset.asset_id)"
                  >···</span>
                  <figcaption class="pa-asset-meta">
                    <span class="pa-asset-mime">{{ mimeLabel(asset.mime_type) }}</span>
                    <span class="pa-asset-size">{{ Math.round(asset.byte_size / 1024) }}KB</span>
                    <button
                      class="pa-icon-btn pa-icon-btn--danger pa-asset-remove"
                      type="button"
                      title="Remove image"
                      @click="removeAsset(q, asset.asset_id)"
                    >✕</button>
                  </figcaption>
                </figure>
              </div>
              <button
                class="pa-btn pa-btn--ghost"
                type="button"
                :disabled="q.question_id == null || assetBusy.has(q.question_id ?? -1)"
                @click="attachAssetToScope(q, 'explanation', null)"
              >
                {{ q.question_id == null
                    ? '🔒 Save paper first to attach images'
                    : '+ Attach image to model-answer' }}
              </button>
            </div>

            <div class="pa-qfoot">
              <label class="pa-inline">
                <span class="pa-inline-label">MARKS</span>
                <input
                  v-model.number="q.marks"
                  type="number"
                  class="pa-inline-input pa-inline-input--narrow"
                  min="0"
                  max="100"
                />
              </label>
              <label class="pa-inline">
                <span class="pa-inline-label">DIFFICULTY</span>
                <input
                  v-model.number="q.difficulty_level"
                  type="number"
                  class="pa-inline-input pa-inline-input--narrow"
                  min="0"
                  max="10000"
                  step="500"
                />
              </label>
              <span v-if="q.question_id" class="pa-id">DB id: {{ q.question_id }}</span>
            </div>
          </li>
        </ol>

        <div v-if="questions.length > 0" class="pa-add-more">
          <button class="pa-btn" :disabled="subjectId == null" @click="addQuestion">
            + Add another question
          </button>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   Admin-facing Nothing-design: less heroic than student views, same
   vocabulary. Mono eyebrows, Space Grotesk body, no color noise.
   ═══════════════════════════════════════════════════════════════════ */
.pa-shell {
  --paper: #faf8f5;
  --paper-dim: #f2efe9;
  --ink: #1a1612;
  --ink-primary: rgba(26, 22, 18, 0.92);
  --ink-secondary: rgba(26, 22, 18, 0.60);
  --ink-muted: rgba(26, 22, 18, 0.40);
  --rule: rgba(26, 22, 18, 0.12);
  --rule-strong: rgba(26, 22, 18, 0.28);
  --success: #15803d;
  --danger: #b91c1c;
  --warm: #c2410c;

  min-height: 100%;
  padding: 36px clamp(20px, 3.5vw, 48px) 64px;
  background: var(--paper);
  color: var(--ink);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  letter-spacing: -0.005em;
}

.pa-head { margin-bottom: 24px; }
.pa-crumbs { margin-bottom: 10px; }
.pa-link {
  background: transparent;
  border: none;
  padding: 4px 0;
  color: var(--ink-secondary);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: color 140ms ease;
}
.pa-link:hover { color: var(--ink); }

.pa-title {
  margin: 0 0 6px;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 400;
  font-size: clamp(28px, 3.2vw, 40px);
  line-height: 1.05;
  letter-spacing: -0.02em;
  color: var(--ink);
}
.pa-sub {
  margin: 0;
  max-width: 66ch;
  font-size: 14px;
  color: var(--ink-secondary);
}

.pa-banner {
  margin: 12px 0 0;
  padding: 10px 14px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  border: 1px solid var(--rule);
  border-radius: 8px;
}
.pa-banner--err { color: var(--danger); border-color: rgba(185, 28, 28, 0.3); }
.pa-banner--ok { color: var(--success); border-color: rgba(21, 128, 61, 0.3); }

.pa-loading {
  padding: 64px 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}

.pa-grid {
  display: grid;
  grid-template-columns: 360px minmax(0, 1fr);
  gap: 32px;
  margin-top: 28px;
}

/* ── Left column ── */
.pa-left { display: grid; gap: 20px; align-content: start; }
.pa-card {
  padding: 20px;
  border: 1px solid var(--rule);
  border-radius: 12px;
  background: transparent;
  display: grid;
  gap: 14px;
}
.pa-eyebrow {
  margin: 0 0 4px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.pa-eyebrow--inset { margin-top: 4px; }

.pa-field { display: grid; gap: 6px; }
.pa-field--dense { gap: 4px; }
.pa-field-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-secondary);
}
.pa-input {
  width: 100%;
  padding: 10px 12px;
  background: var(--paper);
  border: 1px solid var(--rule-strong);
  border-radius: 8px;
  font-family: 'Space Grotesk', sans-serif;
  font-size: 14px;
  color: var(--ink);
  outline: none;
  transition: border-color 140ms ease;
}
.pa-input:focus { border-color: var(--ink); }
.pa-input--bare {
  border: none;
  background: transparent;
  padding: 10px 0;
}
.pa-input--bare:focus { border-bottom: 1px solid var(--ink); }
.pa-textarea { resize: vertical; font-family: inherit; min-height: 84px; }

.pa-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }

/* Tabs */
.pa-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 2px;
  border: 1px solid var(--rule-strong);
  border-radius: 999px;
}
.pa-tab {
  padding: 6px 12px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--ink-secondary);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  cursor: pointer;
}
.pa-tab:hover:not(.pa-tab--on) { color: var(--ink); }
.pa-tab--on { background: var(--ink); color: var(--paper); }

.pa-source-body { display: grid; gap: 10px; }
.pa-csv-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.pa-hint { margin: 0; font-size: 13px; color: var(--ink-secondary); line-height: 1.5; }
.pa-hint code {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  background: var(--paper-dim);
  padding: 2px 5px;
  border-radius: 4px;
}
.pa-file {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  color: var(--ink-primary);
}
.pa-mono-tag {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink-secondary);
}
.pa-mono-tag--warn { color: var(--warm); }

/* Save button */
.pa-save {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 20px;
  border: none;
  border-radius: 999px;
  background: var(--ink);
  color: var(--paper);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: transform 140ms ease, opacity 140ms ease;
}
.pa-save:hover:not(:disabled) { transform: translateY(-1px); }
.pa-save:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Right column ── */
.pa-right { min-width: 0; }
.pa-right-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 14px;
}

.pa-btn {
  padding: 10px 16px;
  border: 1px solid var(--ink);
  border-radius: 999px;
  background: transparent;
  color: var(--ink);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.18em;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}
.pa-btn:hover:not(:disabled) { background: var(--ink); color: var(--paper); }
.pa-btn:disabled { opacity: 0.45; cursor: not-allowed; }
.pa-btn--ghost {
  border-color: var(--rule-strong);
  color: var(--ink-secondary);
  padding: 8px 14px;
  font-size: 11px;
}
.pa-btn--ghost:hover:not(:disabled) { border-color: var(--ink); color: var(--ink); background: transparent; }

.pa-empty {
  padding: 48px 20px;
  margin: 0;
  border: 1px dashed var(--rule-strong);
  border-radius: 12px;
  text-align: center;
  color: var(--ink-muted);
  font-size: 14px;
}

/* Question list */
.pa-qlist {
  list-style: none;
  margin: 0 0 16px;
  padding: 0;
  display: grid;
  gap: 14px;
}
.pa-qrow {
  padding: 18px 20px;
  border: 1px solid var(--rule);
  border-radius: 12px;
  background: transparent;
  display: grid;
  gap: 12px;
}
.pa-qhead {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}
.pa-qhead-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}
.pa-qhead-right { display: flex; align-items: center; gap: 4px; }

.pa-qn {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink);
}
.pa-inline { display: inline-flex; align-items: center; gap: 8px; }
.pa-inline-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted);
}
.pa-inline-input {
  padding: 6px 10px;
  background: var(--paper);
  border: 1px solid var(--rule-strong);
  border-radius: 6px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 400;
  color: var(--ink);
  outline: none;
  transition: border-color 140ms ease;
}
.pa-inline-input:focus { border-color: var(--ink); }
.pa-inline-input--narrow { width: 60px; }
.pa-inline-input--wide { min-width: 220px; }

.pa-ambiguous-dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 10px;
  height: 10px;
  font-size: 12px;
  line-height: 1;
  color: var(--danger);
  cursor: help;
}
.pa-qrow--ambiguous {
  border-color: rgba(185, 28, 28, 0.35);
  background: color-mix(in srgb, var(--danger) 2%, transparent);
}

.pa-option-radio--check { border-radius: 6px; }
.pa-option-radio--check.pa-option-radio--on {
  background: var(--success);
  border-color: var(--success);
  color: var(--paper);
}
.pa-option-radio--static {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: 1px dashed var(--rule-strong);
  border-radius: 6px;
  color: var(--ink-muted);
  font-size: 14px;
  cursor: default;
}

.pa-blank-group {
  padding: 14px 14px 12px;
  border: 1px solid var(--rule);
  border-radius: 10px;
  display: grid;
  gap: 10px;
  margin-bottom: 10px;
}
.pa-blank-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.pa-blank-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink);
}
.pa-option-list--nested { gap: 4px; }

.pa-field-hint {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

/* ─── Image asset strip ─── */
.pa-assets {
  display: grid;
  gap: 8px;
  padding: 8px 0 2px;
}
.pa-asset-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.pa-asset-thumb {
  position: relative;
  width: 140px;
  margin: 0;
  padding: 0;
  border: 1px solid var(--rule);
  border-radius: 8px;
  overflow: hidden;
  display: grid;
  grid-template-rows: 100px auto;
  background: var(--paper-dim);
}
.pa-asset-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: var(--paper);
}
.pa-asset-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-muted);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 14px;
  letter-spacing: 0.3em;
}
.pa-asset-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
  border-top: 1px solid var(--rule);
}
.pa-asset-mime { color: var(--ink-secondary); }
.pa-asset-size { margin-left: auto; }
.pa-asset-remove {
  width: 20px;
  height: 20px;
  font-size: 10px;
}

/* Option row wrapper — allows an optional asset strip to sit below
   the option without disturbing the horizontal flex layout above. */
.pa-option-wrap { display: grid; gap: 6px; }
.pa-option-assets {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-left: 46px; /* aligns under option text, past the letter badge */
}
.pa-asset-thumb--compact {
  width: 90px;
  grid-template-rows: 64px auto;
}
.pa-asset-thumb--compact .pa-asset-img { max-height: 64px; }
.pa-icon-btn--disabled { opacity: 0.35; cursor: not-allowed; }

.pa-icon-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--rule-strong);
  border-radius: 6px;
  background: transparent;
  color: var(--ink-secondary);
  font-size: 14px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 140ms ease, color 140ms ease, border-color 140ms ease;
}
.pa-icon-btn:hover:not(:disabled) { border-color: var(--ink); color: var(--ink); }
.pa-icon-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.pa-icon-btn--danger:hover:not(:disabled) { border-color: var(--danger); color: var(--danger); }

/* Options editor */
.pa-options { display: grid; gap: 10px; }
.pa-option-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}
.pa-option {
  display: grid;
  grid-template-columns: 40px minmax(0, 1fr) 28px;
  align-items: center;
  gap: 10px;
}
.pa-option-radio {
  width: 36px;
  height: 36px;
  border: 1px solid var(--rule-strong);
  border-radius: 999px;
  background: transparent;
  color: var(--ink);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.1em;
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease, border-color 140ms ease;
}
.pa-option-radio:hover { border-color: var(--ink); }
.pa-option-radio--on {
  background: var(--success);
  border-color: var(--success);
  color: var(--paper);
}
.pa-option--correct .pa-input--bare {
  color: var(--ink);
  font-weight: 500;
}

.pa-qfoot {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
  padding-top: 8px;
  border-top: 1px dashed var(--rule);
}
.pa-id {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 10px;
  letter-spacing: 0.18em;
  color: var(--ink-muted);
}

.pa-add-more { display: flex; justify-content: center; padding: 8px 0; }

/* ── Narrow viewports ── */
@media (max-width: 960px) {
  .pa-grid { grid-template-columns: 1fr; }
}
@media (prefers-color-scheme: dark) {
  .pa-shell {
    --paper: #0a0906;
    --paper-dim: #15130f;
    --ink: #f3ede2;
    --ink-primary: rgba(243, 237, 226, 0.92);
    --ink-secondary: rgba(243, 237, 226, 0.60);
    --ink-muted: rgba(243, 237, 226, 0.40);
    --rule: rgba(243, 237, 226, 0.12);
    --rule-strong: rgba(243, 237, 226, 0.28);
  }
}
</style>
