<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import MathText from '@/components/question/MathText.vue'
import { sanitizeLearnerSnippet, stripCurriculumCodes } from '@/utils/learnerCopy'

interface OptionSummary {
  id: number
  label: string
  text: string
  is_correct?: boolean
  distractor_intent?: string | null
  misconception_id?: number | null
}

interface SectionMap {
  howToSolve?: string
  stepByStep?: string
  optionReasons?: string
  wrongAnswerReveals?: string
  originalSolutionNote?: string
}

interface ParsedDistractorIntent {
  misstep: string
  reveal: string
  attention: string
}

interface OptionInsight {
  id: number
  label: string
  text: string
  isCorrect: boolean
  gives: string
  required: string
  comparisonLine: string
  misstep: string
  reveal: string
  attention: string
}

interface SolutionStep {
  note: string
  work: string
}

interface SolutionBundle {
  intro: string
  steps: SolutionStep[]
  mistakesToAvoid: string[]
}

const props = withDefaults(defineProps<{
  isCorrect: boolean
  questionStem?: string | null
  explanation?: string | null
  errorType?: string | null
  diagnosisSummary?: string | null
  recommendedAction?: string | null
  selectedOptionText: string
  correctOptionText?: string | null
  misconceptionInfo?: string | null
  selectedOptionId?: number | null
  correctOptionId?: number | null
  options?: OptionSummary[]
  loadingExplanation?: boolean
  showReviewAction?: boolean
  reviewLabel?: string
  autoAdvanceOnCorrect?: boolean
  autoAdvanceOnWrong?: boolean
  autoAdvanceDelayMs?: number
  autoAdvanceDelayWrongMs?: number
}>(), {
  questionStem: null,
  explanation: null,
  errorType: null,
  diagnosisSummary: null,
  recommendedAction: null,
  correctOptionText: null,
  misconceptionInfo: null,
  selectedOptionId: null,
  correctOptionId: null,
  options: () => [],
  loadingExplanation: false,
  showReviewAction: true,
  reviewLabel: 'Review Mistakes',
  autoAdvanceOnCorrect: false,
  autoAdvanceOnWrong: false,
  autoAdvanceDelayMs: 2400,
  autoAdvanceDelayWrongMs: 5600,
})

const emit = defineEmits<{
  next: []
  review: []
}>()

const errorLabels: Record<string, string> = {
  knowledge_gap: 'Knowledge gap',
  conceptual_confusion: 'Concept confusion',
  recognition_failure: 'Recognition slip',
  execution_error: 'Execution slip',
  carelessness: 'Careless slip',
  pressure_breakdown: 'Pressure slip',
  expression_weakness: 'Expression slip',
  speed_error: 'Speed slip',
  guessing_detected: 'Guessing',
  misconception_triggered: 'Misconception',
  timed_out: 'Time ran out',
}

const countdownMs = ref(0)
let autoAdvanceTimer: number | null = null
let countdownTimer: number | null = null

function clearAutoAdvanceTimers() {
  if (autoAdvanceTimer !== null) {
    window.clearTimeout(autoAdvanceTimer)
    autoAdvanceTimer = null
  }
  if (countdownTimer !== null) {
    window.clearInterval(countdownTimer)
    countdownTimer = null
  }
  countdownMs.value = 0
}

function normalizeWhitespace(value: string): string {
  return value.replace(/\s+/g, ' ').trim()
}

function cleanLearnerText(value: string | null | undefined): string {
  if (!value) return ''
  return normalizeWhitespace(sanitizeLearnerSnippet(value, { dropSyllabusMeta: true }))
}

function cleanDiagnosticPhrase(value: string | null | undefined): string {
  if (!value) return ''
  return cleanLearnerText(value)
    .replace(/^Learner may demonstrate\s+/i, '')
    .replace(/^The learner may\s+/i, '')
    .replace(/\s+while working on\s+[^.]*tasks?\.?/gi, '')
    .replace(/\s+by using that shortcut or incorrect step\.?/gi, '')
    .replace(/^The likely error is:\s*/i, '')
    .replace(/^Likely misstep:\s*/i, '')
    .replace(/^Misstep:\s*/i, '')
    .trim()
}

