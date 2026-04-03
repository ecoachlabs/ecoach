<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const topics = ref<TopicCaseDto[]>([])

// Map readiness band to approximate ring score
const bandToScore: Record<string, number> = { strong: 7500, developing: 4800, weak: 2800, critical: 1500 }

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, topicList] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 20),
    ])
    truth.value = t
    topics.value = topicList
  } catch (e) {
    console.error('Failed to load progress:', e)
  }
  loading.value = false
})

function readinessScore(): number {
  if (!truth.value) return 0
  return bandToScore[truth.value.overall_readiness_band] ?? 4800
}

function readinessColor(): string {
  const band = truth.value?.overall_readiness_band ?? 'developing'
  if (band === 'strong') return 'var(--success)'
  if (band === 'developing') return 'var(--accent)'
  return 'var(--danger)'
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Progress</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Your academic readiness at a glance.</p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/mastery-map')">◐ Mastery Map</AppButton>
        <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/analytics')">Analytics</AppButton>
      </div>
    </div>

    <!-- Stats Row -->
    <div v-if="loading" class="grid grid-cols-5 gap-3 mb-8">
      <div v-for="i in 5" :key="i" class="h-20 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>
    <div v-else class="grid grid-cols-5 gap-3 mb-8">
      <AppCard padding="md" class="text-center">
        <ProgressRing
          :value="readinessScore()"
          :max="10000"
          :size="56"
          :stroke-width="3.5"
          :color="readinessColor()"
        />
        <p class="text-[10px] font-medium mt-2 uppercase" :style="{ color: 'var(--text-3)' }">Readiness</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ truth?.topic_count ?? 0 }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Topics</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ truth?.memory_count ?? 0 }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Memories</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">{{ truth?.due_memory_count ?? 0 }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Due Reviews</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ truth?.pending_review_count ?? 0 }}</p>
        <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Pending</p>
      </AppCard>
    </div>

    <!-- Topic Mastery -->
    <div>
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Topic Mastery</h3>

      <div v-if="loading" class="space-y-2">
        <div v-for="i in 6" :key="i" class="h-12 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else-if="topics.length" class="space-y-2">
        <AppCard v-for="topic in topics" :key="topic.topic_id" padding="sm" hover
          @click="router.push('/student/practice')">
          <div class="flex items-center gap-3">
            <MasteryBadge :state="topic.mastery_state" size="sm" glow />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
            </div>
            <div class="w-32">
              <AppProgress
                :value="topic.mastery_score"
                :max="10000"
                size="sm"
                :color="topic.mastery_score >= 6000 ? 'success' : topic.mastery_score >= 3000 ? 'gold' : 'danger'"
              />
            </div>
            <span class="text-xs tabular-nums font-medium w-10 text-right" :style="{ color: 'var(--text-3)' }">
              {{ (topic.mastery_score / 100).toFixed(0) }}%
            </span>
          </div>
        </AppCard>
      </div>

      <AppCard v-else padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">
          Complete your diagnostic to see topic mastery data.
        </p>
        <AppButton variant="primary" size="sm" class="mt-4" @click="router.push('/student/diagnostic')">
          Start Diagnostic
        </AppButton>
      </AppCard>
    </div>
  </div>
</template>
