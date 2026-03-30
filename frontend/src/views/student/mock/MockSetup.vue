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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Mock Setup</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Configure your mock exam.</p>
    <div class="max-w-lg space-y-6">
      <AppCard padding="md"><p class="text-sm font-medium mb-2" :style="{ color: 'var(--text)' }">Subject</p><div class="flex gap-2"><AppButton variant="primary" size="sm">Mathematics</AppButton><AppButton variant="secondary" size="sm">English</AppButton><AppButton variant="secondary" size="sm">Science</AppButton></div></AppCard>
      <AppCard padding="md"><p class="text-sm font-medium mb-2" :style="{ color: 'var(--text)' }">Duration</p><div class="flex gap-2"><AppButton variant="secondary" size="sm">30 min</AppButton><AppButton variant="primary" size="sm">1 hour</AppButton><AppButton variant="secondary" size="sm">2 hours</AppButton></div></AppCard>
      <AppButton variant="primary" size="lg" @click="$router.push('/student/mock/hall/1')">Enter Exam Hall →</AppButton>
    </div>
  </div>
</template>
