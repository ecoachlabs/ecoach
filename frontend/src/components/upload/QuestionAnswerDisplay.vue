<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  questionNumber: number
  questionText: string
  answerText: string
  markingResult?: 'correct' | 'partial' | 'incorrect' | 'unmarked'
  marksAwarded?: number
  marksTotal?: number
  topicDetected?: string
}>()
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-start gap-3">
      <div class="w-7 h-7 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0"
        :class="{
          'bg-emerald-100 text-emerald-700': markingResult === 'correct',
          'bg-amber-100 text-amber-700': markingResult === 'partial',
          'bg-red-100 text-red-600': markingResult === 'incorrect',
        }"
        :style="!markingResult || markingResult === 'unmarked' ? {backgroundColor:'var(--primary-light)',color:'var(--text-3)'} : {}">
        Q{{ questionNumber }}
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-xs font-medium mb-1" :style="{color:'var(--text)'}">{{ questionText }}</p>
        <div class="px-3 py-2 rounded-[var(--radius-sm)] mb-2" :style="{backgroundColor:'var(--primary-light)'}">
          <p class="text-xs" :style="{color:'var(--text-2)'}">{{ answerText }}</p>
        </div>
        <div class="flex items-center gap-2">
          <AppBadge v-if="markingResult"
            :color="markingResult === 'correct' ? 'success' : markingResult === 'partial' ? 'gold' : markingResult === 'incorrect' ? 'danger' : 'muted'"
            size="xs">{{ markingResult }}</AppBadge>
          <span v-if="marksAwarded !== undefined" class="text-[10px] tabular-nums" :style="{color:'var(--text-3)'}">
            {{ marksAwarded }}/{{ marksTotal }} marks
          </span>
          <AppBadge v-if="topicDetected" color="accent" size="xs">{{ topicDetected }}</AppBadge>
        </div>
      </div>
    </div>
  </AppCard>
</template>
