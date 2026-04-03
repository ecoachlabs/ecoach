import { ipc } from '.'

// ── DTOs ────────────────────────────────────────────────────────────────────

export interface DiagnosticRunDto {
  diagnostic_id: number
}

export interface DiagnosticBatteryDto {
  diagnostic_id: number
  student_id: number
  subject_id: number
  session_mode: string
  status: string
  phases: DiagnosticPhasePlanDto[]
}

export interface DiagnosticPhasePlanDto {
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

export interface DiagnosticPhaseItemDto {
  attempt_id: number
  phase_id: number
  question_id: number
  display_order: number
  condition_type: string
  stem: string
  question_format: string
  topic_id: number
}

export interface SubmitDiagnosticAttemptInput {
  attempt_id: number
  selected_option_id: number | null
  response_time_ms: number | null
  confidence_level: string | null
  changed_answer_count: number
  skipped: boolean
  timed_out: boolean
  first_focus_at: string | null
  first_input_at: string | null
  concept_guess: string | null
  final_answer: unknown | null
  interaction_log: unknown | null
}

export interface DiagnosticCompletionSyncDto {
  diagnostic_id: number
  overall_readiness: number
  readiness_band: string
  analytics: TopicAnalyticsDto[]
  top_hypotheses: DiagnosticRootCauseHypothesisDto[]
}

export interface TopicAnalyticsDto {
  topic_id: number
  topic_name: string
  classification: string
  mastery_score: number
  confidence_score: number
  recommended_action: string
  endurance_score: number
  weakness_type: string | null
  failure_stage: string | null
}

export interface DiagnosticRootCauseHypothesisDto {
  topic_id: number
  topic_name: string
  hypothesis_code: string
  confidence_score: number
  recommended_action: string
}

export interface DiagnosticResultDto {
  diagnostic_id: number
  overall_readiness: number
  readiness_band: string
  completed_at: string | null
}

// ── Commands ─────────────────────────────────────────────────────────────────

export function launchDiagnostic(
  studentId: number,
  subjectId: number,
  mode: string,
): Promise<DiagnosticRunDto> {
  return ipc<DiagnosticRunDto>('launch_diagnostic', {
    studentId,
    subjectId,
    mode,
  })
}

export function getDiagnosticBattery(diagnosticId: number): Promise<DiagnosticBatteryDto> {
  return ipc<DiagnosticBatteryDto>('get_diagnostic_battery', { diagnosticId })
}

export function listDiagnosticPhaseItems(
  diagnosticId: number,
  phaseNumber: number,
): Promise<DiagnosticPhaseItemDto[]> {
  return ipc<DiagnosticPhaseItemDto[]>('list_diagnostic_phase_items', {
    diagnosticId,
    phaseNumber,
  })
}

export function submitDiagnosticAttempt(
  diagnosticId: number,
  input: SubmitDiagnosticAttemptInput,
): Promise<void> {
  return ipc<void>('submit_diagnostic_attempt', { diagnosticId, input })
}

export function advanceDiagnosticPhase(
  diagnosticId: number,
  completedPhaseNumber: number,
): Promise<DiagnosticPhasePlanDto | null> {
  return ipc<DiagnosticPhasePlanDto | null>('advance_diagnostic_phase', {
    diagnosticId,
    completedPhaseNumber,
  })
}

export function completeDiagnosticAndSync(
  diagnosticId: number,
): Promise<DiagnosticCompletionSyncDto> {
  return ipc<DiagnosticCompletionSyncDto>('complete_diagnostic_and_sync', { diagnosticId })
}

export function getDiagnosticReport(diagnosticId: number): Promise<TopicAnalyticsDto[]> {
  return ipc<TopicAnalyticsDto[]>('get_diagnostic_report', { diagnosticId })
}

export function getDiagnosticResult(diagnosticId: number): Promise<DiagnosticResultDto | null> {
  return ipc<DiagnosticResultDto | null>('get_diagnostic_result', { diagnosticId })
}
