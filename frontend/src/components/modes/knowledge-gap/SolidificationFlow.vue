<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  skillName: string
  currentStage: number
}>()

defineEmits<{ complete: [] }>()

const stages = [
  { key: 'diagnose', label: 'Diagnose', icon: '🔍', desc: 'Confirm the gap still exists' },
  { key: 'teach', label: 'Teach', icon: '📖', desc: 'Targeted explanation' },
  { key: 'guide', label: 'Guide', icon: '🤝', desc: 'Scaffolded practice with support' },
  { key: 'prove', label: 'Prove', icon: '✓', desc: 'Independent practice without help' },
  { key: 'lock_in', label: 'Lock In', icon: '🔒', desc: 'Confirm mastery, schedule recheck' },
]
</script>

<template>
  <div>
    <!-- Stage progress -->
    <div class="flex gap-1 mb-6">
      <div v-for="(stage, i) in stages" :key="stage.key"
        class="flex-1 text-center">
        <div class="h-1.5 rounded-full mb-2 transition-all"
          :style="{ backgroundColor: i <= currentStage ? 'var(--accent)' : 'var(--border-soft)' }" />
        <div class="w-8 h-8 rounded-lg mx-auto flex items-center justify-center text-sm mb-1"
          :class="i < currentStage ? 'bg-emerald-100 text-emerald-600' : i === currentStage ? 'bg-[var(--accent-light)] text-[var(--accent)]' : ''"
          :style="i > currentStage ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
          {{ i < currentStage ? '✓' : stage.icon }}
        </div>
        <p class="text-[9px] font-medium" :style="{ color: i === currentStage ? 'var(--accent)' : 'var(--text-3)' }">{{ stage.label }}</p>
      </div>
    </div>

    <!-- Current stage content -->
    <div class="text-center mb-4">
      <h3 class="font-display text-base font-semibold" :style="{ color: 'var(--text)' }">
        {{ stages[currentStage]?.label }}: {{ skillName }}
      </h3>
      <p class="text-xs mt-1" :style="{ color: 'var(--text-3)' }">{{ stages[currentStage]?.desc }}</p>
    </div>

    <!-- Content slot -->
    <slot />
  </div>
</template>
