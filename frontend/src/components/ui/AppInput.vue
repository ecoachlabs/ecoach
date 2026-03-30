<script setup lang="ts">
defineProps<{
  label?: string
  placeholder?: string
  type?: string
  error?: string
  modelValue?: string | number
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="space-y-1.5">
    <label v-if="label" class="block text-xs font-medium tracking-wide uppercase" :style="{ color: 'var(--text-3)' }">
      {{ label }}
    </label>
    <input
      :type="type || 'text'"
      :placeholder="placeholder"
      :value="modelValue"
      class="w-full px-3.5 py-2.5 rounded-[var(--radius-md)] border text-sm transition-colors placeholder:text-[var(--text-3)]"
      :class="error ? 'border-[var(--danger)] bg-[var(--danger-light)]' : 'border-[var(--card-border)] bg-[var(--card-bg)] focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]'"
      :style="{ color: 'var(--text)', outlineColor: 'transparent' }"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <p v-if="error" class="text-xs text-[var(--danger)]">{{ error }}</p>
  </div>
</template>
