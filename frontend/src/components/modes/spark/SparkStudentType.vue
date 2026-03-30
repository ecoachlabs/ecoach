<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  type: string
  description: string
}>()

const typeConfig: Record<string, { icon: string; color: string; label: string }> = {
  overwhelmed: { icon: '😵', color: 'var(--danger)', label: 'Overwhelmed' },
  bored: { icon: '😴', color: 'var(--gold)', label: 'Bored' },
  defeated: { icon: '😞', color: 'var(--warm)', label: 'Defeated' },
  distracted: { icon: '📱', color: 'var(--accent)', label: 'Distracted' },
  disconnected: { icon: '🤷', color: 'var(--text-3)', label: 'Disconnected' },
  nervous: { icon: '😰', color: '#7c3aed', label: 'Nervous' },
}
</script>

<template>
  <AppCard padding="lg" class="text-center">
    <div class="text-4xl mb-3">{{ typeConfig[type]?.icon || '😊' }}</div>
    <h3 class="font-display text-base font-semibold mb-1" :style="{color: typeConfig[type]?.color || 'var(--text)'}">
      {{ typeConfig[type]?.label || type }}
    </h3>
    <p class="text-xs" :style="{color:'var(--text-2)'}">{{ description }}</p>
  </AppCard>
</template>
