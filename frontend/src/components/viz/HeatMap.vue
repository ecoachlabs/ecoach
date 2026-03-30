<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  rows: { label: string; values: number[] }[]
  columns: string[]
  maxValue?: number
  colorScale?: 'green-red' | 'cold-hot' | 'mastery'
}>()

defineEmits<{ cellClick: [row: number, col: number, value: number] }>()

const max = computed(() => props.maxValue ?? 10000)

function cellColor(value: number): string {
  const pct = Math.min(1, value / max.value)
  if (props.colorScale === 'green-red' || !props.colorScale) {
    if (pct >= 0.7) return '#16a34a'
    if (pct >= 0.5) return '#65a30d'
    if (pct >= 0.3) return '#eab308'
    if (pct > 0) return '#ea580c'
    return '#e5e5e5'
  }
  if (props.colorScale === 'mastery') {
    if (pct >= 0.8) return '#0d9488'
    if (pct >= 0.6) return '#22c55e'
    if (pct >= 0.4) return '#fcd34d'
    if (pct >= 0.2) return '#fb923c'
    if (pct > 0) return '#ef4444'
    return '#d6d3d1'
  }
  // cold-hot
  const r = Math.round(pct * 220 + 35)
  const g = Math.round((1 - pct) * 180 + 40)
  const b = Math.round((1 - pct) * 60 + 40)
  return `rgb(${r},${g},${b})`
}

function cellOpacity(value: number): number {
  if (value === 0) return 0.3
  return 0.4 + (Math.min(1, value / max.value)) * 0.6
}
</script>

<template>
  <div class="overflow-x-auto">
    <table class="w-full border-collapse">
      <!-- Column headers -->
      <thead>
        <tr>
          <th class="px-2 py-1 text-left text-[9px] font-semibold uppercase" :style="{ color: 'var(--text-3)' }"></th>
          <th v-for="col in columns" :key="col"
            class="px-2 py-1 text-center text-[9px] font-semibold uppercase whitespace-nowrap"
            :style="{ color: 'var(--text-3)' }">
            {{ col }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, ri) in rows" :key="ri">
          <td class="px-2 py-1 text-[10px] font-medium whitespace-nowrap" :style="{ color: 'var(--text)' }">
            {{ row.label }}
          </td>
          <td v-for="(val, ci) in row.values" :key="ci"
            class="px-1 py-1 text-center cursor-pointer transition-transform hover:scale-110"
            @click="$emit('cellClick', ri, ci, val)">
            <div class="w-8 h-8 mx-auto rounded-[4px] flex items-center justify-center text-[9px] font-bold text-white"
              :style="{ backgroundColor: cellColor(val), opacity: cellOpacity(val) }">
              {{ val > 0 ? (val / 100).toFixed(0) : '' }}
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
