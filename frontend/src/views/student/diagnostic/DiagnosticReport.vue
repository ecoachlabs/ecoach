<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { completeDiagnosticAndSync, getDiagnosticReport, type DiagnosticCompletionSyncDto, type TopicAnalyticsDto } from '@/ipc/diagnostic'
import PageHeader from '@/components/layout/PageHeader.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppTabs from '@/components/ui/AppTabs.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import { usePdf } from '@/composables/usePdf'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const { generating, exportToPdf } = usePdf()

const diagnosticId = computed(() => Number(route.params.id))
const loading = ref(true)
const syncData = ref<DiagnosticCompletionSyncDto | null>(null)
const topics = ref<TopicAnalyticsDto[]>([])
const activeTab = ref('overview')

const tabs = [
  { key: 'overview', label: 'Overview' },
  { key: 'topics', label: 'Topics' },
  { key: 'actions', label: 'Next Steps' },
]

onMounted(async () => {
  try {
    // Try to get the sync data (may already exist if diagnostic was just completed)
    try {
      syncData.value = await completeDiagnosticAndSync(diagnosticId.value)
      topics.value = syncData.value.analytics
    } catch {
      // If already synced, just get the report
      topics.value = await getDiagnosticReport(diagnosticId.value)
    }
  } catch (e) {
    console.error('Failed to load diagnostic report:', e)
  }
  loading.value = false
})

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}

const overallReadiness = computed(() => syncData.value?.overall_readiness ?? 0)
const readinessBand = computed(() => syncData.value?.readiness_band ?? 'developing')

const bandColor = computed(() => {
  const b = readinessBand.value
  if (b === 'strong') return 'success'
  if (b === 'developing') return 'gold'
  return 'danger'
})

const classificationColors: Record<string, string> = {
  solid: 'var(--success)',
  developing: 'var(--gold)',
  weak: 'var(--warning)',
  critical: 'var(--danger)',
  fragile: 'var(--warm)',
}

