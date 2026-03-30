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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Diagnostic Report</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Your comprehensive academic profile.</p>
    <div class="grid grid-cols-3 gap-4 mb-6">
      <AppCard padding="lg" class="text-center"><p class="font-display text-3xl font-bold" :style="{ color: 'var(--accent)' }">48%</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Overall Readiness</p></AppCard>
      <AppCard padding="lg" class="text-center"><p class="font-display text-3xl font-bold" :style="{ color: 'var(--success)' }">6</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Strong Topics</p></AppCard>
      <AppCard padding="lg" class="text-center"><p class="font-display text-3xl font-bold" :style="{ color: 'var(--danger)' }">4</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Weak Topics</p></AppCard>
    </div>
    <AppButton variant="secondary" size="sm">Export PDF</AppButton>
  </div>
</template>
