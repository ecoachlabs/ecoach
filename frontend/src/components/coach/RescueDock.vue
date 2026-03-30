<script setup lang="ts">
defineEmits<{
  action: [type: string]
}>()

const actions = [
  { key: 'simplify', label: 'Simplify', icon: '◇' },
  { key: 'hint', label: 'Hint', icon: '💡' },
  { key: 'first_step', label: 'First Step', icon: '①' },
  { key: 'compare', label: 'Compare', icon: '⟺' },
  { key: 'explain', label: 'Explain Mistake', icon: '✕' },
  { key: 'audio', label: 'Audio', icon: '🔊' },
  { key: 'example', label: 'Example', icon: '📋' },
]
</script>

<template>
  <div class="flex flex-wrap gap-1.5">
    <button
      v-for="action in actions"
      :key="action.key"
      class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-[var(--radius-sm)] text-[11px] font-medium border transition-colors"
      :style="{
        backgroundColor: 'var(--card-bg)',
        borderColor: 'var(--card-border)',
        color: 'var(--text-2)',
      }"
      @mouseenter="($event.target as HTMLElement).style.borderColor = 'var(--accent)'"
      @mouseleave="($event.target as HTMLElement).style.borderColor = 'var(--card-border)'"
      @click="$emit('action', action.key)"
    >
      <span>{{ action.icon }}</span>
      {{ action.label }}
    </button>
  </div>
</template>
