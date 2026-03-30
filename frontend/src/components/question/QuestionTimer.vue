<script setup lang="ts">
import { ref, watch, onUnmounted, computed } from 'vue'

const props = defineProps<{
  totalSeconds: number
  variant?: 'soft' | 'strict' | 'shrinking' | 'burst' | 'pressure' | 'cluster'
  running?: boolean
}>()

const emit = defineEmits<{ timeout: []; tick: [remaining: number] }>()

const remaining = ref(props.totalSeconds)
const intervalId = ref<ReturnType<typeof setInterval> | null>(null)

const percentage = computed(() => (remaining.value / props.totalSeconds) * 100)
const isUrgent = computed(() => percentage.value < 25)
const isWarning = computed(() => percentage.value < 50 && !isUrgent.value)

const barColor = computed(() => {
  if (isUrgent.value) return 'var(--danger)'
  if (isWarning.value) return 'var(--warning)'
  return 'var(--accent)'
})

const formattedTime = computed(() => {
  const mins = Math.floor(remaining.value / 60)
  const secs = remaining.value % 60
  return mins > 0 ? `${mins}:${String(secs).padStart(2, '0')}` : `${secs}s`
})

function start() {
  stop()
  intervalId.value = setInterval(() => {
    remaining.value--
    emit('tick', remaining.value)
    if (remaining.value <= 0) {
      stop()
      emit('timeout')
    }
  }, 1000)
}

function stop() {
  if (intervalId.value) {
    clearInterval(intervalId.value)
    intervalId.value = null
  }
}

function reset() {
  stop()
  remaining.value = props.totalSeconds
}

watch(() => props.running, (val) => {
  if (val) start()
  else stop()
}, { immediate: true })

onUnmounted(() => stop())

defineExpose({ reset, stop, start, remaining })
</script>

<template>
  <div class="space-y-1">
    <!-- Timer bar -->
    <div class="h-1.5 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
      <div
        class="h-full rounded-full transition-all"
        :class="isUrgent ? 'animate-pulse' : ''"
        :style="{
          width: percentage + '%',
          backgroundColor: barColor,
          transitionDuration: '1s',
          transitionTimingFunction: 'linear',
        }"
      />
    </div>
    <!-- Time display -->
    <div class="flex items-center justify-between">
      <span
        class="text-xs font-mono font-semibold tabular-nums"
        :class="isUrgent ? 'text-red-600' : ''"
        :style="!isUrgent ? { color: 'var(--text-3)' } : {}"
      >
        {{ formattedTime }}
      </span>
      <span v-if="variant === 'pressure' || variant === 'shrinking'" class="text-[10px]" :style="{ color: 'var(--text-3)' }">
        {{ variant }}
      </span>
    </div>
  </div>
</template>
