<script setup lang="ts">
import { ref } from 'vue'
import TrapCard from './TrapCard.vue'
import ConceptBin from './ConceptBin.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  conceptA: string
  conceptB: string
  cards: { id: number; text: string; type: string; correctBin: 'a' | 'b' }[]
  currentCardIndex: number
  binAItems: string[]
  binBItems: string[]
  score: number
  streak: number
}>()

defineEmits<{ drop: [cardId: number, bin: 'a' | 'b']; notSure: [cardId: number] }>()

const highlightedBin = ref<'a' | 'b' | null>(null)
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Score bar -->
    <div class="flex items-center justify-between mb-4 text-xs">
      <AppBadge color="accent" size="sm">Score: {{ score }}</AppBadge>
      <span v-if="streak > 0" class="font-bold tabular-nums" :style="{ color: 'var(--gold)' }">🔥 {{ streak }} streak</span>
      <span :style="{ color: 'var(--text-3)' }">{{ currentCardIndex + 1 }} / {{ cards.length }}</span>
    </div>

    <!-- Active card -->
    <div v-if="cards[currentCardIndex]" class="flex justify-center mb-6">
      <TrapCard :text="cards[currentCardIndex].text" :type="cards[currentCardIndex].type" active />
    </div>
    <div v-else class="flex-1 flex items-center justify-center">
      <p class="text-sm" :style="{ color: 'var(--text-3)' }">All cards sorted!</p>
    </div>

    <!-- Bins -->
    <div class="grid grid-cols-2 gap-4 mt-auto">
      <ConceptBin :label="conceptA" side="left" :items="binAItems"
        :highlighted="highlightedBin === 'a'"
        @drop="$emit('drop', cards[currentCardIndex]?.id, 'a')"
        @dragenter="highlightedBin = 'a'"
        @dragleave="highlightedBin = null" />
      <ConceptBin :label="conceptB" side="right" :items="binBItems"
        :highlighted="highlightedBin === 'b'"
        @drop="$emit('drop', cards[currentCardIndex]?.id, 'b')"
        @dragenter="highlightedBin = 'b'"
        @dragleave="highlightedBin = null" />
    </div>

    <!-- Not sure button -->
    <div v-if="cards[currentCardIndex]" class="mt-3 text-center">
      <button class="text-xs px-3 py-1.5 rounded-full border" :style="{ borderColor: 'var(--card-border)', color: 'var(--text-3)' }"
        @click="$emit('notSure', cards[currentCardIndex].id)">Not Sure</button>
    </div>
  </div>
</template>
