<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

defineProps<{
  align?: 'left' | 'right'
}>()

const open = ref(false)
const el = ref<HTMLElement | null>(null)

function toggle() { open.value = !open.value }
function close() { open.value = false }

function handleClickOutside(e: MouseEvent) {
  if (el.value && !el.value.contains(e.target as Node)) close()
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => document.removeEventListener('click', handleClickOutside))
</script>

<template>
  <div ref="el" class="relative inline-block">
    <div @click="toggle">
      <slot name="trigger" />
    </div>
    <Transition name="dropdown">
      <div
        v-if="open"
        class="absolute mt-1 min-w-[180px] py-1 rounded-[var(--radius-md)] overflow-hidden"
        :class="align === 'right' ? 'right-0' : 'left-0'"
        :style="{
          backgroundColor: 'var(--card-bg)',
          boxShadow: 'var(--shadow-lg)',
          zIndex: 'var(--z-dropdown)',
        }"
        @click="close"
      >
        <slot />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.dropdown-enter-active { transition: all var(--dur-fast) var(--ease-out); }
.dropdown-leave-active { transition: all 80ms var(--ease-smooth); }
.dropdown-enter-from, .dropdown-leave-to { opacity: 0; transform: translateY(-4px) scale(0.97); }
</style>
