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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Journey</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Your path from where you are to exam readiness.</p>
    <AppCard padding="lg" glow="accent" class="mb-6"><div class="flex items-center gap-4"><div class="flex gap-1"><span v-for="p in ['Stabilize','Build','Strengthen','Condition','Ready']" :key="p" class="px-2.5 py-1 text-[10px] font-medium rounded-full" :class="p==='Build' ? 'bg-[var(--accent)] text-white' : ''" :style="p!=='Build' ? {backgroundColor:'var(--primary-light)',color:'var(--text-3)'} : {}">{{ p }}</span></div></div></AppCard>
    <AppCard padding="lg" class="text-center"><p class="text-sm mb-4" :style="{ color: 'var(--text-2)' }">Your journey map will visualize stations and progress here.</p><AppButton variant="primary">Start Today's Mission</AppButton></AppCard>
  </div>
</template>
