<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  /** -100 (opponent wins) to +100 (student wins) */
  position: number
}>()

const leftPct = computed(() => 50 + props.position / 2)
const status = computed(() => {
  if (props.position > 20) return { label: 'Winning!', color: 'var(--success)' }
  if (props.position < -20) return { label: 'Losing', color: 'var(--danger)' }
  return { label: 'Contested', color: 'var(--gold)' }
})
</script>

<template>
  <div>
    <!-- Rope visualization -->
    <div class="relative h-8 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
      <!-- Zones -->
      <div class="absolute inset-0 flex">
        <div class="w-[20%] bg-emerald-100 opacity-50" />
        <div class="w-[15%] bg-emerald-50 opacity-30" />
        <div class="w-[30%]" />
        <div class="w-[15%] bg-red-50 opacity-30" />
        <div class="w-[20%] bg-red-100 opacity-50" />
      </div>
      <!-- Position marker -->
      <div class="absolute top-0 bottom-0 w-4 rounded-full transition-all shadow-md"
        :style="{ left: `calc(${leftPct}% - 8px)`, backgroundColor: status.color, transitionDuration: 'var(--dur-slow)', transitionTimingFunction: 'var(--ease-spring)' }" />
      <!-- Center line -->
      <div class="absolute top-0 bottom-0 left-1/2 w-px bg-[var(--border-strong)]" />
    </div>
    <!-- Status -->
    <div class="flex items-center justify-between mt-1.5 text-[10px] font-semibold">
      <span class="text-emerald-600">You</span>
      <span :style="{ color: status.color }">{{ status.label }}</span>
      <span class="text-red-500">Opponent</span>
    </div>
  </div>
</template>
