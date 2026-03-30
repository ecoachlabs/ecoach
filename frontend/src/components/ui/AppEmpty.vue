<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  icon?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-16 px-8 text-center">
    <div
      v-if="icon"
      class="w-14 h-14 rounded-2xl flex items-center justify-center text-2xl mb-4"
      :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }"
    >
      {{ icon }}
    </div>
    <h3 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">{{ title }}</h3>
    <p v-if="description" class="text-sm max-w-xs" :style="{ color: 'var(--text-3)' }">{{ description }}</p>
    <div class="mt-4">
      <slot />
    </div>
  </div>
</template>
