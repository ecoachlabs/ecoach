<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  count: number
  severity: 'critical' | 'warning' | 'improving' | 'slipping' | 'hidden' | 'fixed'
  items: { name: string; score: number }[]
}>()

const severityConfig: Record<string, { color: string; bg: string; icon: string }> = {
  critical: { color: 'var(--danger)', bg: 'var(--danger-light)', icon: '🔴' },
  warning: { color: 'var(--warning)', bg: 'var(--warning-light)', icon: '🟡' },
  improving: { color: 'var(--success)', bg: 'var(--success-light)', icon: '🟢' },
  slipping: { color: 'var(--warm)', bg: 'var(--warm-light)', icon: '🟠' },
  hidden: { color: '#7c3aed', bg: '#f5f3ff', icon: '🟣' },
  fixed: { color: 'var(--success)', bg: 'var(--success-light)', icon: '✅' },
}
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <span>{{ severityConfig[severity]?.icon || '●' }}</span>
        <h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h3>
      </div>
      <AppBadge :color="severity === 'fixed' || severity === 'improving' ? 'success' : severity === 'critical' ? 'danger' : 'warm'" size="xs">
        {{ count }}
      </AppBadge>
    </div>
    <div class="space-y-1.5">
      <div v-for="item in items.slice(0, 4)" :key="item.name"
        class="flex items-center justify-between text-xs py-1">
        <span :style="{ color: 'var(--text-2)' }">{{ item.name }}</span>
        <span class="tabular-nums font-medium" :style="{ color: severityConfig[severity]?.color || 'var(--text-3)' }">
          {{ (item.score / 100).toFixed(0) }}%
        </span>
      </div>
    </div>
  </AppCard>
</template>
