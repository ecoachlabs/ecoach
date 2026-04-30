import { ipc } from '.'

export interface CoachStateDto {
  state: string
  reason: string
}

export interface CoachNextActionDto {
  state: string
  action_type: string
  title: string
  subtitle: string
  estimated_minutes: number | null
  route: string
}

export interface ContentReadinessDto {
  status: string
  subject_codes: string[]
  active_pack_count: number
  topic_count: number
  question_count: number
  reason: string | null
}

export interface TopicCaseDto {
  topic_id: number
  topic_name: string
  subject_code: string
  priority_score: number
  mastery_score: number
  mastery_state: string
  gap_score: number
  fragility_score: number
  memory_strength: number
  decay_risk: number
  evidence_count: number
  requires_probe: boolean
  intervention_mode: string
  intervention_urgency: string
  intervention_reason: string
}

export interface StudentDashboardDto {
  student_name: string
  exam_target: string | null
  overall_readiness_band: string
  subjects: SubjectSummaryDto[]
}

export interface SubjectSummaryDto {
  subject_id: number
  subject_name: string
  readiness_band: string
  mastered_topic_count: number
  weak_topic_count: number
  total_topic_count: number
}

export interface LearnerTruthDto {
  student_id: number
  student_name: string
  overall_mastery_score: number
  overall_readiness_band: string
  pending_review_count: number
  due_memory_count: number
  topic_count: number
  skill_count: number
  memory_count: number
  diagnosis_count: number
}

export interface LearnerMisconceptionSnapshotDto {
  misconception_id: number
  subject_id: number
  topic_id: number | null
  topic_name: string | null
  title: string
  current_status: string
  risk_score: number
  times_detected: number
  cleared_confidence: number
}

export interface MasteryMapNodeDto {
  student_id: number
  subject_id: number
  topic_id: number
  topic_name: string
  mastery_percentage_bp: number
  stability_state: string
  is_blocked: boolean
  blocked_by_topic_ids: number[]
  is_high_yield: boolean
  exam_risk_bp: number
  dependency_count: number
  dependent_count: number
  score_impact_bp: number
  last_activity_at: string | null
  updated_at: string
}

export interface HomeLearningStatsDto {
  streak_days: number
  accuracy_percent: number
  today_minutes: number
  week_questions: number
  total_attempts: number
  correct_attempts: number
}

export interface BeatYesterdayProfileDto {
  student_id: number
  subject_id: number
  current_stage: string
  current_mode: string
  momentum_score: number
  strain_score: number
  readiness_score: number
  recovery_need_score: number
  streak_days: number
}

export interface BeatYesterdayDailyTargetDto {
  id: number
  student_id: number
  subject_id: number
  target_date: string
  stage: string
  mode: string
  target_attempts: number
  target_correct: number
  target_avg_response_time_ms: number | null
  warm_start_minutes: number
  core_climb_minutes: number
  speed_burst_minutes: number
  finish_strong_minutes: number
  focus_topic_ids: number[]
  rationale: unknown
  status: string
}

export interface BeatYesterdayDailySummaryDto {
  id: number
  target_id: number | null
  student_id: number
  subject_id: number
  summary_date: string
  actual_attempts: number
  actual_correct: number
  actual_avg_response_time_ms: number | null
  beat_attempt_target: boolean
  beat_accuracy_target: boolean
  beat_pace_target: boolean
  momentum_score: number
  strain_score: number
  recovery_mode_triggered: boolean
  summary: unknown
}

export interface ClimbTrendPointDto {
  summary_date: string
  actual_attempts: number
  actual_correct: number
  actual_avg_response_time_ms: number | null
  momentum_score: number
  strain_score: number
  recovery_mode_triggered: boolean
}

export interface BeatYesterdayDashboardDto {
  profile: BeatYesterdayProfileDto
  target: BeatYesterdayDailyTargetDto | null
  latest_summary: BeatYesterdayDailySummaryDto | null
  previous_summary: BeatYesterdayDailySummaryDto | null
}

export interface StudentActivityHistoryItemDto {
  id: number
  type_key: 'practice' | 'mock' | 'diagnostic' | 'games' | 'elite' | string
  label: string
  subject: string
  score: number
  answered_questions: number
  total_questions: number
  correct_questions: number
  occurred_at: string
  status: string
}

