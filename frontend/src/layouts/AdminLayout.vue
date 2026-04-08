<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'
import {
  PhBook,
  PhBooks,
  PhChartLineUp,
  PhGear,
  PhGraduationCap,
  PhHouseLine,
  PhMagnifyingGlass,
  PhNewspaper,
  PhPencilSimple,
  PhSignOut,
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
      { to: '/admin', label: 'Command Center', icon: PhHouseLine, color: '#7C3AED', match: 'exact' },
      { to: '/admin/students', label: 'Students', icon: PhUsers, color: '#0EA5E9', match: 'prefix' },
      { to: '/admin/users', label: 'Users', icon: PhUsers, color: '#14B8A6', match: 'exact' },
    ],
  },
  {
    label: 'Curriculum',
    items: [
      { to: '/admin/curriculum', label: 'Curriculum Hub', icon: PhGraduationCap, color: '#10B981', match: 'exact' },
      { to: '/admin/curriculum/upload', label: 'Upload Sources', icon: PhBook, color: '#22C55E', match: 'exact' },
      { to: '/admin/curriculum/review', label: 'Review Extractions', icon: PhMagnifyingGlass, color: '#16A34A', match: 'exact' },
      { to: '/admin/curriculum/editor', label: 'Tree Editor', icon: PhPencilSimple, color: '#15803D', match: 'exact' },
    ],
  },
  {
    label: 'Questions',
    items: [
      { to: '/admin/questions', label: 'Question Hub', icon: PhBook, color: '#F59E0B', match: 'exact' },
      { to: '/admin/questions/author', label: 'Question Author', icon: PhPencilSimple, color: '#FB923C', match: 'exact' },
      { to: '/admin/questions/review', label: 'Review Queue', icon: PhMagnifyingGlass, color: '#F97316', match: 'exact' },
    ],
  },
  {
    label: 'Content',
    items: [
      { to: '/admin/content', label: 'Content Pipeline', icon: PhBooks, color: '#6366F1', match: 'exact' },
      { to: '/admin/content/coverage', label: 'Coverage Heatmap', icon: PhChartLineUp, color: '#4F46E5', match: 'exact' },
      { to: '/admin/packs', label: 'Packs', icon: PhNewspaper, color: '#8B5CF6', match: 'exact' },
    ],
  },
  {
    label: 'Governance',
    items: [
      { to: '/admin/quality', label: 'Quality', icon: PhChartLineUp, color: '#EF4444', match: 'exact' },
      {
        to: '/admin/settings',
        label: 'Settings Hub',
        icon: PhGear,
        color: '#64748B',
        match: 'exact',
        excludeHashes: ['#system', '#health', '#backup', '#tuning', '#entitlements'],
      },
      { to: '/admin/settings', hash: '#system', activeHash: '#system', label: 'System Config', icon: PhGear, color: '#475569', match: 'exact' },
      { to: '/admin/settings', hash: '#health', activeHash: '#health', label: 'System Health', icon: PhChartLineUp, color: '#0EA5E9', match: 'exact' },
      { to: '/admin/settings', hash: '#backup', activeHash: '#backup', label: 'Backup & Restore', icon: PhBooks, color: '#8B5CF6', match: 'exact' },
      { to: '/admin/settings', hash: '#tuning', activeHash: '#tuning', label: 'Coach Tuning', icon: PhPencilSimple, color: '#F59E0B', match: 'exact' },
      { to: '/admin/settings', hash: '#entitlements', activeHash: '#entitlements', label: 'Entitlements', icon: PhUsers, color: '#14B8A6', match: 'exact' },
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
        <div class="w-8 h-8 rounded-[10px] bg-gradient-to-br from-violet-500 to-purple-700 flex items-center justify-center shadow-sm">
          <span class="text-white font-display font-bold text-sm">e</span>
        </div>
        <div>
          <p class="font-display font-bold text-sm leading-none" :style="{ color: 'var(--text)' }">eCoach</p>
          <p class="text-[9px] uppercase tracking-widest mt-0.5" :style="{ color: 'var(--text-3)' }">Admin Portal</p>
        </div>
      </div>

      <nav class="flex-1 px-2 py-3 space-y-4 overflow-x-hidden">
        <div v-for="group in navGroups" :key="group.label">
          <p class="px-2 mb-1.5 text-[9px] font-bold uppercase tracking-[0.12em]" :style="{ color: 'var(--text-3)' }">
            {{ group.label }}
          </p>
          <div class="space-y-px">
            <RouterLink
              v-for="item in group.items"
              :key="item.hash ? `${item.to}${item.hash}` : item.to"
              :to="item.hash ? { path: item.to, hash: item.hash } : item.to"
              class="admin-nav-link flex items-center gap-2.5 px-2.5 py-[7px] rounded-[9px] text-sm relative transition-colors"
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
                :size="17"
                weight="fill"
                class="shrink-0"
                :style="{
                  color: isActive(item) ? item.color : 'var(--text-3)',
                  opacity: isActive(item) ? 1 : 0.65,
                }"
              />
              <span class="truncate text-[13px]">{{ item.label }}</span>
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
            style="background: linear-gradient(135deg, #7C3AED, #4338CA);"
          >
            {{ auth.currentAccount.display_name.charAt(0).toUpperCase() }}
          </div>
          <span class="text-xs font-semibold truncate" :style="{ color: 'var(--text-2)' }">
            {{ auth.currentAccount.display_name }}
          </span>
        </div>

        <button class="sign-out flex items-center gap-2.5 w-full px-2.5 py-[7px] rounded-[9px] text-sm transition-colors" @click="logout">
          <PhSignOut :size="16" weight="fill" style="color: #F43F5E; opacity: 0.65;" />
          <span class="text-[13px]" style="color: var(--text-3);">Sign Out</span>
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
</style>
