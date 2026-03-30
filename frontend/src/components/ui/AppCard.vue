<script setup lang="ts">
defineProps<{
  hover?: boolean
  padding?: 'none' | 'sm' | 'md' | 'lg'
  glow?: 'accent' | 'warm' | 'gold' | null
}>()
</script>

<template>
  <div
    class="rounded-[var(--radius-lg)] border transition-all"
    :class="[
      hover ? 'card-lift cursor-pointer' : '',
      padding === 'none' ? '' :
      padding === 'sm' ? 'p-3' :
      padding === 'lg' ? 'p-8' :
      'p-5',
    ]"
    :style="{
      backgroundColor: 'var(--card-bg)',
      borderColor: 'var(--card-border)',
      boxShadow: glow === 'accent' ? 'var(--shadow-glow-accent)' :
                  glow === 'warm' ? 'var(--shadow-glow-warm)' :
                  glow === 'gold' ? 'var(--shadow-glow-gold)' :
                  'var(--shadow-xs)',
    }"
  >
    <slot />
  </div>
</template>
