<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  statement: string
  conceptA: string
  conceptB: string
  active: boolean
}>()

defineEmits<{ answer: [choice: 'both' | 'a_only' | 'b_only' | 'neither'] }>()

const choices = [
  { key: 'both' as const, label: 'True for Both', color: 'var(--accent)' },
  { key: 'a_only' as const, label: 'A Only', color: 'var(--warm)' },
  { key: 'b_only' as const, label: 'B Only', color: 'var(--gold)' },
  { key: 'neither' as const, label: 'False for Both', color: 'var(--danger)' },
]
</script>

<template>
  <div data-mode="pressure">
    <!-- Danger aesthetic header -->
    <div class="text-center mb-4">
      <AppBadge color="danger" size="sm">⚠ Similarity Trap</AppBadge>
      <div class="flex items-center justify-center gap-4 mt-3 text-xs font-semibold">
        <span :style="{ color: 'var(--warm)' }">{{ conceptA }}</span>
        <span :style="{ color: 'var(--text-3)' }">vs</span>
        <span :style="{ color: 'var(--gold)' }">{{ conceptB }}</span>
      </div>
    </div>

    <!-- Statement card (deliberately deceptive wording) -->
    <AppCard padding="lg" class="mb-6 text-center" :style="{ boxShadow: '0 0 20px rgba(239,68,68,0.1)' }">
      <p class="text-base font-medium leading-relaxed" :style="{ color: 'var(--text)' }">{{ statement }}</p>
    </AppCard>

    <!-- 4-option classification -->
    <div class="grid grid-cols-2 gap-3">
      <button v-for="c in choices" :key="c.key"
        class="py-4 rounded-[var(--radius-lg)] border-2 text-sm font-semibold transition-all active:scale-95"
        :style="{ borderColor: c.color, color: c.color, backgroundColor: c.color + '10' }"
        :disabled="!active"
        @click="$emit('answer', c.key)">
        {{ c.label }}
      </button>
    </div>
  </div>
</template>
