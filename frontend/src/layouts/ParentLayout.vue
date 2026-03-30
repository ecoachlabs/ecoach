<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'
import ErrorBoundary from '@/components/layout/ErrorBoundary.vue'

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()

onMounted(() => {
  ui.setTheme('parent')
})

function logout() {
  auth.logout()
  router.push('/')
}

const navItems = [
  { name: 'home', to: '/parent', label: 'Home', icon: '◉' },
  { name: 'attention', to: '/parent/attention', label: 'Attention Needed', icon: '⚠' },
  { name: 'performance', to: '/parent/performance', label: 'Performance', icon: '◐' },
  { name: 'reports', to: '/parent/reports', label: 'Reports', icon: '☰' },
  { name: 'intervention', to: '/parent/intervention', label: 'Intervention', icon: '🛟' },
  { name: 'settings', to: '/parent/settings', label: 'Settings', icon: '⚙' },
]
</script>

<template>
  <div class="flex h-screen overflow-hidden" :style="{ backgroundColor: 'var(--bg)' }">
    <!-- Sidebar (premium, spacious) -->
    <aside class="w-[var(--sidebar-width)] flex flex-col border-r no-print"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }">
      <div class="px-5 py-4 flex items-center gap-3" :style="{ borderBottom: '1px solid var(--card-border)' }">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-slate-600 to-slate-800 flex items-center justify-center flex-shrink-0">
          <span class="text-white font-display font-bold text-sm">e</span>
        </div>
        <span class="font-display font-bold text-sm" :style="{ color: 'var(--text)' }">eCoach</span>
      </div>

      <nav class="flex-1 px-3 py-3 space-y-0.5">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="item.to"
          class="flex items-center gap-3 px-4 py-2.5 rounded-[var(--radius-md)] transition-colors text-sm"
          :style="{ color: 'var(--text-2)' }"
          active-class="!bg-[var(--primary-light)] !text-[var(--primary)] font-medium"
        >
          <span class="w-5 text-center opacity-60">{{ item.icon }}</span>
          {{ item.label }}
        </RouterLink>
      </nav>

      <div class="shrink-0 p-3 border-t" :style="{ borderColor: 'var(--card-border)' }">
        <button class="flex items-center gap-2.5 w-full px-4 py-2 rounded-[var(--radius-md)] text-sm transition-colors"
          :style="{ color: 'var(--text-3)' }"
          @click="logout">
          <span class="w-5 text-center">⏻</span> Sign Out
        </button>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto p-8" :style="{ backgroundColor: 'var(--bg)' }">
      <ErrorBoundary>
        <RouterView />
      </ErrorBoundary>
    </main>
  </div>
</template>