export interface SubjectDto {
  id: number
  curriculum_version_id: number
  code: string
  name: string
  display_order: number
}

export interface TopicDto {
  id: number
  subject_id: number
  parent_topic_id: number | null
  code: string | null
  name: string
  description: string | null
  node_type: string
  display_order: number
}

export interface GoalRecommendationDto {
  goal_type: string
  urgency_band: string
  urgency_label: string
  days_remaining: number
  default_topic_mode: string
  recommended_actions: string[]
  focus_subjects: string[]
  session_style: string
}

export interface GoalProfileDto {
  id: number
  student_id: number
  parent_goal_id: number | null
  title: string
  description: string | null
  goal_type: string
  goal_category: string
  goal_level: string
  goal_state: string
  subject_id: number | null
  topics: string[]
  urgency_level: string
  start_date: string | null
  deadline: string | null
  exam_id: number | null
  confidence_score_bp: number
  coach_priority_bp: number
  parent_priority_flag: boolean
  evidence_sources: string[]
  dependency_goal_ids: number[]
  risk_level: string
  suggested_weekly_effort_minutes: number | null
  current_momentum_bp: number
  completion_criteria: string[]
  blocked_reason: string | null
  source_bundle_id: number | null
  goal_signal_key: string | null
  metadata: unknown
  created_at: string
  updated_at: string
}

export interface AvailabilityProfileDto {
  student_id: number
  timezone_name: string
  preferred_daily_minutes: number
  ideal_session_minutes: number
  min_session_minutes: number
  max_session_minutes: number
  split_sessions_allowed: boolean
  max_split_sessions: number
  min_break_minutes: number
  trigger_mode: string
  notification_lead_minutes: number
  weekday_capacity_weight_bp: number
  weekend_capacity_weight_bp: number
  schedule_buffer_ratio_bp: number
  fatigue_start_minute: number | null
  fatigue_end_minute: number | null
  thinking_idle_grace_seconds: number
  idle_confirmation_seconds: number
  abandonment_seconds: number
}

export interface AvailabilityWindowDto {
  weekday: number
  start_minute: number
  end_minute: number
  is_preferred: boolean
}

export interface PersonalizationSnapshotDto {
  student_id: number
  subject_id: number
  topic_id: number | null
  scope_key: string
  observed_profile: Record<string, unknown>
  derived_profile: Record<string, unknown>
  inferred_profile: Record<string, unknown>
  strategic_control: Record<string, unknown>
  recommendation: Record<string, unknown>
  confidence_score_bp: number
}

export interface WeeklyPlanBlockDto {
  band_key: string
  session_type: string
  duration_minutes: number
  focus_topic_ids: number[]
  exam_support_scope: string
  rationale: string
}

export interface WeeklyPlanDayDto {
  date: string
  available_minutes: number
  planned_minutes: number
  block_count: number
  blocks: WeeklyPlanBlockDto[]
}

export interface WeeklyPlanBandDto {
  band_key: string
  allocated_minutes: number
  rationale: string
  focus_topic_ids: number[]
}

export interface WeeklyPlanSnapshotDto {
  student_id: number
  subject_id: number | null
  anchor_date: string
  prioritized_exam_id: number | null
  priorities: string[]
  bands: WeeklyPlanBandDto[]
  days: WeeklyPlanDayDto[]
}

export interface AcademicCalendarEventDto {
  id: number
  student_id: number
  legacy_calendar_event_id: number | null
  title: string
  event_type: string
  subject_id: number | null
  subject_name: string | null
  scheduled_date: string
  start_time: string | null
  end_time: string | null
  term: string | null
  academic_year: string | null
  importance_bp: number
  scope: string
  linked_topic_ids: number[]
  preparation_window_days: number
  review_window_days: number
  status: string
  result_after_event: string | null
  coach_priority_weight_bp: number
  expected_weight_bp: number
  timed_performance_weight_bp: number
  coverage_mode: string
  source: string
  days_to_event: number | null
  readiness_score: number | null
  subject_risk_level: string | null
  urgency_level: string | null
  recommended_mode: string | null
  revision_density: string | null
  mission_priority_influence: number | null
  strategy_snapshot: Record<string, unknown>
}

