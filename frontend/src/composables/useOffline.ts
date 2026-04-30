import { storeToRefs } from 'pinia'
import { useConnectivityStore } from '@/stores/connectivity'

export function useOffline() {
  const connectivity = useConnectivityStore()
  const { isOnline, wasOffline } = storeToRefs(connectivity)

  connectivity.startMonitoring()

  return { isOnline, wasOffline }
}
