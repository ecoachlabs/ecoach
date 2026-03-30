<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  value: number    // 0-100 or BasisPoints
  max?: number     // default 100
  size?: number    // px, default 64
  strokeWidth?: number
  color?: string
  label?: string
}>()

const sz = computed(() => props.size ?? 64)
const sw = computed(() => props.strokeWidth ?? 4)
const radius = computed(() => (sz.value - sw.value) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)
const max = computed(() => props.max ?? 100)
const pct = computed(() => Math.min(100, Math.max(0, (props.value / max.value) * 100)))
const offset = computed(() => circumference.value - (pct.value / 100) * circumference.value)
const fill = computed(() => props.color ?? 'var(--accent)')
</script>

<template>
  <div class="relative inline-flex items-center justify-center" :style="{ width: sz + 'px', height: sz + 'px' }">
    <svg :width="sz" :height="sz" class="-rotate-90">
      <!-- Background ring -->
      <circle
        :cx="sz / 2" :cy="sz / 2" :r="radius"
        fill="none" :stroke-width="sw"
        :stroke="'var(--border-soft)'"
      />
      <!-- Progress arc -->
      <circle
        :cx="sz / 2" :cy="sz / 2" :r="radius"
        fill="none" :stroke-width="sw"
        :stroke="fill"
        stroke-linecap="round"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="offset"
        class="transition-all"
        :style="{ transitionDuration: 'var(--dur-slow)', transitionTimingFunction: 'var(--ease-out)' }"
      />
    </svg>
    <!-- Center label -->
    <div class="absolute inset-0 flex flex-col items-center justify-center">
      <span class="font-semibold text-xs tabular-nums" :style="{ color: 'var(--text)', fontSize: (sz * 0.22) + 'px' }">
        {{ Math.round(pct) }}%
      </span>
      <span v-if="label" class="text-[8px] font-medium" :style="{ color: 'var(--text-3)', fontSize: (sz * 0.12) + 'px' }">
        {{ label }}
      </span>
    </div>
  </div>
</template>
