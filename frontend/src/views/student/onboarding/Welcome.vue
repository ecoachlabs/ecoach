<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'

const router = useRouter()
</script>

<template>
  <div class="min-h-[80vh] flex flex-col items-center justify-center text-center px-8 reveal-stagger">
    <div class="w-20 h-20 rounded-3xl bg-gradient-to-br from-teal-500 to-emerald-600 flex items-center justify-center mb-6 shadow-lg shadow-teal-500/20">
      <span class="text-white font-display font-bold text-3xl">e</span>
    </div>
    <h1 class="font-display text-3xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Welcome to eCoach</h1>
    <p class="text-base max-w-md mb-3" :style="{ color: 'var(--text-2)' }">
      Your personal academic intelligence coach for BECE exam preparation.
    </p>
    <p class="text-sm max-w-sm mb-8" :style="{ color: 'var(--text-3)' }">
      We will help you understand your strengths, repair your weaknesses, and prepare you for exam success.
    </p>
    <div class="space-y-3">
      <AppButton variant="primary" size="lg" @click="router.push('/student/onboarding/subjects')">
        Get Started →
      </AppButton>
      <p class="text-xs" :style="{ color: 'var(--text-3)' }">Takes about 5 minutes to set up</p>
    </div>
  </div>
</template>
