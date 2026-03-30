<script setup lang="ts">
defineProps<{
  message: string
  type?: 'info' | 'warning' | 'success' | 'recovery'
}>()
</script>

<template>
  <div
    class="px-4 py-3 rounded-[var(--radius-md)] flex items-start gap-2.5 text-sm"
    :class="{
      'bg-[var(--accent-light)]': type === 'info' || !type,
      'bg-[var(--warning-light)]': type === 'warning',
      'bg-[var(--success-light)]': type === 'success',
      'bg-[var(--gold-light)]': type === 'recovery',
    }"
  >
    <span class="text-base shrink-0 mt-0.5">
      {{ type === 'warning' ? '⚠' : type === 'success' ? '✓' : type === 'recovery' ? '🛟' : '💡' }}
    </span>
    <p class="leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ message }}</p>
  </div>
</template>
