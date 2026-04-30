<script setup lang="ts">
import MathText from '../MathText.vue'
import QuestionAssetGallery from '../QuestionAssetGallery.vue'

/**
 * Multi-correct MCQ. Unlike McqQuestion (radio), the user toggles
 * individual options and correctness is set-equality: selected set must
 * match the `is_correct` set exactly — no partial credit for this first
 * pass (matches WASSCE marking: all-or-nothing per "choose TWO correct
 * answers" style question).
 */

defineProps<{
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
  /** Currently selected option ids. */
  selected: number[]
  answered: boolean
  /** Optional — when present, each option fetches any attached images
   *  (scope='option', scope_ref=option.id) and renders them inline. */
  questionId?: number
}>()

defineEmits<{ toggle: [id: number] }>()

const optionLetters = ['A', 'B', 'C', 'D', 'E', 'F']
</script>

<template>
  <div class="space-y-2">
    <button
      v-for="(opt, i) in options"
      :key="opt.id"
      class="w-full flex items-start gap-3 p-4 rounded-[var(--radius-lg)] border text-left transition-all"
      :class="[
        answered && opt.is_correct ? 'border-emerald-500 bg-emerald-50' :
        answered && selected.includes(opt.id) && !opt.is_correct ? 'border-red-400 bg-red-50' :
        selected.includes(opt.id) ? 'border-[var(--accent)] bg-[var(--accent-light)]' :
        'border-[var(--card-border)] hover:border-[var(--accent)] hover:bg-[var(--primary-light)]',
      ]"
      :style="{
        backgroundColor: (!answered && !selected.includes(opt.id)) ? 'var(--card-bg)' : undefined,
        transitionDuration: 'var(--dur-fast)',
      }"
      :disabled="answered"
      @click="!answered && $emit('toggle', opt.id)"
    >
      <!-- Square checkbox-style badge distinguishes multi-correct from
           the radio-shaped single-correct variant. -->
      <span
        class="w-8 h-8 rounded flex items-center justify-center text-xs font-bold shrink-0 transition-colors"
        :class="[
          answered && opt.is_correct ? 'bg-emerald-500 text-white' :
          answered && selected.includes(opt.id) && !opt.is_correct ? 'bg-red-400 text-white' :
          selected.includes(opt.id) ? 'bg-[var(--accent)] text-white' :
          'bg-[var(--primary-light)] text-[var(--text-3)]',
        ]"
      >
        {{ selected.includes(opt.id) ? '✓' : (opt.label || optionLetters[i] || String(i + 1)) }}
      </span>

      <span class="text-sm leading-relaxed pt-1 min-w-0" :style="{ color: 'var(--text)' }">
        <MathText :text="opt.text" size="sm" />
        <QuestionAssetGallery
          v-if="questionId != null"
          :question-id="questionId"
          scope="option"
          :scope-ref="opt.id"
        />
      </span>

      <span v-if="answered && opt.is_correct" class="ml-auto text-emerald-600 text-sm shrink-0 pt-1">OK</span>
      <span
        v-if="answered && selected.includes(opt.id) && !opt.is_correct"
        class="ml-auto text-red-500 text-sm shrink-0 pt-1"
      >X</span>
    </button>
  </div>
</template>
