<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import DecaySeverityBadge from '@/components/viz/DecaySeverityBadge.vue'

defineProps<{
  results: { topicName: string; status: string; strength: number }[]
  stableCount: number
  vulnerableCount: number
  criticalCount: number
}>()

defineEmits<{ rescue: []; home: [] }>()
</script>

<template>
  <div class="reveal-stagger">
    <div class="text-center mb-6">
      <h2 class="font-display text-lg font-semibold" :style="{color:'var(--text)'}">Scan Complete</h2>
      <p class="text-sm" :style="{color:'var(--text-3)'}">Here is your memory health.</p>
    </div>

    <div class="grid grid-cols-3 gap-3 mb-6">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{color:'var(--success)'}">{{ stableCount }}</p>
        <p class="text-[10px] uppercase" :style="{color:'var(--text-3)'}">Stable</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{color:'var(--gold)'}">{{ vulnerableCount }}</p>
        <p class="text-[10px] uppercase" :style="{color:'var(--text-3)'}">Vulnerable</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{color:'var(--danger)'}">{{ criticalCount }}</p>
        <p class="text-[10px] uppercase" :style="{color:'var(--text-3)'}">Critical</p>
      </AppCard>
    </div>

    <div class="space-y-1.5 mb-6">
      <AppCard v-for="r in results" :key="r.topicName" padding="sm">
        <div class="flex items-center gap-3">
          <DecaySeverityBadge :severity="r.status" size="sm" />
          <p class="text-xs font-medium flex-1 truncate" :style="{color:'var(--text)'}">{{ r.topicName }}</p>
          <span class="text-[10px] tabular-nums" :style="{color:'var(--text-3)'}">{{ (r.strength / 100).toFixed(0) }}%</span>
        </div>
      </AppCard>
    </div>

    <div class="flex gap-3 justify-center">
      <AppButton v-if="criticalCount > 0" variant="primary" @click="$emit('rescue')">Rescue {{ criticalCount }} Critical →</AppButton>
      <AppButton variant="ghost" @click="$emit('home')">Back</AppButton>
    </div>
  </div>
</template>
