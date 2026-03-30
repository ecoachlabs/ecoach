<script setup lang="ts">
defineProps<{
  name: string
  gapScore: number
  state: string
  isBlocker: boolean
  selected: boolean
}>()

defineEmits<{ click: [] }>()

function gapColor(score: number): string {
  if (score >= 7000) return '#dc2626'
  if (score >= 5000) return '#ea580c'
  if (score >= 3000) return '#d97706'
  if (score > 0) return '#65a30d'
  return '#22c55e'
}
</script>

<template>
  <button
    class="px-3 py-2 rounded-[var(--radius-md)] border-2 text-center transition-all cursor-pointer"
    :class="[selected ? 'ring-2 ring-[var(--accent)] scale-105' : 'hover:scale-102', isBlocker ? 'border-dashed' : '']"
    :style="{
      borderColor: gapColor(gapScore) + '60',
      backgroundColor: gapColor(gapScore) + '10',
    }"
    @click="$emit('click')">
    <p class="text-[10px] font-bold truncate" :style="{ color: gapColor(gapScore) }">{{ (gapScore / 100).toFixed(0) }}%</p>
    <p class="text-[9px] font-medium truncate" :style="{ color: 'var(--text)' }">{{ name }}</p>
    <p v-if="isBlocker" class="text-[7px] uppercase font-bold" :style="{ color: '#7c3aed' }">blocker</p>
  </button>
</template>
