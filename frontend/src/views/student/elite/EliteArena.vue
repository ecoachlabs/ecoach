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

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Challenge Arena</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--ink-muted)' }">Choose your challenge type.</p>
    <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
      <AppCard v-for="s in [{l:'Precision Lab',i:'◎',d:'Accuracy focus'},{l:'Elite Sprint',i:'⚡',d:'Speed under pressure'},{l:'Depth Lab',i:'◈',d:'Multi-step problems'},{l:'TrapSense',i:'⚠',d:'Trap detection'},{l:'Endurance Track',i:'∞',d:'30+ questions'},{l:'Perfect Run',i:'★',d:'One error ends it'},{l:'Apex Mock',i:'◉',d:'Full exam, elite difficulty'}]" :key="s.l" hover padding="md">
        <div class="flex items-start gap-2.5"><div class="w-9 h-9 rounded-lg flex items-center justify-center text-base" :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--accent)' }">{{ s.i }}</div><div><p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ s.l }}</p><p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ s.d }}</p></div></div>
      </AppCard>
    </div>
  </div>
</template>
