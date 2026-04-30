export type ExploreTestSlot = 'a' | 'b'

export interface ExploreTestStage<T> {
  slot: ExploreTestSlot
  index: number
  question: T
}

export interface ExploreTestBuffer<T> {
  activeSlot: ExploreTestSlot
  slotA: ExploreTestStage<T> | null
  slotB: ExploreTestStage<T> | null
}

function makeStage<T>(
  questions: readonly T[],
  slot: ExploreTestSlot,
  index: number,
): ExploreTestStage<T> | null {
  const question = questions[index]
  if (question === undefined) return null
  return { slot, index, question }
}

export function createExploreTestBufferFromIndex<T>(
  questions: readonly T[],
  startIndex: number,
  activeSlot: ExploreTestSlot = 'a',
): ExploreTestBuffer<T> {
  const standbySlot: ExploreTestSlot = activeSlot === 'a' ? 'b' : 'a'
  const activeStage = makeStage(questions, activeSlot, startIndex)
  const standbyStage = makeStage(questions, standbySlot, startIndex + 1)

  return activeSlot === 'a'
    ? {
        activeSlot: 'a',
        slotA: activeStage,
        slotB: standbyStage,
      }
    : {
        activeSlot: 'b',
        slotA: standbyStage,
        slotB: activeStage,
      }
}

export function createExploreTestBuffer<T>(questions: readonly T[]): ExploreTestBuffer<T> {
  return createExploreTestBufferFromIndex(questions, 0, 'a')
}

export function getExploreActiveStage<T>(
  buffer: ExploreTestBuffer<T>,
): ExploreTestStage<T> | null {
  return buffer.activeSlot === 'a' ? buffer.slotA : buffer.slotB
}

export function getExploreStandbyStage<T>(
  buffer: ExploreTestBuffer<T>,
): ExploreTestStage<T> | null {
  return buffer.activeSlot === 'a' ? buffer.slotB : buffer.slotA
}

export function advanceExploreTestBuffer<T>(
  buffer: ExploreTestBuffer<T>,
  questions: readonly T[],
): ExploreTestBuffer<T> {
  const standby = getExploreStandbyStage(buffer)
  if (!standby) return buffer

  const nextActiveSlot: ExploreTestSlot = standby.slot
  const refillSlot: ExploreTestSlot = nextActiveSlot === 'a' ? 'b' : 'a'
  const refillIndex = standby.index + 1
  const refillStage = makeStage(questions, refillSlot, refillIndex)

  return nextActiveSlot === 'a'
    ? {
        activeSlot: 'a',
        slotA: standby,
        slotB: refillStage,
      }
    : {
        activeSlot: 'b',
        slotA: refillStage,
        slotB: standby,
      }
}
