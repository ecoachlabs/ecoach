import { sanitizeLearnerSnippet, stripCurriculumCodes } from '@/utils/learnerCopy'
import { measureActivePerfSync } from '@/utils/perfTrace'

export interface SolutionStep {
  note: string
  work: string
}

export interface SolutionBundle {
  intro: string
  steps: SolutionStep[]
  mistakesToAvoid: string[]
}

interface SectionMap {
  howToSolve?: string
  stepByStep?: string
  optionReasons?: string
  wrongAnswerReveals?: string
  originalSolutionNote?: string
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

export function buildQuestionSolution(
  questionStem: string | null | undefined,
  explanation: string | null | undefined,
  correctOptionText: string | null | undefined,
): SolutionBundle {
  return measureActivePerfSync('questionSolution.build', () => {
    const stem = questionStem ?? ''
    const sections = extractSections(explanation)
    return buildFractionSolution(stem)
      || buildWholeNumberRoundingSolution(stem)
      || buildGenericSolution(stem, sections, correctOptionText ?? '')
  }, {
    stemChars: (questionStem ?? '').length,
    explanationChars: (explanation ?? '').length,
  })
}
