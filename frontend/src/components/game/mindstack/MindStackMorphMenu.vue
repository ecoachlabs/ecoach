<script setup lang="ts">
defineProps<{
  visible: boolean
  options: { shape: string; icon: string }[]
  timeLimit: number
}>()

defineEmits<{ select: [shape: string]; timeout: [] }>()
</script>

<template>
  <Transition name="morph">
    <div v-if="visible" class="fixed inset-0 flex items-center justify-center" :style="{ zIndex: 'var(--z-overlay)' }">
      <div class="absolute inset-0 bg-black/30" />
      <div class="relative flex gap-3">
        <button v-for="opt in options" :key="opt.shape"
          class="w-16 h-16 rounded-2xl border-2 flex flex-col items-center justify-center transition-all active:scale-90 hover:scale-110"
          :style="{ backgroundColor: 'var(--card-bg)', borderColor: 'var(--primary)', boxShadow: 'var(--shadow-glow-accent)' }"
          @click="$emit('select', opt.shape)">
          <span class="text-2xl">{{ opt.icon }}</span>
          <span class="text-[8px] font-bold" :style="{ color: 'var(--primary)' }">{{ opt.shape }}</span>
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.morph-enter-active { transition: all 200ms cubic-bezier(0.34,1.56,0.64,1); }
.morph-leave-active { transition: all 100ms; }
.morph-enter-from { opacity: 0; transform: scale(0.7); }
.morph-leave-to { opacity: 0; transform: scale(0.9); }
</style>
