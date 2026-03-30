<script setup lang="ts">
defineProps<{ label?: string; placeholder?: string; error?: string; modelValue?: string; rows?: number }>()
defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="space-y-1.5">
    <label v-if="label" class="block text-xs font-medium tracking-wide uppercase" :style="{ color: 'var(--text-3)' }">{{ label }}</label>
    <textarea :rows="rows || 4" :placeholder="placeholder" :value="modelValue"
      class="w-full px-3.5 py-2.5 rounded-[var(--radius-md)] border text-sm resize-y transition-colors placeholder:text-[var(--text-3)]"
      :class="error ? 'border-[var(--danger)]' : 'border-[var(--card-border)] focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]'"
      :style="{ backgroundColor: 'var(--card-bg)', color: 'var(--text)' }"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)" />
    <p v-if="error" class="text-xs text-[var(--danger)]">{{ error }}</p>
  </div>
</template>
