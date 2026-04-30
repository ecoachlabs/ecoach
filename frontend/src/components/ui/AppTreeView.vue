<script setup lang="ts">
import { defineComponent, h, ref, type Component } from 'vue'

export interface TreeItem {
  id: string | number
  label: string
  icon?: string
  badge?: string
  children?: TreeItem[]
}

defineProps<{ items: TreeItem[]; depth?: number }>()
defineEmits<{ select: [item: TreeItem] }>()

const TreeNode: Component = defineComponent({
  name: 'TreeNode',
  props: { item: { type: Object as () => TreeItem, required: true }, depth: { type: Number, default: 0 } },
  emits: ['select'],
  setup(props, { emit }) {
    const expanded = ref(false)
    return () => {
      const hasChildren = props.item.children && props.item.children.length > 0
      return h('div', [
        h('button', {
          class: 'w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm transition-colors hover:bg-[var(--primary-light)] text-left',
          style: { color: 'var(--text)' },
          onClick: () => { if (hasChildren) expanded.value = !expanded.value; else emit('select', props.item) }
        }, [
          hasChildren ? h('span', { class: 'text-[10px] w-4', style: { color: 'var(--text-3)' } }, expanded.value ? '▾' : '▸') : h('span', { class: 'w-4' }),
          props.item.icon ? h('span', { class: 'text-sm' }, props.item.icon) : null,
          h('span', { class: 'flex-1 truncate' }, props.item.label),
          props.item.badge ? h('span', { class: 'text-[10px] px-1.5 py-0.5 rounded-full font-medium', style: { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } }, props.item.badge) : null,
        ]),
        expanded.value && hasChildren ? h('div', { style: { paddingLeft: '16px' } },
          props.item.children!.map(child => h(TreeNode, { key: child.id, item: child, depth: props.depth + 1, onSelect: (i: TreeItem) => emit('select', i) }))
        ) : null,
      ])
    }
  },
})
</script>

<template>
  <div :style="{ paddingLeft: (depth || 0) > 0 ? '16px' : '0' }">
    <div v-for="item in items" :key="item.id">
      <TreeNode :item="item" :depth="depth || 0" @select="$emit('select', $event)" />
    </div>
  </div>
</template>