function cleanupReasonText(value: string | null | undefined): string {
  if (!value) return ''
  return cleanLearnerText(value)
    .replace(/^correct because\s*/i, '')
    .replace(/^wrong because\s*/i, '')
    .replace(/^Option\s+[A-D]\s+is\s+not\s+correct\s+because\s*/i, '')
    .replace(/^Option\s+[A-D]\s+/i, '')
    .replace(/\s*The likely error is:\s*.*$/i, '')
    .replace(/\s*Likely misstep:\s*.*$/i, '')
    .trim()
}

function titleCaseWords(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ')
}

function formatCategory(value: string | null | undefined): string {
  const cleaned = cleanDiagnosticPhrase(value)
    .replace(/\bmisstep:\s*/i, '')
    .replace(/\bconfusion\b/gi, 'confusion')
    .trim()
  if (!cleaned) return ''
  if (/^[A-Z0-9 _-]+$/.test(cleaned)) {
    return titleCaseWords(cleaned.replace(/[_-]+/g, ' '))
  }
  return cleaned.charAt(0).toUpperCase() + cleaned.slice(1)
}

function extractSections(raw: string | null | undefined): SectionMap {
  if (!raw) return {}
  const labels = [
    { key: 'howToSolve', label: 'How to solve:' },
    { key: 'stepByStep', label: 'Step-by-step solution:' },
    { key: 'optionReasons', label: 'Why each option is right or wrong:' },
    { key: 'wrongAnswerReveals', label: 'What each wrong answer reveals:' },
    { key: 'originalSolutionNote', label: 'Original solution note:' },
  ] as const

  const lower = raw.toLowerCase()
  const found = labels
    .map(item => ({ ...item, index: lower.indexOf(item.label.toLowerCase()) }))
    .filter(item => item.index >= 0)
    .sort((a, b) => a.index - b.index)

  const sections: SectionMap = {}
  for (let index = 0; index < found.length; index += 1) {
    const current = found[index]
    const start = current.index + current.label.length
    const end = index + 1 < found.length ? found[index + 1].index : raw.length
    sections[current.key] = raw.slice(start, end).trim()
  }
  return sections
}

function parseLetteredEntries(value: string | null | undefined): Record<string, string> {
  if (!value) return {}
  const compact = value.replace(/\n+/g, ' ').replace(/\s{2,}/g, ' ').trim()
  const entries: Record<string, string> = {}
  const pattern = /(?:^|\s)([A-D]):\s*([\s\S]*?)(?=(?:\s+[A-D]:\s)|$)/g
  let match: RegExpExecArray | null

  while ((match = pattern.exec(compact)) !== null) {
    entries[match[1]] = match[2].trim()
  }

  return entries
}

function extractBetween(value: string, startLabel: string, endLabel?: string): string {
  const lower = value.toLowerCase()
  const start = lower.indexOf(startLabel.toLowerCase())
  if (start < 0) return ''
  const sliceStart = start + startLabel.length
  const sliceEnd = endLabel
    ? lower.indexOf(endLabel.toLowerCase(), sliceStart)
    : -1
  return value.slice(sliceStart, sliceEnd >= 0 ? sliceEnd : value.length).trim()
}

function parseDistractorIntent(value: string | null | undefined): ParsedDistractorIntent {
  const source = stripCurriculumCodes(value ?? '')
  return {
    misstep: cleanDiagnosticPhrase(extractBetween(source, 'Misstep:', 'Reveals:')),
    reveal: cleanDiagnosticPhrase(extractBetween(source, 'Reveals:', 'Needs attention:')),
    attention: cleanDiagnosticPhrase(extractBetween(source, 'Needs attention:')),
  }
}

function extractComparison(value: string | null | undefined): { gives: string; required: string } {
  if (!value) return { gives: '', required: '' }
  const cleaned = value.replace(/\s+/g, ' ').trim()
  const match = cleaned.match(/gives\s+(.+?),\s+while the required result is\s+(.+?)(?:\.|$)/i)
  if (!match) return { gives: '', required: '' }
  return {
    gives: stripCurriculumCodes(match[1].trim()),
    required: stripCurriculumCodes(match[2].trim()),
  }
}

function parseRevealEntry(value: string | null | undefined): { reveal: string; attention: string } {
  if (!value) return { reveal: '', attention: '' }
  const revealMatch = value.match(/reveals\s+(.+?);/i)
  const attentionMatch = value.match(/needs attention on\s+(.+?)(?:\.|$)/i)
  return {
    reveal: cleanDiagnosticPhrase(revealMatch?.[1] ?? ''),
    attention: cleanDiagnosticPhrase(attentionMatch?.[1] ?? ''),
  }
}

