<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  dimensions: { label: string; value: number; max?: number }[]
  size?: number
  color?: string
  showLabels?: boolean
}>()

const sz = computed(() => props.size ?? 200)
const cx = computed(() => sz.value / 2)
const cy = computed(() => sz.value / 2)
const radius = computed(() => sz.value * 0.38)
const fill = computed(() => props.color ?? 'var(--accent)')
const count = computed(() => props.dimensions.length)

function angleFor(i: number): number {
  return (Math.PI * 2 * i) / count.value - Math.PI / 2
}

function pointAt(i: number, pct: number): { x: number; y: number } {
  const angle = angleFor(i)
  return {
    x: cx.value + Math.cos(angle) * radius.value * pct,
    y: cy.value + Math.sin(angle) * radius.value * pct,
  }
}

const gridLevels = [0.25, 0.5, 0.75, 1.0]

const gridPaths = computed(() =>
  gridLevels.map(level => {
    const points = props.dimensions.map((_, i) => pointAt(i, level))
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ') + ' Z'
  })
)

const dataPath = computed(() => {
  const points = props.dimensions.map((dim, i) => {
    const pct = Math.min(1, dim.value / (dim.max ?? 10000))
    return pointAt(i, pct)
  })
  return points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ') + ' Z'
})

const dataPoints = computed(() =>
  props.dimensions.map((dim, i) => {
    const pct = Math.min(1, dim.value / (dim.max ?? 10000))
    return { ...pointAt(i, pct), label: dim.label, value: dim.value }
  })
)

const labelPositions = computed(() =>
  props.dimensions.map((dim, i) => {
    const p = pointAt(i, 1.18)
    return { x: p.x, y: p.y, label: dim.label }
  })
)
</script>

<template>
  <svg :width="sz" :height="sz" class="overflow-visible">
    <!-- Grid -->
    <path v-for="(path, i) in gridPaths" :key="i" :d="path"
      fill="none" :stroke="'var(--border-soft)'" :stroke-width="i === gridPaths.length - 1 ? 1 : 0.5" />

    <!-- Axes -->
    <line v-for="(_, i) in dimensions" :key="'axis-' + i"
      :x1="cx" :y1="cy" :x2="pointAt(i, 1).x" :y2="pointAt(i, 1).y"
      :stroke="'var(--border-soft)'" stroke-width="0.5" />

    <!-- Data fill -->
    <path :d="dataPath" :fill="fill" fill-opacity="0.15" :stroke="fill" stroke-width="2" stroke-linejoin="round" />

    <!-- Data points -->
    <circle v-for="(p, i) in dataPoints" :key="'pt-' + i"
      :cx="p.x" :cy="p.y" r="3.5" :fill="fill" stroke="white" stroke-width="1.5" />

    <!-- Labels -->
    <text v-if="showLabels !== false" v-for="(l, i) in labelPositions" :key="'lbl-' + i"
      :x="l.x" :y="l.y" text-anchor="middle" dominant-baseline="middle"
      class="text-[9px] font-medium" :fill="'var(--text-3)'">
      {{ l.label }}
    </text>
  </svg>
</template>
