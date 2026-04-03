<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getReadinessReport } from '@/ipc/readiness'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const readiness = ref<any>(null)
const truth = ref<LearnerTruthDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [r, t] = await Promise.all([
      getReadinessReport(auth.currentAccount.id),
      getLearnerTruth(auth.currentAccount.id),
    ])
    readiness.value = r
    truth.value = t
  } catch (e) {
    console.error('Failed to load analytics:', e)
  }
  loading.value = false
})

const bandColor = computed(() => {
  const band = readiness.value?.overall_readiness_band ?? 'developing'
  if (band === 'strong') return 'success'
  if (band === 'developing') return 'gold'
  return 'danger'
})

const coveragePct = computed(() => {
  if (!readiness.value) return 0
  return Math.round(readiness.value.coverage_percent * 100)
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <PageHeader title="Analytics" subtitle="Your academic performance at a deeper level." back-to="/student/progress" />

    <!-- Loading -->
    <div v-if="loading" class="grid grid-cols-4 gap-3 mb-6 mt-6">
      <div v-for="i in 4" :key="i" class="h-20 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <template v-else>
      <!-- Top stats -->
      <div class="grid grid-cols-4 gap-3 mb-6 mt-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ coveragePct }}%</p>
          <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Coverage</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ truth?.diagnosis_count ?? 0 }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Diagnoses</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--gold)' }">{{ truth?.skill_count ?? 0 }}</p>
          <p class="text-[10px] uppercase mt-1" :style="{ color: 'var(--text-3)' }">Skills</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <AppBadge :color="(bandColor as any)" size="sm">{{ readiness?.overall_readiness_band }}</AppBadge>
          <p class="text-[10px] uppercase mt-2" :style="{ color: 'var(--text-3)' }">Band</p>
        </AppCard>
      </div>

      <!-- Subject breakdown -->
      <div v-if="readiness?.subjects?.length">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Subject Readiness</h3>
        <div class="space-y-3">
          <AppCard v-for="s in readiness.subjects" :key="s.subject_id" padding="md">
            <div class="flex items-center gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between mb-2">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ s.subject_name }}</p>
                  <AppBadge
                    :color="s.readiness_band === 'strong' ? 'success' : s.readiness_band === 'developing' ? 'gold' : 'danger'"
                    size="xs"
                  >
                    {{ s.readiness_band }}
                  </AppBadge>
                </div>
                <AppProgress
                  :value="s.mastered_topic_count"
                  :max="s.total_topic_count || 1"
                  size="sm"
                  :color="s.readiness_band === 'strong' ? 'success' : s.readiness_band === 'developing' ? 'gold' : 'danger'"
                />
                <p class="text-[11px] mt-1" :style="{ color: 'var(--text-3)' }">
                  {{ s.mastered_topic_count }} / {{ s.total_topic_count }} topics mastered
                </p>
              </div>
            </div>
          </AppCard>
        </div>
      </div>

      <AppCard v-else padding="lg" class="text-center mt-4">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">Complete your diagnostic to unlock analytics.</p>
        <AppButton variant="primary" size="sm" class="mt-4" @click="router.push('/student/diagnostic')">Start Diagnostic</AppButton>
      </AppCard>
    </template>
  </div>
</template>
