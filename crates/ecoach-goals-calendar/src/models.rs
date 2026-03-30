use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

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
pub struct AvailabilityProfile {
    pub student_id: i64,
    pub timezone_name: String,
    pub preferred_daily_minutes: i64,
    pub min_session_minutes: i64,
    pub max_session_minutes: i64,
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
