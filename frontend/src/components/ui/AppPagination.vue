<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ currentPage: number; totalPages: number }>()
const emit = defineEmits<{ 'update:currentPage': [page: number] }>()

const pages = computed(() => {
  const p: (number | '...')[] = []
  for (let i = 1; i <= props.totalPages; i++) {
    if (i === 1 || i === props.totalPages || Math.abs(i - props.currentPage) <= 1) p.push(i)
    else if (p[p.length - 1] !== '...') p.push('...')
  }
  return p
})
</script>

<template>
  <div class="flex items-center gap-1">
    <button class="w-8 h-8 rounded-[var(--radius-sm)] text-xs flex items-center justify-center" :disabled="currentPage <= 1"
      :style="{ color: 'var(--text-3)' }" @click="emit('update:currentPage', currentPage - 1)">‹</button>
    <button v-for="(p, i) in pages" :key="i" :disabled="p === '...'"
      class="w-8 h-8 rounded-[var(--radius-sm)] text-xs font-medium flex items-center justify-center transition-colors"
      :class="p === currentPage ? 'text-white' : ''"
      :style="{ backgroundColor: p === currentPage ? 'var(--accent)' : 'transparent', color: p === currentPage ? 'white' : 'var(--text-2)' }"
      @click="typeof p === 'number' && emit('update:currentPage', p)">{{ p }}</button>
    <button class="w-8 h-8 rounded-[var(--radius-sm)] text-xs flex items-center justify-center" :disabled="currentPage >= totalPages"
      :style="{ color: 'var(--text-3)' }" @click="emit('update:currentPage', currentPage + 1)">›</button>
  </div>
</template>
