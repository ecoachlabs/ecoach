<script setup lang="ts">
import MathText from '../MathText.vue'

defineProps<{
  selected: number | null
  answered: boolean
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
}>()

defineEmits<{ select: [id: number] }>()
</script>

<template>
  <div class="grid grid-cols-2 gap-3">
    <button
      v-for="opt in options"
      :key="opt.id"
      class="py-6 rounded-[var(--radius-lg)] border text-center font-semibold text-lg transition-all"
      :class="[
        answered && opt.is_correct ? 'border-emerald-500 bg-emerald-50 text-emerald-700' :
        answered && selected === opt.id && !opt.is_correct ? 'border-red-400 bg-red-50 text-red-600' :
        selected === opt.id ? 'border-[var(--accent)] bg-[var(--accent-light)] text-[var(--accent)]' :
        'border-[var(--card-border)] hover:border-[var(--accent)] text-[var(--text-2)]',
      ]"
      :style="{ backgroundColor: (!answered && selected !== opt.id) ? 'var(--card-bg)' : undefined, transitionDuration: 'var(--dur-fast)' }"
      :disabled="answered"
      @click="!answered && $emit('select', opt.id)"
    >
      <MathText :text="opt.text" />
    </button>
  </div>
</template>
