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

    <div class="h-full flex flex-col" data-mode="elite"><div class="shrink-0 px-6 py-3 flex items-center justify-between border-b" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"><AppBadge color="accent" size="sm">◆ Elite Session</AppBadge><AppButton variant="ghost" size="sm">End</AppButton></div><div class="flex-1 flex items-center justify-center"><AppCard padding="lg" class="text-center max-w-md"><div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-3xl" :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--accent)' }">◆</div><h2 class="font-display text-lg font-semibold mb-2" :style="{ color: 'var(--ink)' }">Elite Session Active</h2><p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Elite-difficulty questions with premium feedback will render here.</p></AppCard></div></div>
  </div>
</template>
