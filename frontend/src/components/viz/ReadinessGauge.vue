<script setup lang="ts">
import { computed } from 'vue'
import ProgressRing from './ProgressRing.vue'

const props = defineProps<{
  score: number       // BasisPoints (0-10000)
  band: string        // 'strong' | 'developing' | 'weak' | 'critical'
  label?: string
  size?: number
  showDays?: number   // days until exam
}>()

const color = computed(() => {
  switch (props.band) {
    case 'strong': return 'var(--success)'
    case 'developing': return 'var(--gold)'
    case 'weak': return 'var(--warning)'
    case 'critical': return 'var(--danger)'
    default: return 'var(--text-3)'
  }
})

const bandLabel = computed(() => {
  switch (props.band) {
    case 'strong': return 'Strong'
    case 'developing': return 'Developing'
    case 'weak': return 'Needs Work'
    case 'critical': return 'Critical'
    default: return props.band
  }
})
</script>

<template>
  <div class="flex flex-col items-center gap-2">
    <ProgressRing
      :value="score"
      :max="10000"
      :size="size || 80"
      :stroke-width="5"
      :color="color"
      :label="label || 'Readiness'"
    />
    <div class="text-center">
      <span
        class="text-xs font-semibold px-2 py-0.5 rounded-full"
        :style="{ color, backgroundColor: color + '15' }"
      >
        {{ bandLabel }}
      </span>
      <p v-if="showDays" class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">
        {{ showDays }} days to exam
      </p>
    </div>
  </div>
</template>
