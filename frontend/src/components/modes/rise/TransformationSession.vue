<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  stage: 'rescue' | 'stabilize' | 'accelerate' | 'dominate'
  topicName: string
  sessionNumber: number
  progress: number
}>()

defineEmits<{ start: [] }>()

const stageThemes: Record<string, { label: string; color: string; desc: string; icon: string }> = {
  rescue: { label: 'Rescue', color: 'var(--warm)', desc: 'Small wins. Build from zero. No shame.', icon: '🛟' },
  stabilize: { label: 'Stabilize', color: 'var(--gold)', desc: 'Repeated practice. Build consistency.', icon: '⚖' },
  accelerate: { label: 'Accelerate', color: 'var(--accent)', desc: 'Timed drills. Mixed topics. Push harder.', icon: '🚀' },
  dominate: { label: 'Dominate', color: 'var(--success)', desc: 'Advanced. Traps. Speed + accuracy.', icon: '👑' },
}
</script>

<template>
  <AppCard padding="lg">
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-2xl flex items-center justify-center text-2xl"
        :style="{ backgroundColor: stageThemes[stage]?.color + '15' }">
        {{ stageThemes[stage]?.icon }}
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="text-sm font-bold" :style="{ color: stageThemes[stage]?.color }">{{ stageThemes[stage]?.label }} Stage</h3>
          <AppBadge color="muted" size="xs">Session {{ sessionNumber }}</AppBadge>
        </div>
        <p class="text-xs" :style="{ color: 'var(--text-2)' }">{{ topicName }}</p>
      </div>
    </div>

    <p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">{{ stageThemes[stage]?.desc }}</p>

    <AppProgress :value="progress" size="md" :color="stage === 'rescue' ? 'warm' : stage === 'dominate' ? 'success' : 'accent'" :glow="true" class="mb-4" />

    <AppButton variant="primary" size="lg" class="w-full" @click="$emit('start')">Begin Session →</AppButton>
  </AppCard>
</template>
