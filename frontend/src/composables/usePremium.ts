import { ref, computed } from 'vue'
import { ipc } from '@/ipc'

export function usePremium() {
  const loading = ref(false)
  const error = ref('')
  const riskDashboard = ref<any>(null)
  const entitlement = ref<any>(null)

  const isPremium = computed(() => entitlement.value?.tier === 'premium' || entitlement.value?.tier === 'elite')
  const isElite = computed(() => entitlement.value?.tier === 'elite')

  async function checkEntitlement(studentId: number) {
    try {
      entitlement.value = await ipc<any>('check_entitlement', { studentId })
    } catch {}
  }

  async function isFeatureEnabled(featureName: string, studentId: number): Promise<boolean> {
    try {
      return await ipc<boolean>('is_feature_enabled', { featureName, studentId })
    } catch {
      return false
    }
  }

  async function loadRiskDashboard(studentId: number) {
    loading.value = true
    try {
      riskDashboard.value = await ipc<any>('get_risk_dashboard', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load risk dashboard'
    } finally {
      loading.value = false
    }
  }

  async function autoDetectRisks(studentId: number) {
    try {
      return await ipc<any>('auto_detect_risks', { studentId })
    } catch { return null }
  }

  return { loading, error, riskDashboard, entitlement, isPremium, isElite, checkEntitlement, isFeatureEnabled, loadRiskDashboard, autoDetectRisks }
}
