<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineEmits<{ startTest: [mode: string] }>()

const modes = [
  { key: 'recall', label: 'Definition Recall', desc: 'Match terms to definitions', icon: '📖' },
  { key: 'intruder', label: 'Intruder Detection', desc: 'Which term does not belong?', icon: '🔍' },
  { key: 'confusion', label: 'Confusion Duel', desc: 'Is this A or B?', icon: '⚔' },
  { key: 'context', label: 'Context Clue', desc: 'Identify from context', icon: '📝' },
  { key: 'hidden', label: 'Hidden Term', desc: 'Fill in the missing term', icon: '🕵' },
  { key: 'explain', label: 'Explain It', desc: 'Explain in your own words', icon: '💬' },
  { key: 'bundle', label: 'Build the Bundle', desc: 'Group related terms', icon: '📦' },
  { key: 'family', label: 'Concept Family', desc: 'Test related concepts', icon: '🔗' },
  { key: 'pronounce', label: 'Pronunciation', desc: 'Say it correctly', icon: '🔊' },
  { key: 'builder', label: 'Word Builder', desc: 'Break term into roots', icon: '🏗' },
  { key: 'neighbor', label: 'Neighbor vs Intruder', desc: 'Spot the outsider', icon: '👀' },
  { key: 'formula_words', label: 'Formula to Words', desc: 'Convert formulas to language', icon: '∑' },
  { key: 'application', label: 'Application Match', desc: 'Match to real-world use', icon: '🌍' },
  { key: 'definition_recognition', label: 'Pick the Definition', desc: 'Select correct definition', icon: '✓' },
]
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{color:'var(--text-3)'}">Glossary Test Lab</h3>
    <p class="text-xs mb-4" :style="{color:'var(--text-3)'}">14 ways to test your vocabulary and concept knowledge.</p>
    <div class="grid grid-cols-2 lg:grid-cols-3 gap-2">
      <AppCard v-for="m in modes" :key="m.key" hover padding="sm" @click="$emit('startTest', m.key)">
        <div class="flex items-start gap-2">
          <span class="text-lg">{{ m.icon }}</span>
          <div>
            <p class="text-[11px] font-semibold" :style="{color:'var(--text)'}">{{ m.label }}</p>
            <p class="text-[9px]" :style="{color:'var(--text-3)'}">{{ m.desc }}</p>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
