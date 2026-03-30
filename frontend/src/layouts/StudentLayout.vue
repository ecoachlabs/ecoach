<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useAuthStore } from '@/stores/auth'

const ui = useUiStore()
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()
const collapsed = ref(false)

onMounted(() => {
  ui.setTheme('student')
})

const navGroups = [
  {
    label: 'Core',
    items: [
      { name: 'student-home', to: '/student', icon: '◉', label: 'Home' },
      { name: 'practice', to: '/student/practice', icon: '✎', label: 'Practice' },
      { name: 'diagnostic', to: '/student/diagnostic', icon: '◈', label: 'Diagnostic' },
      { name: 'mock-home', to: '/student/mock', icon: '⊞', label: 'Mock Centre' },
    ],
  },
  {
    label: 'Learn',
    items: [
      { name: 'journey', to: '/student/journey', icon: '⟶', label: 'Journey' },
      { name: 'beat-yesterday', to: '/student/beat-yesterday', icon: '△', label: 'Beat Yesterday' },
      { name: 'elite-home', to: '/student/elite', icon: '◆', label: 'Elite' },
      { name: 'rise', to: '/student/rise', icon: '↑', label: 'Rise' },
      { name: 'knowledge-gap', to: '/student/knowledge-gap', icon: '◌', label: 'Knowledge Gap' },
      { name: 'memory', to: '/student/memory', icon: '∞', label: 'Memory' },
    ],
  },
  {
    label: 'Resources',
    items: [
      { name: 'glossary', to: '/student/glossary', icon: '⊿', label: 'Glossary' },
      { name: 'library', to: '/student/library', icon: '☰', label: 'Library' },
      { name: 'exam-intel', to: '/student/exam-intel', icon: '◎', label: 'Exam Intel' },
      { name: 'games', to: '/student/games', icon: '▣', label: 'Games' },
    ],
  },
  {
    label: 'Track',
    items: [
      { name: 'progress', to: '/student/progress', icon: '◐', label: 'Progress' },
      { name: 'mistakes', to: '/student/mistakes', icon: '✕', label: 'Mistakes' },
      { name: 'calendar', to: '/student/calendar', icon: '▦', label: 'Calendar' },
    ],
  },
]

function isActive(to: string): boolean {
  if (to === '/student') return route.path === '/student'
  return route.path.startsWith(to)
}

function logout() {
  auth.logout()
  router.push('/')
}
</script>

<template>
  <div class="flex h-screen overflow-hidden" :style="{ backgroundColor: 'var(--bg)' }">

    <!-- Sidebar -->
    <aside
      class="flex flex-col border-r transition-all overflow-y-auto overflow-x-hidden no-print"
      :class="collapsed ? 'w-[60px]' : 'w-[var(--sidebar-width)]'"
      :style="{
        backgroundColor: 'var(--surface)',
        borderColor: 'var(--border-soft)',
        transitionDuration: 'var(--dur-slow)',
        transitionTimingFunction: 'var(--ease-spring)',
      }"
    >
      <!-- Logo bar -->
      <div class="flex items-center gap-3 px-4 h-14 shrink-0" :style="{ borderBottom: '1px solid var(--border-soft)' }">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-teal-500 to-emerald-600 flex items-center justify-center flex-shrink-0">
          <span class="text-white font-display font-bold text-sm">e</span>
        </div>
        <span v-if="!collapsed" class="font-display font-bold text-sm" :style="{ color: 'var(--text)' }">eCoach</span>
        <button
          v-if="!collapsed"
          class="ml-auto w-6 h-6 rounded flex items-center justify-center text-[var(--text-3)] hover:bg-[var(--primary-light)] transition-colors"
          @click="collapsed = true"
        >
          ‹
        </button>
        <button
          v-else
          class="w-6 h-6 rounded flex items-center justify-center text-[var(--text-3)] hover:bg-[var(--primary-light)] transition-colors"
          @click="collapsed = false"
        >
          ›
        </button>
      </div>

      <!-- Nav groups -->
      <nav class="flex-1 py-2 space-y-4">
        <div v-for="group in navGroups" :key="group.label">
          <p v-if="!collapsed" class="px-4 mb-1 text-[10px] font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">
            {{ group.label }}
          </p>
          <div class="space-y-0.5 px-2">
            <RouterLink
              v-for="item in group.items"
              :key="item.name"
              :to="item.to"
              class="flex items-center gap-2.5 px-2.5 py-2 rounded-[var(--radius-md)] text-sm transition-all group/nav"
              :class="isActive(item.to)
                ? 'bg-[var(--primary-light)] text-[var(--primary)] font-medium'
                : 'text-[var(--text-2)] hover:bg-[var(--primary-light)] hover:text-[var(--primary)]'"
              :style="{ transitionDuration: 'var(--dur-fast)' }"
              :title="collapsed ? item.label : undefined"
            >
              <span class="w-5 text-center text-base flex-shrink-0 opacity-70 group-hover/nav:opacity-100 transition-opacity">{{ item.icon }}</span>
              <span v-if="!collapsed" class="truncate">{{ item.label }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Bottom -->
      <div class="shrink-0 p-2 border-t" :style="{ borderColor: 'var(--border-soft)' }">
        <button
          class="flex items-center gap-2.5 w-full px-2.5 py-2 rounded-[var(--radius-md)] text-sm text-[var(--text-3)] hover:bg-[var(--danger-light)] hover:text-[var(--danger)] transition-colors"
          @click="logout"
        >
          <span class="w-5 text-center">⏻</span>
          <span v-if="!collapsed">Sign Out</span>
        </button>
      </div>
    </aside>

    <!-- Main -->
    <main class="flex-1 overflow-y-auto" :style="{ backgroundColor: 'var(--bg)' }">
      <RouterView />
    </main>
  </div>
</template>
