<script setup lang="ts">
import { ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'

interface TreeNode {
  id: number
  name: string
  type: string
  children?: TreeNode[]
}

defineProps<{
  node: TreeNode
  depth?: number
}>()

defineEmits<{ select: [id: number] }>()
const expanded = ref(false)
</script>

<template>
  <div :style="{ paddingLeft: (depth || 0) * 16 + 'px' }">
    <button
      class="w-full flex items-center gap-2 px-3 py-2 rounded-[var(--radius-sm)] text-left text-sm transition-colors hover:bg-[var(--primary-light)]"
      @click="node.children?.length ? (expanded = !expanded) : $emit('select', node.id)"
    >
      <span v-if="node.children?.length" class="text-[10px] w-4 text-center" :style="{ color: 'var(--text-3)' }">
        {{ expanded ? '▾' : '▸' }}
      </span>
      <span v-else class="w-4 text-center text-[10px]" :style="{ color: 'var(--text-3)' }">·</span>
      <span class="flex-1 truncate" :style="{ color: 'var(--text)' }">{{ node.name }}</span>
      <AppBadge color="muted" size="xs">{{ node.type }}</AppBadge>
    </button>
    <div v-if="expanded && node.children?.length">
      <CurriculumTreeNode v-for="child in node.children" :key="child.id" :node="child" :depth="(depth || 0) + 1" @select="$emit('select', $event)" />
    </div>
  </div>
</template>
