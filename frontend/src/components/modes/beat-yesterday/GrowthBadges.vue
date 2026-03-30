<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  streakDays: number
  badges: { name: string; icon: string; earned: boolean; requirement: string }[]
}>()
</script>

<template>
  <div>
    <div class="flex items-center gap-3 mb-4">
      <div class="text-3xl">🔥</div>
      <div>
        <p class="font-display text-2xl font-bold" :style="{color:'var(--gold)'}">{{ streakDays }}</p>
        <p class="text-[10px] font-semibold uppercase" :style="{color:'var(--text-3)'}">Day Streak</p>
      </div>
    </div>

    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Growth Badges</h3>
    <div class="grid grid-cols-4 gap-3">
      <div v-for="b in badges" :key="b.name" class="text-center" :class="!b.earned ? 'opacity-30' : ''">
        <div class="w-14 h-14 rounded-2xl mx-auto mb-1.5 flex items-center justify-center text-2xl"
          :style="{backgroundColor: b.earned ? 'var(--gold-light)' : 'var(--primary-light)'}">
          {{ b.earned ? b.icon : '🔒' }}
        </div>
        <p class="text-[9px] font-medium" :style="{color:'var(--text)'}">{{ b.name }}</p>
        <p class="text-[8px]" :style="{color:'var(--text-3)'}">{{ b.requirement }}</p>
      </div>
    </div>
  </div>
</template>
