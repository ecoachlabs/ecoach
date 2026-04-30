<script setup lang="ts">
import { computed } from 'vue'
import MathRenderer from './MathRenderer.vue'
import { stripCurriculumCodes } from '@/utils/learnerCopy'
import { measureActivePerfSync } from '@/utils/perfTrace'

const props = withDefaults(
  defineProps<{
    text?: string | null
    size?: 'sm' | 'md' | 'lg'
  }>(),
  {
    text: '',
    size: 'md',
  },
)

type MathSegment = {
  key: string
  type: 'text' | 'math'
  content: string
  display: boolean
}

function parseMathText(value: string): MathSegment[] {
  const segments: MathSegment[] = []
  const pattern = /\\\[((?:.|\n)*?)\\\]|\\\(((?:.|\n)*?)\\\)|\$\$((?:.|\n)*?)\$\$|\$([^$\n]+?)\$/g
  let cursor = 0
  let index = 0
  let match: RegExpExecArray | null

  while ((match = pattern.exec(value)) !== null) {
    if (match.index > cursor) {
      segments.push({
        key: `text-${index}`,
        type: 'text',
        content: value.slice(cursor, match.index),
        display: false,
      })
      index += 1
    }

    segments.push({
      key: `math-${index}`,
      type: 'math',
      content: match[1] ?? match[2] ?? match[3] ?? match[4] ?? '',
      display: match[1] !== undefined || match[3] !== undefined,
    })
    index += 1
    cursor = pattern.lastIndex
  }

  if (cursor < value.length) {
    segments.push({
      key: `text-${index}`,
      type: 'text',
      content: value.slice(cursor),
      display: false,
    })
  }

  return segments.length
    ? segments
    : [{ key: 'text-0', type: 'text', content: value, display: false }]
}

/**
 * Many backend explanations ship unwrapped LaTeX (e.g. `\frac{6}{12} = \frac{1}{2}`
 * with no surrounding `$…$`). Without wrappers, parseMathText treats them as plain
 * text and KaTeX is never called. This preprocessor wraps bare LaTeX commands so
 * they render correctly, without disturbing already-wrapped math.
 */
const BARE_LATEX = /\\[a-zA-Z]+(?:\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})*(?:\s*[=+\-*/]\s*(?:\\[a-zA-Z]+(?:\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})*|-?\d+(?:\.\d+)?))*/g
const WRAPPED_MATH_SPLIT = /(\$\$(?:.|\n)*?\$\$|\$[^$\n]+?\$|\\\((?:.|\n)*?\\\)|\\\[(?:.|\n)*?\\\])/g

function autoWrapLatex(text: string): string {
  if (!text) return text
  // Fast path: no backslash-letter command, nothing to wrap.
  if (!/\\[a-zA-Z]/.test(text)) return text

  return text.split(WRAPPED_MATH_SPLIT).map(segment => {
    if (!segment) return segment
    // Preserve anything that's already a wrapped math block.
    if (
      segment.startsWith('$$') ||
      (segment.startsWith('$') && !segment.startsWith('$$')) ||
      segment.startsWith('\\(') ||
      segment.startsWith('\\[')
    ) return segment

    // Wrap bare LaTeX commands (and chains joined by = + − × ÷) in `$…$`.
    return segment.replace(BARE_LATEX, m => `$${m}$`)
  }).join('')
}

const displayText = computed(() => measureActivePerfSync(
  'MathText.displayText',
  () => autoWrapLatex(stripCurriculumCodes(props.text ?? '')),
  {
    chars: (props.text ?? '').length,
  },
))
const segments = computed(() => measureActivePerfSync(
  'MathText.parseSegments',
  () => parseMathText(displayText.value),
  {
    chars: displayText.value.length,
  },
))
</script>

<template>
  <span class="math-text">
    <template v-for="segment in segments" :key="segment.key">
      <MathRenderer
        v-if="segment.type === 'math'"
        :expression="segment.content"
        :display="segment.display"
        :size="size"
      />
      <!--
        Text segments render via v-html so author-supplied inline HTML
        (`<sup>`, `<sub>`, `<em>`, `<strong>`, `<br>`, `&deg;`, `&frac12;`
        etc.) displays as intended. Content is admin-authored (packs +
        past-paper seeds), never learner-supplied, so v-html is safe.
        Learner free-text is rendered elsewhere via {{ }} interpolation.
      -->
      <span v-else v-html="segment.content" />
    </template>
  </span>
</template>

<style scoped>
.math-text {
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.math-text :deep(.katex) {
  font-size: 1em;
}
</style>
