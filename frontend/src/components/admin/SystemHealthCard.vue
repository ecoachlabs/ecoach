<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  metrics: { label: string; value: string; status: 'healthy' | 'warning' | 'error' }[]
}>()

const statusColors: Record<string, string> = { healthy: 'success', warning: 'warm', error: 'danger' }
</script>

<template>
  <AppCard padding="md">
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">System Health</h3>
    <div class="space-y-2">
      <div v-for="m in metrics" :key="m.label" class="flex items-center justify-between text-xs">
        <span :style="{color:'var(--text-2)'}">{{ m.label }}</span>
        <div class="flex items-center gap-2">
          <span class="font-medium tabular-nums" :style="{color:'var(--text)'}">{{ m.value }}</span>
          <div class="w-2 h-2 rounded-full" :class="{
            'bg-emerald-500': m.status === 'healthy',
            'bg-amber-500': m.status === 'warning',
            'bg-red-500': m.status === 'error',
          }" />
        </div>
      </div>
    </div>
  </AppCard>
</template>
