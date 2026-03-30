<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'
import PageHeader from '@/components/layout/PageHeader.vue'
import ChildCard from '@/components/parent/ChildCard.vue'
import WeeklyMemo from '@/components/parent/WeeklyMemo.vue'
import AppButton from '@/components/ui/AppButton.vue'

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
  <div class="reveal-stagger">
    <PageHeader title="Family Overview" subtitle="Your children's academic progress at a glance." />

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 2" :key="i" class="h-32 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <template v-else-if="dashboard">
      <!-- Child cards -->
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

      <!-- Weekly memo (first child) -->
      <WeeklyMemo v-if="dashboard.students[0]?.weekly_memo"
        :date="new Date().toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })"
        :content="dashboard.students[0].weekly_memo"
        :highlights="dashboard.students[0].trend_summary?.filter((_, i) => i < 3)"
        :concerns="dashboard.students[0].active_risks?.map(r => r.title)"
      />
    </template>

    <div v-else class="text-center py-16">
      <p class="text-sm" :style="{color:'var(--text-3)'}">No children linked to your account yet.</p>
    </div>
  </div>
</template>
