<script setup lang="ts">
defineProps<{
  message: string
  topicName?: string
}>()

defineEmits<{ dismiss: [] }>()
</script>

<template>
  <div class="px-4 py-3 rounded-[var(--radius-lg)] flex items-start gap-3"
    :style="{ backgroundColor: 'var(--gold-light)' }">
    <span class="text-lg shrink-0 mt-0.5">🛟</span>
    <div class="flex-1">
      <p v-if="topicName" class="text-xs font-semibold mb-0.5" :style="{ color: 'var(--gold)' }">{{ topicName }}</p>
      <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ message }}</p>
    </div>
    <button class="text-xs shrink-0" :style="{ color: 'var(--text-3)' }" @click="$emit('dismiss')">✕</button>
  </div>
</template>
