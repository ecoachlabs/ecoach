<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  centerConcept: string
  prerequisites: { name: string; state: string }[]
  related: { name: string; relationship: string }[]
  advanced: { name: string }[]
  confusedWith: { name: string }[]
}>()

defineEmits<{ navigate: [conceptName: string] }>()
</script>

<template>
  <div class="relative min-h-[300px] flex items-center justify-center">
    <!-- Prerequisites (left) -->
    <div class="absolute left-4 top-1/2 -translate-y-1/2 space-y-2">
      <p class="text-[8px] font-semibold uppercase mb-1" :style="{color:'var(--text-3)'}">Prerequisites</p>
      <button v-for="p in prerequisites" :key="p.name"
        class="block px-2.5 py-1.5 rounded-[var(--radius-sm)] text-[10px] font-medium border transition-all hover:scale-105"
        :style="{borderColor:'var(--card-border)',backgroundColor:'var(--card-bg)',color:'var(--text)'}"
        @click="$emit('navigate', p.name)">
        {{ p.name }}
      </button>
    </div>

    <!-- Center concept -->
    <div class="px-6 py-4 rounded-2xl border-2 text-center z-10"
      :style="{borderColor:'var(--accent)',backgroundColor:'var(--accent-light)'}">
      <p class="font-display text-base font-bold" :style="{color:'var(--accent)'}">{{ centerConcept }}</p>
    </div>

    <!-- Related (around) -->
    <div class="absolute top-4 left-1/2 -translate-x-1/2 flex gap-2">
      <button v-for="r in related" :key="r.name"
        class="px-2 py-1 rounded text-[9px] font-medium border hover:scale-105 transition-all"
        :style="{borderColor:'var(--gold)',backgroundColor:'var(--gold-light)',color:'var(--gold)'}"
        @click="$emit('navigate', r.name)">
        {{ r.name }}
      </button>
    </div>

    <!-- Advanced (right) -->
    <div class="absolute right-4 top-1/2 -translate-y-1/2 space-y-2">
      <p class="text-[8px] font-semibold uppercase mb-1" :style="{color:'var(--text-3)'}">Advanced</p>
      <button v-for="a in advanced" :key="a.name"
        class="block px-2.5 py-1.5 rounded-[var(--radius-sm)] text-[10px] font-medium border transition-all hover:scale-105"
        :style="{borderColor:'var(--card-border)',backgroundColor:'var(--card-bg)',color:'var(--text)'}"
        @click="$emit('navigate', a.name)">
        {{ a.name }}
      </button>
    </div>

    <!-- Confused with (bottom, highlighted red) -->
    <div v-if="confusedWith.length" class="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-2">
      <button v-for="c in confusedWith" :key="c.name"
        class="px-2 py-1 rounded text-[9px] font-medium border hover:scale-105 transition-all"
        :style="{borderColor:'var(--danger)',backgroundColor:'var(--danger-light)',color:'var(--danger)'}"
        @click="$emit('navigate', c.name)">
        ⚠ {{ c.name }}
      </button>
    </div>
  </div>
</template>
