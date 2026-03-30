<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  subjects: { name: string; readiness: number; band: string; riskLevel: string; topWeakness?: string }[]
}>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Subject Risk Map</h3>
    <div class="space-y-2">
      <AppCard v-for="s in subjects" :key="s.name" padding="sm">
        <div class="flex items-center gap-3 mb-2">
          <p class="text-sm font-semibold flex-1" :style="{ color: 'var(--text)' }">{{ s.name }}</p>
          <AppBadge :color="s.band === 'strong' ? 'success' : s.band === 'developing' ? 'gold' : 'danger'" size="xs">
            {{ s.band }}
          </AppBadge>
          <AppBadge v-if="s.riskLevel !== 'none'" :color="s.riskLevel === 'high' ? 'danger' : 'warm'" size="xs">
            {{ s.riskLevel }} risk
          </AppBadge>
        </div>
        <AppProgress :value="s.readiness" :max="10000" size="sm"
          :color="s.band === 'strong' ? 'success' : s.band === 'developing' ? 'gold' : 'danger'" />
        <p v-if="s.topWeakness" class="text-[10px] mt-1.5" :style="{ color: 'var(--text-3)' }">
          Top weakness: {{ s.topWeakness }}
        </p>
      </AppCard>
    </div>
  </div>
</template>
