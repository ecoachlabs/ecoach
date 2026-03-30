<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'warm'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  disabled?: boolean
  icon?: boolean
}>()
</script>

<template>
  <button
    class="inline-flex items-center justify-center gap-2 font-medium transition-all select-none relative overflow-hidden"
    :class="[
      // Size
      size === 'sm' ? 'px-3 py-1.5 text-xs rounded-lg' :
      size === 'lg' ? 'px-6 py-3 text-base rounded-xl' :
      'px-4 py-2 text-sm rounded-[10px]',
      // Icon-only
      icon && (size === 'sm' ? 'px-1.5' : size === 'lg' ? 'px-3' : 'px-2'),
      // Variant
      variant === 'secondary' ? 'bg-[var(--card-bg)] text-[var(--text)] border border-[var(--card-border)] hover:bg-[var(--primary-light)] hover:border-[var(--primary)] hover:text-[var(--primary)]' :
      variant === 'ghost' ? 'bg-transparent text-[var(--text-2)] hover:bg-[var(--primary-light)] hover:text-[var(--primary)]' :
      variant === 'danger' ? 'bg-[var(--danger)] text-white hover:brightness-110 shadow-sm' :
      variant === 'warm' ? 'bg-gradient-to-r from-[var(--warm)] to-[var(--gold)] text-white hover:brightness-110 shadow-sm shadow-[var(--shadow-glow-warm)]' :
      'bg-[var(--primary)] text-white hover:bg-[var(--primary-hover)] shadow-sm',
      // States
      (disabled || loading) ? 'opacity-50 pointer-events-none' : 'cursor-pointer active:scale-[0.97]',
    ]"
    :disabled="disabled || loading"
    :style="{ transitionDuration: 'var(--dur-fast)', transitionTimingFunction: 'var(--ease-spring)' }"
  >
    <!-- Loading spinner -->
    <svg v-if="loading" class="animate-spin -ml-0.5 w-4 h-4" viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-25" />
      <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75" />
    </svg>
    <slot />
  </button>
</template>
