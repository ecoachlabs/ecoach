/** Route name constants */
export const ROUTES = {
  PROFILES: 'profiles',
  PIN: 'pin',

  // Student
  STUDENT_HOME: 'student-home',
  PRACTICE: 'practice',
  CUSTOM_TEST: 'custom-test',
  SESSION: 'session',
  SESSION_DEBRIEF: 'session-debrief',
  DIAGNOSTIC: 'diagnostic',
  DIAGNOSTIC_SESSION: 'diagnostic-session',
  DIAGNOSTIC_REPORT: 'diagnostic-report',
  MOCK_HOME: 'mock-home',
  MOCK_SETUP: 'mock-setup',
  MOCK_HALL: 'mock-hall',
  MOCK_REVIEW: 'mock-review',
  MOCK_HISTORY: 'mock-history',
  PROGRESS: 'progress',
  MASTERY_MAP: 'mastery-map',
  ANALYTICS: 'analytics',
  JOURNEY: 'journey',
  BEAT_YESTERDAY: 'beat-yesterday',
  ELITE_HOME: 'elite-home',
  RISE: 'rise',
  SPARK: 'spark',
  KNOWLEDGE_GAP: 'knowledge-gap',
  MEMORY: 'memory',
  GLOSSARY: 'glossary',
  LIBRARY: 'library',
  EXAM_INTEL: 'exam-intel',
  GAMES: 'games',
  MISTAKES: 'mistakes',
  CALENDAR: 'calendar',
  UPLOAD: 'upload',

  // Parent
  PARENT_HOME: 'parent-home',
  PARENT_CHILD: 'parent-child',
  PARENT_ATTENTION: 'parent-attention',
  PARENT_REPORTS: 'parent-reports',

  // Admin
  ADMIN_HOME: 'admin-home',
  ADMIN_CURRICULUM: 'admin-curriculum',
  ADMIN_QUESTIONS: 'admin-questions',
  ADMIN_CONTENT: 'admin-content',
} as const

/** Question format types */
export const QUESTION_FORMATS = [
  'mcq', 'short_answer', 'true_false', 'fill_blank',
  'drag_reorder', 'matching', 'classification', 'sequencing',
  'essay', 'equation_builder', 'diagram_label', 'comparison_table',
  'step_by_step', 'canvas_draw', 'first_step',
] as const

/** Session types */
export const SESSION_TYPES = [
  'practice', 'custom_test', 'diagnostic', 'mock',
  'beat_yesterday', 'elite', 'rise', 'spark',
  'knowledge_gap', 'memory', 'journey',
] as const

/** Emotional modes */
export const EMOTIONAL_MODES = [
  'normal', 'recovery', 'pressure', 'elite',
  'game', 'celebration', 'focus',
] as const
