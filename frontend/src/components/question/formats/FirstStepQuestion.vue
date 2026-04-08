<script setup lang="ts">
defineProps<{
  problem: string
  options: { id: number; label: string; text: string; is_correct?: boolean }[]
  selected: number | null
  answered: boolean
}>()

defineEmits<{ select: [id: number] }>()
</script>

<template>
  <div>
    <div class="p-4 rounded-[var(--radius-lg)] mb-4" :style="{ backgroundColor: 'var(--card-bg)' }">
      <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--accent)' }">What is the correct first step?</p>
      <p class="text-sm leading-relaxed" :style="{ color: 'var(--text)' }">{{ problem }}</p>
    </div>

    <div class="space-y-2">
      <button v-for="opt in options" :key="opt.id"
        class="w-full flex items-start gap-3 p-3.5 rounded-[var(--radius-md)] border text-left text-sm transition-all"
        :class="[
          answered && opt.is_correct ? 'border-emerald-500 bg-emerald-50' :
          answered && selected === opt.id && !opt.is_correct ? 'border-red-400 bg-red-50' :
          selected === opt.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' :
          'border-[var(--card-border)] hover:border-[var(--accent)]',
        ]"
        :style="{ backgroundColor: (!answered && selected !== opt.id) ? 'var(--card-bg)' : undefined }"
        :disabled="answered"
        @click="!answered && $emit('select', opt.id)">
        <span class="w-7 h-7 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0"
          :class="answered && opt.is_correct ? 'bg-emerald-500 text-white' : answered && selected === opt.id ? 'bg-red-400 text-white' : selected === opt.id ? 'bg-[var(--accent)] text-white' : ''"
          :style="(!answered && selected !== opt.id) ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
          {{ opt.label }}
        </span>
        <span :style="{ color: 'var(--text)' }">{{ opt.text }}</span>
      </button>
    </div>
  </div>
</template>
