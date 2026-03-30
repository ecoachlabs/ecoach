<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const weakTopics = ref<TopicCaseDto[]>([])

const currentStage = ref<'rescue' | 'stabilize' | 'accelerate' | 'dominate'>('rescue')
const stageProgress = ref(35)

const stages = [
  { key: 'rescue', label: 'Rescue', desc: 'Find and fix the foundations', icon: '🛟', color: 'var(--danger)' },
  { key: 'stabilize', label: 'Stabilize', desc: 'Build consistent understanding', icon: '⚖', color: 'var(--warning)' },
  { key: 'accelerate', label: 'Accelerate', desc: 'Push into timed pressure', icon: '🚀', color: 'var(--accent)' },
  { key: 'dominate', label: 'Dominate', desc: 'Master everything under exam conditions', icon: '👑', color: 'var(--gold)' },
]

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
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Rise Mode</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Transform from struggling to dominating. One stage at a time.</p>
    </div>

    <!-- Stage Progress -->
    <AppCard padding="lg" class="mb-6">
      <div class="flex items-center gap-2 mb-4">
        <span v-for="(stage, i) in stages" :key="stage.key"
          class="flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full"
          :class="currentStage === stage.key ? 'text-white' : ''"
          :style="{
            backgroundColor: currentStage === stage.key ? stage.color : 'var(--primary-light)',
            color: currentStage === stage.key ? 'white' : 'var(--text-3)',
          }">
          {{ stage.icon }} {{ stage.label }}
        </span>
      </div>
      <AppProgress :value="stageProgress" size="md" color="warm" :glow="true" />
      <p class="text-xs mt-2" :style="{ color: 'var(--text-3)' }">
        Stage: {{ stages.find(s => s.key === currentStage)?.desc }}
      </p>
    </AppCard>

    <!-- Weakness Map -->
    <div v-if="weakTopics.length" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Weakness Map</h3>
      <div class="space-y-2">
        <AppCard v-for="topic in weakTopics" :key="topic.topic_id" padding="sm" hover>
          <div class="flex items-center gap-3">
            <MasteryBadge :state="topic.mastery_state" size="sm" glow />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ topic.intervention_reason }}</p>
            </div>
            <AppBadge :color="topic.intervention_urgency === 'high' ? 'danger' : 'warm'" size="xs">
              {{ topic.intervention_mode }}
            </AppBadge>
          </div>
        </AppCard>
      </div>
    </div>

    <!-- CTA -->
    <AppButton variant="warm" size="lg">Start Transformation Session →</AppButton>
  </div>
</template>
