<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  text: string
  position?: 'top' | 'bottom' | 'left' | 'right'
}>()

const visible = ref(false)
</script>

<template>
  <div class="relative inline-flex" @mouseenter="visible = true" @mouseleave="visible = false">
    <slot />
    <Transition name="tip">
      <div
        v-if="visible"
        class="absolute px-2.5 py-1.5 text-[11px] font-medium text-white rounded-[var(--radius-sm)] whitespace-nowrap pointer-events-none"
        :class="{
          'bottom-full left-1/2 -translate-x-1/2 mb-1.5': position === 'top' || !position,
          'top-full left-1/2 -translate-x-1/2 mt-1.5': position === 'bottom',
          'right-full top-1/2 -translate-y-1/2 mr-1.5': position === 'left',
          'left-full top-1/2 -translate-y-1/2 ml-1.5': position === 'right',
        }"
        :style="{ backgroundColor: 'var(--ink)', zIndex: 'var(--z-tooltip)' }"
      >
        {{ text }}
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.tip-enter-active { transition: all 100ms var(--ease-out); }
.tip-leave-active { transition: all 60ms; }
.tip-enter-from, .tip-leave-to { opacity: 0; transform: scale(0.95); }
</style>
