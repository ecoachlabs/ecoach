<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  score: number
  streak: number
  level: number
  controlStatus: 'locked' | 'move' | 'rotate' | 'morph' | 'panic'
  powerUps: { type: string; count: number }[]
}>()

const controlLabels: Record<string, { label: string; color: string; icon: string }> = {
  locked: { label: 'LOCKED', color: '#dc2626', icon: '🔒' },
  move: { label: 'MOVE', color: '#0d9488', icon: '←→' },
  rotate: { label: 'ROTATE', color: '#2563eb', icon: '↻' },
  morph: { label: 'MORPH', color: '#7c3aed', icon: '◆' },
  panic: { label: 'PANIC', color: '#dc2626', icon: '⚠' },
}
</script>

<template>
  <div class="flex items-center justify-between px-3 py-2 rounded-[var(--radius-md)] border"
    :style="{ backgroundColor: 'var(--card-bg)', borderColor: 'var(--card-border)' }">
    <!-- Score -->
    <div class="text-center">
      <p class="font-display text-xl font-bold tabular-nums" :style="{ color: 'var(--text)' }">{{ score.toLocaleString() }}</p>
      <p class="text-[8px] uppercase font-semibold" :style="{ color: 'var(--text-3)' }">Score</p>
    </div>

    <!-- Streak -->
    <div class="text-center">
      <p class="font-display text-lg font-bold tabular-nums" :style="{ color: streak > 0 ? 'var(--gold)' : 'var(--text-3)' }">
        {{ streak > 0 ? '🔥' : '' }}{{ streak }}
      </p>
      <p class="text-[8px] uppercase font-semibold" :style="{ color: 'var(--text-3)' }">Streak</p>
    </div>

    <!-- Control Status -->
    <div class="text-center">
      <AppBadge size="sm"
        :style="{ backgroundColor: controlLabels[controlStatus]?.color + '20', color: controlLabels[controlStatus]?.color }">
        {{ controlLabels[controlStatus]?.icon }} {{ controlLabels[controlStatus]?.label }}
      </AppBadge>
    </div>

    <!-- Level -->
    <div class="text-center">
      <p class="text-sm font-bold tabular-nums" :style="{ color: 'var(--accent)' }">L{{ level }}</p>
      <p class="text-[8px] uppercase font-semibold" :style="{ color: 'var(--text-3)' }">Level</p>
    </div>

    <!-- Power-ups -->
    <div class="flex gap-1">
      <div v-for="pu in powerUps.filter(p => p.count > 0)" :key="pu.type"
        class="w-7 h-7 rounded flex items-center justify-center text-[10px] relative"
        :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text-2)' }">
        ⚡
        <span class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-[var(--accent)] text-white text-[8px] font-bold flex items-center justify-center">
          {{ pu.count }}
        </span>
      </div>
    </div>
  </div>
</template>
