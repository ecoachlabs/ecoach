import type { BasisPoints } from './substrate'

export type DiagnosticMode = 'quick' | 'standard' | 'deep'

export type DiagnosticPhaseCode =
  | 'baseline'
  | 'speed'
  | 'precision'
  | 'pressure'
  | 'flex'
  | 'root_cause'

export interface TopicDiagnosticResult {
  topic_id: number
  topic_name: string
  mastery_score: BasisPoints
  fluency_score: BasisPoints
  precision_score: BasisPoints
  pressure_score: BasisPoints
  flexibility_score: BasisPoints
  stability_score: BasisPoints
  classification: string
}

export interface WrongAnswerDiagnosis {
  id: number
  student_id: number
  question_id: number
  topic_id: number
  error_type: string
  primary_diagnosis: string
  secondary_diagnosis: string | null
  severity: string
  diagnosis_summary: string
  recommended_action: string
  confidence_score: BasisPoints
  created_at: string
}

export interface DiagnosticResult {
  overall_readiness: BasisPoints
  readiness_band: string
  topic_results: TopicDiagnosticResult[]
  recommended_next_actions: string[]
}

export interface DiagnosticPhasePlan {
  phase_id: number
  phase_number: number
  phase_code: string
  phase_title: string
  phase_type: string
  status: string
  question_count: number
  time_limit_seconds: number | null
  condition_type: string
}

export interface DiagnosticPhaseItem {
  phase_id: number
  question_id: number
  display_order: number
  condition_type: string
  stem: string
  question_format: string
  topic_id: number
}

export interface DiagnosticBattery {
  diagnostic_id: number
  student_id: number
  subject_id: number
  session_mode: string
  status: string
  phases: DiagnosticPhasePlan[]
}
