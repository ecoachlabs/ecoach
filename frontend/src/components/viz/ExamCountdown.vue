<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  examDate: string    // ISO date string
  examName?: string
  readinessBand?: string
}>()

const daysLeft = computed(() => {
  const now = new Date()
  const exam = new Date(props.examDate)
  const diff = Math.ceil((exam.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
  return Math.max(0, diff)
})

const urgency = computed(() => {
  if (daysLeft.value <= 14) return 'critical'
  if (daysLeft.value <= 30) return 'urgent'
  if (daysLeft.value <= 60) return 'moderate'
  return 'relaxed'
})

const urgencyColor = computed(() => {
  switch (urgency.value) {
    case 'critical': return 'var(--danger)'
    case 'urgent': return 'var(--warning)'
    case 'moderate': return 'var(--gold)'
    default: return 'var(--accent)'
  }
})
</script>

<template>
  <div
    class="inline-flex items-center gap-3 px-4 py-2.5 rounded-[var(--radius-lg)]"
    :style="{
      backgroundColor: urgencyColor + '08',
    }"
  >
    <div class="text-center">
      <span class="font-display text-2xl font-bold tabular-nums" :style="{ color: urgencyColor }">
        {{ daysLeft }}
      </span>
      <p class="text-[8px] font-semibold uppercase" :style="{ color: urgencyColor }">days</p>
    </div>
    <div>
      <p class="text-xs font-medium" :style="{ color: 'var(--text)' }">{{ examName || 'Exam' }}</p>
      <p v-if="readinessBand" class="text-[10px] capitalize" :style="{ color: 'var(--text-3)' }">
        Readiness: {{ readinessBand }}
      </p>
    </div>
  </div>
</template>
