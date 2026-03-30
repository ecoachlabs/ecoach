<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  items: { id: number; type: string; title: string; reason: string; urgency: 'high' | 'medium' | 'low'; actionLabel: string }[]
}>()

defineEmits<{ action: [itemId: number] }>()

const urgencyColors: Record<string, string> = { high: 'danger', medium: 'warm', low: 'muted' }
const typeIcons: Record<string, string> = {
  review: '↻', practice: '✎', learn: '📖', repair: '🔧', memory: '🧠', exam: '📝',
}
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Recommended for You</h3>
    <div class="space-y-2">
      <AppCard v-for="item in items" :key="item.id" padding="sm" hover @click="$emit('action', item.id)">
        <div class="flex items-start gap-3">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"
            :style="{ backgroundColor: 'var(--primary-light)' }">
            {{ typeIcons[item.type] || '●' }}
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-0.5">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ item.title }}</p>
              <AppBadge :color="(urgencyColors[item.urgency] as any)" size="xs">{{ item.urgency }}</AppBadge>
            </div>
            <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ item.reason }}</p>
          </div>
          <AppButton variant="ghost" size="sm">{{ item.actionLabel }}</AppButton>
        </div>
      </AppCard>
      <p v-if="!items.length" class="text-xs text-center py-6" :style="{ color: 'var(--text-3)' }">
        Study more to see personalized recommendations.
      </p>
    </div>
  </div>
</template>
