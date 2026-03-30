<script setup lang="ts">
defineProps<{
  type?: 'info' | 'success' | 'warning' | 'danger'
  title?: string
  dismissible?: boolean
}>()

defineEmits<{ dismiss: [] }>()

const config: Record<string, { bg: string; border: string; icon: string }> = {
  info: { bg: 'var(--accent-light)', border: 'var(--accent)', icon: 'ℹ' },
  success: { bg: 'var(--success-light)', border: 'var(--success)', icon: '✓' },
  warning: { bg: 'var(--warning-light)', border: 'var(--warning)', icon: '⚠' },
  danger: { bg: 'var(--danger-light)', border: 'var(--danger)', icon: '✕' },
}
</script>

<template>
  <div class="px-4 py-3 rounded-[var(--radius-md)] border-l-4 flex items-start gap-2.5"
    :style="{ backgroundColor: config[type || 'info'].bg, borderLeftColor: config[type || 'info'].border }">
    <span class="text-sm mt-0.5 shrink-0">{{ config[type || 'info'].icon }}</span>
    <div class="flex-1 min-w-0">
      <p v-if="title" class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ title }}</p>
      <div class="text-sm" :style="{ color: 'var(--text-2)' }"><slot /></div>
    </div>
    <button v-if="dismissible" class="text-xs shrink-0" :style="{ color: 'var(--text-3)' }" @click="$emit('dismiss')">✕</button>
  </div>
</template>