function dedupeStrings(values: string[]): string[] {
  const seen = new Set<string>()
  const output: string[] = []
  for (const raw of values) {
    const cleaned = normalizeWhitespace(raw)
    if (!cleaned) continue
    const key = cleaned.toLowerCase()
    if (seen.has(key)) continue
    seen.add(key)
    output.push(cleaned)
  }
  return output
}

function rewriteMistakeToAvoid(value: string): string {
  const cleaned = cleanDiagnosticPhrase(value)
  if (!cleaned) return ''
  const lower = cleaned.toLowerCase()

  if (lower.includes('visible number')) {
    return 'Do not stop at the option that simply repeats a number from the question.'
  }
  if (lower.includes('given information')) {
    return 'Keep separate what the question gives you and what it asks you to find.'
  }
  if (lower.includes('operation') || lower.includes('formula')) {
    return 'Check the operation before you calculate.'
  }
  if (lower.includes('final option') || lower.includes('condition in the stem')) {
    return 'Check that your final answer satisfies the exact condition in the question.'
  }
  if (lower.includes('guided examples') || lower.includes('source exemplars')) {
    return 'Slow down and check each step before you compare the options.'
  }
  if (lower.startsWith('use ')) {
    return cleaned.charAt(0).toUpperCase() + cleaned.slice(1)
  }
  return cleaned.endsWith('.') ? cleaned : `${cleaned}.`
}

function parseFractionFromStem(value: string): { numerator: number; denominator: number } | null {
  const latexMatch = value.match(/\\frac\s*\{(-?\d+)\}\s*\{(-?\d+)\}/)
  if (latexMatch) {
    return {
      numerator: Number(latexMatch[1]),
      denominator: Number(latexMatch[2]),
    }
  }

  const plainMatch = value.match(/\b(-?\d+)\s*\/\s*(-?\d+)\b/)
  if (!plainMatch) return null
  return {
    numerator: Number(plainMatch[1]),
    denominator: Number(plainMatch[2]),
  }
}

function gcd(a: number, b: number): number {
  let left = Math.abs(a)
  let right = Math.abs(b)
  while (right !== 0) {
    const next = left % right
    left = right
    right = next
  }
  return left || 1
}

function formatWithGrouping(value: bigint): string {
  const negative = value < 0n
  let text = (negative ? -value : value).toString()
  text = text.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return negative ? `-${text}` : text
}

function placeName(value: bigint): string {
  const labels: Record<string, string> = {
    '1': 'ones',
    '10': 'tens',
    '100': 'hundreds',
    '1000': 'thousands',
    '10000': 'ten-thousands',
    '100000': 'hundred-thousands',
    '1000000': 'millions',
    '10000000': 'ten-millions',
    '100000000': 'hundred-millions',
    '1000000000': 'billions',
  }
  return labels[value.toString()] ?? `${formatWithGrouping(value)}s`
}

function parseWholeNumberPlace(value: string): bigint | null {
  const normalized = value.toLowerCase().replace(/[.?!]/g, '').trim()
  if (/^\d[\d,]*$/.test(normalized)) {
    return BigInt(normalized.replace(/,/g, ''))
  }

  const words: Array<[string, bigint]> = [
    ['hundred million', 100000000n],
    ['ten million', 10000000n],
    ['million', 1000000n],
    ['hundred thousand', 100000n],
    ['ten thousand', 10000n],
    ['thousand', 1000n],
    ['hundred', 100n],
    ['ten', 10n],
  ]

  for (const [label, amount] of words) {
    if (normalized === label || normalized === `${label}s`) return amount
  }
  return null
}

