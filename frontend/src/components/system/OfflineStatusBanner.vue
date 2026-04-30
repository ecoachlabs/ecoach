<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { flushQueuedIpcCalls } from '@/ipc'
import { useConnectivityStore } from '@/stores/connectivity'

const connectivity = useConnectivityStore()

const visible = computed(() => (
  !connectivity.isOnline ||
  connectivity.queuedActionCount > 0 ||
  connectivity.syncState === 'syncing' ||
  connectivity.syncState === 'error'
))

const message = computed(() => {
  if (!connectivity.isOnline) {
    return 'Offline mode. Practice, reports, curriculum, and local content stay available.'
  }

  if (connectivity.syncState === 'syncing') {
    return 'Back online. Syncing queued work now.'
  }

  if (connectivity.syncState === 'error') {
    return 'Some queued work still needs a connection.'
  }

  if (connectivity.queuedActionCount > 0) {
    return `${connectivity.queuedActionCount} queued action${connectivity.queuedActionCount === 1 ? '' : 's'} ready to sync.`
  }

  return ''
})

async function syncNow() {
  if (!connectivity.isOnline) return

  connectivity.beginSync()
  try {
    const result = await flushQueuedIpcCalls()
    connectivity.finishSync(result.remaining)
  } catch (error) {
    connectivity.failSync(error)
  }
}
</script>

<template>
  <Transition name="offline-banner">
    <aside
      v-if="visible"
      class="offline-status"
      :data-state="connectivity.isOnline ? connectivity.syncState : 'offline'"
      role="status"
      aria-live="polite"
    >
      <span class="offline-status__dot" aria-hidden="true" />
      <span class="offline-status__message">{{ message }}</span>
      <AppButton
        v-if="connectivity.isOnline && connectivity.queuedActionCount > 0 && connectivity.syncState !== 'syncing'"
        variant="secondary"
        size="sm"
        @click="syncNow"
      >
        Sync now
      </AppButton>
    </aside>
  </Transition>
</template>

<style scoped>
.offline-status {
  position: fixed;
  left: 50%;
  bottom: 1rem;
  z-index: 80;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: min(calc(100vw - 2rem), 48rem);
  min-height: 3rem;
  padding: 0.6rem 0.8rem;
  color: var(--text);
  background: var(--card-bg);
  border: 1px solid var(--field-border);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  transform: translateX(-50%);
}

.offline-status[data-state="offline"] {
  border-color: color-mix(in srgb, var(--warning) 55%, var(--field-border));
}

.offline-status[data-state="error"] {
  border-color: color-mix(in srgb, var(--danger) 55%, var(--field-border));
}

.offline-status__dot {
  flex: 0 0 0.6rem;
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 999px;
  background: var(--success);
}

.offline-status[data-state="offline"] .offline-status__dot {
  background: var(--warning);
}

.offline-status[data-state="error"] .offline-status__dot {
  background: var(--danger);
}

.offline-status__message {
  flex: 1 1 auto;
  min-width: 0;
  font-size: 0.9rem;
  line-height: 1.35;
}

.offline-banner-enter-active,
.offline-banner-leave-active {
  transition: opacity 180ms ease, transform 180ms ease;
}

.offline-banner-enter-from,
.offline-banner-leave-to {
  opacity: 0;
  transform: translate(-50%, 0.5rem);
}
</style>
