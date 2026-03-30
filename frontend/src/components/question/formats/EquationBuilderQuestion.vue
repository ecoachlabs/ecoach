<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  prompt: string
  availableSymbols: string[]
  answered: boolean
  correctEquation?: string
}>()

const emit = defineEmits<{ submit: [equation: string] }>()
const equation = ref('')

function addSymbol(sym: string) {
  equation.value += sym
}

function backspace() {
  equation.value = equation.value.slice(0, -1)
}

function clear() {
  equation.value = ''
}
</script>

<template>
  <div>
    <p class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">{{ prompt || 'Build the equation' }}</p>

    <!-- Equation display -->
    <div class="px-4 py-3 rounded-[var(--radius-lg)] border mb-4 min-h-[48px] flex items-center font-mono text-lg"
      :class="answered && equation.trim() === correctEquation?.trim() ? 'border-emerald-500 bg-emerald-50' : answered ? 'border-red-400 bg-red-50' : 'border-[var(--card-border)]'"
      :style="{ backgroundColor: !answered ? 'var(--card-bg)' : undefined, color: 'var(--text)' }">
      {{ equation || '...' }}
    </div>

    <!-- Symbol pad -->
    <div v-if="!answered" class="flex flex-wrap gap-1.5 mb-4">
      <button v-for="sym in availableSymbols" :key="sym"
        class="w-10 h-10 rounded-lg border text-base font-mono flex items-center justify-center transition-all hover:border-[var(--accent)] hover:bg-[var(--accent-light)] active:scale-95"
        :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
        @click="addSymbol(sym)">
        {{ sym }}
      </button>
      <button class="w-10 h-10 rounded-lg border text-sm flex items-center justify-center hover:bg-red-50 hover:border-red-300 hover:text-red-500"
        :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text-3)' }"
        @click="backspace">←</button>
      <button class="w-10 h-10 rounded-lg border text-[10px] font-medium flex items-center justify-center hover:bg-red-50 hover:border-red-300 hover:text-red-500"
        :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text-3)' }"
        @click="clear">CLR</button>
    </div>

    <div v-if="!answered && equation.trim()" class="text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white" :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', equation)">Submit Equation</button>
    </div>

    <p v-if="answered && correctEquation && equation.trim() !== correctEquation.trim()" class="text-sm mt-2">
      <span class="text-emerald-600 font-medium">Correct: </span>
      <span class="font-mono" :style="{ color: 'var(--text)' }">{{ correctEquation }}</span>
    </p>
  </div>
</template>
