<script setup lang="ts">
import type { CoachNextActionDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  action: CoachNextActionDto
}>()

defineEmits<{ start: [] }>()

const actionIcons: Record<string, string> = {
  continue_onboarding: '👋',
  select_subjects: '📚',
  resolve_content: '📦',
  start_diagnostic: '◈',
  generate_plan: '🗺',
  start_today_mission: '◉',
  resume_mission: '▶',
  review_results: '📊',
  start_repair: '🔧',
  adjust_plan: '↻',
  view_overview: '◐',
}
</script>

<template>
  <AppCard class="relative overflow-hidden" glow="accent" padding="lg">
    <!-- Accent stripe -->
    <div class="absolute top-0 left-0 w-1 h-full rounded-l-[var(--radius-lg)]" :style="{ backgroundColor: 'var(--accent)' }" />

    <div class="flex items-start justify-between gap-4">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-lg">{{ actionIcons[action.action_type] || '◉' }}</span>
          <AppBadge color="accent" size="xs" dot>Coach Recommendation</AppBadge>
        </div>
        <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">
          {{ action.title }}
        </h2>
        <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">
          {{ action.subtitle }}
        </p>
      </div>

      <span v-if="action.estimated_minutes" class="text-xs font-medium px-2.5 py-1 rounded-full shrink-0"
        :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
        ~{{ action.estimated_minutes }} min
      </span>
    </div>

    <div class="mt-4 flex items-center gap-3">
      <AppButton variant="primary" @click="$emit('start')">Start Now →</AppButton>
      <AppButton variant="ghost" size="sm">Why this?</AppButton>
    </div>
  </AppCard>
</template>
