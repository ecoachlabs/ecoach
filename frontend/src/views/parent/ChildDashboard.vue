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

    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Child Dashboard</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Detailed performance view for your child.</p>
      </div>
    </div>
    <div class="grid grid-cols-2 gap-4 mb-6">
      <AppCard padding="lg" class="text-center"><p class="font-display text-3xl font-bold" :style="{ color: 'var(--success)' }">72%</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Overall Readiness</p></AppCard>
      <AppCard padding="lg" class="text-center"><p class="font-display text-3xl font-bold" :style="{ color: 'var(--accent)' }">8</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Topics Mastered</p></AppCard>
    </div>
    <AppCard padding="md"><p class="text-sm" :style="{ color: 'var(--text-2)' }">Your child is making steady progress in Mathematics. Fractions remain a weak area needing focused attention this week.</p></AppCard>
  </div>
</template>
