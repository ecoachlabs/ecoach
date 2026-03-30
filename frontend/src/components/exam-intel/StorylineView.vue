<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  topicName: string
  years: number
  narrative: string
  chapters: { year: string; title: string; description: string; highlight?: boolean }[]
}>()
</script>

<template>
  <div>
    <h3 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">{{ topicName }}: A {{ years }}-Year Story</h3>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-2)' }">{{ narrative }}</p>

    <div class="space-y-0">
      <div v-for="(ch, i) in chapters" :key="i" class="flex gap-4">
        <div class="flex flex-col items-center shrink-0">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center text-xs font-bold"
            :class="ch.highlight ? 'bg-[var(--accent)] text-white' : ''"
            :style="!ch.highlight ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
            {{ ch.year.slice(-2) }}
          </div>
          <div v-if="i < chapters.length - 1" class="w-px flex-1 min-h-[20px]" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div class="pb-5 flex-1">
          <p class="text-sm font-semibold" :style="{ color: ch.highlight ? 'var(--accent)' : 'var(--text)' }">{{ ch.title }}</p>
          <p class="text-xs mt-0.5" :style="{ color: 'var(--text-2)' }">{{ ch.description }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
