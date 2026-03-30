<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const router = useRouter()

const examDates = ref([
  { id: 1, name: 'Internal Mock', date: '2026-04-15', daysLeft: 17, subject: 'All', type: 'mock' },
  { id: 2, name: 'Mid-Term Exam', date: '2026-05-10', daysLeft: 42, subject: 'All', type: 'exam' },
  { id: 3, name: 'BECE Final', date: '2026-07-15', daysLeft: 108, subject: 'All', type: 'final' },
])

const weeklyPlan = ref([
  { day: 'Mon', subject: 'Mathematics', focus: 'Fractions repair', duration: '45 min' },
  { day: 'Tue', subject: 'English', focus: 'Comprehension', duration: '30 min' },
  { day: 'Wed', subject: 'Science', focus: 'Biology recall', duration: '40 min' },
  { day: 'Thu', subject: 'Mathematics', focus: 'Algebra practice', duration: '45 min' },
  { day: 'Fri', subject: 'All', focus: 'Mixed review', duration: '30 min' },
])

const currentPhase = ref('Build')
const phases = ['Foundation', 'Build', 'Integration', 'Timed', 'Mock', 'Revision', 'Last-Mile']
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Academic Calendar</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Your exam timeline and preparation plan.</p>

    <!-- Exam Timeline -->
    <div class="mb-8">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Exam Timeline</h3>
      <div class="space-y-2">
        <AppCard v-for="exam in examDates" :key="exam.id" padding="md">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl flex flex-col items-center justify-center"
              :class="exam.daysLeft <= 30 ? 'bg-red-50 text-red-600' : exam.daysLeft <= 60 ? 'bg-amber-50 text-amber-600' : 'bg-teal-50 text-teal-600'">
              <span class="text-lg font-bold tabular-nums leading-none">{{ exam.daysLeft }}</span>
              <span class="text-[8px] font-medium uppercase">days</span>
            </div>
            <div class="flex-1">
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ exam.name }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ exam.date }} · {{ exam.subject }}</p>
            </div>
            <AppBadge :color="exam.type === 'final' ? 'danger' : exam.type === 'exam' ? 'warm' : 'accent'" size="xs">
              {{ exam.type }}
            </AppBadge>
          </div>
        </AppCard>
      </div>
    </div>

    <!-- Current Phase -->
    <div class="mb-8">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Preparation Phase</h3>
      <div class="flex gap-1">
        <div v-for="phase in phases" :key="phase"
          class="flex-1 py-2 text-center text-[10px] font-medium rounded-lg transition-all"
          :class="phase === currentPhase ? 'bg-[var(--accent)] text-white' : ''"
          :style="phase !== currentPhase ? { backgroundColor: 'var(--primary-light)', color: 'var(--text-3)' } : {}">
          {{ phase }}
        </div>
      </div>
    </div>

    <!-- Weekly Plan -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">This Week</h3>
      <div class="space-y-2">
        <AppCard v-for="day in weeklyPlan" :key="day.day" padding="sm">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center text-xs font-bold"
              :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">{{ day.day }}</div>
            <div class="flex-1">
              <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ day.focus }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ day.subject }}</p>
            </div>
            <AppBadge color="muted" size="xs">{{ day.duration }}</AppBadge>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
