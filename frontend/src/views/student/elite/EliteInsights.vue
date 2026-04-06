<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { buildEliteSessionBlueprintReport, listEliteTopicDomination, type EliteBlueprintReportDto, type EliteTopicProfileDto } from '@/ipc/elite'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const report = ref<EliteBlueprintReportDto | null>(null)
const topics = ref<EliteTopicProfileDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    subjects.value = await listSubjects()
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
      await loadData(subjects.value[0].id)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load insights'
  }
  loading.value = false
})

async function loadData(subjectId: number) {
  if (!auth.currentAccount) return
  const studentId = auth.currentAccount.id
  ;[report.value, topics.value] = await Promise.all([
    buildEliteSessionBlueprintReport(studentId, subjectId),
    listEliteTopicDomination(studentId, subjectId, 10),
  ])
}

async function onSubjectChange(subjectId: number) {
  selectedSubjectId.value = subjectId
  loading.value = true
  try {
    await loadData(subjectId)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  }
  loading.value = false
}

// Weakest dimension to surface as insight
const weakestDimension = computed(() => {
  const p = report.value?.profile
  if (!p) return null
  const dims = [
    { label: 'Precision', value: p.precision_score },
    { label: 'Speed', value: p.speed_score },
    { label: 'Depth', value: p.depth_score },
    { label: 'Composure', value: p.composure_score },
  ]
  return dims.sort((a, b) => a.value - b.value)[0]
})

const strongestDimension = computed(() => {
  const p = report.value?.profile
  if (!p) return null
  const dims = [
    { label: 'Precision', value: p.precision_score },
    { label: 'Speed', value: p.speed_score },
    { label: 'Depth', value: p.depth_score },
    { label: 'Composure', value: p.composure_score },
  ]
  return dims.sort((a, b) => b.value - a.value)[0]
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-center gap-3 mb-6">
      <button class="text-xs hover:underline" :style="{ color: 'var(--ink-muted)' }" @click="router.push('/student/elite')">
        ← Elite Mode
      </button>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Elite Insights
      </h1>
    </div>

    <!-- Subject picker -->
    <div class="mb-6 flex items-center gap-3 flex-wrap">
      <span class="text-xs font-semibold uppercase" :style="{ color: 'var(--ink-muted)' }">Subject:</span>
      <div class="flex gap-2 flex-wrap">
        <button
          v-for="subj in subjects"
          :key="subj.id"
          class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
          :style="{
            backgroundColor: selectedSubjectId === subj.id ? 'var(--accent)' : 'var(--border-soft)',
            color: selectedSubjectId === subj.id ? 'white' : 'var(--ink-secondary)',
          }"
          @click="onSubjectChange(subj.id)"
        >
          {{ subj.name }}
        </button>
      </div>
    </div>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 4" :key="i" class="h-24 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else-if="report?.profile">
      <!-- Performance summary cards -->
      <div class="grid grid-cols-2 gap-4 mb-6">
        <AppCard padding="lg">
          <h3 class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--ink-muted)' }">
            Strongest Dimension
          </h3>
          <p class="font-display text-2xl font-bold mb-1" :style="{ color: 'var(--accent)' }">
            {{ strongestDimension?.label }}
          </p>
          <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
            {{ Math.round((strongestDimension?.value ?? 0) / 100) }}% — this is your sharpest edge.
          </p>
        </AppCard>
        <AppCard padding="lg">
          <h3 class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--ink-muted)' }">
            Focus Area
          </h3>
          <p class="font-display text-2xl font-bold mb-1" :style="{ color: 'var(--gold)' }">
            {{ weakestDimension?.label }}
          </p>
          <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">
            {{ Math.round((weakestDimension?.value ?? 0) / 100) }}% — target this in your next session.
          </p>
        </AppCard>
      </div>

      <!-- Next session blueprint -->
      <AppCard padding="lg" class="mb-6">
        <h3 class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--ink-muted)' }">Next Session Blueprint</h3>
        <div class="flex items-start gap-4">
          <div class="flex-1">
            <p class="text-sm font-semibold mb-1" :style="{ color: 'var(--ink)' }">
              {{ report.blueprint.session_class }} Session
            </p>
            <p class="text-xs mb-3" :style="{ color: 'var(--ink-muted)' }">{{ report.blueprint.rationale }}</p>
            <div class="flex flex-wrap gap-1">
              <AppBadge v-for="mode in report.blueprint.authoring_modes" :key="mode" color="accent" size="xs">
                {{ mode }}
              </AppBadge>
            </div>
          </div>
          <div class="text-right flex-shrink-0">
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">
              {{ report.blueprint.target_question_count }}
            </p>
            <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">questions</p>
          </div>
        </div>
      </AppCard>

      <!-- Topic domination breakdown -->
      <div v-if="topics.length">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">
          Topic Breakdown
        </h3>
        <div class="space-y-3">
          <AppCard v-for="t in topics" :key="t.topic_id" padding="sm">
            <div class="flex items-center justify-between mb-2">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ t.topic_name }}</p>
              <AppBadge
                :color="t.domination_score >= 8000 ? 'success' : t.domination_score >= 5000 ? 'gold' : 'warm'"
                size="xs"
              >
                {{ t.status }}
              </AppBadge>
            </div>
            <AppProgress :value="t.domination_score" :max="10000" size="sm"
              :color="t.domination_score >= 8000 ? 'success' : t.domination_score >= 5000 ? 'gold' : 'warm'"
              class="mb-2" />
            <div class="grid grid-cols-4 gap-2 text-[9px]" :style="{ color: 'var(--ink-muted)' }">
              <span>Precision: {{ Math.round(t.precision_score / 100) }}%</span>
              <span>Speed: {{ Math.round(t.speed_score / 100) }}%</span>
              <span>Traps: {{ Math.round(t.trap_resistance_score / 100) }}%</span>
              <span>Depth: {{ Math.round(t.depth_score / 100) }}%</span>
            </div>
          </AppCard>
        </div>
      </div>
    </template>

    <div v-else-if="!loading" class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">
        No Elite profile for this subject yet. Start a session to begin.
      </p>
      <AppButton variant="primary" @click="router.push('/student/elite')">Enter Elite Mode</AppButton>
    </div>
  </div>
</template>
