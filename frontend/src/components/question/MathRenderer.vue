<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import katex from 'katex'
import 'katex/dist/katex.min.css'

const props = defineProps<{
  expression: string
  display?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const container = ref<HTMLElement | null>(null)

function render() {
  if (!container.value || !props.expression) return
  try {
    katex.render(props.expression, container.value, {
      throwOnError: false,
      displayMode: props.display ?? false,
      output: 'html',
    })
  } catch {
    if (container.value) container.value.textContent = props.expression
  }
}

onMounted(render)
watch(() => props.expression, render)
</script>

<template>
  <span
    ref="container"
    class="math-renderer"
    :class="[
      size === 'sm' ? 'text-sm' : size === 'lg' ? 'text-xl' : 'text-base',
      display ? 'block text-center my-3' : 'inline',
    ]"
    :style="{ color: 'var(--text)' }"
  >
    {{ expression }}
  </span>
</template>
