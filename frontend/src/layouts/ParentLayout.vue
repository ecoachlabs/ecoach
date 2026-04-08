<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  PhHouseLine,
  PhUsers,
  PhStudent,
  PhWarning,
  PhChartLineUp,
  PhNewspaper,
  PhSiren,
  PhGraduationCap,
  PhChatCircleDots,
  PhGear,
  PhMoon,
  PhSignOut,
  PhSun,
} from '@phosphor-icons/vue'

const ui = useUiStore()
const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

onMounted(() => {
  ui.setTheme('parent')
})

function logout() {
  auth.logout()
  router.push('/')
}

const navGroups = [
  {
    label: 'Overview',
    items: [
      { name: 'home', to: '/parent', label: 'Home', icon: PhHouseLine, color: '#3B82F6' },
      { name: 'household', to: '/parent/household', label: 'Household', icon: PhUsers, color: '#8B5CF6' },
      { name: 'children', to: '/parent/children', label: 'Children', icon: PhStudent, color: '#10B981' },
    ],
  },
  {
    label: 'Insights',
    items: [
      { name: 'attention', to: '/parent/attention', label: 'Attention Needed', icon: PhWarning, color: '#EF4444' },
      { name: 'performance', to: '/parent/performance', label: 'Performance', icon: PhChartLineUp, color: '#F59E0B' },
      { name: 'reports', to: '/parent/reports', label: 'Reports', icon: PhNewspaper, color: '#6366F1' },
      { name: 'intervention', to: '/parent/intervention', label: 'Intervention', icon: PhSiren, color: '#EC4899' },
      { name: 'curriculum', to: '/parent/curriculum', label: 'Curriculum', icon: PhGraduationCap, color: '#14B8A6' },
      { name: 'concierge', to: '/parent/concierge', label: 'Concierge', icon: PhChatCircleDots, color: '#84CC16' },
    ],
  },
  {
    label: 'Account',
    items: [
      { name: 'settings', to: '/parent/settings', label: 'Settings', icon: PhGear, color: '#64748B' },
    ],
  },
]

function isActive(target: string): boolean {
  if (target === '/parent') return route.path === '/parent'
  if (target === '/parent/children') {
    return route.path.startsWith('/parent/children') || route.path.startsWith('/parent/child/')
  }
  return route.path.startsWith(target)
}
</script>

