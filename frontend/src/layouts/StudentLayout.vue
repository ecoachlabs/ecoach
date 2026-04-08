<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  PhArrowFatLinesUp,
  PhArrowsCounterClockwise,
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
  PhFlame,
  PhFlask,
  PhGameController,
  PhGear,
  PhHouse,
  PhLightbulbFilament,
  PhLightning,
  PhMapTrifold,
  PhMagnifyingGlass,
  PhMedal,
  PhNotePencil,
  PhPersonSimpleRun,
  PhRocketLaunch,
  PhSealCheck,
  PhShuffleAngular,
  PhSquaresFour,
  PhStar,
  PhStudent,
  PhTarget,
  PhUploadSimple,
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

type WebAudioWindow = Window & { webkitAudioContext?: typeof AudioContext }

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

onMounted(() => ui.setTheme('student'))

const navSections: NavItem[][] = [
  [
    { name: 'home', label: 'Home', to: '/student', icon: PhHouse, color: '#FF6B35', match: 'exact' },
    { name: 'explore', label: 'Explore', to: '/student/journey', icon: PhCompass, color: '#2563EB', match: 'prefix' },
    { name: 'teach', label: 'Teach', to: '/student/spark', icon: PhStudent, color: '#8B5CF6', match: 'exact' },
    { name: 'past-questions', label: 'Past Questions', to: '/student/practice', icon: PhNotePencil, color: '#F59E0B', match: 'exact' },
    { name: 'custom-test', label: 'Custom Test', to: '/student/practice/custom-test', icon: PhClipboardText, color: '#D97706', match: 'exact' },
    { name: 'prepare-test', label: 'Prepare Test', to: '/student/mock', icon: PhClockCountdown, color: '#F97316', match: 'prefix', excludeBases: ['/student/mock/history'] },
    { name: 'mock-history', label: 'Mock History', to: '/student/mock/history', icon: PhClockCounterClockwise, color: '#D97706', match: 'exact' },
    { name: 'examples', label: 'Examples', to: '/student/glossary/compare', icon: PhSquaresFour, color: '#06B6D4', match: 'exact' },
  ],
  [
    { name: 'diagnostic', label: 'Diagnostic DNA', to: '/student/diagnostic', icon: PhDna, color: '#7C3AED', match: 'prefix' },
    { name: 'knowledge-gap', label: 'Knowledge Gap', to: '/student/knowledge-gap', icon: PhChartLineDown, color: '#22C55E', match: 'prefix', excludeBases: ['/student/knowledge-gap/scan'] },
    { name: 'gap-scan', label: 'Gap Scan', to: '/student/knowledge-gap/scan', icon: PhMagnifyingGlass, color: '#0EA5E9', match: 'exact' },
    { name: 'mistake-lab', label: 'Mistake Lab', to: '/student/mistakes', icon: PhBug, color: '#F43F5E', match: 'exact', excludeHashes: ['#retry-zone'] },
    { name: 'retry-zone', label: 'Retry Zone', to: '/student/mistakes', hash: '#retry-zone', activeHash: '#retry-zone', icon: PhTarget, color: '#EF4444', match: 'exact' },
  ],
  [
    { name: 'library', label: 'Library', to: '/student/library', icon: PhBooks, color: '#6366F1', match: 'prefix', excludeHashes: ['#revision-box'] },
    { name: 'revision-box', label: 'Revision Box', to: '/student/library', hash: '#revision-box', activeHash: '#revision-box', icon: PhChartLineUp, color: '#6366F1', match: 'exact' },
    { name: 'glossary', label: 'Glossary', to: '/student/glossary', icon: PhBookOpenText, color: '#4F46E5', match: 'prefix', excludeBases: ['/student/glossary/compare'] },
    { name: 'audio-glossary', label: 'Audio Glossary', to: '/student/glossary/audio', icon: PhBookOpenText, color: '#0F766E', match: 'exact' },
    { name: 'formula-lab', label: 'Formula Lab', to: '/student/glossary/formula-lab', icon: PhFlask, color: '#0EA5E9', match: 'exact' },
    { name: 'mental', label: 'Mental', to: '/student/memory', icon: PhLightning, color: '#F59E0B', match: 'exact', excludeHashes: ['#reviews'] },
    { name: 'review-queue', label: 'Review Queue', to: '/student/memory', hash: '#reviews', activeHash: '#reviews', icon: PhArrowsCounterClockwise, color: '#7C3AED', match: 'exact' },
    { name: 'memory', label: 'Memory', to: '/student/memory', icon: PhBrain, color: '#EC4899', match: 'exact', excludeHashes: ['#reviews'] },
  ],
  [
    { name: 'progress', label: 'Progress', to: '/student/progress', icon: PhChartBar, color: '#84CC16', match: 'exact' },
    { name: 'analytics', label: 'Analytics', to: '/student/progress/analytics', icon: PhChartBar, color: '#3B82F6', match: 'exact' },
    { name: 'mastery-map', label: 'Mastery Map', to: '/student/progress/mastery-map', icon: PhMapTrifold, color: '#22C55E', match: 'exact' },
    { name: 'history', label: 'History', to: '/student/progress/history', icon: PhClockCounterClockwise, color: '#64748B', match: 'exact' },
    { name: 'calendar', label: 'Calendar', to: '/student/calendar', icon: PhCalendar, color: '#22C55E', match: 'exact' },
    { name: 'exam-intel', label: 'Exam Intel', to: '/student/exam-intel', icon: PhLightbulbFilament, color: '#7C3AED', match: 'prefix' },
  ],
  [
    { name: 'onboarding', label: 'Onboarding', to: '/student/onboarding/welcome', icon: PhMapTrifold, color: '#0EA5E9', match: 'prefix', activeBase: '/student/onboarding' },
    { name: 'beat-yesterday', label: 'Beat Yesterday', to: '/student/beat-yesterday', icon: PhArrowFatLinesUp, color: '#EF4444', match: 'exact' },
    { name: 'marathon', label: 'Marathon', to: '/student/rise', icon: PhPersonSimpleRun, color: '#FB923C', match: 'exact' },
    { name: 'rise', label: 'Rise', to: '/student/rise', icon: PhRocketLaunch, color: '#3B82F6', match: 'exact' },
    { name: 'elite', label: 'Elite', to: '/student/elite', icon: PhCrown, color: '#FBBF24', match: 'prefix', excludeBases: ['/student/elite/arena', '/student/elite/records', '/student/elite/insights'] },
    { name: 'answer-lab', label: 'Answer Lab', to: '/student/elite/arena', icon: PhFlask, color: '#14B8A6', match: 'exact' },
    { name: 'speed-lab', label: 'Speed Lab', to: '/student/elite/records', icon: PhDrop, color: '#0EA5E9', match: 'exact' },
    { name: 'games', label: 'Games', to: '/student/games', icon: PhGameController, color: '#7C3AED', match: 'prefix', excludeBases: ['/student/games/mindstack', '/student/games/tugofwar', '/student/games/traps'] },
    { name: 'traps', label: 'Traps', to: '/student/games/traps', icon: PhShuffleAngular, color: '#EF4444', match: 'exact' },
    { name: 'uploads', label: 'Uploads', to: '/student/upload', icon: PhUploadSimple, color: '#6366F1', match: 'exact' },
    { name: 'settings', label: 'Settings', to: '/student/settings', icon: PhGear, color: '#475569', match: 'exact' },
  ],
]

