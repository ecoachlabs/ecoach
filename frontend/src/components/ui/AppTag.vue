<script setup lang="ts">
defineProps<{ label: string; color?: string; removable?: boolean }>()
defineEmits<{ remove: [] }>()
</script>

<template>
  <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-[var(--radius-sm)] text-[10px] font-medium"
    :style="{backgroundColor: (color || 'var(--accent)') + '15', color: color || 'var(--accent)'}">
    {{ label }}
    <button v-if="removable" class="hover:opacity-70 -mr-0.5" @click.stop="$emit('remove')">✕</button>
  </span>
</template>
