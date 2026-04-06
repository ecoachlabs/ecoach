<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const sections = [
  { name: 'curriculum', to: '/admin/curriculum', label: 'Curriculum', desc: 'Manage curriculum structure', symbol: '≡' },
  { name: 'questions', to: '/admin/questions', label: 'Questions', desc: 'Author and review questions', symbol: '?' },
  { name: 'content', to: '/admin/content', label: 'Content', desc: 'Content pipeline and coverage', symbol: '◈' },
  { name: 'students', to: '/admin/students', label: 'Students', desc: 'Monitor student performance', symbol: '◎' },
  { name: 'packs', to: '/admin/packs', label: 'Packs', desc: 'Manage content packs', symbol: '□' },
  { name: 'users', to: '/admin/users', label: 'Users', desc: 'Account management', symbol: '⊕' },
  { name: 'quality', to: '/admin/quality', label: 'Quality', desc: 'Quality metrics and audit', symbol: '✓' },
  { name: 'settings', to: '/admin/settings', label: 'Settings', desc: 'System configuration', symbol: '⚙' },
]
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Admin</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Command Center</h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">System management and content operations</p>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="grid grid-cols-4 gap-3 max-w-4xl">
        <RouterLink
          v-for="section in sections"
          :key="section.name"
          :to="section.to"
          class="section-card flex flex-col items-center text-center p-5 rounded-2xl border"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
        >
          <div class="section-symbol mb-3">{{ section.symbol }}</div>
          <h3 class="text-sm font-bold mb-1" :style="{ color: 'var(--ink)' }">{{ section.label }}</h3>
          <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ section.desc }}</p>
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--ink-muted); margin-bottom: 4px; }

.section-card {
  text-decoration: none;
  cursor: pointer;
  transition: border-color 130ms, transform 130ms;
}
.section-card:hover { transform: translateY(-2px); border-color: var(--ink-muted); }

.section-symbol {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 900;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid var(--border-soft);
}
</style>
