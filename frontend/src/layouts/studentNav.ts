import {
  PhArrowFatLinesUp,
  PhBookOpenText,
  PhBooks,
  PhBrain,
  PhBug,
  PhCalendar,
  PhChartBar,
  PhChartLineDown,
  PhChartLineUp,
  PhClipboardText,
  PhClockCounterClockwise,
  PhClockCountdown,
  PhCompass,
  PhCrown,
  PhDna,
  PhDrop,
  PhFlask,
  PhGameController,
  PhGear,
  PhHouse,
  PhLightbulbFilament,
  PhLightning,
  PhMagnifyingGlass,
  PhMapTrifold,
  PhNotePencil,
  PhPersonSimpleRun,
  PhSealCheck,
  PhShuffleAngular,
  PhSquaresFour,
  PhStudent,
  PhTarget,
  PhUploadSimple,
} from '@phosphor-icons/vue'

export type NavLinkItem = {
  name: string
  to: string
  hash?: string
  icon: any
  color: string
  label: string
  match: 'exact' | 'prefix'
  activeBases?: string[]
  activePaths?: string[]
  activeHash?: string
  excludeBases?: string[]
  excludeHashes?: string[]
}

export type NavSection = {
  title: string
  items: NavLinkItem[]
}

export const quickLinkItems: NavLinkItem[] = [
  { name: 'custom-test', label: 'Custom Test', to: '/student/practice/custom-test', icon: PhClipboardText, color: '#D97706', match: 'exact' },
  { name: 'mock-history', label: 'Mock History', to: '/student/mock/history', icon: PhClockCounterClockwise, color: '#D97706', match: 'exact' },
  { name: 'gap-scan', label: 'Gap Scan', to: '/student/knowledge-gap/scan', icon: PhMagnifyingGlass, color: '#0EA5E9', match: 'exact' },
  { name: 'retry-zone', label: 'Retry Zone', to: '/student/mistakes', hash: '#retry-zone', activeHash: '#retry-zone', icon: PhTarget, color: '#EF4444', match: 'exact' },
  { name: 'revision-box', label: 'Revision Box', to: '/student/library', hash: '#revision-box', activeHash: '#revision-box', icon: PhChartLineUp, color: '#6366F1', match: 'exact' },
  { name: 'audio-glossary', label: 'Audio Glossary', to: '/student/glossary/audio', icon: PhBookOpenText, color: '#0F766E', match: 'exact' },
  { name: 'formula-lab', label: 'Formula Lab', to: '/student/glossary/formula-lab', icon: PhFlask, color: '#0EA5E9', match: 'exact' },
  { name: 'progress', label: 'Progress', to: '/student/progress', icon: PhChartBar, color: '#84CC16', match: 'exact' },
  { name: 'calendar', label: 'Calendar', to: '/student/calendar', icon: PhCalendar, color: '#22C55E', match: 'exact' },
  { name: 'onboarding', label: 'Onboarding', to: '/student/onboarding/welcome', icon: PhMapTrifold, color: '#0EA5E9', match: 'prefix', activeBases: ['/student/onboarding'] },
  { name: 'beat-yesterday', label: 'Beat Yesterday', to: '/student/beat-yesterday', icon: PhArrowFatLinesUp, color: '#EF4444', match: 'exact' },
  { name: 'elite', label: 'Elite', to: '/student/elite', icon: PhCrown, color: '#FBBF24', match: 'prefix', excludeBases: ['/student/elite/arena', '/student/elite/records'] },
  { name: 'uploads', label: 'Uploads', to: '/student/upload', icon: PhUploadSimple, color: '#6366F1', match: 'exact' },
]

