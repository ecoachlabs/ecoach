<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  items: { id: number; text: string }[]
  categories: { id: number; label: string }[]
  answered: boolean
  correctClassifications?: Record<number, number>
}>()

const emit = defineEmits<{ submit: [classifications: Record<number, number>] }>()
const classifications = ref<Record<number, number>>({})

function assignToCategory(itemId: number, categoryId: number) {
  if (props.answered) return
  classifications.value[itemId] = categoryId
}

const allClassified = () => Object.keys(classifications.value).length === props.items.length
</script>

<template>
  <div>
    <!-- Categories as drop targets -->
    <div class="grid gap-4 mb-4" :style="{ gridTemplateColumns: `repeat(${categories.length}, 1fr)` }">
      <div v-for="cat in categories" :key="cat.id" class="text-center">
        <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">{{ cat.label }}</p>
        <div class="min-h-[60px] rounded-[var(--radius-md)] border-2 border-dashed p-2 space-y-1"
          :style="{ borderColor: 'var(--card-border)' }">
          <div v-for="item in items.filter(i => classifications[i.id] === cat.id)" :key="item.id"
            class="px-2 py-1 rounded text-[11px] font-medium"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            {{ item.text }}
          </div>
        </div>
      </div>
    </div>

    <!-- Unclassified items -->
    <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Items to classify</p>
    <div class="flex flex-wrap gap-2">
      <div v-for="item in items.filter(i => !(i.id in classifications))" :key="item.id"
        class="group relative">
        <div class="px-3 py-2 rounded-[var(--radius-md)] border text-sm cursor-pointer"
          :style="{ backgroundColor: 'var(--card-bg)', borderColor: 'var(--card-border)', color: 'var(--text)' }">
          {{ item.text }}
        </div>
        <!-- Category selector on hover -->
        <div class="absolute top-full left-0 mt-1 hidden group-hover:flex gap-1 z-10">
          <button v-for="cat in categories" :key="cat.id"
            class="px-2 py-1 rounded text-[10px] font-medium shadow-md"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="assignToCategory(item.id, cat.id)">
            → {{ cat.label }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="!answered && allClassified()" class="mt-4 text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white"
        :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', classifications)">Submit</button>
    </div>
  </div>
</template>
