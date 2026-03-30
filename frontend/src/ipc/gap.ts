import { ipc } from '.'

export function listPriorityGaps(studentId: number): Promise<any[]> {
  return ipc('list_priority_gaps', { studentId })
}

export function generateRepairPlan(studentId: number, topicId: number): Promise<any> {
  return ipc('generate_repair_plan', { studentId, topicId })
}

export function advanceRepairItem(itemId: number, result: any): Promise<any> {
  return ipc('advance_repair_item', { itemId, result })
}

export function getGapDashboard(studentId: number): Promise<any> {
  return ipc('get_gap_dashboard', { studentId })
}
