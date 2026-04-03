<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, listTopics, getPriorityTopics, type SubjectDto, type TopicDto, type TopicCaseDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const auth = useAuthStore()
const router = useRouter()

const subjects = ref<SubjectDto[]>([])
const priorityTopics = ref<TopicCaseDto[]>([])
const loading = ref(true)
const starting = ref<number | null>(null)
const error = ref('')

const subjectIcons: Record<string, string> = {
  MATH: '∑', ENG: 'Aa', SCI: '⚛', SS: '⊕', ICT: '⌘',
}

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [subs, topics] = await Promise.all([
      listSubjects(1),
      getPriorityTopics(auth.currentAccount.id, 8),
    ])
    subjects.value = subs
    priorityTopics.value = topics
  } catch (e: any) {
    error.value = 'Failed to load subjects'
  }
  loading.value = false
})

async function startSubjectPractice(subjectId: number) {
  if (!auth.currentAccount || starting.value !== null) return
  starting.value = subjectId
  error.value = ''
  try {
    // Get priority topics for this subject to focus the session
    const subjectTopics = priorityTopics.value
      .filter((t) => t.subject_code === subjects.value.find(s => s.id === subjectId)?.code)
      .slice(0, 3)
      .map((t) => t.topic_id)

    // Fallback: get first 3 topics for the subject
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

const urgencyColors: Record<string, string> = {
  high: 'danger', medium: 'warm', low: 'success',
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Practice</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Choose a subject and start practising. Every question teaches you something.</p>
    </div>

    <!-- Error -->
    <div v-if="error" class="mb-4 p-3 rounded-[var(--radius-md)] text-sm"
      :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">
      {{ error }}
    </div>

    <!-- Subject Cards -->
    <div v-if="loading" class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
      <div v-for="i in 3" :key="i" class="h-32 rounded-[var(--radius-xl)] animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>
    <div v-else class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
      <AppCard
        v-for="subject in subjects"
        :key="subject.id"
        hover padding="lg"
      >
        <div class="flex items-center gap-3 mb-4">
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center text-lg font-bold"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }"
          >
            {{ subjectIcons[subject.code] ?? subject.name.charAt(0) }}
          </div>
          <div>
            <h3 class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ subject.name }}</h3>
            <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ subject.code }}</p>
          </div>
        </div>
        <AppButton
          variant="primary"
          size="sm"
          class="w-full"
          :loading="starting === subject.id"
          :disabled="starting !== null"
          @click="startSubjectPractice(subject.id)"
        >
          Start Practice →
        </AppButton>
      </AppCard>
    </div>

    <!-- Quick Actions -->
    <div class="flex flex-wrap gap-2 mb-8">
      <AppButton variant="secondary" size="sm" @click="router.push('/student/practice/custom-test')">
        ✎ Custom Test
      </AppButton>
      <AppButton variant="secondary" size="sm" @click="router.push('/student/knowledge-gap')">
        ◎ Knowledge Gap
      </AppButton>
      <AppButton variant="secondary" size="sm" @click="router.push('/student/mistakes')">
        ↻ Review Mistakes
      </AppButton>
    </div>

    <!-- Priority Topics -->
    <div v-if="priorityTopics.length">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
        Your Priority Topics
      </h3>
      <div class="space-y-2">
        <AppCard
          v-for="topic in priorityTopics.slice(0, 5)"
          :key="topic.topic_id"
          padding="sm" hover
          @click="router.push('/student/practice/custom-test')"
        >
          <div class="flex items-center gap-3">
            <div
              class="w-10 h-10 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
              :style="{
                backgroundColor: topic.mastery_score >= 6000 ? 'var(--success-light)' : topic.mastery_score >= 3000 ? 'var(--warning-light)' : 'var(--danger-light)',
                color: topic.mastery_score >= 6000 ? 'var(--success)' : topic.mastery_score >= 3000 ? 'var(--warning)' : 'var(--danger)',
              }"
            >
              {{ formatBp(topic.mastery_score) }}
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ topic.topic_name }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">
                Gap: {{ formatBp(topic.gap_score) }} · {{ topic.intervention_mode.replace(/_/g, ' ') }}
              </p>
            </div>
            <AppBadge
              :color="(urgencyColors[topic.intervention_urgency] as any) ?? 'muted'"
              size="xs"
            >
              {{ topic.intervention_urgency }}
            </AppBadge>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
