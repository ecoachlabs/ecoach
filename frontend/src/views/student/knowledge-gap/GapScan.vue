<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listPriorityGaps, type GapScoreCardDto } from '@/ipc/gap'
import { listSubjects, listTopics, type SubjectDto } from '@/ipc/coach'
import { startPracticeSession } from '@/ipc/sessions'
import {
  buildLearnerTopicIndex,
  type LearnerTopic,
} from '@/utils/learnerTopics'
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
const subjectTopicIds = ref<Record<number, number[]>>({})
const subjectLearnerTopics = ref<Record<number, LearnerTopic[]>>({})
const subjectTopicLookup = ref<Record<number, Map<number, LearnerTopic>>>({})

interface GapDisplayCard extends GapScoreCardDto {
  sourceTopicIds: number[]
}

onMounted(async () => {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  try {
    const [gapCards, subjectList] = await Promise.all([
      listPriorityGaps(auth.currentAccount.id, 20),
      listSubjects(),
    ])

    gaps.value = gapCards
    subjects.value = subjectList

    if (subjectList.length > 0) {
      selectedSubjectId.value = subjectList[0].id
      await ensureSubjectTopicIds(subjectList[0].id)
    }
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to load gap data'
  } finally {
    loading.value = false
  }
})

async function ensureSubjectTopicIds(subjectId: number): Promise<number[]> {
  if (subjectTopicIds.value[subjectId]) {
    return subjectTopicIds.value[subjectId]
  }

  const topics = await listTopics(subjectId).catch(() => [])
  const learnerIndex = buildLearnerTopicIndex(topics)
  const ids = topics.map(topic => topic.id)
  subjectTopicIds.value = {
    ...subjectTopicIds.value,
    [subjectId]: ids,
  }
  subjectLearnerTopics.value = {
    ...subjectLearnerTopics.value,
    [subjectId]: learnerIndex.topics,
  }
  subjectTopicLookup.value = {
    ...subjectTopicLookup.value,
    [subjectId]: learnerIndex.bySourceTopicId,
  }
  return ids
}

async function selectSubject(subjectId: number) {
  selectedSubjectId.value = subjectId
  error.value = ''
  await ensureSubjectTopicIds(subjectId)
}

const selectedSubjectGapCards = computed<GapDisplayCard[]>(() => {
  const subjectId = selectedSubjectId.value
  if (subjectId == null) return []

  const ids = new Set(subjectTopicIds.value[subjectId] ?? [])
  const lookup = subjectTopicLookup.value[subjectId]
  if (!ids.size || !lookup) return []

  const grouped = new Map<number, GapDisplayCard>()

  for (const gap of gaps.value) {
    if (!ids.has(gap.topic_id)) continue

    const learnerTopic = lookup.get(gap.topic_id)
    const groupId = learnerTopic?.id ?? gap.topic_id
    const sourceTopicIds = learnerTopic?.sourceTopicIds ?? [gap.topic_id]
    const existing = grouped.get(groupId)

    if (!existing) {
      grouped.set(groupId, {
        ...gap,
        topic_id: groupId,
        topic_name: learnerTopic?.name ?? gap.topic_name,
        sourceTopicIds: [...sourceTopicIds],
      })
      continue
    }

    existing.sourceTopicIds = Array.from(new Set([...existing.sourceTopicIds, ...sourceTopicIds]))
    existing.mastery_score = Math.min(existing.mastery_score, gap.mastery_score)
    existing.repair_priority = Math.max(existing.repair_priority, gap.repair_priority)
    existing.has_active_repair_plan = existing.has_active_repair_plan || gap.has_active_repair_plan

    if (gap.gap_score >= existing.gap_score) {
      existing.gap_score = gap.gap_score
      existing.severity_label = gap.severity_label
    }
  }

  return Array.from(grouped.values()).sort((left, right) =>
    right.gap_score - left.gap_score
    || right.repair_priority - left.repair_priority
    || left.topic_name.localeCompare(right.topic_name),
  )
})

const targetedTopicCount = computed(() => Math.min(5, selectedSubjectGapCards.value.length || 5))
const scanModeLabel = computed(() =>
  selectedSubjectGapCards.value.length > 0 ? 'Recorded gaps' : 'Subject probe',
)