function buildFractionSolution(stem: string): SolutionBundle | null {
  const lower = stem.toLowerCase()
  if (!/(simplest form|lowest terms|reduce)/i.test(lower)) return null
  const fraction = parseFractionFromStem(stem)
  if (!fraction || fraction.denominator === 0) return null

  const commonFactor = gcd(fraction.numerator, fraction.denominator)
  const reducedNumerator = fraction.numerator / commonFactor
  const reducedDenominator = fraction.denominator / commonFactor
  const reducedFraction = `\\frac{${reducedNumerator}}{${reducedDenominator}}`

  return {
    intro: 'Reduce the fraction by dividing the numerator and denominator by the same greatest common factor.',
    steps: [
      {
        note: 'Find the greatest common factor of the numerator and denominator.',
        work: `gcf(${fraction.numerator}, ${fraction.denominator}) = ${commonFactor}`,
      },
      {
        note: 'Divide both parts of the fraction by that factor.',
        work: `\\frac{${fraction.numerator} \\div ${commonFactor}}{${fraction.denominator} \\div ${commonFactor}} = ${reducedFraction}`,
      },
      {
        note: 'Write the reduced fraction.',
        work: reducedFraction,
      },
    ],
    mistakesToAvoid: [
      'Do not divide only the top number or only the bottom number.',
      'Use the same factor on both parts of the fraction.',
      'Stop only when the numerator and denominator no longer share a factor greater than 1.',
    ],
  }
}

function buildWholeNumberRoundingSolution(stem: string): SolutionBundle | null {
  const cleanedStem = stripCurriculumCodes(stem).replace(/\s+/g, ' ').trim()
  const match = cleanedStem.match(/round\s+(-?[\d,]+)\s+to\s+the\s+nearest\s+([a-z0-9,\-\s]+?)(?:[.?!]|$)/i)
  if (!match) return null

  const originalValue = BigInt(match[1].replace(/,/g, ''))
  const roundingPlace = parseWholeNumberPlace(match[2])
  if (!roundingPlace || roundingPlace < 10n) return null

  const rightPlace = roundingPlace / 10n
  const roundedValue = ((originalValue + roundingPlace / 2n) / roundingPlace) * roundingPlace
  const targetDigit = Number((originalValue / roundingPlace) % 10n)
  const checkDigit = Number((originalValue / rightPlace) % 10n)
  const targetPlace = placeName(roundingPlace)
  const checkPlace = placeName(rightPlace)

  return {
    intro: `Round the number by checking the digit immediately to the right of the ${targetPlace.slice(0, -1)} place.`,
    steps: [
      {
        note: 'Locate the place you are rounding to and the digit just to its right.',
        work: `In ${formatWithGrouping(originalValue)}, the ${targetPlace} digit is ${targetDigit} and the ${checkPlace} digit is ${checkDigit}.`,
      },
      {
        note: 'Use the digit on the right to decide whether to keep the place value or round it up.',
        work: checkDigit >= 5
          ? `${checkDigit} is 5 or more, so the ${targetPlace} digit increases by 1.`
          : `${checkDigit} is less than 5, so the ${targetPlace} digit stays the same.`,
      },
      {
        note: 'Replace every digit to the right with zeroes.',
        work: `${formatWithGrouping(originalValue)} -> ${formatWithGrouping(roundedValue)}`,
      },
    ],
    mistakesToAvoid: [
      `Always check the ${checkPlace} digit, not the one you are rounding to.`,
      'If the digit on the right is 5 or more, round up. If it is less than 5, keep the digit the same.',
      'After rounding, every digit to the right of that place becomes 0.',
    ],
  }
}

function extractMethodHint(sections: SectionMap): string {
  const howToSolve = sections.howToSolve ?? ''
  const methodMatch = howToSolve.match(/which\s+(.+?)\s+method is needed/i)
  if (methodMatch) {
    return `${cleanLearnerText(methodMatch[1])} method`
  }

  return cleanLearnerText(sections.originalSolutionNote)
}

function extractWorkingHint(sections: SectionMap): string {
  const stepText = sections.stepByStep ?? ''
  const carried = extractBetween(stepText, 'Carry out the intended method:', 'Compare the result with the options')
  if (carried) return cleanLearnerText(carried)
  return cleanLearnerText(sections.originalSolutionNote)
}

function extractResultHint(sections: SectionMap, correctOptionText: string): string {
  const stepText = sections.stepByStep ?? ''
  const optionMatch = stepText.match(/select\s+([A-D]),\s+which gives\s+(.+?)(?:\.|$)/i)
  if (optionMatch) {
    return `The correct result is ${stripCurriculumCodes(optionMatch[2].trim())}.`
  }
  if (correctOptionText) {
    return `The correct answer is ${stripCurriculumCodes(correctOptionText)}.`
  }
  return ''
}

