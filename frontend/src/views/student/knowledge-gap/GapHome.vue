<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const gapTopics = ref<TopicCaseDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, topics] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 10),
    ])
    truth.value = t
    gapTopics.value = topics.filter(t => t.gap_score > 3000)
  } catch {}
  loading.value = false
})

const gapPercentage = ref(42)
const fixedThisWeek = ref(3)
const criticalCount = ref(4)
const slippingCount = ref(2)
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Knowledge Gap</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">See what you don't know yet. Close the gap to zero.</p>
      </div>
      <ProgressRing :value="100 - gapPercentage" :size="72" :stroke-width="5" color="var(--accent)" label="Known" />
    </div>

    <!-- Gap Stats -->
    <div class="grid grid-cols-4 gap-3 mb-6">
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ gapPercentage }}%</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Gap</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ criticalCount }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Critical</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--warning)' }">{{ slippingCount }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Slipping</p>
      </AppCard>
      <AppCard padding="md" class="text-center">
        <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ fixedThisWeek }}</p>
        <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Fixed</p>
      </AppCard>
    </div>

    <!-- Priority Gaps -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Fix Now</h3>
      <div class="space-y-2">
        <AppCard v-for="topic in gapTopics" :key="topic.topic_id" padding="sm" hover>
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold"
              :class="topic.gap_score > 6000 ? 'bg-red-50 text-red-600' : 'bg-amber-50 text-amber-600'">
              {{ (topic.gap_score / 100).toFixed(0) }}%
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
            </div>
            <AppBadge :color="topic.gap_score > 6000 ? 'danger' : 'warm'" size="xs">{{ topic.intervention_mode }}</AppBadge>
          </div>
        </AppCard>
      </div>
    </div>

    <div class="flex gap-3">
      <AppButton variant="primary" size="lg" @click="router.push('/student/knowledge-gap/scan')">Scan My Gaps →</AppButton>
      <AppButton variant="secondary" @click="router.push('/student/progress/mastery-map')">View Full Map</AppButton>
    </div>
  </div>
</template>
