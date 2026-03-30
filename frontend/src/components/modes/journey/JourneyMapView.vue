<script setup lang="ts">
import JourneyStationCard from './JourneyStationCard.vue'
import JourneyPhaseBar from './JourneyPhaseBar.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  currentPhase: string
  stations: { stationName: string; topicName: string; masteryState: string; locked: boolean; current: boolean; completed: boolean }[]
  completedCount: number
  totalCount: number
}>()

defineEmits<{ enterStation: [index: number] }>()
</script>

<template>
  <div>
    <JourneyPhaseBar :current-phase="currentPhase" class="mb-4" />

    <div class="flex items-center justify-between mb-3">
      <span class="text-xs" :style="{color:'var(--text-3)'}">{{ completedCount }}/{{ totalCount }} stations completed</span>
      <AppBadge color="accent" size="xs">{{ currentPhase }}</AppBadge>
    </div>

    <div class="space-y-2">
      <JourneyStationCard
        v-for="(station, i) in stations" :key="i"
        v-bind="station"
        @enter="$emit('enterStation', i)"
      />
    </div>
  </div>
</template>
