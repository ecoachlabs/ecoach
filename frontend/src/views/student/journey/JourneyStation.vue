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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Station Detail</h1>
    <AppCard padding="lg"><p class="text-sm" :style="{ color: 'var(--text-2)' }">Station content, requirements, and session launcher will be here.</p></AppCard>
  </div>
</template>
