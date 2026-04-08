<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'
import { generateParentDigest } from '@/ipc/readiness'
import AppButton from '@/components/ui/AppButton.vue'

const auth = useAuthStore()

const loading = ref(true)
const error = ref('')
const generating = ref(false)
const digestResult = ref<string | null>(null)
const dashboard = ref<ParentDashboardSnapshot | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    dashboard.value = await buildParentDashboard(auth.currentAccount.id)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load data'
  }
  loading.value = false
})

async function generateDigest() {
  if (!auth.currentAccount || generating.value) return
  generating.value = true
  error.value = ''
  try {
    await generateParentDigest(auth.currentAccount.id)
    digestResult.value = 'Digest generated successfully.'
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to generate digest'
  }
  generating.value = false
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Family</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Reports</h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Progress summaries and AI-generated digests</p>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>
    <div v-if="digestResult" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(13,148,136,0.08); color: var(--accent);">{{ digestResult }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7 space-y-5">

      <!-- Weekly Digest CTA -->
      <div class="px-6 py-5 rounded-2xl"
        :style="{ backgroundColor: 'var(--accent-glow)', boxShadow: 'var(--shadow-xs)' }">
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-sm font-bold mb-1" :style="{ color: 'var(--ink)' }">Weekly Parent Digest</p>
            <p class="text-xs max-w-sm" :style="{ color: 'var(--ink-muted)' }">
              AI-generated summary of your children's progress, trends, and recommended actions.
            </p>
          </div>
          <AppButton variant="primary" size="sm" :loading="generating" @click="generateDigest">
            Generate
          </AppButton>
        </div>
      </div>

      <!-- Child snapshots -->
      <template v-if="!loading && dashboard">
        <p class="section-label">Child Snapshots</p>
        <div class="space-y-3">
          <div
            v-for="student in dashboard.students"
            :key="student.student_id"
            class="snapshot-card px-5 py-4 rounded-2xl"
            :style="{ backgroundColor: 'var(--surface)', boxShadow: 'var(--shadow-xs)' }"
          >
            <div class="flex items-start justify-between gap-4 mb-3">
              <div>
                <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ student.student_name }}</p>
                <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                  Readiness: {{ student.overall_readiness_band.replace(/_/g, ' ') }}
                  <template v-if="student.exam_target"> · Target: {{ student.exam_target }}</template>
                </p>
              </div>
            </div>
            <p v-if="student.weekly_memo" class="text-xs mb-3 leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">
              {{ student.weekly_memo }}
            </p>
            <div v-if="student.trend_summary?.length" class="space-y-1 pt-3 border-t" :style="{ borderColor: 'var(--border-soft)' }">
              <p v-for="trend in student.trend_summary.slice(0, 3)" :key="trend"
                class="text-xs" :style="{ color: 'var(--ink-muted)' }">
                · {{ trend }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div v-else-if="loading" class="space-y-3">
        <div v-for="i in 2" :key="i" class="h-24 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else class="text-center py-8">
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No children linked to your account yet.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.section-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.14em; color: var(--ink-muted); }
.snapshot-card { transition: background-color 100ms, box-shadow 100ms; }
.snapshot-card:hover { background-color: var(--paper) !important; }
</style>

