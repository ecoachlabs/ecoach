<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  missionTitle: string
  missionType: string
  duration: string
  challengeText?: string
  reward?: string
}>()

defineEmits<{ start: [] }>()
</script>

<template>
  <AppCard padding="lg" glow="warm" class="text-center">
    <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
      :style="{ backgroundColor: 'var(--warm-light)', color: 'var(--warm)' }">
      🎯
    </div>
    <AppBadge color="warm" size="sm" class="mb-2">{{ missionType }}</AppBadge>
    <h3 class="font-display text-lg font-bold mb-1" :style="{ color: 'var(--text)' }">{{ missionTitle }}</h3>
    <p v-if="challengeText" class="text-sm mb-3" :style="{ color: 'var(--text-2)' }">{{ challengeText }}</p>
    <div class="flex items-center justify-center gap-3 mb-4 text-xs" :style="{ color: 'var(--text-3)' }">
      <span>⏱ {{ duration }}</span>
      <span v-if="reward">🏆 {{ reward }}</span>
    </div>
    <AppButton variant="warm" size="lg" @click="$emit('start')">Accept Challenge →</AppButton>
  </AppCard>
</template>
