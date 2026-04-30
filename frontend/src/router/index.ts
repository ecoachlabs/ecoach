import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

// Auth views
import ProfileSwitcher from '@/views/auth/ProfileSwitcher.vue'
import PinEntry from '@/views/auth/PinEntry.vue'

// Layouts
import StudentLayout from '@/layouts/StudentLayout.vue'
import ParentLayout from '@/layouts/ParentLayout.vue'
import AdminLayout from '@/layouts/AdminLayout.vue'

// Student views
// v1 and v2 are commented out — v3 is now the default home.
// import CoachHub from '@/views/student/CoachHub.vue'
// import CoachHubV2 from '@/views/student/CoachHubV2.vue'
import CoachHub from '@/views/student/CoachHub.vue'
import CoachHubV3 from '@/views/student/CoachHubV3.vue'

// Parent views
import ParentHome from '@/views/parent/ParentHome.vue'

// Admin views
import AdminHome from '@/views/admin/AdminHome.vue'

function resolveCoachRoute(path: string): string {
  const normalized = path.replace(/\/+$/, '') || '/coach'

  const directRedirects: Record<string, string> = {
    '/coach': '/student/coach',
    '/coach/onboarding': '/student/onboarding/welcome',
    '/coach/onboarding/subjects': '/student/onboarding/subjects',
    '/coach/content': '/student/onboarding/content-packs',
    '/coach/diagnostic': '/student/diagnostic',
    '/coach/plan': '/student/journey',
    '/coach/plan/refresh': '/student/journey',
    '/coach/repair': '/student/knowledge-gap',
    '/coach/mission/today': '/student/journey',
  }

  if (directRedirects[normalized]) {
    return directRedirects[normalized]
  }

  if (/^\/coach\/mission\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  if (/^\/coach\/missions\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  if (/^\/coach\/review\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  return '/student/coach'
}

function resolveRoleHome(): string {
  const authStore = useAuthStoreForGuard()

  if (!authStore.isAuthenticated) {
    return '/'
  }

  if (authStore.role === 'parent') {
    return '/parent'
  }

  if (authStore.role === 'admin') {
    return '/admin'
  }

  return '/student'
}

const routes: RouteRecordRaw[] = [
  // === AUTH ===
  {
    path: '/',
    name: 'profiles',
    component: ProfileSwitcher,
  },
  {
    path: '/pin/:accountId',
    name: 'pin',
    component: PinEntry,
    props: true,
  },
  {
    path: '/coach/:pathMatch(.*)*',
    redirect: to => resolveCoachRoute(to.path),
  },

  // === STUDENT PORTAL ===
  {
    path: '/student',
    component: StudentLayout,
    meta: { role: 'student' },
    children: [
      { path: '', name: 'student-home', component: CoachHub },
      // v1 and v2 disabled — v3 is the main home.
      // { path: '', name: 'student-home', component: CoachHub },
      // { path: 'v2', name: 'student-home-v2', component: CoachHubV2 },
      { path: 'coach', name: 'coach-hub', component: CoachHubV3 },
      { path: 'v3', redirect: { name: 'coach-hub' } },
      // Onboarding
      { path: 'onboarding/welcome', name: 'onboarding-welcome', component: () => import('@/views/student/onboarding/Welcome.vue') },
      { path: 'onboarding/subjects', name: 'onboarding-subjects', component: () => import('@/views/student/onboarding/Subjects.vue') },
      { path: 'onboarding/content-packs', name: 'onboarding-packs', component: () => import('@/views/student/onboarding/ContentPacks.vue') },
      { path: 'onboarding/diagnostic', name: 'onboarding-diagnostic', component: () => import('@/views/student/onboarding/Diagnostic.vue') },
      // Past Questions — the nav item labelled "Past Questions" lands here.
      // PracticeHub.vue is kept on disk for future reuse at a different route.
      { path: 'practice', name: 'practice', component: () => import('@/views/student/practice/PastQuestions.vue') },
      { path: 'practice/custom-test', name: 'custom-test', component: () => import('@/views/student/practice/CustomTest.vue') },
      // Sessions
      { path: 'session/:id', name: 'session', component: () => import('@/views/student/session/SessionView.vue'), props: true },
      { path: 'session/:id/debrief', name: 'session-debrief', component: () => import('@/views/student/session/SessionDebrief.vue'), props: true },
      // Diagnostic
      { path: 'diagnostic', name: 'diagnostic', component: () => import('@/views/student/diagnostic/DiagnosticLauncher.vue') },
      { path: 'diagnostic/:id', name: 'diagnostic-session', component: () => import('@/views/student/diagnostic/DiagnosticSession.vue'), props: true },
      { path: 'diagnostic/:id/report', name: 'diagnostic-report', component: () => import('@/views/student/diagnostic/DiagnosticReport.vue'), props: true },
      // Mock Centre
      { path: 'mock', name: 'mock-home', component: () => import('@/views/student/mock/MockHome.vue') },
      { path: 'mock/setup', name: 'mock-setup', component: () => import('@/views/student/mock/MockSetup.vue') },
      { path: 'mock/hall/:id', name: 'mock-hall', component: () => import('@/views/student/mock/MockHall.vue'), props: true },
      { path: 'mock/review/:id', name: 'mock-review', component: () => import('@/views/student/mock/MockReview.vue'), props: true },
      { path: 'mock/history', name: 'mock-history', component: () => import('@/views/student/mock/MockHistory.vue') },
      // Progress
      { path: 'progress', name: 'progress', component: () => import('@/views/student/progress/ProgressOverview.vue') },
      { path: 'progress/mastery-map', name: 'mastery-map', component: () => import('@/views/student/progress/MasteryMap.vue') },
      { path: 'progress/analytics', name: 'analytics', component: () => import('@/views/student/progress/Analytics.vue') },
      { path: 'progress/history', name: 'history', component: () => import('@/views/student/progress/History.vue') },
      // Learning Modes
      { path: 'journey', name: 'journey', component: () => import('@/views/student/journey/JourneyHome.vue') },
      { path: 'journey/station/:id', name: 'journey-station', component: () => import('@/views/student/journey/JourneyStation.vue'), props: true },
      { path: 'beat-yesterday', name: 'beat-yesterday', component: () => import('@/views/student/beat-yesterday/BeatYesterdayHome.vue') },
      { path: 'elite', name: 'elite-home', component: () => import('@/views/student/elite/EliteHome.vue') },
      { path: 'elite/arena', name: 'elite-arena', component: () => import('@/views/student/elite/EliteArena.vue') },
      { path: 'elite/session/:id', name: 'elite-session', component: () => import('@/views/student/elite/EliteSession.vue'), props: true },
      { path: 'elite/records', name: 'elite-records', component: () => import('@/views/student/elite/EliteRecords.vue') },
      { path: 'elite/insights', name: 'elite-insights', component: () => import('@/views/student/elite/EliteInsights.vue') },
      { path: 'rise', name: 'rise', component: () => import('@/views/student/rise/RiseHome.vue') },
      { path: 'spark', name: 'spark', component: () => import('@/views/student/spark/SparkHome.vue') },
      { path: 'knowledge-gap', name: 'knowledge-gap', component: () => import('@/views/student/knowledge-gap/GapHome.vue') },
      { path: 'knowledge-gap/scan', name: 'gap-scan', component: () => import('@/views/student/knowledge-gap/GapScan.vue') },
      { path: 'memory', name: 'memory', component: () => import('@/views/student/memory/MemoryHome.vue') },
      // Content
      { path: 'curriculum', name: 'student-curriculum', component: () => import('@/views/student/curriculum/CurriculumHome.vue') },
      { path: 'quick-links', name: 'quick-links', component: () => import('@/views/student/QuickLinksHome.vue') },
      { path: 'glossary', name: 'glossary', component: () => import('@/views/student/glossary/GlossaryHome.vue') },
      { path: 'glossary/entry/:id', name: 'glossary-entry', component: () => import('@/views/student/glossary/GlossaryEntry.vue'), props: true },
      { path: 'glossary/audio', name: 'glossary-audio', component: () => import('@/views/student/glossary/GlossaryAudio.vue') },
      { path: 'glossary/formula-lab', name: 'glossary-formula-lab', component: () => import('@/views/student/glossary/FormulaLabView.vue') },
      { path: 'glossary/compare', name: 'glossary-compare', component: () => import('@/views/student/glossary/GlossaryCompareView.vue') },
      { path: 'library', name: 'library', component: () => import('@/views/student/library/LibraryHome.vue') },
      { path: 'teach/:topicId', name: 'teach', component: () => import('@/views/student/teach/TeachMode.vue'), props: true },
      // Exam Intel
      { path: 'exam-intel', name: 'exam-intel', component: () => import('@/views/student/exam-intel/ExamIntelHome.vue') },
      { path: 'exam-intel/family/:id', name: 'exam-family', component: () => import('@/views/student/exam-intel/FamilyView.vue'), props: true },
      // Games
      { path: 'games', name: 'games', component: () => import('@/views/student/games/GamesHub.vue') },
      { path: 'games/mindstack', name: 'mindstack', component: () => import('@/views/student/games/MindStackGame.vue') },
      { path: 'games/tugofwar', name: 'tugofwar', component: () => import('@/views/student/games/TugOfWarGame.vue') },
      { path: 'games/traps', name: 'traps', component: () => import('@/views/student/games/TrapsHub.vue') },
      // Other
      { path: 'mistakes', name: 'mistakes', component: () => import('@/views/student/mistakes/MistakeLab.vue') },
      { path: 'calendar', name: 'calendar', component: () => import('@/views/student/calendar/AcademicCalendar.vue') },
      { path: 'upload', name: 'upload', component: () => import('@/views/student/upload/UploadWizard.vue') },
      { path: 'settings', name: 'student-settings', component: () => import('@/views/student/Settings.vue') },
    ],
  },

  // === PARENT PORTAL ===
  {
    path: '/parent',
    component: ParentLayout,
    meta: { role: 'parent' },
    children: [
      { path: '', name: 'parent-home', component: ParentHome },
      { path: 'household', name: 'parent-household', component: () => import('@/views/parent/Household.vue') },
      { path: 'children', name: 'parent-children', component: () => import('@/views/parent/Children.vue') },
      { path: 'child/:id', name: 'parent-child', component: () => import('@/views/parent/ChildDashboard.vue'), props: true },
      { path: 'attention', name: 'parent-attention', component: () => import('@/views/parent/AttentionNeeded.vue') },
      { path: 'performance', name: 'parent-performance', component: () => import('@/views/parent/Performance.vue') },
      { path: 'reports', name: 'parent-reports', component: () => import('@/views/parent/Reports.vue') },
      { path: 'intervention', name: 'parent-intervention', component: () => import('@/views/parent/InterventionCenter.vue') },
      { path: 'curriculum', name: 'parent-curriculum', component: () => import('@/views/parent/Curriculum.vue') },
      { path: 'concierge', name: 'parent-concierge', component: () => import('@/views/parent/Concierge.vue') },
      { path: 'settings', name: 'parent-settings', component: () => import('@/views/parent/Settings.vue') },
    ],
  },

  // === ADMIN PORTAL ===
  {
    path: '/admin',
    component: AdminLayout,
    meta: { role: 'admin' },
    children: [
      { path: '', name: 'admin-home', component: AdminHome },
      { path: 'content-editor', name: 'admin-content-editor', component: () => import('@/views/admin/content-editor/ContentEditorHome.vue') },
      { path: 'content-editor/question/:id?', name: 'admin-content-editor-question', component: () => import('@/views/admin/content-editor/ContentEditorHome.vue'), props: true },
      { path: 'remote-updates', name: 'admin-remote-updates', component: () => import('@/views/admin/remote/RemoteUpdates.vue') },
      { path: 'curriculum', name: 'admin-curriculum', component: () => import('@/views/admin/curriculum/CurriculumHome.vue') },
      { path: 'curriculum/upload', name: 'admin-curriculum-upload', component: () => import('@/views/admin/curriculum/CurriculumUpload.vue') },
      { path: 'curriculum/review', name: 'admin-curriculum-review', component: () => import('@/views/admin/curriculum/CurriculumReview.vue') },
      { path: 'curriculum/editor', name: 'admin-curriculum-editor', component: () => import('@/views/admin/curriculum/CurriculumEditor.vue') },
      { path: 'questions', name: 'admin-questions', component: () => import('@/views/admin/questions/QuestionsHome.vue') },
      { path: 'questions/author', redirect: to => ({ path: '/admin/content-editor', query: { ...to.query, type: 'question' } }) },
      { path: 'questions/review', name: 'admin-question-review', component: () => import('@/views/admin/questions/QuestionReview.vue') },
      { path: 'past-papers', name: 'admin-past-papers', component: () => import('@/views/admin/past-papers/PastPaperList.vue') },
      { path: 'past-papers/:id', name: 'admin-past-paper-editor', component: () => import('@/views/admin/past-papers/PastPaperAuthor.vue'), props: true },
      { path: 'content', name: 'admin-content', component: () => import('@/views/admin/content/ContentPipeline.vue') },
      { path: 'content/coverage', name: 'admin-coverage', component: () => import('@/views/admin/content/CoverageHeatmap.vue') },
      { path: 'students', name: 'admin-students', component: () => import('@/views/admin/students/StudentMonitor.vue') },
      { path: 'students/:id', name: 'admin-student-detail', component: () => import('@/views/admin/students/StudentDetail.vue'), props: true },
      { path: 'packs', name: 'admin-packs', component: () => import('@/views/admin/PackManager.vue') },
      { path: 'users', name: 'admin-users', component: () => import('@/views/admin/UserManager.vue') },
      { path: 'quality', name: 'admin-quality', component: () => import('@/views/admin/QualityDashboard.vue') },
      { path: 'settings', name: 'admin-settings', component: () => import('@/views/admin/Settings.vue') },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: () => resolveRoleHome(),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Route guard: protect portals by role
router.beforeEach((to, _from, next) => {
  // Auth routes are always accessible
  if (to.path === '/' || to.path.startsWith('/pin')) {
    return next()
  }

  // Check auth
  const authStore = useAuthStoreForGuard()
  if (!authStore.isAuthenticated) {
    return next('/')
  }

  // Check role matches portal
  const role = authStore.role
  if (to.path.startsWith('/student') && role !== 'student') {
    return next(role === 'parent' ? '/parent' : role === 'admin' ? '/admin' : '/')
  }
  if (to.path.startsWith('/parent') && role !== 'parent') {
    return next(role === 'student' ? '/student' : role === 'admin' ? '/admin' : '/')
  }
  if (to.path.startsWith('/admin') && role !== 'admin') {
    return next(role === 'student' ? '/student' : role === 'parent' ? '/parent' : '/')
  }

  next()
})

// Auth store access for guards
import { useAuthStore as _useAuthStore } from '@/stores/auth'
function useAuthStoreForGuard() {
  try {
    return _useAuthStore()
  } catch {
    return { isAuthenticated: false, role: null }
  }
}

export default router
