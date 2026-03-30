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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--text)' }">Quality Dashboard</h1>
    <div class="grid grid-cols-3 gap-3"><AppCard v-for="s in [{l:'Content Quality',v:'87%'},{l:'Question Coverage',v:'92%'},{l:'Trust Score',v:'94%'}]" :key="s.l" padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{ color: 'var(--primary)' }">{{ s.v }}</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">{{ s.l }}</p></AppCard></div>
  </div>
</template>
