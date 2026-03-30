<script setup lang="ts">
import { ref, computed } from 'vue'

defineProps<{
  maxWords?: number
  minWords?: number
  answered: boolean
}>()

const emit = defineEmits<{ submit: [text: string] }>()
const text = ref('')
const wordCount = computed(() => text.value.trim().split(/\s+/).filter(w => w).length)
</script>

<template>
  <div>
    <textarea
      v-model="text"
      :disabled="answered"
      class="w-full px-4 py-3 rounded-[var(--radius-lg)] border text-sm leading-relaxed resize-y min-h-[200px] transition-colors"
      :style="{
        backgroundColor: 'var(--card-bg)',
        borderColor: 'var(--card-border)',
        color: 'var(--text)',
      }"
      placeholder="Write your answer here..."
    />
    <div class="flex items-center justify-between mt-2">
      <span class="text-xs tabular-nums" :style="{ color: wordCount < (minWords || 0) ? 'var(--danger)' : 'var(--text-3)' }">
        {{ wordCount }} words
        <span v-if="minWords"> (min {{ minWords }})</span>
        <span v-if="maxWords"> / {{ maxWords }} max</span>
      </span>
      <button
        v-if="!answered && wordCount >= (minWords || 1)"
        class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
        :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', text)">
        Submit
      </button>
    </div>
  </div>
</template>
