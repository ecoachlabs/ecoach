import assert from 'node:assert/strict'
import test from 'node:test'

import {
  advanceExploreTestBuffer,
  createExploreTestBuffer,
  createExploreTestBufferFromIndex,
  getExploreActiveStage,
  getExploreStandbyStage,
} from './exploreTestBuffer.ts'

test('explore buffer primes the first visible and standby questions', () => {
  const buffer = createExploreTestBuffer(['q1', 'q2', 'q3'])

  assert.equal(buffer.activeSlot, 'a')
  assert.equal(getExploreActiveStage(buffer)?.index, 0)
  assert.equal(getExploreActiveStage(buffer)?.question, 'q1')
  assert.equal(getExploreStandbyStage(buffer)?.index, 1)
  assert.equal(getExploreStandbyStage(buffer)?.question, 'q2')
})

test('explore buffer advances by swapping in the staged question and refilling the old slot', () => {
  const initial = createExploreTestBuffer(['q1', 'q2', 'q3', 'q4'])
  const advanced = advanceExploreTestBuffer(initial, ['q1', 'q2', 'q3', 'q4'])

  assert.equal(advanced.activeSlot, 'b')
  assert.equal(getExploreActiveStage(advanced)?.index, 1)
  assert.equal(getExploreActiveStage(advanced)?.question, 'q2')
  assert.equal(getExploreStandbyStage(advanced)?.index, 2)
  assert.equal(getExploreStandbyStage(advanced)?.question, 'q3')
})

test('explore buffer stays stable at the end when there is no standby question to promote', () => {
  const initial = createExploreTestBuffer(['q1', 'q2'])
  const finalBuffer = advanceExploreTestBuffer(initial, ['q1', 'q2'])
  const afterEnd = advanceExploreTestBuffer(finalBuffer, ['q1', 'q2'])

  assert.equal(finalBuffer.activeSlot, 'b')
  assert.equal(getExploreActiveStage(finalBuffer)?.index, 1)
  assert.equal(getExploreStandbyStage(finalBuffer), null)

  assert.equal(afterEnd, finalBuffer)
})

test('explore buffer can rebuild from an arbitrary current index', () => {
  const buffer = createExploreTestBufferFromIndex(['q1', 'q2', 'q3', 'q4'], 2, 'b')

  assert.equal(buffer.activeSlot, 'b')
  assert.equal(getExploreActiveStage(buffer)?.index, 2)
  assert.equal(getExploreActiveStage(buffer)?.question, 'q3')
  assert.equal(getExploreStandbyStage(buffer)?.index, 3)
  assert.equal(getExploreStandbyStage(buffer)?.question, 'q4')
})
