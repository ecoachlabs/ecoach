<script setup lang="ts">
defineProps<{
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
  selected: number | null
  answered: boolean
}>()

defineEmits<{ select: [id: number] }>()

const optionLetters = ['A', 'B', 'C', 'D', 'E', 'F']
</script>

<template>
  <div class="space-y-2">
    <button
      v-for="(opt, i) in options"
      :key="opt.id"
      class="w-full flex items-start gap-3 p-4 rounded-[var(--radius-lg)] border text-left transition-all"
      :class="[
        answered && opt.is_correct ? 'border-emerald-500 bg-emerald-50' :
        answered && selected === opt.id && !opt.is_correct ? 'border-red-400 bg-red-50' :
        selected === opt.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' :
        'border-[var(--card-border)] hover:border-[var(--accent)] hover:bg-[var(--primary-light)]',
      ]"
      :style="{
        backgroundColor: (!answered && selected !== opt.id) ? 'var(--card-bg)' : undefined,
        transitionDuration: 'var(--dur-fast)',
        transitionTimingFunction: 'var(--ease-spring)',
      }"
      :disabled="answered"
      @click="!answered && $emit('select', opt.id)"
    >
      <!-- Letter badge -->
      <span
        class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold shrink-0 transition-colors"
        :class="[
          answered && opt.is_correct ? 'bg-emerald-500 text-white' :
          answered && selected === opt.id && !opt.is_correct ? 'bg-red-400 text-white' :
          selected === opt.id ? 'bg-[var(--accent)] text-white' :
          'bg-[var(--primary-light)] text-[var(--text-3)]',
        ]"
      >
        {{ opt.label || optionLetters[i] || String(i + 1) }}
      </span>

      <!-- Option text -->
      <span class="text-sm leading-relaxed pt-1" :style="{ color: 'var(--text)' }">
        {{ opt.text }}
      </span>

      <!-- Result indicator -->
      <span v-if="answered && opt.is_correct" class="ml-auto text-emerald-600 text-sm shrink-0 pt-1">✓</span>
      <span v-if="answered && selected === opt.id && !opt.is_correct" class="ml-auto text-red-500 text-sm shrink-0 pt-1">✕</span>
    </button>
  </div>
</template>
