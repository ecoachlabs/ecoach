<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  activeStation?: string
}>()

defineEmits<{ selectStation: [station: string]; play: []; stop: [] }>()

const stations = [
  { key: 'all', label: 'All Subjects', icon: '📚', desc: 'Complete curriculum audio' },
  { key: 'weak', label: 'My Weak Terms', icon: '⚠', desc: 'Focus on what you struggle with' },
  { key: 'hotspot', label: 'Exam Hotspots', icon: '🔥', desc: 'Most tested concepts' },
  { key: 'today', label: "Today's Review", icon: '📅', desc: 'Due for revision today' },
  { key: 'formulas', label: 'Formula Station', icon: '∑', desc: 'All formulas narrated' },
  { key: 'custom', label: 'Custom Playlist', icon: '🎵', desc: 'Your saved selections' },
]
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Audio Stations</h3>
    <div class="grid grid-cols-2 gap-3">
      <AppCard v-for="s in stations" :key="s.key" hover padding="md"
        :class="activeStation === s.key ? 'ring-2 ring-[var(--accent)]' : ''"
        @click="$emit('selectStation', s.key)">
        <div class="flex items-start gap-2.5">
          <span class="text-xl">{{ s.icon }}</span>
          <div>
            <p class="text-xs font-semibold" :style="{ color: 'var(--text)' }">{{ s.label }}</p>
            <p class="text-[9px]" :style="{ color: 'var(--text-3)' }">{{ s.desc }}</p>
          </div>
        </div>
      </AppCard>
    </div>
    <p class="text-xs mt-4 text-center" :style="{ color: 'var(--text-3)' }">
      Audio plays continuously in the background. Listen while the screen is off.
    </p>
  </div>
</template>