function buildGenericSolution(stem: string, sections: SectionMap, correctOptionText: string): SolutionBundle {
  const methodHint = extractMethodHint(sections)
  const workingHint = extractWorkingHint(sections)
  const resultHint = extractResultHint(sections, correctOptionText)
  const intro = cleanLearnerText(sections.howToSolve)
    || 'Work from the question itself: decide the method, do the calculation carefully, then match your result to the options.'

  const steps = [
    methodHint
      ? {
          note: 'Choose the method that matches what the question is asking.',
          work: methodHint,
        }
      : null,
    workingHint
      ? {
          note: 'Carry out the working carefully.',
          work: workingHint,
        }
      : null,
    resultHint
      ? {
          note: 'Match your result to the answer choice.',
          work: resultHint,
        }
      : null,
  ].filter((step): step is SolutionStep => !!step && !!normalizeWhitespace(step.work))

  if (steps.length === 0) {
    steps.push({
      note: 'Solve the question carefully and compare your result with the options.',
      work: resultHint || cleanLearnerText(stem),
    })
  }

  return {
    intro,
    steps,
    mistakesToAvoid: dedupeStrings([
      rewriteMistakeToAvoid('Keep separate what the question gives you and what it asks you to find.'),
      rewriteMistakeToAvoid('Finish the calculation before you compare the options.'),
      rewriteMistakeToAvoid('Check that your final answer matches the exact condition in the question.'),
    ]),
  }
}

const explanationSections = computed(() => extractSections(props.explanation))
const optionReasonEntries = computed(() => parseLetteredEntries(explanationSections.value.optionReasons))
const wrongRevealEntries = computed(() => parseLetteredEntries(explanationSections.value.wrongAnswerReveals))

const derivedCorrectOptionId = computed(() => {
  if (props.correctOptionId !== null && props.correctOptionId !== undefined) return props.correctOptionId
  return props.options.find(option => option.is_correct)?.id ?? null
})

const selectedOption = computed(() => (
  props.options.find(option => option.id === props.selectedOptionId) ?? null
))

const optionInsights = computed<OptionInsight[]>(() => props.options.map((option) => {
  const reasonText = optionReasonEntries.value[option.label] ?? ''
  const revealText = wrongRevealEntries.value[option.label] ?? ''
  const intent = parseDistractorIntent(option.distractor_intent)
  const revealEntry = parseRevealEntry(revealText)
  const comparison = extractComparison(reasonText)
  const isCorrect = option.id === derivedCorrectOptionId.value
    || option.is_correct === true
    || normalizeWhitespace(stripCurriculumCodes(option.text)) === normalizedCorrectChoiceText.value

  const comparisonLine = isCorrect
    ? `This choice matches the correct result: ${stripCurriculumCodes(option.text)}.`
    : comparison.gives && comparison.required
      ? `This choice gives ${comparison.gives}, not ${comparison.required}.`
      : props.correctOptionText
        ? `This choice does not match the correct result ${stripCurriculumCodes(props.correctOptionText)}.`
        : ''

  return {
    id: option.id,
    label: option.label,
    text: option.text,
    isCorrect,
    gives: comparison.gives,
    required: comparison.required || stripCurriculumCodes(props.correctOptionText ?? ''),
    comparisonLine,
    misstep: formatCategory(intent.misstep || extractBetween(reasonText, 'Likely misstep:') || extractBetween(reasonText, 'The likely error is:')),
    reveal: cleanDiagnosticPhrase(intent.reveal || revealEntry.reveal),
    attention: cleanDiagnosticPhrase(intent.attention || revealEntry.attention),
  }
}))

const selectedInsight = computed(() => (
  optionInsights.value.find(option => option.id === props.selectedOptionId) ?? null
))

const primaryCategory = computed(() => {
  if (props.isCorrect) return 'Right method'
  return selectedInsight.value?.misstep
    || formatCategory(props.misconceptionInfo)
    || errorLabels[props.errorType ?? '']
    || 'Common slip'
})

const statusSummary = computed(() => (
  props.isCorrect
    ? 'You got it. Keep the same method and move on.'
    : 'Here is the exact slip, the clean working, and the trap to avoid next time.'
))

const selectedChoiceText = computed(() => stripCurriculumCodes(props.selectedOptionText || selectedOption.value?.text || ''))
const correctChoiceText = computed(() => stripCurriculumCodes(props.correctOptionText || ''))
const normalizedCorrectChoiceText = computed(() => normalizeWhitespace(correctChoiceText.value))

