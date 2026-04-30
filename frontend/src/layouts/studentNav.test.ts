import assert from 'node:assert/strict'
import test from 'node:test'

import { appShortcut, isNavItemActive, quickLinkItems, studentNavSections } from './studentNav.ts'

test('student sidebar keeps quick links as a real destination page', () => {
  assert.deepEqual(
    studentNavSections.map(section => section.title),
    ['Study', 'Train', 'Improve', 'Review', 'Utilities'],
  )

  assert.deepEqual(
    studentNavSections.map(section => section.items.map(item => item.label)),
    [
      ['Home', 'Explore', 'Teach', 'Curriculum', 'Past Questions', 'Examples', 'Library', 'Mastery Map', 'Glossary', 'Traps'],
      ['Mock Centre', 'Mental', 'Marathon', 'Games', 'Answer Lab', 'Speed Lab'],
      ['Diagnostic DNA', 'Knowledge Gap', 'Mistake Lab', 'Analytics', 'Exam Intel'],
      ['History', 'Memory'],
      ['Settings', 'Quick Links'],
    ],
  )

  const quickLinks = studentNavSections
    .flatMap(section => section.items)
    .find(item => item.label === 'Quick Links')

  assert.ok(quickLinks)
  assert.equal(quickLinks.to, '/student/quick-links')
  assert.deepEqual(
    quickLinkItems.map(item => item.label),
    [
      'Custom Test',
      'Mock History',
      'Gap Scan',
      'Retry Zone',
      'Revision Box',
      'Audio Glossary',
      'Formula Lab',
      'Progress',
      'Calendar',
      'Onboarding',
      'Beat Yesterday',
      'Elite',
      'Uploads',
    ],
  )
})

test('child routes stay reachable and quick links opens its own page', () => {
  const navItems = studentNavSections.flatMap(section => section.items)
  const quickLinks = quickLinkItems
  const byLabel = (label: string) => {
    const found = navItems.find(item => item.label === label)
    assert.ok(found, `Expected nav item ${label} to exist`)
    return found
  }
  const quickByLabel = (label: string) => {
    const found = quickLinks.find(item => item.label === label)
    assert.ok(found, `Expected quick link ${label} to exist`)
    return found
  }

  assert.equal(isNavItemActive(byLabel('Quick Links'), '/student/quick-links', ''), true)
  assert.equal(isNavItemActive(byLabel('Past Questions'), '/student/practice/custom-test', ''), false)
  assert.equal(isNavItemActive(quickByLabel('Custom Test'), '/student/practice/custom-test', ''), true)
  assert.equal(isNavItemActive(byLabel('Mock Centre'), '/student/mock/history', ''), false)
  assert.equal(isNavItemActive(quickByLabel('Mock History'), '/student/mock/history', ''), true)
  assert.equal(isNavItemActive(byLabel('Diagnostic DNA'), '/student/diagnostic', ''), true)
  assert.equal(isNavItemActive(byLabel('Knowledge Gap'), '/student/diagnostic', ''), false)
  assert.equal(isNavItemActive(byLabel('Curriculum'), '/student/curriculum', ''), true)
  assert.equal(isNavItemActive(byLabel('Library'), '/student/curriculum', ''), false)
  assert.equal(isNavItemActive(byLabel('Answer Lab'), '/student/elite', ''), false)
  assert.equal(isNavItemActive(quickByLabel('Elite'), '/student/elite', ''), true)
  assert.equal(isNavItemActive(byLabel('Mental'), '/student/memory', '#reviews'), true)
  assert.equal(isNavItemActive(byLabel('Memory'), '/student/memory', ''), true)
  assert.equal(isNavItemActive(byLabel('Games'), '/student/games/traps', ''), false)
  assert.equal(isNavItemActive(byLabel('Traps'), '/student/games/traps', ''), true)
  assert.equal(isNavItemActive(byLabel('Analytics'), '/student/progress', ''), false)
  assert.equal(isNavItemActive(quickByLabel('Progress'), '/student/progress', ''), true)
})

test('footer shortcut stays available as a separate affordance', () => {
  assert.equal(appShortcut.label, 'Adeo v2')
  assert.equal(appShortcut.to, '/student/settings')
})
