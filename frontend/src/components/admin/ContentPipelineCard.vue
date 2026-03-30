<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  stage: 'raw' | 'parsed' | 'verified' | 'published'
  count: number
  label: string
}>()

const stageColors: Record<string, { bg: string; text: string }> = {
  raw: { bg: '#fef3c7', text: '#92400e' },
  parsed: { bg: '#dbeafe', text: '#1e40af' },
  verified: { bg: '#d1fae5', text: '#065f46' },
  published: { bg: '#ccfbf1', text: '#0d9488' },
}
</script>

<template>
  <AppCard padding="md" class="text-center">
    <p class="font-display text-3xl font-bold tabular-nums" :style="{ color: stageColors[stage]?.text || 'var(--text)' }">{{ count }}</p>
    <AppBadge class="mt-2" size="sm"
      :style="{ backgroundColor: stageColors[stage]?.bg, color: stageColors[stage]?.text }">
      {{ label }}
    </AppBadge>
  </AppCard>
</template>
