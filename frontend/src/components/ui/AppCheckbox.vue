<script setup lang="ts">
defineProps<{
  modelValue: boolean
  label?: string
  description?: string
}>()

defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <label class="flex items-start gap-2.5 cursor-pointer group">
    <span class="mt-0.5 w-4 h-4 rounded border-2 flex items-center justify-center shrink-0 transition-colors"
      :class="modelValue ? 'bg-[var(--accent)] border-[var(--accent)]' : 'border-[var(--border-strong)] group-hover:border-[var(--accent)]'"
      @click="$emit('update:modelValue', !modelValue)">
      <svg v-if="modelValue" width="10" height="10" viewBox="0 0 10 10" fill="none">
        <path d="M2 5l2.5 2.5L8 3" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </span>
    <div @click="$emit('update:modelValue', !modelValue)">
      <p v-if="label" class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ label }}</p>
      <p v-if="description" class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ description }}</p>
    </div>
  </label>
</template>
