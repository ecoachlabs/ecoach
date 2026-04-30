<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  PhArrowsClockwise,
  PhBooks,
  PhChartLineUp,
  PhCheckCircle,
  PhDatabase,
  PhGear,
  PhHouseLine,
  PhMagnifyingGlass,
  PhMoon,
  PhNewspaper,
  PhNotePencil,
  PhPencilSimple,
  PhSignOut,
  PhSun,
  PhTreeStructure,
  PhUploadSimple,
  PhUsers,
} from '@phosphor-icons/vue'

type NavItem = {
  to: string
  hash?: string
  label: string
  icon: any
  color: string
  match: 'exact' | 'prefix'
  activeHash?: string
  excludeHashes?: string[]
}

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()


onMounted(() => {
  ui.setTheme('admin')
})

function logout() {
  auth.logout()
  router.push('/')
}

function isActive(item: NavItem): boolean {
  if (item.excludeHashes?.includes(route.hash)) {
    return false
  }
  if (item.match === 'prefix') {
    if (item.activeHash && route.hash !== item.activeHash) {
      return false
    }
    return route.path === item.to || route.path.startsWith(`${item.to}/`)
  }
  if (route.path !== item.to) {
    return false
  }
  if (item.activeHash) {
    return route.hash === item.activeHash
  }
  return true
}

const navGroups: Array<{ label: string; items: NavItem[] }> = [
  {
    label: 'Overview',
    items: [
      { to: '/admin', label: 'Dashboard', icon: PhHouseLine, color: '#0F766E', match: 'exact' },
    ],
  },
  {
    label: 'Manage Content',
    items: [
      { to: '/admin/content-editor', label: 'Content Editor', icon: PhPencilSimple, color: '#0F766E', match: 'prefix' },
      { to: '/admin/questions', label: 'Question Bank', icon: PhDatabase, color: '#B45309', match: 'exact' },
      { to: '/admin/past-papers', label: 'Past Papers', icon: PhNotePencil, color: '#F59E0B', match: 'prefix' },
      { to: '/admin/content', label: 'Sources & Ingestion', icon: PhUploadSimple, color: '#2563EB', match: 'exact' },
      { to: '/admin/questions', hash: '#seeding', activeHash: '#seeding', label: 'Seeding Engine', icon: PhTreeStructure, color: '#7C2D12', match: 'exact' },
    ],
  },
  {
    label: 'Quality',
    items: [
      { to: '/admin/questions/review', label: 'Review Queue', icon: PhCheckCircle, color: '#CA8A04', match: 'exact' },
      { to: '/admin/content/coverage', label: 'Coverage & Stats', icon: PhChartLineUp, color: '#166534', match: 'exact' },
      { to: '/admin/quality', label: 'Quality Dashboard', icon: PhMagnifyingGlass, color: '#991B1B', match: 'exact' },
    ],
  },
  {
    label: 'Distribution',
    items: [
      { to: '/admin/remote-updates', label: 'Remote Updates', icon: PhArrowsClockwise, color: '#0369A1', match: 'exact' },
      { to: '/admin/packs', label: 'Packs & Publishing', icon: PhNewspaper, color: '#4D7C0F', match: 'exact' },
    ],
  },
  {
    label: 'System',
    items: [
      { to: '/admin/students', label: 'Students', icon: PhUsers, color: '#0E7490', match: 'prefix' },
      { to: '/admin/users', label: 'Users', icon: PhUsers, color: '#0F766E', match: 'exact' },
      {
        to: '/admin/settings',
        label: 'Settings',
        icon: PhGear,
        color: '#525252',
        match: 'exact',
        excludeHashes: ['#system', '#health', '#backup', '#tuning', '#entitlements'],
      },
      { to: '/admin/settings', hash: '#health', activeHash: '#health', label: 'System Health', icon: PhChartLineUp, color: '#0369A1', match: 'exact' },
      { to: '/admin/settings', hash: '#backup', activeHash: '#backup', label: 'Backup & Restore', icon: PhBooks, color: '#525252', match: 'exact' },
    ],
  },
]
</script>

