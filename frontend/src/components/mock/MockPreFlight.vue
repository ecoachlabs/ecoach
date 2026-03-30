<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  mockType: string
  subject: string
  questionCount: number
  durationMinutes: number
  readinessScore?: number
  readinessBand?: string
}>()

defineEmits<{ start: []; cancel: [] }>()
</script>

<template>
  <div class="max-w-lg mx-auto text-center reveal-stagger">
    <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-3xl"
      :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">⊞</div>

    <h2 class="font-display text-xl font-bold mb-1" :style="{ color: 'var(--text)' }">Ready for Exam?</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-2)' }">Review your mock settings before entering the exam hall.</p>

    <AppCard padding="lg" class="text-left mb-6">
      <div class="space-y-3">
        <div class="flex justify-between text-sm">
          <span :style="{ color: 'var(--text-3)' }">Type</span>
          <span class="font-medium" :style="{ color: 'var(--text)' }">{{ mockType }}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span :style="{ color: 'var(--text-3)' }">Subject</span>
          <span class="font-medium" :style="{ color: 'var(--text)' }">{{ subject }}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span :style="{ color: 'var(--text-3)' }">Questions</span>
          <span class="font-medium" :style="{ color: 'var(--text)' }">{{ questionCount }}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span :style="{ color: 'var(--text-3)' }">Duration</span>
          <span class="font-medium" :style="{ color: 'var(--text)' }">{{ durationMinutes }} minutes</span>
        </div>
        <div v-if="readinessBand" class="flex justify-between text-sm items-center">
          <span :style="{ color: 'var(--text-3)' }">Your Readiness</span>
          <AppBadge :color="readinessBand === 'strong' ? 'success' : readinessBand === 'developing' ? 'gold' : 'danger'" size="sm">
            {{ readinessBand }}
          </AppBadge>
        </div>
      </div>
    </AppCard>

    <div class="space-y-3">
      <p class="text-xs" :style="{ color: 'var(--text-3)' }">
        Once you enter, the timer starts. No pausing in strict mock mode.
      </p>
      <AppButton variant="danger" size="lg" class="w-full" @click="$emit('start')">Enter Exam Hall →</AppButton>
      <AppButton variant="ghost" @click="$emit('cancel')">Not yet</AppButton>
    </div>
  </div>
</template>
