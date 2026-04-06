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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Review Extractions</h1>
    <p class="text-xs mb-4" :style="{ color: 'var(--ink-muted)' }">Split-pane review workbench: original source vs extracted structure.</p>
    <div class="grid grid-cols-2 gap-4 min-h-[400px]"><AppCard padding="md"><h3 class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Original Source</h3><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">PDF preview renders here</p></AppCard><AppCard padding="md"><h3 class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">Extracted Structure</h3><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Parsed curriculum tree renders here</p></AppCard></div>
  </div>
</template>
