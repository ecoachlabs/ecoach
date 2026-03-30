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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Intervention Center</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">See what the system is handling, what needs your attention, and what you can do.</p>
    <div class="grid grid-cols-3 gap-4">
      <div><h3 class="text-xs font-semibold uppercase tracking-wider mb-3 text-green-600">System Handling</h3><AppCard padding="md"><p class="text-sm" :style="{ color: 'var(--text-2)' }">Fractions repair sessions scheduled automatically</p></AppCard></div>
      <div><h3 class="text-xs font-semibold uppercase tracking-wider mb-3 text-amber-600">Needs You</h3><AppCard padding="md"><p class="text-sm" :style="{ color: 'var(--text-2)' }">Encourage daily study — activity has dropped</p></AppCard></div>
      <div><h3 class="text-xs font-semibold uppercase tracking-wider mb-3 text-blue-600">You Can Do</h3><AppCard padding="md"><p class="text-sm" :style="{ color: 'var(--text-2)' }">Review Science homework together this weekend</p></AppCard></div>
    </div>
  </div>
</template>
