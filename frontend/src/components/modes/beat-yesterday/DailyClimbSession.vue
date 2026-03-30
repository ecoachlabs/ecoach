<script setup lang="ts">
import { ref, computed } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

defineProps<{
  currentBlock: number // 0-3
  blocks: { name: string; duration: string; icon: string; questionCount: number }[]
}>()

defineEmits<{ complete: [] }>()

const blockMoods = ['Warm, easy wins', 'Push beyond yesterday', '60-second rapid fire', 'End on a high note']
const blockColors = ['var(--success)', 'var(--accent)', 'var(--warm)', 'var(--gold)']
</script>

<template>
  <div>
    <!-- Block progress -->
    <div class="flex gap-1 mb-4">
      <div v-for="(block, i) in blocks" :key="i"
        class="flex-1 h-2 rounded-full transition-all"
        :style="{ backgroundColor: i <= currentBlock ? blockColors[i] : 'var(--border-soft)', transitionDuration: 'var(--dur-normal)' }" />
    </div>

    <!-- Current block info -->
    <AppCard padding="md" class="mb-4">
      <div class="flex items-center gap-3">
        <span class="text-2xl">{{ blocks[currentBlock]?.icon || '●' }}</span>
        <div class="flex-1">
          <h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ blocks[currentBlock]?.name }}</h3>
          <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ blockMoods[currentBlock] }}</p>
        </div>
        <AppBadge color="muted" size="xs">{{ blocks[currentBlock]?.duration }}</AppBadge>
      </div>
    </AppCard>

    <!-- Question area (slot) -->
    <slot />
  </div>
</template>
