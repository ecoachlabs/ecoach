<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Elite Insights</h1>
    <div class="grid grid-cols-2 gap-4 mb-6">
      <AppCard padding="lg"><h3 class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--text-3)' }">Performance Trend</h3><p class="text-sm" :style="{ color: 'var(--text-2)' }">Your precision has improved 8% this week. Speed remains your weakest dimension.</p></AppCard>
      <AppCard padding="lg"><h3 class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--text-3)' }">Pressure Behavior</h3><p class="text-sm" :style="{ color: 'var(--text-2)' }">Accuracy drops 12% under timed conditions. Endurance Track will help build resilience.</p></AppCard>
    </div>
  </div>
</template>
