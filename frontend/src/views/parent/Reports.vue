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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Reports</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Download and print progress reports.</p>
    <div class="space-y-3">
      <AppCard v-for="r in ['Weekly Progress', 'Monthly Summary', 'Subject Detail', 'Diagnostic Report']" :key="r" padding="md" hover><div class="flex items-center justify-between"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ r }}</p><AppButton variant="secondary" size="sm">Download PDF</AppButton></div></AppCard>
    </div>
  </div>
</template>
