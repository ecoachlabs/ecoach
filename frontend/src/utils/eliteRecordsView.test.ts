import assert from 'node:assert/strict'
import test from 'node:test'

import { buildEliteRecordsView } from './eliteRecordsView.ts'

test('buildEliteRecordsView prefers persisted personal bests and earned badges', () => {
  const view = buildEliteRecordsView(
    {
      student_id: 7,
      subject_id: 3,
      eps_score: 6150,
      tier: 'Apex',
      precision_score: 8300,
      speed_score: 7900,
      depth_score: 7200,
      composure_score: 6800,
    },
    [
      {
        topic_id: 91,
        topic_name: 'Vectors',
        precision_score: 8400,
        speed_score: 7600,
        depth_score: 7800,
        composure_score: 7100,
        consistency_score: 7000,
        trap_resistance_score: 7400,
        domination_score: 9100,
        status: 'dominant',
      },
    ],
    [
      ['highest_eps', 6150, '2026-04-20T10:00:00Z'],
      ['highest_precision', 8300, '2026-04-19T10:00:00Z'],
      ['highest_depth', 7200, '2026-04-18T10:00:00Z'],
    ],
    [
      ['perfect_run', 'Perfect Run', '2026-04-18T10:00:00Z'],
      ['speed_authority', 'Speed Authority', '2026-04-19T10:00:00Z'],
    ],
  )

  assert.equal(view.records[0]?.category, 'EPS Score')
  assert.equal(view.records[0]?.value, '6150')
  assert.equal(view.records[1]?.category, 'Precision')
  assert.equal(view.records[1]?.value, '83%')
  assert.equal(view.records[view.records.length - 1]?.category, 'Best Topic')
  assert.equal(view.records[view.records.length - 1]?.value, 'Vectors')
  assert.ok(view.badges.find(badge => badge.name === 'Perfect Run')?.earned)
  assert.ok(view.badges.find(badge => badge.name === 'Speed Authority')?.earned)
  assert.ok(view.badges.find(badge => badge.name === 'Legend Status'))
  assert.deepEqual(
    view.titles.filter(title => title.earned).map(title => title.title),
    ['Foundation Scholar', 'Core Contender', 'Prime Performer', 'Apex Achiever'],
  )
})
