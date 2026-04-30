/**
 * CSV bulk-import parser for past-paper authoring.
 *
 * ──────────────────────────────────────────────────────────────────
 * Schema
 * ──────────────────────────────────────────────────────────────────
 *
 *   section,number,type,stem,topic,option_a,option_b,option_c,option_d,option_e,correct,answer,explanation,marks,difficulty
 *
 * • `section`    — A, B, … (section label)
 * • `number`     — question number within the section (free-form string)
 * • `type`       — mcq | mcq_multi | true_false | short_answer | numeric | essay
 *                  (fill_blank is NOT supported in CSV; use Manual / JSON)
 * • `stem`       — the question text (quote if it contains commas / newlines)
 * • `topic`      — topic NAME (case-insensitive match inside the current
 *                  subject). If empty, the first topic is used.
 * • `option_a..option_e` — MCQ choice texts. Leave empty for non-MCQ types.
 * • `correct`    — MCQ: letter(s). Single = "A". Multi = "A;C". True/False = "A" or "B".
 *                  Non-MCQ: leave blank (use `answer` column).
 * • `answer`     — short_answer / numeric: the accepted answer text.
 *                  essay: empty (use `explanation` as the marking guide).
 * • `explanation`— optional marking hint / model answer.
 * • `marks`      — optional integer, default per type.
 * • `difficulty` — optional 0–10000 (basis points), default 5000.
 *
 * The parser accepts arbitrary column ORDER — it looks for column names in
 * the header row. Unknown columns are ignored silently. Quoted fields can
 * span lines and embed commas or double quotes (""double"" for escape).
 *
 * Output matches the DraftPaper shape used by the segmenter so the rest
 * of the author UI can feed CSV rows through the same review flow as
 * PDF/Word extraction.
 */

import type { DraftPaper, DraftSection, DraftQuestion, DraftOption } from './pastPaperSegmenter'

export interface CsvImportResult extends DraftPaper {
  /** Optional topic name per row — the importer resolves these to ids
   *  once the subject-specific topic list is known. */
  topicHints: (string | null)[]
  /** Answer column text for non-MCQ rows — fed into explanation_text
   *  (or a distinct "model answer" field once that split exists). */
  answerHints: (string | null)[]
  /** Advisory messages for the admin (bad rows, unknown types, etc.). */
  warnings: string[]
}

// ── Primitive CSV tokenizer ────────────────────────────────────────
// Full-spec CSV reader: handles quoted fields, embedded commas,
// embedded newlines, and ""escaped"" double quotes. Kept inline so we
// don't pull in papaparse for a 50-row import flow.
function parseCsv(text: string): string[][] {
  const rows: string[][] = []
  let field = ''
  let row: string[] = []
  let inQuotes = false
  let i = 0

  while (i < text.length) {
    const ch = text[i]
    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') { field += '"'; i += 2; continue }
        inQuotes = false; i += 1; continue
      }
      field += ch
      i += 1
      continue
    }
    // not in quotes
    if (ch === '"') { inQuotes = true; i += 1; continue }
    if (ch === ',') { row.push(field); field = ''; i += 1; continue }
    if (ch === '\r') { i += 1; continue } // swallow \r of \r\n
    if (ch === '\n') {
      row.push(field); field = ''
      rows.push(row); row = []
      i += 1
      continue
    }
    field += ch
    i += 1
  }
  // Flush the trailing cell + row (no trailing newline).
  if (field.length > 0 || row.length > 0) {
    row.push(field)
    rows.push(row)
  }
  return rows.filter(r => !(r.length === 1 && r[0].trim() === ''))
}

// ── Column resolution ──────────────────────────────────────────────
function normaliseHeader(s: string): string {
  return s.trim().toLowerCase().replace(/[\s-]+/g, '_')
}

function columnIndex(headers: string[], candidates: string[]): number {
  for (const c of candidates) {
    const idx = headers.indexOf(c)
    if (idx >= 0) return idx
  }
  return -1
}

// ── Type normalisation ─────────────────────────────────────────────
const SUPPORTED_TYPES = new Set([
  'mcq',
  'mcq_multi',
  'true_false',
  'short_answer',
  'numeric',
  'essay',
])

