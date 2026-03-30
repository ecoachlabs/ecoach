<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ContentObjectCard from './ContentObjectCard.vue'

defineProps<{
  title: string
  icon: string
  items: { title: string; contentType: string; state: string; topic?: string }[]
  showCount?: boolean
}>()

defineEmits<{ viewAll: []; itemClick: [index: number] }>()
</script>

<template>
  <div class="mb-6">
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <span class="text-base">{{ icon }}</span>
        <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">{{ title }}</h3>
        <AppBadge v-if="showCount" color="muted" size="xs">{{ items.length }}</AppBadge>
      </div>
      <button v-if="items.length > 3" class="text-[10px] font-medium" :style="{ color: 'var(--accent)' }" @click="$emit('viewAll')">
        View all →
      </button>
    </div>
    <div v-if="items.length" class="space-y-2">
      <ContentObjectCard v-for="(item, i) in items.slice(0, 5)" :key="i"
        v-bind="item" @click="$emit('itemClick', i)" />
    </div>
    <p v-else class="text-xs py-4 text-center" :style="{ color: 'var(--text-3)' }">No items in this shelf yet.</p>
  </div>
</template>
