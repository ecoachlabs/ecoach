<script setup lang="ts">
defineProps<{ label?: string; options: { value: string | number; label: string }[]; modelValue?: string | number; placeholder?: string }>()
defineEmits<{ 'update:modelValue': [value: string | number] }>()
</script>

<template>
  <div class="space-y-1.5">
    <label v-if="label" class="block text-xs font-medium tracking-wide uppercase" :style="{ color: 'var(--text-3)' }">{{ label }}</label>
    <select :value="modelValue"
      class="w-full px-3.5 py-2.5 rounded-[var(--radius-md)] border text-sm appearance-none bg-no-repeat"
      :style="{ backgroundColor: 'var(--card-bg)', borderColor: 'var(--card-border)', color: 'var(--text)', backgroundImage: `url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath d='M3 5l3 3 3-3' stroke='%239e958d' fill='none' stroke-width='1.5'/%3E%3C/svg%3E\")`, backgroundPosition: 'right 12px center' }"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)">
      <option v-if="placeholder" value="" disabled>{{ placeholder }}</option>
      <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
    </select>
  </div>
</template>
