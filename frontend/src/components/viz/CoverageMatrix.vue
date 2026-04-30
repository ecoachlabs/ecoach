<script setup lang="ts">
import HeatMap from './HeatMap.vue'

defineProps<{
  topics: string[]
  contentTypes: string[]
  coverage: number[][]
}>()

defineEmits<{ cellClick: [topic: number, contentType: number, value: number] }>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Content Coverage</h3>
    <HeatMap
      :rows="topics.map((t, i) => ({ label: t, values: coverage[i] || [] }))"
      :columns="contentTypes"
      color-scale="mastery"
      @cell-click="(row, col, value) => $emit('cellClick', row, col, value)"
    />
    <div class="flex items-center gap-4 mt-3 text-[9px]" :style="{ color: 'var(--text-3)' }">
      <span class="flex items-center gap-1"><span class="w-3 h-3 rounded-sm bg-[#e5e5e5]" /> No content</span>
      <span class="flex items-center gap-1"><span class="w-3 h-3 rounded-sm bg-[#ef4444]" /> Low</span>
      <span class="flex items-center gap-1"><span class="w-3 h-3 rounded-sm bg-[#eab308]" /> Medium</span>
      <span class="flex items-center gap-1"><span class="w-3 h-3 rounded-sm bg-[#16a34a]" /> Good</span>
      <span class="flex items-center gap-1"><span class="w-3 h-3 rounded-sm bg-[#0d9488]" /> Complete</span>
    </div>
  </div>
</template>
