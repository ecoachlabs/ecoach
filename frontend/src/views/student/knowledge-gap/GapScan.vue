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

    <div class="min-h-[60vh] flex flex-col items-center justify-center text-center"><div class="w-20 h-20 rounded-3xl mx-auto mb-6 flex items-center justify-center text-3xl" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">◌</div><h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--text)' }">Scanning Your Knowledge</h2><p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">Answer a few targeted questions so we can identify your gaps.</p><div class="w-48"><AppProgress :value="35" size="md" color="accent" :glow="true" /></div><p class="text-xs mt-2" :style="{ color: 'var(--text-3)' }">Question 4 of 12</p></div>
  </div>
</template>
