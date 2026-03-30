<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  questionText: string
  selectedAnswer: string
  correctAnswer: string
}>()

defineEmits<{ complete: [] }>()

const step = ref(0)
const steps = [
  { title: 'What Happened', icon: '❓', desc: 'Let us look at what went wrong.' },
  { title: 'Why It Was Tempting', icon: '🎯', desc: 'Your answer felt right because...' },
  { title: 'What Clue Was Missed', icon: '🔍', desc: 'The key distinction you missed was...' },
  { title: 'Repair Move', icon: '🔧', desc: 'Here is how to avoid this next time.' },
  { title: 'Confirmed Recovery', icon: '✓', desc: 'Let us verify you have got it now.' },
]
</script>

<template>
  <div class="max-w-xl mx-auto reveal-stagger">
    <!-- Progress -->
    <div class="flex gap-1 mb-6">
      <div v-for="(s, i) in steps" :key="i" class="flex-1 h-1.5 rounded-full transition-all"
        :style="{ backgroundColor: i <= step ? 'var(--accent)' : 'var(--border-soft)', transitionDuration: 'var(--dur-normal)' }" />
    </div>

    <!-- Step Header -->
    <div class="text-center mb-6">
      <span class="text-3xl mb-2 block">{{ steps[step].icon }}</span>
      <h3 class="font-display text-lg font-semibold" :style="{ color: 'var(--text)' }">{{ steps[step].title }}</h3>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Step {{ step + 1 }} of {{ steps.length }}</p>
    </div>

    <!-- Step Content -->
    <AppCard padding="lg" class="mb-6">
      <!-- Step 1: What happened -->
      <div v-if="step === 0">
        <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--text-3)' }">You answered:</p>
        <div class="p-3 rounded-[var(--radius-md)] bg-red-50 text-red-700 text-sm mb-3">{{ selectedAnswer }}</div>
        <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--text-3)' }">Correct answer:</p>
        <div class="p-3 rounded-[var(--radius-md)] bg-emerald-50 text-emerald-700 text-sm">{{ correctAnswer }}</div>
      </div>
      <!-- Other steps -->
      <div v-else>
        <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ steps[step].desc }}</p>
        <p class="text-sm mt-3" :style="{ color: 'var(--text-3)' }">Detailed coaching content for this step will be generated from the wrong-answer intelligence engine.</p>
      </div>
    </AppCard>

    <!-- Navigation -->
    <div class="flex items-center justify-between">
      <AppButton v-if="step > 0" variant="ghost" @click="step--">← Back</AppButton>
      <div v-else />
      <AppButton v-if="step < steps.length - 1" variant="primary" @click="step++">Continue →</AppButton>
      <AppButton v-else variant="primary" @click="$emit('complete')">Done ✓</AppButton>
    </div>
  </div>
</template>
