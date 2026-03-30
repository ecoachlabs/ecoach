<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  open: boolean
  title: string
  type: string
  quickMeaning: string
  formula?: string
  entryId?: number
}>()

defineEmits<{ close: []; viewFull: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition name="panel">
      <div v-if="open" class="fixed bottom-0 left-0 right-0" :style="{ zIndex: 'var(--z-overlay)' }">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/20" @click="$emit('close')" />

        <!-- Panel -->
        <div class="relative rounded-t-[var(--radius-2xl)] px-6 py-5 max-w-2xl mx-auto"
          :style="{ backgroundColor: 'var(--card-bg)', boxShadow: 'var(--shadow-xl)' }">

          <!-- Handle -->
          <div class="w-8 h-1 rounded-full mx-auto mb-4" :style="{ backgroundColor: 'var(--border-strong)' }" />

          <!-- Header -->
          <div class="flex items-start justify-between mb-3">
            <div>
              <div class="flex items-center gap-2">
                <h3 class="font-display text-lg font-semibold" :style="{ color: 'var(--text)' }">{{ title }}</h3>
                <AppBadge color="accent" size="xs">{{ type }}</AppBadge>
              </div>
            </div>
            <button class="text-[var(--text-3)] hover:text-[var(--text)] transition-colors" @click="$emit('close')">✕</button>
          </div>

          <!-- Quick meaning -->
          <p class="text-sm leading-relaxed mb-3" :style="{ color: 'var(--text-2)' }">{{ quickMeaning }}</p>

          <!-- Formula if applicable -->
          <div v-if="formula" class="p-3 rounded-[var(--radius-md)] mb-3 font-mono text-sm text-center"
            :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--text)' }">
            {{ formula }}
          </div>

          <!-- Action -->
          <AppButton variant="secondary" size="sm" @click="$emit('viewFull')">View Full Entry →</AppButton>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.panel-enter-active { transition: all var(--dur-slow) var(--ease-out); }
.panel-leave-active { transition: all var(--dur-normal) var(--ease-smooth); }
.panel-enter-from, .panel-leave-to { transform: translateY(100%); }
</style>
