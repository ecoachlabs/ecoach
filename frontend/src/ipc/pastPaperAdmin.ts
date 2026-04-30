import { ipc } from '.'

// ── DTOs ──────────────────────────────────────────────────────────

export interface AdminPastPaperOptionInput {
  /** DB id for existing options; null for freshly added rows. Carrying
   *  this through enables a stable-id upsert on save, so option-scoped
   *  image attachments survive re-saves. */
  option_id: number | null
  option_label: string
  option_text: string
  is_correct: boolean
}

export interface AdminPastPaperQuestionInput {
  question_id: number | null
  section_label: string
  question_number: string | null
  topic_id: number
  subtopic_id: number | null
  stem: string
  /** DB format. Essay + fill-blank both persist as `short_answer` — the
   *  distinction lives in `primary_pedagogic_function`. */
  question_format: string
  primary_pedagogic_function: string | null
  explanation_text: string | null
  difficulty_level: number | null
  marks: number | null
  options: AdminPastPaperOptionInput[]
}

export interface AdminPastPaperSaveInput {
  paper_id: number | null
  subject_id: number
  exam_year: number
  paper_code: string | null
  title: string
  questions: AdminPastPaperQuestionInput[]
}

export interface AdminPastPaperSaveResult {
  paper_id: number
  question_count: number
  question_ids: number[]
}

export interface AdminPastPaperListItem {
  paper_id: number
  subject_id: number
  subject_name: string
  exam_year: number
  paper_code: string | null
  title: string
  question_count: number
  created_at: string
}

export interface AdminPastPaperOptionDto {
  option_id: number
  option_label: string
  option_text: string
  is_correct: boolean
  position: number
}

export interface AdminPastPaperQuestionDto {
  question_id: number
  section_label: string
  question_number: string | null
  topic_id: number
  subtopic_id: number | null
  stem: string
  question_format: string
  primary_pedagogic_function: string | null
  explanation_text: string | null
  difficulty_level: number
  marks: number
  options: AdminPastPaperOptionDto[]
  /** Image attachments metadata. Bytes loaded separately via
   *  getQuestionAssetBytes so a paper full of images doesn't bloat the
   *  editor's initial load. */
  assets: QuestionAssetMetaDto[]
}

export type QuestionAssetScope = 'stem' | 'option' | 'explanation'

export interface QuestionAssetMetaDto {
  asset_id: number
  question_id: number
  scope: QuestionAssetScope
  /** option_id when scope === 'option', null otherwise */
  scope_ref: number | null
  mime_type: string
  byte_size: number
  position: number
  alt_text: string | null
}

export interface QuestionAssetBytesDto {
  mime_type: string
  bytes: number[]
}

export interface AdminPastPaperFullDto {
  paper_id: number
  subject_id: number
  exam_year: number
  paper_code: string | null
  title: string
  questions: AdminPastPaperQuestionDto[]
}

export interface RecoveredTextPageDto {
  page_number: number
  label: string
  confidence_score: number
  text: string
  preview: string | null
}

export interface RecoveredTextDto {
  source: string
  confidence_score: number
  text: string
  page_count: number
  recovered_from_ocr: boolean
  pages: RecoveredTextPageDto[]
}

// ── Commands ──────────────────────────────────────────────────────

export function adminSavePastPaper(
  input: AdminPastPaperSaveInput,
): Promise<AdminPastPaperSaveResult> {
  return ipc<AdminPastPaperSaveResult>('admin_save_past_paper', { input })
}

export function adminListPastPapers(
  subjectId: number | null = null,
): Promise<AdminPastPaperListItem[]> {
  return ipc<AdminPastPaperListItem[]>('admin_list_past_papers', { subjectId })
}

export function adminGetPastPaper(paperId: number): Promise<AdminPastPaperFullDto> {
  return ipc<AdminPastPaperFullDto>('admin_get_past_paper', { paperId })
}

export function adminDeletePastPaper(paperId: number): Promise<void> {
  return ipc<void>('admin_delete_past_paper', { paperId })
}

/**
 * Send a file's bytes to the backend, which persists it to a temp
 * file, runs the extractor (pdftotext / tesseract / docx adapter),
 * returns recovered text, then unlinks. Usable without the native
 * file-dialog plugin — plain <input type="file"> yields a File object
 * whose contents we read as Uint8Array and pass here.
 */
export function adminExtractPastPaperText(
  fileName: string,
  mimeType: string | null,
  fileBytes: Uint8Array,
): Promise<RecoveredTextDto> {
  return ipc<RecoveredTextDto>('admin_extract_past_paper_text', {
    fileName,
    mimeType,
    // Tauri serialises Array<number> as a Rust Vec<u8>; convert the typed
    // array to a regular number array so serde_json can round-trip it.
    fileBytes: Array.from(fileBytes),
  })
}

// ── Asset / image attachments ─────────────────────────────────────

export function adminAttachQuestionAsset(
  questionId: number,
  scope: QuestionAssetScope,
  scopeRef: number | null,
  mimeType: string,
  fileBytes: Uint8Array,
  altText: string | null = null,
): Promise<QuestionAssetMetaDto> {
  return ipc<QuestionAssetMetaDto>('admin_attach_question_asset', {
    questionId,
    scope,
    scopeRef,
    mimeType,
    fileBytes: Array.from(fileBytes),
    altText,
  })
}

export function adminDeleteQuestionAsset(assetId: number): Promise<void> {
  return ipc<void>('admin_delete_question_asset', { assetId })
}

export function listQuestionAssets(questionId: number): Promise<QuestionAssetMetaDto[]> {
  return ipc<QuestionAssetMetaDto[]>('list_question_assets', { questionId })
}

/** Public (non-admin) alias for the same read. Kept separate so the
 *  student-side code doesn't import from the admin module's broader
 *  API surface. */
export const listQuestionAssetsForStudent = listQuestionAssets

/** Fetch raw bytes + mime and wrap them in a browser-side object URL
 *  ready to drop into `<img src>`. Caller is responsible for calling
 *  `URL.revokeObjectURL` on the returned string when the image leaves
 *  the DOM (otherwise we leak memory per render). */
export async function fetchQuestionAssetObjectUrl(assetId: number): Promise<string> {
  const payload = await ipc<QuestionAssetBytesDto>('get_question_asset_bytes', { assetId })
  const uint8 = new Uint8Array(payload.bytes)
  const blob = new Blob([uint8], { type: payload.mime_type })
  return URL.createObjectURL(blob)
}
