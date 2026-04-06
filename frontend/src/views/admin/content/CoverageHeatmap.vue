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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Coverage Heatmap</h1>
    <AppCard padding="lg" class="min-h-[400px] flex items-center justify-center"><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Topic × content-type coverage matrix heatmap will render here.</p></AppCard>
  </div>
</template>
