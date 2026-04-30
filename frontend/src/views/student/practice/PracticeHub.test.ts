import assert from 'node:assert/strict'
import test from 'node:test'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const practiceHubSource = readFileSync(resolve(import.meta.dirname, 'PracticeHub.vue'), 'utf8')

test('practice search icon has explicit dimensions so it cannot expand to default svg size', () => {
  const searchIconRule = practiceHubSource.match(/\.search-icon\s*\{([\s\S]*?)\}/)

  assert.ok(searchIconRule, 'Expected PracticeHub search icon styles to exist')
  assert.match(searchIconRule[1], /width:\s*(?:1\d|2[0-4])px;/, 'Expected search icon width to be explicitly constrained')
  assert.match(searchIconRule[1], /height:\s*(?:1\d|2[0-4])px;/, 'Expected search icon height to be explicitly constrained')
})
