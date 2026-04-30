<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'

defineProps<{
  activeStation?: string
}>()

defineEmits<{ selectStation: [station: string]; play: []; stop: [] }>()

const stations = [
  { key: 'all', label: 'Topic Radio', icon: 'TR', desc: 'Live topic-focused explanation loops' },
  { key: 'weak', label: 'Weakness Radio', icon: 'WR', desc: 'Repair the terms under pressure right now' },
  { key: 'hotspot', label: 'Exam Hotspots', icon: 'EH', desc: 'Most tested ideas with the highest relevance' },
  { key: 'today', label: 'Today Review', icon: 'TD', desc: 'Return loop for entries slipping out of recall' },
  { key: 'formulas', label: 'Formula Radio', icon: 'FR', desc: 'Narrated formulas and setup cues' },
  { key: 'custom', label: 'Bundle Route', icon: 'BR', desc: 'Follow the next structured glossary bundle' },
]
</script>

<template>
  <div>
    <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Audio Stations</h3>
    <div class="grid grid-cols-2 gap-3">
      <AppCard
        v-for="station in stations"
        :key="station.key"
        hover
        padding="md"
        :class="activeStation === station.key ? 'ring-2 ring-[var(--accent)]' : ''"
        @click="$emit('selectStation', station.key)"
      >
        <div class="flex items-start gap-2.5">
          <span class="inline-flex h-7 min-w-7 items-center justify-center rounded-md bg-[var(--primary-light)] px-1 text-[10px] font-bold uppercase tracking-wide text-[var(--primary)]">
            {{ station.icon }}
          </span>
          <div>
            <p class="text-xs font-semibold" :style="{ color: 'var(--text)' }">{{ station.label }}</p>
            <p class="text-[9px]" :style="{ color: 'var(--text-3)' }">{{ station.desc }}</p>
          </div>
        </div>
      </AppCard>
    </div>
    <p class="mt-4 text-center text-xs" :style="{ color: 'var(--text-3)' }">
      Start a real queue and move through the live audio segments one by one.
    </p>
  </div>
</template>
