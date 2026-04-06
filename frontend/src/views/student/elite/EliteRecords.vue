<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { getEliteProfile, listEliteTopicDomination, type EliteProfileDto, type EliteTopicProfileDto } from '@/ipc/elite'
import EliteRecordsWall from '@/components/modes/elite/EliteRecordsWall.vue'
import AppButton from '@/components/ui/AppButton.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const subjects = ref<SubjectDto[]>([])
const selectedSubjectId = ref<number | null>(null)
const profile = ref<EliteProfileDto | null>(null)
const topicDomination = ref<EliteTopicProfileDto[]>([])

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
  ;[profile.value, topicDomination.value] = await Promise.all([
    getEliteProfile(auth.currentAccount.id, subjectId),
    listEliteTopicDomination(auth.currentAccount.id, subjectId, 10),
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

// Derive records from real profile data
const records = computed(() => {
  if (!profile.value) return []
  return [
    { category: 'EPS Score', value: profile.value.eps_score.toString(), date: 'Current', isPersonalBest: true },
    { category: 'Precision', value: Math.round(profile.value.precision_score / 100) + '%', date: 'Current', isPersonalBest: profile.value.precision_score >= 8000 },
    { category: 'Speed', value: Math.round(profile.value.speed_score / 100) + '%', date: 'Current', isPersonalBest: profile.value.speed_score >= 8000 },
    { category: 'Depth', value: Math.round(profile.value.depth_score / 100) + '%', date: 'Current', isPersonalBest: profile.value.depth_score >= 8000 },
    { category: 'Composure', value: Math.round(profile.value.composure_score / 100) + '%', date: 'Current', isPersonalBest: profile.value.composure_score >= 8000 },
    {
      category: 'Best Topic',
      value: topicDomination.value[0]?.topic_name ?? '—',
      date: '',
      isPersonalBest: (topicDomination.value[0]?.domination_score ?? 0) >= 7000,
    },
  ]
})

const badges = computed(() => {
  const p = profile.value
  return [
    { name: 'Precision Master', icon: '◎', earned: !!p && p.precision_score >= 8000, description: '80%+ Precision' },
    { name: 'Speed Demon', icon: '⚡', earned: !!p && p.speed_score >= 8000, description: '80%+ Speed' },
    { name: 'Deep Thinker', icon: '◈', earned: !!p && p.depth_score >= 8000, description: '80%+ Depth' },
    { name: 'Iron Composure', icon: '∞', earned: !!p && p.composure_score >= 8000, description: '80%+ Composure' },
    { name: 'Elite Member', icon: '★', earned: !!p && p.eps_score >= 5000, description: '5000+ EPS' },
    { name: 'Dominator', icon: '👑', earned: topicDomination.value.some(t => t.domination_score >= 9000), description: 'One topic at 90%+' },
  ]
})

const titles = computed(() => {
  const tier = profile.value?.tier ?? ''
  const tierOrder = ['Foundation', 'Core', 'Prime', 'Apex', 'Master', 'Legend']
  const tierIdx = tierOrder.findIndex(t => t.toLowerCase() === tier.toLowerCase())
  return [
    { title: 'Foundation Scholar', earned: tierIdx >= 0 },
    { title: 'Core Contender', earned: tierIdx >= 1 },
    { title: 'Prime Performer', earned: tierIdx >= 2 },
    { title: 'Apex Achiever', earned: tierIdx >= 3 },
    { title: 'Master Strategist', earned: tierIdx >= 4 },
    { title: 'Legend', earned: tierIdx >= 5 },
  ]
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-center gap-3 mb-6">
      <button class="text-xs hover:underline" :style="{ color: 'var(--ink-muted)' }" @click="router.push('/student/elite')">
        ← Elite Mode
      </button>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Records Wall
      </h1>
    </div>

    <!-- Subject picker -->
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
        :records="records"
        :badges="badges"
        :titles="titles"
      />
    </template>

    <div v-else class="text-center py-16">
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">No Elite profile for this subject yet.</p>
      <AppButton variant="primary" @click="router.push('/student/elite')">Enter Elite Mode</AppButton>
    </div>
  </div>
</template>
