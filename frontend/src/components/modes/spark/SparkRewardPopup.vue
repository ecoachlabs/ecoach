<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  visible: boolean
  rewardType: 'badge' | 'title' | 'avatar' | 'streak' | 'surprise'
  rewardName: string
  description?: string
}>()

defineEmits<{ dismiss: [] }>()

const typeIcons: Record<string, string> = {
  badge: '🏅', title: '👑', avatar: '🎭', streak: '🔥', surprise: '🎁',
}
</script>

<template>
  <Teleport to="body">
    <Transition name="reward">
      <div v-if="visible" class="fixed inset-0 flex items-center justify-center" :style="{ zIndex: 'var(--z-modal)' }">
        <div class="absolute inset-0 bg-black/40" @click="$emit('dismiss')" />
        <div class="relative text-center px-8 py-10 rounded-[var(--radius-2xl)] max-w-sm"
          :style="{ backgroundColor: 'var(--card-bg)', boxShadow: 'var(--shadow-glow-gold)' }">
          <div class="text-5xl mb-4">{{ typeIcons[rewardType] || '🎁' }}</div>
          <h2 class="font-display text-xl font-bold mb-1" :style="{ color: 'var(--gold)' }">{{ rewardName }}</h2>
          <p v-if="description" class="text-sm mb-6" :style="{ color: 'var(--text-2)' }">{{ description }}</p>
          <AppButton variant="warm" @click="$emit('dismiss')">Awesome!</AppButton>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.reward-enter-active { transition: all 600ms cubic-bezier(0.34, 1.56, 0.64, 1); }
.reward-leave-active { transition: all 200ms; }
.reward-enter-from { opacity: 0; transform: scale(0.5); }
.reward-leave-to { opacity: 0; transform: scale(0.95); }
</style>
