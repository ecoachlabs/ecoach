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
  definition: { icon: '📖', color: 'accent' },
  formula: { icon: '∑', color: 'warm' },
  concept: { icon: '💡', color: 'gold' },
}
</script>

<template>
  <AppCard hover padding="sm" @click="$emit('click')">
    <div class="flex items-start gap-3">
      <div class="w-9 h-9 rounded-lg flex items-center justify-center text-base shrink-0"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">
        {{ typeConfig[type]?.icon || '●' }}
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <p class="text-sm font-semibold truncate" :style="{ color: 'var(--text)' }">{{ title }}</p>
          <AppBadge :color="(typeConfig[type]?.color as any) || 'muted'" size="xs">{{ type }}</AppBadge>
        </div>
        <p v-if="quickMeaning" class="text-[11px] mt-0.5 line-clamp-1" :style="{ color: 'var(--text-3)' }">
          {{ quickMeaning }}
        </p>
        <p v-if="topic" class="text-[10px] mt-0.5" :style="{ color: 'var(--text-3)' }">{{ topic }}</p>
      </div>
    </div>
  </AppCard>
</template>
