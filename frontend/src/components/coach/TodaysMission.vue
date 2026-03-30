<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  subtitle: string
  estimatedMinutes?: number
  topicName?: string
  reason?: string
  sessionType?: string
}>()

defineEmits<{ start: [] }>()
</script>

<template>
  <AppCard padding="md" glow="accent" class="relative overflow-hidden">
    <div class="absolute top-0 left-0 w-1 h-full" :style="{ backgroundColor: 'var(--accent)' }" />
    <div class="flex items-start justify-between gap-3">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <AppBadge color="accent" size="xs" dot>Today's Mission</AppBadge>
          <AppBadge v-if="sessionType" color="muted" size="xs">{{ sessionType }}</AppBadge>
        </div>
        <h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h3>
        <p class="text-xs mt-0.5" :style="{ color: 'var(--text-2)' }">{{ subtitle }}</p>
        <p v-if="reason" class="text-[10px] mt-1 italic" :style="{ color: 'var(--text-3)' }">{{ reason }}</p>
      </div>
      <div class="flex flex-col items-end gap-2 shrink-0">
        <span v-if="estimatedMinutes" class="text-[10px] font-medium px-2 py-0.5 rounded-full"
          :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">~{{ estimatedMinutes }}m</span>
        <AppButton variant="primary" size="sm" @click="$emit('start')">Start →</AppButton>
      </div>
    </div>
  </AppCard>
</template>
