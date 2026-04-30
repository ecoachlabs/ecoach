import { ipc } from '.'

export interface GameSessionDto {
  id: number
  game_type: string
  session_state: string
  score: number
  rounds_total: number
  rounds_played: number
  streak: number
  best_streak: number
}

export interface GameAnswerResultDto {
  is_correct: boolean
  points_earned: number
  new_score: number
  streak: number
  effect_type: string
  round_number: number
  session_complete: boolean
  explanation: string | null
  misconception_triggered: boolean
}

export interface GameSummaryDto {
  session_id: number
  game_type: string
  score: number
  accuracy_bp: number
  rounds_played: number
  best_streak: number
  average_response_time_ms: number
  misconception_hits: number
  performance_label: string
  focus_signals: string[]
  recommended_next_step: string | null
}

export interface LeaderboardEntryDto {
  student_id: number
  display_name: string
  best_score: number
  games_played: number
}

export interface MindstackStateDto {
  board_height: number
  cleared_rows: number
  pending_block_type: string
}

export interface TugOfWarStateDto {
  position: number
  opponent_difficulty: number
}

export interface TrapChoiceOptionDto {
  code: string
  label: string
}

export interface TrapRoundCardDto {
  id: number
  round_number: number
  pair_id: number
  mode: string
  lane: string
  prompt_text: string
  prompt_payload: Record<string, unknown>
  answer_options: TrapChoiceOptionDto[]
  reveal_count: number
  max_reveal_count: number
  status: string
}

export interface TrapSessionSnapshotDto {
  session_id: number
  session_state: string
  score: number
  mode: string
  pair_id: number
  pair_title: string
  left_label: string
  right_label: string
  summary_text: string | null
  recommended_mode: string
  correct_discriminations: number
  total_discriminations: number
  confusion_score: number
  current_round_id: number | null
  round_count: number
  active_round_number: number
  rounds: TrapRoundCardDto[]
}

export interface TrapRoundResultDto {
  round_id: number
  round_number: number
  is_correct: boolean
  score_earned: number
  new_score: number
  streak: number
  selected_choice_code: string | null
  selected_choice_label: string | null
  correct_choice_code: string
  correct_choice_label: string
  explanation_text: string
  review_payload: Record<string, unknown>
  confusion_signal: string
  next_round_id: number | null
  session_complete: boolean
}

export interface TrapReviewRoundDto {
  round_id: number
  round_number: number
  mode: string
  lane: string
  prompt_text: string
  selected_choice_label: string | null
  correct_choice_label: string
  is_correct: boolean
  timed_out: boolean
  response_time_ms: number | null
  confusion_reason_code: string | null
  confusion_reason_text: string | null
  explanation_text: string
  review_payload: Record<string, unknown>
}

export interface TrapReviewDto {
  session_id: number
  pair_id: number
  pair_title: string
  mode: string
  score: number
  accuracy_bp: number
  confusion_score: number
  weakest_lane: string | null
  timed_out_count: number
  recommended_next_mode: string
  dominant_confusion_reason: string | null
  remediation_actions: string[]
  round_count: number
  rounds: TrapReviewRoundDto[]
}

export interface ContrastPairSummaryDto {
  pair_id: number
  pair_code: string | null
  title: string
  left_label: string
  right_label: string
  summary_text: string | null
  trap_strength: number
  difficulty_score: number
  confusion_score: number
  last_accuracy_bp: number
  recommended_mode: string
  available_modes: string[]
}

export interface ContrastComparisonRowDto {
  id: number
  pair_id: number
  lane: string
  compare_label: string
  left_value: string
  right_value: string
  overlap_note: string | null
  decisive_clue: string | null
  teaching_note: string | null
  diagram_asset_id: number | null
  display_order: number
}

