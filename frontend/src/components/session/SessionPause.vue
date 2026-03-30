<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  visible: boolean
  sessionType?: string
  answeredCount?: number
  totalQuestions?: number
  elapsedTime?: string
}>()

defineEmits<{ resume: []; stop: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition name="pause">
      <div v-if="visible" class="fixed inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm" :style="{ zIndex: 'var(--z-modal)' }">
        <AppCard padding="lg" class="max-w-sm w-full text-center">
          <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl" :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">⏸</div>
          <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">Session Paused</h2>
          <p v-if="answeredCount !== undefined && totalQuestions" class="text-sm mb-1" :style="{ color: 'var(--text-2)' }">
            {{ answeredCount }}/{{ totalQuestions }} answered
          </p>
          <p v-if="elapsedTime" class="text-xs mb-6" :style="{ color: 'var(--text-3)' }">Time: {{ elapsedTime }}</p>
          <div class="flex items-center justify-center gap-3">
            <AppButton variant="primary" size="lg" @click="$emit('resume')">Resume →</AppButton>
            <AppButton variant="ghost" @click="$emit('stop')">End Session</AppButton>
          </div>
        </AppCard>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.pause-enter-active { transition: all var(--dur-slow) var(--ease-out); }
.pause-leave-active { transition: all var(--dur-normal); }
.pause-enter-from, .pause-leave-to { opacity: 0; }
</style>
