<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  conceptA: { title: string; type: string; meaning: string; features: string[] }
  conceptB: { title: string; type: string; meaning: string; features: string[] }
  sharedFeatures?: string[]
  confusionPoints?: string[]
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Side-by-Side Comparison</h3>

    <div class="grid grid-cols-2 gap-4 mb-4">
      <!-- Concept A -->
      <AppCard padding="md">
        <div class="flex items-center gap-2 mb-2">
          <h4 class="text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ conceptA.title }}</h4>
          <AppBadge color="accent" size="xs">{{ conceptA.type }}</AppBadge>
        </div>
        <p class="text-xs leading-relaxed mb-3" :style="{ color: 'var(--text-2)' }">{{ conceptA.meaning }}</p>
        <ul class="space-y-1">
          <li v-for="f in conceptA.features" :key="f" class="text-[10px] flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
            <span :style="{ color: 'var(--accent)' }">●</span> {{ f }}
          </li>
        </ul>
      </AppCard>

      <!-- Concept B -->
      <AppCard padding="md">
        <div class="flex items-center gap-2 mb-2">
          <h4 class="text-sm font-bold" :style="{ color: 'var(--gold)' }">{{ conceptB.title }}</h4>
          <AppBadge color="gold" size="xs">{{ conceptB.type }}</AppBadge>
        </div>
        <p class="text-xs leading-relaxed mb-3" :style="{ color: 'var(--text-2)' }">{{ conceptB.meaning }}</p>
        <ul class="space-y-1">
          <li v-for="f in conceptB.features" :key="f" class="text-[10px] flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
            <span :style="{ color: 'var(--gold)' }">●</span> {{ f }}
          </li>
        </ul>
      </AppCard>
    </div>

    <!-- Shared features -->
    <AppCard v-if="sharedFeatures?.length" padding="sm" class="mb-3">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" :style="{ color: 'var(--success)' }">What they share</p>
      <div class="flex flex-wrap gap-1">
        <span v-for="f in sharedFeatures" :key="f" class="text-[10px] px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-700">{{ f }}</span>
      </div>
    </AppCard>

    <!-- Confusion points -->
    <AppCard v-if="confusionPoints?.length" padding="sm">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" :style="{ color: 'var(--danger)' }">Common confusion</p>
      <ul class="space-y-1">
        <li v-for="c in confusionPoints" :key="c" class="text-[10px] flex items-start gap-1.5 text-red-600">
          <span>⚠</span> {{ c }}
        </li>
      </ul>
    </AppCard>
  </div>
</template>
