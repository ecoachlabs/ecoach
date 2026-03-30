<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  subjectName: string
  readinessBand: string
  masteredCount: number
  weakCount: number
  totalCount: number
  score?: number
}>()

defineEmits<{ click: [] }>()

const bandColors: Record<string, string> = { strong: 'success', developing: 'gold', weak: 'danger', critical: 'danger' }
</script>

<template>
  <AppCard hover padding="md" @click="$emit('click')">
    <div class="flex items-start justify-between mb-3">
      <h4 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subjectName }}</h4>
      <AppBadge :color="(bandColors[readinessBand] as any) || 'muted'" size="xs">{{ readinessBand }}</AppBadge>
    </div>
    <AppProgress v-if="score !== undefined" :value="score" :max="10000" size="sm"
      :color="(bandColors[readinessBand] as any) || 'accent'" class="mb-2" />
    <div class="flex items-center gap-3 text-[11px]" :style="{ color: 'var(--text-3)' }">
      <span class="flex items-center gap-1">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500" /> {{ masteredCount }} mastered
      </span>
      <span v-if="weakCount > 0" class="flex items-center gap-1">
        <span class="w-1.5 h-1.5 rounded-full bg-red-500" /> {{ weakCount }} weak
      </span>
      <span class="ml-auto tabular-nums">{{ masteredCount }}/{{ totalCount }}</span>
    </div>
  </AppCard>
</template>
