<script setup lang="ts">
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  current: number
  total: number
  correct?: number
  showAccuracy?: boolean
}>()
</script>

<template>
  <div class="flex items-center gap-3">
    <AppProgress :value="current" :max="total" size="sm" color="accent" class="flex-1" />
    <span class="text-xs tabular-nums font-medium shrink-0" :style="{ color: 'var(--text-3)' }">
      {{ current }}/{{ total }}
    </span>
    <span v-if="showAccuracy && correct !== undefined && current > 0"
      class="text-xs tabular-nums font-semibold shrink-0"
      :class="(correct / current) >= 0.7 ? 'text-emerald-600' : (correct / current) >= 0.4 ? 'text-amber-600' : 'text-red-500'">
      {{ Math.round((correct / current) * 100) }}%
    </span>
  </div>
</template>
