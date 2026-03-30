<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  text: string
  type: 'feature' | 'example' | 'condition' | 'application' | 'trap' | string
  active: boolean
}>()

defineEmits<{ drop: [side: 'left' | 'right'] }>()

const dragging = ref(false)
</script>

<template>
  <div
    class="px-4 py-3 rounded-[var(--radius-lg)] border text-sm font-medium transition-all cursor-grab active:cursor-grabbing select-none"
    :class="[
      active ? 'shadow-md scale-[1.02]' : '',
      dragging ? 'opacity-70 rotate-1' : '',
      type === 'trap' ? 'border-amber-300 bg-amber-50' : 'border-[var(--card-border)]',
    ]"
    :style="{ backgroundColor: type !== 'trap' ? 'var(--card-bg)' : undefined, color: 'var(--text)' }"
    :draggable="active"
    @dragstart="dragging = true"
    @dragend="dragging = false"
  >
    <div class="flex items-center gap-2">
      <span class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' }">
        {{ type }}
      </span>
      <span class="flex-1">{{ text }}</span>
    </div>
  </div>
</template>
