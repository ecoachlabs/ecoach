<script setup lang="ts">
defineProps<{
  count: number
  label?: string
  size?: 'sm' | 'md' | 'lg'
  animated?: boolean
}>()
</script>

<template>
  <div class="inline-flex items-center gap-1.5" :class="animated ? 'ember-glow' : ''">
    <span
      class="font-display font-bold tabular-nums"
      :class="[
        size === 'sm' ? 'text-lg' : size === 'lg' ? 'text-4xl' : 'text-2xl',
      ]"
      :style="{ color: 'var(--gold)' }"
    >
      🔥 {{ count }}
    </span>
    <span v-if="label" class="text-[10px] font-medium uppercase" :style="{ color: 'var(--text-3)' }">
      {{ label }}
    </span>
  </div>
</template>
