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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--ink)' }">Author Question</h1>
    <div class="max-w-2xl space-y-4"><AppCard padding="md"><label class="text-xs font-semibold uppercase" :style="{ color: 'var(--ink-muted)' }">Question Stem</label><textarea class="w-full mt-2 p-3 rounded-lg border text-sm" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--ink)' }" rows="3" placeholder="Enter question text..."></textarea></AppCard><AppButton variant="primary">Save Question</AppButton></div>
  </div>
</template>
