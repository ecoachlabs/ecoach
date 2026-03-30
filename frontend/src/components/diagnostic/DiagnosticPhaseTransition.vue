<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  currentPhase: string
  nextPhase: string
  phaseNumber: number
  totalPhases: number
  message?: string
}>()

defineEmits<{ continue: [] }>()

const phaseIcons: Record<string, string> = {
  baseline: '◎', speed: '⚡', precision: '◈', pressure: '🔥', flex: '↻', root_cause: '🔍',
}
</script>

<template>
  <div class="min-h-[60vh] flex flex-col items-center justify-center text-center px-8 reveal">
    <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
      :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
      {{ phaseIcons[nextPhase] || '◎' }}
    </div>
    <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">
      Phase {{ phaseNumber }} of {{ totalPhases }}
    </h2>
    <p class="text-sm mb-2" :style="{ color: 'var(--text-2)' }">
      Next: <strong>{{ nextPhase.replace(/_/g, ' ') }}</strong>
    </p>
    <p v-if="message" class="text-xs max-w-sm mb-6" :style="{ color: 'var(--text-3)' }">{{ message }}</p>
    <p v-else class="text-xs max-w-sm mb-6" :style="{ color: 'var(--text-3)' }">
      Take a breath. The next phase tests a different dimension of your knowledge.
    </p>
    <AppButton variant="primary" @click="$emit('continue')">Continue →</AppButton>
  </div>
</template>
