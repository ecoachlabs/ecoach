<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  leftWidth?: string
}>()

const dividerDragging = ref(false)
const leftPaneWidth = ref(50) // percentage

function startDrag() {
  dividerDragging.value = true
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  const container = document.getElementById('split-pane-container')
  if (!container) return
  const rect = container.getBoundingClientRect()
  leftPaneWidth.value = Math.max(20, Math.min(80, ((e.clientX - rect.left) / rect.width) * 100))
}

function stopDrag() {
  dividerDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}
</script>

<template>
  <div id="split-pane-container" class="flex h-full overflow-hidden rounded-[var(--radius-lg)] border"
    :style="{ borderColor: 'var(--card-border)' }">
    <!-- Left pane -->
    <div :style="{ width: leftPaneWidth + '%' }" class="overflow-auto" :class="{ 'select-none': dividerDragging }">
      <slot name="left" />
    </div>

    <!-- Divider -->
    <div
      class="w-1 shrink-0 cursor-col-resize hover:bg-[var(--accent)] transition-colors"
      :class="dividerDragging ? 'bg-[var(--accent)]' : 'bg-[var(--card-border)]'"
      @mousedown="startDrag"
    />

    <!-- Right pane -->
    <div class="flex-1 overflow-auto" :class="{ 'select-none': dividerDragging }">
      <slot name="right" />
    </div>
  </div>
</template>
