<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getMockReport, type MockReportDto } from '@/ipc/mock'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const route = useRoute()
const router = useRouter()
const mockSessionId = computed(() => Number(route.params.id))

const loading = ref(true)
const report = ref<MockReportDto | null>(null)
const error = ref('')

onMounted(async () => {
  try {
    report.value = await getMockReport(mockSessionId.value)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load report'
  }
  loading.value = false
})

function gradeColor(grade: string): string {
  if (['A1', 'B2', 'B3'].includes(grade)) return 'success'
  if (['C4', 'C5', 'C6'].includes(grade)) return 'gold'
  return 'danger'
}

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}m ${s}s`
}
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Mock Review</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--ink-muted)' }">See how you performed in this exam session.</p>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-4">
      <div class="h-40 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div class="grid grid-cols-3 gap-3">
        <div v-for="i in 3" :key="i" class="h-24 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--warm)' }">{{ error }}</p>
      <AppButton variant="secondary" @click="router.push('/student/mock')">Back</AppButton>
    </div>

    <template v-else-if="report">
      <!-- Score Hero -->
      <AppCard padding="lg" glow="accent" class="mb-6">
        <div class="flex items-center gap-6">
          <ProgressRing
            :value="report.accuracy_bp"
            :max="10000"
            :size="90"
            :stroke-width="6"
            :color="report.accuracy_bp >= 7000 ? 'var(--accent)' : report.accuracy_bp >= 5000 ? 'var(--gold)' : 'var(--warm)'"
            label="Score"
          />
          <div class="flex-1">
            <div class="flex items-center gap-3 mb-2">
              <h2 class="font-display text-3xl font-bold" :style="{ color: 'var(--ink)' }">
                {{ report.percentage.toFixed(1) }}%
              </h2>
              <AppBadge :color="(gradeColor(report.grade) as any)" size="lg">{{ report.grade }}</AppBadge>
            </div>
            <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
              Score: {{ report.total_score }} / {{ report.max_score }}
            </p>
            <p v-if="report.improvement_direction" class="text-sm mt-1"
              :style="{ color: report.improvement_direction === 'up' ? 'var(--accent)' : 'var(--warm)' }">
              {{ report.improvement_direction === 'up' ? '↑ Improving' : '↓ Dropped' }} since last mock
            </p>
          </div>
        </div>
      </AppCard>

      <!-- Stats Grid -->
      <div class="grid grid-cols-3 gap-3 mb-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ report.questions_answered }}</p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--ink-muted)' }">Answered</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--warm)' }">{{ report.questions_unanswered }}</p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--ink-muted)' }">Skipped</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">
            {{ formatTime(report.time_used_seconds) }}
          </p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--ink-muted)' }">Time Used</p>
        </AppCard>
      </div>

      <!-- Actions -->
      <div class="flex flex-wrap items-center gap-3">
        <AppButton variant="primary" @click="router.push('/student/mock')">Back to Mock Centre</AppButton>
        <AppButton variant="secondary" @click="router.push('/student/mistakes')">Review Mistakes</AppButton>
        <AppButton variant="ghost" size="sm" @click="router.push('/student/mock/setup')">Try Again</AppButton>
      </div>
    </template>
  </div>
</template>
