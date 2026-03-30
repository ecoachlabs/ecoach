<script setup lang="ts">
defineProps<{
  options: { value: string | number; label: string; description?: string }[]
  modelValue: string | number | null
  direction?: 'vertical' | 'horizontal'
}>()

defineEmits<{ 'update:modelValue': [value: string | number] }>()
</script>

<template>
  <div :class="direction === 'horizontal' ? 'flex flex-wrap gap-3' : 'space-y-2'">
    <label v-for="opt in options" :key="opt.value"
      class="flex items-start gap-2.5 cursor-pointer group">
      <span class="mt-0.5 w-4 h-4 rounded-full border-2 flex items-center justify-center shrink-0 transition-colors"
        :class="modelValue === opt.value ? 'border-[var(--accent)]' : 'border-[var(--border-strong)] group-hover:border-[var(--accent)]'">
        <span v-if="modelValue === opt.value" class="w-2 h-2 rounded-full bg-[var(--accent)]" />
      </span>
      <div @click="$emit('update:modelValue', opt.value)">
        <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ opt.label }}</p>
        <p v-if="opt.description" class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ opt.description }}</p>
      </div>
    </label>
  </div>
</template>
