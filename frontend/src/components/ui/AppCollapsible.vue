<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  title: string
  defaultOpen?: boolean
}>()

const open = ref(false)
</script>

<template>
  <div class="rounded-[var(--radius-md)] overflow-hidden">
    <button
      class="w-full flex items-center justify-between px-4 py-3 text-sm font-medium transition-colors"
      :style="{ backgroundColor: open ? 'var(--primary-light)' : 'var(--card-bg)', color: 'var(--text)' }"
      @click="open = !open"
    >
      <span>{{ title }}</span>
      <span class="text-xs transition-transform" :class="open ? 'rotate-180' : ''" :style="{ color: 'var(--text-3)' }">▾</span>
    </button>
    <div v-if="open" class="px-4 py-3 border-t" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }">
      <slot />
    </div>
  </div>
</template>
