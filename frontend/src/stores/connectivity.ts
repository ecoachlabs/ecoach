import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

type SyncState = 'idle' | 'offline' | 'syncing' | 'error'

const OFFLINE_QUEUE_EVENT = 'ecoach:offline-queue-changed'
const OFFLINE_QUEUE_FLUSHED_EVENT = 'ecoach:offline-queue-flushed'
const CONNECTIVITY_RESTORED_EVENT = 'ecoach:connectivity-restored'

function browserReportsOnline(): boolean {
  return typeof navigator === 'undefined' ? true : navigator.onLine
}

function queueCountFromEvent(event: Event): number | null {
  const detail = (event as CustomEvent<{ count?: number }>).detail
  return typeof detail?.count === 'number' ? detail.count : null
}

export const useConnectivityStore = defineStore('connectivity', () => {
  const isBrowserOnline = ref(browserReportsOnline())
  const isReachable = ref(true)
  const monitoring = ref(false)
  const queuedActionCount = ref(0)
  const lastOnlineAt = ref<string | null>(null)
  const lastOfflineAt = ref<string | null>(null)
  const lastSyncError = ref<string | null>(null)
  const restorePulse = ref(0)
  const syncState = ref<SyncState>(browserReportsOnline() ? 'idle' : 'offline')
  const wasOffline = ref(!browserReportsOnline())

  const isOnline = computed(() => isBrowserOnline.value && isReachable.value)
  const hasQueuedActions = computed(() => queuedActionCount.value > 0)

  async function probeConnectivity(): Promise<boolean> {
    isBrowserOnline.value = browserReportsOnline()
    isReachable.value = isBrowserOnline.value
    return isOnline.value
  }

  function setQueueCount(count: number) {
    queuedActionCount.value = Math.max(0, count)
  }

  function beginSync() {
    if (!isOnline.value) {
      syncState.value = 'offline'
      return
    }

    lastSyncError.value = null
    syncState.value = 'syncing'
  }

  function finishSync(remainingCount = queuedActionCount.value) {
    setQueueCount(remainingCount)
    lastSyncError.value = null

    if (!isOnline.value) {
      syncState.value = 'offline'
      return
    }

    if (remainingCount > 0) {
      lastSyncError.value = 'Queued online work could not be completed yet.'
      syncState.value = 'error'
      return
    }

    syncState.value = 'idle'
  }

  function failSync(error: unknown) {
    lastSyncError.value = error instanceof Error ? error.message : String(error)
    syncState.value = isOnline.value ? 'error' : 'offline'
  }

  async function handleOnline() {
    isBrowserOnline.value = true
    await probeConnectivity()
    lastOnlineAt.value = new Date().toISOString()
    restorePulse.value += 1
    wasOffline.value = false

    if (isOnline.value) {
      syncState.value = queuedActionCount.value > 0 ? 'syncing' : 'idle'
      window.dispatchEvent(new CustomEvent(CONNECTIVITY_RESTORED_EVENT))
    }
  }

  function handleOffline() {
    isBrowserOnline.value = false
    isReachable.value = false
    lastOfflineAt.value = new Date().toISOString()
    wasOffline.value = true
    syncState.value = 'offline'
  }

  function handleQueueChanged(event: Event) {
    const count = queueCountFromEvent(event)
    if (count !== null) {
      setQueueCount(count)
      if (count === 0 && isOnline.value && syncState.value === 'syncing') {
        syncState.value = 'idle'
      }
    }
  }

  function handleQueueFlushed(event: Event) {
    const detail = (event as CustomEvent<{ remaining?: number }>).detail
    const remaining = typeof detail?.remaining === 'number' ? detail.remaining : queuedActionCount.value
    finishSync(remaining)
  }

  function startMonitoring() {
    if (monitoring.value || typeof window === 'undefined') return

    monitoring.value = true
    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)
    window.addEventListener(OFFLINE_QUEUE_EVENT, handleQueueChanged)
    window.addEventListener(OFFLINE_QUEUE_FLUSHED_EVENT, handleQueueFlushed)

    if (browserReportsOnline()) {
      void probeConnectivity()
    } else {
      handleOffline()
    }
  }

  function stopMonitoring() {
    if (!monitoring.value || typeof window === 'undefined') return

    window.removeEventListener('online', handleOnline)
    window.removeEventListener('offline', handleOffline)
    window.removeEventListener(OFFLINE_QUEUE_EVENT, handleQueueChanged)
    window.removeEventListener(OFFLINE_QUEUE_FLUSHED_EVENT, handleQueueFlushed)
    monitoring.value = false
  }

  return {
    isBrowserOnline,
    isReachable,
    isOnline,
    monitoring,
    queuedActionCount,
    hasQueuedActions,
    lastOnlineAt,
    lastOfflineAt,
    lastSyncError,
    restorePulse,
    syncState,
    wasOffline,
    probeConnectivity,
    setQueueCount,
    beginSync,
    finishSync,
    failSync,
    startMonitoring,
    stopMonitoring,
  }
})
