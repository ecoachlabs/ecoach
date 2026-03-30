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

    <h1 class="text-lg font-bold mb-4" :style="{ color: 'var(--text)' }">Curriculum Management</h1>
    <div class="grid grid-cols-2 gap-3">
      <AppCard v-for="s in [{l:'Upload Source',to:'/admin/curriculum/upload',d:'Upload PDF/Word curriculum'},{l:'Review Extractions',to:'/admin/curriculum/review',d:'Verify parsed content'},{l:'Tree Editor',to:'/admin/curriculum/editor',d:'Edit curriculum hierarchy'},{l:'Publish',to:'/admin/curriculum/publish',d:'Publishing controls'}]" :key="s.l" hover padding="md" @click="router.push(s.to)"><h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ s.l }}</h3><p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ s.d }}</p></AppCard>
    </div>
  </div>
</template>
