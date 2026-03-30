<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ExamCountdown from '@/components/viz/ExamCountdown.vue'

defineProps<{
  exams: { name: string; date: string; readinessBand?: string }[]
}>()
</script>

<template>
  <AppCard padding="md">
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Exam Horizon</h3>
    <div class="space-y-2">
      <ExamCountdown
        v-for="exam in exams"
        :key="exam.name"
        :exam-date="exam.date"
        :exam-name="exam.name"
        :readiness-band="exam.readinessBand"
      />
    </div>
    <p v-if="!exams.length" class="text-xs" :style="{ color: 'var(--text-3)' }">No upcoming exams set. Add dates in Calendar.</p>
  </AppCard>
</template>
