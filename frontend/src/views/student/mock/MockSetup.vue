<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { compileMock, startMock } from '@/ipc/mock'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const loading = ref(true)
const starting = ref(false)
const error = ref('')

// Mock type from query param (set by MockHome)
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
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
    }
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
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <div class="mb-8">
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">{{ config.label }}</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">{{ config.desc }}</p>
    </div>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm"
      :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">
      {{ error }}
    </div>

    <div class="space-y-6 max-w-lg">
      <!-- Subject -->
      <AppCard padding="md">
        <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--text)' }">Subject</p>
        <div v-if="loading" class="flex gap-2">
          <div v-for="i in 3" :key="i" class="h-9 w-24 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div v-else class="flex flex-wrap gap-2">
          <button
            v-for="s in subjects"
            :key="s.id"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
            :class="selectedSubjectId === s.id
              ? 'bg-[var(--accent)] text-white shadow-sm'
              : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)] hover:border-[var(--accent)]'"
            @click="selectedSubjectId = s.id"
          >
            {{ s.name }}
          </button>
        </div>
      </AppCard>

      <!-- Duration -->
      <AppCard padding="md">
        <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--text)' }">Duration</p>
        <div class="flex gap-2 flex-wrap">
          <button
            v-for="mins in [20, 40, 60, 90]"
            :key="mins"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-all"
            :class="durationMinutes === mins
              ? 'bg-[var(--accent)] text-white'
              : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
            @click="durationMinutes = mins"
          >
            {{ mins }} min
          </button>
        </div>
      </AppCard>

      <!-- Question Count -->
      <AppCard padding="md">
        <p class="text-sm font-semibold mb-3" :style="{ color: 'var(--text)' }">Questions</p>
        <div class="flex gap-2">
          <button
            v-for="n in [10, 20, 30, 40]"
            :key="n"
            class="w-12 h-10 rounded-lg text-sm font-medium transition-all"
            :class="questionCount === n
              ? 'bg-[var(--accent)] text-white'
              : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
            @click="questionCount = n"
          >
            {{ n }}
          </button>
        </div>
      </AppCard>

      <!-- Summary badges -->
      <div class="flex gap-2 flex-wrap">
        <AppBadge color="accent" size="sm">{{ config.label }}</AppBadge>
        <AppBadge color="muted" size="sm">{{ durationMinutes }} min</AppBadge>
        <AppBadge color="muted" size="sm">{{ questionCount }} questions</AppBadge>
      </div>

      <div class="flex items-center gap-3">
        <AppButton
          variant="primary"
          size="lg"
          :loading="starting"
          :disabled="!selectedSubjectId || loading"
          @click="enterHall"
        >
          Enter Exam Hall →
        </AppButton>
        <AppButton variant="ghost" @click="router.push('/student/mock')">Cancel</AppButton>
      </div>
    </div>
  </div>
</template>
