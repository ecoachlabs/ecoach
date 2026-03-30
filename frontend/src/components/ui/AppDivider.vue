<script setup lang="ts">
defineProps<{
  label?: string
}>()
</script>

<template>
  <div class="flex items-center gap-3 my-4">
    <div class="flex-1 h-px" :style="{ backgroundColor: 'var(--border-soft)' }" />
    <span v-if="label" class="text-[10px] font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">{{ label }}</span>
    <div v-if="label" class="flex-1 h-px" :style="{ backgroundColor: 'var(--border-soft)' }" />
  </div>
</template>
