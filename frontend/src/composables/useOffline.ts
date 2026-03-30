import { ref, onMounted, onUnmounted } from 'vue'

export function useOffline() {
  const isOnline = ref(navigator.onLine)
  const wasOffline = ref(false)

  function handleOnline() {
    isOnline.value = true
    if (wasOffline.value) {
      // Could trigger sync here
      wasOffline.value = false
    }
  }

  function handleOffline() {
    isOnline.value = false
    wasOffline.value = true
  }

  onMounted(() => {
    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)
  })

  onUnmounted(() => {
    window.removeEventListener('online', handleOnline)
    window.removeEventListener('offline', handleOffline)
  })

  return { isOnline, wasOffline }
}
