<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getGapDashboard, type GapDashboardDto, type GapScoreCardDto } from '@/ipc/gap'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppBadge from '@/components/ui/AppBadge.vue'
import GapSectionCard from '@/components/modes/knowledge-gap/GapSectionCard.vue'
import GapMapNode from '@/components/modes/knowledge-gap/GapMapNode.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const gapDash = ref<GapDashboardDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    ;[truth.value, gapDash.value] = await Promise.all([
      getLearnerTruth(auth.currentAccount.id),
      getGapDashboard(auth.currentAccount.id),
    ])
  } catch {}
  loading.value = false
})

const gapPercentage = computed(() => {
  if (!truth.value) return 0
  return Math.max(0, 100 - Math.round(truth.value.overall_mastery_score / 100))
})

const criticalGaps = computed<GapScoreCardDto[]>(() =>
  gapDash.value?.gaps.filter(g => g.gap_score > 7000) ?? [],
)
const slippingGaps = computed<GapScoreCardDto[]>(() =>
  gapDash.value?.gaps.filter(g => g.gap_score > 5000 && g.gap_score <= 7000) ?? [],
)
const improvingGaps = computed<GapScoreCardDto[]>(() =>
  gapDash.value?.gaps.filter(g => g.gap_score > 3000 && g.gap_score <= 5000) ?? [],
)

const sections = computed(() => [
  { title: 'Fix Now', severity: 'critical' as const, count: criticalGaps.value.length, items: criticalGaps.value.map(g => ({ name: g.topic_name, score: g.gap_score })) },
  { title: 'Slipping Areas', severity: 'slipping' as const, count: slippingGaps.value.length, items: slippingGaps.value.map(g => ({ name: g.topic_name, score: g.gap_score })) },
  { title: 'Weak but Improving', severity: 'improving' as const, count: improvingGaps.value.length, items: improvingGaps.value.map(g => ({ name: g.topic_name, score: g.gap_score })) },
  { title: 'Recently Solidified', severity: 'fixed' as const, count: gapDash.value?.topics_solidified ?? 0, items: [] },
])

const allGapTopics = computed(() => gapDash.value?.gaps ?? [])
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Knowledge Gaps</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          See what you don't know yet
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Close the gap to zero — one topic at a time</p>
      </div>
      <div class="flex items-center gap-4">
        <div v-if="!loading" class="text-right">
          <p class="text-3xl font-black tabular-nums" :style="{ color: gapPercentage > 50 ? 'var(--warm)' : gapPercentage > 25 ? 'var(--gold)' : 'var(--accent)' }">
            {{ gapPercentage }}%
          </p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Overall Gap</p>
        </div>
        <button class="cta-btn" @click="router.push('/student/knowledge-gap/scan')">Scan My Gaps →</button>
        <button class="nav-pill" @click="router.push('/student/progress/mastery-map')">Full Map</button>
      </div>
    </div>

    <!-- Stat strip -->
    <div
      class="flex-shrink-0 grid grid-cols-4 divide-x border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ gapPercentage }}%</p>
        <p class="stat-lbl">Gap</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--warm)' }">{{ gapDash?.critical_gap_count ?? 0 }}</p>
        <p class="stat-lbl">Critical</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--gold)' }">{{ gapDash?.active_repair_count ?? 0 }}</p>
        <p class="stat-lbl">In Repair</p>
      </div>
      <div class="stat-cell">
        <p class="stat-big" :style="{ color: 'var(--accent)' }">{{ gapDash?.topics_solidified ?? 0 }}</p>
        <p class="stat-lbl">Solidified</p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div v-for="i in 4" :key="i" class="h-20 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: sections -->
      <div class="flex-1 overflow-y-auto p-6 space-y-5">
        <div v-if="!allGapTopics.length" class="flex flex-col items-center justify-center h-full gap-4 text-center">
          <p class="text-3xl">✓</p>
          <p class="font-display text-xl font-bold" :style="{ color: 'var(--ink)' }">No significant gaps!</p>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Keep practising to stay strong.</p>
        </div>

        <template v-else>
          <div v-if="sections.some(s => s.count > 0)" class="grid grid-cols-2 gap-4">
            <GapSectionCard
              v-for="sec in sections.filter(s => s.count > 0)"
              :key="sec.title"
              :title="sec.title"
              :severity="sec.severity"
              :count="sec.count"
              :items="sec.items"
            />
          </div>

          <div v-if="gapDash?.repairs.length">
            <p class="section-label mb-3">Active Repair Plans</p>
            <div class="space-y-2">
              <div v-for="plan in gapDash.repairs" :key="plan.id"
                class="repair-row flex items-center gap-3 px-4 py-3 rounded-xl border"
                :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ plan.topic_name ?? 'Topic' }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    {{ plan.dominant_focus }} · {{ plan.recommended_session_type }}
                  </p>
                </div>
                <div class="text-right flex-shrink-0 flex items-center gap-3">
                  <div class="w-16">
                    <div class="h-1.5 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                      <div class="h-full rounded-full" :style="{ width: plan.progress_percent + '%', backgroundColor: 'var(--accent)' }" />
                    </div>
                    <p class="text-[10px] font-bold mt-0.5 text-right" :style="{ color: 'var(--accent)' }">{{ plan.progress_percent }}%</p>
                  </div>
                  <AppBadge
                    :color="plan.severity_label === 'critical' ? 'danger' : plan.severity_label === 'high' ? 'warm' : 'muted'"
                    size="xs"
                  >{{ plan.severity_label }}</AppBadge>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- Right: Gap map -->
      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Gap Map</p>
          <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ allGapTopics.length }} topics tracked</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4">
          <div v-if="!allGapTopics.length" class="py-10 text-center px-4">
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No gap data yet.<br>Run a scan first.</p>
          </div>
          <div class="flex flex-wrap gap-2">
            <GapMapNode
              v-for="t in allGapTopics"
              :key="t.topic_id"
              :name="t.topic_name"
              :gap-score="t.gap_score"
              :state="t.gap_score > 7000 ? 'not_started' : t.gap_score > 4000 ? 'weak' : 'developing'"
              :is-blocker="t.gap_score > 7000"
              :selected="false"
            />
          </div>
        </div>
        <div class="p-4 border-t flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full py-2.5 rounded-xl text-[11px] font-bold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/knowledge-gap/scan')">
            Run Gap Scan →
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--warm);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--ink-muted);
}

.cta-btn {
  padding: 9px 18px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  transition: opacity 140ms, transform 140ms;
}
.cta-btn:hover { opacity: 0.85; transform: translateY(-1px); }

.nav-pill {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.stat-cell {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.stat-big {
  font-size: 26px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}
.stat-lbl {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--ink-muted);
}

.repair-row { transition: background-color 100ms; }
.repair-row:hover { background-color: var(--paper) !important; }
</style>
