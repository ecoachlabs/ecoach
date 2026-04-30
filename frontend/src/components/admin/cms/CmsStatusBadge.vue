<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'

const props = defineProps<{
  status: string | null | undefined
  size?: 'xs' | 'sm' | 'md'
}>()

function normalizedStatus() {
  return (props.status || 'unknown').replaceAll('_', ' ')
}

function badgeColor(): 'accent' | 'warm' | 'gold' | 'success' | 'danger' | 'muted' | 'ember' {
  const status = (props.status || '').toLowerCase()
  if (['approved', 'published', 'active', 'installed', 'complete', 'completed'].includes(status)) return 'success'
  if (['failed', 'blocked', 'rejected', 'error'].includes(status)) return 'danger'
  if (['pending', 'needs_review', 'queued', 'draft', 'preview'].includes(status)) return 'gold'
  if (['running', 'extracting', 'processing'].includes(status)) return 'accent'
  return 'muted'
}
</script>

<template>
  <AppBadge :color="badgeColor()" :size="size ?? 'xs'">
    {{ normalizedStatus() }}
  </AppBadge>
</template>
