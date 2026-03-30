<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { ipc } from '@/ipc'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppInput from '@/components/ui/AppInput.vue'

const router = useRouter()
const searchQuery = ref('')
const searchResults = ref<any[]>([])
const searching = ref(false)

async function search() {
  if (!searchQuery.value.trim()) return
  searching.value = true
  try {
    searchResults.value = await ipc('search_glossary', { query: searchQuery.value, type: 'all' })
  } catch {
    searchResults.value = []
  }
  searching.value = false
}

const sections = [
  { key: 'definitions', label: 'Definitions', desc: 'What things mean', icon: '📖', count: 120 },
  { key: 'formulas', label: 'Formulas', desc: 'How things are calculated', icon: '∑', count: 45 },
  { key: 'concepts', label: 'Key Concepts', desc: 'Ideas you must understand', icon: '💡', count: 78 },
]

const quickActions = [
  { label: 'Audio Radio', to: '/student/glossary/audio', icon: '🎧' },
  { label: 'Test Lab', to: '/student/glossary/test-lab', icon: '✎' },
  { label: 'My Weak Terms', to: '/student/glossary/weak', icon: '⚠' },
  { label: 'Exam Hotspots', to: '/student/glossary/hotspots', icon: '🔥' },
]
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Glossary Lab</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Definitions, formulas, and key concepts — searchable, learnable, testable.</p>

    <!-- Search -->
    <div class="mb-8">
      <AppInput v-model="searchQuery" placeholder="Search any term, formula, or concept..." @keydown.enter="search" />
    </div>

    <!-- Content Classes -->
    <div class="grid grid-cols-3 gap-3 mb-8">
      <AppCard v-for="s in sections" :key="s.key" hover padding="lg" class="text-center">
        <div class="text-3xl mb-3">{{ s.icon }}</div>
        <h3 class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ s.label }}</h3>
        <p class="text-[11px] mb-2" :style="{ color: 'var(--text-3)' }">{{ s.desc }}</p>
        <AppBadge color="muted" size="xs">{{ s.count }} entries</AppBadge>
      </AppCard>
    </div>

    <!-- Quick Actions -->
    <div class="flex flex-wrap gap-2 mb-8">
      <AppButton v-for="a in quickActions" :key="a.label" variant="secondary" size="sm"
        @click="router.push(a.to)">{{ a.icon }} {{ a.label }}</AppButton>
    </div>

    <!-- Daily Term -->
    <AppCard padding="md" glow="gold">
      <div class="flex items-center gap-4">
        <div class="w-12 h-12 rounded-xl flex items-center justify-center text-xl"
          :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">📝</div>
        <div class="flex-1">
          <p class="text-xs font-medium uppercase" :style="{ color: 'var(--gold)' }">Daily Term Drill</p>
          <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">5 terms to review today</p>
        </div>
        <AppButton variant="warm" size="sm">Start Drill</AppButton>
      </div>
    </AppCard>
  </div>
</template>