<template>
  <div class="flex h-screen overflow-hidden" :style="{ backgroundColor: 'var(--bg)' }">
    <aside
      class="w-[var(--sidebar-width)] flex flex-col border-r no-print shrink-0 overflow-y-auto"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }"
    >
      <div class="px-4 py-3 flex items-center gap-2.5 shrink-0" :style="{ borderBottom: '1px solid var(--card-border)' }">
        <div class="w-8 h-8 rounded-[8px] bg-[var(--accent)] flex items-center justify-center shadow-sm">
          <span class="text-white font-display font-bold text-sm">e</span>
        </div>
        <div>
          <p class="font-display font-bold text-sm leading-none" :style="{ color: 'var(--text)' }">eCoach</p>
          <p class="text-[9px] uppercase tracking-widest mt-0.5" :style="{ color: 'var(--text-3)' }">CMS Console</p>
        </div>
      </div>

      <nav class="flex-1 px-2 py-3.5 space-y-5 overflow-x-hidden">
        <div v-for="group in navGroups" :key="group.label">
          <p class="px-2 mb-2 text-[10px] font-bold uppercase tracking-[0.12em]" :style="{ color: 'var(--text-3)' }">
            {{ group.label }}
          </p>
          <div class="space-y-1">
            <RouterLink
              v-for="item in group.items"
              :key="item.hash ? `${item.to}${item.hash}` : item.to"
              :to="item.hash ? { path: item.to, hash: item.hash } : item.to"
              class="admin-nav-link flex items-center gap-2.5 px-2.5 py-2 rounded-[9px] text-sm relative transition-colors"
              :style="{
                '--item-color': item.color,
                backgroundColor: isActive(item) ? item.color + '18' : 'transparent',
                color: isActive(item) ? item.color : 'var(--text-2)',
                fontWeight: isActive(item) ? '600' : '500',
              }"
            >
              <span
                v-if="isActive(item)"
                class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 rounded-r-full"
                :style="{ backgroundColor: item.color }"
              />
              <component
                :is="item.icon"
                :size="18"
                weight="fill"
                class="shrink-0"
                :style="{
                  color: isActive(item) ? item.color : 'var(--text-3)',
                  opacity: isActive(item) ? 1 : 0.65,
                }"
              />
              <span class="truncate text-[14px]">{{ item.label }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <div class="shrink-0 p-2 border-t" :style="{ borderColor: 'var(--card-border)' }">
        <div
          v-if="auth.currentAccount"
          class="flex items-center gap-2 px-2.5 py-1.5 mb-1 rounded-[9px]"
          :style="{ backgroundColor: 'var(--primary-light)' }"
        >
          <div
            class="w-6 h-6 rounded-full flex items-center justify-center text-white text-[10px] font-bold shrink-0"
            style="background: var(--accent);"
          >
            {{ auth.currentAccount.display_name.charAt(0).toUpperCase() }}
          </div>
          <span class="text-xs font-semibold truncate" :style="{ color: 'var(--text-2)' }">
            {{ auth.currentAccount.display_name }}
          </span>
        </div>

        <button
          class="dark-toggle flex items-center gap-2.5 w-full px-2.5 py-2 rounded-[9px] text-sm transition-colors"
          :title="ui.isDark ? 'Light mode' : 'Dark mode'"
          @click="ui.toggleDark()"
        >
          <Transition name="dark-icon" mode="out-in">
            <PhSun v-if="ui.isDark" :key="'sun'" :size="17" weight="fill" style="color: #FBBF24; opacity: 0.8;" />
            <PhMoon v-else :key="'moon'" :size="17" weight="fill" style="color: #94a3b8; opacity: 0.65;" />
          </Transition>
          <span class="text-[14px]" style="color: var(--text-3);">{{ ui.isDark ? 'Light Mode' : 'Dark Mode' }}</span>
        </button>
        <button class="sign-out flex items-center gap-2.5 w-full px-2.5 py-2 rounded-[9px] text-sm transition-colors" @click="logout">
          <PhSignOut :size="17" weight="fill" style="color: #F43F5E; opacity: 0.65;" />
          <span class="text-[14px]" style="color: var(--text-3);">Sign Out</span>
        </button>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto p-4" :style="{ backgroundColor: 'var(--bg)' }">
      <ErrorBoundary>
        <RouterView />
      </ErrorBoundary>
    </main>
  </div>
</template>

<style scoped>
.admin-nav-link:hover {
  background-color: color-mix(in srgb, var(--item-color, #888) 10%, transparent) !important;
  color: var(--item-color, var(--text)) !important;
}
.admin-nav-link:hover :deep(svg) {
  color: var(--item-color, var(--text-3)) !important;
  opacity: 1 !important;
}
.sign-out:hover {
  background-color: rgba(244, 63, 94, 0.08);
}
.sign-out:hover :deep(svg) {
  opacity: 1 !important;
}
.dark-toggle:hover {
  background-color: color-mix(in srgb, var(--text-3) 10%, transparent);
}
.dark-icon-enter-active,
.dark-icon-leave-active { transition: opacity 140ms ease, transform 140ms ease; }
.dark-icon-enter-from { opacity: 0; transform: rotate(-30deg) scale(0.7); }
.dark-icon-leave-to   { opacity: 0; transform: rotate(30deg) scale(0.7); }
</style>
