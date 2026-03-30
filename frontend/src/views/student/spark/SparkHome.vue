<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

const router = useRouter()
const step = ref(0)

const questions = [
  {
    question: 'What frustrates you most about studying?',
    options: [
      { key: 'overwhelmed', label: 'Too much to learn, don\'t know where to start', icon: '😵' },
      { key: 'bored', label: 'It\'s boring, I lose focus quickly', icon: '😴' },
      { key: 'defeated', label: 'I try but I keep failing', icon: '😞' },
      { key: 'distracted', label: 'I get distracted by other things', icon: '📱' },
      { key: 'disconnected', label: 'I don\'t see why it matters', icon: '🤷' },
      { key: 'nervous', label: 'I panic when I see test questions', icon: '😰' },
    ],
  },
  {
    question: 'How do you prefer to learn?',
    options: [
      { key: 'challenge', label: 'Challenges and competitions', icon: '🏆' },
      { key: 'story', label: 'Stories and real-life examples', icon: '📖' },
      { key: 'quick', label: 'Quick, short activities', icon: '⚡' },
      { key: 'guided', label: 'Step-by-step guidance', icon: '🗺' },
    ],
  },
]

const answers = ref<string[]>([])

function selectAnswer(key: string) {
  answers.value[step.value] = key
  if (step.value < questions.length - 1) {
    step.value++
  } else {
    router.push('/student')
  }
}
</script>

<template>
  <div class="min-h-[80vh] flex flex-col items-center justify-center px-8 reveal-stagger">
    <div class="max-w-lg w-full">
      <!-- Progress -->
      <div class="flex gap-2 mb-8">
        <div v-for="i in questions.length" :key="i"
          class="h-1 flex-1 rounded-full transition-all"
          :style="{ backgroundColor: i <= step + 1 ? 'var(--accent)' : 'var(--border-soft)' }" />
      </div>

      <!-- Question -->
      <h2 class="font-display text-xl font-bold text-center mb-8" :style="{ color: 'var(--text)' }">
        {{ questions[step].question }}
      </h2>

      <!-- Options -->
      <div class="space-y-3">
        <AppCard
          v-for="opt in questions[step].options"
          :key="opt.key"
          hover padding="md"
          @click="selectAnswer(opt.key)"
        >
          <div class="flex items-center gap-3">
            <span class="text-2xl">{{ opt.icon }}</span>
            <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ opt.label }}</p>
          </div>
        </AppCard>
      </div>

      <!-- Skip -->
      <div class="mt-6 text-center">
        <AppButton variant="ghost" size="sm" @click="router.push('/student')">Skip for now</AppButton>
      </div>
    </div>
  </div>
</template>
