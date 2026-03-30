<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import * as reportingIpc from '@/ipc/reporting'
import type { ParentDashboardSnapshot } from '@/types'

const auth = useAuthStore()
const dashboard = ref<ParentDashboardSnapshot | null>(null)
const loading = ref(true)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    dashboard.value = await reportingIpc.buildParentDashboard(auth.currentAccount.id)
  } catch {
    // Handle error
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div>
    <h1 class="text-2xl font-semibold mb-6" :style="{ color: 'var(--text)' }">Family Overview</h1>

    <div v-if="loading" class="text-center py-16">
      <p :style="{ color: 'var(--text-muted)' }">Loading...</p>
    </div>

    <div v-else-if="dashboard" class="space-y-4">
      <div
        v-for="student in dashboard.students"
        :key="student.student_id"
        class="p-6 rounded-xl border"
        :style="{ backgroundColor: 'var(--surface)', borderColor: 'var(--border)' }"
      >
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-lg font-medium" :style="{ color: 'var(--text)' }">{{ student.student_name }}</h2>
          <span class="text-sm capitalize px-3 py-1 rounded-full bg-blue-50 text-blue-700">
            {{ student.overall_readiness_band }}
          </span>
        </div>

        <div v-if="student.active_risks.length" class="space-y-2 mb-3">
          <div
            v-for="(risk, i) in student.active_risks"
            :key="i"
            class="text-sm p-3 rounded-lg"
            :class="{
              'bg-red-50 text-red-700': risk.severity === 'high',
              'bg-yellow-50 text-yellow-700': risk.severity === 'medium',
              'bg-blue-50 text-blue-700': risk.severity === 'low',
            }"
          >
            <strong>{{ risk.title }}</strong>: {{ risk.description }}
          </div>
        </div>

        <div v-if="student.recommendations.length" class="space-y-1">
          <p v-for="(rec, i) in student.recommendations" :key="i" class="text-sm" :style="{ color: 'var(--text-secondary)' }">
            {{ rec }}
          </p>
        </div>

        <RouterLink
          :to="'/parent/child/' + student.student_id"
          class="inline-block mt-3 text-sm font-medium"
          :style="{ color: 'var(--info)' }"
        >
          View details &rarr;
        </RouterLink>
      </div>
    </div>
  </div>
</template>
