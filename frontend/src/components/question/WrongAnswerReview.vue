<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import MathText from './MathText.vue'

defineProps<{
  selectedText: string
  whyItLookedReasonable?: string
  whyItFails?: string
  whyCorrectWins?: string
  correctText?: string
  otherOptionsExplained?: { label: string; text: string; reason: string }[]
  mistakeType?: string
  likelyThinking?: string
  theLesson?: string
  patternHistory?: string
  repairAction?: string
}>()

const expanded = ref<Set<string>>(new Set())

function toggle(key: string) {
  if (expanded.value.has(key)) expanded.value.delete(key)
  else expanded.value.add(key)
}

const layers = [
  { key: 'chose', label: '1. What you chose', prop: 'selectedText' },
  { key: 'reasonable', label: '2. Why it looked reasonable', prop: 'whyItLookedReasonable' },
  { key: 'fails', label: '3. Why it fails', prop: 'whyItFails' },
  { key: 'correct', label: '4. Why the correct answer wins', prop: 'whyCorrectWins' },
  { key: 'others', label: '5. Why other options fail', prop: 'otherOptionsExplained' },
  { key: 'type', label: '6. Mistake type', prop: 'mistakeType' },
  { key: 'thinking', label: '7. Likely thinking pattern', prop: 'likelyThinking' },
  { key: 'lesson', label: '8. The lesson', prop: 'theLesson' },
  { key: 'pattern', label: '9. Pattern history', prop: 'patternHistory' },
  { key: 'repair', label: '10. Repair action', prop: 'repairAction' },
]
</script>

<template>
  <div class="space-y-2 reveal-stagger">
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
      Deep Wrong Answer Review
    </h3>

    <AppCard
      v-for="layer in layers"
      :key="layer.key"
      padding="none"
      class="overflow-hidden"
    >
      <button
        class="w-full px-4 py-3 flex items-center gap-2 text-left transition-colors"
        :style="{ backgroundColor: expanded.has(layer.key) ? 'var(--primary-light)' : 'var(--card-bg)' }"
        @click="toggle(layer.key)"
      >
        <span class="text-xs" :style="{ color: expanded.has(layer.key) ? 'var(--accent)' : 'var(--text-3)' }">
          {{ expanded.has(layer.key) ? '▾' : '▸' }}
        </span>
        <span class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ layer.label }}</span>
      </button>

      <div v-if="expanded.has(layer.key)" class="px-4 pb-3">
        <!-- Special rendering for "what you chose" -->
        <div v-if="layer.key === 'chose'" class="text-sm p-3 rounded-[var(--radius-sm)] bg-red-50 text-red-700">
          <MathText :text="selectedText" size="sm" />
        </div>
        <!-- Correct answer -->
        <div v-else-if="layer.key === 'correct' && correctText" class="text-sm p-3 rounded-[var(--radius-sm)] bg-emerald-50 text-emerald-700">
          <MathText :text="correctText" size="sm" /><br/>
          <span v-if="whyCorrectWins" class="text-xs mt-1 block opacity-80">
            <MathText :text="whyCorrectWins" size="sm" />
          </span>
        </div>
        <!-- Mistake type badge -->
        <div v-else-if="layer.key === 'type' && mistakeType">
          <AppBadge color="warm" size="md">{{ mistakeType }}</AppBadge>
        </div>
        <!-- Generic text layers -->
        <p v-else class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">
          <MathText :text="(layer.prop === 'whyItLookedReasonable' ? whyItLookedReasonable :
              layer.prop === 'whyItFails' ? whyItFails :
              layer.prop === 'likelyThinking' ? likelyThinking :
              layer.prop === 'theLesson' ? theLesson :
              layer.prop === 'patternHistory' ? patternHistory :
              layer.prop === 'repairAction' ? repairAction : '') || 'Analysis will be available after more data is collected.'" size="sm" />
        </p>
      </div>
    </AppCard>
  </div>
</template>
