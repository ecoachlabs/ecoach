<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import * as d3 from 'd3'

export interface ConstellationNode {
  id: number
  label: string
  familyId: number
  frequency: number
  color?: string
}

export interface ConstellationLink {
  source: number
  target: number
  strength: number
}

const props = defineProps<{
  nodes: ConstellationNode[]
  links: ConstellationLink[]
  width?: number
  height?: number
  selectedFamilyId?: number | null
}>()

const emit = defineEmits<{ selectNode: [nodeId: number] }>()
const container = ref<HTMLElement | null>(null)
let simulation: d3.Simulation<any, any> | null = null

const familyColors = ['#0d9488', '#b45309', '#7c3aed', '#dc2626', '#2563eb', '#059669', '#d97706', '#db2777']

function buildConstellation() {
  if (!container.value || !props.nodes.length) return

  const w = props.width ?? container.value.clientWidth ?? 700
  const h = props.height ?? 500

  d3.select(container.value).selectAll('*').remove()

  const svg = d3.select(container.value)
    .append('svg')
    .attr('width', w)
    .attr('height', h)

  const nodeData = props.nodes.map(n => ({
    ...n,
    color: n.color ?? familyColors[n.familyId % familyColors.length],
  }))

  const linkData = props.links.map(l => ({
    source: nodeData.findIndex(n => n.id === l.source),
    target: nodeData.findIndex(n => n.id === l.target),
    strength: l.strength,
  })).filter(l => l.source >= 0 && l.target >= 0)

  simulation = d3.forceSimulation(nodeData as any)
    .force('link', d3.forceLink(linkData).distance(60).strength(0.2))
    .force('charge', d3.forceManyBody().strength(-100))
    .force('center', d3.forceCenter(w / 2, h / 2))
    .force('collision', d3.forceCollide().radius((d: any) => Math.sqrt(d.frequency) * 2 + 8))

  // Links (faint connections)
  const link = svg.append('g')
    .selectAll('line')
    .data(linkData)
    .join('line')
    .attr('stroke', 'var(--border-soft)')
    .attr('stroke-width', (d: any) => Math.max(0.5, d.strength / 30))
    .attr('stroke-opacity', 0.3)

  // Stars (nodes)
  const node = svg.append('g')
    .selectAll('circle')
    .data(nodeData)
    .join('circle')
    .attr('r', (d: any) => Math.sqrt(d.frequency) * 1.5 + 4)
    .attr('fill', (d: any) => d.color)
    .attr('fill-opacity', 0.8)
    .attr('stroke', (d: any) => d.familyId === props.selectedFamilyId ? 'white' : 'none')
    .attr('stroke-width', 2)
    .style('cursor', 'pointer')
    .on('click', (_, d: any) => emit('selectNode', d.id))

  // Glow effect
  node.append('title').text((d: any) => d.label)

  // Labels for larger nodes
  svg.append('g')
    .selectAll('text')
    .data(nodeData.filter((d: any) => d.frequency > 5))
    .join('text')
    .text((d: any) => d.label.length > 15 ? d.label.slice(0, 15) + '...' : d.label)
    .attr('text-anchor', 'middle')
    .attr('fill', 'var(--text-3)')
    .attr('font-size', '8px')
    .attr('font-family', 'var(--font-body)')
    .attr('pointer-events', 'none')

  simulation.on('tick', () => {
    link
      .attr('x1', (d: any) => d.source.x)
      .attr('y1', (d: any) => d.source.y)
      .attr('x2', (d: any) => d.target.x)
      .attr('y2', (d: any) => d.target.y)

    node
      .attr('cx', (d: any) => d.x)
      .attr('cy', (d: any) => d.y)

    svg.selectAll('text')
      .attr('x', (d: any) => d.x)
      .attr('y', (d: any) => d.y - Math.sqrt(d.frequency) * 1.5 - 8)
  })
}

onMounted(buildConstellation)
watch(() => [props.nodes, props.links], buildConstellation, { deep: true })
onUnmounted(() => { simulation?.stop() })
</script>

<template>
  <div ref="container" class="w-full rounded-[var(--radius-lg)] overflow-hidden"
    :style="{ backgroundColor: '#0c0a09', minHeight: (height ?? 500) + 'px' }">
    <p v-if="!nodes.length" class="text-sm text-center py-16 text-stone-500">
      No constellation data. Past exam patterns will appear here.
    </p>
  </div>
</template>
