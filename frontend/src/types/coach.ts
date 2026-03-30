import type { BasisPoints } from './substrate'

export type LearnerJourneyState =
  | 'onboarding_required'
  | 'subject_selection_required'
  | 'content_readiness_required'
  | 'diagnostic_required'
  | 'plan_generation_required'
  | 'ready_for_today_mission'
  | 'mission_in_progress'
  | 'mission_review_required'
  | 'repair_required'
  | 'blocked_on_topic'
  | 'plan_adjustment_required'
  | 'review_day'
  | 'exam_mode'
  | 'stalled_no_content'

export type ContentReadinessStatus =
  | 'ready'
  | 'no_subjects_selected'
  | 'no_packs_installed'
  | 'no_topics_available'
  | 'topics_exist_but_no_questions'
  | 'insufficient_question_coverage'

export type CoachActionType =
  | 'continue_onboarding'
  | 'select_subjects'
  | 'resolve_content'
  | 'start_diagnostic'
  | 'generate_plan'
  | 'start_today_mission'
  | 'resume_mission'
  | 'review_results'
  | 'start_repair'
  | 'adjust_plan'
  | 'view_overview'

export interface CoachStateResolution {
  state: LearnerJourneyState
  reason: string
}

export interface ContentReadinessResolution {
  status: ContentReadinessStatus
  subject_codes: string[]
  active_pack_count: number
  topic_count: number
  question_count: number
  reason: string | null
}

export interface CoachNextAction {
  state: LearnerJourneyState
  action_type: CoachActionType
  title: string
  subtitle: string
  estimated_minutes: number | null
  route: string
  context: Record<string, unknown>
}

export interface TopicCaseBlocker {
  reason: string
  severity: string
}

export interface TopicCaseDiagnosis {
  diagnosis_id: number
  error_type: string
  primary_diagnosis: string
  severity: string
  diagnosis_summary: string
  recommended_action: string
  confidence_score: BasisPoints
  created_at: string
}

export interface TopicCaseHypothesis {
  code: string
  label: string
  confidence_score: BasisPoints
  evidence_summary: string
  recommended_probe: string | null
  recommended_response: string
}

export interface TopicCaseIntervention {
  mode: string
  urgency: string
  next_action_type: string
  recommended_minutes: number
  reason: string
}

export interface TopicCase {
  student_id: number
  topic_id: number
  topic_name: string
  subject_code: string
  priority_score: BasisPoints
  mastery_score: BasisPoints
  mastery_state: string
  gap_score: BasisPoints
  fragility_score: BasisPoints
  pressure_collapse_index: BasisPoints
  memory_state: string
  memory_strength: BasisPoints
  decay_risk: BasisPoints
  evidence_count: number
  recent_attempt_count: number
  recent_accuracy: BasisPoints | null
  active_blocker: TopicCaseBlocker | null
  recent_diagnoses: TopicCaseDiagnosis[]
  active_hypotheses: TopicCaseHypothesis[]
  primary_hypothesis_code: string
  diagnosis_certainty: BasisPoints
  requires_probe: boolean
  recommended_intervention: TopicCaseIntervention
  proof_gaps: string[]
  open_questions: string[]
}