function normaliseType(raw: string): DraftQuestion['question_format'] | 'mcq_multi' | null {
  const key = raw.trim().toLowerCase().replace(/[\s-]+/g, '_')
  // Map common variants admins might type.
  const alias: Record<string, string> = {
    multiple_choice: 'mcq',
    multi_choice: 'mcq',
    multiple_choice_single: 'mcq',
    multiple_choice_multi: 'mcq_multi',
    multi_correct: 'mcq_multi',
    multiple_answer: 'mcq_multi',
    tf: 'true_false',
    true_or_false: 'true_false',
    short: 'short_answer',
    number: 'numeric',
    numeric_answer: 'numeric',
    long: 'essay',
    long_answer: 'essay',
  }
  const canonical = alias[key] ?? key
  if (!SUPPORTED_TYPES.has(canonical)) return null
  if (canonical === 'mcq_multi') return 'mcq_multi'
  if (canonical === 'essay') return 'essay'
  if (canonical === 'mcq') return 'mcq'
  // short_answer / numeric / true_false — the segmenter's DraftQuestion
  // type only knows mcq/short_answer/essay. We fold numeric + true_false
  // into short_answer and let the author-view detect the refinement
  // from the CSV's original type string via the mapping step below.
  return 'short_answer'
}

// Letter → zero-based index (A=0, B=1, …). Case-insensitive.
function letterToIndex(letter: string): number {
  const code = letter.trim().toUpperCase().charCodeAt(0)
  if (Number.isNaN(code) || code < 65 || code > 90) return -1
  return code - 65
}

