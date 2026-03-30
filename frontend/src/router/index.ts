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
import CoachHub from '@/views/student/CoachHub.vue'

// Parent views
import ParentHome from '@/views/parent/ParentHome.vue'

// Admin views
import AdminHome from '@/views/admin/AdminHome.vue'

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

  // === STUDENT PORTAL ===
  {
    path: '/student',
    component: StudentLayout,
    meta: { role: 'student' },
    children: [
      { path: '', name: 'student-home', component: CoachHub },
      // Onboarding
      { path: 'onboarding/welcome', name: 'onboarding-welcome', component: () => import('@/views/student/onboarding/Welcome.vue') },
      { path: 'onboarding/subjects', name: 'onboarding-subjects', component: () => import('@/views/student/onboarding/Subjects.vue') },
      { path: 'onboarding/content-packs', name: 'onboarding-packs', component: () => import('@/views/student/onboarding/ContentPacks.vue') },
      { path: 'onboarding/diagnostic', name: 'onboarding-diagnostic', component: () => import('@/views/student/onboarding/Diagnostic.vue') },
      // Practice
      { path: 'practice', name: 'practice', component: () => import('@/views/student/practice/PracticeHub.vue') },
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
      { path: 'glossary', name: 'glossary', component: () => import('@/views/student/glossary/GlossaryHome.vue') },
      { path: 'glossary/entry/:id', name: 'glossary-entry', component: () => import('@/views/student/glossary/GlossaryEntry.vue'), props: true },
      { path: 'glossary/audio', name: 'glossary-audio', component: () => import('@/views/student/glossary/GlossaryAudio.vue') },
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
      { path: 'child/:id', name: 'parent-child', component: () => import('@/views/parent/ChildDashboard.vue'), props: true },
      { path: 'attention', name: 'parent-attention', component: () => import('@/views/parent/AttentionNeeded.vue') },
      { path: 'performance', name: 'parent-performance', component: () => import('@/views/parent/Performance.vue') },
      { path: 'reports', name: 'parent-reports', component: () => import('@/views/parent/Reports.vue') },
      { path: 'intervention', name: 'parent-intervention', component: () => import('@/views/parent/InterventionCenter.vue') },
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
      { path: 'curriculum', name: 'admin-curriculum', component: () => import('@/views/admin/curriculum/CurriculumHome.vue') },
      { path: 'curriculum/upload', name: 'admin-curriculum-upload', component: () => import('@/views/admin/curriculum/CurriculumUpload.vue') },
      { path: 'curriculum/review', name: 'admin-curriculum-review', component: () => import('@/views/admin/curriculum/CurriculumReview.vue') },
      { path: 'curriculum/editor', name: 'admin-curriculum-editor', component: () => import('@/views/admin/curriculum/CurriculumEditor.vue') },
      { path: 'questions', name: 'admin-questions', component: () => import('@/views/admin/questions/QuestionsHome.vue') },
      { path: 'questions/author', name: 'admin-question-author', component: () => import('@/views/admin/questions/QuestionAuthor.vue') },
      { path: 'questions/review', name: 'admin-question-review', component: () => import('@/views/admin/questions/QuestionReview.vue') },
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
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Route guard: redirect to correct portal based on role
router.beforeEach((to, _from, next) => {
  // Auth routes are always accessible
  if (to.path === '/' || to.path.startsWith('/pin')) {
    return next()
  }

  // For now, allow all navigation -- role checking will be added
  // when auth store is wired to the backend
  next()
})

export default router
