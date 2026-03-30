<script setup lang="ts">
defineProps<{ color: string; size?: 'sm' | 'md' | 'lg'; pulse?: boolean }>()
</script>

<template>
  <span class="inline-block rounded-full shrink-0"
    :class="[size === 'sm' ? 'w-1.5 h-1.5' : size === 'lg' ? 'w-3 h-3' : 'w-2 h-2', pulse ? 'animate-pulse' : '']"
    :style="{backgroundColor: color}" />
</template>
