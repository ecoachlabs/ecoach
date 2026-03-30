import type { BasisPoints } from './substrate'

export type ErrorType =
  | 'knowledge_gap'
  | 'conceptual_confusion'
  | 'recognition_failure'
  | 'execution_error'
  | 'carelessness'
  | 'pressure_breakdown'
  | 'expression_weakness'
  | 'speed_error'
  | 'guessing_detected'
  | 'misconception_triggered'

export type MasteryState =
  | 'unseen'
  | 'exposed'
  | 'emerging'
  | 'partial'
  | 'fragile'
  | 'stable'
  | 'robust'
  | 'exam_ready'

export interface AnswerSubmission {
  question_id: number
  selected_option_id: number
  session_id: number | null
  session_type: string | null
  started_at: string
  submitted_at: string
  response_time_ms: number | null
  confidence_level: string | null
  hint_count: number
  changed_answer_count: number
  skipped: boolean
  timed_out: boolean
  support_level: string | null
  was_timed: boolean
  was_transfer_variant: boolean
  was_retention_check: boolean
  was_mixed_context: boolean
}

export interface AnswerProcessingResult {
  is_correct: boolean
  error_type: ErrorType | null
  diagnosis_summary: string | null
  recommended_action: string | null
  explanation: string | null
  selected_option_text: string
  correct_option_text: string | null
  updated_mastery: BasisPoints
  updated_gap: BasisPoints
  misconception_info: string | null
}

export interface StudentTopicState {
  id: number
  student_id: number
  topic_id: number
  mastery_score: BasisPoints
  mastery_state: MasteryState
  accuracy_score: BasisPoints
  speed_score: BasisPoints
  confidence_score: BasisPoints
  retention_score: BasisPoints
  transfer_score: BasisPoints
  consistency_score: BasisPoints
  gap_score: BasisPoints
  priority_score: BasisPoints
  trend_state: string
  fragility_score: BasisPoints
  pressure_collapse_index: BasisPoints
  total_attempts: number
  correct_attempts: number
  evidence_count: number
  last_seen_at: string | null
  last_correct_at: string | null
  memory_strength: BasisPoints
  next_review_at: string | null
  version: number
}

export interface LearnerTruthTopicSummary {
  topic_id: number
  topic_name: string
  mastery_score: BasisPoints
  mastery_state: MasteryState
  gap_score: BasisPoints
  priority_score: BasisPoints
  memory_strength: BasisPoints
  next_review_at: string | null
}

export interface LearnerTruthSkillSummary {
  node_id: number
  title: string
  mastery_score: BasisPoints
  gap_score: BasisPoints
  priority_score: BasisPoints
  state: string
}

export interface LearnerTruthMemorySummary {
  topic_id: number
  topic_name: string
  node_id: number
  node_title: string
  memory_state: string
  memory_strength: BasisPoints
  recall_fluency: BasisPoints
  decay_risk: BasisPoints
  review_due_at: string | null
}

export interface LearnerTruthDiagnosisSummary {
  diagnosis_id: number
  topic_id: number
  topic_name: string
  primary_diagnosis: string
  severity: string
  recommended_action: string
  created_at: string
}

export interface LearnerTruthSnapshot {
  student_id: number
  student_name: string
  overall_mastery_score: BasisPoints
  overall_readiness_band: string
  pending_review_count: number
  due_memory_count: number
  topic_summaries: LearnerTruthTopicSummary[]
  skill_summaries: LearnerTruthSkillSummary[]
  memory_summaries: LearnerTruthMemorySummary[]
  recent_diagnoses: LearnerTruthDiagnosisSummary[]
}
