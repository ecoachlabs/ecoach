<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { searchGlossary } from '@/ipc/library'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'
import GlossaryEntryCard from '@/components/glossary/GlossaryEntryCard.vue'
import GlossaryTestLab from '@/components/glossary/GlossaryTestLab.vue'
import AudioRadioMode from '@/components/glossary/AudioRadioMode.vue'

const auth = useAuthStore()
const router = useRouter()
const searchQuery = ref('')
const searchResults = ref<any[]>([])
const searching = ref(false)
const showTestLab = ref(false)
const showRadio = ref(false)

async function search() {
  if (!searchQuery.value.trim()) return
  searching.value = true
  try {
    searchResults.value = await searchGlossary(searchQuery.value)
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
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Glossary Lab</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Definitions, formulas, and key concepts — searchable, learnable, testable.</p>

    <!-- Search -->
    <div class="mb-6">
      <AppSearchInput v-model="searchQuery" placeholder="Search any term, formula, or concept..." :loading="searching" @search="search" />
    </div>

    <!-- Search Results -->
    <div v-if="searchResults.length" class="mb-6 space-y-2">
      <GlossaryEntryCard v-for="r in searchResults" :key="r.id" :entry-id="r.id" :title="r.title" :type="r.type"
        :quick-meaning="r.quickMeaning" @click="router.push('/student/glossary/entry/' + r.id)" />
    </div>

    <!-- Content Classes -->
    <div v-if="!searchResults.length" class="grid grid-cols-3 gap-3 mb-6">
      <AppCard v-for="s in sections" :key="s.key" hover padding="lg" class="text-center">
        <div class="text-3xl mb-3">{{ s.icon }}</div>
        <h3 class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ s.label }}</h3>
        <p class="text-[11px] mb-2" :style="{ color: 'var(--text-3)' }">{{ s.desc }}</p>
        <AppBadge color="muted" size="xs">{{ s.count }} entries</AppBadge>
      </AppCard>
    </div>

    <!-- Quick Actions -->
    <div class="flex flex-wrap gap-2 mb-6">
      <AppButton variant="secondary" size="sm" @click="showRadio = !showRadio">🎧 Audio Radio</AppButton>
      <AppButton variant="secondary" size="sm" @click="showTestLab = !showTestLab">✎ Test Lab</AppButton>
      <AppButton variant="secondary" size="sm" @click="router.push('/student/glossary/formula-lab')">∑ Formula Lab</AppButton>
      <AppButton variant="secondary" size="sm" @click="router.push('/student/glossary/compare')">⟺ Compare</AppButton>
    </div>

    <!-- Audio Radio Mode (toggle) -->
    <div v-if="showRadio" class="mb-6">
      <AudioRadioMode @select-station="(s) => { /* start playback */ }" />
    </div>

    <!-- Test Lab (toggle) -->
    <div v-if="showTestLab" class="mb-6">
      <GlossaryTestLab @start-test="(mode) => { /* start test */ }" />
    </div>

    <!-- Daily Term Drill -->
    <AppCard v-if="!showTestLab && !showRadio && !searchResults.length" padding="md" glow="gold">
      <div class="flex items-center gap-4">
        <div class="w-12 h-12 rounded-xl flex items-center justify-center text-xl" :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">📝</div>
        <div class="flex-1">
          <p class="text-xs font-medium uppercase" :style="{ color: 'var(--gold)' }">Daily Term Drill</p>
          <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">5 terms to review today</p>
        </div>
        <AppButton variant="warm" size="sm">Start Drill</AppButton>
      </div>
    </AppCard>
  </div>
</template>
