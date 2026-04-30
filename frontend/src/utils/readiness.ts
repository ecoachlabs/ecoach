export type NormalizedReadinessBand =
  | 'exam_ready'
  | 'near_ready'
  | 'progressing'
  | 'strong'
  | 'building'
  | 'developing'
  | 'fragile'
  | 'at_risk'
  | 'weak'
  | 'not_ready'
  | 'critical'
  | 'unknown'

function humanizeFallbackBand(band: string | null | undefined): string {
  const raw = band?.trim()
  if (!raw) return 'Unknown'
  return raw
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .replace(/\b\w/g, char => char.toUpperCase())
}

export function normalizeReadinessBand(band: string | null | undefined): NormalizedReadinessBand {
  const normalized = band?.trim().toLowerCase().replace(/[\s-]+/g, '_') ?? 'unknown'
  switch (normalized) {
    case 'exam_ready':
    case 'near_ready':
    case 'progressing':
    case 'strong':
    case 'building':
    case 'developing':
    case 'fragile':
    case 'at_risk':
    case 'weak':
    case 'not_ready':
    case 'critical':
      return normalized
    default:
      return 'unknown'
  }
}

export function getReadinessLabel(band: string | null | undefined): string {
  switch (normalizeReadinessBand(band)) {
    case 'exam_ready':
      return 'Exam Ready'
    case 'near_ready':
      return 'Near Ready'
    case 'progressing':
      return 'Progressing'
    case 'strong':
      return 'Strong'
    case 'building':
      return 'Building'
    case 'developing':
      return 'Developing'
    case 'fragile':
      return 'Fragile'
    case 'at_risk':
      return 'At Risk'
    case 'weak':
      return 'Needs Work'
    case 'not_ready':
      return 'Not Ready'
    case 'critical':
      return 'Critical'
    default:
      return humanizeFallbackBand(band)
  }
}

export function getReadinessColor(band: string | null | undefined): string {
  switch (normalizeReadinessBand(band)) {
    case 'exam_ready':
    case 'near_ready':
    case 'progressing':
    case 'strong':
      return 'var(--accent)'
    case 'building':
    case 'developing':
      return 'var(--gold)'
    case 'critical':
      return 'var(--danger)'
    case 'fragile':
    case 'at_risk':
    case 'weak':
    case 'not_ready':
      return 'var(--warm)'
    default:
      return 'var(--ink-muted)'
  }
}

export function getReadinessProgressColor(
  band: string | null | undefined,
): 'success' | 'gold' | 'danger' {
  switch (normalizeReadinessBand(band)) {
    case 'exam_ready':
    case 'near_ready':
    case 'progressing':
    case 'strong':
      return 'success'
    case 'building':
    case 'developing':
      return 'gold'
    default:
      return 'danger'
  }
}

export function isPositiveReadinessBand(band: string | null | undefined): boolean {
  switch (normalizeReadinessBand(band)) {
    case 'exam_ready':
    case 'near_ready':
    case 'progressing':
    case 'strong':
      return true
    default:
      return false
  }
}
