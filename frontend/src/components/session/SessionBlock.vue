<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  type: 'quiz' | 'explanation' | 'drill' | 'worked_example' | 'reflection' | 'timed_check' | 'recall' | 'memory_anchor'
  title?: string
  active: boolean
  completed: boolean
}>()

const blockConfig: Record<string, { icon: string; label: string; color: string }> = {
  quiz: { icon: '✎', label: 'Quiz', color: 'var(--accent)' },
  explanation: { icon: '📖', label: 'Explanation', color: 'var(--gold)' },
  drill: { icon: '⚡', label: 'Drill', color: 'var(--warm)' },
  worked_example: { icon: '📋', label: 'Worked Example', color: 'var(--accent)' },
  reflection: { icon: '🪞', label: 'Reflection', color: 'var(--gold)' },
  timed_check: { icon: '⏱', label: 'Timed Check', color: 'var(--danger)' },
  recall: { icon: '🧠', label: 'Recall', color: 'var(--accent)' },
  memory_anchor: { icon: '⚓', label: 'Memory Anchor', color: 'var(--success)' },
}
</script>

<template>
  <div class="flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] transition-all"
    :class="active ? 'bg-[var(--primary-light)] ring-1 ring-[var(--accent)]' : completed ? 'opacity-50' : ''"
    :style="{ backgroundColor: !active ? 'var(--card-bg)' : undefined }">
    <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"
      :class="completed ? 'bg-emerald-100 text-emerald-600' : ''"
      :style="!completed ? { backgroundColor: 'var(--primary-light)', color: blockConfig[type]?.color || 'var(--text-3)' } : {}">
      {{ completed ? '✓' : blockConfig[type]?.icon || '●' }}
    </div>
    <div class="flex-1 min-w-0">
      <p class="text-xs font-medium" :style="{ color: active ? 'var(--accent)' : 'var(--text)' }">
        {{ title || blockConfig[type]?.label || type }}
      </p>
    </div>
    <AppBadge v-if="active" color="accent" size="xs">Active</AppBadge>
  </div>
</template>
