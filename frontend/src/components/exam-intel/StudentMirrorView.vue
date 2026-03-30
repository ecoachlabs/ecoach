<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  patterns: { pattern: string; examFrequency: number; yourAccuracy: number; gap: number }[]
  insight: string
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Your Performance vs Exam Patterns</h3>
    <AppCard padding="md" class="mb-4">
      <p class="text-sm" :style="{color:'var(--text-2)'}">{{ insight }}</p>
    </AppCard>
    <div class="space-y-2">
      <AppCard v-for="p in patterns" :key="p.pattern" padding="sm">
        <div class="flex items-center gap-3 mb-2">
          <p class="text-xs font-medium flex-1" :style="{color:'var(--text)'}">{{ p.pattern }}</p>
          <AppBadge :color="p.gap > 20 ? 'danger' : p.gap > 10 ? 'warm' : 'success'" size="xs">
            {{ p.gap > 0 ? '-' + p.gap + '%' : 'On track' }}
          </AppBadge>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div>
            <p class="text-[8px] uppercase mb-0.5" :style="{color:'var(--text-3)'}">Exam frequency</p>
            <AppProgress :value="p.examFrequency" size="sm" color="accent" />
          </div>
          <div>
            <p class="text-[8px] uppercase mb-0.5" :style="{color:'var(--text-3)'}">Your accuracy</p>
            <AppProgress :value="p.yourAccuracy" size="sm" :color="p.yourAccuracy >= 70 ? 'success' : p.yourAccuracy >= 40 ? 'gold' : 'danger'" />
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
