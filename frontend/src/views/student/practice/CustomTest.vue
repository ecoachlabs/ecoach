<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'

const auth = useAuthStore()
const router = useRouter()

const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])
const selectedSubject = ref<number | null>(null)
const selectedTopics = ref<number[]>([])
const questionCount = ref(10)
const isTimed = ref(false)
const loading = ref(true)
const starting = ref(false)
const error = ref('')

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length > 0) {
      selectedSubject.value = subjects.value[0].id
      topics.value = await listTopics(subjects.value[0].id)
    }
  } catch (e) {
    console.error('Failed to load:', e)
  }
  loading.value = false
})

async function selectSubject(id: number) {
  selectedSubject.value = id
  selectedTopics.value = []
  try {
    topics.value = await listTopics(id)
  } catch {}
}

function toggleTopic(id: number) {
  const idx = selectedTopics.value.indexOf(id)
  if (idx >= 0) selectedTopics.value.splice(idx, 1)
  else selectedTopics.value.push(id)
}

function selectAll() {
  selectedTopics.value = topics.value.map(t => t.id)
}

function clearAll() {
  selectedTopics.value = []
}

async function start() {
  if (!auth.currentAccount || !selectedSubject.value || selectedTopics.value.length === 0) return
  starting.value = true
  error.value = ''
  try {
    const snapshot = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubject.value,
      topic_ids: selectedTopics.value,
      question_count: questionCount.value,
      is_timed: isTimed.value,
    })
    router.push(`/student/session/${snapshot.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start session'
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
        <button class="back-btn" @click="router.push('/student/practice')">← Back</button>
        <div>
          <p class="eyebrow">Practice</p>
          <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Custom Test</h1>
        </div>
      </div>
      <div class="flex gap-1.5">
        <button
          v-for="s in subjects"
          :key="s.id"
          class="subj-tab"
          :class="{ active: selectedSubject === s.id }"
          @click="selectSubject(s.id)"
        >{{ s.name }}</button>
      </div>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: topic selection -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        <div class="flex items-center justify-between">
          <p class="section-label">Topics
            <span v-if="selectedTopics.length" class="ml-2 normal-case font-normal text-[11px]"
              :style="{ color: 'var(--accent)' }">{{ selectedTopics.length }} selected</span>
          </p>
          <div class="flex gap-2">
            <button class="text-pill" @click="selectAll">All</button>
            <button class="text-pill" @click="clearAll">Clear</button>
          </div>
        </div>

        <div v-if="loading" class="space-y-2">
          <div v-for="i in 6" :key="i" class="h-12 rounded-xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <div v-else-if="topics.length" class="space-y-1.5">
          <button
            v-for="t in topics"
            :key="t.id"
            class="topic-row w-full flex items-center gap-3 px-4 py-3 rounded-xl border text-left"
            :class="{ selected: selectedTopics.includes(t.id) }"
            :style="{ borderColor: selectedTopics.includes(t.id) ? 'var(--accent)' : 'var(--border-soft)',
                      backgroundColor: selectedTopics.includes(t.id) ? 'var(--accent-glow)' : 'var(--surface)' }"
            @click="toggleTopic(t.id)"
          >
            <div class="check-circle flex-shrink-0" :class="{ checked: selectedTopics.includes(t.id) }">
              <span v-if="selectedTopics.includes(t.id)" class="text-[10px]">✓</span>
            </div>
            <span class="text-sm font-semibold flex-1" :style="{ color: 'var(--ink)' }">{{ t.name }}</span>
            <span v-if="(t as any).node_type" class="type-badge">{{ (t as any).node_type }}</span>
          </button>
        </div>

        <p v-else class="text-sm" :style="{ color: 'var(--ink-muted)' }">No topics available for this subject.</p>
      </div>

      <!-- Right: settings + launch -->
      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Settings</p>
        </div>

        <div class="flex-1 overflow-y-auto p-5 space-y-6">
          <!-- Question count -->
          <div>
            <p class="section-label mb-3">Questions</p>
            <div class="flex gap-2">
              <button
                v-for="n in [5, 10, 15, 20]"
                :key="n"
                class="count-btn"
                :class="{ active: questionCount === n }"
                @click="questionCount = n"
              >{{ n }}</button>
            </div>
          </div>

          <!-- Timer -->
          <div>
            <p class="section-label mb-3">Timer</p>
            <button
              class="timer-toggle w-full"
              :class="{ on: isTimed }"
              @click="isTimed = !isTimed"
            >
              <span class="toggle-knob" />
              <span class="text-xs font-semibold ml-3" :style="{ color: isTimed ? 'var(--accent)' : 'var(--ink-muted)' }">
                {{ isTimed ? 'Timed Mode' : 'Untimed' }}
              </span>
            </button>
          </div>

          <!-- Summary -->
          <div class="pt-2 border-t" :style="{ borderColor: 'var(--border-soft)' }">
            <div class="text-center py-4">
              <p class="text-4xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ questionCount }}</p>
              <p class="text-[10px] uppercase font-semibold mt-1" :style="{ color: 'var(--ink-muted)' }">
                questions · {{ selectedTopics.length }} topics
              </p>
            </div>
          </div>
        </div>

        <div class="p-5 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <button
            class="start-btn w-full"
            :disabled="selectedTopics.length === 0 || starting"
            @click="start"
          >{{ starting ? 'Starting…' : 'Start Practice →' }}</button>
          <button class="cancel-btn w-full" @click="router.push('/student/practice')">Cancel</button>
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
.subj-tab.active, .subj-tab:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.text-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 999px;
  cursor: pointer;
  background: transparent;
  color: var(--ink-muted);
  border: 1px solid transparent;
  transition: all 100ms;
}
.text-pill:hover { color: var(--ink); border-color: var(--ink-muted); }

.topic-row {
  cursor: pointer;
  transition: all 120ms;
}
.topic-row:not(.selected):hover { background-color: var(--paper) !important; }

.check-circle {
  width: 18px;
  height: 18px;
  border-radius: 999px;
  border: 2px solid var(--border-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 120ms;
  color: white;
}
.check-circle.checked {
  background: var(--accent);
  border-color: var(--accent);
}

.type-badge {
  font-size: 9px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
}

.count-btn {
  flex: 1;
  height: 38px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-secondary);
  transition: all 120ms;
}
.count-btn:hover { border-color: var(--ink-muted); color: var(--ink); }
.count-btn.active { background: var(--ink); color: var(--paper); border-color: var(--ink); }

.timer-toggle {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  border-radius: 12px;
  border: 1px solid transparent;
  background: var(--paper);
  cursor: pointer;
  transition: all 120ms;
}
.timer-toggle.on { border-color: var(--accent); background: var(--accent-glow); }

.toggle-knob {
  width: 36px;
  height: 20px;
  border-radius: 999px;
  background: var(--border-soft);
  position: relative;
  transition: background 200ms;
  flex-shrink: 0;
}
.timer-toggle.on .toggle-knob { background: var(--accent); }
.toggle-knob::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.15);
  transition: transform 200ms;
}
.timer-toggle.on .toggle-knob::after { transform: translateX(16px); }

.start-btn {
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
.start-btn:hover:not(:disabled) { opacity: 0.87; transform: translateY(-1px); }
.start-btn:disabled { opacity: 0.4; cursor: not-allowed; }

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


