export interface SubjectSummary {
  subject_id: number
  subject_name: string
  readiness_band: string
  mastered_topic_count: number
  weak_topic_count: number
  total_topic_count: number
}

export interface StudentDashboard {
  student_name: string
  exam_target: string | null
  subject_summaries: SubjectSummary[]
  overall_readiness_band: string
}

export interface ParentRiskSummary {
  severity: string
  title: string
  description: string
}

export interface ParentStudentSummary {
  student_id: number
  student_name: string
  overall_readiness_band: string
  exam_target: string | null
  active_risks: ParentRiskSummary[]
  recommendations: string[]
  trend_summary: string[]
  weekly_memo: string
  subject_summaries: SubjectSummary[]
}

export interface ParentDashboardSnapshot {
  parent_id: number
  parent_name: string
  students: ParentStudentSummary[]
  generated_at: string
}
