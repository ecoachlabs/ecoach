<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  contentType: string
  state: string
  topic?: string
  subtext?: string
}>()

defineEmits<{ click: []; action: [type: string] }>()

const typeIcons: Record<string, string> = {
  question: '❓', formula: '∑', definition: '📖', worked_example: '📋', concept: '💡',
  note: '📝', diagram: '📊', flashcard: '🃏', past_paper: '📄', revision_pack: '📦',
  summary: '📑', audio: '🔊', video: '🎬',
}

const stateColors: Record<string, string> = {
  new: 'accent', saved: 'muted', studied: 'gold', understood: 'success',
  weak: 'danger', revisit: 'warm', fading: 'warm', mastered: 'success',
}
</script>

<template>
  <AppCard hover padding="sm" @click="$emit('click')">
    <div class="flex items-start gap-3">
      <div class="w-9 h-9 rounded-lg flex items-center justify-center text-base shrink-0"
        :style="{ backgroundColor: 'var(--primary-light)' }">
        {{ typeIcons[contentType] || '●' }}
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ title }}</p>
        <div class="flex items-center gap-2 mt-0.5">
          <span class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ contentType }}</span>
          <span v-if="topic" class="text-[10px]" :style="{ color: 'var(--text-3)' }">· {{ topic }}</span>
        </div>
      </div>
      <AppBadge :color="(stateColors[state] as any) || 'muted'" size="xs">{{ state }}</AppBadge>
    </div>
  </AppCard>
</template>
