<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  clues: string[]
  revealedCount: number
  options: string[]
  active: boolean
}>()

defineEmits<{ guess: [concept: string]; revealNext: [] }>()
</script>

<template>
  <div class="text-center">
    <AppBadge color="gold" size="sm" class="mb-4">◉ Unmask</AppBadge>

    <!-- Clues revealed so far -->
    <AppCard padding="lg" class="mb-6 max-w-md mx-auto">
      <p class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Clues</p>
      <div class="space-y-2">
        <div v-for="(clue, i) in clues.slice(0, revealedCount)" :key="i"
          class="px-3 py-2 rounded-[var(--radius-md)] text-sm text-left reveal"
          :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text)' }">
          {{ i + 1 }}. {{ clue }}
        </div>
        <div v-for="i in clues.length - revealedCount" :key="'hidden-' + i"
          class="px-3 py-2 rounded-[var(--radius-md)] text-sm text-center"
          :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--text-3)' }">
          🔒 Hidden clue
        </div>
      </div>
    </AppCard>

    <!-- Reveal more button -->
    <div v-if="revealedCount < clues.length && active" class="mb-4">
      <AppButton variant="secondary" size="sm" @click="$emit('revealNext')">
        Reveal Next Clue (-{{ 10 * (revealedCount + 1) }} pts)
      </AppButton>
    </div>

    <!-- Guess options -->
    <div v-if="active" class="max-w-sm mx-auto">
      <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Which concept is this?</p>
      <div class="space-y-2">
        <button v-for="opt in options" :key="opt"
          class="w-full py-3 rounded-[var(--radius-lg)] border text-sm font-medium transition-all active:scale-97"
          :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
          @click="$emit('guess', opt)">
          {{ opt }}
        </button>
      </div>
    </div>
  </div>
</template>
