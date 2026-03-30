<script setup lang="ts">
defineProps<{
  variant?: 'text' | 'circle' | 'rect'
  width?: string
  height?: string
  lines?: number
}>()
</script>

<template>
  <!-- Multi-line text skeleton -->
  <div v-if="(lines ?? 0) > 0" class="space-y-2">
    <div
      v-for="i in lines"
      :key="i"
      class="h-3 rounded animate-pulse"
      :style="{
        backgroundColor: 'var(--border-soft)',
        width: i === lines ? '60%' : '100%',
      }"
    />
  </div>
  <!-- Circle -->
  <div
    v-else-if="variant === 'circle'"
    class="rounded-full animate-pulse"
    :style="{
      width: width || '40px',
      height: height || width || '40px',
      backgroundColor: 'var(--border-soft)',
    }"
  />
  <!-- Rectangle (default) -->
  <div
    v-else
    class="rounded-[var(--radius-md)] animate-pulse"
    :style="{
      width: width || '100%',
      height: height || '40px',
      backgroundColor: 'var(--border-soft)',
    }"
  />
</template>
