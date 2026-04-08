<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  PhArrowFatLinesUp,
  PhBook,
  PhBooks,
  PhBrain,
  PhBug,
  PhCalendar,
  PhChartBar,
  PhClockCountdown,
  PhCrown,
  PhGameController,
  PhGear,
  PhHouse,
  PhLightbulbFilament,
  PhMagnifyingGlass,
  PhPencilSimple,
  PhRoadHorizon,
  PhRocketLaunch,
  PhSignOut,
  PhTarget,
} from '@phosphor-icons/vue'

type NavItem = {
  name: string
  to: string
  hash?: string
  icon: any
  color: string
  label: string
  match: 'exact' | 'prefix'
  activeBase?: string
  activeHash?: string
  excludeBases?: string[]
  excludeHashes?: string[]
}

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()
const collapsed = ref(false)

onMounted(() => {
  ui.setTheme('student')
})

const navGroups: Array<{ label: string; items: NavItem[] }> = [
  {
    label: 'Start',
    items: [
      { name: 'student-home', to: '/student', icon: PhHouse, color: '#FF6B35', label: 'Coach Hub', match: 'exact' },
      { name: 'onboarding', to: '/student/onboarding/welcome', icon: PhRoadHorizon, color: '#0EA5E9', label: 'Onboarding', match: 'prefix', activeBase: '/student/onboarding' },
      { name: 'practice', to: '/student/practice', icon: PhPencilSimple, color: '#06B6D4', label: 'Practice Hub', match: 'exact' },
      { name: 'custom-test', to: '/student/practice/custom-test', icon: PhPencilSimple, color: '#0891B2', label: 'Custom Test', match: 'exact' },
      { name: 'diagnostic', to: '/student/diagnostic', icon: PhMagnifyingGlass, color: '#A855F7', label: 'Diagnostic', match: 'prefix' },
      { name: 'mock-home', to: '/student/mock', icon: PhClockCountdown, color: '#F59E0B', label: 'Mock Centre', match: 'prefix', activeBase: '/student/mock', excludeBases: ['/student/mock/history'] },
      { name: 'mock-history', to: '/student/mock/history', icon: PhCalendar, color: '#D97706', label: 'Mock History', match: 'exact' },
    ],
  },
  {
    label: 'Modes',
    items: [
      { name: 'journey', to: '/student/journey', icon: PhRoadHorizon, color: '#22C55E', label: 'Journey', match: 'prefix' },
      { name: 'beat-yesterday', to: '/student/beat-yesterday', icon: PhArrowFatLinesUp, color: '#EF4444', label: 'Beat Yesterday', match: 'exact' },
      { name: 'rise', to: '/student/rise', icon: PhRocketLaunch, color: '#3B82F6', label: 'Rise', match: 'exact' },
      { name: 'spark', to: '/student/spark', icon: PhLightbulbFilament, color: '#F97316', label: 'Spark', match: 'exact' },
      { name: 'elite-home', to: '/student/elite', icon: PhCrown, color: '#FBBF24', label: 'Elite', match: 'prefix', activeBase: '/student/elite', excludeBases: ['/student/elite/arena', '/student/elite/records', '/student/elite/insights'] },
      { name: 'elite-arena', to: '/student/elite/arena', icon: PhCrown, color: '#F59E0B', label: 'Elite Arena', match: 'exact' },
      { name: 'elite-records', to: '/student/elite/records', icon: PhArrowFatLinesUp, color: '#D97706', label: 'Elite Records', match: 'exact' },
      { name: 'elite-insights', to: '/student/elite/insights', icon: PhChartBar, color: '#C2410C', label: 'Elite Insights', match: 'exact' },
      { name: 'knowledge-gap', to: '/student/knowledge-gap', icon: PhTarget, color: '#FB923C', label: 'Knowledge Gaps', match: 'prefix' },
      { name: 'memory', to: '/student/memory', icon: PhBrain, color: '#8B5CF6', label: 'Memory', match: 'exact', excludeHashes: ['#reviews'] },
      { name: 'review-queue', to: '/student/memory', hash: '#reviews', activeHash: '#reviews', icon: PhCalendar, color: '#7C3AED', label: 'Review Queue', match: 'exact' },
    ],
  },
  {
    label: 'Resources',
    items: [
      { name: 'glossary', to: '/student/glossary', icon: PhBook, color: '#10B981', label: 'Glossary', match: 'exact' },
      { name: 'glossary-audio', to: '/student/glossary/audio', icon: PhBook, color: '#0F766E', label: 'Audio Glossary', match: 'exact' },
      { name: 'library', to: '/student/library', icon: PhBooks, color: '#6366F1', label: 'Library', match: 'exact' },
      { name: 'exam-intel', to: '/student/exam-intel', icon: PhLightbulbFilament, color: '#EC4899', label: 'Exam Intelligence', match: 'prefix' },
      { name: 'games', to: '/student/games', icon: PhGameController, color: '#14B8A6', label: 'Games Hub', match: 'exact' },
      { name: 'concept-traps', to: '/student/games/traps', icon: PhGameController, color: '#0F766E', label: 'Concept Traps', match: 'exact' },
      { name: 'mindstack', to: '/student/games/mindstack', icon: PhGameController, color: '#0D9488', label: 'MindStack', match: 'exact' },
      { name: 'tugofwar', to: '/student/games/tugofwar', icon: PhGameController, color: '#14B8A6', label: 'Tug of War', match: 'exact' },
      { name: 'upload', to: '/student/upload', icon: PhBooks, color: '#7C3AED', label: 'Uploads', match: 'exact' },
    ],
  },
  {
    label: 'Progress',
    items: [
      { name: 'progress', to: '/student/progress', icon: PhChartBar, color: '#84CC16', label: 'Overview', match: 'exact' },
      { name: 'mastery-map', to: '/student/progress/mastery-map', icon: PhTarget, color: '#65A30D', label: 'Mastery Map', match: 'exact' },
      { name: 'analytics', to: '/student/progress/analytics', icon: PhChartBar, color: '#A3E635', label: 'Analytics', match: 'exact' },
      { name: 'history', to: '/student/progress/history', icon: PhCalendar, color: '#22C55E', label: 'History', match: 'exact' },
      { name: 'mistakes', to: '/student/mistakes', icon: PhBug, color: '#F43F5E', label: 'Mistake Clinic', match: 'exact' },
      { name: 'calendar', to: '/student/calendar', icon: PhCalendar, color: '#64748B', label: 'Study Calendar', match: 'exact' },
    ],
  },
  {
    label: 'Account',
    items: [
      { name: 'student-settings', to: '/student/settings', icon: PhGear, color: '#475569', label: 'Settings', match: 'exact' },
    ],
  },
]