export interface PreparationIntensityProfileDto {
  anchor_date: string
  prioritized_event_id: number | null
  prioritized_event_title: string | null
  phase: string
  days_to_event: number | null
  available_study_days: number
  readiness_score: number
  gap_score: number
  subject_risk_level: string
  urgency_level: string
  recommended_mode: string
  revision_density: string
  mission_priority_influence: number
  explanation_weight_bp: number
  retrieval_weight_bp: number
  timed_drill_weight_bp: number
  breadth_weight_bp: number
  tone: string
  rationale: string[]
}

export interface AcademicCalendarSnapshotDto {
  generated_at: string
  anchor_date: string
  prioritized_event: AcademicCalendarEventDto | null
  intensity: PreparationIntensityProfileDto | null
  strategy_message: string
  events: AcademicCalendarEventDto[]
}

// ── Coach Commands ──

export function getCoachState(studentId: number): Promise<CoachStateDto> {
  return ipc<CoachStateDto>('get_coach_state', { studentId })
}

export function getCoachNextAction(studentId: number): Promise<CoachNextActionDto> {
  return ipc<CoachNextActionDto>('get_coach_next_action', { studentId })
}

export function getContentReadiness(studentId: number): Promise<ContentReadinessDto> {
  return ipc<ContentReadinessDto>('get_content_readiness', { studentId })
}

export function getPriorityTopics(studentId: number, limit: number = 10): Promise<TopicCaseDto[]> {
  return ipc<TopicCaseDto[]>('get_priority_topics', { studentId, limit })
}

export function getStudentDashboard(studentId: number): Promise<StudentDashboardDto> {
  return ipc<StudentDashboardDto>('get_student_dashboard', { studentId })
}

export function getLearnerTruth(studentId: number): Promise<LearnerTruthDto> {
  return ipc<LearnerTruthDto>('get_learner_truth', { studentId })
}

export function listActiveMisconceptions(
  studentId: number,
  subjectId: number,
): Promise<LearnerMisconceptionSnapshotDto[]> {
  return ipc<LearnerMisconceptionSnapshotDto[]>('list_active_misconceptions', {
    studentId,
    subjectId,
  })
}

export function getHomeLearningStats(studentId: number): Promise<HomeLearningStatsDto> {
  return ipc<HomeLearningStatsDto>('get_home_learning_stats', { studentId })
}

export function getBeatYesterdayDashboard(
  studentId: number,
  subjectId: number,
  targetDate: string,
): Promise<BeatYesterdayDashboardDto> {
  return ipc<BeatYesterdayDashboardDto>('get_beat_yesterday_dashboard', {
    studentId,
    subjectId,
    targetDate,
  })
}

export function generateDailyClimbTarget(
  studentId: number,
  subjectId: number,
  date: string,
): Promise<BeatYesterdayDailyTargetDto> {
  return ipc<BeatYesterdayDailyTargetDto>('generate_daily_climb_target', {
    studentId,
    subjectId,
    date,
  })
}

export function completeDailyClimb(
  targetId: number,
): Promise<BeatYesterdayDailySummaryDto> {
  return ipc<BeatYesterdayDailySummaryDto>('complete_daily_climb', {
    targetId,
  })
}

export function listClimbTrend(
  studentId: number,
  subjectId: number,
  days: number = 7,
): Promise<ClimbTrendPointDto[]> {
  return ipc<ClimbTrendPointDto[]>('list_climb_trend', {
    studentId,
    subjectId,
    days,
  })
}

export function listStudentActivityHistory(
  studentId: number,
  limit: number = 32,
): Promise<StudentActivityHistoryItemDto[]> {
  return ipc<StudentActivityHistoryItemDto[]>('list_student_activity_history', {
    studentId,
    limit,
  })
}

export function refreshMasteryMap(
  studentId: number,
  subjectId: number,
): Promise<MasteryMapNodeDto[]> {
  return ipc<MasteryMapNodeDto[]>('refresh_mastery_map', {
    studentId,
    subjectId,
  })
}

