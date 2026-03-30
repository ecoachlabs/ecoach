<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  currentTier: string
  nextTier: string
  progressToNext: number
  eps: number
}>()

const tiers = [
  { key: 'foundation', label: 'Foundation', color: '#78716c', icon: '○' },
  { key: 'core', label: 'Core', color: '#0d9488', icon: '◔' },
  { key: 'prime', label: 'Prime', color: '#2563eb', icon: '◑' },
  { key: 'apex', label: 'Apex', color: '#7c3aed', icon: '◕' },
  { key: 'master', label: 'Master', color: '#b45309', icon: '●' },
  { key: 'legend', label: 'Legend', color: '#dc2626', icon: '★' },
]
</script>

<template>
  <div>
    <div class="flex items-center gap-1 mb-3">
      <div v-for="(t, i) in tiers" :key="t.key"
        class="flex-1 text-center py-1.5 text-[8px] font-bold uppercase rounded-[var(--radius-sm)] transition-all"
        :style="{
          backgroundColor: t.key === currentTier ? t.color : t.color + '10',
          color: t.key === currentTier ? 'white' : t.color + '60',
        }">
        {{ t.icon }} {{ t.label }}
      </div>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-[10px] font-semibold" :style="{color:'var(--text-3)'}">{{ currentTier }} → {{ nextTier }}</span>
      <AppProgress :value="progressToNext" size="sm" color="accent" class="flex-1" :glow="true" />
      <span class="text-[10px] tabular-nums font-medium" :style="{color:'var(--accent)'}">{{ progressToNext }}%</span>
    </div>
  </div>
</template>
