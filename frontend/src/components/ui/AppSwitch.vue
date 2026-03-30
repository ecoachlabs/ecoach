<script setup lang="ts">
defineProps<{
  modelValue: boolean
  label?: string
  size?: 'sm' | 'md'
}>()

defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <label class="inline-flex items-center gap-2.5 cursor-pointer select-none">
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      class="relative inline-flex shrink-0 rounded-full transition-colors"
      :class="[
        size === 'sm' ? 'h-5 w-9' : 'h-6 w-11',
        modelValue ? 'bg-[var(--accent)]' : 'bg-[var(--border-strong)]',
      ]"
      :style="{ transitionDuration: 'var(--dur-normal)', transitionTimingFunction: 'var(--ease-spring)' }"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <span
        class="pointer-events-none block rounded-full bg-white shadow-sm transition-transform"
        :class="[
          size === 'sm' ? 'h-4 w-4 mt-0.5 ml-0.5' : 'h-5 w-5 mt-0.5 ml-0.5',
          modelValue ? (size === 'sm' ? 'translate-x-4' : 'translate-x-5') : 'translate-x-0',
        ]"
        :style="{ transitionDuration: 'var(--dur-normal)', transitionTimingFunction: 'var(--ease-spring)' }"
      />
    </button>
    <span v-if="label" class="text-sm" :style="{ color: 'var(--text-2)' }">{{ label }}</span>
  </label>
</template>
