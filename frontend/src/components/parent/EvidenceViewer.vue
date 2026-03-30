<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  pages: { id: number; type: string; thumbnail?: string; questionsFound: number }[]
  totalQuestions: number
  accuracy: number
}>()
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{color:'var(--text-3)'}">Uploaded Evidence</h3>
      <div class="flex gap-2">
        <AppBadge color="accent" size="sm">{{ totalQuestions }} questions</AppBadge>
        <AppBadge :color="accuracy >= 70 ? 'success' : accuracy >= 40 ? 'gold' : 'danger'" size="sm">{{ accuracy }}% accuracy</AppBadge>
      </div>
    </div>
    <div class="grid grid-cols-4 gap-2">
      <AppCard v-for="page in pages" :key="page.id" padding="sm" class="text-center">
        <div class="w-full aspect-[3/4] rounded-[var(--radius-sm)] mb-2 flex items-center justify-center"
          :style="{backgroundColor:'var(--primary-light)'}">
          <span class="text-2xl">📄</span>
        </div>
        <AppBadge :color="page.type === 'question' ? 'accent' : page.type === 'answer' ? 'success' : 'gold'" size="xs">
          {{ page.type }}
        </AppBadge>
        <p v-if="page.questionsFound" class="text-[9px] mt-1" :style="{color:'var(--text-3)'}">{{ page.questionsFound }} questions</p>
      </AppCard>
    </div>
  </div>
</template>