export function getMasteryMap(
  studentId: number,
  subjectId: number,
): Promise<MasteryMapNodeDto[]> {
  return ipc<MasteryMapNodeDto[]>('get_mastery_map', {
    studentId,
    subjectId,
  })
}

// ── Curriculum Commands ──

export function listSubjects(curriculumVersionId: number = 1): Promise<SubjectDto[]> {
  return ipc<SubjectDto[]>('list_subjects', { curriculumVersionId })
}

export function listTopics(subjectId: number): Promise<TopicDto[]> {
  return ipc<TopicDto[]>('list_topics', { subjectId })
}

export function buildPersonalizationSnapshot(
  studentId: number,
  subjectId: number,
  topicId: number | null = null,
): Promise<PersonalizationSnapshotDto> {
  return ipc<PersonalizationSnapshotDto>('build_personalization_snapshot', {
    studentId,
    subjectId,
    topicId,
  })
}

export function buildAcademicCalendarSnapshot(
  studentId: number,
  anchorDate: string | null = null,
): Promise<AcademicCalendarSnapshotDto> {
  return ipc<AcademicCalendarSnapshotDto>('build_academic_calendar_snapshot', {
    studentId,
    anchorDate,
  })
}

export function getWeeklyPlanSnapshot(
  studentId: number,
  subjectId: number | null,
  anchorDate: string,
): Promise<WeeklyPlanSnapshotDto> {
  return ipc<WeeklyPlanSnapshotDto>('get_weekly_plan_snapshot', {
    studentId,
    subjectId,
    anchorDate,
  })
}

export function getPreparationIntensityProfile(
  studentId: number,
  anchorDate: string | null = null,
): Promise<PreparationIntensityProfileDto | null> {
  return ipc<PreparationIntensityProfileDto | null>('get_preparation_intensity_profile', {
    studentId,
    anchorDate,
  })
}

export function getAvailabilityProfile(
  studentId: number,
): Promise<AvailabilityProfileDto | null> {
  return ipc<AvailabilityProfileDto | null>('get_availability_profile', { studentId })
}

export function upsertAvailabilityProfile(
  profile: AvailabilityProfileDto,
): Promise<AvailabilityProfileDto> {
  return ipc<AvailabilityProfileDto>('upsert_availability_profile', { profile })
}

export function listAvailabilityWindows(
  studentId: number,
): Promise<AvailabilityWindowDto[]> {
  return ipc<AvailabilityWindowDto[]>('list_availability_windows', { studentId })
}

export function replaceAvailabilityWindows(
  studentId: number,
  windows: AvailabilityWindowDto[],
): Promise<AvailabilityWindowDto[]> {
  return ipc<AvailabilityWindowDto[]>('replace_availability_windows', {
    studentId,
    windows,
  })
}

export function saveLearnerGoal(
  studentId: number,
  subjectId: number,
  goalType: string,
  targetExam: string | null = null,
  examDate: string | null = null,
  confidenceLevel: string | null = null,
): Promise<number> {
  return ipc<number>('save_learner_goal', {
    studentId,
    subjectId,
    goalType,
    targetExam,
    examDate,
    confidenceLevel,
  })
}

export function listGoalProfiles(studentId: number): Promise<GoalProfileDto[]> {
  return ipc<GoalProfileDto[]>('list_goal_profiles', { studentId })
}

export function getGoalRecommendation(
  studentId: number,
  subjectId: number,
): Promise<GoalRecommendationDto> {
  return ipc<GoalRecommendationDto>('get_goal_recommendation', {
    studentId,
    subjectId,
  })
}

export interface RevengeQueueItemDto {
  id: number
  student_id: number
  question_id: number
  original_session_id: number | null
  original_error_type: string | null
  original_wrong_answer: string | null
  attempts_to_beat: number
  is_beaten: boolean
  beaten_at: string | null
  added_at: string
  question_text: string | null
  topic_id: number | null
}

export function getRevengeQueue(studentId: number): Promise<RevengeQueueItemDto[]> {
  return ipc<RevengeQueueItemDto[]>('get_revenge_queue', { studentId })
}
