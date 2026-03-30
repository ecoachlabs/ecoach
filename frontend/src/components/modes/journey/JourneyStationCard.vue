<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

defineProps<{
  stationName: string
  topicName: string
  masteryState: string
  locked: boolean
  current: boolean
  completed: boolean
  questionsCount?: number
}>()

defineEmits<{ enter: [] }>()
</script>

<template>
  <AppCard :hover="!locked" padding="md"
    :class="[current ? 'ring-2 ring-[var(--accent)]' : '', locked ? 'opacity-40' : '']"
    @click="!locked && $emit('enter')">
    <div class="flex items-center gap-3">
      <!-- Station indicator -->
      <div class="w-10 h-10 rounded-xl flex items-center justify-center text-sm font-bold shrink-0"
        :class="completed ? 'bg-emerald-100 text-emerald-600' : current ? 'bg-[var(--accent-light)] text-[var(--accent)]' : locked ? '' : ''"
        :style="!completed && !current ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
        {{ completed ? '✓' : locked ? '🔒' : current ? '▶' : '○' }}
      </div>

      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <p class="text-sm font-semibold truncate" :style="{ color: locked ? 'var(--text-3)' : 'var(--text)' }">{{ stationName }}</p>
          <MasteryBadge v-if="!locked" :state="masteryState" size="sm" />
        </div>
        <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ topicName }}</p>
      </div>

      <AppBadge v-if="current" color="accent" size="xs">Current</AppBadge>
      <AppBadge v-else-if="completed" color="success" size="xs">Done</AppBadge>
    </div>
  </AppCard>
</template>
