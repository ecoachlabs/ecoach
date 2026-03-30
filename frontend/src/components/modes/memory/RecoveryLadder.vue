<script setup lang="ts">
defineProps<{
  currentStage: number  // 0-5
}>()

const stages = [
  { label: 'Recognition', icon: '◔', desc: 'Do you recognize this?' },
  { label: 'Guided Recall', icon: '◑', desc: 'With a hint, can you recall?' },
  { label: 'Unguided Recall', icon: '◕', desc: 'Without help, what is...?' },
  { label: 'Variant Recall', icon: '●', desc: 'Same concept, different form' },
  { label: 'Connected Recall', icon: '◉', desc: 'How does this connect to...?' },
  { label: 'Pressure Recall', icon: '★', desc: 'Under time, can you still...?' },
]
</script>

<template>
  <div class="space-y-1">
    <div
      v-for="(stage, i) in stages"
      :key="stage.label"
      class="flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] transition-all"
      :class="i === currentStage ? 'bg-[var(--accent-light)]' : i < currentStage ? 'opacity-50' : ''"
    >
      <span
        class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold"
        :class="i < currentStage ? 'bg-[var(--success)] text-white' : i === currentStage ? 'bg-[var(--accent)] text-white' : ''"
        :style="i >= currentStage && i !== currentStage ? { backgroundColor: 'var(--border-soft)', color: 'var(--text-3)' } : {}"
      >
        {{ i < currentStage ? '✓' : stage.icon }}
      </span>
      <div>
        <p class="text-xs font-semibold" :style="{ color: i === currentStage ? 'var(--accent)' : 'var(--text)' }">{{ stage.label }}</p>
        <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ stage.desc }}</p>
      </div>
    </div>
  </div>
</template>
