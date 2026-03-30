<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  prompt: string
  sessionAccuracy?: number
  correctCount?: number
  totalCount?: number
}>()

defineEmits<{ submit: [reflection: string]; skip: [] }>()

const reflection = ref('')
const prompts = [
  'What felt hardest in this session?',
  'Which topic do you want to review?',
  'How confident do you feel now?',
]
</script>

<template>
  <AppCard padding="lg" class="max-w-lg mx-auto text-center">
    <div class="w-12 h-12 rounded-2xl mx-auto mb-3 flex items-center justify-center text-xl"
      :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">🪞</div>
    <h3 class="font-display text-base font-semibold mb-1" :style="{ color: 'var(--text)' }">Quick Reflection</h3>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-2)' }">{{ prompt }}</p>

    <textarea v-model="reflection" rows="3"
      class="w-full px-3 py-2 rounded-[var(--radius-md)] border text-sm resize-none mb-4"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
      placeholder="Type your thoughts..." />

    <div class="flex items-center justify-center gap-2">
      <AppButton variant="primary" :disabled="!reflection.trim()" @click="$emit('submit', reflection)">Submit</AppButton>
      <AppButton variant="ghost" size="sm" @click="$emit('skip')">Skip</AppButton>
    </div>
  </AppCard>
</template>
