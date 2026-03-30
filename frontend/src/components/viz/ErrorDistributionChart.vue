<script setup lang="ts">
import { computed } from 'vue'
import { getErrorDisplay } from '@/utils/error-types'

const props = defineProps<{
  errors: { type: string; count: number }[]
}>()

const total = computed(() => props.errors.reduce((sum, e) => sum + e.count, 0))
const sorted = computed(() => [...props.errors].sort((a, b) => b.count - a.count))
</script>

<template>
  <div class="space-y-2">
    <div v-for="error in sorted" :key="error.type" class="flex items-center gap-3">
      <div class="w-3 h-3 rounded-full shrink-0" :style="{ backgroundColor: getErrorDisplay(error.type).color }" />
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs font-medium truncate" :style="{ color: 'var(--text)' }">{{ getErrorDisplay(error.type).label }}</span>
          <span class="text-[10px] font-bold tabular-nums" :style="{ color: 'var(--text-3)' }">{{ error.count }}</span>
        </div>
        <div class="h-1.5 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
          <div class="h-full rounded-full transition-all" :style="{
            width: (total > 0 ? (error.count / total) * 100 : 0) + '%',
            backgroundColor: getErrorDisplay(error.type).color,
            transitionDuration: 'var(--dur-slow)',
          }" />
        </div>
      </div>
    </div>
  </div>
</template>
