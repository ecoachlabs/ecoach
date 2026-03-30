<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  leftItems: { id: number; text: string }[]
  rightItems: { id: number; text: string }[]
  answered: boolean
  correctMatches?: Record<number, number>
}>()

const emit = defineEmits<{ submit: [matches: Record<number, number>] }>()

const matches = ref<Record<number, number>>({})
const selectedLeft = ref<number | null>(null)

function selectLeft(id: number) {
  if (props.answered) return
  selectedLeft.value = id
}

function selectRight(id: number) {
  if (props.answered || selectedLeft.value === null) return
  matches.value[selectedLeft.value] = id
  selectedLeft.value = null
}

function isCorrectMatch(leftId: number): boolean | null {
  if (!props.answered || !props.correctMatches) return null
  if (!(leftId in matches.value)) return null
  return matches.value[leftId] === props.correctMatches[leftId]
}

const allMatched = () => Object.keys(matches.value).length === props.leftItems.length
</script>

<template>
  <div class="grid grid-cols-2 gap-6">
    <!-- Left column -->
    <div class="space-y-2">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Match from</p>
      <button
        v-for="item in leftItems" :key="item.id"
        class="w-full px-3 py-2.5 rounded-[var(--radius-md)] border text-left text-sm transition-all"
        :class="[
          answered && isCorrectMatch(item.id) === true ? 'border-emerald-500 bg-emerald-50' :
          answered && isCorrectMatch(item.id) === false ? 'border-red-400 bg-red-50' :
          selectedLeft === item.id ? 'border-[var(--accent)] bg-[var(--accent-light)]' :
          item.id in matches ? 'border-[var(--accent)] opacity-60' :
          'border-[var(--card-border)] hover:border-[var(--accent)]',
        ]"
        :style="{ backgroundColor: (!answered && selectedLeft !== item.id && !(item.id in matches)) ? 'var(--card-bg)' : undefined }"
        @click="selectLeft(item.id)"
      >
        {{ item.text }}
      </button>
    </div>
    <!-- Right column -->
    <div class="space-y-2">
      <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Match to</p>
      <button
        v-for="item in rightItems" :key="item.id"
        class="w-full px-3 py-2.5 rounded-[var(--radius-md)] border text-left text-sm transition-all"
        :class="[
          selectedLeft !== null ? 'border-[var(--card-border)] hover:border-[var(--accent)] hover:bg-[var(--accent-light)]' :
          'border-[var(--card-border)]',
          Object.values(matches).includes(item.id) ? 'opacity-60' : '',
        ]"
        :style="{ backgroundColor: 'var(--card-bg)' }"
        :disabled="answered || selectedLeft === null"
        @click="selectRight(item.id)"
      >
        {{ item.text }}
      </button>
    </div>
  </div>
  <div v-if="!answered && allMatched()" class="mt-4 text-right">
    <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
      :style="{ backgroundColor: 'var(--accent)' }"
      @click="emit('submit', matches)">Submit Matches</button>
  </div>
</template>
