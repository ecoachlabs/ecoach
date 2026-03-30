<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

defineProps<{
  studentName: string
  readinessBand: string
  readinessScore: number
  examName?: string
  daysToExam?: number
  strengths: string[]
  concerns: string[]
  recommendation: string
}>()
</script>

<template>
  <AppCard padding="lg">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h2 class="font-display text-lg font-semibold" :style="{ color: 'var(--text)' }">Readiness Brief</h2>
        <p class="text-xs" :style="{ color: 'var(--text-3)' }">{{ studentName }} · {{ examName }}</p>
      </div>
      <ProgressRing :value="readinessScore" :max="10000" :size="64" :stroke-width="4"
        :color="readinessBand === 'strong' ? 'var(--success)' : readinessBand === 'developing' ? 'var(--gold)' : 'var(--danger)'"
        :label="daysToExam ? daysToExam + ' days' : 'Readiness'" />
    </div>

    <div class="grid grid-cols-2 gap-4 mb-4">
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--success)' }">Strengths</p>
        <ul class="space-y-1">
          <li v-for="s in strengths" :key="s" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
            <span class="text-emerald-500 mt-0.5">✓</span> {{ s }}
          </li>
        </ul>
      </div>
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--warning)' }">Concerns</p>
        <ul class="space-y-1">
          <li v-for="c in concerns" :key="c" class="text-xs flex items-start gap-1.5" :style="{ color: 'var(--text-2)' }">
            <span class="text-amber-500 mt-0.5">⚠</span> {{ c }}
          </li>
        </ul>
      </div>
    </div>

    <div class="pt-3 border-t" :style="{ borderColor: 'var(--card-border)' }">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--accent)' }">Recommendation</p>
      <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ recommendation }}</p>
    </div>
  </AppCard>
</template>