const wrongChoiceLine = computed(() => {
  if (props.isCorrect) return ''
  if (selectedInsight.value?.comparisonLine) return selectedInsight.value.comparisonLine
  if (selectedChoiceText.value && correctChoiceText.value) {
    return `You chose ${selectedChoiceText.value}, but the correct answer is ${correctChoiceText.value}.`
  }
  return 'Your choice does not match the result the question needs.'
})

const wrongMisstepLine = computed(() => {
  if (props.isCorrect) return ''
  if (selectedInsight.value?.misstep) {
    return `Misconception: ${selectedInsight.value.misstep}.`
  }
  const fallback = formatCategory(props.misconceptionInfo)
  return fallback ? `Misconception: ${fallback}.` : ''
})

const wrongRevealLine = computed(() => {
  if (props.isCorrect) return ''
  return selectedInsight.value?.reveal
    || cleanDiagnosticPhrase(props.diagnosisSummary)
    || ''
})

const whyCorrectLine = computed(() => {
  if (!props.isCorrect) return ''
  const correctInsight = optionInsights.value.find(option => option.isCorrect)
  return correctInsight?.comparisonLine
    || cleanLearnerText(explanationSections.value.originalSolutionNote)
    || 'This choice is the one that matches the correct calculation.'
})

const solutionBundle = computed<SolutionBundle>(() => {
  const stem = props.questionStem ?? ''
  return buildFractionSolution(stem)
    || buildWholeNumberRoundingSolution(stem)
    || buildGenericSolution(stem, explanationSections.value, props.correctOptionText ?? '')
})

const solutionSteps = computed(() => solutionBundle.value.steps.filter(step => {
  const combined = `${step.note} ${step.work}`.trim()
  return normalizeWhitespace(combined).length > 0
}))

const mistakesToAvoid = computed(() => {
  if (props.isCorrect) {
    return solutionBundle.value.mistakesToAvoid
  }

  return dedupeStrings([
    ...solutionBundle.value.mistakesToAvoid,
    rewriteMistakeToAvoid(selectedInsight.value?.reveal ?? ''),
    rewriteMistakeToAvoid(selectedInsight.value?.attention ?? ''),
    rewriteMistakeToAvoid(props.recommendedAction ?? ''),
  ]).slice(0, 3)
})

const otherWrongOptions = computed(() => optionInsights.value.filter(option => !option.isCorrect))

const shouldAutoAdvance = computed(() => {
  if (props.selectedOptionId === null || props.selectedOptionId === undefined) return false
  return props.isCorrect ? props.autoAdvanceOnCorrect : props.autoAdvanceOnWrong
})

const autoAdvanceDelay = computed(() => (
  props.isCorrect ? props.autoAdvanceDelayMs : props.autoAdvanceDelayWrongMs
))

const countdownLabel = computed(() => {
  if (!shouldAutoAdvance.value || countdownMs.value <= 0) return ''
  return `Next question in ${Math.max(1, Math.ceil(countdownMs.value / 1000))}s`
})

watch(
  [() => props.selectedOptionId, () => props.isCorrect, shouldAutoAdvance, autoAdvanceDelay],
  ([selectedId, , shouldAdvance, delay]) => {
    clearAutoAdvanceTimers()
    if (selectedId === null || !shouldAdvance) return

    const waitMs = Math.max(1200, delay)
    const deadline = Date.now() + waitMs
    countdownMs.value = waitMs

    countdownTimer = window.setInterval(() => {
      countdownMs.value = Math.max(0, deadline - Date.now())
    }, 150)

    autoAdvanceTimer = window.setTimeout(() => {
      clearAutoAdvanceTimers()
      emit('next')
    }, waitMs)
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  clearAutoAdvanceTimers()
})
</script>

