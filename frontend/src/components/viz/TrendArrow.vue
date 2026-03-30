<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  value: number  // positive = up, negative = down, 0 = flat
  label?: string
  size?: 'sm' | 'md'
}>()

const direction = computed(() => props.value > 0 ? 'up' : props.value < 0 ? 'down' : 'flat')
const color = computed(() => direction.value === 'up' ? 'var(--success)' : direction.value === 'down' ? 'var(--danger)' : 'var(--text-3)')
const arrow = computed(() => direction.value === 'up' ? '↑' : direction.value === 'down' ? '↓' : '→')
</script>

<template>
  <span
    class="inline-flex items-center gap-0.5 font-semibold tabular-nums"
    :class="size === 'sm' ? 'text-[10px]' : 'text-xs'"
    :style="{ color }"
  >
    <span>{{ arrow }}</span>
    <span>{{ value > 0 ? '+' : '' }}{{ value }}{{ label ? ' ' + label : '' }}</span>
  </span>
</template>
