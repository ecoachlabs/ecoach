import { ref, computed, onUnmounted } from 'vue'

export type TimerVariant = 'soft' | 'strict' | 'shrinking' | 'burst' | 'pressure' | 'cluster'

export function useTimer(totalSeconds: number, variant: TimerVariant = 'soft') {
  const remaining = ref(totalSeconds)
  const running = ref(false)
  const elapsed = ref(0)
  let intervalId: ReturnType<typeof setInterval> | null = null

  const percentage = computed(() => (remaining.value / totalSeconds) * 100)
  const isUrgent = computed(() => percentage.value < 25)
  const isWarning = computed(() => percentage.value < 50 && !isUrgent.value)
  const isExpired = computed(() => remaining.value <= 0)

  const formatted = computed(() => {
    const mins = Math.floor(remaining.value / 60)
    const secs = remaining.value % 60
    return mins > 0 ? `${mins}:${String(secs).padStart(2, '0')}` : `${secs}s`
  })

  function start() {
    if (running.value) return
    running.value = true
    intervalId = setInterval(() => {
      remaining.value = Math.max(0, remaining.value - 1)
      elapsed.value++
      if (remaining.value <= 0) {
        stop()
      }
    }, 1000)
  }

  function stop() {
    running.value = false
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  function reset(newTotal?: number) {
    stop()
    remaining.value = newTotal ?? totalSeconds
    elapsed.value = 0
  }

  function pause() { stop() }
  function resume() { start() }

  onUnmounted(() => stop())

  return {
    remaining,
    running,
    elapsed,
    percentage,
    isUrgent,
    isWarning,
    isExpired,
    formatted,
    start,
    stop,
    reset,
    pause,
    resume,
  }
}
