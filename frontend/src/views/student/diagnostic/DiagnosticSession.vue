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

    <div class="h-full flex flex-col"><div class="shrink-0 px-6 py-3 flex items-center justify-between border-b" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }"><AppBadge color="accent" size="sm">Diagnostic in Progress</AppBadge><AppButton variant="ghost" size="sm">End</AppButton></div><div class="flex-1 flex items-center justify-center"><AppCard padding="lg" class="text-center max-w-md"><div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">◈</div><h2 class="font-display text-lg font-semibold mb-2" :style="{ color: 'var(--text)' }">Diagnostic Active</h2><p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">Questions will be delivered here as the question rendering engine is connected.</p></AppCard></div></div>
  </div>
</template>
