<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import {
  buildEliteSessionBlueprintReport,
  eliteUiModeToSessionClass,
  rememberEliteSessionClass,
  type EliteBlueprintReportDto,
} from '@/ipc/elite'
import { startPracticeSession } from '@/ipc/sessions'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const startingClass = ref<string | null>(null)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const report = ref<EliteBlueprintReportDto | null>(null)

const sessionCards = computed(() => {
  const blueprint = report.value?.blueprint
  const trapSignal = report.value?.trap_signal
  const familyTargets = report.value?.family_targets.length ?? 0
  const topicTargets = report.value?.topic_targets.length ?? 0

  return [
    { key: 'precision', title: 'Precision Lab', token: 'PL', desc: 'Tight accuracy work on the blueprint topics.', timed: false },
    { key: 'sprint', title: 'Elite Sprint', token: 'ES', desc: 'Fast recall and clean execution under pressure.', timed: true },
    { key: 'depth', title: 'Depth Lab', token: 'DL', desc: 'Longer chains and multi-step reasoning.', timed: false },
    { key: 'trapsense', title: 'TrapSense', token: 'TS', desc: 'Confusion-heavy pattern repair and contrast work.', timed: false },
    { key: 'endurance', title: 'Endurance Track', token: 'ET', desc: 'Stay stable through a longer elite run.', timed: true },
    { key: 'perfect', title: 'Perfect Run', token: 'PR', desc: 'One-miss tension with clean decision making.', timed: true },
    { key: 'apex', title: 'Apex Mock', token: 'AM', desc: 'The highest-pressure mixed elite set.', timed: true },
  ].map(card => {
    const sessionClass = eliteUiModeToSessionClass[card.key]
    const recommended = blueprint?.session_class === sessionClass
    const questionCount = ({
      endurance_track: 16,
      apex_mock: 14,
      depth_lab: 12,
    } as Record<string, number>)[sessionClass] ?? blueprint?.target_question_count ?? 10

    return {
      ...card,
      sessionClass,
      questionCount,
      recommended,
      detail:
        sessionClass === 'trapsense' && trapSignal?.rationale
          ? trapSignal.rationale
          : `${topicTargets} topic targets - ${familyTargets} family targets`,
    }
  })
})

const recommendedCard = computed(() =>
  sessionCards.value.find(card => card.recommended) ?? sessionCards.value[0] ?? null,
)

onMounted(() => {
  void loadArena()
})

watch(selectedSubjectId, value => {
  if (value != null) {
    void loadReport(value)
  }
})

async function loadArena() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''

  try {
    subjects.value = await listSubjects(1)
    selectedSubjectId.value = subjects.value[0]?.id ?? null
    if (selectedSubjectId.value != null) {
      await loadReport(selectedSubjectId.value)
    }
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to load elite arena'
  } finally {
    loading.value = false
  }
}

async function loadReport(subjectId: number) {
  if (!auth.currentAccount) return
  report.value = await buildEliteSessionBlueprintReport(auth.currentAccount.id, subjectId)
}

