<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import { getHomeLearningStats, type HomeLearningStatsDto } from '@/ipc/coach'
import { recoverDeferredSessionCompletions } from '@/ipc/sessions'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  appShortcut,
  isNavItemActive,
  studentNavSections,
  type NavLinkItem,
} from './studentNav'
import {
  PhBrain,
  PhFlame,
  PhGear,
  PhLightning,
  PhMedal,
  PhMoon,
  PhStar,
  PhSun,
} from '@phosphor-icons/vue'

type WebAudioWindow = Window & { webkitAudioContext?: typeof AudioContext }

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

onMounted(() => ui.setTheme('student'))

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

function isActive(item: NavLinkItem): boolean {
  return isNavItemActive(item, route.path, route.hash)
}

const flatNavItems = studentNavSections.flatMap(section => section.items)
const activeNavItem = computed(() => flatNavItems.find(item => isActive(item)) ?? null)

const displayName = computed(() => auth.currentAccount?.display_name ?? 'Kofi')
const initials = computed(() => auth.currentAccount?.display_name?.charAt(0).toUpperCase() ?? 'K')
const isCoachHubActive = computed(() => route.path === '/student/coach' || route.path === '/student/v3')
const coachToggleLabel = computed(() => isCoachHubActive.value ? 'Coach Active' : 'Activate Coach')
const homeLearningStats = ref<HomeLearningStatsDto | null>(null)
const shellStats = computed(() => ({
  streakDays: homeLearningStats.value?.streak_days ?? 0,
  todayMinutes: homeLearningStats.value?.today_minutes ?? 0,
  weekQuestions: homeLearningStats.value?.week_questions ?? 0,
  accuracyPercent: homeLearningStats.value?.accuracy_percent ?? 0,
}))

let statsRefreshHandle: number | null = null
let deferredRecoveryHandle: number | null = null

const DEFERRED_RECOVERY_INITIAL_DELAY_MS = 1500
const DEFERRED_RECOVERY_DRAIN_DELAY_MS = 5000
const DEFERRED_RECOVERY_RETRY_DELAY_MS = 60000

async function refreshShellStats() {
  const studentId = auth.currentAccount?.id
  if (!studentId) {
    homeLearningStats.value = null
    return
  }

  try {
    homeLearningStats.value = await getHomeLearningStats(studentId)
  } catch (error) {
    console.warn('Failed to load student shell stats', error)
    homeLearningStats.value = null
  }
}

async function recoverDeferredCompletions(studentId: number) {
  try {
    const result = await recoverDeferredSessionCompletions(studentId, 1)
    if (result.succeeded > 0 || result.failed > 0) {
      void refreshShellStats()
    }
    if (auth.currentAccount?.id === studentId) {
      if (result.remaining > 0) {
        scheduleDeferredRecovery(studentId, DEFERRED_RECOVERY_DRAIN_DELAY_MS)
      } else if (result.failed > 0) {
        scheduleDeferredRecovery(studentId, DEFERRED_RECOVERY_RETRY_DELAY_MS)
      }
    }
  } catch (error) {
    console.warn('Failed to recover deferred session completions', error)
    if (auth.currentAccount?.id === studentId) {
      scheduleDeferredRecovery(studentId, DEFERRED_RECOVERY_RETRY_DELAY_MS)
    }
  }
}

function scheduleDeferredRecovery(studentId: number, delayMs: number) {
  if (typeof window === 'undefined') return
  if (deferredRecoveryHandle !== null) {
    window.clearTimeout(deferredRecoveryHandle)
  }
  deferredRecoveryHandle = window.setTimeout(() => {
    deferredRecoveryHandle = null
    void recoverDeferredCompletions(studentId)
  }, delayMs)
}

function toggleCoachHub() {
  router.push(isCoachHubActive.value ? '/student' : '/student/coach')
}

// ── Sliding glass pill ────────────────────────────────────────────
const navRef = ref<HTMLElement | null>(null)
const pillTop = ref(0)
const pillHeight = ref(36)
const pillColor = ref('#3b82f6')
const pillVisible = ref(false)

async function updatePill() {
  await nextTick()
  const nav = navRef.value
  if (!nav) return
  const active = nav.querySelector('.nav-link--active') as HTMLElement | null
  if (!active) { pillVisible.value = false; return }
  pillTop.value = active.offsetTop
  pillHeight.value = active.offsetHeight
  pillVisible.value = true
  if (activeNavItem.value) pillColor.value = activeNavItem.value.color
}

onMounted(updatePill)
onMounted(() => {
  if (typeof window === 'undefined') return
  statsRefreshHandle = window.setInterval(() => {
    void refreshShellStats()
  }, 15000)
})

