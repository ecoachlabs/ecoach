<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  score: number
  level: number
  questionsAnswered: number
  questionsCorrect: number
  bestStreak: number
  linesCleared: number
  personalBest: boolean
}>()

defineEmits<{ retry: []; review: []; home: [] }>()
</script>

<template>
  <div class="max-w-sm mx-auto text-center reveal-stagger">
    <div v-if="personalBest" class="py-2 px-4 rounded-full inline-block mb-4" :style="{backgroundColor:'var(--gold-light)'}">
      <span class="text-sm font-bold" :style="{color:'var(--gold)'}">🏆 New Personal Best!</span>
    </div>

    <h2 class="font-display text-2xl font-bold mb-1" :style="{color:'var(--text)'}">Game Over</h2>
    <p class="font-display text-4xl font-bold mb-6" :style="{color:'var(--accent)'}">{{ score.toLocaleString() }}</p>

    <div class="grid grid-cols-2 gap-3 mb-6">
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-lg font-bold" :style="{color:'var(--text)'}">L{{ level }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Level</p>
      </AppCard>
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-lg font-bold" :style="{color:'var(--gold)'}">{{ bestStreak }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Best Streak</p>
      </AppCard>
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-lg font-bold" :style="{color:'var(--success)'}">{{ questionsCorrect }}/{{ questionsAnswered }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Correct</p>
      </AppCard>
      <AppCard padding="sm" class="text-center">
        <p class="font-display text-lg font-bold" :style="{color:'var(--accent)'}">{{ linesCleared }}</p>
        <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">Lines</p>
      </AppCard>
    </div>

    <div class="flex flex-col gap-2">
      <AppButton variant="primary" class="w-full" @click="$emit('retry')">Play Again</AppButton>
      <AppButton variant="secondary" class="w-full" @click="$emit('review')">Review Missed Questions</AppButton>
      <AppButton variant="ghost" @click="$emit('home')">Back to Games</AppButton>
    </div>
  </div>
</template>
