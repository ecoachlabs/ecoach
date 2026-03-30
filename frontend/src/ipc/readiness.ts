import { ipc } from '.'

export function getReadinessReport(studentId: number): Promise<any> {
  return ipc('get_readiness_report', { studentId })
}

export function generateParentDigest(parentId: number): Promise<any> {
  return ipc('generate_parent_digest', { parentId })
}
