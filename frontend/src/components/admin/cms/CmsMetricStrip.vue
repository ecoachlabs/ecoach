<script setup lang="ts">
import type { CmsMetricItem } from './types'

defineProps<{
  items: CmsMetricItem[]
}>()

function colorFor(tone?: CmsMetricItem['tone']) {
  if (tone === 'good') return 'var(--success)'
  if (tone === 'review') return 'var(--gold)'
  if (tone === 'danger') return 'var(--warm)'
  return 'var(--ink)'
}
</script>

<template>
  <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6 gap-3">
    <div
      v-for="item in items"
      :key="item.label"
      class="rounded-lg border px-4 py-3"
      :style="{ backgroundColor: 'var(--surface)', borderColor: 'var(--border-soft)' }"
    >
      <p class="font-display text-2xl font-bold tabular-nums" :style="{ color: colorFor(item.tone) }">
        {{ item.value }}
      </p>
      <p class="text-[10px] uppercase tracking-wide mt-1" :style="{ color: 'var(--ink-muted)' }">{{ item.label }}</p>
      <p v-if="item.caption" class="text-[10px] mt-1 truncate" :style="{ color: 'var(--ink-muted)' }">{{ item.caption }}</p>
    </div>
  </div>
</template>
