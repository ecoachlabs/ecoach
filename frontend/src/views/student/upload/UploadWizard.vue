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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Upload Evidence</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Upload your schoolwork, tests, or exercise books for analysis.</p>
    <div class="max-w-lg mx-auto">
      <AppCard padding="lg" class="text-center border-2 border-dashed" :style="{ borderColor: 'var(--accent)' }"><div class="py-8"><div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">📄</div><p class="text-sm font-semibold mb-1" :style="{ color: 'var(--text)' }">Drop files here or click to browse</p><p class="text-xs" :style="{ color: 'var(--text-3)' }">Supports images, PDFs, and photos of exercise books</p></div></AppCard>
    </div>
  </div>
</template>
