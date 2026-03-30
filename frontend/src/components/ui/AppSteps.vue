<script setup lang="ts">
defineProps<{
  steps: { label: string; description?: string }[]
  currentStep: number
}>()
</script>

<template>
  <div class="flex items-center gap-2">
    <template v-for="(step, i) in steps" :key="i">
      <div class="flex items-center gap-2">
        <div class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold shrink-0 transition-colors"
          :class="i < currentStep ? 'bg-[var(--accent)] text-white' : i === currentStep ? 'ring-2 ring-[var(--accent)] text-[var(--accent)]' : ''"
          :style="i >= currentStep && i !== currentStep ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : i === currentStep ? { backgroundColor: 'var(--accent-light)' } : {}">
          {{ i < currentStep ? '✓' : i + 1 }}
        </div>
        <span class="text-xs font-medium hidden sm:inline" :style="{ color: i <= currentStep ? 'var(--text)' : 'var(--text-3)' }">
          {{ step.label }}
        </span>
      </div>
      <div v-if="i < steps.length - 1" class="flex-1 h-px min-w-[16px]"
        :style="{ backgroundColor: i < currentStep ? 'var(--accent)' : 'var(--border-soft)' }" />
    </template>
  </div>
</template>
