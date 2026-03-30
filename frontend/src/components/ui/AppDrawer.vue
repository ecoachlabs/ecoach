<script setup lang="ts">
defineProps<{ open: boolean; title?: string; side?: 'left' | 'right'; width?: string }>()
defineEmits<{ close: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition name="drawer">
      <div v-if="open" class="fixed inset-0" :style="{ zIndex: 'var(--z-modal)' }">
        <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="$emit('close')" />
        <div class="absolute top-0 bottom-0 flex flex-col overflow-hidden"
          :class="side === 'left' ? 'left-0' : 'right-0'"
          :style="{ width: width || '400px', backgroundColor: 'var(--card-bg)', boxShadow: 'var(--shadow-xl)' }">
          <div v-if="title" class="shrink-0 flex items-center justify-between px-5 py-4 border-b" :style="{ borderColor: 'var(--card-border)' }">
            <h3 class="font-display text-base font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h3>
            <button class="text-[var(--text-3)] hover:text-[var(--text)]" @click="$emit('close')">✕</button>
          </div>
          <div class="flex-1 overflow-y-auto p-5"><slot /></div>
          <div v-if="$slots.footer" class="shrink-0 px-5 py-3 border-t" :style="{ borderColor: 'var(--card-border)' }">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.drawer-enter-active, .drawer-leave-active { transition: all var(--dur-slow) var(--ease-out); }
.drawer-enter-from > div:last-child, .drawer-leave-to > div:last-child { transform: translateX(100%); }
.drawer-enter-from, .drawer-leave-to { opacity: 0; }
</style>
