<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  gapPercentage: number  // 0-100
  size?: number
}>()

const sz = computed(() => props.size ?? 80)
const sw = 6
const radius = computed(() => (sz.value - sw) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)
const knownPct = computed(() => 100 - props.gapPercentage)
const offset = computed(() => circumference.value - (knownPct.value / 100) * circumference.value)
</script>

<template>
  <div class="relative inline-flex items-center justify-center" :style="{ width: sz + 'px', height: sz + 'px' }">
    <svg :width="sz" :height="sz" class="-rotate-90">
      <!-- Full circle (gap = red) -->
      <circle :cx="sz / 2" :cy="sz / 2" :r="radius" fill="none" :stroke-width="sw" stroke="var(--danger)" opacity="0.15" />
      <!-- Known portion (green arc) -->
      <circle
        :cx="sz / 2" :cy="sz / 2" :r="radius"
        fill="none" :stroke-width="sw"
        stroke="var(--accent)"
        stroke-linecap="round"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="offset"
        class="transition-all"
        :style="{ transitionDuration: 'var(--dur-slow)' }"
      />
    </svg>
    <div class="absolute inset-0 flex flex-col items-center justify-center">
      <span class="font-display font-bold tabular-nums" :style="{ fontSize: (sz * 0.22) + 'px', color: 'var(--danger)' }">
        {{ gapPercentage }}%
      </span>
      <span class="text-[8px] font-semibold uppercase" :style="{ color: 'var(--text-3)' }">Gap</span>
    </div>
  </div>
</template>
