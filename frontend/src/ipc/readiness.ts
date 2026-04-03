import { ipc } from '.'

export interface SubjectReadinessDto {
  subject_id: number
  subject_name: string
  readiness_band: string
  total_topic_count: number
  mastered_topic_count: number
}

export interface ReadinessReportDto {
  student_id: number
  overall_readiness_band: string
  coverage_percent: number
  subjects: SubjectReadinessDto[]
}

export function getReadinessReport(studentId: number): Promise<ReadinessReportDto> {
  return ipc<ReadinessReportDto>('get_readiness_report', { studentId })
}

export function generateParentDigest(parentId: number): Promise<any> {
  return ipc('generate_parent_digest', { parentId })
}
