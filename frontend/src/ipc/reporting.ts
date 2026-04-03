import { ipc } from '.'
import type { StudentDashboard, ParentDashboardSnapshot } from '@/types'

export function getStudentDashboard(studentId: number): Promise<StudentDashboard> {
  return ipc<StudentDashboard>('get_student_dashboard', { studentId })
}

export function buildParentDashboard(parentId: number): Promise<ParentDashboardSnapshot> {
  return ipc<ParentDashboardSnapshot>('get_parent_dashboard', { parentId })
}