export const studentNavSections: NavSection[] = [
  {
    title: 'Study',
    items: [
      { name: 'home', label: 'Home', to: '/student', icon: PhHouse, color: '#FF6B35', match: 'exact' },
      { name: 'explore', label: 'Explore', to: '/student/journey', icon: PhCompass, color: '#2563EB', match: 'prefix' },
      { name: 'teach', label: 'Teach', to: '/student/spark', icon: PhStudent, color: '#8B5CF6', match: 'prefix' },
      { name: 'curriculum', label: 'Curriculum', to: '/student/curriculum', icon: PhBookOpenText, color: '#2563EB', match: 'exact' },
      { name: 'past-questions', label: 'Past Questions', to: '/student/practice', icon: PhNotePencil, color: '#F59E0B', match: 'prefix', excludeBases: ['/student/practice/custom-test'] },
      { name: 'examples', label: 'Examples', to: '/student/glossary/compare', icon: PhSquaresFour, color: '#06B6D4', match: 'exact' },
      { name: 'library', label: 'Library', to: '/student/library', icon: PhBooks, color: '#6366F1', match: 'prefix', excludeHashes: ['#revision-box'] },
      { name: 'mastery-map', label: 'Mastery Map', to: '/student/progress/mastery-map', icon: PhMapTrifold, color: '#22C55E', match: 'exact' },
      { name: 'glossary', label: 'Glossary', to: '/student/glossary', icon: PhBookOpenText, color: '#4F46E5', match: 'prefix', excludeBases: ['/student/glossary/compare', '/student/glossary/audio', '/student/glossary/formula-lab'] },
      { name: 'traps', label: 'Traps', to: '/student/games/traps', icon: PhShuffleAngular, color: '#EF4444', match: 'exact' },
    ],
  },
  {
    title: 'Train',
    items: [
      { name: 'mock-centre', label: 'Mock Centre', to: '/student/mock', icon: PhClockCountdown, color: '#F97316', match: 'prefix', excludeBases: ['/student/mock/history'] },
      { name: 'mental', label: 'Mental', to: '/student/memory', hash: '#reviews', activeHash: '#reviews', icon: PhLightning, color: '#F59E0B', match: 'exact' },
      { name: 'marathon', label: 'Marathon', to: '/student/rise', icon: PhPersonSimpleRun, color: '#FB923C', match: 'exact' },
      { name: 'games', label: 'Games', to: '/student/games', icon: PhGameController, color: '#7C3AED', match: 'prefix', excludeBases: ['/student/games/traps'] },
      { name: 'answer-lab', label: 'Answer Lab', to: '/student/elite/arena', icon: PhFlask, color: '#14B8A6', match: 'prefix' },
      { name: 'speed-lab', label: 'Speed Lab', to: '/student/elite/records', icon: PhDrop, color: '#0EA5E9', match: 'exact' },
    ],
  },
  {
    title: 'Improve',
    items: [
      { name: 'diagnostic-dna', label: 'Diagnostic DNA', to: '/student/diagnostic', icon: PhDna, color: '#7C3AED', match: 'prefix' },
      { name: 'knowledge-gap', label: 'Knowledge Gap', to: '/student/knowledge-gap', icon: PhChartLineDown, color: '#22C55E', match: 'prefix', excludeBases: ['/student/knowledge-gap/scan'] },
      { name: 'mistake-lab', label: 'Mistake Lab', to: '/student/mistakes', icon: PhBug, color: '#F43F5E', match: 'exact', excludeHashes: ['#retry-zone'] },
      { name: 'analytics', label: 'Analytics', to: '/student/progress/analytics', icon: PhChartBar, color: '#3B82F6', match: 'exact' },
      { name: 'exam-intel', label: 'Exam Intel', to: '/student/exam-intel', icon: PhLightbulbFilament, color: '#7C3AED', match: 'prefix' },
    ],
  },
  {
    title: 'Review',
    items: [
      { name: 'history', label: 'History', to: '/student/progress/history', icon: PhClockCounterClockwise, color: '#64748B', match: 'exact' },
      { name: 'memory', label: 'Memory', to: '/student/memory', icon: PhBrain, color: '#EC4899', match: 'exact', excludeHashes: ['#reviews'] },
    ],
  },
  {
    title: 'Utilities',
    items: [
      { name: 'settings', label: 'Settings', to: '/student/settings', icon: PhGear, color: '#475569', match: 'exact' },
      { name: 'quick-links', label: 'Quick Links', to: '/student/quick-links', icon: PhSquaresFour, color: '#475569', match: 'exact' },
    ],
  },
]

export const appShortcut: NavLinkItem = {
  name: 'adeo',
  label: 'Adeo v2',
  to: '/student/settings',
  icon: PhSealCheck,
  color: '#F97316',
  match: 'exact',
}

export function isNavItemActive(item: NavLinkItem, currentPath: string, currentHash: string): boolean {
  if (item.excludeBases?.some(base => currentPath === base || currentPath.startsWith(`${base}/`))) return false
  if (item.excludeHashes?.includes(currentHash)) return false

  if (item.match === 'prefix') {
    if (item.activeHash && currentHash !== item.activeHash) return false
    const bases = [item.to, ...(item.activeBases ?? [])]
    return bases.some(base => currentPath === base || currentPath.startsWith(`${base}/`))
  }

  const paths = [item.to, ...(item.activePaths ?? [])]
  if (!paths.includes(currentPath)) return false
  if (item.activeHash) return currentHash === item.activeHash
  return true
}
