<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getDiagnosticReport } from '@/ipc/diagnostic'
import PageHeader from '@/components/layout/PageHeader.vue'
import DiagnosticReport from '@/components/diagnostic/DiagnosticReport.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { usePdf } from '@/composables/usePdf'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const { generating, exportToPdf } = usePdf()

const diagnosticId = computed(() => Number(route.params.id))
const loading = ref(true)
const report = ref<any>(null)

onMounted(async () => {
  try {
    report.value = await getDiagnosticReport(diagnosticId.value)
  } catch (e) {
    console.error('Failed to load report:', e)
  }
  loading.value = false
})

function handleExport() {
  exportToPdf('diagnostic-report', `diagnostic-report-${diagnosticId.value}.pdf`)
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto">
    <PageHeader title="Diagnostic Report" subtitle="Your comprehensive academic profile." back-to="/student/diagnostic">
      <template #actions>
        <AppButton variant="secondary" size="sm" :loading="generating" @click="handleExport">Export PDF</AppButton>
      </template>
    </PageHeader>

    <div v-if="loading" class="space-y-4">
      <div v-for="i in 4" :key="i" class="h-24 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <div v-else-if="report" id="diagnostic-report">
      <DiagnosticReport
        :overall-readiness="report.overall_readiness ?? 4800"
        :readiness-band="report.readiness_band ?? 'developing'"
        :topic-results="report.topic_results ?? []"
        :recommended-actions="report.recommended_next_actions ?? ['Practice weak topics', 'Take a mock test']"
        @close="router.push('/student')"
        @export="handleExport"
      />
    </div>

    <div v-else class="text-center py-16">
      <p class="text-sm" :style="{color:'var(--text-3)'}">Report not available. Complete a diagnostic first.</p>
      <AppButton variant="primary" class="mt-4" @click="router.push('/student/diagnostic')">Start Diagnostic</AppButton>
    </div>
  </div>
</template>
