<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  label: string
  value: string | number
  change?: number
  icon?: string
}>()
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-start justify-between">
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">{{ label }}</p>
        <p class="font-display text-2xl font-bold mt-1" :style="{ color: 'var(--text)' }">{{ value }}</p>
        <p v-if="change !== undefined" class="text-[10px] font-semibold mt-1"
          :class="change >= 0 ? 'text-emerald-600' : 'text-red-500'">
          {{ change >= 0 ? '↑' : '↓' }} {{ Math.abs(change) }}%
        </p>
      </div>
      <div v-if="icon" class="w-10 h-10 rounded-lg flex items-center justify-center text-lg"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">
        {{ icon }}
      </div>
    </div>
  </AppCard>
</template>
