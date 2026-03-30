<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'

const props = defineProps<{
  expression: string
  display?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const container = ref<HTMLElement | null>(null)
const rendered = ref(false)

async function render() {
  if (!container.value || !props.expression) return
  try {
    // Dynamic import KaTeX - will be available when package is installed
    const katex = await import('katex').catch(() => null)
    if (katex && container.value) {
      katex.default.render(props.expression, container.value, {
        throwOnError: false,
        displayMode: props.display ?? false,
        output: 'html',
      })
      rendered.value = true
    } else {
      // Fallback: display raw expression
      container.value.textContent = props.expression
    }
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
    class="math-renderer inline"
    :class="[
      size === 'sm' ? 'text-sm' : size === 'lg' ? 'text-xl' : 'text-base',
      display ? 'block text-center my-3' : 'inline',
    ]"
    :style="{ fontFamily: 'var(--font-mono, monospace)', color: 'var(--text)' }"
  >
    {{ expression }}
  </span>
</template>
