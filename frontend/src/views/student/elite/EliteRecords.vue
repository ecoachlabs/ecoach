<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import {
  getEliteProfile,
  listEliteEarnedBadges,
  listElitePersonalBests,
  listEliteTopicDomination,
  type EliteEarnedBadgeRow,
  type ElitePersonalBestRow,
  type EliteProfileDto,
  type EliteTopicProfileDto,
} from '@/ipc/elite'
import EliteRecordsWall from '@/components/modes/elite/EliteRecordsWall.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { buildEliteRecordsView } from '@/utils/eliteRecordsView'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const profile = ref<EliteProfileDto | null>(null)
const topicDomination = ref<EliteTopicProfileDto[]>([])
const personalBests = ref<ElitePersonalBestRow[]>([])
const earnedBadges = ref<EliteEarnedBadgeRow[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    subjects.value = await listSubjects()
    if (subjects.value.length > 0) {
      selectedSubjectId.value = subjects.value[0].id
      await loadData(subjects.value[0].id)
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load records'
  }
  loading.value = false
})

async function loadData(subjectId: number) {
  if (!auth.currentAccount) return

  ;[
    profile.value,
    topicDomination.value,
    personalBests.value,
    earnedBadges.value,
  ] = await Promise.all([
    getEliteProfile(auth.currentAccount.id, subjectId),
    listEliteTopicDomination(auth.currentAccount.id, subjectId, 10),
    listElitePersonalBests(auth.currentAccount.id, subjectId).catch(() => []),
    listEliteEarnedBadges(auth.currentAccount.id, subjectId).catch(() => []),
  ])
}

async function onSubjectChange(subjectId: number) {
  selectedSubjectId.value = subjectId
  loading.value = true
  try {
    await loadData(subjectId)
  } catch {}
  loading.value = false
}

const recordsView = computed(() => {
  if (!profile.value) {
    return { records: [], badges: [], titles: [] }
  }

  return buildEliteRecordsView(
    profile.value,
    topicDomination.value,
    personalBests.value,
    earnedBadges.value,
  )
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-center gap-3 mb-6">
      <button class="text-xs hover:underline" :style="{ color: 'var(--ink-muted)' }" @click="router.push('/student/elite')">
        Back to Elite Mode
      </button>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Records Wall
      </h1>
    </div>

    <div class="mb-6 flex items-center gap-3 flex-wrap">
      <span class="text-xs font-semibold uppercase" :style="{ color: 'var(--ink-muted)' }">Subject:</span>
      <div class="flex gap-2 flex-wrap">
        <button
          v-for="subj in subjects"
          :key="subj.id"
          class="px-3 py-1.5 rounded-full text-xs font-semibold transition-all"
          :style="{
            backgroundColor: selectedSubjectId === subj.id ? 'var(--accent)' : 'var(--border-soft)',
            color: selectedSubjectId === subj.id ? 'white' : 'var(--ink-secondary)',
          }"
          @click="onSubjectChange(subj.id)"
        >
          {{ subj.name }}
        </button>
      </div>
    </div>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 3" :key="i" class="h-32 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else-if="profile">
      <EliteRecordsWall
        :records="recordsView.records"
        :badges="recordsView.badges"
        :titles="recordsView.titles"
      />
    </template>

    <div v-else class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">No Elite profile for this subject yet.</p>
      <AppButton variant="primary" @click="router.push('/student/elite')">Enter Elite Mode</AppButton>
    </div>
  </div>
</template>
