<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  title?: string
  visible: boolean
}>()

defineEmits<{ close: []; togglePlay: [] }>()

const playing = ref(false)
const progress = ref(35)
const speed = ref(1)

function toggleSpeed() {
  const speeds = [0.75, 1, 1.25, 1.5]
  const idx = speeds.indexOf(speed.value)
  speed.value = speeds[(idx + 1) % speeds.length]
}
</script>

<template>
  <Transition name="player">
    <div
      v-if="visible"
      class="fixed bottom-0 left-0 right-0 px-4 py-3 flex items-center gap-4"
      :style="{
        backgroundColor: 'var(--card-bg)',
        zIndex: 'var(--z-sticky)',
      }"
    >
      <!-- Play/Pause -->
      <button
        class="w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-bold shrink-0"
        :style="{ backgroundColor: 'var(--accent)' }"
        @click="playing = !playing; $emit('togglePlay')"
      >
        {{ playing ? '⏸' : '▶' }}
      </button>

      <!-- Info + Progress -->
      <div class="flex-1 min-w-0">
        <p class="text-xs font-medium truncate" :style="{ color: 'var(--text)' }">
          {{ title || 'Not playing' }}
        </p>
        <div class="mt-1 h-1 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
          <div class="h-full rounded-full" :style="{ width: progress + '%', backgroundColor: 'var(--accent)' }" />
        </div>
      </div>

      <!-- Speed -->
      <button
        class="px-2 py-1 rounded text-[10px] font-bold tabular-nums"
        :style="{ color: 'var(--accent)', backgroundColor: 'var(--accent-light)' }"
        @click="toggleSpeed"
      >
        {{ speed }}x
      </button>

      <!-- Close -->
      <button class="text-[var(--text-3)] hover:text-[var(--text)]" @click="$emit('close')">✕</button>
    </div>
  </Transition>
</template>

<style scoped>
.player-enter-active { transition: all var(--dur-normal) var(--ease-spring); }
.player-leave-active { transition: all var(--dur-fast); }
.player-enter-from, .player-leave-to { transform: translateY(100%); opacity: 0; }
</style>
