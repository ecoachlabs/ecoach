<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  won: boolean
  score: number
  questionsCorrect: number
  questionsTotal: number
  maxPull: number
}>()

defineEmits<{ rematch: []; home: [] }>()
</script>

<template>
  <div class="max-w-sm mx-auto text-center reveal-stagger">
    <div class="text-5xl mb-4">{{ won ? '🏆' : '💪' }}</div>
    <h2 class="font-display text-2xl font-bold mb-1" :style="{color: won ? 'var(--success)' : 'var(--warm)'}">
      {{ won ? 'Victory!' : 'Good Fight!' }}
    </h2>
    <p class="text-sm mb-6" :style="{color:'var(--text-2)'}">
      {{ won ? 'You pulled the rope all the way!' : 'You put up a strong fight. Try again?' }}
    </p>

    <div class="grid grid-cols-3 gap-3 mb-6">
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-xl font-bold" :style="{color:'var(--accent)'}">{{ score }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Score</p>
      </AppCard>
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-xl font-bold" :style="{color:'var(--success)'}">{{ questionsCorrect }}/{{ questionsTotal }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Correct</p>
      </AppCard>
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-xl font-bold" :style="{color:'var(--gold)'}">{{ maxPull }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Max Pull</p>
      </AppCard>
    </div>

    <div class="flex gap-3 justify-center">
      <AppButton variant="warm" @click="$emit('rematch')">Rematch</AppButton>
      <AppButton variant="ghost" @click="$emit('home')">Back</AppButton>
    </div>
  </div>
</template>
