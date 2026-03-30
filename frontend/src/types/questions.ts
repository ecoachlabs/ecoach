import type { BasisPoints } from './substrate'

export interface Question {
  id: number
  subject_id: number
  topic_id: number
  subtopic_id: number | null
  family_id: number | null
  stem: string
  question_format: string
  explanation_text: string | null
  difficulty_level: BasisPoints
  estimated_time_seconds: number
  marks: number
  primary_skill_id: number | null
}

export interface QuestionOption {
  id: number
  question_id: number
  option_label: string
  option_text: string
  is_correct: boolean
  misconception_id: number | null
  distractor_intent: string | null
  position: number
}

export interface QuestionSelectionRequest {
  subject_id: number
  topic_ids: number[]
  target_question_count: number
  target_difficulty: BasisPoints | null
  weakness_topic_ids: number[]
  recently_seen_question_ids: number[]
  timed: boolean
}

export interface SelectedQuestion {
  question: Question
  fit_score: number
}
