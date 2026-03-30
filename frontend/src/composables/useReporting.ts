import { ref } from 'vue'
import { ipc } from '@/ipc'
import { getStudentDashboard, type StudentDashboardDto } from '@/ipc/coach'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'

export function useReporting() {
  const loading = ref(false)
  const error = ref('')
  const studentDash = ref<StudentDashboardDto | null>(null)
  const parentDash = ref<ParentDashboardSnapshot | null>(null)
  const readinessReport = ref<any>(null)
  const parentDigest = ref<any>(null)

  async function loadStudentDashboard(studentId: number) {
    loading.value = true
    try {
      studentDash.value = await getStudentDashboard(studentId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load dashboard'
    } finally {
      loading.value = false
    }
  }

  async function loadParentDashboard(parentId: number) {
    loading.value = true
    try {
      parentDash.value = await buildParentDashboard(parentId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load parent dashboard'
    } finally {
      loading.value = false
    }
  }

  async function getReadinessReport(studentId: number) {
    try {
      readinessReport.value = await ipc<any>('get_readiness_report', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load readiness report'
    }
  }

  async function generateParentDigest(parentId: number) {
    try {
      parentDigest.value = await ipc<any>('generate_parent_digest', { parentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to generate digest'
    }
  }

  return { loading, error, studentDash, parentDash, readinessReport, parentDigest, loadStudentDashboard, loadParentDashboard, getReadinessReport, generateParentDigest }
}
