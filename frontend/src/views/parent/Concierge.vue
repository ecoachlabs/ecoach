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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Concierge</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Ask questions about your child's academic progress.</p>
    <AppCard padding="lg"><p class="text-sm text-center py-8" :style="{ color: 'var(--text-3)' }">Premium feature. The concierge Q&A interface will be available with your subscription.</p></AppCard>
  </div>
</template>
