<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

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

async function start() {
  if (!auth.currentAccount || !selectedSubject.value || selectedTopics.value.length === 0) return
  starting.value = true
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
    console.error('Failed to start session:', e)
    starting.value = false
  }
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Custom Test</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Configure your practice session.</p>

    <!-- Subject Selection -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Subject</h3>
      <div class="flex gap-2">
        <button v-for="s in subjects" :key="s.id"
          class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium transition-all"
          :class="selectedSubject === s.id
            ? 'bg-[var(--accent)] text-white shadow-sm'
            : 'bg-[var(--card-bg)] text-[var(--text-2)] border border-[var(--card-border)] hover:border-[var(--accent)]'"
          @click="selectSubject(s.id)">
          {{ s.name }}
        </button>
      </div>
    </div>

    <!-- Topic Selection -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
        Topics <span v-if="selectedTopics.length" class="text-[var(--accent)]">({{ selectedTopics.length }} selected)</span>
      </h3>
      <div v-if="topics.length" class="space-y-2">
        <AppCard v-for="t in topics" :key="t.id" padding="sm" hover
          :class="selectedTopics.includes(t.id) ? 'ring-2 ring-[var(--accent)]' : ''"
          @click="toggleTopic(t.id)">
          <div class="flex items-center gap-3">
            <div v-if="selectedTopics.includes(t.id)" class="w-5 h-5 rounded-full bg-[var(--accent)] flex items-center justify-center text-white text-[10px]">✓</div>
            <div v-else class="w-5 h-5 rounded-full border-2" :style="{ borderColor: 'var(--card-border)' }" />
            <span class="text-sm" :style="{ color: 'var(--text)' }">{{ t.name }}</span>
            <AppBadge v-if="t.node_type" color="muted" size="xs">{{ t.node_type }}</AppBadge>
          </div>
        </AppCard>
      </div>
      <p v-else class="text-sm" :style="{ color: 'var(--text-3)' }">No topics available for this subject.</p>
    </div>

    <!-- Settings -->
    <div class="mb-8 flex items-center gap-6">
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Questions</h3>
        <div class="flex gap-2">
          <button v-for="n in [5, 10, 15, 20]" :key="n"
            class="w-10 h-10 rounded-lg text-sm font-medium transition-all"
            :class="questionCount === n ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
            @click="questionCount = n">{{ n }}</button>
        </div>
      </div>
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Timer</h3>
        <button class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
          :class="isTimed ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
          @click="isTimed = !isTimed">
          {{ isTimed ? '⏱ Timed' : '○ Untimed' }}
        </button>
      </div>
    </div>

    <!-- Start Button -->
    <div class="flex items-center gap-3">
      <AppButton variant="primary" size="lg" :loading="starting" :disabled="selectedTopics.length === 0" @click="start">
        Start Practice →
      </AppButton>
      <AppButton variant="ghost" @click="router.push('/student/practice')">Cancel</AppButton>
    </div>
  </div>
</template>
