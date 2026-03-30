<script setup lang="ts">
defineProps<{
  currentPhase: string
  phases?: string[]
}>()

const defaultPhases = ['Stabilize', 'Build', 'Strengthen', 'Condition', 'Ready']
</script>

<template>
  <div class="flex gap-1">
    <div
      v-for="phase in (phases || defaultPhases)"
      :key="phase"
      class="flex-1 py-2 text-center text-[10px] font-semibold rounded-[var(--radius-sm)] transition-all"
      :class="phase === currentPhase ? 'text-white' : ''"
      :style="{
        backgroundColor: phase === currentPhase ? 'var(--accent)' : 'var(--primary-light)',
        color: phase === currentPhase ? undefined : 'var(--text-3)',
        transitionDuration: 'var(--dur-normal)',
      }"
    >
      {{ phase }}
    </div>
  </div>
</template>
