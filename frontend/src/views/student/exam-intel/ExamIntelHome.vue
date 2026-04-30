<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { listExamHotspots, type ExamHotspotDto } from '@/ipc/library'
import { startPracticeSession } from '@/ipc/sessions'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const hotspots = ref<ExamHotspotDto[]>([])
const loading = ref(true)
const launching = ref<number | null>(null)
const error = ref('')

const featuredHotspot = computed(() => hotspots.value[0] ?? null)
const insightHotspots = computed(() => hotspots.value.slice(0, 3))

onMounted(() => {
  void loadSubjects()
})

watch(selectedSubjectId, value => {
  if (value == null) return
  void loadHotspots(value)
})

async function loadSubjects() {
  loading.value = true
  error.value = ''

  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
      await loadHotspots(subjects.value[0].id)
    }
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load exam intelligence'
  } finally {
    loading.value = false
  }
}

async function loadHotspots(subjectId: number) {
  if (!auth.currentAccount) return

  error.value = ''
  try {
    hotspots.value = await listExamHotspots(auth.currentAccount.id, subjectId, 6)
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Could not load hotspot data'
  }
}

function bpToPercent(value: number | null | undefined) {
  if (value == null) return null
  return Math.round(value / 100)
}

function hotspotSymbol(hotspot: ExamHotspotDto) {
  const words = hotspot.family_name.split(/\s+/).filter(Boolean)
  if (words.length === 1) return words[0].slice(0, 2).toUpperCase()
  return `${words[0][0]}${words[1][0]}`.toUpperCase()
}

function openFamily(hotspot: ExamHotspotDto | null) {
  if (!hotspot) return
  void router.push(`/student/exam-intel/family/${hotspot.family_id}`)
}

