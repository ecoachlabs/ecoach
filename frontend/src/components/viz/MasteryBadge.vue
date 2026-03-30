<script setup lang="ts">
import type { MasteryState } from '@/types'

const props = defineProps<{
  state: MasteryState | string
  size?: 'sm' | 'md' | 'lg'
  showLabel?: boolean
  glow?: boolean
}>()

const config: Record<string, { label: string; color: string; bg: string; icon: string }> = {
  unseen:    { label: 'Not Started',   color: '#78716c', bg: '#f5f5f4', icon: '○' },
  exposed:   { label: 'Exposed',       color: '#78716c', bg: '#fafaf9', icon: '◔' },
  emerging:  { label: 'Emerging',      color: '#b45309', bg: '#fef3c7', icon: '◑' },
  partial:   { label: 'Partial',       color: '#a16207', bg: '#fef9c3', icon: '◕' },
  fragile:   { label: 'Fragile',       color: '#c2410c', bg: '#fff7ed', icon: '◉' },
  stable:    { label: 'Stable',        color: '#15803d', bg: '#dcfce7', icon: '●' },
  robust:    { label: 'Robust',        color: '#059669', bg: '#d1fae5', icon: '★' },
  exam_ready:{ label: 'Exam Ready',    color: '#0d9488', bg: '#ccfbf1', icon: '✦' },
}

const c = config[props.state] ?? config.unseen
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 font-medium rounded-full"
    :class="[
      size === 'sm' ? 'px-2 py-0.5 text-[10px]' :
      size === 'lg' ? 'px-3.5 py-1.5 text-sm' :
      'px-2.5 py-1 text-[11px]',
    ]"
    :style="{
      backgroundColor: c.bg,
      color: c.color,
      boxShadow: glow ? `0 0 12px ${c.color}30` : 'none',
    }"
  >
    <span class="leading-none" :class="glow ? 'ember-glow' : ''">{{ c.icon }}</span>
    <span v-if="showLabel !== false">{{ c.label }}</span>
  </span>
</template>
