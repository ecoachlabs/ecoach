/**
 * Heuristic text → draft past-paper segmenter.
 *
 * Past-paper PDFs/Word docs vary enormously in layout; a perfect
 * parser would need ML. This module aims for *useful-80%* instead:
 * detect the obvious structural markers (SECTION A/B headers, numbered
 * question prefixes, MCQ option letters), produce a draft paper the
 * admin can review and correct in the editor.
 *
 * Every draft question is flagged with `is_ambiguous` when confidence
 * is low — the UI should visually mark those rows so the admin knows
 * where to look first.
 */

export interface DraftOption {
  label: string
  text: string
  is_correct: boolean
}

export interface DraftQuestion {
  number: string | null
  stem: string
  question_format: 'mcq' | 'short_answer' | 'essay'
  options: DraftOption[]
  is_ambiguous: boolean
}

export interface DraftSection {
  label: string // "A", "B", or "" when no explicit section header
  kind: 'objective' | 'essay' | 'mixed'
  questions: DraftQuestion[]
}

export interface DraftPaper {
  sections: DraftSection[]
  raw_text: string
  warnings: string[]
}

/**
 * Parsed answer-key block. Maps question number (as string — matches
 * DraftQuestion.number which is also a string) to the option letter
 * that the key marks correct.
 */
export type AnswerKey = Record<string, string>

/** Result of cross-referencing an AnswerKey against a DraftPaper. */
export interface AnswerKeyApplyResult {
  updatedCount: number
  keyEntries: number
  /** Question numbers that appeared in the key but had no matching draft row. */
  unmatched: string[]
}

// ── Regex atlas (kept in one place so tweaks are easy) ───────────

/** Lines that kick off a SECTION A / PART I / Paper 2 block. */
const SECTION_HEADER =
  /^\s*(?:SECTION|PART|PAPER)\s+([A-Z0-9IVX]+)\b.*/i

/** Leading question number: 1. / 1) / (1) / Q1. / 1a. etc. */
const QUESTION_PREFIX =
  /^\s*(?:Q\.?\s*)?\(?(\d{1,3})[a-zA-Z]?\)?[.)\]:]\s+(?=\S)/

/** Leading MCQ option: A. / A) / (A) / a. — but NOT "A" as article or short word. */
const OPTION_PREFIX =
  /^\s*\(?([A-Ea-e])\)?[.)\]]\s+(?=\S)/

/** Instruction-like noise that should NOT become a question stem. */
const INSTRUCTION_LINE =
  /^(?:\s*)(?:answer\s+all|answer\s+any|do\s+not|write\s+your|time\s+allowed|instructions?|attempt)\b/i

// ── Helpers ──────────────────────────────────────────────────────

