<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import MemoryStrandViz from '@/components/viz/MemoryStrandViz.vue'

defineProps<{
  chainName: string
  nodes: { name: string; status: 'strong' | 'weak' | 'broken' }[]
  strands: { id: number; from: string; to: string; strength: number; status: 'strong' | 'weak' | 'broken' | 'rebuilding' }[]
  currentRepairIndex: number
}>()

defineEmits<{ startRepair: [nodeIndex: number] }>()
</script>

<template>
  <div>
    <h3 class="font-display text-base font-semibold mb-1" :style="{ color: 'var(--text)' }">Chain Repair: {{ chainName }}</h3>
    <p class="text-xs mb-4" :style="{ color: 'var(--text-3)' }">This prerequisite chain needs rebuilding from the foundation.</p>

    <!-- Chain visualization -->
    <AppCard padding="md" class="mb-4">
      <div class="flex items-center justify-between gap-2 mb-3">
        <div v-for="(node, i) in nodes" :key="i" class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center text-[10px] font-bold"
            :class="{
              'bg-emerald-100 text-emerald-700': node.status === 'strong',
              'bg-amber-100 text-amber-700': node.status === 'weak',
              'bg-red-100 text-red-600': node.status === 'broken',
            }">
            {{ node.status === 'strong' ? '✓' : node.status === 'weak' ? '◑' : '✕' }}
          </div>
          <div v-if="i < nodes.length - 1" class="w-6 h-px" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
      </div>
      <div class="space-y-0.5 text-[10px]">
        <div v-for="(node, i) in nodes" :key="i" class="flex items-center gap-2">
          <span class="w-4 text-right font-bold" :style="{ color: 'var(--text-3)' }">{{ i + 1 }}</span>
          <span :style="{ color: i === currentRepairIndex ? 'var(--accent)' : 'var(--text-2)' }">{{ node.name }}</span>
          <AppBadge v-if="i === currentRepairIndex" color="accent" size="xs">Repairing</AppBadge>
        </div>
      </div>
    </AppCard>

    <!-- Strand connections -->
    <AppCard padding="md" class="mb-4">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Memory Connections</p>
      <MemoryStrandViz :strands="strands" />
    </AppCard>

    <AppButton variant="primary" @click="$emit('startRepair', currentRepairIndex)">
      Repair {{ nodes[currentRepairIndex]?.name }} →
    </AppButton>
  </div>
</template>
