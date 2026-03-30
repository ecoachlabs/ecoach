<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{ active: boolean; questionCount: number; completedCount: number }>()
</script>

<template>
  <div v-if="active" class="text-center py-4 reveal">
    <div class="w-10 h-10 rounded-xl mx-auto mb-2 flex items-center justify-center text-lg" :style="{backgroundColor:'var(--success-light)',color:'var(--success)'}">☀</div>
    <p class="text-xs font-semibold" :style="{color:'var(--success)'}">Warm Start</p>
    <p class="text-[10px]" :style="{color:'var(--text-3)'}">Easy wins to get you going · {{ completedCount }}/{{ questionCount }}</p>
  </div>
</template>
