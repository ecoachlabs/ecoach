<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  entryId: number
  title: string
  type: 'definition' | 'formula' | 'concept' | string
  topic?: string
  quickMeaning?: string
  masteryScore?: number
}>()

defineEmits<{ click: [] }>()

const typeConfig: Record<string, { icon: string; color: string }> = {
  definition: { icon: 'Df', color: 'accent' },
  formula: { icon: 'Fx', color: 'warm' },
  concept: { icon: 'Cn', color: 'gold' },
}
</script>

<template>
  <AppCard hover padding="sm" @click="$emit('click')">
    <div class="flex items-start gap-3">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-[11px] font-bold uppercase tracking-wide"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">
        {{ typeConfig[type]?.icon || 'Tx' }}
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <p class="truncate text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</p>
          <AppBadge :color="(typeConfig[type]?.color as any) || 'muted'" size="xs">{{ type }}</AppBadge>
        </div>
        <p v-if="quickMeaning" class="mt-0.5 line-clamp-1 text-[11px]" :style="{ color: 'var(--text-3)' }">
          {{ quickMeaning }}
        </p>
        <p v-if="topic" class="mt-0.5 text-[10px]" :style="{ color: 'var(--text-3)' }">{{ topic }}</p>
      </div>
    </div>
  </AppCard>
</template>
