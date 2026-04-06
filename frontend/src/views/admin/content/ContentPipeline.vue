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
  <div class="flex-1 overflow-y-auto p-7">

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Content Pipeline</h1>
    <div class="grid grid-cols-4 gap-3">
      <AppCard v-for="s in [{l:'Raw',c:12},{l:'Parsed',c:8},{l:'Verified',c:24},{l:'Published',c:156}]" :key="s.l" padding="md" class="text-center"><p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ s.c }}</p><p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--ink-muted)' }">{{ s.l }}</p></AppCard>
    </div>
  </div>
</template>
