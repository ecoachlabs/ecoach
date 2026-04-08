<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { searchGlossary } from '@/ipc/library'
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
  { key: 'definitions', label: 'Definitions', desc: 'What things mean', symbol: 'D', count: 120 },
  { key: 'formulas', label: 'Formulas', desc: 'How things are calculated', symbol: '∑', count: 45 },
  { key: 'concepts', label: 'Key Concepts', desc: 'Ideas you must understand', symbol: 'C', count: 78 },
]

const tools = [
  { label: 'Audio Radio', desc: 'Listen & learn on loop', action: () => { showRadio.value = !showRadio.value } },
  { label: 'Test Lab', desc: 'Self-test with flashcards', action: () => { showTestLab.value = !showTestLab.value } },
  { label: 'Formula Lab', desc: 'Practice formula recall', action: () => router.push('/student/glossary/formula-lab') },
  { label: 'Compare', desc: 'Side-by-side term analysis', action: () => router.push('/student/glossary/compare') },
]
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header + search -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between mb-4">
        <div>
          <p class="eyebrow">Glossary Lab</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Definitions. Formulas. Concepts.
          </h1>
        </div>
        <div class="flex items-center gap-3 px-4 py-2.5 rounded-xl border"
          :style="{ borderColor: 'transparent' }">
          <p class="text-[10px] font-bold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Daily Drill</p>
          <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">5 terms due today</p>
          <button class="px-3 py-1 rounded-full text-xs font-bold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }">Start</button>
        </div>
      </div>
      <AppSearchInput
        v-model="searchQuery"
        placeholder="Search any term, formula, or concept…"
        :loading="searching"
        @search="search"
      />
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: sections + tools + results -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">

        <!-- Search results -->
        <div v-if="searchResults.length" class="space-y-2">
          <p class="section-label mb-2">Search Results ({{ searchResults.length }})</p>
          <GlossaryEntryCard
            v-for="r in searchResults"
            :key="r.id"
            :entry-id="r.id"
            :title="r.title"
            :type="r.type"
            :quick-meaning="r.quickMeaning"
            @click="router.push('/student/glossary/entry/' + r.id)"
          />
        </div>

        <!-- Default content -->
        <template v-else>
          <!-- Section tiles -->
          <div>
            <p class="section-label mb-3">Browse by Type</p>
            <div class="grid grid-cols-3 gap-4">
              <button
                v-for="s in sections"
                :key="s.key"
                class="section-tile"
              >
                <div class="tile-symbol">{{ s.symbol }}</div>
                <h3 class="text-sm font-bold mb-0.5" :style="{ color: 'var(--ink)' }">{{ s.label }}</h3>
                <p class="text-[11px] mb-3" :style="{ color: 'var(--ink-muted)' }">{{ s.desc }}</p>
                <span class="count-badge">{{ s.count }} entries</span>
              </button>
            </div>
          </div>

          <!-- Tools -->
          <div>
            <p class="section-label mb-3">Learning Tools</p>
            <div class="grid grid-cols-2 gap-3">
              <button
                v-for="tool in tools"
                :key="tool.label"
                class="tool-card"
                @click="tool.action()"
              >
                <div class="text-left">
                  <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ tool.label }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ tool.desc }}</p>
                </div>
              </button>
            </div>
          </div>

          <AudioRadioMode v-if="showRadio" @select-station="() => {}" />
          <GlossaryTestLab v-if="showTestLab" @start-test="() => {}" />
        </template>
      </div>

      <!-- Right: quick access -->
      <div
        class="w-60 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Access</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-1">
          <button class="quick-link w-full text-left" @click="router.push('/student/glossary/audio')">
            Audio Glossary
          </button>
          <button class="quick-link w-full text-left" @click="router.push('/student/library')">
            Full Library
          </button>
          <button class="quick-link w-full text-left" @click="router.push('/student/memory')">
            Memory Mode
          </button>
        </div>

        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <div class="text-center">
            <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">243</p>
            <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Total Entries</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.section-tile {
  position: relative;
  overflow: hidden;
  border-radius: 16px;
  padding: 20px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid transparent;
  transition: border-color 130ms ease, transform 130ms ease;
}
.section-tile:hover {
  transform: translateY(-2px);
  border-color: var(--ink-muted);
}

.tile-symbol {
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
  border: 1px solid transparent;
  margin-bottom: 12px;
}

.count-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

.tool-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: var(--surface);
  cursor: pointer;
  transition: border-color 120ms, background-color 120ms;
}
.tool-card:hover {
  border-color: var(--ink-muted);
  background: var(--paper);
}

.quick-link {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-secondary);
  cursor: pointer;
  transition: background-color 100ms;
}
.quick-link:hover { background-color: var(--paper); color: var(--ink); }
</style>


