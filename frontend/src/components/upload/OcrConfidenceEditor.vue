<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  segments: { id: number; text: string; confidence: number; corrected?: string }[]
}>()

defineEmits<{ correct: [segmentId: number, newText: string]; accept: [] }>()

function confidenceColor(conf: number): string {
  if (conf >= 90) return 'var(--success)'
  if (conf >= 70) return 'var(--gold)'
  if (conf >= 50) return 'var(--warning)'
  return 'var(--danger)'
}
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
      OCR Result Review
    </h3>
    <p class="text-xs mb-4" :style="{ color: 'var(--text-3)' }">
      Low-confidence text is highlighted. Tap to correct.
    </p>

    <AppCard padding="md" class="mb-4">
      <div class="leading-relaxed text-sm">
        <span v-for="seg in segments" :key="seg.id"
          class="inline px-0.5 rounded cursor-pointer transition-colors"
          :class="seg.confidence < 70 ? 'underline decoration-wavy decoration-1' : ''"
          :style="{
            backgroundColor: seg.confidence < 70 ? confidenceColor(seg.confidence) + '15' : 'transparent',
            textDecorationColor: seg.confidence < 70 ? confidenceColor(seg.confidence) : 'transparent',
            color: 'var(--text)',
          }"
          :title="`Confidence: ${seg.confidence}%`"
          @click="$emit('correct', seg.id, seg.text)">
          {{ seg.corrected || seg.text }}
        </span>
      </div>
    </AppCard>

    <div class="flex items-center justify-between">
      <span class="text-[10px]" :style="{ color: 'var(--text-3)' }">
        {{ segments.filter(s => s.confidence < 70).length }} segments need review
      </span>
      <AppButton variant="primary" size="sm" @click="$emit('accept')">Accept & Continue</AppButton>
    </div>
  </div>
</template>
