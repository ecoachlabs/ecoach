<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  topicName: string
  sessionType: string
  questionCount: number
  estimatedMinutes: number
  reason: string
  phase: string
}>()

defineEmits<{ start: [] }>()
</script>

<template>
  <AppCard padding="lg" glow="accent">
    <div class="flex items-center gap-2 mb-3">
      <AppBadge color="accent" size="xs" dot>Journey Mission</AppBadge>
      <AppBadge color="muted" size="xs">{{ phase }}</AppBadge>
    </div>
    <h3 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">{{ title }}</h3>
    <p class="text-sm mb-2" :style="{ color: 'var(--text-2)' }">{{ topicName }}</p>
    <p class="text-xs italic mb-4" :style="{ color: 'var(--text-3)' }">{{ reason }}</p>
    <div class="flex items-center gap-4 mb-4 text-xs" :style="{ color: 'var(--text-3)' }">
      <span>{{ questionCount }} questions</span>
      <span>~{{ estimatedMinutes }} min</span>
      <AppBadge color="accent" size="xs">{{ sessionType }}</AppBadge>
    </div>
    <AppButton variant="primary" size="lg" @click="$emit('start')">Start Mission →</AppButton>
  </AppCard>
</template>
