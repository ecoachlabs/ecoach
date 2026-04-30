import { ipc } from '.'

// ── DTOs ────────────────────────────────────────────────────────────────────

export interface QuestionOptionDto {
  id: number
  label: string
  text: string
  is_correct: boolean
  misconception_id: number | null
  distractor_intent: string | null
}

export interface SessionQuestionDto {
  item_id: number
  question_id: number
  display_order: number
  stem: string
  explanation_text: string | null
  question_format: string
  difficulty: number
  estimated_time_seconds: number | null
  is_answered: boolean
  flagged: boolean
  /** Exam year of the past paper this question came from (null when the
   *  question is not sourced from a past paper). */
  paper_exam_year: number | null
  /** Question number as printed on the original past paper (null when the
   *  question is not sourced from a past paper). */
  paper_question_number: string | null
  options: QuestionOptionDto[]
}

export interface SubmitAttemptInput {
  student_id: number
  session_id: number
  session_item_id: number
  question_id: number
  selected_option_id: number | null
  response_time_ms: number | null
  confidence_level: string | null
  hint_count: number
  changed_answer_count: number
  skipped?: boolean
  timed_out?: boolean
  was_timed: boolean
  defer_coach_brain?: boolean
  /** Multi-correct MCQ: all option ids the student selected. Backend
   *  grades by set equality against `is_correct=1` options. */
  selected_option_ids?: number[] | null
  /** Fill-in-the-blank: student's input per blank, ordered by blank
   *  index (1-based). Backend grades each against the acceptable
   *  answers stored in `question_options`. */
  blank_answers?: string[] | null
}

export interface AttemptResultDto {
  attempt_id: number
  is_correct: boolean
  explanation: string | null
  correct_option_text: string | null
  selected_option_text: string | null
  misconception_info: string | null
  error_type: string | null
  diagnosis_summary: string | null
  recommended_action: string | null
  updated_mastery: number
  updated_gap: number
  session_answered: number
  session_remaining: number
  session_complete: boolean
  next_action_type: string
  next_action_title: string
  next_action_route: string
}

export interface QuestionFamilyHealthDto {
  family_id: number
  total_instances: number
  generated_instances: number
  active_instances: number
  recent_attempts: number
  recent_correct_attempts: number
  avg_response_time_ms: number
  misconception_hit_count: number
  freshness_score: number
  calibration_score: number
  quality_score: number
  health_status: string
}

// ── Commands ─────────────────────────────────────────────────────────────────

export function getQuestionOptions(questionId: number): Promise<QuestionOptionDto[]> {
  return ipc<QuestionOptionDto[]>('get_question_options', { questionId })
}

export function listSessionQuestions(sessionId: number): Promise<SessionQuestionDto[]> {
  return ipc<SessionQuestionDto[]>('list_session_questions', { sessionId })
}

export function submitAttempt(input: SubmitAttemptInput): Promise<AttemptResultDto> {
  return ipc<AttemptResultDto>('submit_attempt', { input })
}

export function getQuestionFamilyHealth(
  familyId: number,
): Promise<QuestionFamilyHealthDto | null> {
  return ipc<QuestionFamilyHealthDto | null>('get_question_family_health', { familyId })
}
