<script setup lang="ts">
defineProps<{ label: string; removable?: boolean; active?: boolean; color?: string }>()
defineEmits<{ remove: []; click: [] }>()
</script>

<template>
  <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium cursor-pointer transition-all"
    :class="active ? 'text-white shadow-sm' : 'hover:opacity-80'"
    :style="{ backgroundColor: active ? (color || 'var(--accent)') : 'var(--primary-light)', color: active ? 'white' : (color || 'var(--text-2)') }"
    @click="$emit('click')">
    {{ label }}
    <button v-if="removable" class="ml-0.5 hover:opacity-70" @click.stop="$emit('remove')">✕</button>
  </span>
</template>
