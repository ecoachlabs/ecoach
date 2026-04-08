<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { compileMock, startMock } from '@/ipc/mock'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const loading = ref(true)
const starting = ref(false)
const error = ref('')

const mockType = computed(() => (route.query.type as string) ?? 'full')

const mockConfigs: Record<string, { label: string; duration: number; count: number; desc: string }> = {
  full: { label: 'Full Mock', duration: 90, count: 40, desc: 'Complete BECE-style exam with full timing.' },
  topic: { label: 'Topic Mock', duration: 40, count: 20, desc: 'Focus session targeting specific topics.' },
  mini: { label: 'Mini Mock', duration: 20, count: 20, desc: 'Quick 20-question cross-topic check.' },
  pressure: { label: 'Pressure Mock', duration: 35, count: 25, desc: 'Harder questions under tighter timing.' },
}

const config = computed(() => mockConfigs[mockType.value] ?? mockConfigs.full)

const durationMinutes = ref(0)
const questionCount = ref(0)

onMounted(async () => {
  durationMinutes.value = config.value.duration
  questionCount.value = config.value.count
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) selectedSubjectId.value = subjects.value[0].id
  } catch (e) {
    console.error('Failed to load subjects:', e)
  }
  loading.value = false
})

async function enterHall() {
  if (!auth.currentAccount || selectedSubjectId.value === null || starting.value) return
  starting.value = true
  error.value = ''
  try {
    const session = await compileMock({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      duration_minutes: durationMinutes.value,
      question_count: questionCount.value,
      topic_ids: [],
      paper_year: null,
      mock_type: mockType.value,
      blueprint_id: null,
    })
    const started = await startMock(session.id)
    router.push(`/student/mock/hall/${started.id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start mock'
    starting.value = false
  }
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="flex items-center gap-4">
        <button
          class="back-btn"
          @click="router.push('/student/mock')"
        >← Back</button>
        <div>
          <p class="eyebrow">Mock Setup</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
            {{ config.label }}
          </h1>
        </div>
      </div>
      <p class="text-sm max-w-xs text-right" :style="{ color: 'var(--ink-muted)' }">{{ config.desc }}</p>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Config panels -->
      <div class="flex-1 overflow-y-auto p-7 space-y-5">

        <!-- Subject -->
        <div class="config-section">
          <p class="section-label mb-4">Subject</p>
          <div v-if="loading" class="flex gap-2">
            <div v-for="i in 3" :key="i" class="h-10 w-28 rounded-xl animate-pulse"
              :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else class="flex flex-wrap gap-2">
            <button
              v-for="s in subjects"
              :key="s.id"
              class="option-chip"
              :class="{ active: selectedSubjectId === s.id }"
              @click="selectedSubjectId = s.id"
            >{{ s.name }}</button>
          </div>
        </div>

        <!-- Duration -->
        <div class="config-section">
          <p class="section-label mb-4">Duration</p>
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="mins in [20, 40, 60, 90]"
              :key="mins"
              class="option-chip"
              :class="{ active: durationMinutes === mins }"
              @click="durationMinutes = mins"
            >{{ mins }} min</button>
          </div>
        </div>

        <!-- Question Count -->
        <div class="config-section">
          <p class="section-label mb-4">Questions</p>
          <div class="flex gap-2">
            <button
              v-for="n in [10, 20, 30, 40]"
              :key="n"
              class="option-chip w-16 justify-center"
              :class="{ active: questionCount === n }"
              @click="questionCount = n"
            >{{ n }}</button>
          </div>
        </div>
      </div>

      <!-- Right: summary + launch -->
      <div
        class="w-80 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Session Summary</p>
        </div>

        <div class="flex-1 overflow-y-auto p-5 space-y-4">
          <div class="summary-row">
            <span class="summary-label">Type</span>
            <span class="summary-val">{{ config.label }}</span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Duration</span>
            <span class="summary-val">{{ durationMinutes }} minutes</span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Questions</span>
            <span class="summary-val">{{ questionCount }}</span>
          </div>
          <div class="summary-row">
            <span class="summary-label">Subject</span>
            <span class="summary-val">
              {{ subjects.find(s => s.id === selectedSubjectId)?.name ?? '—' }}
            </span>
          </div>

          <div class="pt-2 border-t" :style="{ borderColor: 'var(--border-soft)' }">
            <div class="text-center py-4">
              <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ questionCount }}</p>
              <p class="text-[10px] uppercase font-semibold mt-1" :style="{ color: 'var(--ink-muted)' }">questions · {{ durationMinutes }} min</p>
            </div>
          </div>
        </div>

        <div class="p-5 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="launch-btn w-full"
            :disabled="!selectedSubjectId || loading || starting"
            @click="enterHall"
          >
            {{ starting ? 'Preparing Hall…' : 'Enter Exam Hall →' }}
          </button>
          <button class="cancel-btn w-full" @click="router.push('/student/mock')">Cancel</button>
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

.back-btn {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 100ms;
}
.back-btn:hover { background: var(--border-soft); color: var(--ink); }

.config-section {
  padding: 20px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: var(--surface);
}

.option-chip {
  display: inline-flex;
  align-items: center;
  padding: 8px 18px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}
.option-chip:hover {
  border-color: var(--ink-muted);
  color: var(--ink);
}
.option-chip.active {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.summary-label {
  font-size: 11px;
  color: var(--ink-muted);
  font-weight: 500;
}
.summary-val {
  font-size: 12px;
  font-weight: 700;
  color: var(--ink);
}

.launch-btn {
  padding: 12px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  border: none;
  transition: opacity 140ms, transform 140ms;
}
.launch-btn:hover:not(:disabled) { opacity: 0.87; transform: translateY(-1px); }
.launch-btn:disabled { opacity: 0.4; cursor: not-allowed; }

.cancel-btn {
  padding: 9px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 100ms;
}
.cancel-btn:hover { background: var(--paper); color: var(--ink); }
</style>


