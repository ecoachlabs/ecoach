<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineEmits<{ save: [question: any] }>()

const stem = ref('')
const format = ref('mcq')
const difficulty = ref('medium')
const options = ref([
  { label: 'A', text: '', isCorrect: false },
  { label: 'B', text: '', isCorrect: false },
  { label: 'C', text: '', isCorrect: false },
  { label: 'D', text: '', isCorrect: false },
])
const explanation = ref('')
const topicId = ref<number | null>(null)

function addOption() {
  const labels = 'ABCDEFGH'
  options.value.push({ label: labels[options.value.length] || '?', text: '', isCorrect: false })
}

function removeOption(idx: number) {
  options.value.splice(idx, 1)
}

function setCorrect(idx: number) {
  options.value.forEach((o, i) => { o.isCorrect = i === idx })
}
</script>

<template>
  <div class="space-y-4 max-w-2xl">
    <AppTextarea v-model="stem" label="Question Stem" placeholder="Enter the question text..." :rows="3" />

    <div class="grid grid-cols-2 gap-4">
      <AppSelect v-model="format" label="Format" :options="[
        { value: 'mcq', label: 'Multiple Choice' },
        { value: 'short_answer', label: 'Short Answer' },
        { value: 'true_false', label: 'True/False' },
        { value: 'fill_blank', label: 'Fill in the Blank' },
      ]" />
      <AppSelect v-model="difficulty" label="Difficulty" :options="[
        { value: 'easy', label: 'Easy' },
        { value: 'medium', label: 'Medium' },
        { value: 'hard', label: 'Hard' },
      ]" />
    </div>

    <!-- Options (for MCQ) -->
    <div v-if="format === 'mcq'">
      <p class="text-xs font-medium uppercase tracking-wide mb-2" :style="{ color: 'var(--text-3)' }">Answer Options</p>
      <div class="space-y-2">
        <div v-for="(opt, i) in options" :key="i" class="flex items-center gap-2">
          <button class="w-8 h-8 rounded-lg text-xs font-bold flex items-center justify-center border transition-colors"
            :class="opt.isCorrect ? 'bg-emerald-500 text-white border-emerald-500' : 'border-[var(--card-border)] text-[var(--text-3)]'"
            @click="setCorrect(i)" title="Mark as correct">
            {{ opt.label }}
          </button>
          <input v-model="opt.text" class="flex-1 px-3 py-2 rounded-[var(--radius-md)] border text-sm"
            :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
            :placeholder="`Option ${opt.label}...`" />
          <button v-if="options.length > 2" class="text-xs text-red-400 hover:text-red-600" @click="removeOption(i)">✕</button>
        </div>
      </div>
      <AppButton v-if="options.length < 6" variant="ghost" size="sm" class="mt-2" @click="addOption">+ Add Option</AppButton>
    </div>

    <AppTextarea v-model="explanation" label="Explanation" placeholder="Why is this the correct answer?" :rows="2" />

    <AppButton variant="primary" @click="$emit('save', { stem, format, difficulty, options, explanation, topicId })">
      Save Question
    </AppButton>
  </div>
</template>
