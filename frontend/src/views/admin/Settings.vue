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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--text)' }">Admin Settings</h1>
    <div class="space-y-3 max-w-lg"><AppCard v-for="s in ['System Configuration','Backup & Restore','Coach Tuning','Entitlement Management']" :key="s" padding="md"><div class="flex items-center justify-between"><p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ s }}</p><AppButton variant="secondary" size="sm">Configure</AppButton></div></AppCard></div>
  </div>
</template>
