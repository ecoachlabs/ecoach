<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppButton from '@/components/ui/AppButton.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import StageIndicator from '@/components/modes/rise/StageIndicator.vue'
import TransformationSession from '@/components/modes/rise/TransformationSession.vue'
import BeforeAfterProof from '@/components/modes/rise/BeforeAfterProof.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const weakTopics = ref<TopicCaseDto[]>([])
const currentStage = ref<'rescue' | 'stabilize' | 'accelerate' | 'dominate'>('rescue')
const stageProgress = ref(35)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, topics] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 8),
    ])
    truth.value = t
    weakTopics.value = topics.filter(t => t.mastery_score < 4000)
  } catch {}
  loading.value = false
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <PageHeader title="Rise Mode" subtitle="Transform from struggling to dominating. One stage at a time." back-to="/student" />

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else>
      <!-- Stage Progress -->
      <div class="mb-6">
        <StageIndicator :current-stage="currentStage" />
        <AppProgress :value="stageProgress" size="md" color="warm" :glow="true" class="mt-3" />
      </div>

      <!-- Transformation Session -->
      <div class="mb-6">
        <TransformationSession :stage="currentStage" :topic-name="weakTopics[0]?.topic_name || 'Topic'" :session-number="3" :progress="stageProgress"
          @start="router.push('/student/rise/session/1')" />
      </div>

      <!-- Weakness Map -->
      <div v-if="weakTopics.length" class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Weakness Map</h3>
        <div class="space-y-2">
          <AppCard v-for="topic in weakTopics" :key="topic.topic_id" padding="sm" hover>
            <div class="flex items-center gap-3">
              <MasteryBadge :state="topic.mastery_state" size="sm" glow />
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium truncate" :style="{color:'var(--text)'}">{{ topic.topic_name }}</p>
                <p class="text-[11px]" :style="{color:'var(--text-3)'}">{{ topic.intervention_reason }}</p>
              </div>
              <AppBadge :color="topic.intervention_urgency === 'high' ? 'danger' : 'warm'" size="xs">{{ topic.intervention_mode }}</AppBadge>
            </div>
          </AppCard>
        </div>
      </div>

      <!-- Before/After Proof -->
      <BeforeAfterProof v-if="weakTopics[0]" :topic-name="weakTopics[0].topic_name"
        :before-score="1200" :after-score="weakTopics[0].mastery_score"
        before-date="Start" after-date="Now" :improvements="['Accuracy improving', 'Fewer careless errors']" />
    </template>
  </div>
</template>
