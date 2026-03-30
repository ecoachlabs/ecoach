<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  pairs: { questionA: string; yearA: string; questionB: string; yearB: string; inverseStrength: number }[]
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--text-3)' }">Inverse Appearance Map</h3>
    <p class="text-xs mb-4" :style="{ color: 'var(--text-3)' }">When one question appears, the other disappears.</p>
    <div class="space-y-2">
      <AppCard v-for="(pair, i) in pairs" :key="i" padding="sm">
        <div class="grid grid-cols-[1fr_auto_1fr] gap-3 items-center">
          <div>
            <p class="text-xs font-medium" :style="{ color: 'var(--text)' }">{{ pair.questionA }}</p>
            <p class="text-[9px]" :style="{ color: 'var(--text-3)' }">{{ pair.yearA }}</p>
          </div>
          <div class="text-center">
            <span class="text-xs font-bold" :style="{ color: 'var(--danger)' }">⟺</span>
            <p class="text-[8px]" :style="{ color: 'var(--text-3)' }">{{ pair.inverseStrength }}%</p>
          </div>
          <div class="text-right">
            <p class="text-xs font-medium" :style="{ color: 'var(--text)' }">{{ pair.questionB }}</p>
            <p class="text-[9px]" :style="{ color: 'var(--text-3)' }">{{ pair.yearB }}</p>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
