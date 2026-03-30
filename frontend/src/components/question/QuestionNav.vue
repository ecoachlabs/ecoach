<script setup lang="ts">
defineProps<{
  totalQuestions: number
  currentIndex: number
  answeredIndices: Set<number>
  flaggedIndices: Set<number>
}>()

defineEmits<{ navigate: [index: number] }>()
</script>

<template>
  <div class="flex flex-wrap gap-1">
    <button
      v-for="i in totalQuestions"
      :key="i"
      class="w-8 h-8 rounded text-[10px] font-bold transition-all"
      :class="[
        i - 1 === currentIndex ? 'ring-2 ring-[var(--accent)]' : '',
        flaggedIndices.has(i - 1) ? 'bg-amber-100 text-amber-700' :
        answeredIndices.has(i - 1) ? 'bg-emerald-100 text-emerald-700' :
        'bg-[var(--primary-light)] text-[var(--text-3)] hover:bg-[var(--accent-light)]',
      ]"
      @click="$emit('navigate', i - 1)"
    >
      {{ i }}
    </button>
  </div>
</template>
