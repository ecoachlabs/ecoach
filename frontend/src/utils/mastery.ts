export type MasteryStateKey = 'unseen' | 'exposed' | 'emerging' | 'partial' | 'fragile' | 'stable' | 'robust' | 'exam_ready'

export interface MasteryDisplay {
  label: string
  color: string
  bg: string
  icon: string
  description: string
}

export const masteryStates: Record<MasteryStateKey, MasteryDisplay> = {
  unseen:    { label: 'Not Started',   color: '#78716c', bg: '#f5f5f4', icon: '○',  description: 'Topic not yet encountered' },
  exposed:   { label: 'Exposed',       color: '#78716c', bg: '#fafaf9', icon: '◔',  description: 'Seen but not practiced' },
  emerging:  { label: 'Emerging',      color: '#b45309', bg: '#fef3c7', icon: '◑',  description: 'Beginning to understand' },
  partial:   { label: 'Partial',       color: '#a16207', bg: '#fef9c3', icon: '◕',  description: 'Inconsistent understanding' },
  fragile:   { label: 'Fragile',       color: '#c2410c', bg: '#fff7ed', icon: '◉',  description: 'Knows it but breaks under pressure' },
  stable:    { label: 'Stable',        color: '#15803d', bg: '#dcfce7', icon: '●',  description: 'Consistent correct performance' },
  robust:    { label: 'Robust',        color: '#059669', bg: '#d1fae5', icon: '★',  description: 'Strong and retained over time' },
  exam_ready:{ label: 'Exam Ready',    color: '#0d9488', bg: '#ccfbf1', icon: '✦',  description: 'Ready for exam conditions' },
}

export function getMasteryDisplay(state: string): MasteryDisplay {
  return masteryStates[state as MasteryStateKey] ?? masteryStates.unseen
}

export function getMasteryColor(state: string): string {
  return getMasteryDisplay(state).color
}

export function getMasteryLabel(state: string): string {
  return getMasteryDisplay(state).label
}

/** Get readiness band color */
export function getReadinessColor(band: string): string {
  switch (band) {
    case 'strong': return '#15803d'
    case 'developing': return '#b45309'
    case 'weak': return '#c2410c'
    case 'critical': return '#b91c1c'
    default: return '#78716c'
  }
}