export interface ContrastPairProfileDto {
  pair_summary: ContrastPairSummaryDto
  left_profile: Record<string, unknown>
  right_profile: Record<string, unknown>
  shared_traits: string[]
  decisive_differences: string[]
  common_confusions: string[]
  trap_angles: string[]
  coverage: Record<string, unknown>
  generator_contract: Record<string, unknown>
  concept_attributes: Record<string, unknown>[]
  comparison_rows: ContrastComparisonRowDto[]
  diagram_assets: Record<string, unknown>[]
  mode_items: Record<string, unknown>[]
}

export interface TrapMisconceptionReasonDto {
  code: string
  label: string
  category: string
  modes: string[]
  display_order: number
  is_active: boolean
}

export function startGame(input: any): Promise<GameSessionDto> {
  return ipc<GameSessionDto>('start_game', { input })
}

export function submitGameAnswer(input: any): Promise<GameAnswerResultDto> {
  return ipc<GameAnswerResultDto>('submit_game_answer', { input })
}

export function getGameSummary(sessionId: number): Promise<GameSummaryDto> {
  return ipc<GameSummaryDto>('get_game_summary', { sessionId })
}

export function getMindstackState(sessionId: number): Promise<MindstackStateDto> {
  return ipc<MindstackStateDto>('get_mindstack_state', { sessionId })
}

export function getTugOfWarState(sessionId: number): Promise<TugOfWarStateDto> {
  return ipc<TugOfWarStateDto>('get_tug_of_war_state', { sessionId })
}

export function listGameSessions(
  studentId: number,
  limit: number = 24,
): Promise<GameSessionDto[]> {
  return ipc<GameSessionDto[]>('list_game_sessions', { studentId, limit })
}

export function getLeaderboard(
  gameType: string,
  limit: number = 10,
): Promise<LeaderboardEntryDto[]> {
  return ipc<LeaderboardEntryDto[]>('get_leaderboard', { gameType, limit })
}

export function pauseGame(sessionId: number): Promise<void> {
  return ipc('pause_game', { sessionId })
}

export function resumeGame(sessionId: number): Promise<void> {
  return ipc('resume_game', { sessionId })
}

export function abandonGame(sessionId: number): Promise<void> {
  return ipc('abandon_game', { sessionId })
}

// Traps
export function listTrapsPairs(studentId: number, subjectId: number, topicIds: number[]): Promise<ContrastPairSummaryDto[]> {
  return ipc<ContrastPairSummaryDto[]>('list_traps_pairs', { studentId, subjectId, topicIds })
}

export function getContrastPairProfile(studentId: number, pairId: number): Promise<ContrastPairProfileDto> {
  return ipc<ContrastPairProfileDto>('get_contrast_pair_profile', { studentId, pairId })
}

export function listTrapMisconceptionReasons(mode: string | null = null): Promise<TrapMisconceptionReasonDto[]> {
  return ipc<TrapMisconceptionReasonDto[]>('list_trap_misconception_reasons', { mode })
}

export function startTrapsSession(input: any): Promise<TrapSessionSnapshotDto> {
  return ipc<TrapSessionSnapshotDto>('start_traps_session', { input })
}

export function getTrapSessionSnapshot(sessionId: number): Promise<TrapSessionSnapshotDto> {
  return ipc<TrapSessionSnapshotDto>('get_trap_session_snapshot', { sessionId })
}

export function revealTrapUnmaskClue(sessionId: number, roundId: number): Promise<TrapRoundCardDto> {
  return ipc<TrapRoundCardDto>('reveal_trap_unmask_clue', { sessionId, roundId })
}

export function submitTrapRound(input: any): Promise<TrapRoundResultDto> {
  return ipc<TrapRoundResultDto>('submit_trap_round', { input })
}

export function recordTrapConfusionReason(input: any): Promise<void> {
  return ipc('record_trap_confusion_reason', { input })
}

export function getTrapReview(sessionId: number): Promise<TrapReviewDto> {
  return ipc<TrapReviewDto>('get_trap_review', { sessionId })
}
