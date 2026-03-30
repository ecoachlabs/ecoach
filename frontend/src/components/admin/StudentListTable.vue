<script setup lang="ts">
import AppTable from '@/components/ui/AppTable.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

defineProps<{
  students: {
    id: number
    name: string
    readinessBand: string
    masteryScore: number
    lastActive: string
    riskLevel?: string
  }[]
}>()

defineEmits<{ select: [studentId: number] }>()

const columns = [
  { key: 'name', label: 'Student' },
  { key: 'readinessBand', label: 'Readiness', width: '120px' },
  { key: 'masteryScore', label: 'Mastery', width: '100px', align: 'right' as const },
  { key: 'lastActive', label: 'Last Active', width: '120px' },
  { key: 'riskLevel', label: 'Risk', width: '80px' },
]
</script>

<template>
  <AppTable :columns="columns" :rows="students" compact @row-click="$emit('select', $event.id)">
    <template #readinessBand="{ value }">
      <AppBadge :color="value === 'strong' ? 'success' : value === 'developing' ? 'gold' : 'danger'" size="xs">
        {{ value }}
      </AppBadge>
    </template>
    <template #masteryScore="{ value }">
      <span class="tabular-nums font-medium">{{ (value / 100).toFixed(0) }}%</span>
    </template>
    <template #riskLevel="{ value }">
      <AppBadge v-if="value" :color="value === 'high' ? 'danger' : value === 'medium' ? 'warm' : 'muted'" size="xs">
        {{ value }}
      </AppBadge>
    </template>
  </AppTable>
</template>
