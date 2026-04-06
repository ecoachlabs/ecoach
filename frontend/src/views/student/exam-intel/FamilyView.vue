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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--ink)' }">Question Family</h1>
    <AppCard padding="lg" class="mb-4"><h3 class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Family Profile</h3><p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">This question family has appeared in 8 of the last 10 BECE papers. Your accuracy against it: 55%.</p></AppCard>
    <AppCard padding="lg"><h3 class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Family Tree</h3><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Visual family tree with parent pattern, siblings, mutations, and descendants will render here.</p></AppCard>
  </div>
</template>
