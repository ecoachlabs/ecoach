<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import MathText from '@/components/question/MathText.vue'

defineProps<{
  stem: string
  options: { id: number; label: string; text: string }[]
  timeLimit?: number
  active: boolean
}>()

const emit = defineEmits<{ answer: [optionId: number] }>()
const selectedId = ref<number | null>(null)

function select(id: number) {
  selectedId.value = id
  emit('answer', id)
  setTimeout(() => { selectedId.value = null }, 300)
}
</script>

<template>
  <div class="p-4 rounded-[var(--radius-lg)]" :style="{ backgroundColor: 'var(--card-bg)' }">
    <p class="text-sm font-medium mb-3 leading-relaxed" :style="{ color: 'var(--text)' }">
      <MathText :text="stem" size="sm" />
    </p>
    <div class="space-y-1.5">
      <button v-for="opt in options" :key="opt.id"
        class="w-full px-3 py-2 rounded-[var(--radius-md)] border text-left text-sm transition-all"
        :class="selectedId === opt.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' : 'border-[var(--card-border)] hover:border-[var(--accent)]'"
        :style="{ backgroundColor: selectedId !== opt.id ? 'var(--card-bg)' : undefined, color: 'var(--text)' }"
        :disabled="!active"
        @click="select(opt.id)">
        <span class="font-bold text-xs mr-2" :style="{ color: 'var(--text-3)' }">{{ opt.label }}</span>
        <MathText :text="opt.text" size="sm" />
      </button>
    </div>
  </div>
</template>
