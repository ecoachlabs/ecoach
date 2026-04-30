<script setup lang="ts">
import MathText from '../MathText.vue'
import QuestionAssetGallery from '../QuestionAssetGallery.vue'

defineProps<{
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
  selected: number | null
  answered: boolean
  /** Optional — when present, each option fetches any attached images
   *  (scope='option', scope_ref=option.id) and renders them inline. */
  questionId?: number
}>()

defineEmits<{ select: [id: number] }>()

const optionLetters = ['A', 'B', 'C', 'D', 'E', 'F']
</script>

<template>
  <!--
    Nothing-style MCQ list. No filled cards, no rounded outlines at
    rest. Each row is a typographic line: letter (Space Mono, muted) +
    option text (Space Grotesk). Hover creeps padding-left, reveals the
    ink letter and a trailing arrow. Selected state inverts — ink
    background, paper text. Post-answer, correctness is encoded by a
    thin left marker + a tiny mono tag, not filled colored cards.
  -->
  <ul class="mcq" role="list">
    <li
      v-for="(opt, i) in options"
      :key="opt.id"
      class="mcq-item"
    >
      <button
        type="button"
        class="mcq-row"
        :class="{
          'mcq-row--selected':  !answered && selected === opt.id,
          'mcq-row--correct':    answered && opt.is_correct,
          'mcq-row--incorrect':  answered && selected === opt.id && !opt.is_correct,
          'mcq-row--dimmed':     answered && !opt.is_correct && selected !== opt.id,
        }"
        :disabled="answered"
        @click="!answered && $emit('select', opt.id)"
      >
        <span class="mcq-letter">{{ opt.label || optionLetters[i] || String(i + 1) }}</span>

        <span class="mcq-text">
          <MathText :text="opt.text" size="sm" />
          <QuestionAssetGallery
            v-if="questionId != null"
            :question-id="questionId"
            scope="option"
            :scope-ref="opt.id"
          />
        </span>

        <!-- Trailing glyph: at rest invisible, reveals on hover/selected.
             After answer, carries the correctness mono tag. -->
        <span class="mcq-trail" aria-hidden="true">
          <template v-if="answered && opt.is_correct">CORRECT</template>
          <template v-else-if="answered && selected === opt.id">INCORRECT</template>
          <template v-else>→</template>
        </span>
      </button>
    </li>
  </ul>
</template>

<style scoped>
.mcq {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0;
}
.mcq-item { display: block; }

.mcq-row {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) auto;
  align-items: center;
  gap: 28px;
  width: 100%;
  padding: 28px 4px;
  border: none;
  border-top: 1px solid var(--rule, rgba(0, 0, 0, 0.10));
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    background 120ms ease,
    padding-left 200ms cubic-bezier(0.16, 1, 0.3, 1),
    color 140ms ease;
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}
.mcq-item:last-child .mcq-row { border-bottom: 1px solid var(--rule, rgba(0, 0, 0, 0.10)); }

.mcq-row:hover:not(:disabled) {
  background: var(--paper-warm, rgba(0, 0, 0, 0.03));
  padding-left: 20px;
}
.mcq-row:disabled { cursor: default; }

.mcq-letter {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--ink-muted, rgba(0, 0, 0, 0.40));
  transition: color 140ms ease;
}
.mcq-row:hover:not(:disabled) .mcq-letter { color: var(--ink); }

.mcq-text {
  font-family: 'Space Grotesk', sans-serif;
  font-size: clamp(19px, 1.6vw, 26px);
  font-weight: 400;
  line-height: 1.32;
  color: var(--ink);
  min-width: 0;
}

.mcq-trail {
  justify-self: end;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted, rgba(0, 0, 0, 0.40));
  opacity: 0;
  transform: translateX(-4px);
  transition: opacity 140ms ease, transform 200ms cubic-bezier(0.16, 1, 0.3, 1), color 140ms ease;
  align-self: center;
}
.mcq-row:hover:not(:disabled) .mcq-trail,
.mcq-row:focus-visible .mcq-trail {
  opacity: 1;
  transform: translateX(0);
  color: var(--ink);
}

/* ── Selected (pre-submit) — inverts the row, pill-edged via
   padding; no hard outline. One clear "this is your answer" moment. */
.mcq-row--selected {
  background: var(--ink);
  color: var(--paper);
  padding-left: 20px;
}
.mcq-row--selected .mcq-letter { color: var(--paper); }
.mcq-row--selected .mcq-text   { color: var(--paper); }
.mcq-row--selected .mcq-trail  { opacity: 1; transform: translateX(0); color: var(--paper); }

/* ── Post-answer states ── */
.mcq-row--correct .mcq-letter { color: var(--ink); }
.mcq-row--correct .mcq-text   { font-weight: 500; }
.mcq-row--correct .mcq-trail  {
  opacity: 1;
  transform: translateX(0);
  color: var(--success, #15803d);
}
.mcq-row--incorrect .mcq-text { text-decoration: line-through; text-decoration-thickness: 1px; }
.mcq-row--incorrect .mcq-trail {
  opacity: 1;
  transform: translateX(0);
  color: var(--danger, #b91c1c);
}
.mcq-row--dimmed .mcq-letter,
.mcq-row--dimmed .mcq-text  { opacity: 0.4; }

@media (max-width: 760px) {
  .mcq-row { grid-template-columns: 40px minmax(0, 1fr) auto; gap: 16px; padding: 18px 4px; }
  .mcq-row:hover:not(:disabled) { padding-left: 12px; }
  .mcq-row--selected { padding-left: 12px; }
}
</style>
