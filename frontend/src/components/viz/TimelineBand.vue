<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  events: { date: string; label: string; type: string; color?: string }[]
  startDate?: string
  endDate?: string
  todayMarker?: boolean
}>()

const start = computed(() => new Date(props.startDate ?? props.events[0]?.date ?? new Date().toISOString()))
const end = computed(() => new Date(props.endDate ?? props.events[props.events.length - 1]?.date ?? new Date().toISOString()))
const totalMs = computed(() => Math.max(1, end.value.getTime() - start.value.getTime()))

function positionPct(dateStr: string): number {
  const d = new Date(dateStr)
  return Math.max(0, Math.min(100, ((d.getTime() - start.value.getTime()) / totalMs.value) * 100))
}

const todayPct = computed(() => {
  const now = new Date()
  return Math.max(0, Math.min(100, ((now.getTime() - start.value.getTime()) / totalMs.value) * 100))
})

const typeColors: Record<string, string> = {
  mock: 'var(--accent)', exam: 'var(--danger)', milestone: 'var(--gold)', event: 'var(--warm)',
}
</script>

<template>
  <div class="relative py-6">
    <!-- Track -->
    <div class="h-1.5 rounded-full" :style="{ backgroundColor: 'var(--border-soft)' }" />

    <!-- Today marker -->
    <div v-if="todayMarker" class="absolute top-0 bottom-0 flex flex-col items-center"
      :style="{ left: todayPct + '%', transform: 'translateX(-50%)' }">
      <span class="text-[8px] font-bold" :style="{ color: 'var(--accent)' }">Today</span>
      <div class="w-px flex-1 bg-[var(--accent)] opacity-50" />
    </div>

    <!-- Events -->
    <div v-for="(event, i) in events" :key="i"
      class="absolute top-6 flex flex-col items-center"
      :style="{ left: positionPct(event.date) + '%', transform: 'translateX(-50%)' }">
      <div class="w-3 h-3 rounded-full border-2 border-white shadow-sm"
        :style="{ backgroundColor: event.color ?? typeColors[event.type] ?? 'var(--text-3)' }" />
      <div class="mt-1 text-center whitespace-nowrap">
        <p class="text-[8px] font-semibold" :style="{ color: event.color ?? typeColors[event.type] ?? 'var(--text-3)' }">{{ event.label }}</p>
        <p class="text-[7px]" :style="{ color: 'var(--text-3)' }">{{ new Date(event.date).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' }) }}</p>
      </div>
    </div>
  </div>
</template>
