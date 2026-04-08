<script setup lang="ts">
defineProps<{
  leftLabel: string
  rightLabel: string
  leftValue: number | string
  rightValue: number | string
  leftSubtext?: string
  rightSubtext?: string
  highlight?: 'left' | 'right' | 'none'
}>()
</script>

<template>
  <div
    class="rounded-[var(--radius-lg)] overflow-hidden"
    :style="{ backgroundColor: 'var(--card-bg)' }"
  >
    <div class="grid grid-cols-3">
      <!-- Left -->
      <div class="p-4 text-center" :class="highlight === 'left' ? 'bg-[var(--primary-light)]' : ''">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">
          {{ leftLabel }}
        </p>
        <p class="font-display text-2xl font-bold tabular-nums" :style="{ color: 'var(--text)' }">
          {{ leftValue }}
        </p>
        <p v-if="leftSubtext" class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">{{ leftSubtext }}</p>
      </div>

      <!-- Divider / VS -->
      <div class="flex items-center justify-center" :style="{ backgroundColor: 'var(--primary-light)' }">
        <span class="text-xs font-bold" :style="{ color: 'var(--text-3)' }">vs</span>
      </div>

      <!-- Right -->
      <div class="p-4 text-center" :class="highlight === 'right' ? 'bg-[var(--primary-light)]' : ''">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">
          {{ rightLabel }}
        </p>
        <p class="font-display text-2xl font-bold tabular-nums" :style="{ color: 'var(--primary)' }">
          {{ rightValue }}
        </p>
        <p v-if="rightSubtext" class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">{{ rightSubtext }}</p>
      </div>
    </div>
  </div>
</template>
