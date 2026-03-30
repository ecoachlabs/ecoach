<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, getPriorityTopics, type LearnerTruthDto, type TopicCaseDto } from '@/ipc/coach'
import { getGapDashboard, listPriorityGaps } from '@/ipc/gap'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import GapRing from '@/components/viz/GapRing.vue'
import GapSectionCard from '@/components/modes/knowledge-gap/GapSectionCard.vue'
import GapMapNode from '@/components/modes/knowledge-gap/GapMapNode.vue'
import AppCard from '@/components/ui/AppCard.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const gapTopics = ref<TopicCaseDto[]>([])
const gapDash = ref<any>(null)

const gapPercentage = ref(42)
const fixedThisWeek = ref(3)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [t, topics, gd] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getPriorityTopics(auth.currentAccount.id, 10),
      getGapDashboard(auth.currentAccount.id).catch(() => null),
    ])
    truth.value = t
    gapTopics.value = topics.filter(t => t.gap_score > 3000)
    gapDash.value = gd
  } catch {}
  loading.value = false
})

const sections = [
  { title: 'Fix Now', severity: 'critical' as const, count: 0, items: [] as { name: string; score: number }[] },
  { title: 'Slipping Areas', severity: 'slipping' as const, count: 0, items: [] as { name: string; score: number }[] },
  { title: 'Hidden Blockers', severity: 'hidden' as const, count: 0, items: [] as { name: string; score: number }[] },
  { title: 'Weak but Improving', severity: 'improving' as const, count: 0, items: [] as { name: string; score: number }[] },
  { title: 'Recent Shrinkage', severity: 'fixed' as const, count: 0, items: [] as { name: string; score: number }[] },
]

// Populate sections from gap topics
const populated = ref(false)
function populateSections() {
  if (populated.value) return
  gapTopics.value.forEach(t => {
    const item = { name: t.topic_name, score: t.gap_score }
    if (t.gap_score > 7000) { sections[0].items.push(item); sections[0].count++ }
    else if (t.gap_score > 5000) { sections[1].items.push(item); sections[1].count++ }
    else if (t.gap_score > 3000) { sections[3].items.push(item); sections[3].count++ }
  })
  populated.value = true
}

onMounted(() => setTimeout(populateSections, 500))
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <PageHeader title="Knowledge Gap" subtitle="See what you don't know yet. Close the gap to zero." back-to="/student">
      <template #actions>
        <GapRing :gap-percentage="gapPercentage" :size="64" />
      </template>
    </PageHeader>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else>
      <!-- Stats -->
      <div class="grid grid-cols-4 gap-3 mb-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--danger)'}">{{ gapPercentage }}%</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Gap</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--danger)'}">{{ sections[0].count }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Critical</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--warning)'}">{{ sections[1].count }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Slipping</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{color:'var(--success)'}">{{ fixedThisWeek }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{color:'var(--text-3)'}">Fixed</p>
        </AppCard>
      </div>

      <!-- Gap Sections -->
      <div class="grid grid-cols-2 gap-4 mb-6">
        <GapSectionCard v-for="sec in sections.filter(s => s.count > 0 || s.items.length > 0)" :key="sec.title"
          :title="sec.title" :severity="sec.severity" :count="sec.count" :items="sec.items" />
      </div>

      <!-- Gap Map Nodes -->
      <div v-if="gapTopics.length" class="mb-6">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Gap Map</h3>
        <div class="flex flex-wrap gap-2">
          <GapMapNode v-for="t in gapTopics" :key="t.topic_id"
            :name="t.topic_name" :gap-score="t.gap_score" :state="t.mastery_state"
            :is-blocker="t.requires_probe" :selected="false" />
        </div>
      </div>

      <div class="flex gap-3">
        <AppButton variant="primary" size="lg" @click="router.push('/student/knowledge-gap/scan')">Scan My Gaps →</AppButton>
        <AppButton variant="secondary" @click="router.push('/student/progress/mastery-map')">View Full Map</AppButton>
      </div>
    </template>
  </div>
</template>
