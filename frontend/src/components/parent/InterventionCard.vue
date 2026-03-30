<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  title: string
  description: string
  status: 'active' | 'completed' | 'pending'
  category: 'system' | 'parent' | 'suggestion'
  urgency?: string
}>()

const statusConfig: Record<string, { color: string; label: string }> = {
  active: { color: 'accent', label: 'Active' },
  completed: { color: 'success', label: 'Completed' },
  pending: { color: 'warm', label: 'Pending' },
}

const categoryConfig: Record<string, { label: string; icon: string }> = {
  system: { label: 'System handling', icon: '⚙' },
  parent: { label: 'Needs your action', icon: '👤' },
  suggestion: { label: 'You could do', icon: '💡' },
}
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-start gap-3">
      <span class="text-lg mt-0.5">{{ categoryConfig[category]?.icon || '●' }}</span>
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
          <h4 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h4>
          <AppBadge :color="(statusConfig[status]?.color as any) || 'muted'" size="xs">{{ statusConfig[status]?.label }}</AppBadge>
        </div>
        <p class="text-xs" :style="{ color: 'var(--text-2)' }">{{ description }}</p>
        <p class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">{{ categoryConfig[category]?.label }}</p>
      </div>
    </div>
  </AppCard>
</template>
