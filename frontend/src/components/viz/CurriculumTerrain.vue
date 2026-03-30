<script setup lang="ts">
import { computed } from 'vue'
import { getMasteryDisplay } from '@/utils/mastery'

defineProps<{
  nodes: { id: number; name: string; type: string; masteryState: string; score: number; parentId: number | null; children?: number[] }[]
  selectedId?: number | null
}>()

defineEmits<{ select: [nodeId: number] }>()

function nodeSizeClass(type: string): string {
  if (type === 'subject') return 'w-32 h-16'
  if (type === 'strand') return 'w-28 h-14'
  return 'w-24 h-12'
}
</script>

<template>
  <div class="relative p-4 min-h-[300px] overflow-auto">
    <div class="flex flex-wrap gap-3 justify-center">
      <button v-for="node in nodes" :key="node.id"
        class="rounded-[var(--radius-lg)] border-2 flex flex-col items-center justify-center text-center px-3 py-2 transition-all cursor-pointer"
        :class="[
          nodeSizeClass(node.type),
          selectedId === node.id ? 'ring-2 ring-[var(--accent)] scale-105' : 'hover:scale-102',
        ]"
        :style="{
          backgroundColor: getMasteryDisplay(node.masteryState).bg,
          borderColor: getMasteryDisplay(node.masteryState).color + '40',
          boxShadow: node.score > 6000 ? `0 0 12px ${getMasteryDisplay(node.masteryState).color}25` : 'none',
        }"
        @click="$emit('select', node.id)">
        <span class="text-[10px] font-bold truncate w-full" :style="{ color: getMasteryDisplay(node.masteryState).color }">
          {{ getMasteryDisplay(node.masteryState).icon }}
        </span>
        <span class="text-[9px] font-medium truncate w-full" :style="{ color: 'var(--text)' }">{{ node.name }}</span>
        <span class="text-[8px] tabular-nums" :style="{ color: 'var(--text-3)' }">{{ (node.score / 100).toFixed(0) }}%</span>
      </button>
    </div>
    <p v-if="!nodes.length" class="text-sm text-center py-12" :style="{ color: 'var(--text-3)' }">No curriculum data available.</p>
  </div>
</template>
