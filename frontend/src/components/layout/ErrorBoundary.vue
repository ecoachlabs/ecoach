<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  fallbackTitle?: string
  fallbackMessage?: string
}>()

const error = ref<Error | null>(null)
const errorInfo = ref('')

onErrorCaptured((err, instance, info) => {
  error.value = err
  errorInfo.value = info
  console.error('[ErrorBoundary]', err, info)
  return false // prevent propagation
})

function retry() {
  error.value = null
  errorInfo.value = ''
}
</script>

<template>
  <div v-if="error" class="p-6 max-w-lg mx-auto">
    <AppCard padding="lg" class="text-center">
      <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
        :style="{backgroundColor:'var(--danger-light)',color:'var(--danger)'}">✕</div>
      <h2 class="font-display text-lg font-semibold mb-1" :style="{color:'var(--text)'}">
        {{ fallbackTitle || 'Something went wrong' }}
      </h2>
      <p class="text-sm mb-2" :style="{color:'var(--text-2)'}">
        {{ fallbackMessage || 'An error occurred while loading this page.' }}
      </p>
      <p class="text-xs font-mono mb-4 px-3 py-2 rounded-[var(--radius-md)] text-left overflow-auto max-h-24"
        :style="{backgroundColor:'var(--primary-light)',color:'var(--danger)'}">
        {{ error.message }}
      </p>
      <div class="flex items-center justify-center gap-3">
        <AppButton variant="primary" @click="retry">Try Again</AppButton>
        <AppButton variant="ghost" @click="$router?.push('/student')">Go Home</AppButton>
      </div>
    </AppCard>
  </div>
  <slot v-else />
</template>
