<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  reason: string
  context?: string
}>()

defineEmits<{ dismiss: [] }>()
</script>

<template>
  <AppCard padding="sm">
    <div class="flex items-start gap-2.5">
      <span class="text-base mt-0.5">💡</span>
      <div class="flex-1">
        <p class="text-xs font-semibold mb-0.5" :style="{ color: 'var(--accent)' }">Why this session?</p>
        <p class="text-sm leading-relaxed" :style="{ color: 'var(--text-2)' }">{{ reason }}</p>
        <p v-if="context" class="text-[10px] mt-1" :style="{ color: 'var(--text-3)' }">{{ context }}</p>
      </div>
      <button class="text-[var(--text-3)] hover:text-[var(--text)] text-xs" @click="$emit('dismiss')">✕</button>
    </div>
  </AppCard>
</template>
