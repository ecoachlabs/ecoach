export type TrapMode =
  | 'difference_drill'
  | 'similarity_trap'
  | 'know_the_difference'
  | 'which_is_which'
  | 'unmask'

export type TrapRoundLike = {
  id: number
  status: string
}

export type TrapSessionLike = {
  current_round_id: number | null
  rounds: TrapRoundLike[]
}

export const trapModeMeta: Record<TrapMode, {
  label: string
  description: string
  difficulty: string
  timerSeconds: number
}> = {
  difference_drill: {
    label: 'Difference Drill',
    description: 'Sort clues into the correct concept lane.',
    difficulty: 'All levels',
    timerSeconds: 6,
  },
  similarity_trap: {
    label: 'Similarity Trap',
    description: 'Survive deceptive overlap statements.',
    difficulty: 'Advanced',
    timerSeconds: 8,
  },
  know_the_difference: {
    label: 'Know the Difference',
    description: 'Compare both concepts with slower reveals.',
    difficulty: 'Beginner',
    timerSeconds: 20,
  },
  which_is_which: {
    label: 'Which Is Which',
    description: 'Fast two-lane recognition under pressure.',
    difficulty: 'All levels',
    timerSeconds: 3,
  },
  unmask: {
    label: 'Unmask',
    description: 'Reveal clues until the concept becomes obvious.',
    difficulty: 'Intermediate',
    timerSeconds: 10,
  },
}

export function trapModeLabel(mode: string): string {
  return trapModeMeta[mode as TrapMode]?.label
    ?? mode.replace(/_/g, ' ').replace(/\b\w/g, char => char.toUpperCase())
}

export function defaultTrapTimerSeconds(mode: string): number {
  return trapModeMeta[mode as TrapMode]?.timerSeconds ?? 8
}

function isPendingRound(status: string): boolean {
  const normalized = status.toLowerCase()
  return normalized !== 'answered' && normalized !== 'completed'
}

export function findActiveTrapRound(snapshot: TrapSessionLike): TrapRoundLike | null {
  if (snapshot.current_round_id != null) {
    const matched = snapshot.rounds.find(round => round.id === snapshot.current_round_id)
    if (matched) return matched
  }

  return snapshot.rounds.find(round => isPendingRound(round.status))
    ?? snapshot.rounds[snapshot.rounds.length - 1]
    ?? null
}