onUnmounted(() => {
  if (typeof window === 'undefined') return
  if (statsRefreshHandle !== null) {
    window.clearInterval(statsRefreshHandle)
    statsRefreshHandle = null
  }
  if (deferredRecoveryHandle !== null) {
    window.clearTimeout(deferredRecoveryHandle)
    deferredRecoveryHandle = null
  }
})

watch(() => route.fullPath, () => {
  void updatePill()
  void refreshShellStats()
})
watch(() => auth.currentAccount?.id, (studentId) => {
  void refreshShellStats()
  if (typeof window !== 'undefined' && deferredRecoveryHandle !== null) {
    window.clearTimeout(deferredRecoveryHandle)
    deferredRecoveryHandle = null
  }
  if (studentId) {
    scheduleDeferredRecovery(studentId, DEFERRED_RECOVERY_INITIAL_DELAY_MS)
  }
}, { immediate: true })
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
            <span>{{ shellStats.streakDays }}d</span>
            <span class="profile-goal">{{ shellStats.accuracyPercent }}%</span>
          </p>
        </div>
      </div>

      <nav class="sidebar-nav" ref="navRef">
        <!-- Sliding liquid glass pill -->
        <div
          class="glass-pill"
          :class="{ 'glass-pill--visible': pillVisible }"
          :style="{ '--pc': pillColor, top: pillTop + 'px', height: pillHeight + 'px' }"
        />
        <div v-for="(section, sectionIndex) in studentNavSections" :key="section.title" class="nav-section">
          <p class="nav-section-title">{{ section.title }}</p>
          <div class="nav-section-items">
            <RouterLink
              v-for="item in section.items"
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
          </div>
          <div v-if="sectionIndex < studentNavSections.length - 1" class="nav-divider" />
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
          <div class="top-stat" title="Current streak">
            <PhFlame :size="15" weight="fill" style="color: #FF6B35;" />
            <span class="top-stat-val">{{ shellStats.streakDays }}</span>
          </div>
          <div class="top-stat" title="Questions answered this week">
            <PhLightning :size="15" weight="fill" style="color: #FBBF24;" />
            <span class="top-stat-val">{{ shellStats.weekQuestions }}</span>
          </div>
          <div class="level-badge" :title="`${shellStats.todayMinutes} minutes studied today`">
            <PhMedal :size="13" weight="fill" style="color: #7C3AED;" />
            <span class="level-text">{{ shellStats.todayMinutes }}m today</span>
          </div>
          <div class="top-sep" />
          <button
            class="coach-toggle"
            :class="{ 'coach-toggle--active': isCoachHubActive }"
            :title="coachToggleLabel"
            @click="toggleCoachHub"
          >
            <PhBrain :size="15" weight="fill" />
            <span>{{ coachToggleLabel }}</span>
          </button>
          <button class="top-icon-btn dark-toggle" :title="ui.isDark ? 'Light mode' : 'Dark mode'" @click="ui.toggleDark()">
            <Transition name="dark-icon" mode="out-in">
              <PhSun v-if="ui.isDark" :key="'sun'" :size="17" weight="fill" style="color: #FBBF24;" />
              <PhMoon v-else :key="'moon'" :size="17" weight="fill" style="color: #94a3b8;" />
            </Transition>
          </button>
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
  background: var(--paper);
  transition: background-color 240ms ease;
}

.sidebar {
  width: 198px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--paper);
  border-right: none;
  overflow: hidden;
  transition: background-color 240ms ease;
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
  color: var(--ink);
  transition: color 240ms ease;
}

.profile-stats {
  margin: 2px 0 0;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--ink-muted);
  font-weight: 600;
  transition: color 240ms ease;
}

.profile-goal {
  color: var(--ink-muted);
}

.sidebar-nav {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 4px 0 8px;
  scrollbar-width: none;
  -ms-overflow-style: none;
  position: relative; /* pill anchor */
}

.sidebar-nav::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}

