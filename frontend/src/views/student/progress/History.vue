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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Activity History</h1>
    <div class="space-y-3">
      <AppCard v-for="i in 8" :key="i" padding="sm"><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-lg flex items-center justify-center text-xs font-bold" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">{{ 30 - i }}</div><div class="flex-1"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Practice Session</p><p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ i }} day{{ i > 1 ? 's' : '' }} ago · 12 questions</p></div><AppBadge color="success" size="xs">82%</AppBadge></div></AppCard>
    </div>
  </div>
</template>