function normaliseWhitespace(text: string): string {
  return text
    .replace(/\r\n/g, '\n')
    .replace(/\f/g, '\n\n') // form-feed ⇒ paragraph break
    .replace(/[ \t]+/g, ' ')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

function stripLeadingMarker(line: string, re: RegExp): string {
  const match = line.match(re)
  if (!match) return line.trim()
  return line.slice(match[0].length).trim()
}

function inferKind(questions: DraftQuestion[]): 'objective' | 'essay' | 'mixed' {
  if (questions.length === 0) return 'objective'
  const objective = questions.filter(q => q.question_format === 'mcq').length
  const essay = questions.length - objective
  if (objective === 0) return 'essay'
  if (essay === 0) return 'objective'
  return 'mixed'
}

// ── The main pass ────────────────────────────────────────────────

interface InProgressQuestion {
  number: string | null
  stemLines: string[]
  options: DraftOption[]
}

function finaliseQuestion(wip: InProgressQuestion): DraftQuestion | null {
  const stem = wip.stemLines.join(' ').trim()
  if (stem.length === 0 && wip.options.length === 0) return null

  // Heuristic: if a question has ≥2 options, call it MCQ. If no options,
  // short_answer vs essay hinges on stem length — tail ≥120 chars is
  // essay territory. These are defaults; the admin can change.
  let format: DraftQuestion['question_format']
  if (wip.options.length >= 2) format = 'mcq'
  else if (stem.length > 180) format = 'essay'
  else format = 'short_answer'

  const is_ambiguous =
    stem.length < 15 ||
    (format === 'mcq' && wip.options.length < 3) ||
    (format === 'mcq' && !wip.options.some(o => o.is_correct))

  return {
    number: wip.number,
    stem,
    question_format: format,
    options: wip.options,
    is_ambiguous,
  }
}

export function segmentPastPaperText(rawText: string): DraftPaper {
  const text = normaliseWhitespace(rawText)
  const lines = text.split('\n')

  const sections: DraftSection[] = []
  const warnings: string[] = []

  // We always start with an implicit section "" that sweeps up anything
  // authored before an explicit SECTION header appears.
  let currentLabel = ''
  let currentQuestions: DraftQuestion[] = []
  let wip: InProgressQuestion | null = null

  function commitSection(): void {
    if (wip) {
      const q = finaliseQuestion(wip)
      if (q) currentQuestions.push(q)
      wip = null
    }
    if (currentQuestions.length > 0) {
      sections.push({
        label: currentLabel,
        kind: inferKind(currentQuestions),
        questions: currentQuestions,
      })
    }
    currentQuestions = []
  }

  for (const rawLine of lines) {
    const line = rawLine.trim()
    if (line.length === 0) {
      // Blank lines end an MCQ option run but don't close a question.
      continue
    }

    // (1) Section header flips the current section.
    const secMatch = line.match(SECTION_HEADER)
    if (secMatch) {
      commitSection()
      const tokens = secMatch[1].toUpperCase()
      // Accept letters A/B/C/... and roman numerals; fall back to raw.
      currentLabel = tokens
      continue
    }

    // (2) Skip obvious instruction / boilerplate lines.
    if (INSTRUCTION_LINE.test(line)) continue

    // (3) New question number starts a fresh WIP.
    const qMatch = line.match(QUESTION_PREFIX)
    if (qMatch) {
      if (wip) {
        const done = finaliseQuestion(wip)
        if (done) currentQuestions.push(done)
      }
      wip = {
        number: qMatch[1],
        stemLines: [stripLeadingMarker(line, QUESTION_PREFIX)],
        options: [],
      }
      continue
    }

    // (4) Option line (only meaningful inside a WIP question).
    if (wip) {
      const oMatch = line.match(OPTION_PREFIX)
      if (oMatch) {
        wip.options.push({
          label: oMatch[1].toUpperCase(),
          text: stripLeadingMarker(line, OPTION_PREFIX),
          is_correct: false,
        })
        continue
      }

      // (5) Continuation: append to stem (or most recent option).
      if (wip.options.length > 0) {
        wip.options[wip.options.length - 1].text += ' ' + line
      } else {
        wip.stemLines.push(line)
      }
    }
    // Lines that arrive with no open WIP (e.g. preamble, header banners)
    // are dropped — they'd otherwise pollute stems.
  }

  // Close whatever's open.
  commitSection()

  // Post-processing: if we found zero questions, flag it loudly.
  if (sections.every(s => s.questions.length === 0)) {
    warnings.push(
      'No questions were detected. The extractor may have produced layout-heavy text; try the Manual tab.',
    )
  }

  // If only one section was inferred and it's labelled "" (nothing
  // explicit), give it a friendlier default label.
  if (sections.length === 1 && sections[0].label === '') {
    sections[0].label = 'A'
  }

  // Answer-key cross-reference. If the source text contains a block
  // shaped like "1. B  2. C  3. A" at the tail end — typical of the
  // back page of a past-paper booklet — auto-apply it to any MCQ rows
  // that don't already have a correct option marked. Non-destructive:
  // rows with existing `is_correct` flags are left alone.
  const answerKey = detectAnswerKey(text)
  if (Object.keys(answerKey).length > 0) {
    const applied = applyAnswerKey(sections, answerKey)
    if (applied.updatedCount > 0) {
      warnings.push(
        `Answer key detected (${applied.keyEntries} entries) · auto-marked ${applied.updatedCount} option(s). Review each.`,
      )
    }
    if (applied.unmatched.length > 0 && applied.unmatched.length < 20) {
      warnings.push(
        `Answer-key entries without matching draft rows: ${applied.unmatched.join(', ')}`,
      )
    }
  }

  return { sections, raw_text: text, warnings }
}

// ── Answer-key detection ──────────────────────────────────────────

/**
 * Scan the text for a plausible answer key. Two shapes supported:
 *   (a) one-per-line:   "1. B", "1) C", "1 - A"   (one entry per line)
 *   (b) inline runs:    "1. B  2. C  3. D  4. A"  (multiple per line)
 *
 * We look only at the LAST 20% of the document so we don't confuse
 * genuine questions numbered early in the paper. Needs ≥ 5 entries in
 * tight succession to qualify (kills false positives on pages that
 * happen to have "1. Some text" in them).
 */
export function detectAnswerKey(rawText: string): AnswerKey {
  const text = rawText.replace(/\r\n/g, '\n')
  const tailStart = Math.floor(text.length * 0.8)
  const tail = text.slice(tailStart)

  const key: AnswerKey = {}

  // Pattern (a): one per line. Permissive — accepts any separator.
  const lineRe = /^\s*(\d{1,3})\s*[.)\-:]\s*([A-Ea-e])\s*$/gm
  let m: RegExpExecArray | null
  while ((m = lineRe.exec(tail)) !== null) {
    const num = m[1]
    const letter = m[2].toUpperCase()
    if (!(num in key)) key[num] = letter
  }

  // Pattern (b): inline runs "1. B  2. C  3. A" — catch the
  // number→letter pairs regardless of whitespace.
  const inlineRe = /\b(\d{1,3})\s*[.)\-:]\s*([A-Ea-e])\b/g
  // Only trust this form if there are multiple pairs on a single line;
  // otherwise "1. The first reason is ..." would pollute the map.
  for (const line of tail.split('\n')) {
    const matches = Array.from(line.matchAll(inlineRe))
    if (matches.length >= 3) {
      for (const mm of matches) {
        const num = mm[1]
        const letter = mm[2].toUpperCase()
        if (!(num in key)) key[num] = letter
      }
    }
  }

  // Confidence gate: ≥5 entries to count as a real key.
  if (Object.keys(key).length < 5) return {}
  return key
}

