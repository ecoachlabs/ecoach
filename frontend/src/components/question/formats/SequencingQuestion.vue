<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  prompt: string
  items: { id: number; text: string }[]
  answered: boolean
  correctOrder?: number[]
}>()

const emit = defineEmits<{ submit: [order: number[]] }>()
const orderedItems = ref([...props.items].sort(() => Math.random() - 0.5))

function moveUp(idx: number) {
  if (idx === 0 || props.answered) return
  const items = [...orderedItems.value]
  ;[items[idx - 1], items[idx]] = [items[idx], items[idx - 1]]
  orderedItems.value = items
}

function moveDown(idx: number) {
  if (idx >= orderedItems.value.length - 1 || props.answered) return
  const items = [...orderedItems.value]
  ;[items[idx], items[idx + 1]] = [items[idx + 1], items[idx]]
  orderedItems.value = items
}

function isCorrect(idx: number): boolean | null {
  if (!props.answered || !props.correctOrder) return null
  return orderedItems.value[idx].id === props.correctOrder[idx]
}
</script>

<template>
  <div>
    <p class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">{{ prompt || 'Arrange in the correct order' }}</p>
    <div class="space-y-1.5">
      <div v-for="(item, idx) in orderedItems" :key="item.id"
        class="flex items-center gap-2 px-3 py-2.5 rounded-[var(--radius-md)] border transition-all"
        :class="answered && isCorrect(idx) === true ? 'border-emerald-500 bg-emerald-50' : answered && isCorrect(idx) === false ? 'border-red-400 bg-red-50' : 'border-[var(--card-border)]'"
        :style="{ backgroundColor: !answered ? 'var(--card-bg)' : undefined }">
        <span class="w-6 h-6 rounded flex items-center justify-center text-[10px] font-bold" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }">{{ idx + 1 }}</span>
        <span class="flex-1 text-sm" :style="{ color: 'var(--text)' }">{{ item.text }}</span>
        <div v-if="!answered" class="flex flex-col gap-0.5">
          <button class="text-[10px] px-1 rounded hover:bg-[var(--primary-light)]" :style="{ color: 'var(--text-3)' }" @click="moveUp(idx)">▲</button>
          <button class="text-[10px] px-1 rounded hover:bg-[var(--primary-light)]" :style="{ color: 'var(--text-3)' }" @click="moveDown(idx)">▼</button>
        </div>
      </div>
    </div>
    <div v-if="!answered" class="mt-3 text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white" :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', orderedItems.map(i => i.id))">Submit Order</button>
    </div>
  </div>
</template>
