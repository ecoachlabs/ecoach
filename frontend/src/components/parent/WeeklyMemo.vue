<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  date: string
  content: string
  highlights?: string[]
  concerns?: string[]
}>()
</script>

<template>
  <AppCard padding="lg">
    <div class="flex items-center justify-between mb-4">
      <h3 class="font-display text-base font-semibold" :style="{ color: 'var(--text)' }">Weekly Strategy Memo</h3>
      <AppBadge color="muted" size="xs">{{ date }}</AppBadge>
    </div>
    <p class="text-sm leading-relaxed mb-4" :style="{ color: 'var(--text-2)' }">{{ content }}</p>
    <div v-if="highlights?.length" class="mb-3">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" style="color: var(--success)">Highlights</p>
      <ul class="space-y-1">
        <li v-for="h in highlights" :key="h" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
          <span class="text-emerald-500 mt-0.5">✓</span> {{ h }}
        </li>
      </ul>
    </div>
    <div v-if="concerns?.length">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" style="color: var(--warning)">Areas of Concern</p>
      <ul class="space-y-1">
        <li v-for="c in concerns" :key="c" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
          <span class="text-amber-500 mt-0.5">⚠</span> {{ c }}
        </li>
      </ul>
    </div>
  </AppCard>
</template>
