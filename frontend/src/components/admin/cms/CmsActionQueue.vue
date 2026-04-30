<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import type { CmsActionItem } from './types'

defineProps<{
  items: CmsActionItem[]
  emptyText?: string
}>()

defineEmits<{
  open: [item: CmsActionItem]
}>()
</script>

<template>
  <div class="space-y-3">
    <div
      v-for="item in items"
      :key="item.key"
      class="rounded-lg border p-3"
      :style="{ backgroundColor: 'var(--paper)', borderColor: 'var(--border-soft)' }"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="flex items-center gap-2 mb-1">
            <AppBadge :color="item.tone === 'danger' ? 'danger' : item.tone === 'review' ? 'gold' : 'muted'" size="xs">
              {{ item.tone === 'danger' ? 'Issue' : item.tone === 'review' ? 'Review' : 'Info' }}
            </AppBadge>
            <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
          </div>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ item.summary }}</p>
        </div>
        <AppButton v-if="item.actionLabel" variant="ghost" size="sm" @click="$emit('open', item)">
          {{ item.actionLabel }}
        </AppButton>
      </div>
    </div>

    <p v-if="!items.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
      {{ emptyText ?? 'No admin actions need attention.' }}
    </p>
  </div>
</template>
