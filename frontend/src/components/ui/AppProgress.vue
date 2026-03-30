<script setup lang="ts">
const props = defineProps<{
  value: number  // 0-100 or 0-10000 (BasisPoints)
  max?: number   // default 100
  color?: 'accent' | 'warm' | 'gold' | 'success' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  showLabel?: boolean
  glow?: boolean
}>()

const percentage = computed(() => {
  const max = props.max ?? 100
  return Math.min(100, Math.max(0, (props.value / max) * 100))
})

import { computed } from 'vue'

const colorVar = computed(() => {
  switch (props.color) {
    case 'warm': return 'var(--warm)'
    case 'gold': return 'var(--gold)'
    case 'success': return 'var(--success)'
    case 'danger': return 'var(--danger)'
    default: return 'var(--accent)'
  }
})
</script>

<template>
  <div class="flex items-center gap-2">
    <div
      class="flex-1 rounded-full overflow-hidden"
      :class="size === 'sm' ? 'h-1' : size === 'lg' ? 'h-3' : 'h-2'"
      :style="{ backgroundColor: 'var(--border-soft)' }"
    >
      <div
        class="h-full rounded-full transition-all"
        :style="{
          width: percentage + '%',
          backgroundColor: colorVar,
          boxShadow: glow ? `0 0 8px ${colorVar}40` : 'none',
          transitionDuration: 'var(--dur-slow)',
          transitionTimingFunction: 'var(--ease-out)',
        }"
      />
    </div>
    <span v-if="showLabel" class="text-xs font-medium tabular-nums" :style="{ color: 'var(--text-3)' }">
      {{ Math.round(percentage) }}%
    </span>
  </div>
</template>
