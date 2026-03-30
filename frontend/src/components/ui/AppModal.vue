<script setup lang="ts">
defineProps<{
  open: boolean
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
}>()

defineEmits<{ close: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open"
        class="fixed inset-0 flex items-center justify-center p-4"
        :style="{ zIndex: 'var(--z-modal)' }"
        @click.self="$emit('close')"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" />

        <!-- Panel -->
        <div
          class="relative w-full rounded-[var(--radius-xl)] overflow-hidden"
          :class="[
            size === 'sm' ? 'max-w-sm' :
            size === 'lg' ? 'max-w-2xl' :
            size === 'xl' ? 'max-w-4xl' :
            'max-w-lg',
          ]"
          :style="{
            backgroundColor: 'var(--card-bg)',
            boxShadow: 'var(--shadow-xl)',
          }"
        >
          <!-- Header -->
          <div v-if="title" class="flex items-center justify-between px-6 pt-5 pb-0">
            <h2 class="font-display text-lg font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h2>
            <button
              class="w-8 h-8 rounded-lg flex items-center justify-center text-[var(--text-3)] hover:bg-[var(--primary-light)] hover:text-[var(--text)] transition-colors"
              @click="$emit('close')"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M12 4L4 12M4 4l8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
            </button>
          </div>

          <!-- Body -->
          <div class="px-6 py-5">
            <slot />
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="px-6 pb-5 flex items-center justify-end gap-2">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active { transition: all var(--dur-slow) var(--ease-out); }
.modal-leave-active { transition: all var(--dur-normal) var(--ease-smooth); }
.modal-enter-from, .modal-leave-to {
  opacity: 0;
}
.modal-enter-from > div:last-child {
  transform: scale(0.95) translateY(8px);
}
</style>
