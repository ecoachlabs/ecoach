<script setup lang="ts">
import { ref, onMounted } from 'vue'

defineProps<{ active: boolean; title?: string }>()
defineEmits<{ exit: [] }>()

const entered = ref(false)

onMounted(() => {
  // Ceremonial 3-second entrance
  setTimeout(() => { entered.value = true }, 3000)
})
</script>

<template>
  <Transition name="exam-hall">
    <div v-if="active" class="fixed inset-0 flex flex-col" :style="{zIndex:'var(--z-overlay)'}" data-mode="pressure">
      <!-- Ceremonial entrance -->
      <Transition name="ceremony">
        <div v-if="!entered" class="absolute inset-0 flex items-center justify-center" :style="{backgroundColor:'#0c0a09',zIndex:10}">
          <div class="text-center reveal">
            <div class="w-20 h-20 rounded-3xl mx-auto mb-4 flex items-center justify-center text-4xl" style="background:rgba(239,68,68,0.1)">⊞</div>
            <h2 class="font-display text-2xl font-bold text-white mb-2">Entering Exam Hall</h2>
            <p class="text-sm text-stone-400">Focus. No distractions. You are ready.</p>
          </div>
        </div>
      </Transition>

      <!-- Actual exam content -->
      <div v-if="entered" class="flex-1 flex flex-col" :style="{backgroundColor:'var(--bg)'}">
        <slot />
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.exam-hall-enter-active { transition: all var(--dur-ceremonial) var(--ease-out); }
.exam-hall-leave-active { transition: all var(--dur-slow); }
.exam-hall-enter-from { opacity: 0; }
.exam-hall-leave-to { opacity: 0; }
.ceremony-leave-active { transition: opacity 800ms ease; }
.ceremony-leave-to { opacity: 0; }
</style>
