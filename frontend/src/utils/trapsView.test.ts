import assert from 'node:assert/strict'
import test from 'node:test'

import {
  defaultTrapTimerSeconds,
  findActiveTrapRound,
  trapModeLabel,
} from './trapsView.ts'

test('findActiveTrapRound prefers the backend current round id', () => {
  const round = findActiveTrapRound({
    current_round_id: 22,
    rounds: [
      { id: 20, status: 'answered' },
      { id: 21, status: 'answered' },
      { id: 22, status: 'active' },
    ],
  })

  assert.equal(round?.id, 22)
})

test('findActiveTrapRound falls back to the first unanswered round', () => {
  const round = findActiveTrapRound({
    current_round_id: null,
    rounds: [
      { id: 31, status: 'answered' },
      { id: 32, status: 'pending' },
      { id: 33, status: 'pending' },
    ],
  })

  assert.equal(round?.id, 32)
})

test('trap mode labels and timer defaults match backend behavior', () => {
  assert.equal(trapModeLabel('difference_drill'), 'Difference Drill')
  assert.equal(trapModeLabel('which_is_which'), 'Which Is Which')
  assert.equal(defaultTrapTimerSeconds('difference_drill'), 6)
  assert.equal(defaultTrapTimerSeconds('which_is_which'), 3)
  assert.equal(defaultTrapTimerSeconds('unmask'), 10)
})