<template>
  <section class="qf-shell" :class="{ 'qf-shell--wrong': !isCorrect }">
    <header class="qf-top">
      <div class="qf-top-copy">
        <p class="qf-kicker">Answer Review</p>
        <h3 class="qf-title">{{ isCorrect ? 'Correct' : 'Not Quite' }}</h3>
        <p class="qf-summary">{{ statusSummary }}</p>
      </div>

      <div class="qf-meta">
        <span class="qf-chip" :class="{ 'qf-chip--wrong': !isCorrect }">{{ primaryCategory }}</span>
        <span v-if="countdownLabel" class="qf-chip qf-chip--timer">{{ countdownLabel }}</span>
      </div>
    </header>

    <section v-if="questionStem" class="qf-section">
      <p class="qf-section-label">Question</p>
      <p class="qf-question"><MathText :text="questionStem" /></p>
    </section>

    <section class="qf-section">
      <div class="qf-answer-grid" :class="{ 'qf-answer-grid--two': !isCorrect && !!correctChoiceText }">
        <div class="qf-answer-box">
          <p class="qf-section-label">Your Choice</p>
          <p class="qf-answer-text"><MathText :text="selectedChoiceText" /></p>
        </div>

        <div v-if="!isCorrect && correctChoiceText" class="qf-answer-box qf-answer-box--correct">
          <p class="qf-section-label">Right Answer</p>
          <p class="qf-answer-text"><MathText :text="correctChoiceText" /></p>
        </div>
      </div>
    </section>

    <section v-if="!isCorrect" class="qf-section">
      <p class="qf-section-label">What Went Wrong</p>
      <div class="qf-note-block qf-note-block--wrong">
        <p v-if="wrongChoiceLine" class="qf-note-line"><MathText :text="wrongChoiceLine" /></p>
        <p v-if="wrongMisstepLine" class="qf-note-line">{{ wrongMisstepLine }}</p>
        <p v-if="wrongRevealLine" class="qf-note-line">{{ wrongRevealLine }}</p>
      </div>
    </section>

    <section v-else class="qf-section">
      <p class="qf-section-label">Why This Works</p>
      <div class="qf-note-block qf-note-block--right">
        <p class="qf-note-line"><MathText :text="whyCorrectLine" /></p>
      </div>
    </section>

    <section class="qf-section">
      <p class="qf-section-label">Step-by-Step Solution</p>
      <p class="qf-intro">{{ solutionBundle.intro }}</p>

      <ol class="qf-steps">
        <li v-for="(step, index) in solutionSteps" :key="`${index}-${step.note}`" class="qf-step">
          <span class="qf-step-index">{{ index + 1 }}</span>
          <div class="qf-step-copy">
            <p class="qf-step-note">{{ step.note }}</p>
            <p v-if="step.work" class="qf-step-work"><MathText :text="step.work" /></p>
          </div>
        </li>
      </ol>
    </section>

    <section v-if="mistakesToAvoid.length" class="qf-section">
      <p class="qf-section-label">Mistakes To Avoid</p>
      <ul class="qf-bullets">
        <li v-for="item in mistakesToAvoid" :key="item" class="qf-bullet">{{ item }}</li>
      </ul>
    </section>

    <section v-if="!isCorrect && otherWrongOptions.length" class="qf-section">
      <p class="qf-section-label">How The Other Options Mislead</p>
      <div class="qf-option-list">
        <article
          v-for="option in otherWrongOptions"
          :key="option.id"
          class="qf-option"
          :class="{ 'qf-option--selected': option.id === selectedOptionId }"
        >
          <div class="qf-option-head">
            <div class="qf-option-id">
              <span class="qf-option-letter">{{ option.label }}</span>
              <span class="qf-option-choice"><MathText :text="option.text" /></span>
            </div>
            <span class="qf-option-tag">{{ option.id === selectedOptionId ? 'Your Pick' : 'Trap' }}</span>
          </div>

          <p v-if="option.comparisonLine" class="qf-option-line">
            <MathText :text="option.comparisonLine" />
          </p>
          <p v-if="option.misstep" class="qf-option-line">{{ option.misstep }}</p>
          <p v-if="option.reveal" class="qf-option-line">{{ option.reveal }}</p>
        </article>
      </div>
    </section>

    <footer class="qf-actions">
      <button type="button" class="qf-btn qf-btn--primary" @click="emit('next')">
        {{ shouldAutoAdvance ? 'Next Now' : 'Next Question' }}
      </button>
      <button
        v-if="!isCorrect && showReviewAction"
        type="button"
        class="qf-btn qf-btn--secondary"
        @click="emit('review')"
      >
        {{ reviewLabel }}
      </button>
    </footer>
  </section>
</template>

