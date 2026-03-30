<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  totalPages: number
  questionsFound: number
  topicsDetected: string[]
  weaknessesFound: string[]
  coachReaction?: string
}>()

defineEmits<{ done: []; viewReview: [] }>()
</script>

<template>
  <div class="max-w-lg mx-auto text-center reveal-stagger">
    <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-3xl" :style="{backgroundColor:'var(--success-light)',color:'var(--success)'}">✓</div>
    <h2 class="font-display text-xl font-bold mb-1" :style="{color:'var(--text)'}">Upload Complete</h2>
    <p class="text-sm mb-6" :style="{color:'var(--text-2)'}">Your evidence has been processed and analyzed.</p>

    <div class="grid grid-cols-2 gap-3 mb-6">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{color:'var(--accent)'}">{{ totalPages }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Pages Processed</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{color:'var(--gold)'}">{{ questionsFound }}</p>
        <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Questions Found</p>
      </AppCard>
    </div>

    <AppCard v-if="topicsDetected.length" padding="sm" class="mb-4 text-left">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" :style="{color:'var(--text-3)'}">Topics Detected</p>
      <div class="flex flex-wrap gap-1">
        <AppBadge v-for="t in topicsDetected" :key="t" color="accent" size="xs">{{ t }}</AppBadge>
      </div>
    </AppCard>

    <AppCard v-if="weaknessesFound.length" padding="sm" class="mb-4 text-left">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1.5" :style="{color:'var(--warm)'}">Weaknesses Found</p>
      <div class="flex flex-wrap gap-1">
        <AppBadge v-for="w in weaknessesFound" :key="w" color="warm" size="xs">{{ w }}</AppBadge>
      </div>
    </AppCard>

    <AppCard v-if="coachReaction" padding="md" class="mb-6 text-left">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-1" :style="{color:'var(--accent)'}">Coach Reaction</p>
      <p class="text-sm" :style="{color:'var(--text-2)'}">{{ coachReaction }}</p>
    </AppCard>

    <div class="flex items-center justify-center gap-3">
      <AppButton variant="primary" @click="$emit('done')">Back to Home</AppButton>
      <AppButton variant="secondary" @click="$emit('viewReview')">View Smart Review</AppButton>
    </div>
  </div>
</template>
