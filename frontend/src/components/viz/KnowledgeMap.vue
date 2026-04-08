<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import * as d3 from 'd3'
import { getMasteryDisplay } from '@/utils/mastery'

export interface MapNode {
  id: number
  name: string
  masteryState: string
  score: number
  type: 'subject' | 'topic' | 'subtopic' | 'skill'
  group?: number
}

export interface MapLink {
  source: number
  target: number
  type: 'prerequisite' | 'related' | 'confused_with'
}

const props = defineProps<{
  nodes: MapNode[]
  links: MapLink[]
  width?: number
  height?: number
  selectedNodeId?: number | null
}>()

const emit = defineEmits<{ selectNode: [nodeId: number] }>()

const container = ref<HTMLElement | null>(null)
let simulation: d3.Simulation<any, any> | null = null

function nodeRadius(type: string): number {
  switch (type) {
    case 'subject': return 24
    case 'topic': return 16
    case 'subtopic': return 12
    default: return 8
  }
}

function linkColor(type: string): string {
  switch (type) {
    case 'prerequisite': return 'var(--accent)'
    case 'confused_with': return 'var(--danger)'
    default: return 'var(--border-soft)'
  }
}

function buildGraph() {
  if (!container.value || !props.nodes.length) return

  const w = props.width ?? container.value.clientWidth ?? 600
  const h = props.height ?? 400

  // Clear previous
  d3.select(container.value).selectAll('*').remove()

  const svg = d3.select(container.value)
    .append('svg')
    .attr('width', w)
    .attr('height', h)
    .attr('viewBox', `0 0 ${w} ${h}`)

  // Map nodes/links for D3
  const nodeData = props.nodes.map(n => ({ ...n }))
  const linkData = props.links.map(l => ({
    source: nodeData.findIndex(n => n.id === l.source),
    target: nodeData.findIndex(n => n.id === l.target),
    type: l.type,
  })).filter(l => l.source >= 0 && l.target >= 0)

  // Simulation
  simulation = d3.forceSimulation(nodeData as any)
    .force('link', d3.forceLink(linkData).distance(80).strength(0.3))
    .force('charge', d3.forceManyBody().strength(-200))
    .force('center', d3.forceCenter(w / 2, h / 2))
    .force('collision', d3.forceCollide().radius((d: any) => nodeRadius(d.type) + 5))

  // Links
  const link = svg.append('g')
    .selectAll('line')
    .data(linkData)
    .join('line')
    .attr('stroke', d => linkColor(d.type))
    .attr('stroke-width', 1.5)
    .attr('stroke-opacity', 0.4)
    .attr('stroke-dasharray', d => d.type === 'confused_with' ? '4,4' : 'none')

  // Nodes
  const node = svg.append('g')
    .selectAll('g')
    .data(nodeData)
    .join('g')
    .style('cursor', 'pointer')
    .on('click', (_, d: any) => emit('selectNode', d.id))

  // Node circles
  node.append('circle')
    .attr('r', (d: any) => nodeRadius(d.type))
    .attr('fill', (d: any) => getMasteryDisplay(d.masteryState).bg)
    .attr('stroke', (d: any) => getMasteryDisplay(d.masteryState).color)
    .attr('stroke-width', (d: any) => d.id === props.selectedNodeId ? 3 : 1.5)

  // Glow for high-mastery nodes
  node.filter((d: any) => d.score > 6000)
    .append('circle')
    .attr('r', (d: any) => nodeRadius(d.type) + 4)
    .attr('fill', 'none')
    .attr('stroke', (d: any) => getMasteryDisplay(d.masteryState).color)
    .attr('stroke-width', 1)
    .attr('stroke-opacity', 0.2)

  // Labels
  node.append('text')
    .text((d: any) => d.name.length > 12 ? d.name.slice(0, 12) + '...' : d.name)
    .attr('text-anchor', 'middle')
    .attr('dy', (d: any) => nodeRadius(d.type) + 12)
    .attr('fill', 'var(--text-3)')
    .attr('font-size', '9px')
    .attr('font-family', 'var(--font-body)')

  // Drag behavior
  const drag = d3.drag<SVGGElement, any>()
    .on('start', (event, d) => {
      if (!event.active) simulation?.alphaTarget(0.3).restart()
      d.fx = d.x
      d.fy = d.y
    })
    .on('drag', (event, d) => {
      d.fx = event.x
      d.fy = event.y
    })
    .on('end', (event, d) => {
      if (!event.active) simulation?.alphaTarget(0)
      d.fx = null
      d.fy = null
    })

  node.call(drag as any)

  // Tick
  simulation.on('tick', () => {
    link
      .attr('x1', (d: any) => d.source.x)
      .attr('y1', (d: any) => d.source.y)
      .attr('x2', (d: any) => d.target.x)
      .attr('y2', (d: any) => d.target.y)

    node.attr('transform', (d: any) => `translate(${d.x},${d.y})`)
  })
}

onMounted(buildGraph)
watch(() => [props.nodes, props.links], buildGraph, { deep: true })
onUnmounted(() => { simulation?.stop() })
</script>

<template>
  <div ref="container" class="w-full rounded-[var(--radius-lg)] overflow-hidden"
    :style="{ backgroundColor: 'var(--card-bg)', minHeight: (height ?? 400) + 'px' }">
    <p v-if="!nodes.length" class="text-sm text-center py-16" :style="{ color: 'var(--text-3)' }">
      No knowledge map data available. Complete more practice sessions to build your map.
    </p>
  </div>
</template>
