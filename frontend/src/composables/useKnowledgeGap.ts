import { ref } from 'vue'
import { ipc } from '@/ipc'

export function useKnowledgeGap() {
  const loading = ref(false)
  const error = ref('')
  const gapDashboard = ref<any>(null)
  const priorities = ref<any[]>([])
  const repairPlan = ref<any>(null)

  async function loadDashboard(studentId: number) {
    loading.value = true
    try {
      gapDashboard.value = await ipc<any>('get_gap_dashboard', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load gap dashboard'
    } finally {
      loading.value = false
    }
  }

  async function loadPriorities(studentId: number) {
    try {
      priorities.value = await ipc<any[]>('list_priority_gaps', { studentId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load gap priorities'
    }
  }

  async function generateRepairPlan(studentId: number, topicId: number) {
    loading.value = true
    try {
      repairPlan.value = await ipc<any>('generate_repair_plan', { studentId, topicId })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to generate repair plan'
    } finally {
      loading.value = false
    }
  }

  async function advanceRepairItem(itemId: number, result: any) {
    try {
      return await ipc<any>('advance_repair_item', { itemId, result })
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to advance repair'
      return null
    }
  }

  return { loading, error, gapDashboard, priorities, repairPlan, loadDashboard, loadPriorities, generateRepairPlan, advanceRepairItem }
}
