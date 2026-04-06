<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const step = ref(0)

const questions = [
  {
    question: 'What frustrates you most about studying?',
    options: [
      { key: 'overwhelmed', label: "Too much to learn, don't know where to start" },
      { key: 'bored', label: "It's boring, I lose focus quickly" },
      { key: 'defeated', label: 'I try but I keep failing' },
      { key: 'distracted', label: 'I get distracted by other things' },
      { key: 'disconnected', label: "I don't see why it matters" },
      { key: 'nervous', label: 'I panic when I see test questions' },
    ],
  },
  {
    question: 'How do you prefer to learn?',
    options: [
      { key: 'challenge', label: 'Challenges and competitions' },
      { key: 'story', label: 'Stories and real-life examples' },
      { key: 'quick', label: 'Quick, short activities' },
      { key: 'guided', label: 'Step-by-step guidance' },
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
  <div class="h-full flex overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Left: identity panel -->
    <div
      class="w-64 flex-shrink-0 flex flex-col justify-between p-8 border-r"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow mb-4">Spark Mode</p>
        <h2 class="font-display text-2xl font-bold leading-tight" :style="{ color: 'var(--ink)' }">
          Find your learning style
        </h2>
        <p class="text-xs mt-3" :style="{ color: 'var(--ink-muted)' }">
          Answer a few questions so your coach can adapt your experience.
        </p>
      </div>

      <!-- Step indicator -->
      <div>
        <div class="flex gap-2 mb-3">
          <div
            v-for="(_, i) in questions"
            :key="i"
            class="h-1 rounded-full transition-all duration-300"
            :style="{
              flex: i === step ? '2' : '1',
              backgroundColor: i <= step ? 'var(--ink)' : 'var(--border-soft)',
            }"
          />
        </div>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          Question {{ step + 1 }} of {{ questions.length }}
        </p>
      </div>
    </div>

    <!-- Right: question + options -->
    <div class="flex-1 flex flex-col overflow-hidden">

      <!-- Question -->
      <div class="flex-shrink-0 px-10 pt-12 pb-8 border-b" :style="{ borderColor: 'var(--border-soft)' }">
        <h2 class="font-display text-2xl font-bold" :style="{ color: 'var(--ink)' }">
          {{ questions[step].question }}
        </h2>
      </div>

      <!-- Options -->
      <div class="flex-1 overflow-y-auto px-10 py-6">
        <div class="space-y-2.5 max-w-xl">
          <button
            v-for="opt in questions[step].options"
            :key="opt.key"
            class="option-card w-full text-left"
            @click="selectAnswer(opt.key)"
          >
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ opt.label }}</p>
          </button>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex-shrink-0 px-10 py-5 border-t flex items-center justify-between"
        :style="{ borderColor: 'var(--border-soft)' }">
        <button class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }"
          @click="router.push('/student')">Skip for now →</button>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">~2 minutes to complete</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
}

.option-card {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid var(--border-soft);
  cursor: pointer;
  transition: border-color 120ms ease, transform 100ms ease, background-color 100ms ease;
}
.option-card:hover {
  transform: translateX(4px);
  border-color: var(--ink);
  background-color: var(--paper);
}
</style>