// ── Public entrypoint ──────────────────────────────────────────────
export function parsePastPaperCsv(csvText: string): CsvImportResult {
  const warnings: string[] = []
  const rows = parseCsv(csvText)
  if (rows.length < 2) {
    return {
      sections: [],
      raw_text: csvText,
      warnings: ['CSV is empty or has no data rows.'],
      topicHints: [],
      answerHints: [],
    }
  }

  const headers = rows[0].map(normaliseHeader)
  const body = rows.slice(1)

  const ix = {
    section: columnIndex(headers, ['section', 'section_label']),
    number: columnIndex(headers, ['number', 'question_number', 'q', 'qno']),
    type: columnIndex(headers, ['type', 'question_type', 'format']),
    stem: columnIndex(headers, ['stem', 'question', 'prompt']),
    topic: columnIndex(headers, ['topic', 'topic_name']),
    correct: columnIndex(headers, ['correct', 'correct_option', 'answer_letter']),
    answer: columnIndex(headers, ['answer', 'model_answer', 'expected']),
    explanation: columnIndex(headers, ['explanation', 'explanation_text', 'marking_guide']),
    marks: columnIndex(headers, ['marks', 'mark', 'points']),
    difficulty: columnIndex(headers, ['difficulty', 'difficulty_level']),
    options: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']
      .map(l => columnIndex(headers, [`option_${l}`, `opt_${l}`, `choice_${l}`])),
  }

  // Require the minimum set of columns. Missing optional ones get
  // flagged but the import still proceeds with defaults.
  if (ix.type < 0 || ix.stem < 0) {
    warnings.push(
      `CSV is missing required columns. Expected at least "type" and "stem"; got [${headers.join(', ')}].`,
    )
    return { sections: [], raw_text: csvText, warnings, topicHints: [], answerHints: [] }
  }

  // Accumulate by section_label so each section's questions stay grouped.
  const sectionMap = new Map<string, DraftSection>()
  const topicHints: (string | null)[] = []
  const answerHints: (string | null)[] = []

  body.forEach((cells, rowIdx) => {
    const rowNumber = rowIdx + 2 // +1 for header, +1 for 1-index
    const rawType = (cells[ix.type] ?? '').trim()
    const stemText = (cells[ix.stem] ?? '').trim()
    if (stemText.length === 0) {
      warnings.push(`Row ${rowNumber}: skipped — empty stem.`)
      return
    }
    const format = normaliseType(rawType)
    if (format == null) {
      warnings.push(`Row ${rowNumber}: skipped — unknown type "${rawType}".`)
      return
    }

    const sectionLabel = ((ix.section >= 0 ? cells[ix.section] : '') || 'A').trim() || 'A'
    const qNumber = (ix.number >= 0 ? (cells[ix.number] ?? '').trim() : '') || null

    // MCQ / multi / true_false → gather options + parse `correct` letters.
    const isMcqShaped =
      rawType.toLowerCase().includes('mcq') ||
      rawType.toLowerCase().includes('multiple') ||
      rawType.toLowerCase().includes('multi') ||
      rawType.toLowerCase().includes('true')
    const options: DraftOption[] = []
    if (isMcqShaped) {
      const correctLetters = (cells[ix.correct] ?? '')
        .trim()
        .split(/[;,\s]+/)
        .map(s => s.trim())
        .filter(s => s.length > 0)
        .map(letterToIndex)
        .filter(i => i >= 0)

      const optionTexts: string[] = ix.options
        .filter(ci => ci >= 0)
        .map(ci => (cells[ci] ?? '').trim())
      // Drop empty trailing options (admins often leave option_e blank).
      let lastFilled = -1
      optionTexts.forEach((t, i) => { if (t.length > 0) lastFilled = i })
      const keptOptionTexts = optionTexts.slice(0, lastFilled + 1)

      // True/false: auto-seed if no options were provided.
      if (keptOptionTexts.length === 0 && rawType.toLowerCase().includes('true')) {
        keptOptionTexts.push('True', 'False')
      }

      if (keptOptionTexts.length < 2) {
        warnings.push(`Row ${rowNumber}: MCQ-shaped row has fewer than 2 options — review manually.`)
      }
      if (correctLetters.length === 0) {
        warnings.push(`Row ${rowNumber}: no "correct" letter — all options left unmarked.`)
      }

      keptOptionTexts.forEach((text, i) => {
        options.push({
          label: String.fromCharCode(65 + i),
          text,
          is_correct: correctLetters.includes(i),
        })
      })
    }

    // Final DraftQuestion shape — the segmenter's union is ('mcq' | 'short_answer' | 'essay').
    // CSV can distinguish more — we let the author UI's downstream
    // mapping detect multi_correct / true_false / numeric by shape.
    const draftFormat: DraftQuestion['question_format'] =
      format === 'mcq_multi' ? 'mcq' : format

    const q: DraftQuestion = {
      number: qNumber,
      stem: stemText,
      question_format: draftFormat,
      options,
      is_ambiguous: false,
    }

    if (!sectionMap.has(sectionLabel)) {
      sectionMap.set(sectionLabel, {
        label: sectionLabel,
        kind: 'mixed',
        questions: [],
      })
    }
    sectionMap.get(sectionLabel)!.questions.push(q)

    topicHints.push(ix.topic >= 0 ? (cells[ix.topic] ?? '').trim() || null : null)
    answerHints.push(ix.answer >= 0 ? (cells[ix.answer] ?? '').trim() || null : null)
    // explanation, marks, difficulty flow through via the authoring UI
    // attaching them from explanationHints / marksHints when it ingests
    // the draft — out of scope for the segmenter-shaped DTO.
  })

  const sections = Array.from(sectionMap.values()).sort((a, b) =>
    a.label.localeCompare(b.label),
  )
  // Per-section kind = inferred from its questions.
  for (const s of sections) {
    const objective = s.questions.filter(q => q.question_format === 'mcq').length
    const essay = s.questions.length - objective
    s.kind = objective === 0 ? 'essay' : essay === 0 ? 'objective' : 'mixed'
  }

  return { sections, raw_text: csvText, warnings, topicHints, answerHints }
}

// ── Template generator ─────────────────────────────────────────────
/**
 * Generate a ready-to-fill CSV with the full schema + two example
 * rows. The string is UTF-8 with a BOM so Excel opens it with correct
 * encoding when the admin double-clicks.
 */
export function generatePastPaperCsvTemplate(): string {
  const BOM = '\uFEFF'
  const header =
    'section,number,type,stem,topic,option_a,option_b,option_c,option_d,option_e,correct,answer,explanation,marks,difficulty'
  const example1 =
    'A,1,mcq,"What is the value of 3 + 4?",Whole Number Operations,5,6,7,8,,C,,"3 + 4 = 7",1,3000'
  const example2 =
    'B,1,essay,"Discuss three causes of the Industrial Revolution.",World History,,,,,,,,' +
    '"Outline: (1) agricultural surplus, (2) technological innovation, (3) access to raw materials. 2 marks per cause.",10,6000'
  return [BOM + header, example1, example2, ''].join('\n')
}