function isActive(item: NavItem): boolean {
  if (item.excludeBases?.some(base => route.path === base || route.path.startsWith(`${base}/`))) return false
  if (item.excludeHashes?.includes(route.hash)) return false
  if (item.match === 'prefix') {
    const base = item.activeBase ?? item.to
    if (item.activeHash && route.hash !== item.activeHash) return false
    return route.path === base || route.path.startsWith(`${base}/`)
  }
  if (route.path !== item.to) return false
  if (item.activeHash) return route.hash === item.activeHash
  return true
}

function logout() {
  auth.logout()
  router.push('/')
}
</script>

<template>
  <div class="app-shell">
    <!-- Sidebar -->
    <aside class="sidebar" :class="collapsed ? 'sidebar--collapsed' : ''">

      <!-- Brand -->
      <div class="sidebar-brand">
        <div class="brand-mark">
          <span class="brand-letter">e</span>
        </div>
        <div v-if="!collapsed" class="brand-text">
          <p class="brand-name">eCoach</p>
          <p class="brand-tagline">Study Smarter</p>
        </div>
        <button class="collapse-btn" :class="collapsed ? 'collapse-btn--center' : ''" @click="collapsed = !collapsed">
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
            <path v-if="!collapsed" d="M1.5 3.5h10M1.5 6.5h10M1.5 9.5h10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
            <path v-else d="M4.5 2.5l4 4-4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>

      <!-- Navigation -->
      <nav class="sidebar-nav">
        <div v-for="group in navGroups" :key="group.label" class="nav-group">
          <p v-if="!collapsed" class="nav-group-label">{{ group.label }}</p>
          <div v-else class="nav-group-divider" />

          <RouterLink
            v-for="item in group.items"
            :key="item.name"
            :to="item.hash ? { path: item.to, hash: item.hash } : item.to"
            class="nav-item"
            :class="[collapsed ? 'nav-item--collapsed' : '']"
            :style="{
              '--item-color': item.color,
              backgroundColor: isActive(item) ? item.color + '18' : 'transparent',
              color: isActive(item) ? item.color : 'var(--ink-secondary)',
              fontWeight: isActive(item) ? '600' : '500',
            }"
            :title="collapsed ? item.label : undefined"
          >
            <span
              v-if="isActive(item) && !collapsed"
              class="nav-active-bar"
              :style="{ backgroundColor: item.color }"
            />
            <component
              :is="item.icon"
              :size="16"
              weight="fill"
              class="nav-icon"
              :style="{
                color: item.color,
                opacity: isActive(item) ? 1 : 0.65,
              }"
            />
            <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
          </RouterLink>
        </div>
      </nav>

      <!-- User footer -->
      <div class="sidebar-footer">
        <div v-if="!collapsed && auth.currentAccount" class="user-chip">
          <div class="user-avatar">{{ auth.currentAccount.display_name.charAt(0).toUpperCase() }}</div>
          <span class="user-name">{{ auth.currentAccount.display_name }}</span>
        </div>
        <button class="signout-btn" :class="collapsed ? 'signout-btn--center' : ''" :title="collapsed ? 'Sign Out' : undefined" @click="logout">
          <PhSignOut :size="15" weight="fill" style="color: #F43F5E; opacity: 0.65;" />
          <span v-if="!collapsed" class="signout-label">Sign Out</span>
        </button>
      </div>
    </aside>

    <!-- Main content -->
    <main class="main-content">
      <ErrorBoundary>
        <RouterView />
      </ErrorBoundary>
    </main>
  </div>
