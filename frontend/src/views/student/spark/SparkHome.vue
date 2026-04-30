<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getGoalRecommendation,
  listSubjects,
  saveLearnerGoal,
  type GoalRecommendationDto,
  type SubjectDto,
} from '@/ipc/coach'

const router = useRouter()
const auth = useAuthStore()
const step = ref(0)
const loading = ref(true)
const saving = ref(false)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const answers = ref<string[]>([])
const recommendation = ref<GoalRecommendationDto | null>(null)

const questions = [
  {
    question: 'What frustrates you most about studying?',
    options: [
      { key: 'overwhelmed', label: "Too much to learn, don't know where to start" },
      { key: 'bored', label: "It's boring, I lose focus quickly" },
      { key: 'defeated', label: 'I try but I keep failing' },
      { key: 'distracted', label: 'I get distracted by other things' },
      { key: 'disconnected', label: "I don't see why it matters" },
      { key: 'nervous', label: 'I panic when I see test questions' },
    ],
  },
  {
    question: 'How do you prefer to learn?',
    options: [
      { key: 'challenge', label: 'Challenges and competitions' },
      { key: 'story', label: 'Stories and real-life examples' },
      { key: 'quick', label: 'Quick, short activities' },
      { key: 'guided', label: 'Step-by-step guidance' },
    ],
  },
]

const isComplete = computed(() => recommendation.value !== null)
const selectedSubject = computed(
  () => subjects.value.find(subject => subject.id === selectedSubjectId.value) ?? null,
)

onMounted(() => {
  void loadSpark()
})

async function loadSpark() {
  loading.value = true
  error.value = ''

  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
    }
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load your setup'
  } finally {
    loading.value = false
  }
}

function selectAnswer(key: string) {
  answers.value[step.value] = key
  if (step.value < questions.length - 1) {
    step.value += 1
    return
  }

  void finishSpark()
}

async function finishSpark() {
  if (!auth.currentAccount || selectedSubjectId.value == null) return

  saving.value = true
  error.value = ''

  try {
    const goalType = deriveGoalType()
    const confidenceLevel = deriveConfidenceLevel()
    await saveLearnerGoal(
      auth.currentAccount.id,
      selectedSubjectId.value,
      goalType,
      'Adaptive focus plan',
      null,
      confidenceLevel,
    )
    recommendation.value = await getGoalRecommendation(auth.currentAccount.id, selectedSubjectId.value)
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Could not save your learning setup'
  } finally {
    saving.value = false
  }
}

function deriveGoalType() {
  const frustration = answers.value[0]
  const preference = answers.value[1]

  if (frustration === 'defeated' || frustration === 'nervous') return 'confidence_recovery'
  if (frustration === 'overwhelmed' || preference === 'guided') return 'weakness_repair'
  if (frustration === 'disconnected' || preference === 'story') return 'foundation_building'
  if (preference === 'challenge') return 'top_performance'
  if (preference === 'quick' || frustration === 'distracted' || frustration === 'bored') {
    return 'revision_refresh'
  }
  return 'exam_readiness'
}

function deriveConfidenceLevel() {
  const frustration = answers.value[0]
  const preference = answers.value[1]

  if (frustration === 'defeated' || frustration === 'nervous') return 'low'
  if (preference === 'challenge') return 'high'
  return 'medium'
}

function stepWidth(index: number) {
  if (recommendation.value) return '2'
  return index === step.value ? '2' : '1'
}
</script>

