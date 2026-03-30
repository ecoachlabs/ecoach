<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  data: { label: string; value: number; color?: string }[]
  maxValue?: number
  height?: number
  horizontal?: boolean
  showValues?: boolean
}>()

const max = computed(() => props.maxValue ?? Math.max(...props.data.map(d => d.value), 1))
const barHeight = computed(() => props.height ?? 24)

function pct(value: number): number {
  return Math.min(100, (value / max.value) * 100)
}

function defaultColor(index: number): string {
  const colors = ['var(--accent)', 'var(--gold)', 'var(--warm)', 'var(--success)', 'var(--danger)', '#7c3aed', '#0891b2']
  return colors[index % colors.length]
}
</script>

<template>
  <!-- Horizontal bars (default) -->
  <div v-if="!horizontal" class="space-y-2">
    <div v-for="(item, i) in data" :key="i" class="flex items-center gap-2">
      <span class="text-[10px] font-medium w-20 text-right truncate" :style="{ color: 'var(--text-3)' }">{{ item.label }}</span>
      <div class="flex-1 rounded-full overflow-hidden" :style="{ height: barHeight + 'px', backgroundColor: 'var(--border-soft)' }">
        <div class="h-full rounded-full transition-all flex items-center justify-end pr-2"
          :style="{ width: pct(item.value) + '%', backgroundColor: item.color ?? defaultColor(i), transitionDuration: 'var(--dur-slow)' }">
          <span v-if="showValues" class="text-[9px] font-bold text-white">{{ item.value }}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Vertical bars -->
  <div v-else class="flex items-end gap-2 justify-center" :style="{ height: (props.height ?? 160) + 'px' }">
    <div v-for="(item, i) in data" :key="i" class="flex flex-col items-center gap-1 flex-1">
      <span v-if="showValues" class="text-[9px] font-bold tabular-nums" :style="{ color: item.color ?? defaultColor(i) }">
        {{ item.value }}
      </span>
      <div class="w-full rounded-t transition-all"
        :style="{
          height: pct(item.value) + '%',
          backgroundColor: item.color ?? defaultColor(i),
          minHeight: item.value > 0 ? '4px' : '0',
          transitionDuration: 'var(--dur-slow)',
        }" />
      <span class="text-[8px] font-medium text-center" :style="{ color: 'var(--text-3)' }">{{ item.label }}</span>
    </div>
  </div>
</template>
