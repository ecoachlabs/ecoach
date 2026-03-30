<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  questions: { id: number; text: string }[]
  answers: { id: number; text: string }[]
}>()

defineEmits<{ align: [alignments: Record<number, number>]; skip: [] }>()

const alignments = ref<Record<number, number>>({})
const selectedQuestion = ref<number | null>(null)

function selectQuestion(id: number) { selectedQuestion.value = id }
function selectAnswer(id: number) {
  if (selectedQuestion.value !== null) {
    alignments.value[selectedQuestion.value] = id
    selectedQuestion.value = null
  }
}
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Match Questions to Answers</h3>
    <p class="text-xs mb-4" :style="{color:'var(--text-3)'}">Click a question, then click its matching answer.</p>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--accent)'}">Questions</p>
        <div class="space-y-1.5">
          <button v-for="q in questions" :key="q.id" class="w-full text-left px-3 py-2 rounded-[var(--radius-md)] border text-xs transition-all"
            :class="selectedQuestion === q.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' : q.id in alignments ? 'opacity-50' : 'border-[var(--card-border)]'"
            :style="{backgroundColor: selectedQuestion !== q.id && !(q.id in alignments) ? 'var(--card-bg)' : undefined, color:'var(--text)'}"
            @click="selectQuestion(q.id)">
            <AppBadge color="accent" size="xs" class="mr-1">Q{{ q.id }}</AppBadge> {{ q.text }}
          </button>
        </div>
      </div>
      <div>
        <p class="text-[10px] font-semibold uppercase mb-2" :style="{color:'var(--gold)'}">Answers</p>
        <div class="space-y-1.5">
          <button v-for="a in answers" :key="a.id" class="w-full text-left px-3 py-2 rounded-[var(--radius-md)] border text-xs transition-all"
            :class="Object.values(alignments).includes(a.id) ? 'opacity-50' : selectedQuestion !== null ? 'border-[var(--card-border)] hover:border-[var(--gold)]' : 'border-[var(--card-border)]'"
            :style="{backgroundColor:'var(--card-bg)',color:'var(--text)'}"
            :disabled="selectedQuestion === null"
            @click="selectAnswer(a.id)">
            <AppBadge color="gold" size="xs" class="mr-1">A{{ a.id }}</AppBadge> {{ a.text }}
          </button>
        </div>
      </div>
    </div>
    <div class="mt-4 flex gap-2">
      <AppButton variant="primary" :disabled="Object.keys(alignments).length < questions.length"
        @click="$emit('align', alignments)">Continue →</AppButton>
      <AppButton variant="ghost" size="sm" @click="$emit('skip')">Skip alignment</AppButton>
    </div>
  </div>
</template>