const appShortcut: NavItem = {
  name: 'adeo',
  label: 'Adeo v2',
  to: '/student/settings',
  icon: PhSealCheck,
  color: '#F97316',
  match: 'exact',
}

let navAudioContext: AudioContext | null = null
let lastNavSoundAt = 0

function playNavSelectSound() {
  if (typeof window === 'undefined') return
  const now = performance.now()
  if (now - lastNavSoundAt < 70) return
  lastNavSoundAt = now

  try {
    const AudioContextCtor = window.AudioContext || (window as WebAudioWindow).webkitAudioContext
    if (!AudioContextCtor) return

    if (!navAudioContext) {
      navAudioContext = new AudioContextCtor()
    }
    if (navAudioContext.state === 'suspended') {
      void navAudioContext.resume()
    }

    const t = navAudioContext.currentTime
    const osc = navAudioContext.createOscillator()
    const filter = navAudioContext.createBiquadFilter()
    const gain = navAudioContext.createGain()

    osc.type = 'triangle'
    osc.frequency.setValueAtTime(820, t)
    osc.frequency.exponentialRampToValueAtTime(620, t + 0.09)

    filter.type = 'lowpass'
    filter.frequency.setValueAtTime(1800, t)

    gain.gain.setValueAtTime(0.0001, t)
    gain.gain.exponentialRampToValueAtTime(0.018, t + 0.012)
    gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.13)

    osc.connect(filter)
    filter.connect(gain)
    gain.connect(navAudioContext.destination)

    osc.start(t)
    osc.stop(t + 0.13)
  } catch {
    // Ignore audio failures on unsupported/locked environments.
  }
}

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

