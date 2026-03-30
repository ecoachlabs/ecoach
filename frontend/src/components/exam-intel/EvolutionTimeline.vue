<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  topicName: string
  entries: { year: string; description: string; changeType: 'new' | 'modified' | 'disappeared' | 'replaced' }[]
}>()

const changeColors: Record<string, { bg: string; text: string; icon: string }> = {
  new: { bg: 'bg-emerald-50', text: 'text-emerald-700', icon: '+' },
  modified: { bg: 'bg-amber-50', text: 'text-amber-700', icon: '↻' },
  disappeared: { bg: 'bg-red-50', text: 'text-red-600', icon: '−' },
  replaced: { bg: 'bg-blue-50', text: 'text-blue-700', icon: '→' },
}
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-4" :style="{color:'var(--text-3)'}">{{ topicName }} — Question Evolution</h3>
    <div class="space-y-0">
      <div v-for="(entry, i) in entries" :key="i" class="flex gap-3">
        <div class="flex flex-col items-center shrink-0">
          <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold"
            :class="[changeColors[entry.changeType]?.bg, changeColors[entry.changeType]?.text]">
            {{ changeColors[entry.changeType]?.icon }}
          </div>
          <div v-if="i < entries.length - 1" class="w-px flex-1 min-h-[16px]" :style="{backgroundColor:'var(--border-soft)'}" />
        </div>
        <div class="pb-4 flex-1">
          <p class="text-xs font-bold" :style="{color:'var(--text)'}">{{ entry.year }}</p>
          <p class="text-xs mt-0.5" :style="{color:'var(--text-2)'}">{{ entry.description }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
