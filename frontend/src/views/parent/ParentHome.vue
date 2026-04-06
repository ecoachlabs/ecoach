<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'
import ChildCard from '@/components/parent/ChildCard.vue'
import WeeklyMemo from '@/components/parent/WeeklyMemo.vue'

const auth = useAuthStore()
const router = useRouter()
const dashboard = ref<ParentDashboardSnapshot | null>(null)
const loading = ref(true)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    dashboard.value = await buildParentDashboard(auth.currentAccount.id)
  } catch (e) {
    console.error('Failed to load parent dashboard:', e)
  }
  loading.value = false
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Family Overview</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Your children's progress
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Academic snapshot at a glance</p>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/parent/children')">Manage Children</button>
        <button class="nav-pill" @click="router.push('/parent/household')">Household</button>
        <button class="nav-pill" @click="router.push('/parent/curriculum')">Curriculum</button>
        <button class="nav-pill" @click="router.push('/parent/concierge')">Concierge</button>
      </div>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7">
      <div v-if="loading" class="space-y-4">
        <div v-for="i in 2" :key="i" class="h-32 rounded-xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <template v-else-if="dashboard">
        <div class="space-y-4 mb-8">
          <ChildCard
            v-for="student in dashboard.students"
            :key="student.student_id"
            :student-id="student.student_id"
            :student-name="student.student_name"
            :readiness-band="student.overall_readiness_band"
            :exam-target="student.exam_target ?? undefined"
            :risks="student.active_risks"
            :recommendations="student.recommendations"
            @click="router.push('/parent/child/' + student.student_id)"
          />
        </div>

        <WeeklyMemo v-if="dashboard.students[0]?.weekly_memo"
          :date="new Date().toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })"
          :content="dashboard.students[0].weekly_memo"
          :highlights="dashboard.students[0].trend_summary?.filter((_, i) => i < 3)"
          :concerns="dashboard.students[0].active_risks?.map(r => r.title)"
        />
      </template>

      <div v-else class="flex flex-col items-center justify-center h-64 gap-4">
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No children linked to your account yet.</p>
        <button class="px-5 py-2.5 rounded-xl font-semibold text-sm"
          :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
          @click="router.push('/parent/children')">Link a Child</button>
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
  color: var(--accent);
  margin-bottom: 4px;
}

.nav-pill {
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
</style>
