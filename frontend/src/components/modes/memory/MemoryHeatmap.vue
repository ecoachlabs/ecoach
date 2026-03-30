<script setup lang="ts">
defineProps<{
  topics: { name: string; strength: number; status: 'strong' | 'vulnerable' | 'fading' | 'critical' | 'recovered' }[]
}>()

const statusGlow: Record<string, { bg: string; glow: string }> = {
  strong: { bg: '#22c55e', glow: '0 0 8px #22c55e60' },
  vulnerable: { bg: '#fbbf24', glow: '0 0 6px #fbbf2440' },
  fading: { bg: '#f59e0b', glow: '0 0 8px #f59e0b50' },
  critical: { bg: '#ef4444', glow: '0 0 10px #ef444460' },
  recovered: { bg: '#10b981', glow: '0 0 8px #10b98150' },
}
</script>

<template>
  <div class="grid grid-cols-4 gap-2">
    <div v-for="topic in topics" :key="topic.name"
      class="p-3 rounded-[var(--radius-md)] text-center transition-all"
      :style="{
        backgroundColor: statusGlow[topic.status]?.bg + '15',
        boxShadow: statusGlow[topic.status]?.glow,
      }">
      <div class="w-6 h-6 rounded-full mx-auto mb-1.5 ember-glow"
        :style="{ backgroundColor: statusGlow[topic.status]?.bg, opacity: 0.3 + (topic.strength / 10000) * 0.7 }" />
      <p class="text-[10px] font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.name }}</p>
      <p class="text-[9px] capitalize" :style="{ color: statusGlow[topic.status]?.bg }">{{ topic.status }}</p>
    </div>
  </div>
</template>
