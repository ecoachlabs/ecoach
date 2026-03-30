<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  items: { id: number; text: string }[]
  answered: boolean
  correctOrder?: number[]
}>()

const emit = defineEmits<{ submit: [order: number[]] }>()

const orderedItems = ref([...props.items])
const draggingIdx = ref<number | null>(null)

function onDragStart(idx: number) {
  draggingIdx.value = idx
}

function onDragOver(e: DragEvent, idx: number) {
  e.preventDefault()
  if (draggingIdx.value === null || draggingIdx.value === idx) return
  const items = [...orderedItems.value]
  const [moved] = items.splice(draggingIdx.value, 1)
  items.splice(idx, 0, moved)
  orderedItems.value = items
  draggingIdx.value = idx
}

function onDragEnd() {
  draggingIdx.value = null
}

const currentOrder = computed(() => orderedItems.value.map(item => item.id))

function isCorrectPosition(idx: number): boolean | null {
  if (!props.answered || !props.correctOrder) return null
  return orderedItems.value[idx].id === props.correctOrder[idx]
}
</script>

<template>
  <div class="space-y-2">
    <p class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
      Drag to arrange in the correct order
    </p>
    <div
      v-for="(item, idx) in orderedItems"
      :key="item.id"
      :draggable="!answered"
      class="flex items-center gap-3 px-4 py-3 rounded-[var(--radius-md)] border transition-all cursor-grab active:cursor-grabbing"
      :class="[
        answered && isCorrectPosition(idx) === true ? 'border-emerald-500 bg-emerald-50' :
        answered && isCorrectPosition(idx) === false ? 'border-red-400 bg-red-50' :
        draggingIdx === idx ? 'border-[var(--accent)] bg-[var(--accent-light)] scale-[1.02] shadow-md' :
        'border-[var(--card-border)] hover:border-[var(--accent)]',
      ]"
      :style="{ backgroundColor: (!answered && draggingIdx !== idx) ? 'var(--card-bg)' : undefined }"
      @dragstart="onDragStart(idx)"
      @dragover="onDragOver($event, idx)"
      @dragend="onDragEnd"
    >
      <span class="w-6 h-6 rounded flex items-center justify-center text-[10px] font-bold"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }">
        {{ idx + 1 }}
      </span>
      <span class="text-sm" :style="{ color: 'var(--text)' }">{{ item.text }}</span>
      <span v-if="!answered" class="ml-auto text-xs" :style="{ color: 'var(--text-3)' }">⋮⋮</span>
    </div>
  </div>
  <div v-if="!answered" class="mt-4 text-right">
    <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
      :style="{ backgroundColor: 'var(--accent)' }"
      @click="emit('submit', currentOrder)">Submit Order</button>
  </div>
</template>
