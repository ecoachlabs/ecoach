<script setup lang="ts">
defineProps<{
  type?: 'success' | 'error' | 'warning' | 'info'
  message: string
  visible: boolean
}>()

defineEmits<{ close: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition name="toast">
      <div
        v-if="visible"
        class="fixed top-4 right-4 max-w-sm px-4 py-3 rounded-[var(--radius-lg)] shadow-lg flex items-center gap-3 cursor-pointer"
        :style="{ zIndex: 'var(--z-toast)' }"
        :class="{
          'bg-emerald-600 text-white': type === 'success',
          'bg-red-600 text-white': type === 'error',
          'bg-amber-500 text-white': type === 'warning',
          'bg-[var(--accent)] text-white': type === 'info' || !type,
        }"
        @click="$emit('close')"
      >
        <span class="text-lg">
          {{ type === 'success' ? '✓' : type === 'error' ? '✕' : type === 'warning' ? '⚠' : 'ℹ' }}
        </span>
        <p class="text-sm font-medium flex-1">{{ message }}</p>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.toast-enter-active { transition: all var(--dur-slow) var(--ease-spring); }
.toast-leave-active { transition: all var(--dur-normal) var(--ease-smooth); }
.toast-enter-from { opacity: 0; transform: translateX(20px) scale(0.95); }
.toast-leave-to { opacity: 0; transform: translateX(20px); }
</style>
