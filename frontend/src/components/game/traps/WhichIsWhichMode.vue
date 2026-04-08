<script setup lang="ts">
import { ref, computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  prompt: string
  conceptA: string
  conceptB: string
  active: boolean
}>()

const emit = defineEmits<{ answer: [choice: 'a' | 'b'] }>()

const streak = ref(0)
const speed = ref(1)
</script>

<template>
  <div class="text-center">
    <!-- Streak & Speed -->
    <div class="flex items-center justify-center gap-4 mb-6">
      <span class="text-xs font-bold tabular-nums" :style="{ color: streak > 0 ? 'var(--gold)' : 'var(--text-3)' }">
        🔥 {{ streak }} streak
      </span>
      <span class="text-xs font-bold tabular-nums" :style="{ color: 'var(--accent)' }">⚡ {{ speed }}x</span>
    </div>

    <!-- Prompt -->
    <div class="p-6 rounded-[var(--radius-xl)] mb-8 max-w-lg mx-auto"
      :style="{ backgroundColor: 'var(--card-bg)' }">
      <p class="text-lg font-medium leading-relaxed" :style="{ color: 'var(--text)' }">{{ prompt }}</p>
    </div>

    <!-- Two big buttons -->
    <div class="grid grid-cols-2 gap-4 max-w-md mx-auto">
      <button
        class="py-8 rounded-[var(--radius-xl)] border-2 text-lg font-bold transition-all active:scale-95"
        :style="{ borderColor: 'var(--accent)', color: 'var(--accent)', backgroundColor: 'var(--accent-light)' }"
        :disabled="!active"
        @click="emit('answer', 'a')">
        {{ conceptA }}
      </button>
      <button
        class="py-8 rounded-[var(--radius-xl)] border-2 text-lg font-bold transition-all active:scale-95"
        :style="{ borderColor: 'var(--gold)', color: 'var(--gold)', backgroundColor: 'var(--gold-light)' }"
        :disabled="!active"
        @click="emit('answer', 'b')">
        {{ conceptB }}
      </button>
    </div>
  </div>
</template>
