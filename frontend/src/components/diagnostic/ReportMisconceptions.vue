<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  misconceptions: { topic: string; description: string; severity: string; frequency: number }[]
}>()
</script>

<template>
  <div class="space-y-2">
    <p v-if="!misconceptions.length" class="text-sm text-center py-6" :style="{color:'var(--text-3)'}">No misconceptions detected.</p>
    <AppCard v-for="m in misconceptions" :key="m.description" padding="sm">
      <div class="flex items-start gap-3">
        <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"
          :class="m.severity === 'high' ? 'bg-red-100 text-red-600' : 'bg-amber-100 text-amber-600'">🚫</div>
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-0.5">
            <p class="text-sm font-medium" :style="{color:'var(--text)'}">{{ m.description }}</p>
            <AppBadge :color="m.severity === 'high' ? 'danger' : 'warm'" size="xs">{{ m.severity }}</AppBadge>
          </div>
          <p class="text-[10px]" :style="{color:'var(--text-3)'}">{{ m.topic }} · Occurred {{ m.frequency }} times</p>
        </div>
      </div>
    </AppCard>
  </div>
</template>
