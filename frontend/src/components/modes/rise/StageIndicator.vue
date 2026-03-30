<script setup lang="ts">
defineProps<{
  currentStage: 'rescue' | 'stabilize' | 'accelerate' | 'dominate' | string
  progress?: number
}>()

const stages = [
  { key: 'rescue', label: 'Rescue', icon: '🛟', color: '#dc2626' },
  { key: 'stabilize', label: 'Stabilize', icon: '⚖', color: '#d97706' },
  { key: 'accelerate', label: 'Accelerate', icon: '🚀', color: '#0d9488' },
  { key: 'dominate', label: 'Dominate', icon: '👑', color: '#b45309' },
]
</script>

<template>
  <div class="flex items-center gap-1">
    <template v-for="(stage, i) in stages" :key="stage.key">
      <div
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
        :class="currentStage === stage.key ? 'text-white shadow-sm' : ''"
        :style="{
          backgroundColor: currentStage === stage.key ? stage.color :
            stages.findIndex(s => s.key === currentStage) > i ? stage.color + '20' : 'var(--primary-light)',
          color: currentStage === stage.key ? 'white' :
            stages.findIndex(s => s.key === currentStage) > i ? stage.color : 'var(--text-3)',
        }"
      >
        <span>{{ stage.icon }}</span>
        <span>{{ stage.label }}</span>
      </div>
      <div v-if="i < stages.length - 1" class="w-4 h-px" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </template>
  </div>
</template>
