<script setup lang="ts">
defineProps<{
  severity: string
  title: string
  description: string
  recommendation?: string
}>()

const severityConfig: Record<string, { bg: string; text: string; border: string; icon: string }> = {
  high: { bg: 'bg-red-50', text: 'text-red-700', border: 'border-red-200', icon: '⚠' },
  medium: { bg: 'bg-amber-50', text: 'text-amber-700', border: 'border-amber-200', icon: '◈' },
  low: { bg: 'bg-blue-50', text: 'text-blue-700', border: 'border-blue-200', icon: 'ℹ' },
}
</script>

<template>
  <div
    class="px-4 py-3 rounded-[var(--radius-md)] border"
    :class="[
      severityConfig[severity]?.bg || 'bg-stone-50',
      severityConfig[severity]?.border || 'border-stone-200',
    ]"
  >
    <div class="flex items-start gap-2.5">
      <span class="text-base shrink-0">{{ severityConfig[severity]?.icon || '●' }}</span>
      <div class="flex-1">
        <p class="text-sm font-semibold" :class="severityConfig[severity]?.text || 'text-stone-700'">{{ title }}</p>
        <p class="text-xs mt-0.5 opacity-80" :class="severityConfig[severity]?.text || 'text-stone-600'">{{ description }}</p>
        <p v-if="recommendation" class="text-xs mt-2 font-medium" :style="{ color: 'var(--accent)' }">
          → {{ recommendation }}
        </p>
      </div>
    </div>
  </div>
</template>
