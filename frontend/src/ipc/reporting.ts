import { ipc } from '.'
import type { StudentDashboard, ParentDashboardSnapshot } from '@/types'

export type { StudentDashboard, ParentDashboardSnapshot } from '@/types'

export function getStudentDashboard(studentId: number): Promise<StudentDashboard> {
  return ipc<StudentDashboard>('get_student_dashboard', { studentId })
}

export function buildParentDashboard(parentId: number): Promise<ParentDashboardSnapshot> {
  return ipc<ParentDashboardSnapshot>('get_parent_dashboard', { parentId })
}

export interface HouseholdActionItemDto {
  urgency: string
  title: string
  detail: string
}

export interface HouseholdInterventionSummaryDto {
  intervention_id: number
  title: string
  status: string
  linked_risk_title: string | null
  risk_severity: string | null
  progress_percent: number
  next_step: string | null
  target_topic_name: string | null
  updated_at: string
}

export interface HouseholdStudentSnapshotDto {
  student_id: number
  student_name: string
  overall_readiness_band: string
  attention_level: string
  exam_target: string | null
  exam_target_date: string | null
  active_risks: Array<{
    severity: string
    title: string
    description: string
  }>
  active_interventions: HouseholdInterventionSummaryDto[]
  household_actions: HouseholdActionItemDto[]
  weekly_memo: string
  subject_summaries: Array<{
    subject_id: number
    subject_name: string
    readiness_band: string
    mastered_topic_count: number
    weak_topic_count: number
    total_topic_count: number
  }>
}

export interface HouseholdDashboardSnapshotDto {
  parent_id: number
  parent_name: string
  household_attention_level: string
  students_needing_attention: number
  active_interventions: number
  household_actions: HouseholdActionItemDto[]
  students: HouseholdStudentSnapshotDto[]
  generated_at: string
}

export function getHouseholdDashboard(parentId: number): Promise<HouseholdDashboardSnapshotDto> {
  return ipc<HouseholdDashboardSnapshotDto>('get_household_dashboard', { parentId })
}
