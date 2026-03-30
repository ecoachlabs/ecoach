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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Analytics</h1>
    <div class="grid grid-cols-4 gap-3 mb-6">
      <AppCard v-for="s in [{l:'Accuracy',v:'74%',c:'var(--success)'},{l:'Avg Speed',v:'28s',c:'var(--accent)'},{l:'Retention',v:'61%',c:'var(--gold)'},{l:'Pressure',v:'52%',c:'var(--danger)'}]" :key="s.l" padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: s.c }">{{ s.v }}</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">{{ s.l }}</p>
      </AppCard>
    </div>
    <AppCard padding="lg" class="min-h-[200px] flex items-center justify-center"><p class="text-sm" :style="{ color: 'var(--text-3)' }">Trend line charts and error distribution visualizations will render here.</p></AppCard>
  </div>
</template>
