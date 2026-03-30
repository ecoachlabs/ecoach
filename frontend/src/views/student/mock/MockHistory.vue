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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Battle History</h1>
    <div class="space-y-3">
      <AppCard v-for="i in 5" :key="i" padding="md"><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold" :class="i % 2 ? 'bg-emerald-50 text-emerald-600' : 'bg-amber-50 text-amber-600'">{{ 50 + i * 5 }}%</div><div class="flex-1"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Full Mock #{{ 6 - i }}</p><p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ i }} days ago</p></div></div></AppCard>
    </div>
  </div>
</template>
