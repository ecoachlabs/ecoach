use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: i64,
    pub student_id: i64,
    pub goal_type: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: i64,
    pub student_id: i64,
    pub event_type: String,
    pub title: String,
    pub scheduled_for: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicCalendarEventInput {
    pub title: String,
    pub event_type: String,
    pub subject_id: Option<i64>,
    pub scheduled_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub term: Option<String>,
    pub academic_year: Option<String>,
    pub importance_bp: BasisPoints,
    pub scope: String,
    pub linked_topic_ids: Vec<i64>,
    pub preparation_window_days: i64,
    pub review_window_days: i64,
    pub status: String,
    pub result_after_event: Option<String>,
    pub coach_priority_weight_bp: BasisPoints,
    pub expected_weight_bp: BasisPoints,
    pub timed_performance_weight_bp: BasisPoints,
    pub coverage_mode: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicCalendarEvent {
    pub id: i64,
    pub student_id: i64,
    pub legacy_calendar_event_id: Option<i64>,
    pub title: String,
    pub event_type: String,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub scheduled_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub term: Option<String>,
    pub academic_year: Option<String>,
    pub importance_bp: BasisPoints,
    pub scope: String,
    pub linked_topic_ids: Vec<i64>,
    pub preparation_window_days: i64,
    pub review_window_days: i64,
    pub status: String,
    pub result_after_event: Option<String>,
    pub coach_priority_weight_bp: BasisPoints,
    pub expected_weight_bp: BasisPoints,
    pub timed_performance_weight_bp: BasisPoints,
    pub coverage_mode: String,
    pub source: String,
    pub days_to_event: Option<i64>,
    pub readiness_score: Option<BasisPoints>,
    pub subject_risk_level: Option<String>,
    pub urgency_level: Option<String>,
    pub recommended_mode: Option<String>,
    pub revision_density: Option<String>,
    pub mission_priority_influence: Option<BasisPoints>,
    pub strategy_snapshot: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationIntensityProfile {
    pub anchor_date: String,
    pub prioritized_event_id: Option<i64>,
    pub prioritized_event_title: Option<String>,
    pub phase: String,
    pub days_to_event: Option<i64>,
    pub available_study_days: i64,
    pub readiness_score: BasisPoints,
    pub gap_score: BasisPoints,
    pub subject_risk_level: String,
    pub urgency_level: String,
    pub recommended_mode: String,
    pub revision_density: String,
    pub mission_priority_influence: BasisPoints,
    pub explanation_weight_bp: BasisPoints,
    pub retrieval_weight_bp: BasisPoints,
    pub timed_drill_weight_bp: BasisPoints,
    pub breadth_weight_bp: BasisPoints,
    pub tone: String,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicCalendarSnapshot {
    pub generated_at: String,
    pub anchor_date: String,
    pub prioritized_event: Option<AcademicCalendarEvent>,
    pub intensity: Option<PreparationIntensityProfile>,
    pub strategy_message: String,
    pub events: Vec<AcademicCalendarEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityProfile {
    pub student_id: i64,
    pub timezone_name: String,
    pub preferred_daily_minutes: i64,
    pub ideal_session_minutes: i64,
    pub min_session_minutes: i64,
    pub max_session_minutes: i64,
    pub split_sessions_allowed: bool,
    pub max_split_sessions: i64,
    pub min_break_minutes: i64,
    pub trigger_mode: String,
    pub notification_lead_minutes: i64,
    pub weekday_capacity_weight_bp: BasisPoints,
    pub weekend_capacity_weight_bp: BasisPoints,
    pub schedule_buffer_ratio_bp: BasisPoints,
    pub fatigue_start_minute: Option<i64>,
    pub fatigue_end_minute: Option<i64>,
    pub thinking_idle_grace_seconds: i64,
    pub idle_confirmation_seconds: i64,
    pub abandonment_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityWindow {
    pub weekday: i64,
    pub start_minute: i64,
    pub end_minute: i64,
    pub is_preferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityException {
    pub exception_date: String,
    pub start_minute: Option<i64>,
    pub end_minute: Option<i64>,
    pub availability_mode: String,
    pub minutes_delta: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAvailabilitySummary {
    pub date: String,
    pub base_minutes: i64,
    pub adjusted_minutes: i64,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeNowRecommendation {
    pub date: String,
    pub minute_of_day: i64,
    pub available_now: bool,
    pub window_end_minute: Option<i64>,
    pub suggested_duration_minutes: i64,
    pub session_type: String,
    pub rationale: String,
    pub focus_topic_ids: Vec<i64>,
    pub target_id: Option<i64>,
    pub carryover_attempts: i64,
    pub carryover_correct: i64,
    pub pressure_score: BasisPoints,
    pub repair_buffer_minutes: i64,
    pub recommended_comeback_topic_id: Option<i64>,
    pub recent_repair_outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReplan {
    pub date: String,
    pub available_now: bool,
    pub remaining_capacity_minutes: i64,
    pub remaining_target_minutes: i64,
    pub recommended_session_count: i64,
    pub next_session_type: String,
    pub focus_topic_ids: Vec<i64>,
    pub target_id: Option<i64>,
    pub rationale: String,
    pub pressure_score: BasisPoints,
    pub repair_buffer_minutes: i64,
    pub recommended_comeback_topic_id: Option<i64>,
    pub recent_repair_outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamPlanStateInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub anchor_date: String,
    pub exam_date: String,
    pub target_effective_minutes: i64,
    pub plan_mode: Option<String>,
    pub auto_trigger_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamPlanState {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub anchor_date: String,
    pub exam_date: String,
    pub target_effective_minutes: i64,
    pub completed_effective_minutes: i64,
    pub remaining_effective_minutes: i64,
    pub available_study_days: i64,
    pub required_weekly_minutes: i64,
    pub protected_buffer_minutes: i64,
    pub buffer_consumed_minutes: i64,
    pub missed_debt_minutes: i64,
    pub bonus_credit_minutes: i64,
    pub pressure_score_bp: BasisPoints,
    pub feasibility_score_bp: BasisPoints,
    pub schedule_truth_score_bp: BasisPoints,
    pub plan_mode: String,
    pub auto_trigger_mode: String,
    pub explanation: Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleLedgerEntry {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub ledger_date: String,
    pub scheduled_minutes: i64,
    pub completed_minutes: i64,
    pub effective_credit_minutes: i64,
    pub buffer_minutes_reserved: i64,
    pub buffer_minutes_consumed: i64,
    pub missed_minutes_debt: i64,
    pub bonus_minutes_credit: i64,
    pub pressure_score_bp: BasisPoints,
    pub feasibility_score_bp: BasisPoints,
    pub explanation: Value,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSessionBlock {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub plan_state_id: Option<i64>,
    pub block_date: String,
    pub start_minute: Option<i64>,
    pub end_minute: Option<i64>,
    pub target_minutes: i64,
    pub session_type: String,
    pub objective_summary: String,
    pub focus_topic_ids: Vec<i64>,
    pub trigger_mode: String,
    pub fit_score_bp: BasisPoints,
    pub priority_score_bp: BasisPoints,
    pub status: String,
    pub fallback_session_type: Option<String>,
    pub replacement_options: Vec<String>,
    pub created_by: String,
    pub source_kind: String,
    pub explanation_text: String,
    pub linked_session_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTriggerJob {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub session_block_id: Option<i64>,
    pub trigger_kind: String,
    pub scheduled_for: String,
    pub lead_minutes: i64,
    pub status: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOrchestrationSnapshot {
    pub student_id: i64,
    pub subject_id: i64,
    pub anchor_date: String,
    pub minute_of_day: i64,
    pub availability_profile: AvailabilityProfile,
    pub daily_availability: DailyAvailabilitySummary,
    pub plan_state: ExamPlanState,
    pub ledger: ScheduleLedgerEntry,
    pub free_now: FreeNowRecommendation,
    pub replan: DailyReplan,
    pub session_blocks: Vec<TimeSessionBlock>,
    pub trigger_jobs: Vec<ScheduleTriggerJob>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatYesterdayProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub current_stage: String,
    pub current_mode: String,
    pub momentum_score: BasisPoints,
    pub strain_score: BasisPoints,
    pub readiness_score: BasisPoints,
    pub recovery_need_score: BasisPoints,
    pub streak_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatYesterdayDailyTarget {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub target_date: String,
    pub stage: String,
    pub mode: String,
    pub target_attempts: i64,
    pub target_correct: i64,
    pub target_avg_response_time_ms: Option<i64>,
    pub warm_start_minutes: i64,
    pub core_climb_minutes: i64,
    pub speed_burst_minutes: i64,
    pub finish_strong_minutes: i64,
    pub focus_topic_ids: Vec<i64>,
    pub rationale: serde_json::Value,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatYesterdayDailySummary {
    pub id: i64,
    pub target_id: Option<i64>,
    pub student_id: i64,
    pub subject_id: i64,
    pub summary_date: String,
    pub actual_attempts: i64,
    pub actual_correct: i64,
    pub actual_avg_response_time_ms: Option<i64>,
    pub beat_attempt_target: bool,
    pub beat_accuracy_target: bool,
    pub beat_pace_target: bool,
    pub momentum_score: BasisPoints,
    pub strain_score: BasisPoints,
    pub recovery_mode_triggered: bool,
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimbTrendPoint {
    pub summary_date: String,
    pub actual_attempts: i64,
    pub actual_correct: i64,
    pub actual_avg_response_time_ms: Option<i64>,
    pub momentum_score: BasisPoints,
    pub strain_score: BasisPoints,
    pub recovery_mode_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatYesterdayDashboard {
    pub profile: BeatYesterdayProfile,
    pub target: Option<BeatYesterdayDailyTarget>,
    pub latest_summary: Option<BeatYesterdayDailySummary>,
    pub previous_summary: Option<BeatYesterdayDailySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentMomentum {
    pub id: i64,
    pub student_id: i64,
    pub momentum_state: String,
    pub current_streak_days: i64,
    pub best_streak_days: i64,
    pub consistency_7d_bp: BasisPoints,
    pub consistency_14d_bp: BasisPoints,
    pub consistency_30d_bp: BasisPoints,
    pub dropout_risk_bp: BasisPoints,
    pub last_session_date: Option<String>,
    pub days_since_last_session: i64,
    pub comeback_session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComebackFlowTemplate {
    pub id: i64,
    pub template_code: String,
    pub display_name: String,
    pub trigger_condition: String,
    pub steps: Vec<String>,
    pub estimated_minutes: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComebackFlow {
    pub id: i64,
    pub student_id: i64,
    pub trigger_reason: String,
    pub days_inactive: i64,
    pub template_code: Option<String>,
    pub template_name: Option<String>,
    pub flow_steps: Vec<String>,
    pub current_step: i64,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevengeQueueItem {
    pub id: i64,
    pub student_id: i64,
    pub question_id: i64,
    pub original_session_id: Option<i64>,
    pub original_error_type: Option<String>,
    pub original_wrong_answer: Option<String>,
    pub attempts_to_beat: i64,
    pub is_beaten: bool,
    pub beaten_at: Option<String>,
    pub added_at: String,
    pub question_text: Option<String>,
    pub topic_id: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderScheduleInput {
    pub mission_id: Option<i64>,
    pub academic_event_id: Option<i64>,
    pub reminder_type: String,
    pub scheduled_time: String,
    pub audience: String,
    pub escalation_level: i64,
    pub message: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSchedule {
    pub id: i64,
    pub learner_id: i64,
    pub mission_id: Option<i64>,
    pub academic_event_id: Option<i64>,
    pub reminder_type: String,
    pub scheduled_time: String,
    pub audience: String,
    pub status: String,
    pub escalation_level: i64,
    pub message: String,
    pub metadata: Value,
    pub sent_at: Option<String>,
    pub acknowledged_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEventInput {
    pub mission_id: Option<i64>,
    pub session_id: Option<i64>,
    pub reminder_schedule_id: Option<i64>,
    pub academic_event_id: Option<i64>,
    pub session_state: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub completion_percent: i64,
    pub missed_reason: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEvent {
    pub id: i64,
    pub learner_id: i64,
    pub mission_id: Option<i64>,
    pub session_id: Option<i64>,
    pub reminder_schedule_id: Option<i64>,
    pub academic_event_id: Option<i64>,
    pub session_state: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub completion_percent: i64,
    pub missed_reason: Option<String>,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementRiskProfile {
    pub learner_id: i64,
    pub risk_level: String,
    pub risk_score_bp: BasisPoints,
    pub consecutive_misses: i64,
    pub recent_partial_sessions: i64,
    pub last_session_state: Option<String>,
    pub next_recovery_action: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentAccessSettingsInput {
    pub visibility_mode: String,
    pub reminders_enabled: bool,
    pub alerts_enabled: bool,
    pub feedback_enabled: bool,
    pub can_excuse_sessions: bool,
    pub preferred_channel: String,
    pub quiet_hours: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentAccessSettings {
    pub id: i64,
    pub parent_id: i64,
    pub learner_id: i64,
    pub visibility_mode: String,
    pub reminders_enabled: bool,
    pub alerts_enabled: bool,
    pub feedback_enabled: bool,
    pub can_excuse_sessions: bool,
    pub preferred_channel: String,
    pub quiet_hours: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentFeedbackInput {
    pub category: String,
    pub message: String,
    pub urgency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentFeedbackRecord {
    pub id: i64,
    pub learner_id: i64,
    pub parent_id: i64,
    pub category: String,
    pub message: String,
    pub interpreted_signal: String,
    pub urgency: String,
    pub suggested_support_action: Option<String>,
    pub visible_strategy_change: Option<String>,
    pub status: String,
    pub submitted_at: String,
    pub applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentAlertRecord {
    pub id: i64,
    pub learner_id: i64,
    pub parent_id: i64,
    pub trigger_type: String,
    pub severity: String,
    pub message: String,
    pub action_required: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub created_at: String,
    pub acknowledged_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAdjustmentLog {
    pub id: i64,
    pub learner_id: i64,
    pub reason: String,
    pub source: String,
    pub old_strategy_snapshot: Value,
    pub new_strategy_snapshot: Value,
    pub visible_message_student: Option<String>,
    pub visible_message_parent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachBadgeAward {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub badge_name: String,
    pub badge_family: String,
    pub reason: String,
    pub related_session_id: Option<i64>,
    pub related_title_state_id: Option<i64>,
    pub metadata: Value,
    pub awarded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachTitleCard {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub title_name: String,
    pub state: String,
    pub earned_at: Option<String>,
    pub last_defended_at: Option<String>,
    pub next_defense_due_at: Option<String>,
    pub coach_note: Option<String>,
    pub evidence_snapshot: Value,
    pub reclaim_plan: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachTitleHistoryEntry {
    pub id: i64,
    pub title_state_id: i64,
    pub previous_state: Option<String>,
    pub new_state: String,
    pub reason: String,
    pub snapshot: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitlesHallSnapshot {
    pub student_id: i64,
    pub generated_at: String,
    pub active_titles: Vec<CoachTitleCard>,
    pub defenses_due: Vec<CoachTitleCard>,
    pub contested_titles: Vec<CoachTitleCard>,
    pub reclaimed_titles: Vec<CoachTitleCard>,
    pub badges: Vec<CoachBadgeAward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleDefenseBrief {
    pub run_id: i64,
    pub title_id: i64,
    pub title_name: String,
    pub subject_name: Option<String>,
    pub topic_name: Option<String>,
    pub reason_due: String,
    pub components: Vec<String>,
    pub timed: bool,
    pub coach_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleDefenseCompletionInput {
    pub recall_passed: bool,
    pub application_passed: bool,
    pub transfer_passed: bool,
    pub timed_challenge_passed: Option<bool>,
    pub confidence_held: Option<bool>,
    pub accuracy_score_bp: Option<BasisPoints>,
    pub notes: Option<String>,
    pub triggered_misconception: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleDefenseResult {
    pub run_id: i64,
    pub title_id: i64,
    pub title_name: String,
    pub outcome: String,
    pub new_state: String,
    pub coach_note: String,
    pub next_defense_due_at: Option<String>,
    pub reclaim_plan: Option<Value>,
    pub badge_awarded: Option<CoachBadgeAward>,
    pub strategy_adjustment_id: Option<i64>,
}
