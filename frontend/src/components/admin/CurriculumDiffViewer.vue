<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  versionA: string
  versionB: string
  changes: { type: 'added' | 'removed' | 'modified'; path: string; detail: string }[]
}>()
</script>

<template>
  <div>
    <div class="flex items-center gap-3 mb-4">
      <AppBadge color="muted" size="sm">{{ versionA }}</AppBadge>
      <span :style="{color:'var(--text-3)'}">→</span>
      <AppBadge color="accent" size="sm">{{ versionB }}</AppBadge>
      <span class="text-xs ml-auto" :style="{color:'var(--text-3)'}">{{ changes.length }} changes</span>
    </div>
    <div class="space-y-1">
      <div v-for="(c, i) in changes" :key="i"
        class="flex items-start gap-2 px-3 py-2 rounded-[var(--radius-sm)] text-xs"
        :class="{
          'bg-emerald-50': c.type === 'added',
          'bg-red-50': c.type === 'removed',
          'bg-amber-50': c.type === 'modified',
        }">
        <span class="font-bold shrink-0"
          :class="c.type === 'added' ? 'text-emerald-600' : c.type === 'removed' ? 'text-red-500' : 'text-amber-600'">
          {{ c.type === 'added' ? '+' : c.type === 'removed' ? '−' : '~' }}
        </span>
        <div class="flex-1 min-w-0">
          <p class="font-medium font-mono truncate" :style="{color:'var(--text)'}">{{ c.path }}</p>
          <p class="text-[10px]" :style="{color:'var(--text-3)'}">{{ c.detail }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
