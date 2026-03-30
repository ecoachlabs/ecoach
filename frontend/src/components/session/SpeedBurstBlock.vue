<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{ active: boolean; timeRemaining: number; questionsAnswered: number }>()
</script>

<template>
  <div v-if="active" class="text-center py-4 reveal" data-mode="pressure">
    <div class="w-10 h-10 rounded-xl mx-auto mb-2 flex items-center justify-center text-lg" :style="{backgroundColor:'var(--warm-light)',color:'var(--warm)'}">⚡</div>
    <p class="text-xs font-semibold" :style="{color:'var(--warm)'}">Speed Burst</p>
    <p class="text-[10px]" :style="{color:'var(--text-3)'}">60-second rapid fire · {{ questionsAnswered }} answered · {{ timeRemaining }}s left</p>
  </div>
</template>
