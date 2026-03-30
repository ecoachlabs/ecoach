<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  year: string
  subject: string
  questionCount: number
  trapDensity?: number
  recallRatio?: number
  dominantFamilies?: string[]
}>()

defineEmits<{ click: [] }>()
</script>

<template>
  <AppCard hover padding="md" @click="$emit('click')">
    <div class="flex items-start justify-between mb-3">
      <div>
        <h3 class="text-base font-display font-bold" :style="{ color: 'var(--text)' }">{{ year }}</h3>
        <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ subject }} · {{ questionCount }} questions</p>
      </div>
      <AppBadge color="accent" size="sm">Paper DNA</AppBadge>
    </div>

    <div class="space-y-2">
      <div v-if="trapDensity !== undefined">
        <div class="flex items-center justify-between text-[10px] mb-1">
          <span :style="{ color: 'var(--text-3)' }">Trap Density</span>
          <span class="font-medium" :style="{ color: trapDensity > 50 ? 'var(--danger)' : 'var(--text-2)' }">{{ trapDensity }}%</span>
        </div>
        <AppProgress :value="trapDensity" size="sm" :color="trapDensity > 50 ? 'danger' : 'accent'" />
      </div>
      <div v-if="recallRatio !== undefined">
        <div class="flex items-center justify-between text-[10px] mb-1">
          <span :style="{ color: 'var(--text-3)' }">Recall vs Reasoning</span>
          <span class="font-medium" :style="{ color: 'var(--text-2)' }">{{ recallRatio }}% recall</span>
        </div>
        <AppProgress :value="recallRatio" size="sm" color="gold" />
      </div>
    </div>

    <div v-if="dominantFamilies?.length" class="mt-3 flex flex-wrap gap-1">
      <AppBadge v-for="f in dominantFamilies.slice(0, 3)" :key="f" color="muted" size="xs">{{ f }}</AppBadge>
    </div>
  </AppCard>
</template>
