<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { getLibraryHome } from '@/ipc/library'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import LibrarySearch from './LibrarySearch.vue'
import LibraryShelf from './LibraryShelf.vue'
import StudyFeed from './StudyFeed.vue'

const auth = useAuthStore()
const loading = ref(true)
const home = ref<any>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try { home.value = await getLibraryHome(auth.currentAccount.id) } catch {}
  loading.value = false
})

const shelves = [
  { key: 'personal', title: 'My Shelf', icon: '📌', items: [] },
  { key: 'topics', title: 'Topic Library', icon: '📚', items: [] },
  { key: 'past-exams', title: 'Past Exam Vault', icon: '📋', items: [] },
  { key: 'mistakes', title: 'Mistake Bank', icon: '✕', items: [] },
  { key: 'memory', title: 'Memory Shelf', icon: '🧠', items: [] },
  { key: 'teach', title: 'Teach Shelf', icon: '🎓', items: [] },
  { key: 'revision', title: 'Revision Packs', icon: '📦', items: [] },
  { key: 'feed', title: 'Study Feed', icon: '✨', items: [] },
]
</script>

<template>
  <div>
    <LibrarySearch :results="[]" :loading="false" class="mb-6" />

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-24 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else>
      <LibraryShelf v-for="shelf in shelves.slice(0, 4)" :key="shelf.key"
        :title="shelf.title" :icon="shelf.icon" :items="shelf.items" show-count />

      <StudyFeed :items="[]" class="mt-4" />
    </template>
  </div>
</template>
