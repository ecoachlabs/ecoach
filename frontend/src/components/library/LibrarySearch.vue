<script setup lang="ts">
import { ref } from 'vue'
import AppSearchInput from '@/components/ui/AppSearchInput.vue'
import AppChip from '@/components/ui/AppChip.vue'
import ContentObjectCard from './ContentObjectCard.vue'

defineProps<{
  results: { title: string; contentType: string; state: string; topic?: string }[]
  loading: boolean
}>()

defineEmits<{ search: [query: string]; filter: [type: string] }>()

const query = ref('')
const activeFilter = ref<string | null>(null)
const filters = ['question', 'formula', 'definition', 'worked_example', 'concept', 'note', 'past_paper']
</script>

<template>
  <div>
    <AppSearchInput v-model="query" placeholder="Search library..." :loading="loading" @search="$emit('search', query)" />
    <div class="flex flex-wrap gap-1 mt-3 mb-4">
      <AppChip v-for="f in filters" :key="f" :label="f" :active="activeFilter === f"
        @click="activeFilter = activeFilter === f ? null : f; $emit('filter', activeFilter || '')" />
    </div>
    <div v-if="results.length" class="space-y-2">
      <ContentObjectCard v-for="(r, i) in results" :key="i" v-bind="r" />
    </div>
    <p v-else-if="query && !loading" class="text-xs text-center py-6" :style="{ color: 'var(--text-3)' }">
      No results for "{{ query }}"
    </p>
  </div>
</template>
