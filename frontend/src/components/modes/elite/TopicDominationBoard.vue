<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  topics: { name: string; status: string; dominationScore: number; accuracy: number; speed: number; trapResistance: number }[]
}>()

defineEmits<{ selectTopic: [name: string] }>()
</script>

<template>
  <div class="space-y-2">
    <AppCard v-for="t in topics" :key="t.name" hover padding="sm" @click="$emit('selectTopic', t.name)">
      <div class="flex items-center gap-3 mb-2">
        <p class="text-sm font-semibold flex-1 truncate" :style="{ color: 'var(--text)' }">{{ t.name }}</p>
        <AppBadge :color="t.dominationScore >= 8000 ? 'success' : t.dominationScore >= 5000 ? 'gold' : 'warm'" size="xs">
          {{ t.status }}
        </AppBadge>
      </div>
      <AppProgress :value="t.dominationScore" :max="10000" size="sm"
        :color="t.dominationScore >= 8000 ? 'success' : t.dominationScore >= 5000 ? 'gold' : 'warm'" class="mb-2" />
      <div class="flex gap-4 text-[9px]" :style="{ color: 'var(--text-3)' }">
        <span>Acc: {{ (t.accuracy / 100).toFixed(0) }}%</span>
        <span>Speed: {{ (t.speed / 100).toFixed(0) }}%</span>
        <span>Traps: {{ (t.trapResistance / 100).toFixed(0) }}%</span>
      </div>
    </AppCard>
  </div>
</template>
