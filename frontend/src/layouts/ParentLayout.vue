<script setup lang="ts">
import { onMounted } from 'vue'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

onMounted(() => {
  ui.setTheme('parent')
})
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <aside class="w-60 flex flex-col border-r" :style="{ borderColor: 'var(--border)', backgroundColor: 'var(--surface)' }">
      <div class="p-5 font-semibold text-xl" :style="{ color: 'var(--primary)' }">eCoach</div>
      <nav class="flex-1 px-3 space-y-1">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="item.to"
          class="flex items-center gap-3 px-4 py-3 rounded-lg transition-colors"
          :style="{ color: 'var(--text-secondary)', fontSize: 'var(--font-size-base)' }"
          active-class="!bg-[var(--info-light)] !text-[var(--info)]"
        >
          {{ item.label }}
        </RouterLink>
      </nav>
    </aside>
    <main class="flex-1 overflow-y-auto p-8" :style="{ backgroundColor: 'var(--bg)' }">
      <RouterView />
    </main>
  </div>
</template>

<script lang="ts">
const navItems = [
  { name: 'home', to: '/parent', label: 'Home' },
  { name: 'attention', to: '/parent/attention', label: 'Attention Needed' },
  { name: 'performance', to: '/parent/performance', label: 'Performance' },
  { name: 'reports', to: '/parent/reports', label: 'Reports' },
  { name: 'settings', to: '/parent/settings', label: 'Settings' },
]
</script>
