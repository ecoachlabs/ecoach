<script setup lang="ts">
import { ref, computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  title: string
  duration?: number
  entryType?: string
}>()

defineEmits<{ close: [] }>()

const playing = ref(false)
const progress = ref(0)
const speed = ref(1)
const speeds = [0.75, 1, 1.25, 1.5]

function togglePlay() { playing.value = !playing.value }
function nextSpeed() { speed.value = speeds[(speeds.indexOf(speed.value) + 1) % speeds.length] }

const formattedProgress = computed(() => {
  const mins = Math.floor(progress.value / 60)
  const secs = progress.value % 60
  return `${mins}:${String(secs).padStart(2, '0')}`
})
</script>

<template>
  <div class="p-5 rounded-[var(--radius-xl)]" :style="{ backgroundColor: 'var(--card-bg)' }">
    <!-- Now playing -->
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-xl flex items-center justify-center text-xl" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">🔊</div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-semibold truncate" :style="{ color: 'var(--text)' }">{{ title }}</p>
        <AppBadge v-if="entryType" color="muted" size="xs">{{ entryType }}</AppBadge>
      </div>
      <button class="text-xs" :style="{ color: 'var(--text-3)' }" @click="$emit('close')">✕</button>
    </div>

    <!-- Progress bar -->
    <div class="mb-3">
      <div class="h-1.5 rounded-full overflow-hidden cursor-pointer" :style="{ backgroundColor: 'var(--border-soft)' }">
        <div class="h-full rounded-full" :style="{ width: progress + '%', backgroundColor: 'var(--accent)' }" />
      </div>
      <div class="flex justify-between mt-1 text-[9px] tabular-nums" :style="{ color: 'var(--text-3)' }">
        <span>{{ formattedProgress }}</span>
        <span>{{ duration ? Math.floor(duration / 60) + ':' + String(duration % 60).padStart(2, '0') : '--:--' }}</span>
      </div>
    </div>

    <!-- Controls -->
    <div class="flex items-center justify-center gap-4">
      <button class="text-lg" :style="{ color: 'var(--text-3)' }">⏮</button>
      <button class="w-12 h-12 rounded-full flex items-center justify-center text-xl text-white shadow-md"
        :style="{ backgroundColor: 'var(--accent)' }" @click="togglePlay">
        {{ playing ? '⏸' : '▶' }}
      </button>
      <button class="text-lg" :style="{ color: 'var(--text-3)' }">⏭</button>
      <button class="px-2 py-1 rounded text-[10px] font-bold tabular-nums"
        :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }" @click="nextSpeed">
        {{ speed }}x
      </button>
    </div>
  </div>
</template>
