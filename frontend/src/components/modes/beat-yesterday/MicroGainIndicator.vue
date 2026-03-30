<script setup lang="ts">
defineProps<{
  type: 'volume' | 'accuracy' | 'speed'
  value: number
  unit?: string
}>()
</script>

<template>
  <span
    class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-full text-[10px] font-bold tabular-nums"
    :class="{
      'bg-emerald-50 text-emerald-700': value > 0,
      'bg-red-50 text-red-600': value < 0,
      'bg-stone-50 text-stone-500': value === 0,
    }"
  >
    {{ value > 0 ? '+' : '' }}{{ value }}{{ unit || '' }}
    <span class="opacity-60">{{ type === 'volume' ? 'attempted' : type === 'accuracy' ? 'correct' : 'sec' }}</span>
  </span>
</template>
