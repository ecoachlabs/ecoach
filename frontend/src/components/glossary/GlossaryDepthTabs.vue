<script setup lang="ts">
defineProps<{
  activeDepth: string
  availableDepths?: string[]
}>()

defineEmits<{ select: [depth: string] }>()

const defaultDepths = [
  { key: 'quick', label: 'Quick', icon: '⚡' },
  { key: 'simple', label: 'Simple', icon: '💡' },
  { key: 'exam', label: 'Exam', icon: '📝' },
  { key: 'deep', label: 'Deep', icon: '◈' },
  { key: 'visual', label: 'Visual', icon: '🎨' },
  { key: 'audio', label: 'Audio', icon: '🔊' },
]
</script>

<template>
  <div class="flex gap-1 p-1 rounded-[var(--radius-lg)]" :style="{ backgroundColor: 'var(--primary-light)' }">
    <button v-for="depth in defaultDepths" :key="depth.key"
      :class="activeDepth === depth.key ? 'bg-[var(--card-bg)] shadow-sm text-[var(--text)]' : 'text-[var(--text-3)]'"
      class="flex items-center gap-1 px-3 py-1.5 rounded-[var(--radius-md)] text-[11px] font-medium transition-all"
      :style="{ transitionDuration: 'var(--dur-fast)' }"
      @click="$emit('select', depth.key)">
      <span>{{ depth.icon }}</span>
      <span>{{ depth.label }}</span>
    </button>
  </div>
</template>
