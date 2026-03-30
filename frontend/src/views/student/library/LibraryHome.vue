<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ipc } from '@/ipc'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppInput from '@/components/ui/AppInput.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const searchQuery = ref('')

const shelves = [
  { key: 'personal', label: 'My Shelf', icon: '📌', count: 12, desc: 'Saved items & bookmarks' },
  { key: 'topics', label: 'Topic Library', icon: '📚', count: 48, desc: 'Browse by subject & topic' },
  { key: 'past-exams', label: 'Past Exam Vault', icon: '📋', count: 30, desc: 'Historical exam papers' },
  { key: 'mistakes', label: 'Mistake Bank', icon: '✕', count: 8, desc: 'Your categorized errors' },
  { key: 'memory', label: 'Memory Shelf', icon: '🧠', count: 5, desc: 'Items due for review' },
  { key: 'teach', label: 'Teach Shelf', icon: '🎓', count: 24, desc: 'Explanations & examples' },
  { key: 'revision', label: 'Revision Packs', icon: '📦', count: 3, desc: 'Custom study packs' },
  { key: 'feed', label: 'Study Feed', icon: '✨', count: 0, desc: 'Smart recommendations' },
]

onMounted(async () => {
  // Would call library IPC here when available
  loading.value = false
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-6">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Library</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Your academic knowledge hub. Everything in one place.</p>
      </div>
      <AppButton variant="secondary" size="sm">📦 Build Revision Pack</AppButton>
    </div>

    <div class="mb-6">
      <AppInput v-model="searchQuery" placeholder="Search library..." />
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
      <AppCard v-for="shelf in shelves" :key="shelf.key" hover padding="md"
        @click="router.push('/student/library')">
        <div class="text-center">
          <div class="text-2xl mb-2">{{ shelf.icon }}</div>
          <h3 class="text-xs font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ shelf.label }}</h3>
          <p class="text-[10px] mb-2" :style="{ color: 'var(--text-3)' }">{{ shelf.desc }}</p>
          <AppBadge v-if="shelf.count" color="muted" size="xs">{{ shelf.count }}</AppBadge>
        </div>
      </AppCard>
    </div>
  </div>
</template>
