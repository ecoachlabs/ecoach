<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

const router = useRouter()
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Your First Diagnostic</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--ink-muted)' }">A short assessment to understand where you are. This helps your coach plan the right path for you.</p>

    <AppCard padding="lg" glow="accent">
      <div class="text-center">
        <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
          :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">◈</div>
        <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--ink)' }">Quick Diagnostic</h2>
        <p class="text-sm mb-2" :style="{ color: 'var(--ink-secondary)' }">15-20 questions · About 15 minutes</p>
        <p class="text-xs mb-6 max-w-sm mx-auto" :style="{ color: 'var(--ink-muted)' }">
          Answer honestly. There is no passing or failing here.
          We want to understand your current level so your coach can build the right plan.
        </p>
        <AppButton variant="primary" size="lg" @click="router.push('/student/diagnostic')">
          Start Diagnostic →
        </AppButton>
      </div>
    </AppCard>

    <div class="mt-6 text-center">
      <AppButton variant="ghost" size="sm" @click="router.push('/student')">Skip for now</AppButton>
    </div>
  </div>
</template>