<template>
  <div class="flex h-screen overflow-hidden" :style="{ backgroundColor: 'var(--bg)' }">

    <!-- Sidebar -->
    <aside
      class="w-[var(--sidebar-width)] flex flex-col border-r no-print shrink-0 overflow-y-auto"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }"
    >
      <!-- Logo -->
      <div class="flex items-center gap-2.5 px-4 h-14 shrink-0" :style="{ borderBottom: '1px solid var(--card-border)' }">
        <div class="w-8 h-8 rounded-[10px] flex items-center justify-center shadow-sm"
          style="background: linear-gradient(135deg, #1E293B, #3730A3);">
          <span class="text-white font-display font-black text-sm">e</span>
        </div>
        <div>
          <p class="font-display font-bold text-sm leading-none" :style="{ color: 'var(--text)' }">eCoach</p>
          <p class="text-[9px] uppercase tracking-widest mt-0.5" :style="{ color: 'var(--text-3)' }">Parent Portal</p>
        </div>
      </div>

      <!-- Nav -->
      <nav class="flex-1 py-3.5 space-y-5 overflow-x-hidden px-2">
        <div v-for="group in navGroups" :key="group.label">
          <p class="px-2 mb-2 text-[10px] font-bold uppercase tracking-[0.12em]" :style="{ color: 'var(--text-3)' }">
            {{ group.label }}
          </p>
          <div class="space-y-1">
            <RouterLink
              v-for="item in group.items"
              :key="item.name"
              :to="item.to"
              class="parent-nav-link flex items-center gap-2.5 px-2.5 py-2 rounded-[9px] text-sm relative"
              :style="{
                '--item-color': item.color,
                backgroundColor: isActive(item.to) ? item.color + '18' : 'transparent',
                color: isActive(item.to) ? item.color : 'var(--text-2)',
                fontWeight: isActive(item.to) ? '600' : '500',
                transition: 'background-color 120ms ease, color 120ms ease',
              }"
            >
              <span
                v-if="isActive(item.to)"
                class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 rounded-r-full"
                :style="{ backgroundColor: item.color }"
              />
              <component
                :is="item.icon"
                :size="18"
                weight="fill"
                :style="{
                  color: isActive(item.to) ? item.color : 'var(--text-3)',
                  opacity: isActive(item.to) ? 1 : 0.65,
                  transition: 'color 120ms ease, opacity 120ms ease',
                }"
                class="shrink-0"
              />
              <span class="truncate text-[14px]">{{ item.label }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Bottom -->
      <div class="shrink-0 pb-3 pt-2 border-t px-2" :style="{ borderColor: 'var(--card-border)' }">
        <div v-if="auth.currentAccount"
          class="flex items-center gap-2 px-2.5 py-1.5 mb-1 rounded-[9px]"
          :style="{ backgroundColor: 'var(--primary-light)' }">
          <div class="w-6 h-6 rounded-full flex items-center justify-center text-white text-[10px] font-bold shrink-0"
            style="background: linear-gradient(135deg, #1E293B, #3730A3);">
            {{ auth.currentAccount.display_name.charAt(0).toUpperCase() }}
          </div>
          <span class="text-xs font-semibold truncate" :style="{ color: 'var(--text-2)' }">
            {{ auth.currentAccount.display_name }}
          </span>
        </div>
        <button
          class="parent-dark-toggle flex items-center gap-2.5 w-full px-2.5 py-2 rounded-[9px] text-sm transition-colors"
          :title="ui.isDark ? 'Light mode' : 'Dark mode'"
          @click="ui.toggleDark()"
        >
          <Transition name="dark-icon" mode="out-in">
            <PhSun v-if="ui.isDark" :key="'sun'" :size="17" weight="fill" style="color: #FBBF24; opacity: 0.8;" />
            <PhMoon v-else :key="'moon'" :size="17" weight="fill" style="color: #94a3b8; opacity: 0.65;" />
          </Transition>
          <span class="text-[14px]" style="color: var(--text-3);">{{ ui.isDark ? 'Light Mode' : 'Dark Mode' }}</span>
        </button>
        <button
          class="sign-out flex items-center gap-2.5 w-full px-2.5 py-2 rounded-[9px] text-sm transition-colors"
          @click="logout"
        >
          <PhSignOut :size="17" weight="fill" style="color: #F43F5E; opacity: 0.65;" />
          <span class="text-[14px]" style="color: var(--text-3);">Sign Out</span>
        </button>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto" :style="{ backgroundColor: 'var(--bg)' }">
      <ErrorBoundary>
        <RouterView />
      </ErrorBoundary>
    </main>
  </div>
</template>

<style scoped>
.parent-nav-link:hover {
  background-color: color-mix(in srgb, var(--item-color, #888) 10%, transparent) !important;
  color: var(--item-color, var(--text)) !important;
}
.parent-nav-link:hover :deep(svg) {
  color: var(--item-color, var(--text-3)) !important;
  opacity: 1 !important;
}
.sign-out:hover { background-color: rgba(244, 63, 94, 0.08); }
.sign-out:hover :deep(svg) { opacity: 1 !important; }
.parent-dark-toggle:hover { background-color: color-mix(in srgb, var(--text-3) 10%, transparent); }
.dark-icon-enter-active, .dark-icon-leave-active { transition: opacity 140ms ease, transform 140ms ease; }
.dark-icon-enter-from { opacity: 0; transform: rotate(-30deg) scale(0.7); }
.dark-icon-leave-to   { opacity: 0; transform: rotate(30deg) scale(0.7); }
</style>
