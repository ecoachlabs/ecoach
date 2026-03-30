/** Matches SessionSnapshotDto from ecoach-commands */
export interface SessionSnapshotDto {
  session_id: number
  session_type: string
  status: string
  active_item_index: number
  item_count: number
}

/** Matches SessionSummaryDto from ecoach-commands */
export interface SessionSummaryDto {
  session_id: number
  accuracy_score: number | null
  answered_questions: number
  correct_questions: number
  status: string
}

/** Matches PracticeSessionStartInput from ecoach-sessions */
export interface PracticeSessionStartInput {
  student_id: number
  subject_id: number
  topic_ids: number[]
  question_count: number
  is_timed: boolean
}

/** Matches CustomTestStartInput from ecoach-sessions */
export interface CustomTestStartInput {
  student_id: number
  subject_id: number
  topic_ids: number[]
  question_count: number
  duration_minutes: number | null
  is_timed: boolean
  target_difficulty: number | null
  weakness_bias: boolean
}

/** Matches MockBlueprintDto from ecoach-commands */
export interface MockBlueprintDto {
  id: number
  title: string
  blueprint_type: string
  question_count: number
  readiness_score: number
  readiness_band: string
  coverage: Record<string, unknown>
  status: string
}

/** Matches MockBlueprintInput from ecoach-sessions */
export interface MockBlueprintInput {
  student_id: number
  subject_id: number
  blueprint_type: string
}

/** Matches PackSummaryDto from ecoach-commands */
export interface PackSummaryDto {
  pack_id: string
  pack_version: string
  subject_code: string
  status: string
}

/** Matches PackInstallResultDto from ecoach-commands */
export interface PackInstallResultDto {
  pack_id: string
  pack_version: string
  install_path: string
}
