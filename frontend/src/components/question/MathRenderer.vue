<script setup lang="ts">
import { computed } from 'vue'
import 'katex/dist/katex.min.css'
import { renderKatex } from '@/utils/mathCache'
import { measureActivePerfSync } from '@/utils/perfTrace'

const props = defineProps<{
  expression: string
  display?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const renderedHtml = computed(() => {
  if (!props.expression) return ''
  return measureActivePerfSync(
    'MathRenderer.renderedHtml',
    () => renderKatex(props.expression, props.display ?? false),
    {
      chars: props.expression.length,
      display: props.display ?? false,
    },
  )
})
</script>

<template>
  <span
    class="math-renderer"
    :class="{ 'math-renderer--display': display }"
    :style="{
      color: 'inherit',
      fontSize: size === 'sm' ? '0.95em' : size === 'lg' ? '1.12em' : '1em',
    }"
    v-html="renderedHtml"
  />
</template>

<style scoped>
.math-renderer {
  display: inline-block;
  vertical-align: baseline;
  line-height: inherit;
}

.math-renderer--display {
  display: block;
  text-align: center;
  margin: 0.65em 0;
}
</style>
