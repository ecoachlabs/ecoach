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
    <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Side-by-Side Comparison</h3>

    <div class="mb-4 grid grid-cols-2 gap-4">
      <AppCard padding="md">
        <div class="mb-2 flex items-center gap-2">
          <h4 class="text-sm font-bold" :style="{ color: 'var(--accent)' }">{{ conceptA.title }}</h4>
          <AppBadge color="accent" size="xs">{{ conceptA.type }}</AppBadge>
        </div>
        <p class="mb-3 text-xs leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ conceptA.meaning }}</p>
        <ul class="space-y-1">
          <li v-for="feature in conceptA.features" :key="feature" class="flex items-start gap-1.5 text-[10px]" :style="{ color: 'var(--text-2)' }">
            <span :style="{ color: 'var(--accent)' }">+</span> {{ feature }}
          </li>
        </ul>
      </AppCard>

      <AppCard padding="md">
        <div class="mb-2 flex items-center gap-2">
          <h4 class="text-sm font-bold" :style="{ color: 'var(--gold)' }">{{ conceptB.title }}</h4>
          <AppBadge color="gold" size="xs">{{ conceptB.type }}</AppBadge>
        </div>
        <p class="mb-3 text-xs leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ conceptB.meaning }}</p>
        <ul class="space-y-1">
          <li v-for="feature in conceptB.features" :key="feature" class="flex items-start gap-1.5 text-[10px]" :style="{ color: 'var(--text-2)' }">
            <span :style="{ color: 'var(--gold)' }">+</span> {{ feature }}
          </li>
        </ul>
      </AppCard>
    </div>

    <AppCard v-if="sharedFeatures?.length" padding="sm" class="mb-3">
      <p class="mb-1.5 text-[10px] font-semibold uppercase tracking-wider" :style="{ color: 'var(--success)' }">What they share</p>
      <div class="flex flex-wrap gap-1">
        <span
          v-for="feature in sharedFeatures"
          :key="feature"
          class="rounded-full bg-emerald-50 px-2 py-0.5 text-[10px] text-emerald-700"
        >
          {{ feature }}
        </span>
      </div>
    </AppCard>

    <AppCard v-if="confusionPoints?.length" padding="sm">
      <p class="mb-1.5 text-[10px] font-semibold uppercase tracking-wider" :style="{ color: 'var(--danger)' }">Common confusion</p>
      <ul class="space-y-1">
        <li v-for="point in confusionPoints" :key="point" class="flex items-start gap-1.5 text-[10px] text-red-600">
          <span>!</span> {{ point }}
        </li>
      </ul>
    </AppCard>
  </div>
</template>
