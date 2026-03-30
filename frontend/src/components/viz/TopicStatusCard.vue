<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import MasteryBadge from './MasteryBadge.vue'
import TrendArrow from './TrendArrow.vue'

defineProps<{
  topicName: string
  masteryState: string
  masteryScore: number  // BasisPoints
  gapScore?: number
  trend?: number
  accuracy?: number
  speed?: number
  pressure?: number
  clickable?: boolean
}>()

defineEmits<{ click: [] }>()

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}
</script>

<template>
  <AppCard :hover="clickable" padding="sm" @click="clickable && $emit('click')">
    <div class="flex items-center gap-3">
      <MasteryBadge :state="masteryState" size="sm" glow />

      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topicName }}</p>
          <TrendArrow v-if="trend !== undefined" :value="trend" size="sm" />
        </div>
        <div class="mt-1.5">
          <AppProgress
            :value="masteryScore"
            :max="10000"
            size="sm"
            :color="masteryScore >= 6000 ? 'success' : masteryScore >= 3000 ? 'gold' : 'danger'"
          />
        </div>
      </div>

      <!-- Score chips -->
      <div class="flex items-center gap-2 shrink-0">
        <span class="text-xs tabular-nums font-medium" :style="{ color: 'var(--text-3)' }">
          {{ formatBp(masteryScore) }}
        </span>
      </div>
    </div>

    <!-- Detail row -->
    <div v-if="accuracy !== undefined || speed !== undefined || pressure !== undefined"
      class="flex items-center gap-4 mt-2 pl-10 text-[10px]" :style="{ color: 'var(--text-3)' }">
      <span v-if="accuracy !== undefined">Acc: {{ formatBp(accuracy) }}</span>
      <span v-if="speed !== undefined">Speed: {{ formatBp(speed) }}</span>
      <span v-if="pressure !== undefined">Pressure: {{ formatBp(pressure) }}</span>
      <span v-if="gapScore !== undefined && gapScore > 0" :style="{ color: 'var(--danger)' }">
        Gap: {{ formatBp(gapScore) }}
      </span>
    </div>
  </AppCard>
</template>
