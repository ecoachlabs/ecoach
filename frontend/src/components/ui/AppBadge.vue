<script setup lang="ts">
defineProps<{
  color?: 'accent' | 'warm' | 'gold' | 'success' | 'danger' | 'muted' | 'ember'
  size?: 'xs' | 'sm' | 'md'
  dot?: boolean
}>()
</script>

<template>
  <span
    class="inline-flex items-center gap-1 font-medium rounded-full whitespace-nowrap"
    :class="[
      size === 'xs' ? 'px-1.5 py-0.5 text-[10px]' :
      size === 'md' ? 'px-3 py-1 text-xs' :
      'px-2 py-0.5 text-[11px]',
      color === 'accent' ? 'bg-[var(--accent-light)] text-[var(--accent)]' :
      color === 'warm' ? 'bg-[var(--warm-light)] text-[var(--warm)]' :
      color === 'gold' ? 'bg-[var(--gold-light)] text-[var(--gold)]' :
      color === 'success' ? 'bg-[var(--success-light)] text-[var(--success)]' :
      color === 'danger' ? 'bg-[var(--danger-light)] text-[var(--danger)]' :
      color === 'ember' ? 'bg-amber-50 text-amber-700' :
      'bg-stone-100 text-stone-500',
    ]"
  >
    <span v-if="dot" class="w-1.5 h-1.5 rounded-full bg-current opacity-80" />
    <slot />
  </span>
</template>
