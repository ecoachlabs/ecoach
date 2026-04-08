<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  /** Text with ___BLANK___ placeholder(s) */
  stemWithBlanks: string
  answered: boolean
  correctAnswers?: string[]
}>()

const emit = defineEmits<{ submit: [answers: string[]] }>()

const blankCount = computed(() => (props.stemWithBlanks.match(/___BLANK___/g) || []).length)
const answers = ref<string[]>(Array(blankCount.value).fill(''))
const parts = computed(() => props.stemWithBlanks.split('___BLANK___'))
</script>

<template>
  <div class="p-5 rounded-[var(--radius-lg)] leading-relaxed text-base"
    :style="{ backgroundColor: 'var(--card-bg)', color: 'var(--text)' }">
    <template v-for="(part, i) in parts" :key="i">
      <span>{{ part }}</span>
      <input
        v-if="i < parts.length - 1"
        v-model="answers[i]"
        :disabled="answered"
        class="inline-block mx-1 px-2 py-0.5 w-28 border-b-2 text-center text-base font-medium bg-transparent outline-none transition-colors"
        :class="answered
          ? (answers[i]?.toLowerCase().trim() === correctAnswers?.[i]?.toLowerCase().trim()
            ? 'border-emerald-500 text-emerald-700'
            : 'border-red-400 text-red-600')
          : 'border-[var(--accent)] text-[var(--accent)]'"
        placeholder="..."
        @keydown.enter="!answered && answers.every(a => a.trim()) && emit('submit', answers)"
      />
    </template>
  </div>
  <div v-if="!answered && answers.every(a => a.trim())" class="mt-3 text-right">
    <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
      :style="{ backgroundColor: 'var(--accent)' }"
      @click="emit('submit', answers)">Submit</button>
  </div>
</template>
