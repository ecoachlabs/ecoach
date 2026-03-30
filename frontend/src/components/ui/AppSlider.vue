<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  modelValue: number
  min?: number
  max?: number
  step?: number
  label?: string
  showValue?: boolean
  formatValue?: (v: number) => string
}>()

defineEmits<{ 'update:modelValue': [value: number] }>()

const displayValue = computed(() =>
  props.formatValue ? props.formatValue(props.modelValue) : String(props.modelValue)
)
</script>

<template>
  <div>
    <div v-if="label || showValue" class="flex items-center justify-between mb-1.5">
      <label v-if="label" class="text-xs font-medium uppercase tracking-wide" :style="{ color: 'var(--text-3)' }">{{ label }}</label>
      <span v-if="showValue" class="text-xs font-semibold tabular-nums" :style="{ color: 'var(--accent)' }">{{ displayValue }}</span>
    </div>
    <input type="range" :value="modelValue" :min="min ?? 0" :max="max ?? 100" :step="step ?? 1"
      class="w-full h-1.5 rounded-full appearance-none cursor-pointer accent-[var(--accent)]"
      :style="{ backgroundColor: 'var(--border-soft)' }"
      @input="$emit('update:modelValue', Number(($event.target as HTMLInputElement).value))" />
  </div>
</template>
