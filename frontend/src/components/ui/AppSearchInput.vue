<script setup lang="ts">
defineProps<{
  modelValue: string
  placeholder?: string
  loading?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string]
  search: []
}>()
</script>

<template>
  <div class="relative">
    <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-sm" :style="{ color: 'var(--text-3)' }">⌕</span>
    <input
      :value="modelValue"
      :placeholder="placeholder || 'Search...'"
      class="w-full pl-10 pr-4 py-2.5 rounded-[var(--radius-lg)] border text-sm transition-colors placeholder:text-[var(--text-3)]"
      :style="{
        backgroundColor: 'var(--card-bg)',
        borderColor: 'var(--card-border)',
        color: 'var(--text)',
      }"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keydown.enter="$emit('search')"
    />
    <div v-if="loading" class="absolute right-3 top-1/2 -translate-y-1/2">
      <div class="w-4 h-4 border-2 rounded-full animate-spin" :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
    </div>
  </div>
</template>
