<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  pairTitle: string
  score: number
  accuracy: number
  rounds: { text: string; isCorrect: boolean; correctChoice: string; yourChoice: string }[]
}>()

defineEmits<{ retry: []; home: [] }>()
</script>

<template>
  <div class="max-w-lg mx-auto reveal-stagger">
    <div class="text-center mb-6">
      <p class="font-display text-3xl font-bold" :class="accuracy >= 70 ? 'text-emerald-600' : accuracy >= 40 ? 'text-amber-600' : 'text-red-500'">
        {{ accuracy }}%
      </p>
      <p class="text-sm" :style="{color:'var(--text-2)'}">{{ pairTitle }}</p>
      <p class="text-xs mt-1" :style="{color:'var(--text-3)'}">Score: {{ score }}</p>
    </div>

    <div class="space-y-2 mb-6">
      <AppCard v-for="(r, i) in rounds" :key="i" padding="sm">
        <div class="flex items-start gap-2">
          <span class="text-sm mt-0.5" :class="r.isCorrect ? 'text-emerald-500' : 'text-red-500'">{{ r.isCorrect ? '✓' : '✕' }}</span>
          <div class="flex-1">
            <p class="text-xs" :style="{color:'var(--text)'}">{{ r.text }}</p>
            <div v-if="!r.isCorrect" class="flex gap-2 mt-1 text-[10px]">
              <span :style="{color:'var(--danger)'}">You: {{ r.yourChoice }}</span>
              <span :style="{color:'var(--success)'}">Correct: {{ r.correctChoice }}</span>
            </div>
          </div>
        </div>
      </AppCard>
    </div>

    <div class="flex gap-3 justify-center">
      <AppButton variant="primary" @click="$emit('retry')">Try Again</AppButton>
      <AppButton variant="ghost" @click="$emit('home')">Back</AppButton>
    </div>
  </div>
</template>
