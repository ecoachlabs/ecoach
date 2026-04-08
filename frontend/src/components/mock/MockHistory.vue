<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import TrendLine from '@/components/viz/TrendLine.vue'

defineProps<{
  sessions: { id: number; type: string; date: string; score: number; change: number; questionCount: number }[]
}>()

defineEmits<{ viewReview: [sessionId: number] }>()
</script>

<template>
  <div>
    <!-- Trend chart -->
    <div v-if="sessions.length >= 3" class="mb-6 p-4 rounded-[var(--radius-lg)]" :style="{backgroundColor:'var(--card-bg)'}">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Score Trend</h3>
      <TrendLine :data="sessions.map(s => ({ label: new Date(s.date).toLocaleDateString('en-GB',{day:'numeric',month:'short'}), value: s.score }))" :max-value="10000" :height="100" color="var(--accent)" />
    </div>

    <!-- Session list -->
    <div class="space-y-2">
      <AppCard v-for="s in sessions" :key="s.id" padding="sm" hover @click="$emit('viewReview', s.id)">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold tabular-nums"
            :class="s.score >= 7000 ? 'bg-emerald-50 text-emerald-600' : s.score >= 4000 ? 'bg-amber-50 text-amber-600' : 'bg-red-50 text-red-600'">
            {{ (s.score/100).toFixed(0) }}%
          </div>
          <div class="flex-1">
            <p class="text-sm font-medium" :style="{color:'var(--text)'}">{{ s.type }}</p>
            <p class="text-[10px]" :style="{color:'var(--text-3)'}">{{ s.date }} · {{ s.questionCount }} questions</p>
          </div>
          <span class="text-xs font-semibold tabular-nums" :class="s.change >= 0 ? 'text-emerald-600' : 'text-red-500'">
            {{ s.change >= 0 ? '+' : '' }}{{ (s.change/100).toFixed(0) }}%
          </span>
        </div>
      </AppCard>
    </div>
  </div>
</template>
