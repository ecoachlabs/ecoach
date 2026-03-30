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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Attention Needed</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Issues that may need your support.</p>
    <div class="space-y-3">
      <AppCard padding="md"><div class="flex items-center gap-3"><div class="w-3 h-3 rounded-full bg-red-500"></div><div class="flex-1"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Declining accuracy in Science</p><p class="text-xs" :style="{ color: 'var(--text-3)' }">Dropped from 68% to 52% over 2 weeks</p></div><AppBadge color="danger" size="xs">High</AppBadge></div></AppCard>
      <AppCard padding="md"><div class="flex items-center gap-3"><div class="w-3 h-3 rounded-full bg-amber-500"></div><div class="flex-1"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">3 days inactive</p><p class="text-xs" :style="{ color: 'var(--text-3)' }">No study sessions since Monday</p></div><AppBadge color="warm" size="xs">Medium</AppBadge></div></AppCard>
    </div>
  </div>
</template>
