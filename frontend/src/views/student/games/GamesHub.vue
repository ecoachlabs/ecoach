<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const router = useRouter()

const games = [
  {
    key: 'mindstack', label: 'MindStack', to: '/student/games/mindstack',
    desc: 'Control through knowledge. Answer correctly to control falling blocks.',
    icon: '▣', color: 'accent', modes: 7,
  },
  {
    key: 'tugofwar', label: 'Tug of War', to: '/student/games/tugofwar',
    desc: 'Pressure-based recall battles. Pull the rope with correct answers.',
    icon: '⟷', color: 'warm', modes: 3,
  },
  {
    key: 'traps', label: 'Concept Traps', to: '/student/games/traps',
    desc: 'Can you tell the difference? Sort, classify, and survive deception.',
    icon: '◈', color: 'gold', modes: 5,
  },
]
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Games</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Learn through play. Every game trains a real academic skill.</p>

    <div class="space-y-4">
      <AppCard v-for="game in games" :key="game.key" hover padding="lg" @click="router.push(game.to)">
        <div class="flex items-start gap-4">
          <div class="w-14 h-14 rounded-2xl flex items-center justify-center text-2xl shadow-sm"
            :style="{ backgroundColor: `var(--${game.color}-light, var(--accent-light))`, color: `var(--${game.color}, var(--accent))` }">
            {{ game.icon }}
          </div>
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <h3 class="text-base font-semibold" :style="{ color: 'var(--text)' }">{{ game.label }}</h3>
              <AppBadge color="muted" size="xs">{{ game.modes }} modes</AppBadge>
            </div>
            <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ game.desc }}</p>
          </div>
          <AppButton variant="secondary" size="sm">Play →</AppButton>
        </div>
      </AppCard>
    </div>
  </div>
</template>
