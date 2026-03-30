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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--text)' }">Student Monitor</h1>
    <p class="text-xs mb-4" :style="{ color: 'var(--text-3)' }">Cross-student analytics dashboard.</p>
    <AppCard padding="lg"><p class="text-sm" :style="{ color: 'var(--text-3)' }">Student list with filtering by mastery, readiness, and risk. Class-level heatmaps.</p></AppCard>
  </div>
</template>