const displayName = computed(() => auth.currentAccount?.display_name ?? 'Kofi')
const initials = computed(() => auth.currentAccount?.display_name?.charAt(0).toUpperCase() ?? 'K')
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="profile-block">
        <div class="profile-avatar">{{ initials }}</div>
        <div class="profile-meta">
          <p class="profile-name">{{ displayName }}</p>
          <p class="profile-stats">
            <PhStar :size="10" weight="fill" />
            <span>7</span>
            <span class="profile-goal">0/20</span>
          </p>
        </div>
      </div>

      <nav class="sidebar-nav">
        <div v-for="(section, sectionIndex) in navSections" :key="`sec-${sectionIndex}`" class="nav-section">
          <RouterLink
            v-for="item in section"
            :key="item.hash ? `${item.to}${item.hash}` : item.name"
            :to="item.hash ? { path: item.to, hash: item.hash } : item.to"
            class="nav-link"
            :class="{ 'nav-link--active': isActive(item) }"
            :style="{ '--item-color': item.color }"
            @click="playNavSelectSound"
          >
            <component
              :is="item.icon"
              :size="15"
              weight="fill"
              class="nav-icon"
              :style="{ color: item.color }"
            />
            <span class="nav-label">{{ item.label }}</span>
          </RouterLink>
          <div v-if="sectionIndex < navSections.length - 1" class="nav-divider" />
        </div>
      </nav>

      <div class="sidebar-bottom">
        <RouterLink
          :to="appShortcut.to"
          class="app-shortcut"
          :class="{ 'app-shortcut--active': isActive(appShortcut) }"
          :style="{ '--item-color': appShortcut.color }"
          @click="playNavSelectSound"
        >
          <component :is="appShortcut.icon" :size="15" weight="fill" :style="{ color: appShortcut.color }" />
          <span>{{ appShortcut.label }}</span>
        </RouterLink>
      </div>
    </aside>

    <div class="page-body">
      <header class="top-bar">
        <div class="top-spacer" />
        <div class="top-right">
          <div class="top-stat">
            <PhFlame :size="15" weight="fill" style="color: #FF6B35;" />
            <span class="top-stat-val">7</span>
          </div>
          <div class="top-stat">
            <PhLightning :size="15" weight="fill" style="color: #FBBF24;" />
            <span class="top-stat-val">5,299</span>
          </div>
          <div class="level-badge">
            <PhMedal :size="13" weight="fill" style="color: #7C3AED;" />
            <span class="level-text">Scholar Lv.8</span>
          </div>
          <div class="top-sep" />
          <button class="top-icon-btn" @click="router.push('/student/settings')">
            <PhGear :size="17" weight="fill" style="color: #94a3b8;" />
          </button>
          <div class="top-avatar">{{ initials }}</div>
        </div>
      </header>

      <main class="main-content">
        <ErrorBoundary>
          <RouterView />
        </ErrorBoundary>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: #f6f7f4;
}

.sidebar {
  width: 182px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: #f8f8f5;
  border-right: none;
  overflow: hidden;
}

.profile-block {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 12px 12px;
}

.profile-avatar {
  width: 28px;
  height: 28px;
  border-radius: 10px;
  background: #f97316;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.profile-meta {
  min-width: 0;
}

.profile-name {
  margin: 0;
  font-size: 12px;
  line-height: 1.1;
  font-weight: 600;
  color: #1f2937;
}

.profile-stats {
  margin: 2px 0 0;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: #8a8f98;
  font-weight: 600;
}

.profile-goal {
  color: #6b7280;
}

.sidebar-nav {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 4px 0 8px;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.sidebar-nav::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}

.nav-section {
  display: block;
}

.nav-link {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 1px 8px;
  padding: 8px 8px 8px 12px;
  border-radius: 10px;
  border: 1px solid transparent;
  text-decoration: none;
  color: #7b818b;
  overflow: hidden;
  isolation: isolate;
  transform: translateX(0);
  transition: background-color 180ms ease, color 180ms ease, transform 220ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 220ms ease, border-color 180ms ease;
}