/**
 * Apply a detected answer key to a DraftPaper in-place. For each MCQ
 * draft whose `number` matches a key entry AND that has no correct
 * option yet, set the matching letter's is_correct=true. Returns
 * counts so the caller can surface a warning.
 */
export function applyAnswerKey(
  sections: DraftSection[],
  key: AnswerKey,
): AnswerKeyApplyResult {
  const keyEntries = Object.keys(key).length
  let updatedCount = 0
  const matched = new Set<string>()

  for (const section of sections) {
    for (const q of section.questions) {
      if (q.number == null) continue
      const letter = key[q.number]
      if (!letter) continue
      matched.add(q.number)

      // Only auto-mark if the row is MCQ-shaped and has no correct pick.
      if (q.options.length === 0) continue
      if (q.options.some(o => o.is_correct)) continue

      const targetIndex = letter.charCodeAt(0) - 65
      const target = q.options[targetIndex] ?? q.options.find(o => o.label === letter)
      if (!target) continue
      target.is_correct = true
      // A row that had ≥2 options but no correct answer was flagged
      // ambiguous earlier — now that we've resolved it, clear the flag.
      q.is_ambiguous = false
      updatedCount += 1
    }
  }

  const unmatched = Object.keys(key).filter(n => !matched.has(n))
  return { updatedCount, keyEntries, unmatched }
}
