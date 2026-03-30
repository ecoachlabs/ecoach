<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Performance</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Academic performance analytics for your child.</p>
    <div class="grid grid-cols-3 gap-4 mb-6">
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">74%</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Accuracy</p></AppCard>
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">28s</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Avg Speed</p></AppCard>
      <AppCard padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">12</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Day Streak</p></AppCard>
    </div>
  </div>
</template>