</template>

<style scoped>
/* ─── Shell ─────────────────────────────────────── */
.app-shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--paper);
}

/* ─── Sidebar ────────────────────────────────────── */
.sidebar {
  display: flex;
  flex-direction: column;
  width: var(--sidebar-width, 240px);
  flex-shrink: 0;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--paper-warm);
  border-right: 1px solid var(--border-soft);
  transition: width var(--dur-slow, 250ms) ease;
}
.sidebar--collapsed { width: 56px; }

/* Brand */
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  height: 56px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-soft);
}
.brand-mark {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: var(--ink);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.brand-letter {
  color: var(--paper);
  font-family: var(--font-display);
  font-weight: 900;
  font-size: 13px;
}
.brand-text { flex: 1; min-width: 0; }
.brand-name {
  font-family: var(--font-display);
  font-weight: 700;
  font-size: 13px;
  color: var(--ink);
  line-height: 1.1;
}
.brand-tagline {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
  margin-top: 1px;
}
.collapse-btn {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-muted);
  margin-left: auto;
  flex-shrink: 0;
  transition: background 120ms, color 120ms;
}
.collapse-btn--center { margin: 0 auto; }
.collapse-btn:hover { background: var(--border-soft); color: var(--ink); }

/* Navigation */
.sidebar-nav {
  flex: 1;
  padding: 12px 0;
  overflow-x: hidden;
}
.nav-group { margin-bottom: 20px; }
.nav-group-label {
  padding: 0 16px;
  margin-bottom: 4px;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
}
.nav-group-divider {
  height: 1px;
  background: var(--border-soft);
  margin: 0 10px 8px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 1px 8px;
  padding: 7px 8px;
  border-radius: 8px;
  text-decoration: none;
  overflow: hidden;
  transition: background 110ms ease, color 110ms ease;
}
.nav-item--collapsed { justify-content: center; }
.nav-item:hover {
  background-color: color-mix(in srgb, var(--item-color, #888) 10%, transparent) !important;
  color: var(--item-color, var(--ink)) !important;
}
.nav-item:hover :deep(svg) {
  opacity: 1 !important;
}
.nav-active-bar {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 18px;
  border-radius: 0 3px 3px 0;
}
.nav-icon {
  flex-shrink: 0;
  transition: opacity 110ms;
}
.nav-label {
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Footer */
.sidebar-footer {
  flex-shrink: 0;
  padding: 8px;
  border-top: 1px solid var(--border-soft);
}
.user-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 8px;
  margin-bottom: 2px;
  background: var(--border-soft);
}
.user-avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--ink);
  color: var(--surface);
  font-size: 9px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.user-name {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ink-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.signout-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 8px;
  transition: background 110ms;
}
.signout-btn--center { justify-content: center; }
.signout-btn:hover { background: rgba(244, 63, 94, 0.08); }
.signout-label { font-size: 12.5px; color: var(--ink-muted); font-weight: 500; }

/* Main */
.main-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--paper);
}
</style>
