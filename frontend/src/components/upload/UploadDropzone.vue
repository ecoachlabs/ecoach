<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{ files: [files: File[]] }>()

const dragging = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function handleDrop(e: DragEvent) {
  dragging.value = false
  if (e.dataTransfer?.files) {
    emit('files', Array.from(e.dataTransfer.files))
  }
}

function handleInput(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) {
    emit('files', Array.from(input.files))
  }
}

function triggerFileInput() {
  fileInput.value?.click()
}
</script>

<template>
  <div
    class="border-2 border-dashed rounded-[var(--radius-xl)] p-10 text-center cursor-pointer transition-all"
    :class="dragging ? 'border-[var(--accent)] bg-[var(--accent-light)] scale-[1.01]' : 'border-transparent hover:border-[var(--accent)]'"
    :style="{ backgroundColor: !dragging ? 'var(--card-bg)' : undefined }"
    @dragenter.prevent="dragging = true"
    @dragover.prevent="dragging = true"
    @dragleave="dragging = false"
    @drop.prevent="handleDrop"
    @click="triggerFileInput"
  >
    <input ref="fileInput" type="file" multiple accept="image/*,.pdf" class="hidden" @change="handleInput" />
    <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
      :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
      📄
    </div>
    <p class="text-sm font-semibold mb-1" :style="{ color: 'var(--text)' }">
      {{ dragging ? 'Drop files here' : 'Drop files here or click to browse' }}
    </p>
    <p class="text-xs" :style="{ color: 'var(--text-3)' }">
      Supports images, PDFs, and photos of exercise books
    </p>
  </div>
</template>
