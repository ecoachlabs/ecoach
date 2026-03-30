<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  familyName: string
  familyCode?: string
  firstSeen?: string
  repeatCount: number
  peakYears?: string[]
  topicName?: string
  yourAccuracy?: number
}>()

defineEmits<{ click: [] }>()
</script>

<template>
  <AppCard hover padding="md" @click="$emit('click')">
    <div class="flex items-start gap-3">
      <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg shrink-0"
        :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
        🧬
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <p class="text-sm font-semibold truncate" :style="{ color: 'var(--text)' }">{{ familyName }}</p>
          <AppBadge v-if="familyCode" color="muted" size="xs">{{ familyCode }}</AppBadge>
        </div>
        <div class="flex items-center gap-3 text-[10px]" :style="{ color: 'var(--text-3)' }">
          <span v-if="firstSeen">Since {{ firstSeen }}</span>
          <span>{{ repeatCount }} appearances</span>
          <span v-if="topicName">{{ topicName }}</span>
        </div>
        <div v-if="yourAccuracy !== undefined" class="mt-1.5 flex items-center gap-1.5">
          <span class="text-[10px] font-medium" :style="{ color: 'var(--text-3)' }">Your accuracy:</span>
          <span class="text-xs font-bold tabular-nums"
            :class="yourAccuracy >= 70 ? 'text-emerald-600' : yourAccuracy >= 40 ? 'text-amber-600' : 'text-red-500'">
            {{ yourAccuracy }}%
          </span>
        </div>
      </div>
    </div>
  </AppCard>
</template>
