<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  timestamp: string
  action: string
  entity: string
  entityId?: number
  actor?: string
  details?: string
}>()
</script>

<template>
  <div class="flex items-start gap-3 py-2 border-b" :style="{ borderColor: 'var(--card-border)' }">
    <span class="text-[10px] font-mono tabular-nums shrink-0 mt-0.5" :style="{ color: 'var(--text-3)' }">
      {{ new Date(timestamp).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' }) }}
    </span>
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-xs font-medium" :style="{ color: 'var(--text)' }">{{ action }}</span>
        <AppBadge color="muted" size="xs">{{ entity }}</AppBadge>
        <span v-if="entityId" class="text-[10px] font-mono" :style="{ color: 'var(--text-3)' }">#{{ entityId }}</span>
      </div>
      <p v-if="details" class="text-[10px] mt-0.5 truncate" :style="{ color: 'var(--text-3)' }">{{ details }}</p>
    </div>
    <span v-if="actor" class="text-[10px] shrink-0" :style="{ color: 'var(--text-3)' }">{{ actor }}</span>
  </div>
</template>
