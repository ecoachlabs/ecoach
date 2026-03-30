<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  placeholder?: string
  answered: boolean
  correctAnswer?: string
}>()

const emit = defineEmits<{ submit: [answer: string] }>()
const answer = ref('')
</script>

<template>
  <div>
    <div class="relative">
      <input
        v-model="answer"
        :disabled="answered"
        :placeholder="placeholder || 'Type your answer...'"
        class="w-full px-4 py-3 rounded-[var(--radius-lg)] border text-base transition-colors"
        :class="answered
          ? (answer.toLowerCase().trim() === correctAnswer?.toLowerCase().trim()
            ? 'border-emerald-500 bg-emerald-50'
            : 'border-red-400 bg-red-50')
          : 'border-[var(--card-border)] focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]'"
        :style="{ backgroundColor: !answered ? 'var(--card-bg)' : undefined, color: 'var(--text)' }"
        @keydown.enter="!answered && answer.trim() && emit('submit', answer.trim())"
      />
      <button
        v-if="!answered && answer.trim()"
        class="absolute right-2 top-1/2 -translate-y-1/2 px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-medium text-white"
        :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', answer.trim())"
      >
        Submit
      </button>
    </div>
    <div v-if="answered && correctAnswer" class="mt-2 text-sm">
      <span v-if="answer.toLowerCase().trim() === correctAnswer.toLowerCase().trim()" class="text-emerald-600 font-medium">✓ Correct</span>
      <span v-else class="text-red-500">Correct answer: <strong :style="{ color: 'var(--text)' }">{{ correctAnswer }}</strong></span>
    </div>
  </div>
</template>
