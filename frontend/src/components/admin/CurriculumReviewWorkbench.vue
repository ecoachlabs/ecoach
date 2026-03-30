<script setup lang="ts">
import SplitPane from './SplitPane.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  sourceTitle: string
  sourcePages: { pageNumber: number; text: string }[]
  extractedNodes: { id: number; type: string; label: string; confidence: number; status: 'pending' | 'approved' | 'rejected' }[]
}>()

defineEmits<{ approve: [nodeId: number]; reject: [nodeId: number]; edit: [nodeId: number] }>()
</script>

<template>
  <div class="h-[calc(100vh-120px)]">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-sm font-bold" :style="{color:'var(--text)'}">Review: {{ sourceTitle }}</h2>
      <div class="flex gap-2">
        <AppBadge color="accent" size="sm">{{ extractedNodes.filter(n => n.status === 'pending').length }} pending</AppBadge>
      </div>
    </div>

    <SplitPane>
      <template #left>
        <div class="p-4">
          <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Original Source</h3>
          <div class="space-y-4">
            <div v-for="page in sourcePages" :key="page.pageNumber" class="pb-4 border-b" :style="{borderColor:'var(--card-border)'}">
              <p class="text-[9px] font-semibold mb-1" :style="{color:'var(--text-3)'}">Page {{ page.pageNumber }}</p>
              <p class="text-xs leading-relaxed whitespace-pre-wrap" :style="{color:'var(--text-2)'}">{{ page.text }}</p>
            </div>
          </div>
        </div>
      </template>

      <template #right>
        <div class="p-4">
          <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Extracted Structure</h3>
          <div class="space-y-1.5">
            <div v-for="node in extractedNodes" :key="node.id"
              class="flex items-center gap-2 px-3 py-2 rounded-[var(--radius-sm)] border"
              :class="node.confidence < 70 ? 'border-amber-300 bg-amber-50' : 'border-[var(--card-border)]'"
              :style="{backgroundColor: node.confidence >= 70 ? 'var(--card-bg)' : undefined}">
              <AppBadge :color="node.type === 'subject' ? 'accent' : node.type === 'topic' ? 'gold' : 'muted'" size="xs">{{ node.type }}</AppBadge>
              <span class="text-xs flex-1 truncate" :style="{color:'var(--text)'}">{{ node.label }}</span>
              <span class="text-[9px] tabular-nums" :style="{color: node.confidence < 70 ? 'var(--danger)' : 'var(--success)'}">
                {{ node.confidence }}%
              </span>
              <div v-if="node.status === 'pending'" class="flex gap-0.5">
                <button class="w-5 h-5 rounded text-[10px] flex items-center justify-center hover:bg-emerald-100 text-emerald-600" @click="$emit('approve', node.id)">✓</button>
                <button class="w-5 h-5 rounded text-[10px] flex items-center justify-center hover:bg-red-100 text-red-500" @click="$emit('reject', node.id)">✕</button>
                <button class="w-5 h-5 rounded text-[10px] flex items-center justify-center hover:bg-blue-100 text-blue-500" @click="$emit('edit', node.id)">✎</button>
              </div>
              <AppBadge v-else :color="node.status === 'approved' ? 'success' : 'danger'" size="xs">{{ node.status }}</AppBadge>
            </div>
          </div>
        </div>
      </template>
    </SplitPane>
  </div>
</template>
