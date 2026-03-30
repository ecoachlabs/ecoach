<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppCollapsible from '@/components/ui/AppCollapsible.vue'

defineProps<{
  blockNumber: number
  title: string
  content: string
  type: string
  expandedByDefault?: boolean
}>()

const blockIcons: Record<string, string> = {
  hero: '🎯', snapshot: '📌', why: '❓', prerequisites: '🔗', simple: '💡',
  structured: '📐', visual: '🎨', examples: '📋', patterns: '🔄', mistakes: '⚠',
  misconceptions: '🚫', exam: '📝', checks: '✓', practice: '✎', related: '↔', next: '→',
}
</script>

<template>
  <AppCollapsible :title="`${blockIcons[type] || '●'} ${title}`" :default-open="expandedByDefault">
    <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ content }}</p>
    <slot />
  </AppCollapsible>
</template>