.nav-link:hover {
  background: color-mix(in srgb, var(--item-color, #94a3b8) 10%, transparent);
  color: color-mix(in srgb, var(--item-color, #475569) 70%, #2a313a);
  transform: translateX(1px);
}

.nav-link--active {
  background: color-mix(in srgb, var(--item-color, #3b82f6) 11%, rgba(255, 255, 255, 0.88));
  color: color-mix(in srgb, var(--item-color, #1d4ed8) 85%, #1f2937);
  border-color: color-mix(in srgb, var(--item-color, #3b82f6) 28%, rgba(255, 255, 255, 0.55));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.90),
    inset 0 -1px 0 rgba(255, 255, 255, 0.30),
    0 4px 14px color-mix(in srgb, var(--item-color, #3b82f6) 18%, transparent),
    0 1px 3px rgba(15, 23, 42, 0.08);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  transform: translateX(3px);
}

/* Top-left light catch — the primary glass sheen */
.nav-link--active::before {
  content: '';
  position: absolute;
  inset: 1px;
  border-radius: 9px;
  z-index: 0;
  pointer-events: none;
  background:
    linear-gradient(
      148deg,
      rgba(255, 255, 255, 0.72) 0%,
      rgba(255, 255, 255, 0.18) 30%,
      rgba(255, 255, 255, 0.04) 65%,
      rgba(255, 255, 255, 0.00) 100%
    ),
    radial-gradient(
      88% 60% at 14% 14%,
      rgba(255, 255, 255, 0.52) 0%,
      rgba(255, 255, 255, 0.00) 72%
    );
}

/* Color wash layer */
.nav-link--active::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 10px;
  z-index: 1;
  pointer-events: none;
  background: color-mix(in srgb, var(--item-color, #3b82f6) 13%, transparent);
}

.nav-icon {
  flex-shrink: 0;
  opacity: 0.92;
  position: relative;
  z-index: 2;
  transition: opacity 150ms ease, transform 200ms ease;
}

.nav-label {
  font-size: 13px;
  line-height: 1.1;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  position: relative;
  z-index: 2;
}

.nav-link--active .nav-label {
  font-weight: 600;
}

.nav-link:hover .nav-icon,
.nav-link--active .nav-icon {
  opacity: 1;
  transform: scale(1.03);
}

.nav-divider {
  margin: 8px 10px;
  height: 1px;
  background: #e5e9e2;
}

.sidebar-bottom {
  border-top: none;
  padding: 8px 8px 10px;
}

.app-shortcut {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid transparent;
  text-decoration: none;
  color: #7b818b;
  font-size: 12px;
  font-weight: 600;
  position: relative;
  overflow: hidden;
  isolation: isolate;
  transition: background-color 180ms ease, color 180ms ease, transform 220ms cubic-bezier(0.22, 1, 0.36, 1), box-shadow 220ms ease, border-color 180ms ease;
}

.app-shortcut:hover {
  background: color-mix(in srgb, var(--item-color, #f97316) 10%, transparent);
  color: color-mix(in srgb, var(--item-color, #f97316) 70%, #2a313a);
  transform: translateX(1px);
}

.app-shortcut--active {
  background: color-mix(in srgb, var(--item-color, #f97316) 11%, rgba(255, 255, 255, 0.88));
  color: color-mix(in srgb, var(--item-color, #f97316) 85%, #1f2937);
  border-color: color-mix(in srgb, var(--item-color, #f97316) 28%, rgba(255, 255, 255, 0.55));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.90),
    inset 0 -1px 0 rgba(255, 255, 255, 0.30),
    0 4px 14px color-mix(in srgb, var(--item-color, #f97316) 18%, transparent),
    0 1px 3px rgba(15, 23, 42, 0.08);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  transform: translateX(3px);
}

.app-shortcut--active::before {
  content: '';
  position: absolute;
  inset: 1px;
  border-radius: 9px;
  z-index: 0;
  pointer-events: none;
  background:
    linear-gradient(
      148deg,
      rgba(255, 255, 255, 0.72) 0%,
      rgba(255, 255, 255, 0.18) 30%,
      rgba(255, 255, 255, 0.04) 65%,
      rgba(255, 255, 255, 0.00) 100%
    ),
    radial-gradient(
      88% 60% at 14% 14%,
      rgba(255, 255, 255, 0.52) 0%,
      rgba(255, 255, 255, 0.00) 72%
    );
}

.app-shortcut--active::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 10px;
  z-index: 1;
  pointer-events: none;
  background: color-mix(in srgb, var(--item-color, #f97316) 13%, transparent);
}

.app-shortcut > * {
  position: relative;
  z-index: 2;
}

.page-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.top-bar {
  display: flex;
  align-items: center;
  height: 52px;
  padding: 0 20px;
  background: #ffffff;
  border-bottom: none;
  flex-shrink: 0;
}

.top-spacer { flex: 1; }

.top-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

.top-stat {
  display: flex;
  align-items: center;
  gap: 4px;
}

.top-stat-val {
  font-size: 13px;
  font-weight: 700;
  color: var(--ink-secondary);
}

.level-badge {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px 3px 8px;
  border-radius: 20px;
  background: rgba(124, 58, 237, 0.07);
  border: 1px solid rgba(124, 58, 237, 0.14);
}

.level-text {
  font-size: 11.5px;
  font-weight: 700;
  color: #7C3AED;
}

.top-sep {
  width: 1px;
  height: 18px;
  background: var(--border-soft);
}

.top-icon-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  transition: background 120ms;
}

.top-icon-btn:hover { background: var(--border-soft); }

.top-avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: #FF6B35;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.main-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  background: #f6f7f4;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.main-content::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}
</style>
