<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) selectedSubjectId.value = subjects.value[0].id
  } catch {}
  loading.value = false
})

const entryPoints = [
  { key: 'year', label: 'By Year', desc: 'Browse papers by exam year', symbol: '◫' },
  { key: 'topic', label: 'By Topic', desc: 'See how topics are tested', symbol: '≡' },
  { key: 'patterns', label: 'Patterns', desc: 'Recurring question families', symbol: '∿' },
  { key: 'weakness', label: 'My Weaknesses', desc: 'Your weak exam patterns', symbol: '⚠' },
  { key: 'replay', label: 'Exam Replay', desc: 'Sit a real past paper', symbol: '▶' },
  { key: 'predict', label: 'Likely Next', desc: 'Predicted question styles', symbol: '◈' },
  { key: 'atlas', label: 'Family Atlas', desc: 'Visual question network', symbol: '⬡' },
]
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between">
        <div>
          <p class="eyebrow">Intelligence</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Exam Intelligence
          </h1>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
            Past questions are signals — discover the patterns
          </p>
        </div>
        <!-- Subject filter -->
        <div v-if="subjects.length" class="flex gap-1.5">
          <button
            v-for="s in subjects"
            :key="s.id"
            class="subj-tab"
            :class="{ active: selectedSubjectId === s.id }"
            @click="selectedSubjectId = s.id"
          >{{ s.name }}</button>
        </div>
      </div>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: entry points -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div>
          <p class="section-label mb-4">Explore</p>
          <div class="grid grid-cols-3 gap-3">
            <button
              v-for="ep in entryPoints"
              :key="ep.key"
              class="entry-card"
            >
              <div class="entry-symbol">{{ ep.symbol }}</div>
              <p class="text-sm font-bold mb-0.5" :style="{ color: 'var(--ink)' }">{{ ep.label }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ ep.desc }}</p>
            </button>
          </div>
        </div>
      </div>

      <!-- Right: quick insight -->
      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Insight</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div class="insight-card">
            <p class="text-xs font-bold mb-2" :style="{ color: 'var(--ink)' }">Fractions Frequency</p>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              Fractions have appeared in 9 of the last 10 BECE papers — your most at-risk recurring pattern.
            </p>
            <div class="mt-3 flex items-center gap-2">
              <div class="h-1.5 rounded-full flex-1 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full w-[90%]" :style="{ backgroundColor: 'var(--warm)' }" />
              </div>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--warm)' }">90%</span>
            </div>
          </div>

          <div class="insight-card">
            <p class="text-xs font-bold mb-2" :style="{ color: 'var(--ink)' }">Algebra Patterns</p>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              Linear equations appear every year. Quadratics introduced last 3 years.
            </p>
            <div class="mt-3 flex items-center gap-2">
              <div class="h-1.5 rounded-full flex-1 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full w-[100%]" :style="{ backgroundColor: 'var(--gold)' }" />
              </div>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--gold)' }">100%</span>
            </div>
          </div>

          <div class="insight-card">
            <p class="text-xs font-bold mb-2" :style="{ color: 'var(--ink)' }">Predicted High-Value</p>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              Based on patterns, Measurement & Statistics are likely for this year's paper.
            </p>
          </div>
        </div>
        <div class="p-4 border-t" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full insight-btn">View Full Analysis →</button>
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

.subj-tab {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--border-soft);
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}
.subj-tab.active, .subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.entry-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px 14px;
  border-radius: 16px;
  border: 1px solid var(--border-soft);
  background: var(--surface);
  cursor: pointer;
  transition: border-color 130ms, transform 130ms;
}
.entry-card:hover { transform: translateY(-2px); border-color: var(--ink-muted); }

.entry-symbol {
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
  margin-bottom: 10px;
}

.insight-card {
  padding: 14px;
  border-radius: 14px;
  border: 1px solid var(--border-soft);
  background: var(--paper);
}

.insight-btn {
  padding: 9px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.insight-btn:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
</style>
