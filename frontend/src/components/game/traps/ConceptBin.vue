<script setup lang="ts">
defineProps<{
  label: string
  side: 'left' | 'right'
  items: string[]
  highlighted: boolean
}>()

defineEmits<{ drop: [] }>()
</script>

<template>
  <div
    class="rounded-[var(--radius-xl)] border-2 border-dashed p-4 min-h-[120px] transition-all"
    :class="highlighted ? 'border-[var(--accent)] bg-[var(--accent-light)] scale-[1.01]' : 'border-[var(--card-border)]'"
    :style="{ backgroundColor: !highlighted ? 'var(--card-bg)' : undefined }"
    @dragover.prevent
    @dragenter.prevent="$emit('drop')"
  >
    <p class="text-xs font-bold uppercase tracking-wider text-center mb-3"
      :style="{ color: highlighted ? 'var(--accent)' : 'var(--text-3)' }">
      {{ label }}
    </p>
    <div class="space-y-1.5">
      <div v-for="(item, i) in items" :key="i"
        class="px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-medium text-center reveal"
        :style="{ backgroundColor: side === 'left' ? 'var(--accent-light)' : 'var(--gold-light)', color: side === 'left' ? 'var(--accent)' : 'var(--gold)' }">
        {{ item }}
      </div>
    </div>
  </div>
</template>
