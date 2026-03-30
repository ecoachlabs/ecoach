<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  conceptA: string
  conceptB: string
  comparisonRows: { dimension: string; a: string; b: string; revealed?: boolean }[]
}>()

defineEmits<{ reveal: [dimension: string]; complete: [] }>()
</script>

<template>
  <div>
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="font-display text-base font-semibold" :style="{ color: 'var(--text)' }">Know the Difference</h3>
      <AppBadge color="gold" size="sm">Comparative</AppBadge>
    </div>

    <!-- Split comparison -->
    <div class="grid grid-cols-[1fr_auto_1fr] gap-0 rounded-[var(--radius-xl)] border overflow-hidden"
      :style="{ borderColor: 'var(--card-border)' }">

      <!-- Headers -->
      <div class="px-4 py-3 text-center font-semibold text-sm" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
        {{ conceptA }}
      </div>
      <div class="px-2 py-3 text-center text-xs font-bold" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }">vs</div>
      <div class="px-4 py-3 text-center font-semibold text-sm" :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">
        {{ conceptB }}
      </div>

      <!-- Comparison rows -->
      <template v-for="row in comparisonRows" :key="row.dimension">
        <div class="px-4 py-3 border-t text-sm" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }">
          <span v-if="row.revealed">{{ row.a }}</span>
          <button v-else class="text-xs px-2 py-1 rounded" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }"
            @click="$emit('reveal', row.dimension)">Tap to reveal</button>
        </div>
        <div class="px-2 py-3 border-t text-center text-[10px] font-medium"
          :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }">
          {{ row.dimension }}
        </div>
        <div class="px-4 py-3 border-t text-sm" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }">
          <span v-if="row.revealed">{{ row.b }}</span>
          <button v-else class="text-xs px-2 py-1 rounded" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }"
            @click="$emit('reveal', row.dimension)">Tap to reveal</button>
        </div>
      </template>
    </div>
  </div>
</template>
