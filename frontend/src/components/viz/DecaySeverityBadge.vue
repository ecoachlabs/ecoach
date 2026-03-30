<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  severity: 'stable' | 'watchlist' | 'fragile' | 'decaying' | 'collapsed' | string
  showLabel?: boolean
  size?: 'sm' | 'md'
}>()

const config: Record<string, { label: string; color: string; bg: string; icon: string }> = {
  stable:    { label: 'Stable',    color: 'var(--decay-stable)',    bg: '#dcfce7', icon: '●' },
  watchlist: { label: 'Watchlist', color: 'var(--decay-watch)',     bg: '#fef9c3', icon: '◔' },
  fragile:   { label: 'Fragile',   color: 'var(--decay-fragile)',   bg: '#ffedd5', icon: '◑' },
  decaying:  { label: 'Decaying',  color: 'var(--decay-decaying)',  bg: '#fee2e2', icon: '◕' },
  collapsed: { label: 'Collapsed', color: 'var(--decay-collapsed)', bg: '#f5f5f4', icon: '○' },
}

const c = computed(() => config[props.severity] ?? config.stable)
</script>

<template>
  <span
    class="inline-flex items-center gap-1 font-medium rounded-full"
    :class="size === 'sm' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-[11px]'"
    :style="{ backgroundColor: c.bg, color: c.color }"
  >
    <span>{{ c.icon }}</span>
    <span v-if="showLabel !== false">{{ c.label }}</span>
  </span>
</template>
