<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  activities: { date: string; type: string; description: string; result?: string }[]
}>()
</script>

<template>
  <div class="space-y-0">
    <div v-for="(activity, i) in activities" :key="i" class="flex gap-3">
      <!-- Timeline line + dot -->
      <div class="flex flex-col items-center shrink-0">
        <div class="w-2.5 h-2.5 rounded-full border-2 mt-1.5"
          :class="activity.type === 'session' ? 'bg-[var(--accent)] border-[var(--accent)]' :
                  activity.type === 'diagnostic' ? 'bg-[var(--gold)] border-[var(--gold)]' :
                  activity.type === 'mock' ? 'bg-[var(--danger)] border-[var(--danger)]' :
                  'border-[var(--border-strong)]'" />
        <div v-if="i < activities.length - 1" class="w-px flex-1 min-h-[24px]" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <!-- Content -->
      <div class="pb-4 flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ activity.description }}</p>
          <AppBadge v-if="activity.result" :color="activity.result === 'good' ? 'success' : 'muted'" size="xs">{{ activity.result }}</AppBadge>
        </div>
        <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ activity.date }}</p>
      </div>
    </div>
    <p v-if="!activities.length" class="text-xs text-center py-6" :style="{ color: 'var(--text-3)' }">No recent activity.</p>
  </div>
</template>
