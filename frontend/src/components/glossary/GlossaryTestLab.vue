<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineEmits<{ startTest: [mode: string] }>()

const modes = [
  { key: 'definition-recall', backendMode: 'recall', label: 'Definition Recall', desc: 'Pull the exact term from its meaning', icon: 'DR' },
  { key: 'intruder', backendMode: 'which_one_does_not_belong', label: 'Intruder Detection', desc: 'Spot the outsider in a term cluster', icon: 'IN' },
  { key: 'confusion', backendMode: 'confusion_duel', label: 'Confusion Duel', desc: 'Choose the clean boundary between lookalikes', icon: 'CD' },
  { key: 'context', backendMode: 'context_recognition', label: 'Context Clue', desc: 'Name the term from a live example or question', icon: 'CX' },
  { key: 'hidden', backendMode: 'fill_gap', label: 'Hidden Term', desc: 'Fill the missing piece in the pattern', icon: 'HG' },
  { key: 'reverse', backendMode: 'reverse_recall', label: 'Explain It', desc: 'Work backward from the definition to the name', icon: 'RV' },
  { key: 'bundle', backendMode: 'connection_map', label: 'Build the Bundle', desc: 'Name a directly related idea', icon: 'BN' },
  { key: 'family', backendMode: 'relationship_hunt', label: 'Concept Family', desc: 'Trace how nearby ideas depend on each other', icon: 'FM' },
  { key: 'audio', backendMode: 'audio_recall', label: 'Audio Recall', desc: 'Use the listening cue to retrieve the term', icon: 'AR' },
  { key: 'neighbor', backendMode: 'intruder_mode', label: 'Neighbor vs Intruder', desc: 'Keep near ideas together and reject the trap', icon: 'NV' },
  { key: 'formula', backendMode: 'formula_reverse_recall', label: 'Formula Builder', desc: 'Rebuild the formula from the use case', icon: 'FB' },
  { key: 'signal', backendMode: 'question_signal', label: 'Application Match', desc: 'Read the question signal and name the right idea', icon: 'QS' },
]
</script>

<template>
  <div>
    <h3 class="mb-1 text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Glossary Test Lab</h3>
    <p class="mb-4 text-xs" :style="{ color: 'var(--text-3)' }">Use real glossary test modes instead of static flashcards.</p>
    <div class="grid grid-cols-2 gap-2 lg:grid-cols-3">
      <AppCard v-for="mode in modes" :key="mode.key" hover padding="sm" @click="$emit('startTest', mode.backendMode)">
        <div class="flex items-start gap-2">
          <span class="inline-flex h-7 min-w-7 items-center justify-center rounded-md bg-[var(--primary-light)] px-1 text-[10px] font-bold uppercase tracking-wide text-[var(--primary)]">
            {{ mode.icon }}
          </span>
          <div>
            <p class="text-[11px] font-semibold" :style="{ color: 'var(--text)' }">{{ mode.label }}</p>
            <p class="text-[9px]" :style="{ color: 'var(--text-3)' }">{{ mode.desc }}</p>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