<template>
  <div class="h-full flex overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">
    <div
      class="w-72 flex-shrink-0 flex flex-col justify-between p-8 border-r"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow mb-4">Spark Mode</p>
        <h2 class="font-display text-2xl font-bold leading-tight" :style="{ color: 'var(--ink)' }">
          Tune your learning setup
        </h2>
        <p class="text-xs mt-3" :style="{ color: 'var(--ink-muted)' }">
          This feeds your real study goal so the coach can shape the next steps around it.
        </p>

        <div v-if="subjects.length" class="mt-5 space-y-2">
          <p class="text-[10px] font-semibold uppercase tracking-[0.12em]" :style="{ color: 'var(--ink-muted)' }">
            Focus Subject
          </p>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="subject in subjects"
              :key="subject.id"
              class="subject-chip"
              :class="{ active: selectedSubjectId === subject.id }"
              @click="selectedSubjectId = subject.id"
            >
              {{ subject.name }}
            </button>
          </div>
        </div>
      </div>

      <div>
        <div class="flex gap-2 mb-3">
          <div
            v-for="(_, i) in questions"
            :key="i"
            class="h-1 rounded-full transition-all duration-300"
            :style="{
              flex: stepWidth(i),
              backgroundColor: recommendation || i <= step ? 'var(--ink)' : 'var(--border-soft)',
            }"
          />
        </div>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          {{ recommendation ? 'Setup saved' : `Question ${step + 1} of ${questions.length}` }}
        </p>
      </div>
    </div>

    <div class="flex-1 flex flex-col overflow-hidden">
      <div v-if="loading" class="flex-1 p-10 space-y-4">
        <div v-for="i in 4" :key="i" class="h-16 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <template v-else>
        <div v-if="error" class="mx-10 mt-8 rounded-xl px-4 py-3 text-sm"
          :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
          {{ error }}
        </div>

        <template v-if="!isComplete">
          <div class="flex-shrink-0 px-10 pt-12 pb-8 border-b" :style="{ borderColor: 'var(--border-soft)' }">
            <h2 class="font-display text-2xl font-bold" :style="{ color: 'var(--ink)' }">
              {{ questions[step].question }}
            </h2>
            <p v-if="selectedSubject" class="text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">
              {{ selectedSubject.name }}
            </p>
          </div>

          <div class="flex-1 overflow-y-auto px-10 py-6">
            <div class="space-y-2.5 max-w-xl">
              <button
                v-for="opt in questions[step].options"
                :key="opt.key"
                class="option-card w-full text-left"
                :disabled="saving"
                @click="selectAnswer(opt.key)"
              >
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ opt.label }}</p>
              </button>
            </div>
          </div>

          <div class="flex-shrink-0 px-10 py-5 border-t flex items-center justify-between"
            :style="{ borderColor: 'transparent' }">
            <button class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }"
              @click="router.push('/student')">Skip for now</button>
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
              {{ saving ? 'Saving your setup...' : '~2 minutes to complete' }}
            </p>
          </div>
        </template>

        <template v-else>
          <div class="flex-shrink-0 px-10 pt-12 pb-8 border-b" :style="{ borderColor: 'var(--border-soft)' }">
            <p class="text-[10px] font-semibold uppercase tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">
              Your Coaching Setup
            </p>
            <h2 class="font-display text-2xl font-bold mt-2" :style="{ color: 'var(--ink)' }">
              {{ recommendation?.urgency_label }}
            </h2>
            <p class="text-sm mt-2" :style="{ color: 'var(--ink-secondary)' }">
              {{ selectedSubject?.name }} · {{ recommendation?.session_style }}
            </p>
          </div>

          <div class="flex-1 overflow-y-auto px-10 py-8">
            <div class="max-w-2xl space-y-6">
              <div class="rounded-2xl p-5" :style="{ backgroundColor: 'var(--surface)' }">
                <p class="text-[10px] uppercase font-semibold tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">
                  Default Mode
                </p>
                <p class="text-lg font-bold mt-2" :style="{ color: 'var(--ink)' }">
                  {{ recommendation?.default_topic_mode.replace(/_/g, ' ') }}
                </p>
                <p class="text-xs mt-2" :style="{ color: 'var(--ink-secondary)' }">
                  {{ recommendation?.days_remaining }} days remaining in the current goal window
                </p>
              </div>

              <div>
                <p class="text-[10px] uppercase font-semibold tracking-[0.14em] mb-3" :style="{ color: 'var(--ink-muted)' }">
                  Recommended Actions
                </p>
                <div class="space-y-2">
                  <div
                    v-for="action in recommendation?.recommended_actions ?? []"
                    :key="action"
                    class="action-row"
                  >
                    {{ action }}
                  </div>
                </div>
              </div>

              <div v-if="recommendation?.focus_subjects.length" class="rounded-2xl p-5"
                :style="{ backgroundColor: 'var(--surface)' }">
                <p class="text-[10px] uppercase font-semibold tracking-[0.14em]" :style="{ color: 'var(--ink-muted)' }">
                  Focus Subjects
                </p>
                <div class="flex flex-wrap gap-2 mt-3">
                  <span v-for="subject in recommendation?.focus_subjects" :key="subject" class="subject-chip active">
                    {{ subject }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div class="flex-shrink-0 px-10 py-5 border-t flex items-center justify-between"
            :style="{ borderColor: 'transparent' }">
            <button class="text-xs font-semibold" :style="{ color: 'var(--ink-muted)' }"
              @click="router.push('/student')">Return home</button>
            <button class="primary-btn" @click="router.push({ name: 'custom-test', query: selectedSubjectId ? { subjectId: String(selectedSubjectId), source: 'spark' } : { source: 'spark' } })">
              Open practice
            </button>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
}

.subject-chip {
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  border: 1px solid transparent;
  color: var(--ink-secondary);
  background: var(--paper);
  transition: all 120ms ease;
}

.subject-chip.active,
.subject-chip:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-glow);
}

.option-card {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid transparent;
  cursor: pointer;
  transition: border-color 120ms ease, transform 100ms ease, background-color 100ms ease;
}

.option-card:hover {
  transform: translateX(4px);
  border-color: var(--ink);
  background-color: var(--paper);
}

.option-card:disabled {
  opacity: 0.6;
  cursor: wait;
}

.action-row {
  padding: 14px 16px;
  border-radius: 12px;
  background: var(--surface);
  color: var(--ink);
  font-size: 13px;
  font-weight: 600;
}

.primary-btn {
  padding: 10px 16px;
  border-radius: 8px;
  background: var(--ink);
  color: var(--paper);
  font-size: 12px;
  font-weight: 700;
  border: none;
}
</style>
