<script setup lang="ts">
import { computed, watch } from 'vue'
import MathText from '../MathText.vue'

/**
 * Fill-in-the-blank renderer for past-paper authored questions.
 *
 * Distinct from the legacy FillBlankQuestion.vue because the
 * conventions differ: past-paper stems use `[[N]]` markers (1-indexed)
 * and acceptable answers live in question_options rows where
 * `option_label` = the blank index. Multiple rows for the same blank
 * number = alternative accepted answers (case/whitespace-insensitive).
 */

const props = defineProps<{
  stem: string
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
  /** Map from blank index (as string) → current input value. */
  values: Record<string, string>
  answered: boolean
}>()

const emit = defineEmits<{ input: [blank: string, value: string] }>()

interface TextSegment { kind: 'text'; content: string }
interface BlankSegment { kind: 'blank'; index: string }
type Segment = TextSegment | BlankSegment

const segments = computed<Segment[]>(() => {
  const out: Segment[] = []
  const pattern = /\[\[(\d+)\]\]/g
  let cursor = 0
  let match: RegExpExecArray | null
  while ((match = pattern.exec(props.stem)) !== null) {
    if (match.index > cursor) {
      out.push({ kind: 'text', content: props.stem.slice(cursor, match.index) })
    }
    out.push({ kind: 'blank', index: match[1] })
    cursor = pattern.lastIndex
  }
  if (cursor < props.stem.length) {
    out.push({ kind: 'text', content: props.stem.slice(cursor) })
  }
  return out
})

const blankIndices = computed<string[]>(() => {
  const set = new Set<string>()
  for (const seg of segments.value) if (seg.kind === 'blank') set.add(seg.index)
  return Array.from(set).sort((a, b) => Number(a) - Number(b))
})

function acceptableFor(blankIndex: string): string[] {
  return props.options
    .filter(o => o.label === blankIndex)
    .map(o => o.text.trim().toLowerCase())
    .filter(s => s.length > 0)
}

function isBlankCorrect(blankIndex: string): boolean {
  const answer = (props.values[blankIndex] ?? '').trim().toLowerCase()
  if (!answer) return false
  return acceptableFor(blankIndex).includes(answer)
}

function widthForInput(blankIndex: string): number {
  const accepted = acceptableFor(blankIndex)
  const maxLen = accepted.reduce((m, s) => Math.max(m, s.length), 0)
  return Math.max(10, Math.min(28, maxLen + 3))
}

function onInput(ev: Event, blankIndex: string): void {
  const value = (ev.target as HTMLInputElement).value
  emit('input', blankIndex, value)
}

defineExpose({
  allCorrect: () => blankIndices.value.every(isBlankCorrect),
  isBlankCorrect,
  blankIndices,
})

// Silent watcher — placeholder for future focus-management logic; no
// behaviour change, just avoids the "defineExpose returns new refs
// every render" lint when the component is held in a ref upstream.
watch(() => props.answered, () => {})
</script>

<template>
  <div class="pfb-root">
    <p class="pfb-stem" :style="{ color: 'var(--text)' }">
      <template v-for="(seg, i) in segments" :key="i">
        <MathText v-if="seg.kind === 'text'" :text="seg.content" />
        <input
          v-else
          type="text"
          class="pfb-input"
          :class="{
            'pfb-input--ok': answered && isBlankCorrect(seg.index),
            'pfb-input--err': answered && !isBlankCorrect(seg.index),
          }"
          :style="{ width: widthForInput(seg.index) + 'ch' }"
          :value="values[seg.index] ?? ''"
          :disabled="answered"
          :aria-label="`Blank ${seg.index}`"
          :placeholder="`blank ${seg.index}`"
          autocomplete="off"
          spellcheck="false"
          @input="(e) => onInput(e, seg.index)"
        />
      </template>
    </p>

    <div v-if="answered" class="pfb-feedback">
      <div
        v-for="idx in blankIndices"
        :key="`pfb-${idx}`"
        class="pfb-feedback-row"
      >
        <span class="pfb-feedback-label">[[{{ idx }}]]</span>
        <span class="pfb-feedback-given">
          you wrote <strong>{{ values[idx] || '—' }}</strong>
        </span>
        <span
          class="pfb-feedback-verdict"
          :class="{
            'pfb-feedback-verdict--ok': isBlankCorrect(idx),
            'pfb-feedback-verdict--err': !isBlankCorrect(idx),
          }"
        >
          {{ isBlankCorrect(idx) ? 'correct' : 'missed' }}
        </span>
        <span
          v-if="!isBlankCorrect(idx) && acceptableFor(idx).length > 0"
          class="pfb-feedback-accept"
        >
          accepted: {{ acceptableFor(idx).join(' · ') }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pfb-root { display: grid; gap: 14px; }

.pfb-stem {
  font-size: 15px;
  line-height: 1.9;
  margin: 0;
}

.pfb-input {
  display: inline-block;
  margin: 0 4px;
  padding: 2px 6px;
  border: none;
  border-bottom: 2px solid var(--text-3, rgba(0, 0, 0, 0.35));
  background: transparent;
  font-family: inherit;
  font-size: 0.95em;
  color: var(--text, #111);
  outline: none;
  transition: border-color 160ms ease, color 160ms ease;
  min-width: 10ch;
}
.pfb-input:focus:not(:disabled) {
  border-bottom-color: var(--accent, #1a1612);
}
.pfb-input::placeholder {
  color: var(--text-3, rgba(0, 0, 0, 0.35));
  font-style: italic;
  font-size: 0.85em;
}
.pfb-input:disabled { cursor: default; }
.pfb-input--ok {
  border-bottom-color: var(--success, #15803d);
  color: var(--success, #15803d);
}
.pfb-input--err {
  border-bottom-color: var(--danger, #b91c1c);
  color: var(--danger, #b91c1c);
}

.pfb-feedback {
  display: grid;
  gap: 6px;
  padding-top: 10px;
  border-top: 1px solid var(--card-border, rgba(0, 0, 0, 0.1));
}
.pfb-feedback-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 10px;
  font-size: 13px;
  color: var(--text-2, rgba(0, 0, 0, 0.7));
}
.pfb-feedback-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--text-3, rgba(0, 0, 0, 0.5));
}
.pfb-feedback-given strong { color: var(--text, #111); }
.pfb-feedback-verdict {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
}
.pfb-feedback-verdict--ok { color: var(--success, #15803d); }
.pfb-feedback-verdict--err { color: var(--danger, #b91c1c); }
.pfb-feedback-accept {
  font-size: 12px;
  color: var(--text-3, rgba(0, 0, 0, 0.55));
}
</style>
