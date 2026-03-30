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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Glossary Entry</h1>
    <div class="flex gap-2 mb-6"><AppButton v-for="d in ['Quick','Simple','Exam','Deep','Visual','Audio']" :key="d" variant="secondary" size="sm" :class="d==='Simple'?'!bg-[var(--accent)] !text-white':''">{{ d }}</AppButton></div>
    <AppCard padding="lg" class="mb-4"><h2 class="font-display text-lg font-semibold mb-2" :style="{ color: 'var(--text)' }">Term Definition</h2><p class="text-sm" :style="{ color: 'var(--text-2)' }">The full definition, formula, or concept explanation will render here with KaTeX math support.</p></AppCard>
    <AppCard padding="md"><h3 class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--text-3)' }">Connected Concepts</h3><p class="text-sm" :style="{ color: 'var(--text-3)' }">Related entries will link here.</p></AppCard>
  </div>
</template>
