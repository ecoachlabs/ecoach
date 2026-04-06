<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, getPriorityTopics, type SubjectDto, type TopicCaseDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import { PhArrowRight, PhTarget, PhFlame, PhClock, PhStar } from '@phosphor-icons/vue'

const auth = useAuthStore()
const router = useRouter()

const subjects = ref<SubjectDto[]>([])
const priorityTopics = ref<TopicCaseDto[]>([])
const loading = ref(true)
const starting = ref<number | null>(null)
const error = ref('')

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [subs, topics] = await Promise.all([
      listSubjects(1),
      getPriorityTopics(auth.currentAccount.id, 8),
    ])
    subjects.value = subs
    priorityTopics.value = topics
  } catch {
    error.value = 'Failed to load subjects'
  }
  loading.value = false
})

async function startSubjectPractice(subjectId: number) {
  if (!auth.currentAccount || starting.value !== null) return
  starting.value = subjectId
  error.value = ''
  try {
    const subjectTopics = priorityTopics.value
      .filter((t) => t.subject_code === subjects.value.find(s => s.id === subjectId)?.code)
      .slice(0, 3)
      .map((t) => t.topic_id)

    let topicIds = subjectTopics
    if (topicIds.length === 0) {
      const topics = await listTopics(subjectId)
      topicIds = topics.slice(0, 3).map((t) => t.id)
    }

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: subjectId,
      topic_ids: topicIds,
      question_count: 10,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
  } finally {
    starting.value = null
  }
}

function formatBp(bp: number) {
  return (bp / 100).toFixed(0) + '%'
}

const subjectIndex: Record<string, { symbol: string }> = {
  MATH: { symbol: 'Σ' },
  ENG:  { symbol: 'Aa' },
  SCI:  { symbol: '⚛' },
  SS:   { symbol: '⊕' },
  ICT:  { symbol: '⌘' },
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Practice Hub</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Choose your subject
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Select a subject to start a smart practice session</p>
      </div>
      <div class="flex gap-2">
        <button class="header-link" @click="router.push('/student/practice/custom-test')">Custom Test</button>
        <button class="header-link" @click="router.push('/student/knowledge-gap')">Gap Analysis</button>
        <button class="header-link" @click="router.push('/student/mistakes')">Mistakes</button>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Subject list -->
      <div class="flex-1 overflow-y-auto p-7">

        <!-- Loading -->
        <div v-if="loading" class="space-y-3">
          <div v-for="i in 4" :key="i" class="h-24 rounded-2xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <!-- Subjects -->
        <div v-else class="space-y-3">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="subject-row w-full text-left"
            :disabled="starting !== null"
            @click="startSubjectPractice(subject.id)"
          >
            <div class="symbol-box">
              {{ subjectIndex[subject.code]?.symbol ?? subject.name.charAt(0) }}
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-base font-bold" :style="{ color: 'var(--ink)' }">{{ subject.name }}</h3>
              <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ subject.code }} · Smart selection</p>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
              <span v-if="starting === subject.id" class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Starting…</span>
              <span v-else class="go-arrow">→</span>
            </div>
          </button>
        </div>

        <!-- More options -->
        <div class="mt-8">
          <p class="section-label mb-3">More Options</p>
          <div class="grid grid-cols-3 gap-3">
            <button class="option-tile" @click="router.push('/student/practice/custom-test')">
              <PhTarget :size="20" weight="duotone" :style="{ color: 'var(--ink-secondary)' }" />
              <span class="text-[11px] font-semibold mt-2" :style="{ color: 'var(--ink)' }">Custom Test</span>
              <span class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Pick topics & length</span>
            </button>
            <button class="option-tile" @click="router.push('/student/mistakes')">
              <PhFlame :size="20" weight="duotone" :style="{ color: 'var(--ink-secondary)' }" />
              <span class="text-[11px] font-semibold mt-2" :style="{ color: 'var(--ink)' }">Mistake Lab</span>
              <span class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Revisit errors</span>
            </button>
            <button class="option-tile" @click="router.push('/student/spark')">
              <PhStar :size="20" weight="duotone" :style="{ color: 'var(--ink-secondary)' }" />
              <span class="text-[11px] font-semibold mt-2" :style="{ color: 'var(--ink)' }">Spark</span>
              <span class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Random challenge</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Priority topics sidebar -->
      <div
        class="w-72 flex-shrink-0 flex flex-col overflow-hidden border-l"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Priority Topics</p>
          <p class="text-[11px] mt-1" :style="{ color: 'var(--ink-muted)' }">Focus your next session here</p>
        </div>

        <div class="flex-1 overflow-y-auto p-3 space-y-1">
          <div v-if="loading">
            <div v-for="i in 6" :key="i" class="h-14 rounded-xl animate-pulse mb-2"
              :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else-if="!priorityTopics.length" class="py-10 text-center px-4">
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No priority topics yet.<br>Complete a diagnostic first.</p>
          </div>
          <button
            v-for="topic in priorityTopics"
            :key="topic.topic_id"
            class="topic-btn w-full text-left px-3 py-2.5 rounded-xl"
            @click="router.push('/student/practice/custom-test')"
          >
            <div class="flex items-center gap-3">
              <div
                class="w-9 h-9 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0"
                :style="{
                  backgroundColor: topic.mastery_score >= 6000 ? 'rgba(13,148,136,0.1)' : topic.mastery_score >= 3000 ? 'rgba(180,83,9,0.1)' : 'rgba(194,65,12,0.1)',
                  color: topic.mastery_score >= 6000 ? 'var(--accent)' : topic.mastery_score >= 3000 ? 'var(--gold)' : 'var(--warm)',
                }"
              >
                {{ formatBp(topic.mastery_score) }}
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[11px] font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                <p class="text-[9px] truncate capitalize" :style="{ color: 'var(--ink-muted)' }">
                  {{ topic.intervention_urgency }} · {{ topic.intervention_mode.replace(/_/g, ' ') }}
                </p>
              </div>
            </div>
          </button>
        </div>

        <div class="p-3 border-t flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="w-full py-2.5 rounded-xl text-[11px] font-bold"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/practice/custom-test')"
          >
            Custom Practice →
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
  color: var(--accent);
  margin-bottom: 4px;
}

.header-link {
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
.header-link:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.subject-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px 20px;
  border-radius: 16px;
  border: 1px solid var(--border-soft);
  background-color: var(--surface);
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease, transform 120ms ease;
}
.subject-row:hover:not(:disabled) {
  border-color: var(--ink);
  transform: translateX(2px);
}
.subject-row:disabled { opacity: 0.6; cursor: not-allowed; }

.symbol-box {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background-color: var(--paper);
  border: 1px solid var(--border-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 900;
  color: var(--ink);
  flex-shrink: 0;
}

.go-arrow {
  font-size: 16px;
  color: var(--ink-muted);
  transition: transform 120ms ease, color 120ms ease;
}
.subject-row:hover .go-arrow { transform: translateX(3px); color: var(--ink); }

.option-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 12px;
  border-radius: 14px;
  border: 1px solid var(--border-soft);
  background-color: var(--surface);
  cursor: pointer;
  transition: border-color 100ms, background-color 100ms;
  gap: 0;
}
.option-tile:hover { border-color: var(--ink); background-color: var(--paper); }

.topic-btn {
  transition: background-color 100ms;
}
.topic-btn:hover { background-color: var(--paper); }
</style>
