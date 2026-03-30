<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  stage: number  // 0-3 (Hook, Engage, Learn, Transform)
  progress: number // 0-100 within current stage
}>()

const stages = ['Hook', 'Engage', 'Learn', 'Transform']
const overallProgress = computed(() => (props.stage * 25) + (props.progress / 4))
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-2 text-[10px] font-semibold" :style="{ color: 'var(--text-3)' }">
      <span>Disengaged</span>
      <span :style="{ color: 'var(--success)' }">Transformed</span>
    </div>
    <div class="h-3 rounded-full overflow-hidden relative" :style="{ backgroundColor: 'var(--border-soft)' }">
      <div class="h-full rounded-full transition-all bg-gradient-to-r from-amber-500 via-teal-500 to-emerald-500"
        :style="{ width: overallProgress + '%', transitionDuration: 'var(--dur-slow)' }" />
      <!-- Stage markers -->
      <div v-for="i in 3" :key="i" class="absolute top-0 bottom-0 w-px" :style="{ left: (i * 25) + '%', backgroundColor: 'white', opacity: 0.5 }" />
    </div>
    <div class="flex justify-between mt-1">
      <span v-for="(s, i) in stages" :key="s" class="text-[8px] font-medium"
        :style="{ color: i <= stage ? 'var(--accent)' : 'var(--text-3)' }">{{ s }}</span>
    </div>
  </div>
</template>
