<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  records: { category: string; value: string; date: string; isPersonalBest: boolean }[]
  badges: { name: string; icon: string; earned: boolean; description: string }[]
  titles: { title: string; earned: boolean }[]
}>()
</script>

<template>
  <div class="space-y-6">
    <!-- Personal Bests -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Personal Records</h3>
      <div class="grid grid-cols-3 gap-2">
        <AppCard v-for="r in records" :key="r.category" padding="sm" class="text-center" :class="r.isPersonalBest ? 'ring-1 ring-[var(--gold)]' : ''">
          <p class="font-display text-lg font-bold" :style="{color: r.isPersonalBest ? 'var(--gold)' : 'var(--text)'}">{{ r.value }}</p>
          <p class="text-[9px] uppercase" :style="{color:'var(--text-3)'}">{{ r.category }}</p>
          <p class="text-[8px]" :style="{color:'var(--text-3)'}">{{ r.date }}</p>
        </AppCard>
      </div>
    </div>

    <!-- Badges -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Badges</h3>
      <div class="flex flex-wrap gap-2">
        <div v-for="b in badges" :key="b.name" class="text-center" :class="!b.earned ? 'opacity-30' : ''">
          <div class="w-12 h-12 rounded-xl flex items-center justify-center text-2xl mx-auto mb-1"
            :style="{backgroundColor: b.earned ? 'var(--gold-light)' : 'var(--primary-light)'}">{{ b.icon }}</div>
          <p class="text-[8px] font-medium" :style="{color:'var(--text-3)'}">{{ b.name }}</p>
        </div>
      </div>
    </div>

    <!-- Titles -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Titles</h3>
      <div class="flex flex-wrap gap-1.5">
        <AppBadge v-for="t in titles" :key="t.title" :color="t.earned ? 'gold' : 'muted'" size="sm">
          {{ t.earned ? '👑' : '🔒' }} {{ t.title }}
        </AppBadge>
      </div>
    </div>
  </div>
</template>
