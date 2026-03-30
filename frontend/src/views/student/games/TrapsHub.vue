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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Concept Traps</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Can you tell the difference? Five modes to test your concept separation skills.</p>
    <div class="space-y-3">
      <AppCard v-for="m in [{l:'Difference Drill',d:'Sort cards into concept bins',i:'◈'},{l:'Similarity Trap',d:'Survive deliberately deceptive questions',i:'⚠'},{l:'Know the Difference',d:'Compare concepts side by side',i:'⟺'},{l:'Which Is Which',d:'Speed recognition challenge',i:'⚡'},{l:'Unmask',d:'Guess the concept from clues',i:'◉'}]" :key="m.l" hover padding="md">
        <div class="flex items-center gap-3"><div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg" :style="{ backgroundColor: 'var(--gold-light)', color: 'var(--gold)' }">{{ m.i }}</div><div class="flex-1"><p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ m.l }}</p><p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ m.d }}</p></div><AppButton variant="secondary" size="sm">Play</AppButton></div>
      </AppCard>
    </div>
  </div>
</template>
