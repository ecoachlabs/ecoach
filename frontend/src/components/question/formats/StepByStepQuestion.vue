<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  totalSteps: number
  stepPrompts?: string[]
  answered: boolean
  correctSteps?: string[]
}>()

const emit = defineEmits<{ submit: [steps: string[]] }>()
const steps = ref<string[]>(Array(props.totalSteps).fill(''))
const allFilled = () => steps.value.every(s => s.trim())
</script>

<template>
  <div class="space-y-3">
    <p class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Show your working step by step</p>
    <div v-for="(_, i) in steps" :key="i" class="flex items-start gap-3">
      <span class="w-7 h-7 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0 mt-1"
        :class="answered && correctSteps?.[i] && steps[i].toLowerCase().trim() === correctSteps[i].toLowerCase().trim() ? 'bg-emerald-100 text-emerald-700' : answered ? 'bg-red-50 text-red-500' : ''"
        :style="!answered ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
        {{ i + 1 }}
      </span>
      <div class="flex-1">
        <p v-if="stepPrompts?.[i]" class="text-[10px] mb-1" :style="{ color: 'var(--text-3)' }">{{ stepPrompts[i] }}</p>
        <input v-model="steps[i]" :disabled="answered"
          class="w-full px-3 py-2 rounded-[var(--radius-md)] border text-sm"
          :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
          :placeholder="`Step ${i + 1}...`" />
        <p v-if="answered && correctSteps?.[i] && steps[i].toLowerCase().trim() !== correctSteps[i].toLowerCase().trim()"
          class="text-[10px] mt-1 text-emerald-600">
          Correct: {{ correctSteps[i] }}
        </p>
      </div>
    </div>
    <div v-if="!answered && allFilled()" class="text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white" :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', steps)">Submit Steps</button>
    </div>
  </div>
</template>
