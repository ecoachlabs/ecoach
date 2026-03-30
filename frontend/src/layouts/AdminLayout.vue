<script setup lang="ts">
import { onMounted } from 'vue'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

onMounted(() => {
  ui.setTheme('admin')
})
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <aside class="w-56 flex flex-col border-r" :style="{ borderColor: 'var(--border)', backgroundColor: 'var(--surface)' }">
      <div class="p-3 font-bold text-sm tracking-wide uppercase" :style="{ color: 'var(--primary)' }">eCoach Admin</div>
      <nav class="flex-1 px-2 space-y-0.5 text-sm">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="item.to"
          class="block px-3 py-1.5 rounded transition-colors"
          :style="{ color: 'var(--text-secondary)' }"
          active-class="!bg-[var(--info-light)] !text-[var(--info)]"
        >
          {{ item.label }}
        </RouterLink>
      </nav>
    </aside>
    <main class="flex-1 overflow-y-auto p-4" :style="{ backgroundColor: 'var(--bg)' }">
      <RouterView />
    </main>
  </div>
</template>

<script lang="ts">
const navItems = [
  { name: 'home', to: '/admin', label: 'Command Center' },
  { name: 'curriculum', to: '/admin/curriculum', label: 'Curriculum' },
  { name: 'questions', to: '/admin/questions', label: 'Questions' },
  { name: 'content', to: '/admin/content', label: 'Content' },
  { name: 'students', to: '/admin/students', label: 'Students' },
  { name: 'packs', to: '/admin/packs', label: 'Packs' },
  { name: 'users', to: '/admin/users', label: 'Users' },
  { name: 'quality', to: '/admin/quality', label: 'Quality' },
  { name: 'settings', to: '/admin/settings', label: 'Settings' },
]
</script>
