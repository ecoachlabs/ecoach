import assert from 'node:assert/strict'
import test from 'node:test'

import { buildLearnerTopicTree, flattenLearnerTopics } from './learnerTopics.ts'

type TopicSeed = {
  id: number
  subject_id: number
  parent_topic_id: number | null
  code: string | null
  name: string
  description: string | null
  node_type: string
  display_order: number
}

const topicSeeds: TopicSeed[] = [
  { id: 1, subject_id: 1, parent_topic_id: null, code: null, name: 'Algebra', description: null, node_type: 'strand', display_order: 1 },
  { id: 2, subject_id: 1, parent_topic_id: 1, code: null, name: 'Algebraic Expressions', description: null, node_type: 'sub_strand', display_order: 1 },
  { id: 3, subject_id: 1, parent_topic_id: 1, code: null, name: 'Variables and Equations', description: null, node_type: 'sub_strand', display_order: 2 },
  { id: 4, subject_id: 1, parent_topic_id: 2, code: 'B7.2.1.1', name: 'Algebraic Expressions', description: 'Identify simple algebraic expressions.', node_type: 'topic', display_order: 1 },
  { id: 5, subject_id: 1, parent_topic_id: 2, code: 'B8.2.1.1', name: 'Algebraic Expressions', description: 'Simplify and manipulate algebraic expressions.', node_type: 'topic', display_order: 2 },
  { id: 6, subject_id: 1, parent_topic_id: 2, code: 'B8.2.1.2', name: 'Formula Manipulation and Factorisation', description: 'Rearrange simple formulas.', node_type: 'topic', display_order: 3 },
  { id: 7, subject_id: 1, parent_topic_id: 3, code: 'B7.2.3.1', name: 'Linear Equations', description: 'Model story problems as linear equations.', node_type: 'topic', display_order: 1 },
  { id: 8, subject_id: 1, parent_topic_id: 3, code: 'B8.2.3.1', name: 'Linear Inequalities', description: 'Solve linear inequalities in one variable.', node_type: 'topic', display_order: 2 },
  { id: 9, subject_id: 1, parent_topic_id: 3, code: 'B9.2.3.1', name: 'Linear Inequalities', description: 'Represent linear inequalities on number lines.', node_type: 'topic', display_order: 3 },
  { id: 10, subject_id: 1, parent_topic_id: 1, code: null, name: 'Graphs', description: null, node_type: 'sub_strand', display_order: 3 },
  { id: 11, subject_id: 1, parent_topic_id: 10, code: 'B8.4.1.1', name: 'Linear Equations', description: 'Link linear equations to straight-line graphs.', node_type: 'topic', display_order: 1 },
]

test('consolidates repeated learner topic names within the same sub-strand and keeps ordered goals', () => {
  const strands = buildLearnerTopicTree(topicSeeds)
  const algebra = strands.find(strand => strand.name === 'Algebra')

  assert.ok(algebra)
  const expressions = algebra.subStrands.find(subStrand => subStrand.name === 'Algebraic Expressions')
  assert.ok(expressions)
  assert.equal(expressions.topics.length, 2)

  const consolidated = expressions.topics.find(topic => topic.name === 'Algebraic Expressions')
  assert.ok(consolidated)
  assert.deepEqual(consolidated.sourceTopicIds, [4, 5])
  assert.deepEqual(consolidated.goalDescriptions, [
    'Identify simple algebraic expressions.',
    'Simplify and manipulate algebraic expressions.',
  ])
  assert.equal(consolidated.description, 'Identify simple algebraic expressions.')
})

test('keeps repeated learner-facing names separate when they live in different lineage branches', () => {
  const flatTopics = flattenLearnerTopics(buildLearnerTopicTree(topicSeeds))
  const linearEquationTopics = flatTopics.filter(topic => topic.name === 'Linear Equations')

  assert.equal(linearEquationTopics.length, 2)
  assert.deepEqual(
    linearEquationTopics.map(topic => topic.sourceTopicIds),
    [[7], [11]],
  )
})