/* ── Liquid glass sliding pill ───────────────────────── */
.glass-pill {
  position: absolute;
  left: 8px;
  right: 8px;
  border-radius: 10px;
  pointer-events: none;
  z-index: 0;
  opacity: 0;

  /* Spring movement — slides + tiny overshoot = liquid feel */
  transition:
    top    400ms cubic-bezier(0.34, 1.56, 0.64, 1),
    height 260ms cubic-bezier(0.34, 1.56, 0.64, 1),
    opacity 180ms ease,
    background 300ms ease,
    border-color 300ms ease,
    box-shadow 300ms ease;

  /* Glass surface — uses CSS vars that flip in dark mode */
  background: color-mix(in srgb, var(--pc, #3b82f6) 11%, var(--glass-base));
  border: 1px solid color-mix(in srgb, var(--pc, #3b82f6) 22%, var(--glass-border));
  box-shadow:
    inset 0 1px 0 var(--glass-highlight),
    inset 0 -1px 0 var(--glass-shadow-inset),
    0 4px 18px color-mix(in srgb, var(--pc, #3b82f6) 14%, transparent),
    0 1px 3px rgba(15, 23, 42, 0.06);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
}

.glass-pill--visible { opacity: 1; }

/* Top-left light catch — uses glass vars so it dims in dark mode */
.glass-pill::before {
  content: '';
  position: absolute;
  inset: 1px;
  border-radius: 9px;
  pointer-events: none;
  background:
    linear-gradient(
      148deg,
      var(--glass-sheen-strong) 0%,
      var(--glass-sheen-mid) 30%,
      rgba(255, 255, 255, 0.02) 62%,
      transparent 100%
    ),
    radial-gradient(
      82% 52% at 12% 18%,
      var(--glass-sheen-radial) 0%,
      transparent 66%
    );
}

/* Left handle bar — solid colour, fully rounded pill */
.glass-pill::after {
  content: '';
  position: absolute;
  left: 7px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 58%;
  min-height: 14px;
  border-radius: 99px;
  background: var(--pc, #3b82f6);
  pointer-events: none;
  /* transitions match the pill colour change */
  transition: background 300ms ease, height 260ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.nav-section {
  display: block;
}

.nav-section-title {
  margin: 10px 18px 6px;
  font-size: 10px;
  line-height: 1;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-muted);
  transition: color 240ms ease;
}

.nav-section:first-child .nav-section-title {
  margin-top: 6px;
}

.nav-section-items {
  display: block;
}

.nav-link {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 1px 8px;
  padding: 8px 8px 8px 18px;
  border-radius: 10px;
  border: 1px solid transparent;
  text-decoration: none;
  color: var(--ink-muted);
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

/* Active item: pill handles the background, we just colour the text/icon */
.nav-link--active {
  color: color-mix(in srgb, var(--item-color, #1d4ed8) 88%, #1f2937);
  transform: translateX(3px);
  position: relative;
  z-index: 1; /* sit above the pill */
}

.nav-link--active .nav-label { font-weight: 600; }

/* Press animation — item briefly sinks on click */
.nav-link:active {
  transform: scale(0.96) translateX(1px);
  transition: transform 60ms ease !important;
}
.nav-link--active:active {
  transform: scale(0.97) translateX(3px);
  transition: transform 60ms ease !important;
}

.nav-icon {
  flex-shrink: 0;
  opacity: 0.60;
  position: relative;
  z-index: 2;
  transition: opacity 200ms ease, transform 200ms ease;
}

.nav-link--active .nav-icon,
.nav-link:hover .nav-icon {
  opacity: 1;
  transform: scale(1.05);
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
  margin: 10px 14px 6px;
  height: 1px;
  background: var(--border-soft);
  transition: background-color 240ms ease;
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
  color: var(--ink-muted);
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
  background: var(--surface);
  border-bottom: 1px solid var(--border-soft);
  flex-shrink: 0;
  transition: background-color 240ms ease, border-color 240ms ease;
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
  transition: color 240ms ease;
}

/* Dark mode toggle animation */
.dark-icon-enter-active,
.dark-icon-leave-active {
  transition: opacity 140ms ease, transform 140ms ease;
}
.dark-icon-enter-from { opacity: 0; transform: rotate(-30deg) scale(0.7); }
.dark-icon-leave-to   { opacity: 0; transform: rotate(30deg) scale(0.7); }

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

.coach-toggle {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, #f59e0b 24%, var(--border-soft));
  background: color-mix(in srgb, #f59e0b 8%, var(--surface));
  color: #8a4b07;
  font-size: 12px;
  font-weight: 700;
  transition: background-color 140ms ease, border-color 140ms ease, color 140ms ease, transform 140ms ease;
}

.coach-toggle:hover {
  background: color-mix(in srgb, #f59e0b 14%, var(--surface));
  transform: translateY(-0.5px);
}

.coach-toggle:active {
  transform: translateY(0);
}

.coach-toggle--active {
  border-color: color-mix(in srgb, #7c3aed 30%, var(--border-soft));
  background: color-mix(in srgb, #7c3aed 12%, var(--surface));
  color: #6d28d9;
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
  background: var(--paper);
  scrollbar-width: none;
  -ms-overflow-style: none;
  transition: background-color 240ms ease;
}

.main-content::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}
</style>
