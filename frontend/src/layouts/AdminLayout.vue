<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

onMounted(() => { ui.setTheme('admin') })

function logout() { auth.logout(); router.push('/') }
function isActive(to: string): boolean {
  if (to === '/admin') return route.path === '/admin'
  return route.path.startsWith(to)
}

const navItems = [
  { to: '/admin', label: 'Command Center', icon: '◉' },
  { to: '/admin/curriculum', label: 'Curriculum', icon: '📚' },
  { to: '/admin/questions', label: 'Questions', icon: '❓' },
  { to: '/admin/content', label: 'Content', icon: '📦' },
  { to: '/admin/students', label: 'Students', icon: '👤' },
  { to: '/admin/packs', label: 'Packs', icon: '💾' },
  { to: '/admin/users', label: 'Users', icon: '⚙' },
  { to: '/admin/quality', label: 'Quality', icon: '✓' },
  { to: '/admin/settings', label: 'Settings', icon: '☰' },
]
</script>

<template>
  <div class="flex h-screen overflow-hidden" :style="{ backgroundColor: 'var(--bg)' }">
    <aside class="w-[var(--sidebar-width)] flex flex-col border-r no-print"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }">
      <div class="px-3 py-3 flex items-center gap-2" :style="{ borderBottom: '1px solid var(--card-border)' }">
        <div class="w-7 h-7 rounded-md bg-gradient-to-br from-violet-500 to-purple-700 flex items-center justify-center">
          <span class="text-white font-display font-bold text-[10px]">e</span>
        </div>
        <span class="font-bold text-[11px] uppercase tracking-wider" :style="{ color: 'var(--text)' }">Admin</span>
      </div>

      <nav class="flex-1 px-1.5 py-2 space-y-0.5">
        <RouterLink v-for="item in navItems" :key="item.to" :to="item.to"
          class="flex items-center gap-2 px-2.5 py-1.5 rounded-[var(--radius-sm)] text-xs transition-colors"
          :class="isActive(item.to) ? 'bg-[var(--primary-light)] text-[var(--primary)] font-medium' : 'text-[var(--text-2)] hover:bg-[var(--primary-light)]'">
          <span class="w-4 text-center text-[10px] opacity-60">{{ item.icon }}</span>
          {{ item.label }}
        </RouterLink>
      </nav>

      <div class="shrink-0 p-1.5 border-t" :style="{ borderColor: 'var(--card-border)' }">
        <button class="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-[var(--radius-sm)] text-xs text-[var(--text-3)] hover:text-[var(--danger)] transition-colors"
          @click="logout">
          <span class="w-4 text-center">⏻</span> Sign Out
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
