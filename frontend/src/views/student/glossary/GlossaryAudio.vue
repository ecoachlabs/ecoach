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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Audio Radio</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--ink-muted)' }">Listen and learn. Continuous audio revision.</p>
    <div class="grid grid-cols-2 gap-3 mb-6">
      <AppCard v-for="s in ['All Subjects','My Weak Terms','Exam Hotspots','Today\'s Review']" :key="s" hover padding="lg" class="text-center"><div class="text-2xl mb-2">🎧</div><p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ s }}</p></AppCard>
    </div>
    <AppCard padding="md" class="flex items-center gap-4"><div class="w-10 h-10 rounded-full bg-[var(--accent)] flex items-center justify-center text-white text-sm">▶</div><div class="flex-1"><p class="text-xs font-medium" :style="{ color: 'var(--ink)' }">Not playing</p><div class="mt-1"><AppProgress :value="0" size="sm" /></div></div><div class="flex gap-2 text-xs" :style="{ color: 'var(--ink-muted)' }"><span>0.75x</span><span class="font-bold" :style="{ color: 'var(--accent)' }">1x</span><span>1.5x</span></div></AppCard>
  </div>
</template>
