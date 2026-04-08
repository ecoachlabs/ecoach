<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const loading = ref(false)
const searchQuery = ref('')

const shelves = [
  { key: 'personal', label: 'My Shelf', symbol: '◈', count: 12, desc: 'Saved items & bookmarks' },
  { key: 'topics', label: 'Topic Library', symbol: '≡', count: 48, desc: 'Browse by subject & topic' },
  { key: 'past-exams', label: 'Past Exam Vault', symbol: '▣', count: 30, desc: 'Historical exam papers' },
  { key: 'mistakes', label: 'Mistake Bank', symbol: '✕', count: 8, desc: 'Your categorized errors' },
  { key: 'memory', label: 'Memory Shelf', symbol: '◎', count: 5, desc: 'Items due for review' },
  { key: 'teach', label: 'Teach Shelf', symbol: '△', count: 24, desc: 'Explanations & examples' },
  { key: 'revision', label: 'Revision Packs', symbol: '□', count: 3, desc: 'Custom study packs' },
  { key: 'feed', label: 'Study Feed', symbol: '∿', count: 0, desc: 'Smart recommendations' },
]

const recentItems = [
  { title: 'Fractions & Decimals', type: 'topic', subject: 'Mathematics' },
  { title: 'Photosynthesis', type: 'topic', subject: 'Science' },
  { title: '2024 BECE Paper 1', type: 'exam', subject: 'All' },
  { title: 'Comprehension Errors', type: 'mistake', subject: 'English' },
]

onMounted(() => {
  if (route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    })
  }
})

watch(() => route.hash, (hash) => {
  if (!hash) return
  nextTick(() => {
    document.getElementById(hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  })
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between mb-4">
        <div>
          <p class="eyebrow">Library</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Your Academic Hub
          </h1>
        </div>
        <button class="build-btn">Build Revision Pack</button>
      </div>
      <AppSearchInput
        v-model="searchQuery"
        placeholder="Search topics, formulas, past papers…"
      />
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: shelves -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div>
          <p class="section-label mb-4">Shelves</p>
          <div class="grid grid-cols-4 gap-3">
            <button
              v-for="shelf in shelves"
              :key="shelf.key"
              :id="shelf.key === 'revision' ? 'revision-box' : undefined"
              class="shelf-tile"
              @click="router.push('/student/library')"
            >
              <div class="shelf-symbol">{{ shelf.symbol }}</div>
              <h3 class="text-xs font-bold mb-0.5" :style="{ color: 'var(--ink)' }">{{ shelf.label }}</h3>
              <p class="text-[10px] mb-2" :style="{ color: 'var(--ink-muted)' }">{{ shelf.desc }}</p>
              <span v-if="shelf.count" class="count-badge">{{ shelf.count }}</span>
            </button>
          </div>
        </div>

        <!-- Recent -->
        <div>
          <p class="section-label mb-3">Recently Accessed</p>
          <div class="space-y-2">
            <div
              v-for="item in recentItems"
              :key="item.title"
              class="recent-row flex items-center gap-4 px-4 py-3 rounded-xl border"
              :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
            >
              <div class="item-type-box flex-shrink-0">{{ item.type.charAt(0).toUpperCase() }}</div>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ item.subject }}</p>
              </div>
              <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full border"
                :style="{ borderColor: 'transparent', color: 'var(--ink-secondary)' }">
                {{ item.type }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right sidebar -->
      <div
        class="w-60 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Access</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-1">
          <button class="quick-link w-full text-left" @click="router.push('/student/mistakes')">Mistake Bank</button>
          <button class="quick-link w-full text-left" @click="router.push('/student/memory')">Memory Mode</button>
          <button class="quick-link w-full text-left" @click="router.push('/student/glossary')">Glossary Lab</button>
          <button class="quick-link w-full text-left" @click="router.push('/student/exam-intel')">Exam Intelligence</button>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <div class="text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">
              {{ shelves.reduce((a, s) => a + s.count, 0) }}
            </p>
            <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Total Items</p>
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
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.build-btn {
  padding: 8px 18px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  background: var(--ink);
  color: var(--paper);
  border: none;
  transition: opacity 120ms;
}
.build-btn:hover { opacity: 0.85; }

.shelf-tile {
  position: relative;
  overflow: hidden;
  border-radius: 16px;
  padding: 18px 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  cursor: pointer;
  background: var(--surface);
  border: 1px solid transparent;
  transition: border-color 130ms ease, transform 130ms ease;
}
.shelf-tile:hover {
  transform: translateY(-2px);
  border-color: var(--ink-muted);
}

.shelf-symbol {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 900;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
  margin-bottom: 10px;
}

.count-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

.recent-row {
  transition: background-color 100ms, transform 100ms;
  cursor: pointer;
}
.recent-row:hover { background-color: var(--paper) !important; transform: translateX(2px); }

.item-type-box {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 800;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
}

.quick-link {
  display: flex;
  align-items: center;
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


