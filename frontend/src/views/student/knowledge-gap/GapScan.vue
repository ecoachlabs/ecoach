<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listPriorityGaps, type GapScoreCardDto } from '@/ipc/gap'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const launching = ref(false)
const error = ref('')
const gaps = ref<GapScoreCardDto[]>([])
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    ;[gaps.value, subjects.value] = await Promise.all([
      listPriorityGaps(auth.currentAccount.id, 15),
      listSubjects(),
    ])
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load gap data'
  }
  loading.value = false
})

async function launchScan() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value) return
  launching.value = true
  error.value = ''
  try {
    // Pick top gap topics for this subject to build a targeted gap scan session
    const topTopicIds = gaps.value.slice(0, 5).map(g => g.topic_id)
    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topTopicIds,
      question_count: 12,
      is_timed: false,
    })
    router.push(`/student/session/${session.session_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start gap scan'
    launching.value = false
  }
}
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <button
      class="flex items-center gap-1 text-xs mb-6 hover:underline"
      :style="{ color: 'var(--ink-muted)' }"
      @click="router.push('/student/knowledge-gap')"
    >
      ← Back to Knowledge Gap
    </button>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else>
      <!-- Header -->
      <div class="text-center mb-8">
        <div class="w-20 h-20 rounded-3xl mx-auto mb-6 flex items-center justify-center text-3xl"
          :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">
          ◌
        </div>
        <h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--ink)' }">
          Knowledge Gap Scan
        </h2>
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
          Answer targeted questions to pinpoint exactly what you don't know yet.
          The AI will identify your real gaps — not just low scores, but missing understanding.
        </p>
      </div>

      <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
        {{ error }}
      </div>

      <!-- What we'll scan -->
      <AppCard padding="lg" class="mb-6">
        <h3 class="text-sm font-semibold mb-4" :style="{ color: 'var(--ink)' }">What the scan targets</h3>
        <div class="space-y-3">
          <div v-if="gaps.length > 0">
            <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">
              Your top {{ Math.min(5, gaps.length) }} gap areas
            </p>
            <div class="space-y-2">
              <div v-for="gap in gaps.slice(0, 5)" :key="gap.topic_id" class="flex items-center gap-3">
                <div class="flex-1">
                  <div class="flex items-center justify-between mb-0.5">
                    <span class="text-xs font-medium" :style="{ color: 'var(--ink-secondary)' }">{{ gap.topic_name }}</span>
                    <span class="text-xs" :style="{ color: gap.gap_score > 7000 ? 'var(--warm)' : 'var(--gold)' }">
                      {{ gap.severity_label }}
                    </span>
                  </div>
                  <AppProgress
                    :value="gap.gap_score"
                    :max="10000"
                    size="sm"
                    :color="gap.gap_score > 7000 ? 'danger' : gap.gap_score > 4000 ? 'warm' : 'accent'"
                  />
                </div>
              </div>
            </div>
          </div>
          <div v-else>
            <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
              No significant gaps detected yet. The scan will explore broad topic coverage.
            </p>
          </div>
        </div>
      </AppCard>

      <!-- Subject selection -->
      <AppCard padding="md" class="mb-6">
        <p class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--ink-muted)' }">Subject to scan</p>
        <div class="flex gap-2 flex-wrap">
          <button
            v-for="subj in subjects"
            :key="subj.id"
            class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
            :style="{
              backgroundColor: selectedSubjectId === subj.id ? 'var(--accent)' : 'var(--border-soft)',
              color: selectedSubjectId === subj.id ? 'white' : 'var(--ink-secondary)',
            }"
            @click="selectedSubjectId = subj.id"
          >
            {{ subj.name }}
          </button>
        </div>
      </AppCard>

      <!-- Scan details -->
      <div class="grid grid-cols-3 gap-3 mb-8">
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">12</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Questions</p>
        </AppCard>
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">~8</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Minutes</p>
        </AppCard>
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">AI</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Targeted</p>
        </AppCard>
      </div>

      <AppButton
        variant="primary"
        size="lg"
        class="w-full"
        :loading="launching"
        :disabled="!selectedSubjectId"
        @click="launchScan"
      >
        Start Gap Scan →
      </AppButton>
    </template>
  </div>
</template>
