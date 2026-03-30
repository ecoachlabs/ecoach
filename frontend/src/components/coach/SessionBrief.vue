<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  subtitle: string
  questionCount?: number
  estimatedMinutes?: number
  sessionType?: string
  topicName?: string
  reason?: string
}>()

defineEmits<{ start: []; cancel: [] }>()
</script>

<template>
  <AppCard padding="lg" glow="accent">
    <div class="text-center mb-6">
      <div class="w-14 h-14 rounded-2xl mx-auto mb-3 flex items-center justify-center text-2xl"
        :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
        ✎
      </div>
      <h2 class="font-display text-xl font-semibold mb-1" :style="{ color: 'var(--text)' }">{{ title }}</h2>
      <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ subtitle }}</p>
    </div>

    <!-- Session details -->
    <div class="flex items-center justify-center gap-4 mb-4 text-xs" :style="{ color: 'var(--text-3)' }">
      <span v-if="questionCount">{{ questionCount }} questions</span>
      <span v-if="estimatedMinutes">~{{ estimatedMinutes }} min</span>
      <AppBadge v-if="sessionType" color="accent" size="xs">{{ sessionType }}</AppBadge>
    </div>

    <div v-if="topicName" class="text-center mb-4">
      <AppBadge color="warm" size="sm">{{ topicName }}</AppBadge>
    </div>

    <div v-if="reason" class="text-center mb-6">
      <p class="text-xs italic" :style="{ color: 'var(--text-3)' }">{{ reason }}</p>
    </div>

    <div class="flex items-center justify-center gap-3">
      <AppButton variant="primary" size="lg" @click="$emit('start')">Begin Session →</AppButton>
      <AppButton variant="ghost" @click="$emit('cancel')">Not now</AppButton>
    </div>
  </AppCard>
</template>
