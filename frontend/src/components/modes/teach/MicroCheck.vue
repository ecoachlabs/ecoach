<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  question: string
  options: { id: number; text: string; correct: boolean }[]
}>()

defineEmits<{ answer: [correct: boolean] }>()

const selected = ref<number | null>(null)
const answered = ref(false)

function select(id: number, correct: boolean) {
  selected.value = id
  answered.value = true
  setTimeout(() => { /* emit after brief delay */ }, 300)
}
</script>

<template>
  <AppCard padding="md" class="border-l-4" :style="{borderLeftColor:'var(--accent)'}">
    <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--accent)'}">✓ Quick Check</p>
    <p class="text-sm mb-3" :style="{color:'var(--text)'}">{{ question }}</p>
    <div class="space-y-1.5">
      <button v-for="opt in options" :key="opt.id"
        class="w-full text-left px-3 py-2 rounded-[var(--radius-sm)] text-xs border transition-all"
        :class="answered && opt.correct ? 'border-emerald-500 bg-emerald-50' : answered && selected === opt.id && !opt.correct ? 'border-red-400 bg-red-50' : selected === opt.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' : 'border-[var(--card-border)]'"
        :style="{backgroundColor: !answered && selected !== opt.id ? 'var(--card-bg)' : undefined, color:'var(--text)'}"
        :disabled="answered"
        @click="select(opt.id, opt.correct); $emit('answer', opt.correct)">
        {{ opt.text }}
      </button>
    </div>
  </AppCard>
</template>
