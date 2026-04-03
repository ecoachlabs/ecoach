import { ipc } from '.'

// ── DTOs ────────────────────────────────────────────────────────────────────

export interface QuestionOptionDto {
  id: number
  label: string
  text: string
  misconception_id: number | null
  distractor_intent: string | null
}

export interface SessionQuestionDto {
  item_id: number
  question_id: number
  display_order: number
  stem: string
  question_format: string
  difficulty: number
  estimated_time_seconds: number | null
  is_answered: boolean
  flagged: boolean
  options: QuestionOptionDto[]
}

export interface SubmitAttemptInput {
  student_id: number
  session_id: number
  session_item_id: number
  question_id: number
  selected_option_id: number
  response_time_ms: number | null
  confidence_level: string | null
  hint_count: number
  changed_answer_count: number
  was_timed: boolean
}

export interface AttemptResultDto {
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
