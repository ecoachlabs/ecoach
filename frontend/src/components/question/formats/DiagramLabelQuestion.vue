<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  imageUrl: string
  labels: { id: number; x: number; y: number; correctText: string }[]
  answered: boolean
}>()

const emit = defineEmits<{ submit: [answers: Record<number, string>] }>()
const answers = ref<Record<number, string>>({})
props.labels.forEach(l => { answers.value[l.id] = '' })

const allFilled = () => Object.values(answers.value).every(v => v.trim())

function isCorrect(labelId: number): boolean | null {
  if (!props.answered) return null
  const label = props.labels.find(l => l.id === labelId)
  return label ? answers.value[labelId]?.toLowerCase().trim() === label.correctText.toLowerCase().trim() : null
}
</script>

<template>
  <div class="relative inline-block">
    <!-- Diagram image -->
    <div class="rounded-[var(--radius-lg)] overflow-hidden border" :style="{ borderColor: 'var(--card-border)' }">
      <img :src="imageUrl" alt="Diagram" class="max-w-full" />
    </div>

    <!-- Label inputs positioned on the image -->
    <div v-for="label in labels" :key="label.id" class="absolute" :style="{ left: label.x + '%', top: label.y + '%', transform: 'translate(-50%, -50%)' }">
      <div class="flex items-center gap-1">
        <span class="w-5 h-5 rounded-full flex items-center justify-center text-[9px] font-bold text-white shadow-sm"
          :class="answered && isCorrect(label.id) ? 'bg-emerald-500' : answered && isCorrect(label.id) === false ? 'bg-red-500' : 'bg-[var(--accent)]'">
          {{ label.id }}
        </span>
        <input v-model="answers[label.id]" :disabled="answered"
          class="w-24 px-2 py-1 rounded text-[11px] border shadow-sm"
          :class="answered && isCorrect(label.id) ? 'border-emerald-500 bg-emerald-50' : answered && isCorrect(label.id) === false ? 'border-red-400 bg-red-50' : 'border-[var(--card-border)]'"
          :style="{ backgroundColor: !answered ? 'white' : undefined, color: 'var(--text)' }"
          placeholder="Label..." />
      </div>
      <p v-if="answered && isCorrect(label.id) === false" class="text-[9px] text-emerald-600 mt-0.5 ml-6">{{ label.correctText }}</p>
    </div>

    <div v-if="!answered && allFilled()" class="mt-3 text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white" :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', answers)">Submit Labels</button>
    </div>
  </div>
</template>