async function startChallenge(sessionClass: string, timed: boolean) {
  if (!auth.currentAccount || !selectedSubjectId.value || !report.value || startingClass.value) return

  startingClass.value = sessionClass
  error.value = ''

  try {
    const topicIds = resolveTopicIds(sessionClass)
    const questionCount = ({
      endurance_track: 16,
      apex_mock: 14,
      depth_lab: 12,
    } as Record<string, number>)[sessionClass] ?? report.value.blueprint.target_question_count

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topicIds,
      question_count: questionCount,
      is_timed: timed,
    })

    rememberEliteSessionClass(session.session_id, sessionClass)
    void router.push(`/student/elite/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string'
      ? cause
      : cause instanceof Error
        ? cause.message
        : 'Failed to start elite challenge'
    startingClass.value = null
  }
}

function resolveTopicIds(sessionClass: string) {
  const baseIds = [...(report.value?.blueprint.target_topic_ids ?? [])]
  const trapTopicId = report.value?.trap_signal?.topic_id

  if (sessionClass === 'trapsense' && trapTopicId != null) {
    return [trapTopicId, ...baseIds.filter(topicId => topicId !== trapTopicId)].slice(0, 3)
  }

  if (sessionClass === 'depth_lab') {
    return baseIds.slice(0, 3)
  }

  return baseIds.slice(0, 5)
}

function percent(value: number) {
  return Math.round(value / 100)
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="flex-shrink-0 border-b px-7 pt-6 pb-5"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-start justify-between gap-6">
        <div>
          <p class="eyebrow">Elite Arena</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Challenge Arena
          </h1>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
            Live blueprint targets, trap pressure, and session-class recommendations.
          </p>
        </div>

        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="chip"
            :class="{ active: selectedSubjectId === subject.id }"
            @click="selectedSubjectId = subject.id"
          >
            {{ subject.name }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="index in 4" :key="index" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 flex overflow-hidden">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <section v-if="recommendedCard && report" class="rounded-2xl border p-5"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <div class="flex items-start justify-between gap-6">
            <div>
              <p class="section-label mb-2">Recommended Lane</p>
              <h2 class="text-xl font-bold" :style="{ color: 'var(--ink)' }">{{ recommendedCard.title }}</h2>
              <p class="text-sm mt-2 leading-6" :style="{ color: 'var(--ink-muted)' }">
                {{ report.blueprint.rationale }}
              </p>
            </div>
            <div class="hero-token">{{ recommendedCard.token }}</div>
          </div>

          <div class="mt-5 grid gap-3 md:grid-cols-4">
            <div class="stat-card">
              <span class="stat-label">EPS</span>
              <strong class="stat-value">{{ report.profile ? percent(report.profile.eps_score) : 0 }}%</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Topic Targets</span>
              <strong class="stat-value">{{ report.topic_targets.length }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Family Targets</span>
              <strong class="stat-value">{{ report.family_targets.length }}</strong>
            </div>
            <div class="stat-card">
              <span class="stat-label">Trap Pressure</span>
              <strong class="stat-value">{{ report.trap_signal ? percent(report.trap_signal.confusion_score) : 0 }}%</strong>
            </div>
          </div>
        </section>

        <section class="rounded-2xl border p-5" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
          <p class="section-label mb-4">Session Classes</p>
          <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
            <button
              v-for="card in sessionCards"
              :key="card.sessionClass"
              class="challenge-card"
              :class="{ recommended: card.recommended }"
              @click="startChallenge(card.sessionClass, card.timed)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="challenge-token">{{ card.token }}</div>
                <span v-if="card.recommended" class="recommend-pill">Recommended</span>
              </div>
              <p class="text-sm font-bold mt-4" :style="{ color: 'var(--ink)' }">{{ card.title }}</p>
              <p class="text-[11px] mt-2 leading-5" :style="{ color: 'var(--ink-muted)' }">{{ card.desc }}</p>
              <p class="text-[11px] mt-3 leading-5" :style="{ color: 'var(--ink-secondary)' }">{{ card.detail }}</p>
              <div class="mt-4 flex items-center justify-between">
                <span class="text-[11px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ card.questionCount }} questions</span>
                <span class="launch-pill">
                  {{ startingClass === card.sessionClass ? 'Starting...' : 'Launch' }}
                </span>
              </div>
            </button>
          </div>
        </section>
      </div>

      <aside
        class="w-80 flex-shrink-0 border-l overflow-y-auto p-5 space-y-5"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Topic Targets</p>
          <div v-if="report?.topic_targets.length" class="space-y-3">
            <div v-for="topic in report.topic_targets" :key="topic.topic_id">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
              <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">{{ topic.selection_reason }}</p>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No topic targets yet.</p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Family Targets</p>
          <div v-if="report?.family_targets.length" class="space-y-3">
            <div v-for="family in report.family_targets.slice(0, 6)" :key="family.family_id">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ family.family_name }}</p>
              <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">{{ family.selection_reason }}</p>
            </div>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No family targets yet.</p>
        </section>

        <section class="rounded-2xl border p-4" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
          <p class="section-label mb-3">Trap Signal</p>
          <div v-if="report?.trap_signal" class="space-y-3 text-sm">
            <div class="detail-row">
              <span>Topic</span>
              <strong>{{ report.trap_signal.topic_name ?? 'Mixed' }}</strong>
            </div>
            <div class="detail-row">
              <span>Confusion</span>
              <strong>{{ percent(report.trap_signal.confusion_score) }}%</strong>
            </div>
            <div class="detail-row">
              <span>Timed Out</span>
              <strong>{{ report.trap_signal.timed_out_count }}</strong>
            </div>
            <p class="text-[11px] leading-5" :style="{ color: 'var(--ink-muted)' }">
              {{ report.trap_signal.rationale ?? 'Trap signal is active for this subject.' }}
            </p>
          </div>
          <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">Trap pressure is quiet right now.</p>
        </section>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--gold);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.chip,
.challenge-card {
  border: 1px solid transparent;
  transition: all 120ms;
}

.chip {
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  background: var(--paper);
  color: var(--ink-secondary);
}

.chip:hover,
.chip.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.hero-token,
.challenge-token {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--paper);
  color: var(--ink);
  font-weight: 900;
}

.hero-token {
  width: 72px;
  height: 72px;
  border-radius: 20px;
  font-size: 18px;
}

.challenge-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 18px;
  border-radius: 16px;
  background: var(--paper);
  text-align: left;
}

.challenge-card:hover {
  transform: translateY(-2px);
  border-color: var(--ink-muted);
}

.challenge-card.recommended {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent-glow) 55%, var(--paper));
}

.challenge-token {
  width: 38px;
  height: 38px;
  border-radius: 12px;
  font-size: 12px;
}

.recommend-pill,
.launch-pill {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--surface);
  color: var(--ink-secondary);
  font-size: 11px;
  font-weight: 700;
}

.stat-card {
  padding: 16px;
  border-radius: 14px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stat-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ink-muted);
}

.stat-value {
  font-size: 22px;
  font-weight: 900;
  color: var(--ink);
}

.detail-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: var(--ink-muted);
}

.detail-row strong {
  color: var(--ink);
}
</style>