async function launchScan() {
  if (!auth.currentAccount || !selectedSubjectId.value || launching.value) return

  launching.value = true
  error.value = ''

  try {
    const ids = await ensureSubjectTopicIds(selectedSubjectId.value)
    if (!ids.length) {
      throw new Error('No topics are available for the selected subject yet.')
    }

    const learnerTopics = subjectLearnerTopics.value[selectedSubjectId.value] ?? []
    let topTopicIds = Array.from(
      new Set(selectedSubjectGapCards.value.slice(0, 5).flatMap(gap => gap.sourceTopicIds)),
    )

    if (!topTopicIds.length) {
      topTopicIds = Array.from(
        new Set(learnerTopics.slice(0, 5).flatMap(topic => topic.sourceTopicIds)),
      )
    }

    if (!topTopicIds.length) {
      topTopicIds = ids.slice(0, 5)
    }

    const session = await startPracticeSession({
      student_id: auth.currentAccount.id,
      subject_id: selectedSubjectId.value,
      topic_ids: topTopicIds,
      question_count: 12,
      is_timed: false,
    })

    router.push(`/student/session/${session.session_id}`)
  } catch (cause: unknown) {
    error.value = typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : 'Failed to start gap scan'
    launching.value = false
    return
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
      <- Back to Knowledge Gap
    </button>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-20 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else>
      <div class="text-center mb-8">
        <div
          class="w-20 h-20 rounded-lg mx-auto mb-6 flex items-center justify-center text-xl font-black"
          :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }"
        >
          SCAN
        </div>
        <h2 class="font-display text-xl font-semibold mb-2" :style="{ color: 'var(--ink)' }">
          Knowledge Gap Scan
        </h2>
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
          This scan stays inside the subject you pick. When recorded gaps exist, it starts there; otherwise it probes the first available topics in that subject.
        </p>
      </div>

      <div
        v-if="error"
        class="mb-4 p-3 rounded-lg text-sm"
        :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }"
      >
        {{ error }}
      </div>

      <AppCard padding="md" class="mb-6">
        <p class="text-xs font-semibold uppercase mb-3" :style="{ color: 'var(--ink-muted)' }">Subject to scan</p>
        <div class="flex gap-2 flex-wrap">
          <button
            v-for="subject in subjects"
            :key="subject.id"
            class="subject-pill"
            :class="{ active: selectedSubjectId === subject.id }"
            @click="selectSubject(subject.id)"
          >
            {{ subject.name }}
          </button>
        </div>
      </AppCard>

      <AppCard padding="lg" class="mb-6">
        <h3 class="text-sm font-semibold mb-4" :style="{ color: 'var(--ink)' }">What the scan targets</h3>
        <div class="space-y-3">
          <div v-if="selectedSubjectGapCards.length > 0">
            <p class="text-xs font-semibold uppercase mb-2" :style="{ color: 'var(--ink-muted)' }">
              Top {{ Math.min(5, selectedSubjectGapCards.length) }} gap areas in this subject
            </p>
            <div class="space-y-2">
              <div v-for="gap in selectedSubjectGapCards.slice(0, 5)" :key="gap.topic_id" class="flex items-center gap-3">
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
              No strong gap signals are recorded for this subject yet. The scan will still stay inside the selected subject and probe the first available topics there.
            </p>
          </div>
        </div>
      </AppCard>

      <div class="grid grid-cols-3 gap-3 mb-8">
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">12</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Question Count</p>
        </AppCard>
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">{{ targetedTopicCount }}</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Topics Targeted</p>
        </AppCard>
        <AppCard padding="sm" class="text-center">
          <p class="font-display text-lg font-bold" :style="{ color: 'var(--accent)' }">{{ scanModeLabel }}</p>
          <p class="text-[10px] uppercase" :style="{ color: 'var(--ink-muted)' }">Selection Mode</p>
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
        Start Gap Scan ->
      </AppButton>
    </template>
  </div>
</template>

<style scoped>
.subject-pill {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--border-soft);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 100ms;
}

.subject-pill:hover,
.subject-pill.active {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>