async function openPractice(hotspot: ExamHotspotDto | null) {
  if (!hotspot || !auth.currentAccount || hotspot.topic_id == null) {
    return
  }

  launching.value = hotspot.family_id
  error.value = ''

  try {
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: hotspot.subject_id,
      topic_ids: [hotspot.topic_id],
      family_ids: [hotspot.family_id],
      question_count: 10,
      is_timed: false,
    })
    void router.push(`/student/session/${session.session_id}`)
  } catch (cause: any) {
    error.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to start family drill'
    launching.value = null
  }
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center justify-between">
        <div>
          <p class="eyebrow">Intelligence</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            Exam Intelligence
          </h1>
          <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
            Past questions leave a trail. Follow the real recurring families, then launch drills filtered to that exact family.
          </p>
        </div>
        <div v-if="subjects.length" class="flex gap-1.5">
          <button
            v-for="s in subjects"
            :key="s.id"
            class="subj-tab"
            :class="{ active: selectedSubjectId === s.id }"
            @click="selectedSubjectId = s.id"
          >{{ s.name }}</button>
        </div>
      </div>
    </div>

    <div v-if="error" class="mx-6 mt-4 rounded-xl px-4 py-3 text-sm"
      :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="flex-1 p-6 space-y-4">
      <div v-for="i in 4" :key="i" class="h-24 rounded-2xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="flex-1 overflow-hidden flex">
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <div>
          <p class="section-label mb-4">Recurring Families</p>
          <div v-if="hotspots.length" class="grid grid-cols-3 gap-3">
            <button
              v-for="hotspot in hotspots"
              :key="hotspot.family_id"
              class="entry-card"
              @click="openFamily(hotspot)"
            >
              <div class="entry-symbol">{{ hotspotSymbol(hotspot) }}</div>
              <p class="text-sm font-bold mb-0.5" :style="{ color: 'var(--ink)' }">{{ hotspot.family_name }}</p>
              <p class="text-[10px] mb-2" :style="{ color: 'var(--ink-muted)' }">
                {{ hotspot.topic_name ?? 'Whole subject pattern' }}
              </p>
              <div class="metric-row">
                <span>Recurs</span>
                <strong>{{ bpToPercent(hotspot.recurrence_rate_bp) }}%</strong>
              </div>
              <div class="metric-row">
                <span>Accuracy</span>
                <strong>{{ bpToPercent(hotspot.student_accuracy_bp) ?? '--' }}%</strong>
              </div>
              <div class="mt-3 grid w-full gap-2">
                <button class="entry-action" @click.stop="openFamily(hotspot)">Open Family</button>
                <button
                  class="entry-action"
                  :disabled="hotspot.topic_id == null || launching === hotspot.family_id"
                  @click.stop="openPractice(hotspot)"
                >
                  {{
                    hotspot.topic_id == null
                      ? 'Topic Needed'
                      : launching === hotspot.family_id
                        ? 'Starting...'
                        : 'Practice Family'
                  }}
                </button>
              </div>
            </button>
          </div>
          <div v-else class="rounded-2xl px-5 py-5 text-sm"
            :style="{ backgroundColor: 'var(--surface)', color: 'var(--ink-secondary)' }">
            No hotspot data has been surfaced for this subject yet.
          </div>
        </div>
      </div>

      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Quick Insight</p>
        </div>
        <div class="flex-1 overflow-y-auto p-4 space-y-3">
          <div v-if="featuredHotspot" class="insight-card">
            <p class="text-xs font-bold mb-2" :style="{ color: 'var(--ink)' }">{{ featuredHotspot.family_name }}</p>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              {{ featuredHotspot.reason }}
            </p>
            <div class="mt-3 flex items-center gap-2">
              <div class="h-1.5 rounded-full flex-1 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full"
                  :style="{ width: `${bpToPercent(featuredHotspot.current_relevance_bp) ?? 0}%`, backgroundColor: 'var(--warm)' }" />
              </div>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--warm)' }">
                {{ bpToPercent(featuredHotspot.current_relevance_bp) ?? 0 }}%
              </span>
            </div>
          </div>

          <div
            v-for="hotspot in insightHotspots.slice(1)"
            :key="`insight-${hotspot.family_id}`"
            class="insight-card"
          >
            <p class="text-xs font-bold mb-2" :style="{ color: 'var(--ink)' }">{{ hotspot.family_name }}</p>
            <p class="text-[11px] leading-relaxed" :style="{ color: 'var(--ink-muted)' }">
              {{ hotspot.reason }}
            </p>
            <div class="mt-3 flex items-center gap-2">
              <div class="h-1.5 rounded-full flex-1 overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                <div class="h-full rounded-full"
                  :style="{ width: `${bpToPercent(hotspot.recurrence_rate_bp) ?? 0}%`, backgroundColor: 'var(--gold)' }" />
              </div>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--gold)' }">
                {{ bpToPercent(hotspot.recurrence_rate_bp) ?? 0 }}%
              </span>
            </div>
          </div>
        </div>
        <div class="p-4 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button class="w-full insight-btn" @click="openFamily(featuredHotspot)">Open Family</button>
          <button
            class="w-full insight-btn"
            :disabled="!featuredHotspot || featuredHotspot.topic_id == null || launching === featuredHotspot.family_id"
            @click="openPractice(featuredHotspot)"
          >
            {{
              !featuredHotspot || featuredHotspot.topic_id == null
                ? 'Topic Needed'
                : launching === featuredHotspot.family_id
                  ? 'Starting...'
                  : 'Practice Family'
            }}
          </button>
          <button class="w-full insight-btn" @click="router.push('/student/library')">Open Library</button>
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
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.subj-tab {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}

.subj-tab.active,
.subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.entry-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px 14px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: var(--surface);
  cursor: pointer;
  transition: border-color 130ms, transform 130ms;
}

.entry-card:hover {
  transform: translateY(-2px);
  border-color: var(--ink-muted);
}

.entry-symbol {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 900;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid transparent;
  margin-bottom: 10px;
}

.metric-row {
  width: 100%;
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--ink-muted);
  margin-top: 4px;
}

.metric-row strong {
  color: var(--ink);
}

.entry-action {
  width: 100%;
  padding: 8px 10px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 700;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}

.entry-action:hover:not(:disabled),
.insight-btn:hover:not(:disabled) {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.entry-action:disabled,
.insight-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.insight-card {
  padding: 14px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: var(--paper);
}

.insight-btn {
  padding: 9px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
}
</style>
