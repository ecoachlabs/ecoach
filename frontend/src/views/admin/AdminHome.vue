<script setup lang="ts">
const sections = [
  { name: 'curriculum', to: '/admin/curriculum', label: 'Curriculum', desc: 'Manage curriculum structure' },
  { name: 'questions', to: '/admin/questions', label: 'Questions', desc: 'Author and review questions' },
  { name: 'content', to: '/admin/content', label: 'Content', desc: 'Content pipeline and coverage' },
  { name: 'students', to: '/admin/students', label: 'Students', desc: 'Monitor student performance' },
  { name: 'packs', to: '/admin/packs', label: 'Packs', desc: 'Manage content packs' },
  { name: 'users', to: '/admin/users', label: 'Users', desc: 'Account management' },
  { name: 'quality', to: '/admin/quality', label: 'Quality', desc: 'Quality metrics and audit' },
  { name: 'settings', to: '/admin/settings', label: 'Settings', desc: 'System configuration' },
]
</script>

<template>
  <div>
    <h1 class="text-lg font-bold mb-4 text-gray-900">Command Center</h1>
    <div class="grid grid-cols-3 gap-3">
      <RouterLink
        v-for="section in sections"
        :key="section.name"
        :to="section.to"
        class="p-4 rounded-lg border border-gray-200 bg-white hover:shadow-sm transition-shadow"
      >
        <h3 class="font-medium text-sm text-gray-900">{{ section.label }}</h3>
        <p class="text-xs mt-1 text-gray-400">{{ section.desc }}</p>
      </RouterLink>
    </div>
  </div>
</template>
