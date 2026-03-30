<script setup lang="ts">
defineProps<{
  state: string
}>()

const stateConfig: Record<string, { label: string; icon: string; color: string }> = {
  calm_guide: { label: 'Calm Guide', icon: '🧭', color: 'var(--accent)' },
  teacher: { label: 'Teacher Mode', icon: '📖', color: 'var(--gold)' },
  rescue: { label: 'Rescue Mode', icon: '🛟', color: 'var(--warm)' },
  confidence_repair: { label: 'Confidence Repair', icon: '💪', color: 'var(--success)' },
  performance_coach: { label: 'Performance Coach', icon: '🏆', color: 'var(--accent)' },
  accountability: { label: 'Accountability', icon: '📋', color: 'var(--text-2)' },
}
</script>

<template>
  <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-medium"
    :style="{
      backgroundColor: (stateConfig[state]?.color ?? 'var(--text-3)') + '12',
      color: stateConfig[state]?.color ?? 'var(--text-3)',
    }">
    <span>{{ stateConfig[state]?.icon ?? '◉' }}</span>
    {{ stateConfig[state]?.label ?? state }}
  </span>
</template>
