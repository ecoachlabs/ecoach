const CURRICULUM_CODE_PATTERN = /\b[A-Z]{1,3}\s*\d+(?:\s*\.\s*\d+)+\b/g
const CURRICULUM_CODE_FRAGMENT_PATTERN = /\b[A-Z]{1,3}\s*\d+(?:[.\-\\()]+\d+){2,}\b/g
const NUMERIC_CURRICULUM_CODE_PATTERN = /\b\d+(?:\s*\.\s*\d+){2,}\b/g
const NUMERIC_HYPHEN_CODE_PATTERN = /\b\d+(?:\s*\.\s*\d+)+(?:\s*[−-]\s*\d+)+\b/g
const MISCONCEPTION_CODE_PATTERN = /\(\s*\.?\s*\d+(?:\s*\.\s*\d+)*(?:\s*[−-]\s*\d+)+\s*\)/g

function tidyWhitespace(value: string): string {
  return value
    .replace(/\s+([,.;:!?])/g, '$1')
    .replace(/\(\s+/g, '(')
    .replace(/\s+\)/g, ')')
    .replace(/\[\s+/g, '[')
    .replace(/\s+\]/g, ']')
    .replace(/\s{2,}/g, ' ')
    .trim()
}

function splitChunks(value: string): string[] {
  return value
    .split(/\n+/)
    .flatMap(line => line.split(/(?<=[.!?])\s+(?=[A-Z0-9])/))
    .flatMap(line => line.split(/\s*;\s+(?=[A-Z0-9])/))
    .map(line => line.trim())
    .filter(Boolean)
}

function isExplicitSyllabusMeta(value: string): boolean {
  const lower = value.toLowerCase()
  return (
    lower.includes('curriculum link') ||
    lower.includes('source anchors') ||
    lower.includes('skill focus') ||
    lower.includes('original solution note') ||
    lower.startsWith('objective:') ||
    lower.startsWith('objective ')
  )
}

function stripSyllabusScaffolding(value: string): string {
  return value
    .replace(
      /(?:^|\n{2,})(?:Curriculum link|Objective|Skill focus|Source anchors|Original solution note):[\s\S]*?(?=\n{2,}[A-Z][A-Za-z -]+:|$)/gi,
      ' ',
    )
    .replace(
      /Read the question as (?:an?\s+)?[^.]*?\bitem and restate the target:\s*/gi,
      'Restate the target: ',
    )
    .replace(
      /Start by reading the command in the stem, identifying the given information, and deciding which .*? method is needed before looking at the options\.?/gi,
      ' ',
    )
    .replace(
      /Read the question and restate the target:\s*Demonstrate understanding of .*?(?:\.|$)/gi,
      ' ',
    )
    .replace(
      /This tells you which rule, formula, representation, or calculation path to use\.?/gi,
      ' ',
    )
    .replace(/Identify the relevant skill:\s*[^.]*\.\s*/gi, '')
    .replace(/\s*while working on\s+[^.]*?\btasks?\b\.?/gi, '')
    .replace(/\btied to source exemplars\b\.?/gi, '')
    .replace(/\bsource exemplars\b/gi, 'worked examples')
}

export function stripCurriculumCodes(value: string | null | undefined): string {
  if (!value) return ''
  const cleaned = stripSyllabusScaffolding(value)
    .replace(CURRICULUM_CODE_PATTERN, ' ')
    .replace(CURRICULUM_CODE_FRAGMENT_PATTERN, ' ')
    .replace(NUMERIC_CURRICULUM_CODE_PATTERN, ' ')
    .replace(NUMERIC_HYPHEN_CODE_PATTERN, ' ')
    .replace(MISCONCEPTION_CODE_PATTERN, ' ')
    .replace(/\bas\s+(?:an?\s+)?item\b/gi, '')
    .replace(/\bfor\s+(?:an?\s+)?item\b/gi, '')
    .replace(/\bthis item\b/gi, 'this question')
    .replace(/\bthe item\b/gi, 'the question')
    .replace(/\bitem\b/gi, 'question')
    .replace(/\(\s*\)/g, ' ')
    .replace(/:\s*\./g, '.')
    .replace(/\s{2,}/g, ' ')

  return tidyWhitespace(cleaned)
}

export function sanitizeLearnerSnippet(
  value: string | null | undefined,
  options: { dropSyllabusMeta?: boolean } = {},
): string {
  if (!value) return ''

  const parts = splitChunks(value)
    .filter(part => !options.dropSyllabusMeta || !isExplicitSyllabusMeta(part))
    .map(part => stripCurriculumCodes(part))
    .filter(Boolean)

  if (parts.length === 0) return stripCurriculumCodes(value)
  return tidyWhitespace(parts.join(' '))
}
