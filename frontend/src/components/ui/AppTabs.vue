<script setup lang="ts">
defineProps<{
  tabs: { key: string; label: string; count?: number }[]
  modelValue: string
}>()

defineEmits<{
  'update:modelValue': [key: string]
}>()
</script>

<template>
  <div class="flex gap-1 p-1 rounded-[var(--radius-lg)]" :style="{ backgroundColor: 'var(--primary-light)' }">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      class="relative px-4 py-2 text-sm font-medium rounded-[var(--radius-md)] transition-all"
      :class="modelValue === tab.key
        ? 'bg-[var(--card-bg)] text-[var(--text)] shadow-sm'
        : 'text-[var(--text-3)] hover:text-[var(--text-2)]'"
      :style="{ transitionDuration: 'var(--dur-normal)', transitionTimingFunction: 'var(--ease-spring)' }"
      @click="$emit('update:modelValue', tab.key)"
    >
      {{ tab.label }}
      <span
        v-if="tab.count !== undefined"
        class="ml-1.5 text-[10px] px-1.5 py-0.5 rounded-full"
        :class="modelValue === tab.key ? 'bg-[var(--primary)] text-white' : 'bg-[var(--card-border)] text-[var(--text-3)]'"
      >
        {{ tab.count }}
      </span>
    </button>
  </div>
</template>
