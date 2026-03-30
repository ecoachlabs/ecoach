<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  data: { label: string; value: number }[]
  maxValue?: number
  height?: number
  color?: string
  showDots?: boolean
  showLabels?: boolean
  showArea?: boolean
}>()

const h = computed(() => props.height ?? 120)
const w = 400  // viewbox width, scales with container
const max = computed(() => props.maxValue ?? Math.max(...props.data.map(d => d.value), 1))
const padding = { top: 10, right: 20, bottom: 25, left: 10 }
const chartW = w - padding.left - padding.right
const chartH = computed(() => h.value - padding.top - padding.bottom)
const fill = computed(() => props.color ?? 'var(--accent)')

const points = computed(() =>
  props.data.map((d, i) => ({
    x: padding.left + (i / Math.max(1, props.data.length - 1)) * chartW,
    y: padding.top + chartH.value - (d.value / max.value) * chartH.value,
    label: d.label,
    value: d.value,
  }))
)

const linePath = computed(() =>
  points.value.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ')
)

const areaPath = computed(() =>
  linePath.value + ` L ${points.value[points.value.length - 1]?.x ?? 0} ${padding.top + chartH.value} L ${padding.left} ${padding.top + chartH.value} Z`
)
</script>

<template>
  <svg :viewBox="`0 0 ${w} ${h}`" class="w-full" preserveAspectRatio="xMidYMid meet">
    <!-- Grid lines -->
    <line v-for="i in 4" :key="'g' + i"
      :x1="padding.left" :x2="w - padding.right"
      :y1="padding.top + (chartH * (i - 1)) / 3" :y2="padding.top + (chartH * (i - 1)) / 3"
      :stroke="'var(--border-soft)'" stroke-width="0.5" stroke-dasharray="4,4" />

    <!-- Area fill -->
    <path v-if="showArea !== false && points.length > 1" :d="areaPath" :fill="fill" fill-opacity="0.08" />

    <!-- Line -->
    <path v-if="points.length > 1" :d="linePath" :stroke="fill" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" />

    <!-- Dots -->
    <circle v-if="showDots !== false" v-for="(p, i) in points" :key="'d' + i"
      :cx="p.x" :cy="p.y" r="3" :fill="fill" stroke="white" stroke-width="1.5" />

    <!-- Labels -->
    <text v-if="showLabels !== false" v-for="(p, i) in points" :key="'l' + i"
      :x="p.x" :y="h - 4" text-anchor="middle"
      class="text-[8px]" :fill="'var(--text-3)'">
      {{ p.label }}
    </text>
  </svg>
</template>
