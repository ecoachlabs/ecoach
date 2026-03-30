<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  strands: { id: number; from: string; to: string; strength: number; status: 'strong' | 'weak' | 'broken' | 'rebuilding' }[]
}>()

const statusStyles: Record<string, { color: string; dash: string; opacity: number }> = {
  strong: { color: 'var(--success)', dash: 'none', opacity: 1 },
  weak: { color: 'var(--gold)', dash: '4,4', opacity: 0.7 },
  broken: { color: 'var(--danger)', dash: '2,6', opacity: 0.4 },
  rebuilding: { color: 'var(--accent)', dash: '6,3', opacity: 0.8 },
}
</script>

<template>
  <div class="space-y-2">
    <div v-for="strand in strands" :key="strand.id" class="flex items-center gap-3">
      <span class="text-[10px] font-medium w-24 text-right truncate" :style="{ color: 'var(--text)' }">{{ strand.from }}</span>
      <div class="flex-1 relative h-3 flex items-center">
        <svg class="w-full h-3" preserveAspectRatio="none">
          <line x1="0" y1="6" x2="100%" y2="6"
            :stroke="statusStyles[strand.status]?.color || 'var(--border-soft)'"
            :stroke-width="strand.strength > 5000 ? 3 : 2"
            :stroke-dasharray="statusStyles[strand.status]?.dash || 'none'"
            :opacity="statusStyles[strand.status]?.opacity || 0.5"
            stroke-linecap="round" />
        </svg>
        <div v-if="strand.status === 'rebuilding'" class="absolute inset-0 flex items-center justify-center">
          <div class="w-2 h-2 rounded-full ember-glow" :style="{ backgroundColor: 'var(--accent)' }" />
        </div>
      </div>
      <span class="text-[10px] font-medium w-24 truncate" :style="{ color: 'var(--text)' }">{{ strand.to }}</span>
      <span class="text-[9px] capitalize px-1.5 py-0.5 rounded-full"
        :style="{ backgroundColor: (statusStyles[strand.status]?.color || 'var(--text-3)') + '15', color: statusStyles[strand.status]?.color || 'var(--text-3)' }">
        {{ strand.status }}
      </span>
    </div>
  </div>
</template>
