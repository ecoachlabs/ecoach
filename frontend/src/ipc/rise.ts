import { ipc } from '.'

export interface RiseModeProfileDto {
  student_id: number
  subject_id: number
  current_stage: string
  stage_label: string
  stage_purpose: string
  foundation_score: number
  recall_score: number
  speed_score: number
  accuracy_score: number
  pressure_stability_score: number
  misconception_density_score: number
  momentum_score: number
  transformation_readiness_score: number
  confidence_score: number
  weakness_map: Record<string, unknown>
  recovery_plan: Record<string, unknown>
}

export interface StageTransitionResultDto {
  previous_stage: string
  new_stage: string
  reason: string
  next_stage_purpose: string
}

export function enterRiseMode(
  studentId: number,
  subjectId: number,
): Promise<RiseModeProfileDto> {
  return ipc<RiseModeProfileDto>('enter_rise_mode', { studentId, subjectId })
}

export function getRiseModeProfile(
  studentId: number,
  subjectId: number,
): Promise<RiseModeProfileDto | null> {
  return ipc<RiseModeProfileDto | null>('get_rise_mode_profile', { studentId, subjectId })
}

export function checkRiseModeTransition(
  studentId: number,
  subjectId: number,
): Promise<StageTransitionResultDto | null> {
  return ipc<StageTransitionResultDto | null>('check_rise_mode_transition', { studentId, subjectId })
}
