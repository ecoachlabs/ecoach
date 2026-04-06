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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Upload Curriculum Source</h1>
    <AppCard padding="lg" class="max-w-2xl border-2 border-dashed text-center" :style="{ borderColor: 'var(--accent)' }"><div class="py-8"><p class="text-sm font-medium mb-1" :style="{ color: 'var(--ink)' }">Drop curriculum files here</p><p class="text-xs" :style="{ color: 'var(--ink-muted)' }">PDF, Word, Excel, CSV, JSON supported</p></div></AppCard>
  </div>
</template>