function handleExport() {
  exportToPdf('diagnostic-report', `diagnostic-report-${diagnosticId.value}.pdf`)
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto">
    <PageHeader
      title="Diagnostic Report"
      subtitle="Your comprehensive academic profile from this assessment."
      back-to="/student/diagnostic"
    >
      <template #actions>
        <AppButton variant="secondary" size="sm" :loading="generating" @click="handleExport">
          Export PDF
        </AppButton>
        <AppButton variant="primary" size="sm" @click="router.push('/student')">
          Go to Coach Hub
        </AppButton>
      </template>
    </PageHeader>

    <!-- Loading -->
    <div v-if="loading" class="space-y-4 mt-6">
      <div v-for="i in 4" :key="i" class="h-24 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Report Content -->
    <div v-else id="diagnostic-report" class="mt-6 reveal-stagger">

      <!-- Readiness Hero -->
      <AppCard v-if="syncData" padding="lg" glow="accent" class="mb-6">
        <div class="flex items-center gap-8 flex-wrap">
          <ProgressRing
            :value="overallReadiness"
            :max="10000"
            :size="110"
            :stroke-width="7"
            :color="overallReadiness >= 7000 ? 'var(--success)' : overallReadiness >= 4000 ? 'var(--gold)' : 'var(--danger)'"
            label="Readiness"
          />
          <div class="flex-1">
            <p class="text-xs font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--text-3)' }">
              Overall Readiness
            </p>
            <h2 class="font-display text-3xl font-bold mb-2" :style="{ color: 'var(--text)' }">
              {{ formatBp(overallReadiness) }}
            </h2>
            <AppBadge :color="(bandColor as any)" size="md">{{ readinessBand }}</AppBadge>
            <p class="text-sm mt-3" :style="{ color: 'var(--text-2)' }">
              <template v-if="readinessBand === 'strong'">
                You are performing strongly. Keep up the momentum and refine your weak areas.
              </template>
              <template v-else-if="readinessBand === 'developing'">
                Good foundation. Your coach has identified targeted areas to improve your score.
              </template>
              <template v-else>
                Significant gaps detected. Your coach will build a focused recovery plan for you.
              </template>
            </p>
          </div>
        </div>
      </AppCard>

      <!-- Stats row -->
      <div v-if="topics.length" class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ topics.length }}</p>
          <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Topics Assessed</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">
            {{ topics.filter(t => t.classification === 'solid').length }}
          </p>
          <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Solid</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">
            {{ topics.filter(t => t.classification === 'developing').length }}
          </p>
          <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Developing</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">
            {{ topics.filter(t => ['weak', 'critical'].includes(t.classification)).length }}
          </p>
          <p class="text-[10px] font-medium uppercase mt-1" :style="{ color: 'var(--text-3)' }">Needs Work</p>
        </AppCard>
      </div>

      <!-- Tabs -->
      <AppTabs :tabs="tabs" v-model="activeTab" class="mb-6" />

      <!-- Overview tab -->
      <div v-if="activeTab === 'overview'">
        <div v-if="syncData?.top_hypotheses?.length" class="space-y-3 mb-6">
          <h3 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">
            Coach's Key Findings
          </h3>
          <AppCard
            v-for="h in syncData.top_hypotheses.slice(0, 4)"
            :key="h.topic_id"
            padding="md"
          >
            <div class="flex items-start gap-3">
              <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
                :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
                {{ h.confidence_score >= 7000 ? '!' : '?' }}
              </div>
              <div class="flex-1">
                <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ h.topic_name }}</p>
                <p class="text-xs mt-0.5" :style="{ color: 'var(--text-3)' }">
                  {{ h.hypothesis_code.replace(/_/g, ' ') }}
                </p>
                <p class="text-xs mt-1" :style="{ color: 'var(--accent)' }">→ {{ h.recommended_action }}</p>
              </div>
              <div class="text-right">
                <p class="text-xs font-medium" :style="{ color: 'var(--text-3)' }">
                  Confidence: {{ formatBp(h.confidence_score) }}
                </p>
              </div>
            </div>
          </AppCard>
        </div>

        <!-- Weak topics summary -->
        <div v-if="topics.filter(t => t.mastery_score < 4000).length">
          <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
            Priority Areas to Fix
          </h3>
          <div class="space-y-2">
            <AppCard
              v-for="t in topics.filter(t => t.mastery_score < 4000).slice(0, 5)"
              :key="t.topic_id"
              padding="sm"
            >
              <div class="flex items-center gap-3">
                <MasteryBadge :state="t.classification" size="sm" glow />
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ t.topic_name }}</p>
                  <p v-if="t.weakness_type" class="text-[11px]" :style="{ color: 'var(--text-3)' }">
                    {{ t.weakness_type.replace(/_/g, ' ') }}
                  </p>
                </div>
                <AppProgress
                  :value="t.mastery_score"
                  :max="10000"
                  size="sm"
                  color="danger"
                  class="w-20"
                />
              </div>
            </AppCard>
          </div>
        </div>
      </div>

      <!-- Topics tab -->
      <div v-if="activeTab === 'topics'" class="space-y-2">
        <AppCard v-for="topic in topics" :key="topic.topic_id" padding="sm">
          <div class="flex items-center gap-3">
            <MasteryBadge :state="topic.classification" size="sm" glow />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
              <div class="flex flex-wrap gap-3 mt-1 text-[10px]" :style="{ color: 'var(--text-3)' }">
                <span>Mastery: {{ formatBp(topic.mastery_score) }}</span>
                <span>Confidence: {{ formatBp(topic.confidence_score) }}</span>
                <span>Endurance: {{ formatBp(topic.endurance_score) }}</span>
                <span v-if="topic.weakness_type">Issue: {{ topic.weakness_type.replace(/_/g, ' ') }}</span>
              </div>
            </div>
            <div class="w-20 shrink-0">
              <AppProgress
                :value="topic.mastery_score"
                :max="10000"
                size="sm"
                :color="topic.mastery_score >= 6000 ? 'success' : topic.mastery_score >= 3000 ? 'gold' : 'danger'"
              />
            </div>
          </div>
        </AppCard>
      </div>

      <!-- Actions tab -->
      <div v-if="activeTab === 'actions'" class="space-y-3">
        <AppCard v-for="(topic, i) in topics.filter(t => t.recommended_action).slice(0, 8)" :key="topic.topic_id" padding="md">
          <div class="flex items-start gap-3">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
              :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">{{ i + 1 }}</div>
            <div>
              <p class="text-sm font-semibold mb-0.5" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
              <p class="text-sm" :style="{ color: 'var(--text-2)' }">{{ topic.recommended_action }}</p>
            </div>
          </div>
        </AppCard>

        <AppCard v-if="!topics.filter(t => t.recommended_action).length" padding="lg" class="text-center">
          <p class="text-sm" :style="{ color: 'var(--text-3)' }">Your coach is building your personalised action plan.</p>
        </AppCard>

        <div class="mt-6">
          <AppButton variant="primary" size="lg" @click="router.push('/student')">
            Go to Coach Hub — Your Plan is Ready
          </AppButton>
        </div>
      </div>

      <!-- No data -->
      <div v-if="!loading && topics.length === 0" class="text-center py-16">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">
          Report not available yet. Please complete the diagnostic first.
        </p>
        <AppButton variant="primary" class="mt-4" @click="router.push('/student/diagnostic')">
          Start Diagnostic
        </AppButton>
      </div>

    </div>
  </div>
</template>