<style scoped>
.qf-shell {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 100%;
  padding: clamp(22px, 2.8vw, 32px);
  border: 1px solid color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 92%, white);
  border-radius: 8px;
  background: var(--surface, #ffffff);
  color: var(--text, #1a1612);
}

.qf-shell--wrong {
  border-color: color-mix(in srgb, var(--danger, #b91c1c) 16%, var(--border, rgba(26, 22, 18, 0.12)));
}

.qf-top {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
  flex-wrap: wrap;
}

.qf-top-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.qf-kicker,
.qf-section-label {
  margin: 0;
  font-size: 11px;
  line-height: 1.2;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--text-3, #8a8178);
}

.qf-title {
  margin: 0;
  font-size: clamp(24px, 3vw, 30px);
  line-height: 1.05;
  font-weight: 700;
  color: var(--text, #1a1612);
}

.qf-summary,
.qf-intro,
.qf-note-line,
.qf-bullet,
.qf-option-line,
.qf-question {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-2, #4e4740);
}

.qf-meta {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.qf-chip {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--success, #15803d) 20%, transparent);
  background: color-mix(in srgb, var(--success, #15803d) 8%, white);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--success, #15803d);
}

.qf-chip--wrong {
  border-color: color-mix(in srgb, var(--danger, #b91c1c) 20%, transparent);
  background: color-mix(in srgb, var(--danger, #b91c1c) 8%, white);
  color: var(--danger, #b91c1c);
}

.qf-chip--timer {
  border-color: color-mix(in srgb, var(--ink-muted, #8a8178) 22%, transparent);
  background: color-mix(in srgb, var(--paper, #faf8f5) 70%, white);
  color: var(--text-2, #4e4740);
}

.qf-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.qf-answer-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 12px;
}

.qf-answer-grid--two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.qf-answer-box,
.qf-note-block,
.qf-option {
  padding: 14px 16px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 92%, white);
  background: color-mix(in srgb, var(--paper, #faf8f5) 76%, white);
}

.qf-answer-box--correct,
.qf-note-block--right {
  background: color-mix(in srgb, var(--success, #15803d) 6%, white);
  border-color: color-mix(in srgb, var(--success, #15803d) 18%, var(--border, rgba(26, 22, 18, 0.12)));
}

.qf-note-block--wrong,
.qf-option--selected {
  background: color-mix(in srgb, var(--danger, #b91c1c) 5%, white);
  border-color: color-mix(in srgb, var(--danger, #b91c1c) 18%, var(--border, rgba(26, 22, 18, 0.12)));
}

.qf-answer-text {
  margin: 0;
  font-size: 16px;
  line-height: 1.55;
  color: var(--text, #1a1612);
}

.qf-steps {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.qf-step {
  display: grid;
  grid-template-columns: 30px minmax(0, 1fr);
  gap: 12px;
  align-items: flex-start;
}

.qf-step-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--paper, #faf8f5) 72%, white);
  border: 1px solid color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 92%, white);
  font-size: 12px;
  font-weight: 700;
  color: var(--text, #1a1612);
}

.qf-step-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.qf-step-note {
  margin: 0;
  font-size: 14px;
  line-height: 1.45;
  font-weight: 600;
  color: var(--text, #1a1612);
}

.qf-step-work {
  margin: 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--paper, #faf8f5) 82%, white);
  border: 1px solid color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 90%, white);
  font-size: 14px;
  line-height: 1.55;
  color: var(--text, #1a1612);
}

.qf-bullets {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
  padding-left: 18px;
}

.qf-option-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.qf-option-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.qf-option-id {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 0;
}

.qf-option-letter,
.qf-option-tag {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--paper, #faf8f5) 68%, white);
  border: 1px solid color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 92%, white);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-2, #4e4740);
}

.qf-option-choice {
  min-width: 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text, #1a1612);
}

.qf-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: auto;
}

.qf-btn {
  min-height: 42px;
  padding: 0 16px;
  border-radius: 8px;
  border: 1px solid transparent;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  cursor: pointer;
  transition: transform 160ms ease, background 160ms ease, border-color 160ms ease, color 160ms ease;
}

.qf-btn:hover {
  transform: translateY(-1px);
}

.qf-btn--primary {
  background: var(--text, #1a1612);
  color: #ffffff;
}

.qf-btn--secondary {
  background: transparent;
  border-color: color-mix(in srgb, var(--border, rgba(26, 22, 18, 0.12)) 92%, white);
  color: var(--text, #1a1612);
}

@media (max-width: 720px) {
  .qf-answer-grid--two {
    grid-template-columns: minmax(0, 1fr);
  }

  .qf-top {
    flex-direction: column;
  }

  .qf-meta {
    justify-content: flex-start;
  }
}
</style>
