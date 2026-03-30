<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const router = useRouter()
const readiness = ref({ score: 4800, band: 'developing' })

const mockTypes = ref([
  { key: 'full', label: 'Full Mock', desc: 'Complete exam simulation with all sections', icon: '⊞', time: '2h 30m' },
  { key: 'topic', label: 'Topic Mock', desc: 'Focus on specific topics you need to strengthen', icon: '◎', time: '30-45m' },
  { key: 'mini', label: 'Mini Mock', desc: 'Quick 20-question check across all topics', icon: '◇', time: '15-20m' },
  { key: 'pressure', label: 'Pressure Mock', desc: 'Tighter timing. Harder questions. Exam pressure.', icon: '⚡', time: '45m' },
])

const mockHistory = ref([
  { id: 1, type: 'Full Mock', date: 'Mar 25', score: 62, change: +5 },
  { id: 2, type: 'Topic Mock', date: 'Mar 22', score: 71, change: +8 },
  { id: 3, type: 'Mini Mock', date: 'Mar 20', score: 55, change: -3 },
])
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Mock Centre</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Simulate real exam conditions. Discover where you truly stand.</p>
      </div>
      <ProgressRing :value="readiness.score" :max="10000" :size="64" :stroke-width="4" color="var(--accent)" label="Readiness" />
    </div>

    <div class="grid grid-cols-2 gap-4 mb-8">
      <AppCard v-for="mock in mockTypes" :key="mock.key" hover padding="lg" @click="router.push('/student/mock/setup')">
        <div class="flex items-start gap-3">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg" :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            {{ mock.icon }}
          </div>
          <div class="flex-1">
            <h3 class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ mock.label }}</h3>
            <p class="text-[11px] leading-relaxed mb-3" :style="{ color: 'var(--text-3)' }">{{ mock.desc }}</p>
            <AppBadge color="accent" size="xs">{{ mock.time }}</AppBadge>
          </div>
        </div>
      </AppCard>
    </div>

    <div>
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">Battle History</h3>
        <AppButton variant="ghost" size="sm" @click="router.push('/student/mock/history')">View All</AppButton>
      </div>
      <div class="space-y-2">
        <AppCard v-for="entry in mockHistory" :key="entry.id" padding="sm">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold tabular-nums"
              :class="entry.score >= 70 ? 'bg-emerald-50 text-emerald-600' : entry.score >= 50 ? 'bg-amber-50 text-amber-600' : 'bg-red-50 text-red-600'">
              {{ entry.score }}%
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ entry.type }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ entry.date }}</p>
            </div>
            <span class="text-xs font-semibold tabular-nums" :class="entry.change >= 0 ? 'text-emerald-600' : 'text-red-500'">
              {{ entry.change >= 0 ? '+' : '' }}{{ entry.change }}%
            </span>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
