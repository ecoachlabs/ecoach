<script setup lang="ts">
defineProps<{
  fromBlock: string
  toBlock: string
  message?: string
  visible: boolean
}>()

defineEmits<{ continue: [] }>()
</script>

<template>
  <Transition name="block-transition">
    <div v-if="visible" class="flex flex-col items-center justify-center py-16 text-center reveal">
      <div class="w-12 h-12 rounded-2xl mx-auto mb-4 flex items-center justify-center text-xl"
        :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">→</div>
      <p class="text-sm font-medium mb-1" :style="{ color: 'var(--text)' }">{{ message || 'Next block' }}</p>
      <p class="text-xs mb-4" :style="{ color: 'var(--text-3)' }">Moving from {{ fromBlock }} to {{ toBlock }}</p>
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
        :style="{ backgroundColor: 'var(--accent)' }" @click="$emit('continue')">Continue →</button>
    </div>
  </Transition>
</template>

<style scoped>
.block-transition-enter-active { transition: all var(--dur-slow) var(--ease-out); }
.block-transition-leave-active { transition: all var(--dur-normal); }
.block-transition-enter-from { opacity: 0; transform: translateY(10px); }
.block-transition-leave-to { opacity: 0; }
</style>
