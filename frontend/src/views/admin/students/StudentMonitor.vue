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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Student Monitor</h1>
    <p class="text-xs mb-4" :style="{ color: 'var(--ink-muted)' }">Cross-student analytics dashboard.</p>
    <AppCard padding="lg"><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Student list with filtering by mastery, readiness, and risk. Class-level heatmaps.</p></AppCard>
  </div>
</template>
