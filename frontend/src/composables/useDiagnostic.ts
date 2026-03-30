import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useDiagnostic() {
  const loading = ref(false)
  const error = ref('')
  const diagnosticId = ref<number | null>(null)
  const report = ref<any>(null)

  async function launch(studentId: number, subjectId: number, mode: string) {
    loading.value = true
    error.value = ''
    try {
      const result = await ipc<any>('launch_diagnostic', { studentId, subjectId, mode })
      diagnosticId.value = result.diagnostic_id ?? result.id
      return result
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to launch diagnostic'
      return null
    } finally {
      loading.value = false
    }
  }

  async function getReport(diagId: number) {
    loading.value = true
    try {
      report.value = await ipc<any>('get_diagnostic_report', { diagnosticId: diagId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load report'
    } finally {
      loading.value = false
    }
  }

  return { loading, error, diagnosticId, report, launch, getReport }
}
