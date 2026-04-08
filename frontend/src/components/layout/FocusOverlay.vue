<script setup lang="ts">
defineProps<{ active: boolean }>()
defineEmits<{ exit: [] }>()
</script>

<template>
  <Transition name="focus">
    <div v-if="active" class="fixed inset-0 flex flex-col" :style="{zIndex:'var(--z-overlay)',backgroundColor:'var(--bg)'}" data-mode="focus">
      <div class="shrink-0 flex items-center justify-between px-6 py-3" :style="{backgroundColor:'var(--card-bg)'}">
        <span class="text-xs font-medium" :style="{color:'var(--text-3)'}">Focus Mode</span>
        <button class="text-xs px-3 py-1 rounded-[var(--radius-md)]" :style="{color:'var(--text-3)'}" @click="$emit('exit')">Exit Focus</button>
      </div>
      <div class="flex-1 overflow-y-auto"><slot /></div>
    </div>
  </Transition>
</template>

<style scoped>
.focus-enter-active { transition: all var(--dur-ceremonial) var(--ease-out); }
.focus-leave-active { transition: all var(--dur-slow); }
.focus-enter-from, .focus-leave-to { opacity: 0; }
</style>
