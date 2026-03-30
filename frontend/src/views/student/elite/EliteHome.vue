<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import AppButton from '@/components/ui/AppButton.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import EliteIdentityPanel from '@/components/modes/elite/EliteIdentityPanel.vue'
import EliteSessionArena from '@/components/modes/elite/EliteSessionArena.vue'
import EliteTierProgress from '@/components/modes/elite/EliteTierProgress.vue'
import TopicDominationBoard from '@/components/modes/elite/TopicDominationBoard.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const topics = ref<TopicCaseDto[]>([])
const selectedSession = ref<string | null>(null)

const tier = ref('Core')
const eps = ref(6240)
const streak = ref(7)

const identityDimensions = ref([
  { label: 'Precision', score: 7200, icon: '◎', trend: 3 },
  { label: 'Speed', score: 5800, icon: '⚡', trend: -1 },
  { label: 'Depth', score: 6100, icon: '◈', trend: 5 },
  { label: 'Endurance', score: 5400, icon: '∞', trend: 2 },
  { label: 'Trap Sense', score: 4900, icon: '⚠', trend: 0 },
  { label: 'Independence', score: 6800, icon: '★', trend: 4 },
])

const dominationTopics = ref([
  { name: 'Number Operations', status: 'Dominant', dominationScore: 8500, accuracy: 9200, speed: 7800, trapResistance: 8100 },
  { name: 'Fractions', status: 'Accurate, trap-vulnerable', dominationScore: 6200, accuracy: 7800, speed: 5400, trapResistance: 3900 },
  { name: 'Algebra', status: 'Fast but inconsistent', dominationScore: 5800, accuracy: 5200, speed: 8100, trapResistance: 5500 },
])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, tc] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 5),
    ])
    truth.value = t
    topics.value = tc
  } catch {}
  loading.value = false
})

function startSession() {
  if (selectedSession.value) router.push('/student/elite/session/1')
}
</script>

<template>
  <div class="min-h-screen" data-mode="elite">
    <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
      <PageHeader title="Elite Mode" subtitle="Push your performance limits within the syllabus." back-to="/student">
        <template #actions>
          <div class="flex items-center gap-3">
            <EliteTierProgress :current-tier="tier" next-tier="Prime" :progress-to-next="62" :eps="eps" />
            <ProgressRing :value="eps" :max="10000" :size="56" :stroke-width="3.5" color="var(--primary)" label="EPS" />
          </div>
        </template>
      </PageHeader>

      <div v-if="loading" class="space-y-4">
        <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
      </div>

      <template v-else>
        <!-- Identity Panel -->
        <div class="mb-6">
          <EliteIdentityPanel :dimensions="identityDimensions" />
        </div>

        <!-- Session Arena -->
        <div class="mb-6">
          <EliteSessionArena v-model:selected-session="selectedSession" @start="startSession" />
        </div>

        <!-- Topic Domination -->
        <div class="mb-6">
          <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Topic Domination</h3>
          <TopicDominationBoard :topics="dominationTopics" @select-topic="(name) => router.push('/student/elite/insights')" />
        </div>

        <!-- Quick Actions -->
        <div class="flex flex-wrap gap-2">
          <AppButton variant="secondary" size="sm" @click="router.push('/student/elite/records')">★ Records Wall</AppButton>
          <AppButton variant="secondary" size="sm" @click="router.push('/student/elite/insights')">◐ Insights</AppButton>
        </div>
      </template>
    </div>
  </div>
</template>
