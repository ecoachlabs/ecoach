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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-6" :style="{ color: 'var(--text)' }">Settings</h1>
    <div class="space-y-4 max-w-lg">
      <AppCard padding="md"><div class="flex items-center justify-between"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Change PIN</p><AppButton variant="secondary" size="sm">Update</AppButton></div></AppCard>
      <AppCard padding="md"><div class="flex items-center justify-between"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Sound Effects</p><AppButton variant="secondary" size="sm">On</AppButton></div></AppCard>
      <AppCard padding="md"><div class="flex items-center justify-between"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">Dark Mode</p><AppButton variant="secondary" size="sm">Off</AppButton></div></AppCard>
    </div>
  </div>
</template>
