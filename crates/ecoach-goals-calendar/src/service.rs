use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, Duration, NaiveDate, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    AcademicCalendarEvent, AcademicCalendarEventInput, AcademicCalendarSnapshot,
    AvailabilityException, AvailabilityProfile, AvailabilityWindow, BeatYesterdayDailySummary,
    BeatYesterdayDailyTarget, BeatYesterdayDashboard, BeatYesterdayProfile, CalendarEvent,
    ClimbTrendPoint, CoachBadgeAward, CoachTitleCard, CoachTitleHistoryEntry, ComebackFlow,
    ComebackFlowTemplate, DailyAvailabilitySummary, DailyReplan, EngagementEvent,
    EngagementEventInput, EngagementRiskProfile, ExamPlanState, ExamPlanStateInput,
    FreeNowRecommendation, Goal, GoalArbitrationSnapshot, GoalConflict, GoalProfile,
    GoalProfileInput, ParentAccessSettings, ParentAccessSettingsInput, ParentAlertRecord,
    ParentFeedbackInput, ParentFeedbackRecord, PreparationIntensityProfile, ReminderSchedule,
    ReminderScheduleInput, RevengeQueueItem, ScheduleLedgerEntry, ScheduleTriggerJob,
    StrategyAdjustmentLog, StudentMomentum, TimeOrchestrationSnapshot, TimeSessionBlock,
    TitleDefenseBrief, TitleDefenseCompletionInput, TitleDefenseResult, TitlesHallSnapshot,
    WeeklyPlanBand, WeeklyPlanBlock, WeeklyPlanDay, WeeklyPlanSnapshot,
};

pub struct GoalsCalendarService<'a> {
    conn: &'a Connection,
}

struct SessionPerformanceBaseline {
    attempts: i64,
    correct: i64,
    accuracy_score: BasisPoints,
    avg_response_time_ms: Option<i64>,
    strain_score: BasisPoints,
}

struct PendingMissionWindow {
    title: String,
    activity_type: String,
    primary_topic_id: Option<i64>,
}

#[derive(Clone)]
struct RecentSolidificationOutcome {
    topic_id: i64,
    topic_name: String,
    outcome: String,
    next_action_hint: String,
    accuracy_score: Option<BasisPoints>,
}

#[derive(Clone)]
struct TopicPressureCandidate {
    topic_id: i64,
    topic_name: String,
    priority_score: BasisPoints,
    gap_score: BasisPoints,
    repair_priority: BasisPoints,
    fragility_score: BasisPoints,
    is_urgent: bool,
    due_items: i64,
    fragile_items: i64,
    collapsed_items: i64,
}

struct RankedPressureTopic {
    candidate: TopicPressureCandidate,
    outcome: Option<RecentSolidificationOutcome>,
    adjusted_score: BasisPoints,
}

struct ComebackPressure {
    focus_topic_ids: Vec<i64>,
    recommended_topic_id: Option<i64>,
    pressure_score: BasisPoints,
    repair_buffer_minutes: i64,
    recommended_session_type: Option<String>,
    recent_repair_outcome: Option<String>,
    rationale: Option<String>,
}

#[derive(Clone)]
struct EventReadinessSnapshot {
    phase: String,
    available_study_days: i64,
    readiness_score: BasisPoints,
    gap_score: BasisPoints,
    subject_risk_level: String,
    urgency_level: String,
    recommended_mode: String,
    revision_density: String,
    mission_priority_influence: BasisPoints,
    explanation_weight_bp: BasisPoints,
    retrieval_weight_bp: BasisPoints,
    timed_drill_weight_bp: BasisPoints,
    breadth_weight_bp: BasisPoints,
    tone: String,
    rationale: Vec<String>,
}

struct TitleEligibilitySnapshot {
    readiness_score: BasisPoints,
    independent_successes: i64,
    timed_successes: i64,
    unresolved_misconceptions: i64,
    candidate: bool,
    awardable: bool,
}

fn map_daily_target(row: &rusqlite::Row<'_>) -> rusqlite::Result<BeatYesterdayDailyTarget> {
    let focus_topic_ids_json: String = row.get(13)?;
    let rationale_json: String = row.get(14)?;
    Ok(BeatYesterdayDailyTarget {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        target_date: row.get(3)?,
        stage: row.get(4)?,
        mode: row.get(5)?,
        target_attempts: row.get(6)?,
        target_correct: row.get(7)?,
        target_avg_response_time_ms: row.get(8)?,
        warm_start_minutes: row.get(9)?,
        core_climb_minutes: row.get(10)?,
        speed_burst_minutes: row.get(11)?,
        finish_strong_minutes: row.get(12)?,
        focus_topic_ids: parse_i64_list(&focus_topic_ids_json).map_err(to_sql_conversion_error)?,
        rationale: parse_json_value(&rationale_json).map_err(to_sql_conversion_error)?,
        status: row.get(15)?,
    })
}

fn map_exam_plan_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExamPlanState> {
    let explanation_json: String = row.get(19)?;
    Ok(ExamPlanState {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        anchor_date: row.get(3)?,
        exam_date: row.get(4)?,
        target_effective_minutes: row.get(5)?,
        completed_effective_minutes: row.get(6)?,
        remaining_effective_minutes: row.get(7)?,
        available_study_days: row.get(8)?,
        required_weekly_minutes: row.get(9)?,
        protected_buffer_minutes: row.get(10)?,
        buffer_consumed_minutes: row.get(11)?,
        missed_debt_minutes: row.get(12)?,
        bonus_credit_minutes: row.get(13)?,
        pressure_score_bp: row.get(14)?,
        feasibility_score_bp: row.get(15)?,
        schedule_truth_score_bp: row.get(16)?,
        plan_mode: row.get(17)?,
        auto_trigger_mode: row.get(18)?,
        explanation: parse_json_value(&explanation_json).map_err(to_sql_conversion_error)?,
        updated_at: row.get(20)?,
    })
}

fn map_schedule_ledger_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduleLedgerEntry> {
    let explanation_json: String = row.get(12)?;
    Ok(ScheduleLedgerEntry {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        ledger_date: row.get(3)?,
        scheduled_minutes: row.get(4)?,
        completed_minutes: row.get(5)?,
        effective_credit_minutes: row.get(6)?,
        buffer_minutes_reserved: row.get(7)?,
        buffer_minutes_consumed: row.get(8)?,
        missed_minutes_debt: row.get(9)?,
        bonus_minutes_credit: row.get(10)?,
        pressure_score_bp: row.get(11)?,
        feasibility_score_bp: row.get(13)?,
        explanation: parse_json_value(&explanation_json).map_err(to_sql_conversion_error)?,
        updated_at: row.get(14)?,
    })
}

fn map_time_session_block(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimeSessionBlock> {
    let focus_topic_ids_json: String = row.get(10)?;
    let replacement_options_json: String = row.get(16)?;
    Ok(TimeSessionBlock {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        plan_state_id: row.get(3)?,
        block_date: row.get(4)?,
        start_minute: row.get(5)?,
        end_minute: row.get(6)?,
        target_minutes: row.get(7)?,
        session_type: row.get(8)?,
        objective_summary: row.get(9)?,
        focus_topic_ids: parse_i64_list(&focus_topic_ids_json).map_err(to_sql_conversion_error)?,
        trigger_mode: row.get(11)?,
        fit_score_bp: row.get(12)?,
        priority_score_bp: row.get(13)?,
        status: row.get(14)?,
        fallback_session_type: row.get(15)?,
        replacement_options: parse_string_list(&replacement_options_json)
            .map_err(to_sql_conversion_error)?,
        created_by: row.get(17)?,
        source_kind: row.get(18)?,
        explanation_text: row.get(19)?,
        linked_session_id: row.get(20)?,
    })
}

fn map_schedule_trigger_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduleTriggerJob> {
    let payload_json: String = row.get(8)?;
    Ok(ScheduleTriggerJob {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        session_block_id: row.get(3)?,
        trigger_kind: row.get(4)?,
        scheduled_for: row.get(5)?,
        lead_minutes: row.get(6)?,
        status: row.get(7)?,
        payload: parse_json_value(&payload_json).map_err(to_sql_conversion_error)?,
    })
}

fn map_goal_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<GoalProfile> {
    let topics_json: String = row.get(10)?;
    let evidence_sources_json: String = row.get(17)?;
    let dependency_goals_json: String = row.get(18)?;
    let completion_criteria_json: String = row.get(22)?;
    let metadata_json: String = row.get(27)?;
    Ok(GoalProfile {
        id: row.get(0)?,
        student_id: row.get(1)?,
        parent_goal_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        goal_type: row.get(5)?,
        goal_category: row.get(6)?,
        goal_level: row.get(7)?,
        goal_state: row.get(8)?,
        subject_id: row.get(9)?,
        topics: parse_string_list(&topics_json).map_err(to_sql_conversion_error)?,
        urgency_level: row.get(11)?,
        start_date: row.get(12)?,
        deadline: row.get(13)?,
        exam_id: row.get(14)?,
        confidence_score_bp: row.get(15)?,
        coach_priority_bp: row.get(16)?,
        parent_priority_flag: row.get::<_, i64>(19)? == 1,
        evidence_sources: parse_string_list(&evidence_sources_json)
            .map_err(to_sql_conversion_error)?,
        dependency_goal_ids: parse_i64_list(&dependency_goals_json)
            .map_err(to_sql_conversion_error)?,
        risk_level: row
            .get::<_, Option<String>>(20)?
            .unwrap_or_else(|| "normal".to_string()),
        suggested_weekly_effort_minutes: row.get(21)?,
        current_momentum_bp: row.get(23)?,
        completion_criteria: parse_string_list(&completion_criteria_json)
            .map_err(to_sql_conversion_error)?,
        blocked_reason: row.get(24)?,
        source_bundle_id: row.get(25)?,
        goal_signal_key: row.get(26)?,
        metadata: parse_json_value(&metadata_json).map_err(to_sql_conversion_error)?,
        created_at: row.get(28)?,
        updated_at: row.get(29)?,
    })
}

fn map_daily_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<BeatYesterdayDailySummary> {
    let summary_json: String = row.get(14)?;
    Ok(BeatYesterdayDailySummary {
        id: row.get(0)?,
        target_id: row.get(1)?,
        student_id: row.get(2)?,
        subject_id: row.get(3)?,
        summary_date: row.get(4)?,
        actual_attempts: row.get(5)?,
        actual_correct: row.get(6)?,
        actual_avg_response_time_ms: row.get(7)?,
        beat_attempt_target: row.get::<_, i64>(8)? == 1,
        beat_accuracy_target: row.get::<_, i64>(9)? == 1,
        beat_pace_target: row.get::<_, i64>(10)? == 1,
        momentum_score: row.get(11)?,
        strain_score: row.get(12)?,
        recovery_mode_triggered: row.get::<_, i64>(13)? == 1,
        summary: parse_json_value(&summary_json).map_err(to_sql_conversion_error)?,
    })
}

fn parse_i64_list(raw: &str) -> EcoachResult<Vec<i64>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_value(raw: &str) -> EcoachResult<Value> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_string_list(raw: &str) -> EcoachResult<Vec<String>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_or_default(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| json!({}))
}

fn to_sql_conversion_error(err: EcoachError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
}

fn default_availability_profile(student_id: i64) -> AvailabilityProfile {
    AvailabilityProfile {
        student_id,
        timezone_name: "Africa/Accra".to_string(),
        preferred_daily_minutes: 60,
        ideal_session_minutes: 60,
        min_session_minutes: 15,
        max_session_minutes: 90,
        split_sessions_allowed: true,
        max_split_sessions: 2,
        min_break_minutes: 20,
        trigger_mode: "hybrid".to_string(),
        notification_lead_minutes: 10,
        weekday_capacity_weight_bp: 10_000,
        weekend_capacity_weight_bp: 11_500,
        schedule_buffer_ratio_bp: 1_500,
        fatigue_start_minute: None,
        fatigue_end_minute: None,
        thinking_idle_grace_seconds: 180,
        idle_confirmation_seconds: 120,
        abandonment_seconds: 900,
    }
}

fn validate_time_plan_mode(value: Option<&str>) -> EcoachResult<()> {
    if value.is_none() {
        return Ok(());
    }
    match value.unwrap_or_default() {
        "mastery" | "exam_performance" | "rescue" => Ok(()),
        other => Err(EcoachError::Validation(format!(
            "unsupported plan_mode '{}'",
            other
        ))),
    }
}

fn validate_time_trigger_mode(value: Option<&str>) -> EcoachResult<()> {
    if value.is_none() {
        return Ok(());
    }
    match value.unwrap_or_default() {
        "manual" | "auto" | "hybrid" => Ok(()),
        other => Err(EcoachError::Validation(format!(
            "unsupported trigger_mode '{}'",
            other
        ))),
    }
}

fn days_between_inclusive(start: &str, end: &str) -> EcoachResult<i64> {
    let start = NaiveDate::parse_from_str(start, "%Y-%m-%d")
        .map_err(|err| EcoachError::Validation(err.to_string()))?;
    let end = NaiveDate::parse_from_str(end, "%Y-%m-%d")
        .map_err(|err| EcoachError::Validation(err.to_string()))?;
    if end < start {
        return Ok(0);
    }
    Ok((end - start).num_days() + 1)
}

fn derive_time_plan_mode(
    days_to_exam: i64,
    pressure_score_bp: BasisPoints,
    feasibility_score_bp: BasisPoints,
) -> String {
    if pressure_score_bp >= 8_000 || feasibility_score_bp <= 4_500 {
        "rescue".to_string()
    } else if days_to_exam <= 21 {
        "exam_performance".to_string()
    } else {
        "mastery".to_string()
    }
}

fn derive_block_trigger_mode(
    profile_trigger_mode: &str,
    pressure_score_bp: BasisPoints,
    weekday: i64,
    session_type: &str,
) -> String {
    match profile_trigger_mode {
        "auto" | "manual" => profile_trigger_mode.to_string(),
        _ => {
            if pressure_score_bp >= 7_500
                || matches!(session_type, "timed_exam_rehearsal" | "repair_recovery")
                || weekday >= 5
            {
                "auto".to_string()
            } else {
                "manual".to_string()
            }
        }
    }
}

fn combine_date_minute(date: &str, minute_of_day: i64) -> String {
    let normalized = minute_of_day.clamp(0, 1_439);
    format!("{}T{:02}:{:02}:00", date, normalized / 60, normalized % 60)
}

fn map_student_momentum(row: &rusqlite::Row<'_>) -> rusqlite::Result<StudentMomentum> {
    Ok(StudentMomentum {
        id: row.get(0)?,
        student_id: row.get(1)?,
        momentum_state: row.get(2)?,
        current_streak_days: row.get(3)?,
        best_streak_days: row.get(4)?,
        consistency_7d_bp: row.get(5)?,
        consistency_14d_bp: row.get(6)?,
        consistency_30d_bp: row.get(7)?,
        dropout_risk_bp: row.get(8)?,
        last_session_date: row.get(9)?,
        days_since_last_session: row.get(10)?,
        comeback_session_count: row.get(11)?,
    })
}

fn map_comeback_flow_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<ComebackFlowTemplate> {
    let steps_json: String = row.get(4)?;
    Ok(ComebackFlowTemplate {
        id: row.get(0)?,
        template_code: row.get(1)?,
        display_name: row.get(2)?,
        trigger_condition: row.get(3)?,
        steps: parse_string_list(&steps_json).map_err(to_sql_conversion_error)?,
        estimated_minutes: row.get(5)?,
        description: row.get(6)?,
    })
}

fn map_comeback_flow(row: &rusqlite::Row<'_>) -> rusqlite::Result<ComebackFlow> {
    let flow_payload: String = row.get(5)?;
    let payload: Value = serde_json::from_str(&flow_payload).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })?;
    Ok(ComebackFlow {
        id: row.get(0)?,
        student_id: row.get(1)?,
        trigger_reason: row.get(2)?,
        days_inactive: row.get(3)?,
        template_code: payload
            .get("template_code")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        template_name: payload
            .get("template_name")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        flow_steps: payload
            .get("steps")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|value| value.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        current_step: row.get(4)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        completed_at: row.get(8)?,
    })
}

fn map_revenge_queue_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<RevengeQueueItem> {
    let metadata = row
        .get::<_, Option<String>>(12)?
        .map(|raw| serde_json::from_str::<Value>(&raw))
        .transpose()
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(EcoachError::Serialization(err.to_string())),
            )
        })?;
    Ok(RevengeQueueItem {
        id: row.get(0)?,
        student_id: row.get(1)?,
        question_id: row.get(2)?,
        original_session_id: row.get(3)?,
        original_error_type: row.get(4)?,
        original_wrong_answer: row.get(5)?,
        attempts_to_beat: row.get(6)?,
        is_beaten: row.get::<_, i64>(7)? == 1,
        beaten_at: row.get(8)?,
        added_at: row.get(9)?,
        question_text: row.get(10)?,
        topic_id: row.get(11)?,
        metadata,
    })
}

fn map_academic_calendar_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<AcademicCalendarEvent> {
    let linked_topic_ids_json: String = row.get(13)?;
    let strategy_snapshot_json: String = row.get(23)?;
    Ok(AcademicCalendarEvent {
        id: row.get(0)?,
        student_id: row.get(1)?,
        legacy_calendar_event_id: row.get(2)?,
        title: row.get(3)?,
        event_type: row.get(4)?,
        subject_id: row.get(5)?,
        subject_name: row.get(24)?,
        scheduled_date: row.get(6)?,
        start_time: row.get(7)?,
        end_time: row.get(8)?,
        term: row.get(9)?,
        academic_year: row.get(10)?,
        importance_bp: row.get(11)?,
        scope: row.get(12)?,
        linked_topic_ids: parse_i64_list(&linked_topic_ids_json)
            .map_err(to_sql_conversion_error)?,
        preparation_window_days: row.get(14)?,
        review_window_days: row.get(15)?,
        status: row.get(16)?,
        result_after_event: row.get(17)?,
        coach_priority_weight_bp: row.get(18)?,
        expected_weight_bp: row.get(19)?,
        timed_performance_weight_bp: row.get(20)?,
        coverage_mode: row.get(21)?,
        source: row.get(22)?,
        days_to_event: None,
        readiness_score: None,
        subject_risk_level: None,
        urgency_level: None,
        recommended_mode: None,
        revision_density: None,
        mission_priority_influence: None,
        strategy_snapshot: parse_json_or_default(&strategy_snapshot_json),
    })
}

fn map_reminder_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReminderSchedule> {
    let metadata_json: String = row.get(9)?;
    Ok(ReminderSchedule {
        id: row.get(0)?,
        learner_id: row.get(1)?,
        mission_id: row.get(2)?,
        academic_event_id: row.get(3)?,
        reminder_type: row.get(4)?,
        scheduled_time: row.get(5)?,
        audience: row.get(6)?,
        status: row.get(7)?,
        escalation_level: row.get(8)?,
        message: row.get(10)?,
        metadata: parse_json_or_default(&metadata_json),
        sent_at: row.get(11)?,
        acknowledged_at: row.get(12)?,
        created_at: row.get(13)?,
    })
}

fn map_engagement_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<EngagementEvent> {
    Ok(EngagementEvent {
        id: row.get(0)?,
        learner_id: row.get(1)?,
        mission_id: row.get(2)?,
        session_id: row.get(3)?,
        reminder_schedule_id: row.get(4)?,
        academic_event_id: row.get(5)?,
        session_state: row.get(6)?,
        started_at: row.get(7)?,
        ended_at: row.get(8)?,
        completion_percent: row.get(9)?,
        missed_reason: row.get(10)?,
        source: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn map_engagement_risk_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<EngagementRiskProfile> {
    Ok(EngagementRiskProfile {
        learner_id: row.get(0)?,
        risk_level: row.get(1)?,
        risk_score_bp: row.get(2)?,
        consecutive_misses: row.get(3)?,
        recent_partial_sessions: row.get(4)?,
        last_session_state: row.get(5)?,
        next_recovery_action: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_parent_access_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<ParentAccessSettings> {
    let quiet_hours_json: String = row.get(8)?;
    Ok(ParentAccessSettings {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        learner_id: row.get(2)?,
        visibility_mode: row.get(3)?,
        reminders_enabled: row.get::<_, i64>(4)? == 1,
        alerts_enabled: row.get::<_, i64>(5)? == 1,
        feedback_enabled: row.get::<_, i64>(6)? == 1,
        can_excuse_sessions: row.get::<_, i64>(7)? == 1,
        preferred_channel: row.get(9)?,
        quiet_hours: parse_json_or_default(&quiet_hours_json),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn map_parent_feedback_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ParentFeedbackRecord> {
    Ok(ParentFeedbackRecord {
        id: row.get(0)?,
        learner_id: row.get(1)?,
        parent_id: row.get(2)?,
        category: row.get(3)?,
        message: row.get(4)?,
        interpreted_signal: row.get(5)?,
        urgency: row.get(6)?,
        suggested_support_action: row.get(7)?,
        visible_strategy_change: row.get(8)?,
        status: row.get(9)?,
        submitted_at: row.get(10)?,
        applied_at: row.get(11)?,
    })
}

fn map_parent_alert_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ParentAlertRecord> {
    let metadata_json: String = row.get(8)?;
    Ok(ParentAlertRecord {
        id: row.get(0)?,
        learner_id: row.get(1)?,
        parent_id: row.get(2)?,
        trigger_type: row.get(3)?,
        severity: row.get(4)?,
        message: row.get(5)?,
        action_required: row.get(6)?,
        status: row.get(7)?,
        metadata: parse_json_or_default(&metadata_json),
        created_at: row.get(9)?,
        acknowledged_at: row.get(10)?,
        resolved_at: row.get(11)?,
    })
}

fn map_strategy_adjustment_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<StrategyAdjustmentLog> {
    let old_snapshot_json: String = row.get(4)?;
    let new_snapshot_json: String = row.get(5)?;
    Ok(StrategyAdjustmentLog {
        id: row.get(0)?,
        learner_id: row.get(1)?,
        reason: row.get(2)?,
        source: row.get(3)?,
        old_strategy_snapshot: parse_json_or_default(&old_snapshot_json),
        new_strategy_snapshot: parse_json_or_default(&new_snapshot_json),
        visible_message_student: row.get(6)?,
        visible_message_parent: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn map_coach_badge_award(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoachBadgeAward> {
    let metadata_json: String = row.get(9)?;
    Ok(CoachBadgeAward {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        subject_name: row.get(10)?,
        topic_id: row.get(3)?,
        topic_name: row.get(11)?,
        badge_name: row.get(4)?,
        badge_family: row.get(5)?,
        reason: row.get(6)?,
        related_session_id: row.get(7)?,
        related_title_state_id: row.get(8)?,
        metadata: parse_json_or_default(&metadata_json),
        awarded_at: row.get(12)?,
    })
}

fn map_coach_title_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoachTitleCard> {
    let evidence_snapshot_json: String = row.get(10)?;
    let reclaim_plan_json: String = row.get(11)?;
    Ok(CoachTitleCard {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        subject_name: row.get(12)?,
        topic_id: row.get(3)?,
        topic_name: row.get(13)?,
        title_name: row.get(4)?,
        state: row.get(5)?,
        earned_at: row.get(6)?,
        last_defended_at: row.get(7)?,
        next_defense_due_at: row.get(8)?,
        coach_note: row.get(9)?,
        evidence_snapshot: parse_json_or_default(&evidence_snapshot_json),
        reclaim_plan: parse_json_or_default(&reclaim_plan_json),
    })
}

fn map_title_history_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoachTitleHistoryEntry> {
    let snapshot_json: String = row.get(5)?;
    Ok(CoachTitleHistoryEntry {
        id: row.get(0)?,
        title_state_id: row.get(1)?,
        previous_state: row.get(2)?,
        new_state: row.get(3)?,
        reason: row.get(4)?,
        snapshot: parse_json_or_default(&snapshot_json),
        created_at: row.get(6)?,
    })
}

fn beat_stage_from_readiness(readiness_score: BasisPoints) -> &'static str {
    match readiness_score {
        0..=2999 => "rescue",
        3000..=5499 => "stabilize",
        5500..=7799 => "accelerate",
        _ => "dominate",
    }
}

fn beat_mode_from_baseline(baseline: &SessionPerformanceBaseline) -> &'static str {
    if baseline.strain_score >= 7000 {
        return "recovery_mode";
    }
    if baseline.attempts < 8 {
        return "volume_push";
    }
    if baseline.accuracy_score < 6500 {
        return "accuracy_repair";
    }
    if baseline.avg_response_time_ms.unwrap_or(30_000) > 35_000 {
        return "speed_lift";
    }
    "volume_push"
}

fn beat_mode_reason(mode: &str) -> &'static str {
    match mode {
        "volume_push" => "The student needs a slightly larger daily output target.",
        "accuracy_repair" => {
            "Correctness is the current limiter, so today's climb protects pace while fixing errors."
        }
        "speed_lift" => {
            "Knowledge is present, but timing needs to improve without collapsing accuracy."
        }
        "recovery_mode" => {
            "Recent strain or regression suggests a lighter stabilizing climb is safer today."
        }
        _ => "Today's climb is aimed at steady compounding improvement.",
    }
}

fn beat_momentum_score(
    baseline_attempts: i64,
    baseline_accuracy_score: BasisPoints,
    baseline_avg_response_time_ms: Option<i64>,
    actual_attempts: i64,
    actual_accuracy_score: BasisPoints,
    actual_avg_response_time_ms: Option<i64>,
) -> BasisPoints {
    let volume_growth = centered_growth(actual_attempts as f64, baseline_attempts.max(1) as f64);
    let accuracy_growth = centered_growth(
        actual_accuracy_score as f64,
        baseline_accuracy_score.max(1) as f64,
    );
    let pace_growth = centered_inverse_growth(
        actual_avg_response_time_ms.unwrap_or(30_000) as f64,
        baseline_avg_response_time_ms.unwrap_or(30_000) as f64,
    );

    clamp_bp((0.35 * volume_growth + 0.40 * accuracy_growth + 0.25 * pace_growth).round() as i64)
        as BasisPoints
}

fn beat_strain_score(
    beat_attempt_target: bool,
    beat_accuracy_target: bool,
    beat_pace_target: bool,
    baseline_accuracy_score: BasisPoints,
    actual_accuracy_score: BasisPoints,
    actual_avg_response_time_ms: Option<i64>,
) -> BasisPoints {
    let missed_targets = [beat_attempt_target, beat_accuracy_target, beat_pace_target]
        .into_iter()
        .filter(|flag| !*flag)
        .count() as f64;
    let accuracy_drop = (baseline_accuracy_score as f64 - actual_accuracy_score as f64).max(0.0);
    let pace_drag = actual_avg_response_time_ms
        .unwrap_or(30_000)
        .saturating_sub(30_000) as f64;
    clamp_bp(
        (missed_targets / 3.0 * 4500.0 + accuracy_drop * 0.35 + pace_drag / 10.0).round() as i64,
    ) as BasisPoints
}

fn centered_growth(actual: f64, baseline: f64) -> f64 {
    let delta = ((actual - baseline) / baseline.max(1.0)).clamp(-1.0, 1.0);
    (5000.0 + 5000.0 * delta).clamp(0.0, 10_000.0)
}

fn centered_inverse_growth(actual: f64, baseline: f64) -> f64 {
    let delta = ((baseline - actual) / baseline.max(1.0)).clamp(-1.0, 1.0);
    (5000.0 + 5000.0 * delta).clamp(0.0, 10_000.0)
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn free_now_session_type_for_mode(mode: &str) -> &'static str {
    match mode {
        "accuracy_repair" => "repair_push",
        "speed_lift" => "speed_burst",
        "recovery_mode" => "light_reactivation",
        _ => "priority_push",
    }
}

fn total_target_minutes(target: &BeatYesterdayDailyTarget) -> i64 {
    target.warm_start_minutes
        + target.core_climb_minutes
        + target.speed_burst_minutes
        + target.finish_strong_minutes
}

fn estimate_remaining_target_minutes(
    target: &BeatYesterdayDailyTarget,
    actual: &SessionPerformanceBaseline,
) -> i64 {
    let total_minutes = total_target_minutes(target);
    let attempt_progress = if target.target_attempts > 0 {
        (actual.attempts as f64 / target.target_attempts as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let correct_progress = if target.target_correct > 0 {
        (actual.correct as f64 / target.target_correct as f64).clamp(0.0, 1.0)
    } else {
        attempt_progress
    };
    let blended_progress = (0.65 * attempt_progress + 0.35 * correct_progress).clamp(0.0, 1.0);
    ((total_minutes as f64) * (1.0 - blended_progress)).ceil() as i64
}

fn merge_topic_ids(primary: &[i64], secondary: &[i64], limit: usize) -> Vec<i64> {
    let mut seen = BTreeSet::new();
    let mut merged = Vec::new();
    for topic_id in primary.iter().chain(secondary.iter()) {
        if seen.insert(*topic_id) {
            merged.push(*topic_id);
        }
        if merged.len() >= limit {
            break;
        }
    }
    merged
}

fn window_consistency_bp(
    session_dates: &BTreeSet<NaiveDate>,
    as_of: NaiveDate,
    days: i64,
) -> BasisPoints {
    if days <= 0 {
        return 0;
    }
    let start = as_of - Duration::days(days - 1);
    let active_days = session_dates
        .iter()
        .filter(|date| **date >= start && **date <= as_of)
        .count() as i64;
    clamp_bp(((active_days as f64 / days as f64) * 10_000.0).round() as i64)
}

fn derive_dropout_risk_bp(
    current_streak_days: i64,
    days_since_last_session: i64,
    consistency_7d_bp: BasisPoints,
    consistency_14d_bp: BasisPoints,
    consistency_30d_bp: BasisPoints,
) -> BasisPoints {
    let recency_component = (days_since_last_session.max(0) * 750).min(10_000);
    let consistency_drag = ((10_000 - i64::from(consistency_7d_bp)).max(0)
        + (10_000 - i64::from(consistency_14d_bp)).max(0)
        + (10_000 - i64::from(consistency_30d_bp)).max(0))
        / 3;
    let streak_relief = if current_streak_days >= 3 {
        (current_streak_days.min(10) * 220).min(2_200)
    } else {
        0
    };
    clamp_bp(recency_component + consistency_drag / 2 - streak_relief)
}

fn derive_momentum_state(
    current_streak_days: i64,
    days_since_last_session: i64,
    consistency_7d_bp: BasisPoints,
    consistency_14d_bp: BasisPoints,
    dropout_risk_bp: BasisPoints,
) -> String {
    if days_since_last_session >= 10 || dropout_risk_bp >= 8_000 {
        "broken".to_string()
    } else if days_since_last_session >= 4
        || (current_streak_days == 0 && consistency_7d_bp < 3_000)
    {
        "comeback".to_string()
    } else if current_streak_days >= 5 && consistency_14d_bp >= 7_000 && dropout_risk_bp < 3_500 {
        "strong".to_string()
    } else if days_since_last_session >= 2 || consistency_7d_bp < 5_500 {
        "slipping".to_string()
    } else {
        "building".to_string()
    }
}

fn validate_comeback_trigger_reason(trigger_reason: &str) -> EcoachResult<()> {
    match trigger_reason {
        "missed_days" | "broken_momentum" | "long_absence" | "failed_session"
        | "exam_date_passed" | "subject_change" => Ok(()),
        _ => Err(EcoachError::Validation(format!(
            "unknown comeback trigger reason: {}",
            trigger_reason
        ))),
    }
}

fn normalize_goal_level(level: &str) -> &'static str {
    match level {
        "north_star" => "north_star",
        "current_campaign" | "campaign" => "campaign",
        "background" => "background",
        _ => "tactical",
    }
}

fn normalize_goal_state(goal_state: &str) -> &'static str {
    match goal_state {
        "drafted" => "drafted",
        "confirmed" => "confirmed",
        "paused" => "paused",
        "blocked" => "blocked",
        "completed" => "completed",
        "at_risk" => "at_risk",
        "recalibrating" => "recalibrating",
        _ => "active",
    }
}

fn legacy_goal_status(goal_state: &str) -> &'static str {
    match goal_state {
        "drafted" => "draft",
        "paused" => "paused",
        "completed" => "completed",
        _ => "active",
    }
}

fn score_goal_urgency(urgency_level: &str) -> BasisPoints {
    match urgency_level {
        "critical" => 10_000,
        "high" => 8_300,
        "elevated" => 7_000,
        "low" => 3_500,
        _ => 5_500,
    }
}

fn score_goal_level(level: &str) -> BasisPoints {
    match level {
        "north_star" => 9_400,
        "campaign" | "current_campaign" => 8_100,
        "background" => 2_800,
        _ => 6_200,
    }
}

impl<'a> GoalsCalendarService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_goal(&self, student_id: i64, title: &str, goal_type: &str) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO goals (student_id, goal_type, title, status) VALUES (?1, ?2, ?3, 'active')",
            params![student_id, goal_type, title],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_goals(&self, student_id: i64) -> EcoachResult<Vec<Goal>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, student_id, goal_type, title, description, status FROM goals WHERE student_id = ?1 ORDER BY created_at DESC"
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([student_id], |row| {
                Ok(Goal {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    goal_type: row.get(2)?,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    pub fn save_goal_profile(
        &self,
        student_id: i64,
        input: GoalProfileInput,
    ) -> EcoachResult<GoalProfile> {
        let goal_level = normalize_goal_level(&input.level).to_string();
        let goal_state = normalize_goal_state(&input.goal_state).to_string();
        let topics_json = serde_json::to_string(&input.topics)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let evidence_sources_json = serde_json::to_string(&input.evidence_sources)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let dependency_goals_json = serde_json::to_string(&input.dependency_goal_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let completion_criteria_json = serde_json::to_string(&input.completion_criteria)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let metadata_json = serde_json::to_string(&input.metadata)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let existing_id = if input.source_bundle_id.is_some() || input.goal_signal_key.is_some() {
            self.conn
                .query_row(
                    "SELECT id
                     FROM goals
                     WHERE student_id = ?1
                       AND COALESCE(source_bundle_id, -1) = COALESCE(?2, -1)
                       AND COALESCE(goal_signal_key, '') = COALESCE(?3, '')
                     ORDER BY updated_at DESC
                     LIMIT 1",
                    params![student_id, input.source_bundle_id, input.goal_signal_key],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else {
            None
        };

        if let Some(goal_id) = existing_id {
            self.conn
                .execute(
                    "UPDATE goals
                     SET title = ?2,
                         description = ?3,
                         goal_type = ?4,
                         goal_category = ?5,
                         goal_level = ?6,
                         goal_state = ?7,
                         status = ?8,
                         subject_id = ?9,
                         topics_json = ?10,
                         urgency_level = ?11,
                         start_date = ?12,
                         deadline = ?13,
                         exam_id = ?14,
                         confidence_score_bp = ?15,
                         coach_priority_bp = ?16,
                         parent_priority_flag = ?17,
                         evidence_sources_json = ?18,
                         dependency_goals_json = ?19,
                         risk_level = ?20,
                         suggested_weekly_effort_minutes = ?21,
                         current_momentum_bp = ?22,
                         completion_criteria_json = ?23,
                         blocked_reason = ?24,
                         source_bundle_id = ?25,
                         goal_signal_key = ?26,
                         metadata_json = ?27,
                         updated_at = datetime('now')
                     WHERE id = ?1",
                    params![
                        goal_id,
                        input.title,
                        input.description,
                        goal_level,
                        input.category,
                        goal_level,
                        goal_state,
                        legacy_goal_status(&goal_state),
                        input.subject_id,
                        topics_json,
                        input.urgency_level,
                        input.start_date,
                        input.deadline,
                        input.exam_id,
                        input.confidence_score_bp,
                        input.coach_priority_bp,
                        if input.parent_priority_flag { 1 } else { 0 },
                        evidence_sources_json,
                        dependency_goals_json,
                        input.risk_level,
                        input.suggested_weekly_effort_minutes,
                        input.current_momentum_bp,
                        completion_criteria_json,
                        input.blocked_reason,
                        input.source_bundle_id,
                        input.goal_signal_key,
                        metadata_json,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            return self
                .load_goal_profile(goal_id)?
                .ok_or_else(|| EcoachError::NotFound(format!("goal {} not found", goal_id)));
        }

        self.conn
            .execute(
                "INSERT INTO goals (
                    student_id, goal_type, title, description, status, goal_level, goal_state,
                    coach_priority_bp, evidence_sources_json, dependency_goals_json, risk_level,
                    completion_criteria_json, momentum_state, goal_category, subject_id,
                    topics_json, urgency_level, start_date, deadline, exam_id,
                    confidence_score_bp, parent_priority_flag, suggested_weekly_effort_minutes,
                    current_momentum_bp, blocked_reason, goal_signal_key, source_bundle_id,
                    metadata_json, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28,
                    datetime('now'), datetime('now')
                 )",
                params![
                    student_id,
                    goal_level,
                    input.title,
                    input.description,
                    legacy_goal_status(&goal_state),
                    goal_level,
                    goal_state,
                    input.coach_priority_bp,
                    evidence_sources_json,
                    dependency_goals_json,
                    input.risk_level,
                    completion_criteria_json,
                    Some(format!("bp:{}", input.current_momentum_bp)),
                    input.category,
                    input.subject_id,
                    topics_json,
                    input.urgency_level,
                    input.start_date,
                    input.deadline,
                    input.exam_id,
                    input.confidence_score_bp,
                    if input.parent_priority_flag { 1 } else { 0 },
                    input.suggested_weekly_effort_minutes,
                    input.current_momentum_bp,
                    input.blocked_reason,
                    input.goal_signal_key,
                    input.source_bundle_id,
                    metadata_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let goal_id = self.conn.last_insert_rowid();
        self.load_goal_profile(goal_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("goal {} not found", goal_id)))
    }

    pub fn update_goal_profile_state(
        &self,
        goal_id: i64,
        goal_state: &str,
        blocked_reason: Option<&str>,
    ) -> EcoachResult<GoalProfile> {
        let normalized = normalize_goal_state(goal_state);
        self.conn
            .execute(
                "UPDATE goals
                 SET goal_state = ?2,
                     status = ?3,
                     blocked_reason = ?4,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    goal_id,
                    normalized,
                    legacy_goal_status(normalized),
                    blocked_reason
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.load_goal_profile(goal_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("goal {} not found", goal_id)))
    }

    pub fn list_goal_profiles(&self, student_id: i64) -> EcoachResult<Vec<GoalProfile>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, parent_goal_id, title, description, goal_type,
                        goal_category, goal_level, goal_state, subject_id, topics_json,
                        urgency_level, start_date, deadline, exam_id, confidence_score_bp,
                        coach_priority_bp, evidence_sources_json, dependency_goals_json,
                        parent_priority_flag, risk_level, suggested_weekly_effort_minutes,
                        completion_criteria_json, current_momentum_bp, blocked_reason,
                        source_bundle_id, goal_signal_key, metadata_json, created_at, updated_at
                 FROM goals
                 WHERE student_id = ?1
                 ORDER BY coach_priority_bp DESC, parent_priority_flag DESC, deadline ASC, updated_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], map_goal_profile)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn build_goal_arbitration_snapshot(
        &self,
        student_id: i64,
    ) -> EcoachResult<GoalArbitrationSnapshot> {
        let goals = self.list_goal_profiles(student_id)?;
        let mut ranked: Vec<(BasisPoints, &GoalProfile)> = goals
            .iter()
            .map(|goal| {
                let mut score = goal.coach_priority_bp as i64
                    + score_goal_urgency(&goal.urgency_level) as i64 / 3
                    + score_goal_level(&goal.goal_level) as i64 / 4
                    + goal.current_momentum_bp as i64 / 6
                    + if goal.parent_priority_flag { 900 } else { 0 };
                if matches!(goal.goal_state.as_str(), "blocked" | "paused") {
                    score -= 2_500;
                }
                if goal.goal_state == "at_risk" {
                    score += 1_200;
                }
                if goal.deadline.is_some() {
                    score += 700;
                }
                (clamp_bp(score), goal)
            })
            .collect();
        ranked.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });

        let focus_goal_ids = ranked
            .iter()
            .filter(|(_, goal)| {
                !matches!(goal.goal_state.as_str(), "paused" | "blocked" | "completed")
            })
            .take(3)
            .map(|(_, goal)| goal.id)
            .collect::<Vec<_>>();
        let primary_goal_id = focus_goal_ids.first().copied();
        let paused_goal_ids = goals
            .iter()
            .filter(|goal| goal.goal_state == "paused")
            .map(|goal| goal.id)
            .collect::<Vec<_>>();
        let blocked_goal_ids = goals
            .iter()
            .filter(|goal| goal.goal_state == "blocked")
            .map(|goal| goal.id)
            .collect::<Vec<_>>();
        let mut conflicts = Vec::new();
        for left in &goals {
            for right in &goals {
                if left.id >= right.id {
                    continue;
                }
                if left.subject_id == right.subject_id
                    && left.goal_state != "completed"
                    && right.goal_state != "completed"
                    && left.goal_level == right.goal_level
                    && left.deadline.is_some()
                    && right.deadline.is_some()
                {
                    conflicts.push(GoalConflict {
                        goal_id: left.id,
                        conflicting_goal_id: right.id,
                        conflict_type: "same_lane_pressure".to_string(),
                        summary: format!(
                            "{} and {} are competing for the same study lane.",
                            left.title, right.title
                        ),
                    });
                }
            }
        }
        let mut rationale = Vec::new();
        if let Some(primary_goal_id) = primary_goal_id {
            if let Some(primary) = goals.iter().find(|goal| goal.id == primary_goal_id) {
                rationale.push(format!(
                    "{} is primary because it leads on coach priority and urgency.",
                    primary.title
                ));
            }
        }
        if !conflicts.is_empty() {
            rationale.push("Some goals are competing for the same weekly capacity.".to_string());
        }
        if !blocked_goal_ids.is_empty() {
            rationale.push(
                "Blocked goals should be resolved before they re-enter the main queue.".to_string(),
            );
        }

        Ok(GoalArbitrationSnapshot {
            student_id,
            generated_at: Utc::now().to_rfc3339(),
            primary_goal_id,
            focus_goal_ids,
            paused_goal_ids,
            blocked_goal_ids,
            conflicts,
            rationale,
            goals,
        })
    }

    pub fn build_weekly_plan_snapshot(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        anchor_date: &str,
    ) -> EcoachResult<WeeklyPlanSnapshot> {
        let anchor = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d").map_err(|err| {
            EcoachError::Validation(format!("invalid anchor_date {}: {}", anchor_date, err))
        })?;
        let arbitration = self.build_goal_arbitration_snapshot(student_id)?;
        let prioritized_event = self
            .build_academic_calendar_snapshot(student_id, Some(anchor_date))?
            .prioritized_event;
        let prioritized_exam_id = prioritized_event.as_ref().map(|event| event.id);
        let prioritized_subject_id = subject_id.or_else(|| {
            prioritized_event
                .as_ref()
                .and_then(|event| event.subject_id)
        });
        let weak_topic_ids = if let Some(subject_id) = prioritized_subject_id {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT topic_id
                     FROM student_topic_states
                     WHERE student_id = ?1
                       AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)
                     ORDER BY priority_score DESC, gap_score DESC
                     LIMIT 6",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map(params![student_id, subject_id], |row| row.get(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
            items
        } else {
            Vec::new()
        };
        let due_memory_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM memory_states
                 WHERE student_id = ?1
                   AND review_due_at IS NOT NULL
                   AND review_due_at <= ?2",
                params![student_id, Utc::now().to_rfc3339()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let exam_weight = if prioritized_event.is_some() { 3 } else { 2 };
        let weakness_weight = if !weak_topic_ids.is_empty() { 3 } else { 2 };
        let maintenance_weight = if due_memory_count > 0 { 2 } else { 1 };
        let continuity_weight = 1;
        let weight_total = exam_weight + weakness_weight + maintenance_weight + continuity_weight;
        let mut bands = vec![
            WeeklyPlanBand {
                band_key: "exam_prep".to_string(),
                allocated_minutes: 0,
                rationale: "Protect time for the nearest exam horizon.".to_string(),
                focus_topic_ids: prioritized_event
                    .as_ref()
                    .map(|event| event.linked_topic_ids.clone())
                    .unwrap_or_default(),
            },
            WeeklyPlanBand {
                band_key: "weakness_repair".to_string(),
                allocated_minutes: 0,
                rationale: "Repair high-gap topics before they spread into new work.".to_string(),
                focus_topic_ids: weak_topic_ids.clone(),
            },
            WeeklyPlanBand {
                band_key: "maintenance".to_string(),
                allocated_minutes: 0,
                rationale: "Keep memory-return and review debt under control.".to_string(),
                focus_topic_ids: weak_topic_ids.iter().copied().take(3).collect(),
            },
            WeeklyPlanBand {
                band_key: "continuity".to_string(),
                allocated_minutes: 0,
                rationale: "Preserve rhythm even on lighter days.".to_string(),
                focus_topic_ids: Vec::new(),
            },
        ];
        let mut days = Vec::new();
        for offset in 0..7 {
            let date = anchor + Duration::days(offset);
            let date_text = date.format("%Y-%m-%d").to_string();
            let availability = self.get_daily_availability(student_id, &date_text)?;
            let planned_minutes = (availability.adjusted_minutes * 82 / 100).max(0);
            let mut blocks = Vec::new();
            if planned_minutes > 0 {
                let exam_minutes = planned_minutes * exam_weight / weight_total;
                let repair_minutes = planned_minutes * weakness_weight / weight_total;
                let maintenance_minutes = planned_minutes * maintenance_weight / weight_total;
                let continuity_minutes =
                    (planned_minutes - exam_minutes - repair_minutes - maintenance_minutes).max(0);
                bands[0].allocated_minutes += exam_minutes;
                bands[1].allocated_minutes += repair_minutes;
                bands[2].allocated_minutes += maintenance_minutes;
                bands[3].allocated_minutes += continuity_minutes;

                if exam_minutes > 0 {
                    blocks.push(WeeklyPlanBlock {
                        band_key: "exam_prep".to_string(),
                        session_type: if offset >= 4 { "timed_drill" } else { "exam_focus" }
                            .to_string(),
                        duration_minutes: exam_minutes,
                        focus_topic_ids: bands[0].focus_topic_ids.clone(),
                        exam_support_scope: if prioritized_event.is_some() {
                            "nearest_plus_final".to_string()
                        } else {
                            "all_active_tracks".to_string()
                        },
                        rationale: "Move the nearest exam track forward without dropping long-range preparation.".to_string(),
                    });
                }
                if repair_minutes > 0 {
                    blocks.push(WeeklyPlanBlock {
                        band_key: "weakness_repair".to_string(),
                        session_type: "repair".to_string(),
                        duration_minutes: repair_minutes,
                        focus_topic_ids: weak_topic_ids.iter().copied().take(3).collect(),
                        exam_support_scope: "all_active_tracks".to_string(),
                        rationale: "Repair recurring upload and learner-truth weaknesses."
                            .to_string(),
                    });
                }
                if maintenance_minutes > 0 {
                    blocks.push(WeeklyPlanBlock {
                        band_key: "maintenance".to_string(),
                        session_type: "memory_return".to_string(),
                        duration_minutes: maintenance_minutes,
                        focus_topic_ids: weak_topic_ids.iter().copied().skip(1).take(2).collect(),
                        exam_support_scope: "maintenance".to_string(),
                        rationale: "Prevent fragile knowledge from slipping out of reach."
                            .to_string(),
                    });
                }
                if continuity_minutes > 0 {
                    blocks.push(WeeklyPlanBlock {
                        band_key: "continuity".to_string(),
                        session_type: "light_continuity".to_string(),
                        duration_minutes: continuity_minutes,
                        focus_topic_ids: Vec::new(),
                        exam_support_scope: "all_active_tracks".to_string(),
                        rationale: "Keep momentum alive on the day, even if capacity is tight."
                            .to_string(),
                    });
                }
            }
            days.push(WeeklyPlanDay {
                date: date_text,
                available_minutes: availability.adjusted_minutes,
                planned_minutes,
                block_count: blocks.len() as i64,
                blocks,
            });
        }

        let mut priorities = arbitration
            .goals
            .iter()
            .take(3)
            .map(|goal| format!("{} ({})", goal.title, goal.goal_state))
            .collect::<Vec<_>>();
        if due_memory_count > 0 {
            priorities.push(format!(
                "{} memory items are due for return-loop work.",
                due_memory_count
            ));
        }
        if let Some(event) = prioritized_event.as_ref() {
            priorities.push(format!("{} is shaping this week.", event.title));
        }

        Ok(WeeklyPlanSnapshot {
            student_id,
            subject_id: prioritized_subject_id,
            anchor_date: anchor_date.to_string(),
            prioritized_exam_id,
            priorities,
            bands,
            days,
        })
    }

    pub fn create_event(
        &self,
        student_id: i64,
        event_type: &str,
        title: &str,
        scheduled_for: &str,
    ) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO calendar_events (student_id, event_type, title, scheduled_for) VALUES (?1, ?2, ?3, ?4)",
            params![student_id, event_type, title, scheduled_for],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        let legacy_event_id = self.conn.last_insert_rowid();
        let (scheduled_date, start_time) = split_legacy_scheduled_for(scheduled_for);
        self.conn
            .execute(
                "INSERT INTO academic_calendar_events (
                    student_id, legacy_calendar_event_id, title, event_type, scheduled_date,
                    start_time, status, importance_bp, scope, preparation_window_days,
                    review_window_days, coach_priority_weight_bp, expected_weight_bp,
                    timed_performance_weight_bp, coverage_mode, source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'scheduled', ?7, 'focused', 14, 7, ?8, ?9, ?10, 'mixed', 'legacy')",
                params![
                    student_id,
                    legacy_event_id,
                    title,
                    normalize_academic_event_type(event_type),
                    scheduled_date,
                    start_time,
                    legacy_event_importance(event_type),
                    legacy_event_importance(event_type),
                    legacy_event_importance(event_type),
                    if event_type == "mock" || event_type == "exam" {
                        7000
                    } else {
                        4500
                    },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(legacy_event_id)
    }

    pub fn next_event(&self, student_id: i64) -> EcoachResult<Option<CalendarEvent>> {
        let next_academic = self
            .conn
            .query_row(
                "SELECT id, student_id, event_type, title, scheduled_date, start_time
                 FROM academic_calendar_events
                 WHERE student_id = ?1 AND status IN ('scheduled', 'postponed')
                 ORDER BY scheduled_date ASC, COALESCE(start_time, '23:59') ASC LIMIT 1",
                [student_id],
                |row| {
                    Ok(CalendarEvent {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        event_type: row.get(2)?,
                        title: row.get(3)?,
                        scheduled_for: combine_date_and_time(
                            &row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?.as_deref(),
                        ),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if next_academic.is_some() {
            return Ok(next_academic);
        }
        self.conn
            .query_row(
                "SELECT id, student_id, event_type, title, scheduled_for
                 FROM calendar_events WHERE student_id = ?1 ORDER BY scheduled_for ASC LIMIT 1",
                [student_id],
                |row| {
                    Ok(CalendarEvent {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        event_type: row.get(2)?,
                        title: row.get(3)?,
                        scheduled_for: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn upsert_academic_event(
        &self,
        student_id: i64,
        event_id: Option<i64>,
        input: &AcademicCalendarEventInput,
    ) -> EcoachResult<AcademicCalendarEvent> {
        NaiveDate::parse_from_str(&input.scheduled_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let normalized_type = normalize_academic_event_type(&input.event_type);
        let source = input.source.clone().unwrap_or_else(|| "manual".to_string());
        let old_snapshot = self.capture_strategy_snapshot(student_id).ok();
        let linked_topic_ids_json = serde_json::to_string(&input.linked_topic_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let strategy_anchor = self.plan_anchor_date(student_id)?;

        if let Some(event_id) = event_id {
            self.conn
                .execute(
                    "UPDATE academic_calendar_events
                     SET title = ?2,
                         event_type = ?3,
                         subject_id = ?4,
                         scheduled_date = ?5,
                         start_time = ?6,
                         end_time = ?7,
                         term = ?8,
                         academic_year = ?9,
                         importance_bp = ?10,
                         scope = ?11,
                         linked_topic_ids_json = ?12,
                         preparation_window_days = ?13,
                         review_window_days = ?14,
                         status = ?15,
                         result_after_event = ?16,
                         coach_priority_weight_bp = ?17,
                         expected_weight_bp = ?18,
                         timed_performance_weight_bp = ?19,
                         coverage_mode = ?20,
                         source = ?21,
                         updated_at = datetime('now')
                     WHERE id = ?1 AND student_id = ?22",
                    params![
                        event_id,
                        input.title,
                        normalized_type,
                        input.subject_id,
                        input.scheduled_date,
                        input.start_time,
                        input.end_time,
                        input.term,
                        input.academic_year,
                        input.importance_bp,
                        input.scope,
                        linked_topic_ids_json,
                        input.preparation_window_days,
                        input.review_window_days,
                        input.status,
                        input.result_after_event,
                        input.coach_priority_weight_bp,
                        input.expected_weight_bp,
                        input.timed_performance_weight_bp,
                        input.coverage_mode,
                        source,
                        student_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO academic_calendar_events (
                        student_id, title, event_type, subject_id, scheduled_date,
                        start_time, end_time, term, academic_year, importance_bp, scope,
                        linked_topic_ids_json, preparation_window_days, review_window_days,
                        status, result_after_event, coach_priority_weight_bp, expected_weight_bp,
                        timed_performance_weight_bp, coverage_mode, source
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
                    params![
                        student_id,
                        input.title,
                        normalized_type,
                        input.subject_id,
                        input.scheduled_date,
                        input.start_time,
                        input.end_time,
                        input.term,
                        input.academic_year,
                        input.importance_bp,
                        input.scope,
                        linked_topic_ids_json,
                        input.preparation_window_days,
                        input.review_window_days,
                        input.status,
                        input.result_after_event,
                        input.coach_priority_weight_bp,
                        input.expected_weight_bp,
                        input.timed_performance_weight_bp,
                        input.coverage_mode,
                        source,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        let stored_id = event_id.unwrap_or_else(|| self.conn.last_insert_rowid());
        let profile = self.get_preparation_intensity_profile(student_id, Some(&strategy_anchor))?;
        if let Some(profile) = profile.as_ref() {
            let snapshot = json!({
                "phase": profile.phase,
                "urgency_level": profile.urgency_level,
                "recommended_mode": profile.recommended_mode,
                "revision_density": profile.revision_density,
                "mission_priority_influence": profile.mission_priority_influence,
            });
            self.conn
                .execute(
                    "UPDATE academic_calendar_events
                     SET last_strategy_snapshot_json = ?2, updated_at = datetime('now')
                     WHERE id = ?1",
                    params![stored_id, snapshot.to_string()],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.apply_intensity_to_active_plan(student_id, profile)?;
            let new_snapshot = self.capture_strategy_snapshot(student_id)?;
            self.record_strategy_adjustment(
                student_id,
                &format!("Academic calendar updated for {}", input.title),
                "calendar",
                old_snapshot.unwrap_or_else(|| json!({})),
                new_snapshot,
                Some(format!(
                    "{} is now shaping strategy. I shifted the plan into {} mode.",
                    input.title, profile.recommended_mode
                )),
                Some(format!(
                    "{} changed the preparation runway. The coach shifted into {} mode.",
                    input.title, profile.recommended_mode
                )),
            )?;
        }
        self.emit_runtime_event(
            if event_id.is_some() {
                "academic_event_updated"
            } else {
                "academic_event_created"
            },
            "academic_calendar_event",
            stored_id.to_string(),
            json!({
                "eventId": stored_id,
                "type": normalized_type,
                "date": input.scheduled_date,
                "subjectScope": input.subject_id,
                "source": input.source,
            }),
        )?;
        self.get_academic_event(student_id, stored_id, Some(&strategy_anchor))?
            .ok_or_else(|| EcoachError::NotFound("Academic event missing after upsert".to_string()))
    }

    pub fn list_academic_events(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<Vec<AcademicCalendarEvent>> {
        self.load_academic_events(student_id, anchor_date, false)
    }

    pub fn get_preparation_intensity_profile(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<Option<PreparationIntensityProfile>> {
        let anchor_date = self.resolve_anchor_date(anchor_date)?;
        let events = self.load_academic_events(student_id, Some(&anchor_date), false)?;
        let prioritized = events
            .into_iter()
            .filter(|event| event.status == "scheduled" || event.status == "postponed")
            .max_by_key(|event| event.mission_priority_influence.unwrap_or_default());

        let Some(event) = prioritized else {
            return Ok(None);
        };
        let snapshot = self.compute_event_readiness(student_id, &event, &anchor_date)?;
        Ok(Some(PreparationIntensityProfile {
            anchor_date,
            prioritized_event_id: Some(event.id),
            prioritized_event_title: Some(event.title),
            phase: snapshot.phase,
            days_to_event: event.days_to_event,
            available_study_days: snapshot.available_study_days,
            readiness_score: snapshot.readiness_score,
            gap_score: snapshot.gap_score,
            subject_risk_level: snapshot.subject_risk_level,
            urgency_level: snapshot.urgency_level,
            recommended_mode: snapshot.recommended_mode,
            revision_density: snapshot.revision_density,
            mission_priority_influence: snapshot.mission_priority_influence,
            explanation_weight_bp: snapshot.explanation_weight_bp,
            retrieval_weight_bp: snapshot.retrieval_weight_bp,
            timed_drill_weight_bp: snapshot.timed_drill_weight_bp,
            breadth_weight_bp: snapshot.breadth_weight_bp,
            tone: snapshot.tone,
            rationale: snapshot.rationale,
        }))
    }

    pub fn build_academic_calendar_snapshot(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<AcademicCalendarSnapshot> {
        let anchor_date = self.resolve_anchor_date(anchor_date)?;
        let events = self.load_academic_events(student_id, Some(&anchor_date), false)?;
        let intensity = self.get_preparation_intensity_profile(student_id, Some(&anchor_date))?;
        let prioritized_event = intensity
            .as_ref()
            .and_then(|profile| profile.prioritized_event_id)
            .and_then(|event_id| events.iter().find(|event| event.id == event_id).cloned());
        let strategy_message = if let Some(profile) = intensity.as_ref() {
            let title = prioritized_event
                .as_ref()
                .map(|event| event.title.clone())
                .unwrap_or_else(|| "the next key event".to_string());
            format!(
                "{} is driving strategy. The coach is operating in {} mode with {} revision density.",
                title, profile.recommended_mode, profile.revision_density
            )
        } else {
            "No academic event is currently shaping strategy, so the coach stays in steady build mode.".to_string()
        };

        Ok(AcademicCalendarSnapshot {
            generated_at: Utc::now().to_rfc3339(),
            anchor_date,
            prioritized_event,
            intensity,
            strategy_message,
            events,
        })
    }

    pub fn schedule_reminder(
        &self,
        learner_id: i64,
        input: &ReminderScheduleInput,
    ) -> EcoachResult<ReminderSchedule> {
        let message = input
            .message
            .clone()
            .unwrap_or(self.build_reminder_message(learner_id, input)?);
        self.conn
            .execute(
                "INSERT INTO reminder_schedules (
                    learner_id, mission_id, academic_event_id, reminder_type, scheduled_time,
                    audience, status, escalation_level, message, metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'scheduled', ?7, ?8, ?9)",
                params![
                    learner_id,
                    input.mission_id,
                    input.academic_event_id,
                    input.reminder_type,
                    input.scheduled_time,
                    input.audience,
                    input.escalation_level,
                    message,
                    input.metadata.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let reminder_id = self.conn.last_insert_rowid();
        self.emit_runtime_event(
            "reminder_scheduled",
            "learner",
            learner_id.to_string(),
            json!({
                "reminderId": reminder_id,
                "type": input.reminder_type,
                "audience": input.audience,
                "scheduledTime": input.scheduled_time,
            }),
        )?;
        self.get_reminder_by_id(reminder_id)?
            .ok_or_else(|| EcoachError::NotFound("Reminder not found after insert".to_string()))
    }

    pub fn list_reminders(
        &self,
        learner_id: i64,
        audience: Option<&str>,
        status: Option<&str>,
        limit: i64,
    ) -> EcoachResult<Vec<ReminderSchedule>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, learner_id, mission_id, academic_event_id, reminder_type,
                        scheduled_time, audience, status, escalation_level, metadata_json,
                        message, sent_at, acknowledged_at, created_at
                 FROM reminder_schedules
                 WHERE learner_id = ?1
                   AND (?2 IS NULL OR audience = ?2)
                   AND (?3 IS NULL OR status = ?3)
                 ORDER BY scheduled_time ASC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![learner_id, audience, status, limit.max(1)],
                map_reminder_schedule,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.collect_rows(rows)
    }

    pub fn acknowledge_reminder(&self, reminder_id: i64) -> EcoachResult<Option<ReminderSchedule>> {
        self.conn
            .execute(
                "UPDATE reminder_schedules
                 SET status = 'acknowledged', acknowledged_at = datetime('now')
                 WHERE id = ?1",
                [reminder_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let reminder = self.get_reminder_by_id(reminder_id)?;
        if let Some(reminder) = reminder.as_ref() {
            self.emit_runtime_event(
                "reminder_acknowledged",
                "learner",
                reminder.learner_id.to_string(),
                json!({
                    "reminderId": reminder.id,
                    "type": reminder.reminder_type,
                }),
            )?;
        }
        Ok(reminder)
    }

    pub fn record_engagement_event(
        &self,
        learner_id: i64,
        input: &EngagementEventInput,
    ) -> EcoachResult<EngagementEvent> {
        let old_snapshot = self.capture_strategy_snapshot(learner_id).ok();
        self.conn
            .execute(
                "INSERT INTO engagement_events (
                    learner_id, mission_id, session_id, reminder_schedule_id, academic_event_id,
                    session_state, started_at, ended_at, completion_percent, missed_reason, source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    learner_id,
                    input.mission_id,
                    input.session_id,
                    input.reminder_schedule_id,
                    input.academic_event_id,
                    input.session_state,
                    input.started_at,
                    input.ended_at,
                    input.completion_percent,
                    input.missed_reason,
                    input.source.clone().unwrap_or_else(|| "coach".to_string()),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let event_id = self.conn.last_insert_rowid();
        let event = self.get_engagement_event_by_id(event_id)?.ok_or_else(|| {
            EcoachError::NotFound("Engagement event missing after insert".to_string())
        })?;
        let risk_profile = self.sync_engagement_risk_profile(learner_id)?;

        let strategy_adjustment_id = if matches!(
            event.session_state.as_str(),
            "missed" | "partially_completed" | "rescheduled"
        ) {
            if event.session_state == "missed" {
                self.schedule_recovery_mission(learner_id, input.mission_id, &event)?;
            }
            let severity = self.compute_parent_alert_severity(
                learner_id,
                &event.session_state,
                input.mission_id,
                input.academic_event_id,
                risk_profile.consecutive_misses,
            )?;
            let new_snapshot = self.capture_strategy_snapshot(learner_id)?;
            let adjustment_id = self.record_strategy_adjustment(
                learner_id,
                &format!("{} session detected", event.session_state),
                "engagement",
                old_snapshot.unwrap_or_else(|| json!({})),
                new_snapshot,
                Some(self.student_engagement_message(&event.session_state)),
                Some(self.parent_engagement_message(&event.session_state)),
            )?;
            self.create_parent_alerts_for_engagement(
                learner_id,
                &event,
                &severity,
                Some(adjustment_id),
            )?;
            Some(adjustment_id)
        } else {
            None
        };

        self.emit_runtime_event(
            "engagement_event_recorded",
            "learner",
            learner_id.to_string(),
            json!({
                "engagementEventId": event.id,
                "missionId": event.mission_id,
                "sessionId": event.session_id,
                "sessionState": event.session_state,
                "completionPercent": event.completion_percent,
                "strategyAdjustmentId": strategy_adjustment_id,
            }),
        )?;
        Ok(event)
    }

    pub fn get_engagement_risk_profile(
        &self,
        learner_id: i64,
    ) -> EcoachResult<Option<EngagementRiskProfile>> {
        self.conn
            .query_row(
                "SELECT learner_id, risk_level, risk_score_bp, consecutive_misses,
                        recent_partial_sessions, last_session_state, next_recovery_action, updated_at
                 FROM engagement_risk_profiles
                 WHERE learner_id = ?1",
                [learner_id],
                map_engagement_risk_profile,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn upsert_parent_access_settings(
        &self,
        parent_id: i64,
        learner_id: i64,
        input: &ParentAccessSettingsInput,
    ) -> EcoachResult<ParentAccessSettings> {
        self.ensure_parent_link(parent_id, learner_id)?;
        self.conn
            .execute(
                "INSERT INTO parent_access_settings (
                    parent_id, learner_id, visibility_mode, reminders_enabled, alerts_enabled,
                    feedback_enabled, can_excuse_sessions, preferred_channel, quiet_hours_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(parent_id, learner_id) DO UPDATE SET
                    visibility_mode = excluded.visibility_mode,
                    reminders_enabled = excluded.reminders_enabled,
                    alerts_enabled = excluded.alerts_enabled,
                    feedback_enabled = excluded.feedback_enabled,
                    can_excuse_sessions = excluded.can_excuse_sessions,
                    preferred_channel = excluded.preferred_channel,
                    quiet_hours_json = excluded.quiet_hours_json,
                    updated_at = datetime('now')",
                params![
                    parent_id,
                    learner_id,
                    input.visibility_mode,
                    bool_to_i64(input.reminders_enabled),
                    bool_to_i64(input.alerts_enabled),
                    bool_to_i64(input.feedback_enabled),
                    bool_to_i64(input.can_excuse_sessions),
                    input.preferred_channel,
                    input.quiet_hours.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_parent_access_settings(parent_id, learner_id)?
            .ok_or_else(|| {
                EcoachError::NotFound("Parent access settings missing after upsert".to_string())
            })
    }

    pub fn get_parent_access_settings(
        &self,
        parent_id: i64,
        learner_id: i64,
    ) -> EcoachResult<Option<ParentAccessSettings>> {
        self.conn
            .query_row(
                "SELECT id, parent_id, learner_id, visibility_mode, reminders_enabled,
                        alerts_enabled, feedback_enabled, can_excuse_sessions, quiet_hours_json,
                        preferred_channel, created_at, updated_at
                 FROM parent_access_settings
                 WHERE parent_id = ?1 AND learner_id = ?2",
                params![parent_id, learner_id],
                map_parent_access_settings,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn submit_parent_feedback(
        &self,
        parent_id: i64,
        learner_id: i64,
        input: &ParentFeedbackInput,
    ) -> EcoachResult<ParentFeedbackRecord> {
        self.ensure_parent_link(parent_id, learner_id)?;
        let old_snapshot = self.capture_strategy_snapshot(learner_id)?;
        let interpreted_signal = interpret_parent_feedback_signal(&input.category, &input.message);
        let suggested_support_action = parent_support_action(&interpreted_signal);
        let visible_strategy_change = strategy_change_from_feedback(&interpreted_signal);
        self.conn
            .execute(
                "INSERT INTO parent_feedback_records (
                    learner_id, parent_id, category, message, interpreted_signal, urgency,
                    suggested_support_action, visible_strategy_change, status, applied_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'applied', datetime('now'))",
                params![
                    learner_id,
                    parent_id,
                    input.category,
                    input.message,
                    interpreted_signal,
                    input.urgency,
                    suggested_support_action,
                    visible_strategy_change,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let feedback_id = self.conn.last_insert_rowid();
        self.apply_feedback_to_active_plan(learner_id, &interpreted_signal)?;
        let new_snapshot = self.capture_strategy_snapshot(learner_id)?;
        self.record_strategy_adjustment(
            learner_id,
            &format!("Parent feedback: {}", input.category),
            "feedback",
            old_snapshot,
            new_snapshot,
            Some(visible_strategy_change.clone()),
            Some(format!(
                "Parent feedback flagged {}. The coach adjusted the plan.",
                interpreted_signal.replace('_', " ")
            )),
        )?;
        self.emit_runtime_event(
            "parent_feedback_recorded",
            "learner",
            learner_id.to_string(),
            json!({
                "feedbackId": feedback_id,
                "parentId": parent_id,
                "category": input.category,
                "interpretedSignal": interpreted_signal,
                "urgency": input.urgency,
            }),
        )?;
        self.get_parent_feedback_by_id(feedback_id)?.ok_or_else(|| {
            EcoachError::NotFound("Parent feedback missing after insert".to_string())
        })
    }

    pub fn list_parent_feedback(
        &self,
        learner_id: i64,
        parent_id: Option<i64>,
        limit: i64,
    ) -> EcoachResult<Vec<ParentFeedbackRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, learner_id, parent_id, category, message, interpreted_signal, urgency,
                        suggested_support_action, visible_strategy_change, status, submitted_at, applied_at
                 FROM parent_feedback_records
                 WHERE learner_id = ?1
                   AND (?2 IS NULL OR parent_id = ?2)
                 ORDER BY submitted_at DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![learner_id, parent_id, limit.max(1)],
                map_parent_feedback_record,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.collect_rows(rows)
    }

    pub fn list_parent_alerts(
        &self,
        parent_id: i64,
        learner_id: Option<i64>,
        status: Option<&str>,
        limit: i64,
    ) -> EcoachResult<Vec<ParentAlertRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, learner_id, parent_id, trigger_type, severity, message,
                        action_required, status, metadata_json, created_at, acknowledged_at, resolved_at
                 FROM parent_alert_records
                 WHERE parent_id = ?1
                   AND (?2 IS NULL OR learner_id = ?2)
                   AND (?3 IS NULL OR status = ?3)
                 ORDER BY created_at DESC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![parent_id, learner_id, status, limit.max(1)],
                map_parent_alert_record,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.collect_rows(rows)
    }

    pub fn acknowledge_parent_alert(
        &self,
        alert_id: i64,
    ) -> EcoachResult<Option<ParentAlertRecord>> {
        self.conn
            .execute(
                "UPDATE parent_alert_records
                 SET status = 'acknowledged', acknowledged_at = datetime('now')
                 WHERE id = ?1",
                [alert_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let alert = self.get_parent_alert_by_id(alert_id)?;
        if let Some(alert) = alert.as_ref() {
            self.emit_runtime_event(
                "parent_alert_acknowledged",
                "learner",
                alert.learner_id.to_string(),
                json!({
                    "alertId": alert.id,
                    "parentId": alert.parent_id,
                    "triggerType": alert.trigger_type,
                }),
            )?;
        }
        Ok(alert)
    }

    pub fn list_strategy_adjustments(
        &self,
        learner_id: i64,
        limit: i64,
    ) -> EcoachResult<Vec<StrategyAdjustmentLog>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, learner_id, reason, source, old_strategy_snapshot_json,
                        new_strategy_snapshot_json, visible_message_student,
                        visible_message_parent, created_at
                 FROM strategy_adjustment_logs
                 WHERE learner_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![learner_id, limit.max(1)],
                map_strategy_adjustment_log,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.collect_rows(rows)
    }

    pub fn refresh_titles_hall(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<TitlesHallSnapshot> {
        let anchor_date = self.resolve_anchor_date(anchor_date)?;
        self.refresh_title_states(student_id, &anchor_date)?;
        self.refresh_badges(student_id)?;
        self.load_titles_hall(student_id)
    }

    pub fn get_titles_hall(&self, student_id: i64) -> EcoachResult<TitlesHallSnapshot> {
        self.refresh_titles_hall(student_id, None)
    }

    pub fn list_title_history(
        &self,
        title_id: i64,
        limit: i64,
    ) -> EcoachResult<Vec<CoachTitleHistoryEntry>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, title_state_id, previous_state, new_state, reason, snapshot_json, created_at
                 FROM coach_title_history
                 WHERE title_state_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![title_id, limit.max(1)], map_title_history_entry)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.collect_rows(rows)
    }

    pub fn begin_title_defense(
        &self,
        student_id: i64,
        title_id: i64,
    ) -> EcoachResult<TitleDefenseBrief> {
        let title = self
            .get_title_by_id(title_id)?
            .ok_or_else(|| EcoachError::NotFound("Title not found".to_string()))?;
        if title.student_id != student_id {
            return Err(EcoachError::Validation(
                "Title does not belong to the requested student".to_string(),
            ));
        }
        let timed = title
            .evidence_snapshot
            .get("timed_successes")
            .and_then(|value| value.as_i64())
            .unwrap_or(0)
            == 0
            || matches!(title.state.as_str(), "defense_due" | "narrowly_defended");
        let components = if timed {
            vec![
                "recall".to_string(),
                "application".to_string(),
                "transfer".to_string(),
                "timed_challenge".to_string(),
            ]
        } else {
            vec![
                "recall".to_string(),
                "application".to_string(),
                "transfer".to_string(),
            ]
        };
        let reason_due = if let Some(next_due) = title.next_defense_due_at.as_deref() {
            format!("This title is due for defense on or before {}.", next_due)
        } else {
            "This title is being checked to confirm mastery is still stable.".to_string()
        };
        self.conn
            .execute(
                "INSERT INTO title_defense_runs (
                    title_state_id, student_id, composition_json, outcome, coach_note
                 ) VALUES (?1, ?2, ?3, 'pending', ?4)",
                params![
                    title.id,
                    student_id,
                    json!({
                        "components": components,
                        "timed": timed,
                    })
                    .to_string(),
                    title
                        .coach_note
                        .clone()
                        .unwrap_or_else(|| "Mastery is being re-verified.".to_string()),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let run_id = self.conn.last_insert_rowid();
        self.record_title_history(
            title.id,
            Some(title.state.as_str()),
            title.state.as_str(),
            "Title defense started",
            json!({ "run_id": run_id }),
        )?;
        self.emit_runtime_event(
            "title_defense_started",
            "coach_title",
            title.id.to_string(),
            json!({
                "titleId": title.id,
                "runId": run_id,
                "titleName": title.title_name,
                "components": components,
            }),
        )?;
        Ok(TitleDefenseBrief {
            run_id,
            title_id: title.id,
            title_name: title.title_name,
            subject_name: title.subject_name,
            topic_name: title.topic_name,
            reason_due,
            components,
            timed,
            coach_note: title.coach_note.unwrap_or_else(|| {
                "This defense checks recall, application, and transfer stability.".to_string()
            }),
        })
    }

    pub fn complete_title_defense(
        &self,
        run_id: i64,
        input: &TitleDefenseCompletionInput,
    ) -> EcoachResult<TitleDefenseResult> {
        let (title_id, student_id, composition_json, current_state, title_name) = self
            .conn
            .query_row(
                "SELECT t.id,
                        t.student_id,
                        r.composition_json,
                        t.state,
                        t.title_name
                 FROM title_defense_runs r
                 INNER JOIN coach_title_states t ON t.id = r.title_state_id
                 WHERE r.id = ?1",
                [run_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let old_snapshot = self.capture_strategy_snapshot(student_id)?;
        let composition = parse_json_or_default(&composition_json);
        let timed_required = composition
            .get("timed")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let mut passed_components = 0;
        let mut total_components = 3;
        if input.recall_passed {
            passed_components += 1;
        }
        if input.application_passed {
            passed_components += 1;
        }
        if input.transfer_passed {
            passed_components += 1;
        }
        if timed_required {
            total_components += 1;
            if input.timed_challenge_passed.unwrap_or(false) {
                passed_components += 1;
            }
        }
        let catastrophic_break = !input.recall_passed || !input.application_passed;
        let ratio = passed_components as f64 / total_components as f64;
        let (outcome, new_state, next_defense_days, coach_note) = if !catastrophic_break
            && ratio >= 0.95
        {
            (
                "defended",
                "defended",
                14,
                "Mastery held across the full defense. The title remains secure.",
            )
        } else if !catastrophic_break && ratio >= 0.66 {
            (
                "narrowly_defended",
                "narrowly_defended",
                7,
                "The title stays active, but one component is fragile enough to need earlier re-checking.",
            )
        } else {
            (
                "contested",
                "contested",
                4,
                "The core knowledge is still present, but it is no longer stable enough to keep the title fully secure.",
            )
        };
        let next_defense_due_at = (Utc::now().date_naive() + Duration::days(next_defense_days))
            .format("%Y-%m-%d")
            .to_string();
        let reclaim_plan = if new_state == "contested" {
            Some(json!({
                "steps": [
                    "targeted_repair",
                    "term_contrast_review",
                    "transfer_check",
                    "reclaim_defense"
                ],
                "reason": input.triggered_misconception,
            }))
        } else {
            None
        };

        self.conn
            .execute(
                "UPDATE title_defense_runs
                 SET completed_at = datetime('now'),
                     outcome = ?2,
                     evaluation_json = ?3,
                     coach_note = ?4
                 WHERE id = ?1",
                params![
                    run_id,
                    outcome,
                    json!({
                        "recall_passed": input.recall_passed,
                        "application_passed": input.application_passed,
                        "transfer_passed": input.transfer_passed,
                        "timed_challenge_passed": input.timed_challenge_passed,
                        "confidence_held": input.confidence_held,
                        "accuracy_score_bp": input.accuracy_score_bp,
                        "notes": input.notes,
                        "triggered_misconception": input.triggered_misconception,
                    })
                    .to_string(),
                    format!(
                        "{}{}",
                        coach_note,
                        input
                            .notes
                            .as_ref()
                            .map(|notes| format!(" {}", notes))
                            .unwrap_or_default()
                    ),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE coach_title_states
                 SET state = ?2,
                     last_defended_at = datetime('now'),
                     next_defense_due_at = ?3,
                     coach_note = ?4,
                     reclaim_plan_json = ?5,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    title_id,
                    new_state,
                    next_defense_due_at,
                    coach_note,
                    reclaim_plan
                        .clone()
                        .unwrap_or_else(|| json!({}))
                        .to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.record_title_history(
            title_id,
            Some(current_state.as_str()),
            new_state,
            "Title defense completed",
            json!({
                "run_id": run_id,
                "outcome": outcome,
                "timed_required": timed_required,
                "timed_passed": input.timed_challenge_passed,
            }),
        )?;

        let (subject_id, topic_id) = self.title_subject_topic(title_id)?;
        let badge_awarded = if new_state == "contested" {
            self.schedule_title_follow_up_mission(title_id, student_id, "Title reclaim mission")?;
            None
        } else if new_state == "narrowly_defended" {
            self.schedule_title_follow_up_mission(
                title_id,
                student_id,
                "Short speed maintenance mission",
            )?;
            None
        } else if timed_required && input.timed_challenge_passed.unwrap_or(false) {
            self.award_badge_if_due(
                student_id,
                subject_id,
                topic_id,
                "Pressure Stabilizer",
                "performance",
                "Held accuracy under timed title defense pressure.",
                None,
                Some(title_id),
                json!({ "run_id": run_id }),
            )?
        } else {
            None
        };
        let new_snapshot = self.capture_strategy_snapshot(student_id)?;
        let strategy_adjustment_id = Some(self.record_strategy_adjustment(
            student_id,
            &format!("Title defense result for {}", title_name),
            "title",
            old_snapshot,
            new_snapshot,
            Some(match new_state {
                "defended" => format!("{} remains active after a full defense.", title_name),
                "narrowly_defended" => format!(
                    "{} remains active, but the coach added a short maintenance mission.",
                    title_name
                ),
                _ => format!(
                    "{} is now contested, so a reclaim mission has been scheduled.",
                    title_name
                ),
            }),
            None,
        )?);
        self.emit_runtime_event(
            "title_defense_completed",
            "coach_title",
            title_id.to_string(),
            json!({
                "titleId": title_id,
                "runId": run_id,
                "outcome": outcome,
                "newState": new_state,
                "strategyAdjustmentId": strategy_adjustment_id,
            }),
        )?;

        Ok(TitleDefenseResult {
            run_id,
            title_id,
            title_name,
            outcome: outcome.to_string(),
            new_state: new_state.to_string(),
            coach_note: coach_note.to_string(),
            next_defense_due_at: Some(next_defense_due_at),
            reclaim_plan,
            badge_awarded,
            strategy_adjustment_id,
        })
    }

    pub fn upsert_availability_profile(&self, profile: &AvailabilityProfile) -> EcoachResult<()> {
        validate_time_trigger_mode(Some(&profile.trigger_mode))?;
        self.conn
            .execute(
                "INSERT INTO availability_profiles (
                student_id, timezone_name, preferred_daily_minutes, ideal_session_minutes,
                min_session_minutes, max_session_minutes, split_sessions_allowed,
                max_split_sessions, min_break_minutes, trigger_mode,
                notification_lead_minutes, weekday_capacity_weight_bp,
                weekend_capacity_weight_bp, schedule_buffer_ratio_bp,
                fatigue_start_minute, fatigue_end_minute, thinking_idle_grace_seconds,
                idle_confirmation_seconds, abandonment_seconds
             ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12,
                ?13, ?14,
                ?15, ?16, ?17,
                ?18, ?19
             )
             ON CONFLICT(student_id) DO UPDATE SET
                timezone_name = excluded.timezone_name,
                preferred_daily_minutes = excluded.preferred_daily_minutes,
                ideal_session_minutes = excluded.ideal_session_minutes,
                min_session_minutes = excluded.min_session_minutes,
                max_session_minutes = excluded.max_session_minutes,
                split_sessions_allowed = excluded.split_sessions_allowed,
                max_split_sessions = excluded.max_split_sessions,
                min_break_minutes = excluded.min_break_minutes,
                trigger_mode = excluded.trigger_mode,
                notification_lead_minutes = excluded.notification_lead_minutes,
                weekday_capacity_weight_bp = excluded.weekday_capacity_weight_bp,
                weekend_capacity_weight_bp = excluded.weekend_capacity_weight_bp,
                schedule_buffer_ratio_bp = excluded.schedule_buffer_ratio_bp,
                fatigue_start_minute = excluded.fatigue_start_minute,
                fatigue_end_minute = excluded.fatigue_end_minute,
                thinking_idle_grace_seconds = excluded.thinking_idle_grace_seconds,
                idle_confirmation_seconds = excluded.idle_confirmation_seconds,
                abandonment_seconds = excluded.abandonment_seconds,
                updated_at = datetime('now')",
                params![
                    profile.student_id,
                    profile.timezone_name,
                    profile.preferred_daily_minutes,
                    profile.ideal_session_minutes,
                    profile.min_session_minutes,
                    profile.max_session_minutes,
                    if profile.split_sessions_allowed { 1 } else { 0 },
                    profile.max_split_sessions,
                    profile.min_break_minutes,
                    profile.trigger_mode,
                    profile.notification_lead_minutes,
                    profile.weekday_capacity_weight_bp,
                    profile.weekend_capacity_weight_bp,
                    profile.schedule_buffer_ratio_bp,
                    profile.fatigue_start_minute,
                    profile.fatigue_end_minute,
                    profile.thinking_idle_grace_seconds,
                    profile.idle_confirmation_seconds,
                    profile.abandonment_seconds,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn replace_availability_windows(
        &self,
        student_id: i64,
        windows: &[AvailabilityWindow],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM availability_windows WHERE student_id = ?1",
                [student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for window in windows {
            self.conn
                .execute(
                    "INSERT INTO availability_windows (
                    student_id, weekday, start_minute, end_minute, is_preferred
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        student_id,
                        window.weekday,
                        window.start_minute,
                        window.end_minute,
                        if window.is_preferred { 1 } else { 0 },
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    pub fn add_availability_exception(
        &self,
        student_id: i64,
        exception: &AvailabilityException,
    ) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO availability_exceptions (
                student_id, exception_date, start_minute, end_minute, availability_mode, minutes_delta, reason
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                student_id,
                exception.exception_date,
                exception.start_minute,
                exception.end_minute,
                exception.availability_mode,
                exception.minutes_delta,
                exception.reason,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_availability_profile(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<AvailabilityProfile>> {
        self.load_availability_profile(student_id)
    }

    pub fn list_availability_windows(
        &self,
        student_id: i64,
    ) -> EcoachResult<Vec<AvailabilityWindow>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT weekday, start_minute, end_minute, COALESCE(is_preferred, 0)
                 FROM availability_windows
                 WHERE student_id = ?1
                 ORDER BY weekday ASC, start_minute ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok(AvailabilityWindow {
                    weekday: row.get(0)?,
                    start_minute: row.get(1)?,
                    end_minute: row.get(2)?,
                    is_preferred: row.get::<_, i64>(3)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut windows = Vec::new();
        for row in rows {
            windows.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(windows)
    }

    pub fn list_availability_exceptions(
        &self,
        student_id: i64,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> EcoachResult<Vec<AvailabilityException>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT exception_date, start_minute, end_minute, availability_mode, minutes_delta, reason
                 FROM availability_exceptions
                 WHERE student_id = ?1
                   AND (?2 IS NULL OR exception_date >= ?2)
                   AND (?3 IS NULL OR exception_date <= ?3)
                 ORDER BY exception_date ASC, COALESCE(start_minute, 0) ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, start_date, end_date], |row| {
                Ok(AvailabilityException {
                    exception_date: row.get(0)?,
                    start_minute: row.get(1)?,
                    end_minute: row.get(2)?,
                    availability_mode: row.get(3)?,
                    minutes_delta: row.get(4)?,
                    reason: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut exceptions = Vec::new();
        for row in rows {
            exceptions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(exceptions)
    }

    pub fn sync_exam_plan_state(&self, input: &ExamPlanStateInput) -> EcoachResult<ExamPlanState> {
        validate_time_plan_mode(input.plan_mode.as_deref())?;
        validate_time_trigger_mode(input.auto_trigger_mode.as_deref())?;
        let profile = self.load_availability_profile_or_default(input.student_id)?;
        let available_study_days = self.count_available_study_days(
            input.student_id,
            &input.anchor_date,
            &input.exam_date,
        )?;
        let total_available_minutes = self.total_available_minutes_between(
            input.student_id,
            &input.anchor_date,
            &input.exam_date,
        )?;
        let completed_effective_minutes =
            self.load_effective_minutes_completed(input.student_id, input.subject_id)?;
        let (buffer_consumed_minutes, missed_debt_minutes, bonus_credit_minutes) =
            self.load_schedule_carryover(input.student_id, input.subject_id)?;
        let target_effective_minutes = input.target_effective_minutes.max(0);
        let protected_buffer_minutes = total_available_minutes
            .saturating_mul(profile.schedule_buffer_ratio_bp as i64)
            / 10_000;
        let raw_remaining_minutes = target_effective_minutes - completed_effective_minutes
            + missed_debt_minutes
            - bonus_credit_minutes;
        let remaining_effective_minutes = raw_remaining_minutes.max(0);
        let weeks_remaining = (((available_study_days.max(1) as f64) / 7.0).ceil() as i64).max(1);
        let required_weekly_minutes = if remaining_effective_minutes == 0 {
            0
        } else {
            ((remaining_effective_minutes as f64) / weeks_remaining as f64).ceil() as i64
        };
        let usable_capacity_minutes = (total_available_minutes - protected_buffer_minutes).max(0);
        let feasibility_score_bp = if remaining_effective_minutes == 0 {
            10_000
        } else {
            clamp_bp((usable_capacity_minutes * 10_000) / remaining_effective_minutes.max(1))
        };
        let pressure_score_bp =
            clamp_bp(((remaining_effective_minutes * 10_000) / usable_capacity_minutes.max(30)));
        let schedule_truth_score_bp =
            clamp_bp(((feasibility_score_bp as i64 + (10_000 - pressure_score_bp as i64)) / 2));
        let days_to_exam = days_between_inclusive(&input.anchor_date, &input.exam_date)?;
        let plan_mode = input.plan_mode.clone().unwrap_or_else(|| {
            derive_time_plan_mode(days_to_exam, pressure_score_bp, feasibility_score_bp)
        });
        let auto_trigger_mode = input
            .auto_trigger_mode
            .clone()
            .unwrap_or_else(|| profile.trigger_mode.clone());
        let explanation = json!({
            "available_study_days": available_study_days,
            "total_available_minutes": total_available_minutes,
            "usable_capacity_minutes": usable_capacity_minutes,
            "protected_buffer_minutes": protected_buffer_minutes,
            "completed_effective_minutes": completed_effective_minutes,
            "remaining_effective_minutes": remaining_effective_minutes,
            "missed_debt_minutes": missed_debt_minutes,
            "bonus_credit_minutes": bonus_credit_minutes,
            "required_weekly_minutes": required_weekly_minutes,
            "days_to_exam": days_to_exam,
            "trigger_mode": auto_trigger_mode,
            "plan_mode": plan_mode,
        });
        self.conn
            .execute(
                "INSERT INTO exam_plan_states (
                    student_id, subject_id, anchor_date, exam_date, target_effective_minutes,
                    completed_effective_minutes, remaining_effective_minutes, available_study_days,
                    required_weekly_minutes, protected_buffer_minutes, buffer_consumed_minutes,
                    missed_debt_minutes, bonus_credit_minutes, pressure_score_bp,
                    feasibility_score_bp, schedule_truth_score_bp, plan_mode, auto_trigger_mode,
                    explanation_json
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8,
                    ?9, ?10, ?11,
                    ?12, ?13, ?14,
                    ?15, ?16, ?17, ?18,
                    ?19
                 )
                 ON CONFLICT(student_id, subject_id, exam_date) DO UPDATE SET
                    anchor_date = excluded.anchor_date,
                    target_effective_minutes = excluded.target_effective_minutes,
                    completed_effective_minutes = excluded.completed_effective_minutes,
                    remaining_effective_minutes = excluded.remaining_effective_minutes,
                    available_study_days = excluded.available_study_days,
                    required_weekly_minutes = excluded.required_weekly_minutes,
                    protected_buffer_minutes = excluded.protected_buffer_minutes,
                    buffer_consumed_minutes = excluded.buffer_consumed_minutes,
                    missed_debt_minutes = excluded.missed_debt_minutes,
                    bonus_credit_minutes = excluded.bonus_credit_minutes,
                    pressure_score_bp = excluded.pressure_score_bp,
                    feasibility_score_bp = excluded.feasibility_score_bp,
                    schedule_truth_score_bp = excluded.schedule_truth_score_bp,
                    plan_mode = excluded.plan_mode,
                    auto_trigger_mode = excluded.auto_trigger_mode,
                    explanation_json = excluded.explanation_json,
                    updated_at = datetime('now')",
                params![
                    input.student_id,
                    input.subject_id,
                    input.anchor_date,
                    input.exam_date,
                    target_effective_minutes,
                    completed_effective_minutes,
                    remaining_effective_minutes,
                    available_study_days,
                    required_weekly_minutes,
                    protected_buffer_minutes,
                    buffer_consumed_minutes,
                    missed_debt_minutes,
                    bonus_credit_minutes,
                    pressure_score_bp,
                    feasibility_score_bp,
                    schedule_truth_score_bp,
                    plan_mode,
                    auto_trigger_mode,
                    explanation.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_exam_plan_state(input.student_id, input.subject_id, &input.exam_date)?
            .ok_or_else(|| EcoachError::NotFound("exam plan state was not created".to_string()))
    }

    pub fn get_exam_plan_state(
        &self,
        student_id: i64,
        subject_id: i64,
        exam_date: &str,
    ) -> EcoachResult<Option<ExamPlanState>> {
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id, anchor_date, exam_date,
                        target_effective_minutes, completed_effective_minutes,
                        remaining_effective_minutes, available_study_days,
                        required_weekly_minutes, protected_buffer_minutes,
                        buffer_consumed_minutes, missed_debt_minutes, bonus_credit_minutes,
                        pressure_score_bp, feasibility_score_bp, schedule_truth_score_bp,
                        plan_mode, auto_trigger_mode, explanation_json, updated_at
                 FROM exam_plan_states
                 WHERE student_id = ?1 AND subject_id = ?2 AND exam_date = ?3",
                params![student_id, subject_id, exam_date],
                map_exam_plan_state,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_schedule_ledger(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
    ) -> EcoachResult<ScheduleLedgerEntry> {
        self.sync_schedule_ledger_entry(student_id, subject_id, date)
    }

    pub fn list_time_session_blocks(
        &self,
        student_id: i64,
        subject_id: i64,
        from_date: Option<&str>,
        limit: usize,
    ) -> EcoachResult<Vec<TimeSessionBlock>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, subject_id, plan_state_id, block_date, start_minute,
                        end_minute, target_minutes, session_type, objective_summary,
                        focus_topic_ids_json, trigger_mode, fit_score_bp, priority_score_bp,
                        status, fallback_session_type, replacement_options_json, created_by,
                        source_kind, explanation_text, linked_session_id
                 FROM time_session_blocks
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND (?3 IS NULL OR block_date >= ?3)
                 ORDER BY block_date ASC, COALESCE(start_minute, 0) ASC, id ASC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, subject_id, from_date, limit.max(1) as i64],
                map_time_session_block,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut blocks = Vec::new();
        for row in rows {
            blocks.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(blocks)
    }

    pub fn dispatch_due_trigger_jobs(
        &self,
        as_of: &str,
        student_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<ScheduleTriggerJob>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, subject_id, session_block_id, trigger_kind, scheduled_for,
                        lead_minutes, status, payload_json
                 FROM schedule_trigger_jobs
                 WHERE status = 'scheduled'
                   AND scheduled_for <= ?1
                   AND (?2 IS NULL OR student_id = ?2)
                 ORDER BY scheduled_for ASC, id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![as_of, student_id, limit.max(1) as i64],
                map_schedule_trigger_job,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut jobs = Vec::new();
        for row in rows {
            let job = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn
                .execute(
                    "UPDATE schedule_trigger_jobs
                     SET status = 'fired', updated_at = datetime('now')
                     WHERE id = ?1",
                    [job.id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            jobs.push(ScheduleTriggerJob {
                status: "fired".to_string(),
                ..job
            });
        }
        Ok(jobs)
    }

    pub fn build_time_orchestration_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
        anchor_date: &str,
        minute_of_day: i64,
        exam_date: Option<&str>,
        target_effective_minutes: Option<i64>,
        plan_mode: Option<&str>,
        auto_trigger_mode: Option<&str>,
        horizon_days: usize,
    ) -> EcoachResult<TimeOrchestrationSnapshot> {
        let availability_profile = self.load_availability_profile_or_default(student_id)?;
        let plan_state = if let Some(existing) =
            self.latest_exam_plan_state(student_id, subject_id, exam_date)?
        {
            existing
        } else {
            let exam_date = exam_date.ok_or_else(|| {
                EcoachError::Validation(
                    "exam_date is required when no exam plan state exists".to_string(),
                )
            })?;
            let target_effective_minutes = target_effective_minutes.ok_or_else(|| {
                EcoachError::Validation(
                    "target_effective_minutes is required when no exam plan state exists"
                        .to_string(),
                )
            })?;
            self.sync_exam_plan_state(&ExamPlanStateInput {
                student_id,
                subject_id,
                anchor_date: anchor_date.to_string(),
                exam_date: exam_date.to_string(),
                target_effective_minutes,
                plan_mode: plan_mode.map(str::to_string),
                auto_trigger_mode: auto_trigger_mode.map(str::to_string),
            })?
        };
        let daily_availability = self.get_daily_availability(student_id, anchor_date)?;
        let free_now = self.recommend_free_now_session(
            student_id,
            subject_id,
            anchor_date,
            minute_of_day,
            daily_availability.adjusted_minutes.max(0),
        )?;
        let replan =
            self.replan_remaining_day(student_id, subject_id, anchor_date, minute_of_day)?;
        let ledger = self.sync_schedule_ledger_entry(student_id, subject_id, anchor_date)?;
        let session_blocks = self.rebuild_time_session_blocks(
            student_id,
            subject_id,
            &plan_state,
            anchor_date,
            horizon_days.max(1),
            &availability_profile,
            &replan,
        )?;
        let trigger_jobs = self.upsert_trigger_jobs(
            student_id,
            subject_id,
            &availability_profile,
            &session_blocks,
        )?;
        let explanation = format!(
            "{} minute(s) remain against the current plan, {} window(s) are active in the next horizon, and the coach is using {} trigger behavior.",
            plan_state.remaining_effective_minutes,
            session_blocks.len(),
            plan_state.auto_trigger_mode,
        );
        Ok(TimeOrchestrationSnapshot {
            student_id,
            subject_id,
            anchor_date: anchor_date.to_string(),
            minute_of_day,
            availability_profile,
            daily_availability,
            plan_state,
            ledger,
            free_now,
            replan,
            session_blocks,
            trigger_jobs,
            explanation,
        })
    }

    pub fn get_daily_availability(
        &self,
        student_id: i64,
        date: &str,
    ) -> EcoachResult<DailyAvailabilitySummary> {
        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let weekday = target_date.weekday().num_days_from_monday() as i64;
        let base_minutes = self.base_minutes_for_day(student_id, weekday)?;
        let mut adjusted_minutes = base_minutes;
        let mut blocked = false;
        let mut reason = None;

        let mut statement = self
            .conn
            .prepare(
                "SELECT availability_mode, minutes_delta, reason
             FROM availability_exceptions
             WHERE student_id = ?1 AND exception_date = ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, date], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for row in rows {
            let (mode, minutes_delta, exception_reason) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            match mode.as_str() {
                "blocked" => {
                    blocked = true;
                    adjusted_minutes = 0;
                    reason = exception_reason;
                }
                "reduced" => {
                    adjusted_minutes = (adjusted_minutes - minutes_delta.abs()).max(0);
                    if reason.is_none() {
                        reason = exception_reason;
                    }
                }
                "extended" => {
                    adjusted_minutes += minutes_delta.abs();
                    if reason.is_none() {
                        reason = exception_reason;
                    }
                }
                _ => {}
            }
        }

        Ok(DailyAvailabilitySummary {
            date: date.to_string(),
            base_minutes,
            adjusted_minutes,
            blocked,
            reason,
        })
    }

    pub fn is_free_now(
        &self,
        student_id: i64,
        date: &str,
        minute_of_day: i64,
    ) -> EcoachResult<bool> {
        let summary = self.get_daily_availability(student_id, date)?;
        if summary.blocked || summary.adjusted_minutes <= 0 {
            return Ok(false);
        }

        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let weekday = target_date.weekday().num_days_from_monday() as i64;
        let mut statement = self
            .conn
            .prepare(
                "SELECT start_minute, end_minute
             FROM availability_windows
             WHERE student_id = ?1 AND weekday = ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, weekday], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for row in rows {
            let (start_minute, end_minute) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if minute_of_day >= start_minute && minute_of_day < end_minute {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn recommend_free_now_session(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
        minute_of_day: i64,
        available_minutes: i64,
    ) -> EcoachResult<FreeNowRecommendation> {
        let available_now = self.is_free_now(student_id, date, minute_of_day)?;
        let window_end_minute = self.current_window_end_minute(student_id, date, minute_of_day)?;
        let base_focus_topic_ids = self.load_focus_topic_ids(student_id, subject_id, 3)?;
        let target = self.get_daily_climb_target(student_id, subject_id, date)?;
        let actual = self.load_actual_performance(student_id, subject_id, date)?;
        let active_mission = self.load_active_mission(student_id, subject_id, date)?;
        let comeback_pressure = self.build_comeback_pressure(student_id, subject_id, date, 3)?;

        let carryover_attempts = target
            .as_ref()
            .map(|item| (item.target_attempts - actual.attempts).max(0))
            .unwrap_or(0);
        let carryover_correct = target
            .as_ref()
            .map(|item| (item.target_correct - actual.correct).max(0))
            .unwrap_or(0);

        if !available_now {
            return Ok(FreeNowRecommendation {
                date: date.to_string(),
                minute_of_day,
                available_now: false,
                window_end_minute,
                suggested_duration_minutes: 0,
                session_type: "wait_for_window".to_string(),
                rationale: "The student is currently outside an active study window.".to_string(),
                focus_topic_ids: merge_topic_ids(
                    &comeback_pressure.focus_topic_ids,
                    &base_focus_topic_ids,
                    3,
                ),
                target_id: target.as_ref().map(|item| item.id),
                carryover_attempts,
                carryover_correct,
                pressure_score: comeback_pressure.pressure_score,
                repair_buffer_minutes: comeback_pressure.repair_buffer_minutes,
                recommended_comeback_topic_id: comeback_pressure.recommended_topic_id,
                recent_repair_outcome: comeback_pressure.recent_repair_outcome,
            });
        }

        let (min_session_minutes, max_session_minutes) = self.load_session_bounds(student_id)?;
        let remaining_window_minutes = window_end_minute
            .map(|end_minute| (end_minute - minute_of_day).max(0))
            .unwrap_or(available_minutes.max(0));
        let raw_capacity = available_minutes.min(remaining_window_minutes).max(0);
        let suggested_duration_minutes = if raw_capacity <= 0 {
            0
        } else if raw_capacity < min_session_minutes {
            raw_capacity.max(5)
        } else {
            raw_capacity.min(max_session_minutes)
        };
        let remaining_target_minutes = target
            .as_ref()
            .map(|item| estimate_remaining_target_minutes(item, &actual))
            .unwrap_or(0);
        let comeback_focus_topic_ids =
            merge_topic_ids(&comeback_pressure.focus_topic_ids, &base_focus_topic_ids, 3);
        let should_preempt_target = match comeback_pressure.recommended_session_type.as_deref() {
            Some("comeback_reteach" | "comeback_repair" | "guided_reinforcement") => true,
            Some("memory_rescue") => {
                carryover_attempts <= 2 || remaining_target_minutes <= min_session_minutes
            }
            Some("retention_check") => {
                target.is_none()
                    || carryover_attempts <= 2
                    || remaining_target_minutes <= min_session_minutes
            }
            _ => false,
        };

        let (session_type, rationale, focus_topic_ids) = if let Some(mission) = active_mission {
            let mut mission_focus = comeback_focus_topic_ids.clone();
            if let Some(topic_id) = mission.primary_topic_id {
                mission_focus.retain(|value| *value != topic_id);
                mission_focus.insert(0, topic_id);
            }
            let rationale = if mission.primary_topic_id == comeback_pressure.recommended_topic_id
                && comeback_pressure
                    .recent_repair_outcome
                    .as_deref()
                    .is_some_and(|value| value == "failed")
            {
                format!(
                    "A planned coach mission is already waiting for this window, and it lines up with the latest failed repair comeback on the same topic: {}.",
                    mission.title
                )
            } else {
                format!(
                    "A planned coach mission is already waiting for this window: {}.",
                    mission.title
                )
            };
            (
                format!("planned_{}", mission.activity_type),
                rationale,
                mission_focus,
            )
        } else if should_preempt_target {
            (
                comeback_pressure
                    .recommended_session_type
                    .clone()
                    .unwrap_or_else(|| "memory_rescue".to_string()),
                comeback_pressure
                    .rationale
                    .clone()
                    .unwrap_or_else(|| {
                        "Recent repair pressure should take precedence over the default free-now plan.".to_string()
                    }),
                comeback_focus_topic_ids.clone(),
            )
        } else if let Some(target) = target.as_ref() {
            let session_type = free_now_session_type_for_mode(&target.mode);
            let mut rationale = format!(
                "Use this free window to advance today's {} target while {} attempts and {} correct answers still remain.",
                target.mode, carryover_attempts, carryover_correct
            );
            if let Some(extra_rationale) = comeback_pressure.rationale.as_ref() {
                rationale.push(' ');
                rationale.push_str(extra_rationale);
            }
            (
                session_type.to_string(),
                rationale,
                merge_topic_ids(&target.focus_topic_ids, &comeback_focus_topic_ids, 3),
            )
        } else if let Some(session_type) = comeback_pressure.recommended_session_type.clone() {
            let rationale = comeback_pressure.rationale.clone().unwrap_or_else(|| {
                "The best use of this window is to absorb the latest repair pressure.".to_string()
            });
            (session_type, rationale, comeback_focus_topic_ids.clone())
        } else if self.count_due_memory_topics(student_id, subject_id, date)? > 0 {
            (
                "memory_rescue".to_string(),
                "A spaced review is due, so the best use of this window is retrieval rescue."
                    .to_string(),
                comeback_focus_topic_ids.clone(),
            )
        } else {
            (
                "bonus_priority_push".to_string(),
                "No hard target is due right now, so this window can safely pull forward a high-priority topic."
                    .to_string(),
                comeback_focus_topic_ids.clone(),
            )
        };

        let lower_floor = if raw_capacity <= 0 {
            0
        } else if raw_capacity < min_session_minutes {
            raw_capacity.max(5)
        } else {
            min_session_minutes
        };
        let suggested_duration_minutes = match session_type.as_str() {
            "comeback_reteach" | "comeback_repair" | "guided_reinforcement" | "memory_rescue" => {
                suggested_duration_minutes.max(
                    comeback_pressure
                        .repair_buffer_minutes
                        .min(raw_capacity)
                        .min(max_session_minutes)
                        .max(lower_floor),
                )
            }
            "retention_check" => {
                let retention_floor = if raw_capacity < min_session_minutes {
                    raw_capacity.max(5)
                } else {
                    lower_floor
                };
                suggested_duration_minutes.min(
                    comeback_pressure
                        .repair_buffer_minutes
                        .max(8)
                        .min(raw_capacity)
                        .min(max_session_minutes)
                        .max(retention_floor),
                )
            }
            _ => suggested_duration_minutes,
        };

        Ok(FreeNowRecommendation {
            date: date.to_string(),
            minute_of_day,
            available_now: true,
            window_end_minute,
            suggested_duration_minutes,
            session_type,
            rationale,
            focus_topic_ids,
            target_id: target.as_ref().map(|item| item.id),
            carryover_attempts,
            carryover_correct,
            pressure_score: comeback_pressure.pressure_score,
            repair_buffer_minutes: comeback_pressure.repair_buffer_minutes,
            recommended_comeback_topic_id: comeback_pressure.recommended_topic_id,
            recent_repair_outcome: comeback_pressure.recent_repair_outcome,
        })
    }

    pub fn replan_remaining_day(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
        minute_of_day: i64,
    ) -> EcoachResult<DailyReplan> {
        let available_now = self.is_free_now(student_id, date, minute_of_day)?;
        let remaining_capacity_minutes =
            self.remaining_day_capacity_minutes(student_id, date, minute_of_day)?;
        let target = self.get_daily_climb_target(student_id, subject_id, date)?;
        let actual = self.load_actual_performance(student_id, subject_id, date)?;
        let base_focus_topic_ids = target
            .as_ref()
            .map(|item| item.focus_topic_ids.clone())
            .unwrap_or(self.load_focus_topic_ids(student_id, subject_id, 3)?);
        let base_remaining_target_minutes = target
            .as_ref()
            .map(|item| estimate_remaining_target_minutes(item, &actual))
            .unwrap_or(0);
        let comeback_pressure = self.build_comeback_pressure(student_id, subject_id, date, 3)?;
        let should_preempt_target = match comeback_pressure.recommended_session_type.as_deref() {
            Some(
                "comeback_reteach" | "comeback_repair" | "guided_reinforcement" | "memory_rescue",
            ) => true,
            Some("retention_check") => target.is_none() || base_remaining_target_minutes <= 10,
            _ => false,
        };
        let repair_buffer_minutes = if should_preempt_target
            || target.is_none()
            || comeback_pressure
                .recommended_session_type
                .as_deref()
                .is_some_and(|value| value == "retention_check")
        {
            comeback_pressure.repair_buffer_minutes
        } else {
            0
        };
        let focus_topic_ids = if should_preempt_target {
            merge_topic_ids(&comeback_pressure.focus_topic_ids, &base_focus_topic_ids, 3)
        } else {
            merge_topic_ids(&base_focus_topic_ids, &comeback_pressure.focus_topic_ids, 3)
        };
        let remaining_target_minutes = base_remaining_target_minutes + repair_buffer_minutes;
        let (_, max_session_minutes) = self.load_session_bounds(student_id)?;
        let recommended_session_count = if remaining_target_minutes == 0 {
            0
        } else {
            ((remaining_target_minutes as f64) / max_session_minutes.max(1) as f64).ceil() as i64
        };
        let next_session_type = if should_preempt_target {
            comeback_pressure
                .recommended_session_type
                .clone()
                .unwrap_or_else(|| "memory_rescue".to_string())
        } else if target.is_none()
            && comeback_pressure
                .recommended_session_type
                .as_deref()
                .is_some_and(|value| value == "retention_check")
        {
            "retention_check".to_string()
        } else {
            target
                .as_ref()
                .map(|item| free_now_session_type_for_mode(&item.mode).to_string())
                .unwrap_or_else(|| "bonus_priority_push".to_string())
        };
        let rationale = if should_preempt_target {
            comeback_pressure.rationale.clone().unwrap_or_else(|| {
                "Recent repair pressure should take precedence in the rest-of-day plan.".to_string()
            })
        } else if remaining_target_minutes == 0 {
            "Today's scheduled learning load is effectively complete, so any new work is a bonus pull-forward."
                .to_string()
        } else if repair_buffer_minutes > 0 {
            format!(
                "The original target load is still manageable, but {} extra comeback minute(s) should be reserved because the latest memory or solidification signal still needs follow-through.",
                repair_buffer_minutes
            )
        } else if remaining_capacity_minutes < remaining_target_minutes {
            format!(
                "Today's remaining capacity is tighter than the remaining target load, so the coach should compress the rest of the day into {} focused block(s).",
                recommended_session_count.max(1)
            )
        } else {
            format!(
                "There is still enough room today to finish the remaining target load in {} focused block(s).",
                recommended_session_count.max(1)
            )
        };

        Ok(DailyReplan {
            date: date.to_string(),
            available_now,
            remaining_capacity_minutes,
            remaining_target_minutes,
            recommended_session_count,
            next_session_type,
            focus_topic_ids,
            target_id: target.as_ref().map(|item| item.id),
            rationale,
            pressure_score: comeback_pressure.pressure_score,
            repair_buffer_minutes,
            recommended_comeback_topic_id: comeback_pressure.recommended_topic_id,
            recent_repair_outcome: comeback_pressure.recent_repair_outcome,
        })
    }

    pub fn generate_daily_climb_target(
        &self,
        student_id: i64,
        subject_id: i64,
        target_date: &str,
    ) -> EcoachResult<BeatYesterdayDailyTarget> {
        let baseline =
            self.load_recent_performance_baseline(student_id, subject_id, target_date)?;
        let readiness_score = self.load_subject_readiness(student_id, subject_id)?;
        let focus_topic_ids = self.load_focus_topic_ids(student_id, subject_id, 3)?;
        let stage = beat_stage_from_readiness(readiness_score);
        let mode = beat_mode_from_baseline(&baseline);

        let target_attempts = match mode {
            "volume_push" => (baseline.attempts + 2).max(6),
            "accuracy_repair" => baseline.attempts.max(6),
            "speed_lift" => (baseline.attempts + 1).max(6),
            "recovery_mode" => baseline.attempts.clamp(4, 8),
            _ => baseline.attempts.max(6),
        };
        let target_correct = match mode {
            "accuracy_repair" => (baseline.correct + 1).min(target_attempts),
            "recovery_mode" => ((target_attempts as f64) * 0.60).ceil() as i64,
            _ => (baseline.correct + 1).min(target_attempts),
        };
        let target_avg_response_time_ms = match mode {
            "speed_lift" => baseline
                .avg_response_time_ms
                .map(|value| (value - 2_000).max(12_000)),
            "recovery_mode" => baseline.avg_response_time_ms.map(|value| value + 2_000),
            _ => baseline.avg_response_time_ms,
        };

        let (warm_start_minutes, core_climb_minutes, speed_burst_minutes, finish_strong_minutes) =
            match mode {
                "recovery_mode" => (3, 4, 0, 2),
                "accuracy_repair" => (2, 6, 1, 1),
                "speed_lift" => (2, 4, 2, 1),
                _ => (2, 5, 1, 1),
            };

        let rationale = json!({
            "baseline_attempts": baseline.attempts,
            "baseline_correct": baseline.correct,
            "baseline_accuracy_score": baseline.accuracy_score,
            "baseline_avg_response_time_ms": baseline.avg_response_time_ms,
            "readiness_score": readiness_score,
            "focus_topic_ids": focus_topic_ids,
            "mode_reason": beat_mode_reason(&mode),
        });
        let focus_topic_ids_json = serde_json::to_string(&focus_topic_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let rationale_json = serde_json::to_string(&rationale)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn.execute(
            "INSERT INTO beat_yesterday_daily_targets (
                student_id, subject_id, target_date, stage, mode, target_attempts, target_correct,
                target_avg_response_time_ms, warm_start_minutes, core_climb_minutes,
                speed_burst_minutes, finish_strong_minutes, focus_topic_ids_json, rationale_json, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'planned')
             ON CONFLICT(student_id, subject_id, target_date) DO UPDATE SET
                stage = excluded.stage,
                mode = excluded.mode,
                target_attempts = excluded.target_attempts,
                target_correct = excluded.target_correct,
                target_avg_response_time_ms = excluded.target_avg_response_time_ms,
                warm_start_minutes = excluded.warm_start_minutes,
                core_climb_minutes = excluded.core_climb_minutes,
                speed_burst_minutes = excluded.speed_burst_minutes,
                finish_strong_minutes = excluded.finish_strong_minutes,
                focus_topic_ids_json = excluded.focus_topic_ids_json,
                rationale_json = excluded.rationale_json,
                updated_at = datetime('now')",
            params![
                student_id,
                subject_id,
                target_date,
                stage,
                mode,
                target_attempts,
                target_correct,
                target_avg_response_time_ms,
                warm_start_minutes,
                core_climb_minutes,
                speed_burst_minutes,
                finish_strong_minutes,
                focus_topic_ids_json,
                rationale_json,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.upsert_beat_yesterday_profile(
            student_id,
            subject_id,
            stage,
            mode,
            5000,
            baseline.strain_score,
            readiness_score,
        )?;
        self.get_daily_climb_target(student_id, subject_id, target_date)?
            .ok_or_else(|| EcoachError::NotFound("daily climb target was not created".to_string()))
    }

    pub fn complete_daily_climb(&self, target_id: i64) -> EcoachResult<BeatYesterdayDailySummary> {
        let target = self
            .get_daily_climb_target_by_id(target_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("beat yesterday target {} not found", target_id))
            })?;
        let actual = self.load_actual_performance(
            target.student_id,
            target.subject_id,
            &target.target_date,
        )?;
        let baseline = self.load_recent_performance_baseline(
            target.student_id,
            target.subject_id,
            &target.target_date,
        )?;

        let beat_attempt_target = actual.attempts >= target.target_attempts;
        let beat_accuracy_target = actual.correct >= target.target_correct;
        let beat_pace_target = match (
            actual.avg_response_time_ms,
            target.target_avg_response_time_ms,
        ) {
            (Some(actual_ms), Some(target_ms)) => actual_ms <= target_ms,
            (Some(_), None) => true,
            _ => false,
        };

        let momentum_score = beat_momentum_score(
            baseline.attempts,
            baseline.accuracy_score,
            baseline.avg_response_time_ms,
            actual.attempts,
            actual.accuracy_score,
            actual.avg_response_time_ms,
        );
        let strain_score = beat_strain_score(
            beat_attempt_target,
            beat_accuracy_target,
            beat_pace_target,
            baseline.accuracy_score,
            actual.accuracy_score,
            actual.avg_response_time_ms,
        );
        let recovery_mode_triggered =
            strain_score >= 7000 || (momentum_score < 4500 && !beat_accuracy_target);
        let beats_total = [beat_attempt_target, beat_accuracy_target, beat_pace_target]
            .into_iter()
            .filter(|flag| *flag)
            .count();
        let summary = json!({
            "beat_attempt_target": beat_attempt_target,
            "beat_accuracy_target": beat_accuracy_target,
            "beat_pace_target": beat_pace_target,
            "beats_total": beats_total,
            "mode": target.mode,
            "stage": target.stage,
            "focus_topic_ids": target.focus_topic_ids,
        });

        self.conn.execute(
            "INSERT INTO beat_yesterday_daily_summaries (
                target_id, student_id, subject_id, summary_date, actual_attempts, actual_correct,
                actual_avg_response_time_ms, beat_attempt_target, beat_accuracy_target, beat_pace_target,
                momentum_score, strain_score, recovery_mode_triggered, summary_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(student_id, subject_id, summary_date) DO UPDATE SET
                target_id = excluded.target_id,
                actual_attempts = excluded.actual_attempts,
                actual_correct = excluded.actual_correct,
                actual_avg_response_time_ms = excluded.actual_avg_response_time_ms,
                beat_attempt_target = excluded.beat_attempt_target,
                beat_accuracy_target = excluded.beat_accuracy_target,
                beat_pace_target = excluded.beat_pace_target,
                momentum_score = excluded.momentum_score,
                strain_score = excluded.strain_score,
                recovery_mode_triggered = excluded.recovery_mode_triggered,
                summary_json = excluded.summary_json",
            params![
                target_id,
                target.student_id,
                target.subject_id,
                target.target_date,
                actual.attempts,
                actual.correct,
                actual.avg_response_time_ms,
                bool_to_i64(beat_attempt_target),
                bool_to_i64(beat_accuracy_target),
                bool_to_i64(beat_pace_target),
                momentum_score,
                strain_score,
                bool_to_i64(recovery_mode_triggered),
                serde_json::to_string(&summary).map_err(|err| EcoachError::Serialization(err.to_string()))?,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE beat_yesterday_daily_targets
             SET status = 'completed', updated_at = datetime('now')
             WHERE id = ?1",
                [target_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let streak_days = self.compute_beat_streak(target.student_id, target.subject_id)?;
        let current_mode = if recovery_mode_triggered {
            "recovery_mode"
        } else {
            target.mode.as_str()
        };
        self.upsert_beat_yesterday_profile(
            target.student_id,
            target.subject_id,
            target.stage.as_str(),
            current_mode,
            momentum_score,
            strain_score,
            self.load_subject_readiness(target.student_id, target.subject_id)?,
        )?;
        self.conn
            .execute(
                "UPDATE beat_yesterday_profiles
             SET streak_days = ?1,
                 recovery_need_score = ?2,
                 updated_at = datetime('now')
             WHERE student_id = ?3 AND subject_id = ?4",
                params![
                    streak_days,
                    strain_score,
                    target.student_id,
                    target.subject_id
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.get_daily_climb_summary(target.student_id, target.subject_id, &target.target_date)?
            .ok_or_else(|| EcoachError::NotFound("daily climb summary was not created".to_string()))
    }

    pub fn get_beat_yesterday_dashboard(
        &self,
        student_id: i64,
        subject_id: i64,
        target_date: &str,
    ) -> EcoachResult<BeatYesterdayDashboard> {
        let target = self.get_daily_climb_target(student_id, subject_id, target_date)?;
        let latest_summary = self.get_daily_climb_summary(student_id, subject_id, target_date)?;
        let previous_summary =
            self.get_previous_daily_climb_summary(student_id, subject_id, target_date)?;
        let profile = self
            .get_beat_yesterday_profile(student_id, subject_id)?
            .unwrap_or(BeatYesterdayProfile {
                student_id,
                subject_id,
                current_stage: beat_stage_from_readiness(
                    self.load_subject_readiness(student_id, subject_id)?,
                )
                .to_string(),
                current_mode: "volume_push".to_string(),
                momentum_score: 5000,
                strain_score: 0,
                readiness_score: self.load_subject_readiness(student_id, subject_id)?,
                recovery_need_score: 0,
                streak_days: 0,
            });

        Ok(BeatYesterdayDashboard {
            profile,
            target,
            latest_summary,
            previous_summary,
        })
    }

    pub fn list_climb_trend(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ClimbTrendPoint>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT summary_date, actual_attempts, actual_correct, actual_avg_response_time_ms,
                    momentum_score, strain_score, recovery_mode_triggered
             FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY summary_date DESC
             LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                Ok(ClimbTrendPoint {
                    summary_date: row.get(0)?,
                    actual_attempts: row.get(1)?,
                    actual_correct: row.get(2)?,
                    actual_avg_response_time_ms: row.get(3)?,
                    momentum_score: row.get(4)?,
                    strain_score: row.get(5)?,
                    recovery_mode_triggered: row.get::<_, i64>(6)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut points = Vec::new();
        for row in rows {
            points.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(points)
    }

    fn compute_student_momentum(&self, student_id: i64) -> EcoachResult<StudentMomentum> {
        let as_of = Utc::now().date_naive();
        let session_dates = self.load_student_session_dates(student_id)?;
        let date_set = session_dates.iter().copied().collect::<BTreeSet<_>>();
        let last_session_date = session_dates.last().copied();
        let days_since_last_session = last_session_date
            .map(|date| (as_of - date).num_days().max(0))
            .unwrap_or(0);
        let current_streak_days = match last_session_date {
            Some(last_date) => {
                let mut streak = 0;
                let mut cursor = last_date;
                while date_set.contains(&cursor) {
                    streak += 1;
                    cursor = cursor - Duration::days(1);
                }
                streak
            }
            None => 0,
        };
        let mut best_streak_days = 0;
        let mut run = 0;
        let mut previous: Option<NaiveDate> = None;
        for date in &date_set {
            if previous
                .map(|prev| *date == prev + Duration::days(1))
                .unwrap_or(false)
            {
                run += 1;
            } else {
                run = 1;
            }
            best_streak_days = best_streak_days.max(run);
            previous = Some(*date);
        }
        let consistency_7d_bp = window_consistency_bp(&date_set, as_of, 7);
        let consistency_14d_bp = window_consistency_bp(&date_set, as_of, 14);
        let consistency_30d_bp = window_consistency_bp(&date_set, as_of, 30);
        let dropout_risk_bp = derive_dropout_risk_bp(
            current_streak_days,
            days_since_last_session,
            consistency_7d_bp,
            consistency_14d_bp,
            consistency_30d_bp,
        );
        let momentum_state = derive_momentum_state(
            current_streak_days,
            days_since_last_session,
            consistency_7d_bp,
            consistency_14d_bp,
            dropout_risk_bp,
        );
        let comeback_session_count = self.count_comeback_sessions(student_id)?;

        Ok(StudentMomentum {
            id: 0,
            student_id,
            momentum_state,
            current_streak_days,
            best_streak_days,
            consistency_7d_bp,
            consistency_14d_bp,
            consistency_30d_bp,
            dropout_risk_bp,
            last_session_date: last_session_date.map(|date| date.to_string()),
            days_since_last_session,
            comeback_session_count,
        })
    }

    fn load_student_session_dates(&self, student_id: i64) -> EcoachResult<Vec<NaiveDate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT DATE(COALESCE(completed_at, last_activity_at, started_at))
                 FROM sessions
                 WHERE student_id = ?1
                   AND COALESCE(completed_at, last_activity_at, started_at) IS NOT NULL
                 ORDER BY 1 ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| row.get::<_, Option<String>>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut dates = Vec::new();
        for row in rows {
            let Some(raw_date) = row.map_err(|err| EcoachError::Storage(err.to_string()))? else {
                continue;
            };
            let date = NaiveDate::parse_from_str(&raw_date, "%Y-%m-%d")
                .map_err(|err| EcoachError::Validation(err.to_string()))?;
            dates.push(date);
        }
        Ok(dates)
    }

    fn count_comeback_sessions(&self, student_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM sessions
                 WHERE student_id = ?1
                   AND (
                        session_type LIKE 'comeback_%'
                        OR session_type IN ('guided_reinforcement', 'memory_rescue', 'retention_check')
                   )",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn select_comeback_flow_template(
        &self,
        trigger_reason: &str,
        days_inactive: i64,
    ) -> EcoachResult<Option<ComebackFlowTemplate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, template_code, display_name, trigger_condition, steps_json,
                        estimated_minutes, description
                 FROM comeback_flow_templates
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], map_comeback_flow_template)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let selected = match trigger_reason {
            "failed_session" => templates
                .iter()
                .find(|template| template.template_code == "post_failure")
                .cloned(),
            "exam_date_passed" | "subject_change" => templates
                .iter()
                .find(|template| template.template_code == "exam_recovery")
                .cloned(),
            "missed_days" | "broken_momentum" | "long_absence" => {
                if days_inactive <= 3 {
                    templates
                        .iter()
                        .find(|template| template.template_code == "short_absence")
                        .cloned()
                } else if days_inactive <= 7 {
                    templates
                        .iter()
                        .find(|template| template.template_code == "medium_absence")
                        .cloned()
                } else {
                    templates
                        .iter()
                        .find(|template| template.template_code == "long_absence")
                        .cloned()
                }
            }
            _ if days_inactive <= 3 => templates
                .iter()
                .find(|template| template.template_code == "short_absence")
                .cloned(),
            _ if days_inactive <= 7 => templates
                .iter()
                .find(|template| template.template_code == "medium_absence")
                .cloned(),
            _ => templates
                .iter()
                .find(|template| template.template_code == "long_absence")
                .cloned(),
        };

        Ok(selected)
    }

    fn get_comeback_flow(&self, flow_id: i64) -> EcoachResult<Option<ComebackFlow>> {
        self.conn
            .query_row(
                "SELECT id, student_id, trigger_reason, days_inactive, current_step,
                        flow_steps_json, status, created_at, completed_at
                 FROM comeback_flows
                 WHERE id = ?1",
                [flow_id],
                map_comeback_flow,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn sync_student_momentum(&self, student_id: i64) -> EcoachResult<StudentMomentum> {
        let momentum = self.compute_student_momentum(student_id)?;
        self.conn
            .execute(
                "INSERT INTO student_momentum (
                    student_id, momentum_state, current_streak_days, best_streak_days,
                    consistency_7d_bp, consistency_14d_bp, consistency_30d_bp,
                    dropout_risk_bp, last_session_date, days_since_last_session,
                    comeback_session_count
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(student_id) DO UPDATE SET
                    momentum_state = excluded.momentum_state,
                    current_streak_days = excluded.current_streak_days,
                    best_streak_days = excluded.best_streak_days,
                    consistency_7d_bp = excluded.consistency_7d_bp,
                    consistency_14d_bp = excluded.consistency_14d_bp,
                    consistency_30d_bp = excluded.consistency_30d_bp,
                    dropout_risk_bp = excluded.dropout_risk_bp,
                    last_session_date = excluded.last_session_date,
                    days_since_last_session = excluded.days_since_last_session,
                    comeback_session_count = excluded.comeback_session_count,
                    updated_at = datetime('now')",
                params![
                    momentum.student_id,
                    momentum.momentum_state,
                    momentum.current_streak_days,
                    momentum.best_streak_days,
                    momentum.consistency_7d_bp,
                    momentum.consistency_14d_bp,
                    momentum.consistency_30d_bp,
                    momentum.dropout_risk_bp,
                    momentum.last_session_date,
                    momentum.days_since_last_session,
                    momentum.comeback_session_count,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_student_momentum(student_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("student momentum {} not found", student_id))
        })
    }

    pub fn get_student_momentum(&self, student_id: i64) -> EcoachResult<Option<StudentMomentum>> {
        self.conn
            .query_row(
                "SELECT id, student_id, momentum_state, current_streak_days, best_streak_days,
                        consistency_7d_bp, consistency_14d_bp, consistency_30d_bp,
                        dropout_risk_bp, last_session_date, days_since_last_session,
                        comeback_session_count
                 FROM student_momentum
                 WHERE student_id = ?1",
                [student_id],
                map_student_momentum,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn start_comeback_flow(
        &self,
        student_id: i64,
        trigger_reason: &str,
        days_inactive: i64,
    ) -> EcoachResult<ComebackFlow> {
        validate_comeback_trigger_reason(trigger_reason)?;
        let template = self.select_comeback_flow_template(trigger_reason, days_inactive)?;
        let flow_steps = template
            .as_ref()
            .map(|item| item.steps.clone())
            .unwrap_or_else(|| {
                vec![
                    "momentum_reset".to_string(),
                    "low_pressure_reentry".to_string(),
                    "confidence_check".to_string(),
                ]
            });
        let payload = json!({
            "template_code": template.as_ref().map(|item| item.template_code.clone()),
            "template_name": template.as_ref().map(|item| item.display_name.clone()),
            "trigger_condition": template.as_ref().map(|item| item.trigger_condition.clone()),
            "steps": flow_steps,
            "estimated_minutes": template.as_ref().map(|item| item.estimated_minutes),
        });
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO comeback_flows (
                    student_id, trigger_reason, days_inactive, flow_steps_json, current_step, status, created_at
                 ) VALUES (?1, ?2, ?3, ?4, 0, 'active', ?5)",
                params![
                    student_id,
                    trigger_reason,
                    days_inactive.max(0),
                    serde_json::to_string(&payload)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let flow_id = self.conn.last_insert_rowid();
        self.get_comeback_flow(flow_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("comeback flow {} not found", flow_id)))
    }

    pub fn get_active_comeback_flow(&self, student_id: i64) -> EcoachResult<Option<ComebackFlow>> {
        self.conn
            .query_row(
                "SELECT id, student_id, trigger_reason, days_inactive, current_step,
                        flow_steps_json, status, created_at, completed_at
                 FROM comeback_flows
                 WHERE student_id = ?1 AND status = 'active'
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                [student_id],
                map_comeback_flow,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_revenge_queue(&self, student_id: i64) -> EcoachResult<Vec<RevengeQueueItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT rq.id, rq.student_id, rq.question_id, rq.original_session_id,
                        rq.original_error_type, rq.original_wrong_answer, rq.attempts_to_beat,
                        rq.is_beaten, rq.beaten_at, rq.added_at, q.stem, q.topic_id, NULL
                 FROM revenge_queue rq
                 INNER JOIN questions q ON q.id = rq.question_id
                 WHERE rq.student_id = ?1
                 ORDER BY rq.is_beaten ASC, rq.added_at DESC, rq.id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], map_revenge_queue_item)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    fn base_minutes_for_day(&self, student_id: i64, weekday: i64) -> EcoachResult<i64> {
        let total_window_minutes: Option<i64> = self
            .conn
            .query_row(
                "SELECT SUM(end_minute - start_minute)
             FROM availability_windows
             WHERE student_id = ?1 AND weekday = ?2",
                params![student_id, weekday],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(total_window_minutes) = total_window_minutes {
            return Ok(total_window_minutes.max(0));
        }

        let profile_minutes: Option<i64> = self
            .conn
            .query_row(
                "SELECT preferred_daily_minutes
             FROM availability_profiles
             WHERE student_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(profile_minutes.unwrap_or(60))
    }

    fn load_session_bounds(&self, student_id: i64) -> EcoachResult<(i64, i64)> {
        let bounds = self
            .conn
            .query_row(
                "SELECT min_session_minutes, max_session_minutes
                 FROM availability_profiles
                 WHERE student_id = ?1",
                [student_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(bounds.unwrap_or((15, 90)))
    }

    fn current_window_end_minute(
        &self,
        student_id: i64,
        date: &str,
        minute_of_day: i64,
    ) -> EcoachResult<Option<i64>> {
        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let weekday = target_date.weekday().num_days_from_monday() as i64;
        self.conn
            .query_row(
                "SELECT MIN(end_minute)
                 FROM availability_windows
                 WHERE student_id = ?1
                   AND weekday = ?2
                   AND ?3 >= start_minute
                   AND ?3 < end_minute",
                params![student_id, weekday, minute_of_day],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn remaining_day_capacity_minutes(
        &self,
        student_id: i64,
        date: &str,
        minute_of_day: i64,
    ) -> EcoachResult<i64> {
        let summary = self.get_daily_availability(student_id, date)?;
        if summary.blocked || summary.adjusted_minutes <= 0 {
            return Ok(0);
        }

        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let weekday = target_date.weekday().num_days_from_monday() as i64;
        let mut statement = self
            .conn
            .prepare(
                "SELECT start_minute, end_minute
                 FROM availability_windows
                 WHERE student_id = ?1 AND weekday = ?2
                 ORDER BY start_minute ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, weekday], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut remaining = 0;
        for row in rows {
            let (start_minute, end_minute) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if end_minute <= minute_of_day {
                continue;
            }
            let effective_start = start_minute.max(minute_of_day);
            remaining += (end_minute - effective_start).max(0);
        }

        Ok(remaining.min(summary.adjusted_minutes).max(0))
    }

    fn load_availability_profile(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<AvailabilityProfile>> {
        self.conn
            .query_row(
                "SELECT student_id, timezone_name, preferred_daily_minutes, ideal_session_minutes,
                        min_session_minutes, max_session_minutes, split_sessions_allowed,
                        max_split_sessions, min_break_minutes, trigger_mode,
                        notification_lead_minutes, weekday_capacity_weight_bp,
                        weekend_capacity_weight_bp, schedule_buffer_ratio_bp,
                        fatigue_start_minute, fatigue_end_minute, thinking_idle_grace_seconds,
                        idle_confirmation_seconds, abandonment_seconds
                 FROM availability_profiles
                 WHERE student_id = ?1",
                [student_id],
                |row| {
                    Ok(AvailabilityProfile {
                        student_id: row.get(0)?,
                        timezone_name: row.get(1)?,
                        preferred_daily_minutes: row.get(2)?,
                        ideal_session_minutes: row.get(3)?,
                        min_session_minutes: row.get(4)?,
                        max_session_minutes: row.get(5)?,
                        split_sessions_allowed: row.get::<_, i64>(6)? == 1,
                        max_split_sessions: row.get(7)?,
                        min_break_minutes: row.get(8)?,
                        trigger_mode: row.get(9)?,
                        notification_lead_minutes: row.get(10)?,
                        weekday_capacity_weight_bp: row.get(11)?,
                        weekend_capacity_weight_bp: row.get(12)?,
                        schedule_buffer_ratio_bp: row.get(13)?,
                        fatigue_start_minute: row.get(14)?,
                        fatigue_end_minute: row.get(15)?,
                        thinking_idle_grace_seconds: row.get(16)?,
                        idle_confirmation_seconds: row.get(17)?,
                        abandonment_seconds: row.get(18)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_availability_profile_or_default(
        &self,
        student_id: i64,
    ) -> EcoachResult<AvailabilityProfile> {
        Ok(self
            .load_availability_profile(student_id)?
            .unwrap_or_else(|| default_availability_profile(student_id)))
    }

    fn load_goal_profile(&self, goal_id: i64) -> EcoachResult<Option<GoalProfile>> {
        self.conn
            .query_row(
                "SELECT id, student_id, parent_goal_id, title, description, goal_type,
                        goal_category, goal_level, goal_state, subject_id, topics_json,
                        urgency_level, start_date, deadline, exam_id, confidence_score_bp,
                        coach_priority_bp, evidence_sources_json, dependency_goals_json,
                        parent_priority_flag, risk_level, suggested_weekly_effort_minutes,
                        completion_criteria_json, current_momentum_bp, blocked_reason,
                        source_bundle_id, goal_signal_key, metadata_json, created_at, updated_at
                 FROM goals
                 WHERE id = ?1",
                [goal_id],
                map_goal_profile,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn latest_exam_plan_state(
        &self,
        student_id: i64,
        subject_id: i64,
        exam_date: Option<&str>,
    ) -> EcoachResult<Option<ExamPlanState>> {
        let sql = if exam_date.is_some() {
            "SELECT id, student_id, subject_id, anchor_date, exam_date,
                    target_effective_minutes, completed_effective_minutes,
                    remaining_effective_minutes, available_study_days,
                    required_weekly_minutes, protected_buffer_minutes,
                    buffer_consumed_minutes, missed_debt_minutes, bonus_credit_minutes,
                    pressure_score_bp, feasibility_score_bp, schedule_truth_score_bp,
                    plan_mode, auto_trigger_mode, explanation_json, updated_at
             FROM exam_plan_states
             WHERE student_id = ?1 AND subject_id = ?2 AND exam_date = ?3
             ORDER BY updated_at DESC, id DESC
             LIMIT 1"
        } else {
            "SELECT id, student_id, subject_id, anchor_date, exam_date,
                    target_effective_minutes, completed_effective_minutes,
                    remaining_effective_minutes, available_study_days,
                    required_weekly_minutes, protected_buffer_minutes,
                    buffer_consumed_minutes, missed_debt_minutes, bonus_credit_minutes,
                    pressure_score_bp, feasibility_score_bp, schedule_truth_score_bp,
                    plan_mode, auto_trigger_mode, explanation_json, updated_at
             FROM exam_plan_states
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY exam_date ASC, updated_at DESC, id DESC
             LIMIT 1"
        };
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let result = if let Some(exam_date) = exam_date {
            statement.query_row(
                params![student_id, subject_id, exam_date],
                map_exam_plan_state,
            )
        } else {
            statement.query_row(params![student_id, subject_id], map_exam_plan_state)
        };
        result
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_available_study_days(
        &self,
        student_id: i64,
        anchor_date: &str,
        exam_date: &str,
    ) -> EcoachResult<i64> {
        let start = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let end = NaiveDate::parse_from_str(exam_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        if end < start {
            return Ok(0);
        }
        let mut count = 0;
        let mut cursor = start;
        while cursor <= end {
            let summary = self.get_daily_availability(student_id, &cursor.to_string())?;
            if !summary.blocked && summary.adjusted_minutes > 0 {
                count += 1;
            }
            cursor += Duration::days(1);
        }
        Ok(count)
    }

    fn total_available_minutes_between(
        &self,
        student_id: i64,
        anchor_date: &str,
        exam_date: &str,
    ) -> EcoachResult<i64> {
        let start = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let end = NaiveDate::parse_from_str(exam_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        if end < start {
            return Ok(0);
        }
        let mut total = 0;
        let mut cursor = start;
        while cursor <= end {
            let summary = self.get_daily_availability(student_id, &cursor.to_string())?;
            total += summary.adjusted_minutes.max(0);
            cursor += Duration::days(1);
        }
        Ok(total)
    }

    fn load_effective_minutes_completed(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<i64> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT COALESCE(active_study_time_ms, 0), accuracy_score, session_type
                 FROM sessions
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'completed'",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut effective_minutes = 0_i64;
        let mut saw_session_minutes = false;
        for row in rows {
            let (active_study_time_ms, accuracy_score, session_type) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if active_study_time_ms <= 0 {
                continue;
            }
            saw_session_minutes = true;
            let mut quality_bp = 5_500 + accuracy_score.unwrap_or(5_500) / 2;
            if matches!(session_type.as_str(), "mock" | "custom_test") {
                quality_bp += 500;
            }
            let credited_ms =
                active_study_time_ms.saturating_mul(clamp_bp(quality_bp) as i64) / 10_000;
            effective_minutes += credited_ms / 60_000;
        }
        if saw_session_minutes {
            return Ok(effective_minutes.max(0));
        }
        let fallback_minutes: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(COALESCE(sqa.response_time_ms, 0)), 0)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1 AND q.subject_id = ?2",
                params![student_id, subject_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok((fallback_minutes * 6 / 10) / 60_000)
    }

    fn load_schedule_carryover(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<(i64, i64, i64)> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(buffer_minutes_consumed), 0),
                        COALESCE(SUM(missed_minutes_debt), 0),
                        COALESCE(SUM(bonus_minutes_credit), 0)
                 FROM schedule_ledger
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_completed_minutes_for_date(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
    ) -> EcoachResult<(i64, i64)> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT COALESCE(active_study_time_ms, 0), accuracy_score, session_type
                 FROM sessions
                 WHERE student_id = ?1
                   AND subject_id = ?2
                   AND DATE(COALESCE(completed_at, started_at, created_at)) = DATE(?3)",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, date], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut completed_minutes = 0_i64;
        let mut effective_minutes = 0_i64;
        for row in rows {
            let (active_study_time_ms, accuracy_score, session_type) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let active_minutes = active_study_time_ms.max(0) / 60_000;
            completed_minutes += active_minutes;
            let mut quality_bp = 5_500 + accuracy_score.unwrap_or(5_500) / 2;
            if matches!(session_type.as_str(), "mock" | "custom_test") {
                quality_bp += 500;
            }
            effective_minutes +=
                active_minutes.saturating_mul(clamp_bp(quality_bp) as i64) / 10_000;
        }
        Ok((completed_minutes.max(0), effective_minutes.max(0)))
    }

    fn sync_schedule_ledger_entry(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
    ) -> EcoachResult<ScheduleLedgerEntry> {
        let profile = self.load_availability_profile_or_default(student_id)?;
        let availability = self.get_daily_availability(student_id, date)?;
        let plan_state = self.latest_exam_plan_state(student_id, subject_id, None)?;
        let buffer_minutes_reserved = availability
            .adjusted_minutes
            .saturating_mul(profile.schedule_buffer_ratio_bp as i64)
            / 10_000;
        let scheduled_minutes = if availability.blocked || availability.adjusted_minutes <= 0 {
            0
        } else if let Some(plan_state) = plan_state.as_ref() {
            let daily_goal = if plan_state.available_study_days <= 0 {
                plan_state.remaining_effective_minutes
            } else {
                ((plan_state.remaining_effective_minutes as f64)
                    / plan_state.available_study_days.max(1) as f64)
                    .ceil() as i64
            };
            daily_goal
                .min((availability.adjusted_minutes - buffer_minutes_reserved).max(0))
                .max(
                    profile
                        .min_session_minutes
                        .min(availability.adjusted_minutes),
                )
        } else {
            profile
                .ideal_session_minutes
                .min((availability.adjusted_minutes - buffer_minutes_reserved).max(0))
                .max(0)
        };
        let (completed_minutes, effective_credit_minutes) =
            self.load_completed_minutes_for_date(student_id, subject_id, date)?;
        let bonus_minutes_credit = (effective_credit_minutes - scheduled_minutes).max(0);
        let buffer_minutes_consumed = (scheduled_minutes - completed_minutes)
            .max(0)
            .min(buffer_minutes_reserved.max(0));
        let missed_minutes_debt =
            (scheduled_minutes - effective_credit_minutes - buffer_minutes_reserved).max(0);
        let pressure_score_bp = plan_state
            .as_ref()
            .map(|item| item.pressure_score_bp)
            .unwrap_or_else(|| clamp_bp((missed_minutes_debt * 10_000) / scheduled_minutes.max(1)));
        let feasibility_score_bp = if scheduled_minutes <= 0 {
            10_000
        } else {
            clamp_bp((effective_credit_minutes * 10_000) / scheduled_minutes.max(1))
        };
        let explanation = json!({
            "adjusted_minutes": availability.adjusted_minutes,
            "scheduled_minutes": scheduled_minutes,
            "completed_minutes": completed_minutes,
            "effective_credit_minutes": effective_credit_minutes,
            "buffer_minutes_reserved": buffer_minutes_reserved,
            "buffer_minutes_consumed": buffer_minutes_consumed,
            "missed_minutes_debt": missed_minutes_debt,
            "bonus_minutes_credit": bonus_minutes_credit,
        });
        self.conn
            .execute(
                "INSERT INTO schedule_ledger (
                    student_id, subject_id, ledger_date, scheduled_minutes, completed_minutes,
                    effective_credit_minutes, buffer_minutes_reserved, buffer_minutes_consumed,
                    missed_minutes_debt, bonus_minutes_credit, pressure_score_bp,
                    feasibility_score_bp, explanation_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(student_id, subject_id, ledger_date) DO UPDATE SET
                    scheduled_minutes = excluded.scheduled_minutes,
                    completed_minutes = excluded.completed_minutes,
                    effective_credit_minutes = excluded.effective_credit_minutes,
                    buffer_minutes_reserved = excluded.buffer_minutes_reserved,
                    buffer_minutes_consumed = excluded.buffer_minutes_consumed,
                    missed_minutes_debt = excluded.missed_minutes_debt,
                    bonus_minutes_credit = excluded.bonus_minutes_credit,
                    pressure_score_bp = excluded.pressure_score_bp,
                    feasibility_score_bp = excluded.feasibility_score_bp,
                    explanation_json = excluded.explanation_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    date,
                    scheduled_minutes,
                    completed_minutes,
                    effective_credit_minutes,
                    buffer_minutes_reserved,
                    buffer_minutes_consumed,
                    missed_minutes_debt,
                    bonus_minutes_credit,
                    pressure_score_bp,
                    feasibility_score_bp,
                    explanation.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id, ledger_date, scheduled_minutes,
                        completed_minutes, effective_credit_minutes, buffer_minutes_reserved,
                        buffer_minutes_consumed, missed_minutes_debt, bonus_minutes_credit,
                        pressure_score_bp, explanation_json, feasibility_score_bp, updated_at
                 FROM schedule_ledger
                 WHERE student_id = ?1 AND subject_id = ?2 AND ledger_date = ?3",
                params![student_id, subject_id, date],
                map_schedule_ledger_entry,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn rebuild_time_session_blocks(
        &self,
        student_id: i64,
        subject_id: i64,
        plan_state: &ExamPlanState,
        anchor_date: &str,
        horizon_days: usize,
        profile: &AvailabilityProfile,
        replan: &DailyReplan,
    ) -> EcoachResult<Vec<TimeSessionBlock>> {
        let start = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let end = start + Duration::days(horizon_days.saturating_sub(1) as i64);
        self.conn
            .execute(
                "DELETE FROM schedule_trigger_jobs
                 WHERE session_block_id IN (
                    SELECT id FROM time_session_blocks
                    WHERE student_id = ?1 AND subject_id = ?2 AND block_date BETWEEN ?3 AND ?4
                 )",
                params![student_id, subject_id, anchor_date, end.to_string()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM time_session_blocks
                 WHERE student_id = ?1 AND subject_id = ?2 AND block_date BETWEEN ?3 AND ?4",
                params![student_id, subject_id, anchor_date, end.to_string()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let windows = self.list_availability_windows(student_id)?;
        let focus_topic_ids = self.load_focus_topic_ids(student_id, subject_id, 3)?;
        let mut remaining_minutes = plan_state.remaining_effective_minutes.max(0);
        let mut blocks = Vec::new();
        for offset in 0..horizon_days.max(1) {
            let date = start + Duration::days(offset as i64);
            let date_string = date.to_string();
            let availability = self.get_daily_availability(student_id, &date_string)?;
            if availability.blocked || availability.adjusted_minutes <= 0 {
                continue;
            }
            let day_windows: Vec<_> = windows
                .iter()
                .filter(|window| window.weekday == date.weekday().num_days_from_monday() as i64)
                .collect();
            let lead_window = day_windows
                .iter()
                .find(|window| window.is_preferred)
                .copied()
                .or_else(|| day_windows.first().copied());
            let base_minutes = (availability.adjusted_minutes
                - availability
                    .adjusted_minutes
                    .saturating_mul(profile.schedule_buffer_ratio_bp as i64)
                    / 10_000)
                .max(0);
            let remaining_days = (horizon_days - offset).max(1) as i64;
            let evenly_spread = if remaining_minutes <= 0 {
                profile.ideal_session_minutes
            } else {
                ((remaining_minutes as f64) / remaining_days as f64).ceil() as i64
            };
            let target_minutes = evenly_spread
                .min(base_minutes.max(profile.min_session_minutes.min(base_minutes)))
                .min(profile.max_session_minutes.max(profile.min_session_minutes))
                .max(0);
            if target_minutes <= 0 {
                continue;
            }
            let session_type = if offset == 0 {
                replan.next_session_type.clone()
            } else if plan_state.plan_mode == "exam_performance" {
                "timed_exam_rehearsal".to_string()
            } else if plan_state.plan_mode == "rescue" {
                "repair_recovery".to_string()
            } else {
                "guided_mastery".to_string()
            };
            let trigger_mode = derive_block_trigger_mode(
                &profile.trigger_mode,
                plan_state.pressure_score_bp,
                date.weekday().num_days_from_monday() as i64,
                &session_type,
            );
            let window_minutes = lead_window
                .map(|window| (window.end_minute - window.start_minute).max(0))
                .unwrap_or(target_minutes);
            let fit_score_bp = clamp_bp((window_minutes * 10_000) / target_minutes.max(1));
            let priority_score_bp = clamp_bp(
                ((plan_state.pressure_score_bp as i64 + replan.pressure_score as i64) / 2),
            );
            let objective_summary = if offset == 0 {
                replan.rationale.clone()
            } else {
                format!(
                    "Protect {} minutes for {} while preserving spacing and recovery buffer.",
                    target_minutes, session_type
                )
            };
            let explanation_text = format!(
                "{} window reserved with {} trigger behavior because the plan is in {} mode.",
                target_minutes, trigger_mode, plan_state.plan_mode
            );
            self.conn
                .execute(
                    "INSERT INTO time_session_blocks (
                        student_id, subject_id, plan_state_id, block_date, start_minute,
                        end_minute, target_minutes, session_type, objective_summary,
                        focus_topic_ids_json, trigger_mode, fit_score_bp, priority_score_bp,
                        status, fallback_session_type, replacement_options_json, created_by,
                        source_kind, explanation_text
                     ) VALUES (
                        ?1, ?2, ?3, ?4, ?5,
                        ?6, ?7, ?8, ?9,
                        ?10, ?11, ?12, ?13,
                        'planned', ?14, ?15, 'orchestrator',
                        'time_orchestrator', ?16
                     )",
                    params![
                        student_id,
                        subject_id,
                        plan_state.id,
                        date_string,
                        lead_window.map(|window| window.start_minute),
                        lead_window.map(|window| window.end_minute),
                        target_minutes,
                        session_type,
                        objective_summary,
                        serde_json::to_string(&focus_topic_ids)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        trigger_mode,
                        fit_score_bp,
                        priority_score_bp,
                        Some("free_now_recovery".to_string()),
                        serde_json::to_string(&vec!["bonus_priority_push", "retention_check"])
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        explanation_text,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let block_id = self.conn.last_insert_rowid();
            blocks.push(self.conn.query_row(
                "SELECT id, student_id, subject_id, plan_state_id, block_date, start_minute,
                        end_minute, target_minutes, session_type, objective_summary,
                        focus_topic_ids_json, trigger_mode, fit_score_bp, priority_score_bp,
                        status, fallback_session_type, replacement_options_json, created_by,
                        source_kind, explanation_text, linked_session_id
                 FROM time_session_blocks WHERE id = ?1",
                [block_id],
                map_time_session_block,
            ).map_err(|err| EcoachError::Storage(err.to_string()))?);
            remaining_minutes = remaining_minutes.saturating_sub(target_minutes);
        }
        Ok(blocks)
    }

    fn upsert_trigger_jobs(
        &self,
        student_id: i64,
        subject_id: i64,
        profile: &AvailabilityProfile,
        blocks: &[TimeSessionBlock],
    ) -> EcoachResult<Vec<ScheduleTriggerJob>> {
        self.conn
            .execute(
                "DELETE FROM schedule_trigger_jobs
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'scheduled'",
                params![student_id, subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut jobs = Vec::new();
        for block in blocks {
            let trigger_kind = match block.trigger_mode.as_str() {
                "auto" => "launch",
                "hybrid" => "hybrid_prompt",
                _ => "reminder",
            };
            let scheduled_minute = block
                .start_minute
                .unwrap_or(18 * 60)
                .saturating_sub(profile.notification_lead_minutes.max(0));
            let scheduled_for = combine_date_minute(&block.block_date, scheduled_minute);
            let payload = json!({
                "session_block_id": block.id,
                "session_type": block.session_type,
                "objective_summary": block.objective_summary,
            });
            self.conn
                .execute(
                    "INSERT INTO schedule_trigger_jobs (
                        student_id, subject_id, session_block_id, trigger_kind,
                        scheduled_for, lead_minutes, status, payload_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'scheduled', ?7)",
                    params![
                        student_id,
                        subject_id,
                        block.id,
                        trigger_kind,
                        scheduled_for,
                        profile.notification_lead_minutes,
                        payload.to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let job_id = self.conn.last_insert_rowid();
            jobs.push(self.conn.query_row(
                "SELECT id, student_id, subject_id, session_block_id, trigger_kind, scheduled_for,
                        lead_minutes, status, payload_json
                 FROM schedule_trigger_jobs WHERE id = ?1",
                [job_id],
                map_schedule_trigger_job,
            ).map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(jobs)
    }

    fn load_active_mission(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
    ) -> EcoachResult<Option<PendingMissionWindow>> {
        self.conn
            .query_row(
                "SELECT cm.title, cm.activity_type, cm.primary_topic_id
                 FROM coach_missions cm
                 LEFT JOIN coach_plan_days cpd ON cpd.id = cm.plan_day_id
                 WHERE cm.student_id = ?1
                   AND cm.subject_id = ?2
                   AND cm.status IN ('active', 'pending')
                   AND (cpd.date = ?3 OR cm.status = 'active')
                 ORDER BY CASE cm.status WHEN 'active' THEN 0 ELSE 1 END, cm.id DESC
                 LIMIT 1",
                params![student_id, subject_id, date],
                |row| {
                    Ok(PendingMissionWindow {
                        title: row.get(0)?,
                        activity_type: row.get(1)?,
                        primary_topic_id: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_due_memory_topics(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM memory_states ms
                 INNER JOIN topics t ON t.id = ms.topic_id
                 WHERE ms.student_id = ?1
                   AND t.subject_id = ?2
                   AND ms.review_due_at IS NOT NULL
                   AND DATE(ms.review_due_at) <= DATE(?3)",
                params![student_id, subject_id, date],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_recent_solidification_outcomes(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
        limit: usize,
    ) -> EcoachResult<BTreeMap<i64, RecentSolidificationOutcome>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    ss.topic_id,
                    t.name,
                    re.payload_json
                 FROM solidification_sessions ss
                 INNER JOIN topics t ON t.id = ss.topic_id
                 INNER JOIN sessions s ON s.id = ss.session_id
                 INNER JOIN runtime_events re
                    ON re.aggregate_kind = 'session'
                   AND re.aggregate_id = CAST(s.id AS TEXT)
                   AND re.event_type = 'session.interpreted'
                 WHERE ss.student_id = ?1
                   AND t.subject_id = ?2
                   AND DATE(COALESCE(ss.completed_at, s.completed_at, re.occurred_at)) >= DATE(?3, '-3 day')
                 ORDER BY COALESCE(ss.completed_at, s.completed_at, re.occurred_at) DESC, re.id DESC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![student_id, subject_id, date, limit.max(1) as i64],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut outcomes = BTreeMap::new();
        for row in rows {
            let (topic_id, topic_name, payload_json) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if outcomes.contains_key(&topic_id) {
                continue;
            }
            let payload: Value = serde_json::from_str(&payload_json)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
            outcomes.insert(
                topic_id,
                RecentSolidificationOutcome {
                    topic_id,
                    topic_name,
                    outcome: payload
                        .get("repair_outcome")
                        .and_then(|value| value.as_str())
                        .unwrap_or("mixed")
                        .to_string(),
                    next_action_hint: payload
                        .get("next_action_hint")
                        .and_then(|value| value.as_str())
                        .unwrap_or("repair_retry")
                        .to_string(),
                    accuracy_score: payload
                        .get("topic_summaries")
                        .and_then(|value| value.as_array())
                        .and_then(|items| items.first())
                        .and_then(|item| item.get("accuracy_score"))
                        .and_then(|value| value.as_u64())
                        .map(|value| value as BasisPoints),
                },
            );
        }

        Ok(outcomes)
    }

    fn load_topic_pressure_candidates(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
        limit: usize,
    ) -> EcoachResult<BTreeMap<i64, TopicPressureCandidate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    sts.topic_id,
                    t.name,
                    sts.priority_score,
                    sts.gap_score,
                    sts.repair_priority,
                    sts.fragility_score,
                    sts.is_urgent,
                    COALESCE(ms_stats.due_items, 0),
                    COALESCE(ms_stats.fragile_items, 0),
                    COALESCE(ms_stats.collapsed_items, 0)
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 LEFT JOIN (
                    SELECT
                        topic_id,
                        SUM(CASE WHEN review_due_at IS NOT NULL AND DATE(review_due_at) <= DATE(?3) THEN 1 ELSE 0 END) AS due_items,
                        SUM(CASE WHEN memory_state IN ('fragile', 'at_risk', 'fading', 'rebuilding') THEN 1 ELSE 0 END) AS fragile_items,
                        SUM(CASE WHEN memory_state = 'collapsed' THEN 1 ELSE 0 END) AS collapsed_items
                    FROM memory_states
                    WHERE student_id = ?1
                      AND topic_id IS NOT NULL
                    GROUP BY topic_id
                 ) ms_stats ON ms_stats.topic_id = sts.topic_id
                 WHERE sts.student_id = ?1
                   AND t.subject_id = ?2
                 ORDER BY
                    sts.is_urgent DESC,
                    sts.repair_priority DESC,
                    COALESCE(ms_stats.collapsed_items, 0) DESC,
                    COALESCE(ms_stats.due_items, 0) DESC,
                    sts.priority_score DESC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![student_id, subject_id, date, limit.max(1) as i64],
                |row| {
                    Ok(TopicPressureCandidate {
                        topic_id: row.get(0)?,
                        topic_name: row.get(1)?,
                        priority_score: row.get(2)?,
                        gap_score: row.get(3)?,
                        repair_priority: row.get(4)?,
                        fragility_score: row.get(5)?,
                        is_urgent: row.get::<_, i64>(6)? == 1,
                        due_items: row.get(7)?,
                        fragile_items: row.get(8)?,
                        collapsed_items: row.get(9)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut candidates = BTreeMap::new();
        for row in rows {
            let candidate = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            candidates.insert(candidate.topic_id, candidate);
        }
        Ok(candidates)
    }

    fn build_comeback_pressure(
        &self,
        student_id: i64,
        subject_id: i64,
        date: &str,
        limit: usize,
    ) -> EcoachResult<ComebackPressure> {
        let outcomes =
            self.load_recent_solidification_outcomes(student_id, subject_id, date, 12)?;
        let mut candidates =
            self.load_topic_pressure_candidates(student_id, subject_id, date, limit.max(6))?;

        for outcome in outcomes.values() {
            candidates
                .entry(outcome.topic_id)
                .or_insert_with(|| TopicPressureCandidate {
                    topic_id: outcome.topic_id,
                    topic_name: outcome.topic_name.clone(),
                    priority_score: 0,
                    gap_score: 0,
                    repair_priority: 0,
                    fragility_score: 0,
                    is_urgent: false,
                    due_items: 0,
                    fragile_items: 0,
                    collapsed_items: 0,
                });
        }

        let mut ranked = candidates
            .into_values()
            .map(|candidate| {
                let outcome = outcomes.get(&candidate.topic_id).cloned();
                let outcome_bonus = match outcome.as_ref().map(|item| item.outcome.as_str()) {
                    Some("failed") => 2600,
                    Some("mixed") => 1200,
                    Some("success") => -900,
                    _ => 0,
                };
                let accuracy_adjustment =
                    match outcome.as_ref().and_then(|item| item.accuracy_score) {
                        Some(score) if score < 3000 => 900,
                        Some(score) if score >= 8000 => -400,
                        _ => 0,
                    };
                let adjusted_score = clamp_bp(
                    (candidate.repair_priority as i64 * 45 / 100)
                        + (candidate.priority_score as i64 * 20 / 100)
                        + (candidate.gap_score as i64 * 10 / 100)
                        + (candidate.fragility_score as i64 * 10 / 100)
                        + candidate.due_items.min(4) * 650
                        + candidate.fragile_items.min(4) * 450
                        + candidate.collapsed_items.min(3) * 1200
                        + if candidate.is_urgent { 1100 } else { 0 }
                        + outcome_bonus
                        + accuracy_adjustment,
                );
                RankedPressureTopic {
                    candidate,
                    outcome,
                    adjusted_score,
                }
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| {
            right
                .adjusted_score
                .cmp(&left.adjusted_score)
                .then(
                    right
                        .candidate
                        .repair_priority
                        .cmp(&left.candidate.repair_priority),
                )
                .then(
                    right
                        .candidate
                        .priority_score
                        .cmp(&left.candidate.priority_score),
                )
                .then(left.candidate.topic_id.cmp(&right.candidate.topic_id))
        });

        let focus_topic_ids = ranked
            .iter()
            .map(|item| item.candidate.topic_id)
            .take(limit.max(1))
            .collect::<Vec<_>>();

        let Some(top) = ranked.first() else {
            return Ok(ComebackPressure {
                focus_topic_ids,
                recommended_topic_id: None,
                pressure_score: 0,
                repair_buffer_minutes: 0,
                recommended_session_type: None,
                recent_repair_outcome: None,
                rationale: None,
            });
        };

        let due_topic_count = ranked
            .iter()
            .filter(|item| item.candidate.due_items > 0)
            .count() as i64;
        let urgent_topic_count = ranked
            .iter()
            .filter(|item| item.candidate.is_urgent || item.candidate.repair_priority >= 7000)
            .count() as i64;

        let (recommended_session_type, repair_buffer_minutes, rationale) = match top
            .outcome
            .as_ref()
            .map(|item| item.outcome.as_str())
        {
            Some("failed") => {
                let session_type = if top.candidate.collapsed_items > 0
                    || top
                        .outcome
                        .as_ref()
                        .is_some_and(|item| item.next_action_hint.contains("reteach"))
                {
                    "comeback_reteach"
                } else {
                    "comeback_repair"
                };
                let buffer = (12
                    + top.candidate.due_items.min(3) * 3
                    + top.candidate.fragile_items.min(3) * 2
                    + top.candidate.collapsed_items.min(2) * 5
                    + if top.candidate.is_urgent { 4 } else { 0 })
                .clamp(12, 32);
                let rationale = format!(
                    "{} did not hold in recent solidification work, and the return loop is still carrying {} due review(s), {} fragile trace(s), and {} collapsed trace(s). Replan around a comeback block before pushing forward.",
                    top.candidate.topic_name,
                    top.candidate.due_items,
                    top.candidate.fragile_items,
                    top.candidate.collapsed_items,
                );
                (Some(session_type.to_string()), buffer, Some(rationale))
            }
            Some("mixed") => {
                let buffer = (10
                    + top.candidate.due_items.min(3) * 2
                    + top.candidate.fragile_items.min(3) * 2
                    + top.candidate.collapsed_items.min(2) * 3)
                    .clamp(10, 24);
                let rationale = format!(
                    "{} only partially held in the last solidification session, so the next block should reinforce the repair while the remaining memory pressure is still visible.",
                    top.candidate.topic_name
                );
                (
                    Some("guided_reinforcement".to_string()),
                    buffer,
                    Some(rationale),
                )
            }
            Some("success")
                if top.candidate.collapsed_items == 0
                    && top.candidate.due_items <= 1
                    && top.candidate.fragile_items <= 1
                    && top.adjusted_score < 7000 =>
            {
                let rationale = format!(
                    "{} recently held in solidification and the live memory pressure is light, so a short retention check is enough instead of another heavy repair block.",
                    top.candidate.topic_name
                );
                (Some("retention_check".to_string()), 6, Some(rationale))
            }
            _ if top.candidate.collapsed_items > 0
                || top.candidate.due_items > 0
                || top.candidate.fragile_items > 1
                || top.candidate.is_urgent
                || due_topic_count > 1
                || urgent_topic_count > 0 =>
            {
                let buffer = (8
                    + top.candidate.due_items.min(4) * 2
                    + top.candidate.collapsed_items.min(2) * 4
                    + top.candidate.fragile_items.min(3))
                .clamp(8, 26);
                let rationale = format!(
                    "Return-loop pressure is building around {} and {} topic(s) are already due, so the next plan should reserve time for memory rescue before the rest of the day slips.",
                    top.candidate.topic_name,
                    due_topic_count.max(1),
                );
                (Some("memory_rescue".to_string()), buffer, Some(rationale))
            }
            _ => (None, 0, None),
        };

        Ok(ComebackPressure {
            focus_topic_ids,
            recommended_topic_id: Some(top.candidate.topic_id),
            pressure_score: top.adjusted_score,
            repair_buffer_minutes,
            recommended_session_type,
            recent_repair_outcome: top.outcome.as_ref().map(|item| item.outcome.clone()),
            rationale,
        })
    }

    fn get_beat_yesterday_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<BeatYesterdayProfile>> {
        self.conn
            .query_row(
                "SELECT student_id, subject_id, current_stage, current_mode, momentum_score,
                        strain_score, readiness_score, recovery_need_score, streak_days
                 FROM beat_yesterday_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| {
                    Ok(BeatYesterdayProfile {
                        student_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        current_stage: row.get(2)?,
                        current_mode: row.get(3)?,
                        momentum_score: row.get(4)?,
                        strain_score: row.get(5)?,
                        readiness_score: row.get(6)?,
                        recovery_need_score: row.get(7)?,
                        streak_days: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn upsert_beat_yesterday_profile(
        &self,
        student_id: i64,
        subject_id: i64,
        current_stage: &str,
        current_mode: &str,
        momentum_score: BasisPoints,
        strain_score: BasisPoints,
        readiness_score: BasisPoints,
    ) -> EcoachResult<()> {
        let recovery_need_score = strain_score;
        self.conn
            .execute(
                "INSERT INTO beat_yesterday_profiles (
                student_id, subject_id, current_stage, current_mode, momentum_score, strain_score,
                readiness_score, recovery_need_score
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(student_id, subject_id) DO UPDATE SET
                current_stage = excluded.current_stage,
                current_mode = excluded.current_mode,
                momentum_score = excluded.momentum_score,
                strain_score = excluded.strain_score,
                readiness_score = excluded.readiness_score,
                recovery_need_score = excluded.recovery_need_score,
                updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    current_stage,
                    current_mode,
                    momentum_score,
                    strain_score,
                    readiness_score,
                    recovery_need_score,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn get_daily_climb_target(
        &self,
        student_id: i64,
        subject_id: i64,
        target_date: &str,
    ) -> EcoachResult<Option<BeatYesterdayDailyTarget>> {
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id, target_date, stage, mode, target_attempts,
                        target_correct, target_avg_response_time_ms, warm_start_minutes,
                        core_climb_minutes, speed_burst_minutes, finish_strong_minutes,
                        focus_topic_ids_json, rationale_json, status
                 FROM beat_yesterday_daily_targets
                 WHERE student_id = ?1 AND subject_id = ?2 AND target_date = ?3",
                params![student_id, subject_id, target_date],
                map_daily_target,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_daily_climb_target_by_id(
        &self,
        target_id: i64,
    ) -> EcoachResult<Option<BeatYesterdayDailyTarget>> {
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id, target_date, stage, mode, target_attempts,
                        target_correct, target_avg_response_time_ms, warm_start_minutes,
                        core_climb_minutes, speed_burst_minutes, finish_strong_minutes,
                        focus_topic_ids_json, rationale_json, status
                 FROM beat_yesterday_daily_targets
                 WHERE id = ?1",
                [target_id],
                map_daily_target,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_daily_climb_summary(
        &self,
        student_id: i64,
        subject_id: i64,
        summary_date: &str,
    ) -> EcoachResult<Option<BeatYesterdayDailySummary>> {
        self.conn
            .query_row(
                "SELECT id, target_id, student_id, subject_id, summary_date, actual_attempts,
                        actual_correct, actual_avg_response_time_ms, beat_attempt_target,
                        beat_accuracy_target, beat_pace_target, momentum_score, strain_score,
                        recovery_mode_triggered, summary_json
                 FROM beat_yesterday_daily_summaries
                 WHERE student_id = ?1 AND subject_id = ?2 AND summary_date = ?3",
                params![student_id, subject_id, summary_date],
                map_daily_summary,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_previous_daily_climb_summary(
        &self,
        student_id: i64,
        subject_id: i64,
        before_date: &str,
    ) -> EcoachResult<Option<BeatYesterdayDailySummary>> {
        self.conn
            .query_row(
                "SELECT id, target_id, student_id, subject_id, summary_date, actual_attempts,
                        actual_correct, actual_avg_response_time_ms, beat_attempt_target,
                        beat_accuracy_target, beat_pace_target, momentum_score, strain_score,
                        recovery_mode_triggered, summary_json
                 FROM beat_yesterday_daily_summaries
                 WHERE student_id = ?1 AND subject_id = ?2 AND summary_date < ?3
                 ORDER BY summary_date DESC
                 LIMIT 1",
                params![student_id, subject_id, before_date],
                map_daily_summary,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_subject_readiness(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let readiness: i64 = self
            .conn
            .query_row(
                "SELECT CAST(COALESCE(AVG(sts.mastery_score), 0) AS INTEGER)
             FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1 AND t.subject_id = ?2",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(readiness.clamp(0, 10_000) as BasisPoints)
    }

    fn load_focus_topic_ids(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id
             FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1 AND t.subject_id = ?2
             ORDER BY
                sts.is_urgent DESC,
                sts.repair_priority DESC,
                sts.priority_score DESC,
                sts.gap_score DESC,
                sts.mastery_score ASC
             LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                row.get(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_ids = Vec::new();
        for row in rows {
            topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(topic_ids)
    }

    fn load_recent_performance_baseline(
        &self,
        student_id: i64,
        subject_id: i64,
        before_date: &str,
    ) -> EcoachResult<SessionPerformanceBaseline> {
        if let Some(summary) =
            self.get_previous_daily_climb_summary(student_id, subject_id, before_date)?
        {
            let attempts = summary.actual_attempts.max(1);
            let accuracy_score = if attempts > 0 {
                to_bp(summary.actual_correct as f64 / attempts as f64)
            } else {
                0
            };
            return Ok(SessionPerformanceBaseline {
                attempts,
                correct: summary.actual_correct,
                accuracy_score,
                avg_response_time_ms: summary.actual_avg_response_time_ms,
                strain_score: summary.strain_score,
            });
        }

        let baseline = self.conn.query_row(
            "SELECT
                COALESCE(SUM(answered_questions), 0),
                COALESCE(SUM(correct_questions), 0),
                CASE
                    WHEN COALESCE(SUM(answered_questions), 0) > 0
                    THEN CAST(SUM(COALESCE(avg_response_time_ms, 0) * answered_questions) AS INTEGER) / SUM(answered_questions)
                    ELSE NULL
                END
             FROM sessions
             WHERE student_id = ?1
               AND subject_id = ?2
               AND status = 'completed'
               AND DATE(COALESCE(completed_at, created_at)) < DATE(?3)",
            params![student_id, subject_id, before_date],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;

        let attempts = baseline.0.max(1);
        let accuracy_score = to_bp(baseline.1 as f64 / attempts as f64);
        Ok(SessionPerformanceBaseline {
            attempts,
            correct: baseline.1,
            accuracy_score,
            avg_response_time_ms: baseline.2,
            strain_score: 0,
        })
    }

    fn load_actual_performance(
        &self,
        student_id: i64,
        subject_id: i64,
        summary_date: &str,
    ) -> EcoachResult<SessionPerformanceBaseline> {
        let actual = self.conn.query_row(
            "SELECT
                COALESCE(SUM(answered_questions), 0),
                COALESCE(SUM(correct_questions), 0),
                CASE
                    WHEN COALESCE(SUM(answered_questions), 0) > 0
                    THEN CAST(SUM(COALESCE(avg_response_time_ms, 0) * answered_questions) AS INTEGER) / SUM(answered_questions)
                    ELSE NULL
                END
             FROM sessions
             WHERE student_id = ?1
               AND subject_id = ?2
               AND status = 'completed'
               AND DATE(COALESCE(completed_at, created_at)) = DATE(?3)",
            params![student_id, subject_id, summary_date],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;

        let attempts = actual.0;
        let accuracy_score = if attempts > 0 {
            to_bp(actual.1 as f64 / attempts as f64)
        } else {
            0
        };
        Ok(SessionPerformanceBaseline {
            attempts,
            correct: actual.1,
            accuracy_score,
            avg_response_time_ms: actual.2,
            strain_score: 0,
        })
    }

    fn compute_beat_streak(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT beat_attempt_target, beat_accuracy_target, beat_pace_target
             FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY summary_date DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)? == 1,
                    row.get::<_, i64>(1)? == 1,
                    row.get::<_, i64>(2)? == 1,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut streak = 0;
        for row in rows {
            let (beat_attempt, beat_accuracy, beat_pace) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let achieved = [beat_attempt, beat_accuracy, beat_pace]
                .into_iter()
                .filter(|value| *value)
                .count()
                >= 2;
            if !achieved {
                break;
            }
            streak += 1;
        }
        Ok(streak)
    }

    fn collect_rows<T, F>(&self, rows: rusqlite::MappedRows<'_, F>) -> EcoachResult<Vec<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn resolve_anchor_date(&self, anchor_date: Option<&str>) -> EcoachResult<String> {
        match anchor_date {
            Some(date) => {
                NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map_err(|err| EcoachError::Validation(err.to_string()))?;
                Ok(date.to_string())
            }
            None => Ok(Utc::now().date_naive().format("%Y-%m-%d").to_string()),
        }
    }

    fn plan_anchor_date(&self, learner_id: i64) -> EcoachResult<String> {
        let plan_date = self
            .conn
            .query_row(
                "SELECT date
                 FROM coach_plan_days cpd
                 INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                 WHERE cp.student_id = ?1
                   AND cp.status IN ('active', 'draft')
                 ORDER BY CASE cpd.status WHEN 'active' THEN 0 ELSE 1 END, cpd.date ASC
                 LIMIT 1",
                [learner_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        match plan_date {
            Some(date) => self.resolve_anchor_date(Some(&date)),
            None => self.resolve_anchor_date(None),
        }
    }

    fn get_academic_event(
        &self,
        student_id: i64,
        event_id: i64,
        anchor_date: Option<&str>,
    ) -> EcoachResult<Option<AcademicCalendarEvent>> {
        let anchor_date = self.resolve_anchor_date(anchor_date)?;
        let events = self.load_academic_events(student_id, Some(&anchor_date), true)?;
        Ok(events.into_iter().find(|event| event.id == event_id))
    }

    fn load_academic_events(
        &self,
        student_id: i64,
        anchor_date: Option<&str>,
        include_completed: bool,
    ) -> EcoachResult<Vec<AcademicCalendarEvent>> {
        let anchor_date = self.resolve_anchor_date(anchor_date)?;
        let mut statement = self
            .conn
            .prepare(
                "SELECT ace.id, ace.student_id, ace.legacy_calendar_event_id, ace.title, ace.event_type,
                        ace.subject_id, ace.scheduled_date, ace.start_time, ace.end_time, ace.term,
                        ace.academic_year, ace.importance_bp, ace.scope, ace.linked_topic_ids_json,
                        ace.preparation_window_days, ace.review_window_days, ace.status,
                        ace.result_after_event, ace.coach_priority_weight_bp, ace.expected_weight_bp,
                        ace.timed_performance_weight_bp, ace.coverage_mode, ace.source,
                        ace.last_strategy_snapshot_json, s.name
                 FROM academic_calendar_events ace
                 LEFT JOIN subjects s ON s.id = ace.subject_id
                 WHERE ace.student_id = ?1
                   AND (?2 = 1 OR ace.status IN ('scheduled', 'postponed'))
                 ORDER BY ace.scheduled_date ASC, COALESCE(ace.start_time, '23:59') ASC, ace.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, bool_to_i64(include_completed)],
                map_academic_calendar_event,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut events = self.collect_rows(rows)?;
        for event in &mut events {
            let snapshot = self.compute_event_readiness(student_id, event, &anchor_date)?;
            let event_date = NaiveDate::parse_from_str(&event.scheduled_date, "%Y-%m-%d")
                .map_err(|err| EcoachError::Validation(err.to_string()))?;
            let anchor = NaiveDate::parse_from_str(&anchor_date, "%Y-%m-%d")
                .map_err(|err| EcoachError::Validation(err.to_string()))?;
            event.days_to_event = Some((event_date - anchor).num_days());
            event.readiness_score = Some(snapshot.readiness_score);
            event.subject_risk_level = Some(snapshot.subject_risk_level.clone());
            event.urgency_level = Some(snapshot.urgency_level.clone());
            event.recommended_mode = Some(snapshot.recommended_mode.clone());
            event.revision_density = Some(snapshot.revision_density.clone());
            event.mission_priority_influence = Some(snapshot.mission_priority_influence);
            event.strategy_snapshot = json!({
                "phase": snapshot.phase,
                "available_study_days": snapshot.available_study_days,
                "gap_score": snapshot.gap_score,
                "subject_risk_level": snapshot.subject_risk_level,
                "urgency_level": snapshot.urgency_level,
                "recommended_mode": snapshot.recommended_mode,
                "revision_density": snapshot.revision_density,
                "mission_priority_influence": snapshot.mission_priority_influence,
                "tone": snapshot.tone,
                "rationale": snapshot.rationale,
            });
        }
        Ok(events)
    }

    fn compute_event_readiness(
        &self,
        student_id: i64,
        event: &AcademicCalendarEvent,
        anchor_date: &str,
    ) -> EcoachResult<EventReadinessSnapshot> {
        let event_date = NaiveDate::parse_from_str(&event.scheduled_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let anchor = NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let days_to_event = (event_date - anchor).num_days();
        let available_study_days = (days_to_event + 1
            - self.count_blocked_days(student_id, anchor_date, &event.scheduled_date)?)
        .max(0);

        let metrics = if !event.linked_topic_ids.is_empty() {
            let topic_ids_json = serde_json::to_string(&event.linked_topic_ids)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
            self.conn
                .query_row(
                    "SELECT
                        CAST(COALESCE(AVG(sts.mastery_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.fragility_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.speed_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.transfer_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.confidence_score), 0) AS INTEGER)
                     FROM student_topic_states sts
                     WHERE sts.student_id = ?1
                       AND EXISTS (
                            SELECT 1
                            FROM json_each(?2) jt
                            WHERE jt.value = sts.topic_id
                       )",
                    params![student_id, topic_ids_json],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, i64>(4)?,
                        ))
                    },
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else if let Some(subject_id) = event.subject_id {
            self.conn
                .query_row(
                    "SELECT
                        CAST(COALESCE(AVG(sts.mastery_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.fragility_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.speed_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.transfer_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sts.confidence_score), 0) AS INTEGER)
                     FROM student_topic_states sts
                     INNER JOIN topics t ON t.id = sts.topic_id
                     WHERE sts.student_id = ?1
                       AND t.subject_id = ?2",
                    params![student_id, subject_id],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, i64>(4)?,
                        ))
                    },
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else {
            self.conn
                .query_row(
                    "SELECT
                        CAST(COALESCE(AVG(mastery_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(fragility_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(speed_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(transfer_score), 0) AS INTEGER),
                        CAST(COALESCE(AVG(confidence_score), 0) AS INTEGER)
                     FROM student_topic_states
                     WHERE student_id = ?1",
                    [student_id],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, i64>(4)?,
                        ))
                    },
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        };

        let readiness_score = clamp_bp(
            ((metrics.0 as f64 * 0.35)
                + (metrics.2 as f64 * 0.15)
                + (metrics.3 as f64 * 0.20)
                + (metrics.4 as f64 * 0.10)
                + ((10_000 - metrics.1) as f64 * 0.20))
                .round() as i64,
        );
        let required_readiness =
            required_readiness_for_event(&event.event_type, event.importance_bp);
        let gap_score = clamp_bp(required_readiness.saturating_sub(readiness_score as i64));

        let phase = if days_to_event <= 1 {
            "performance"
        } else if days_to_event <= 3 {
            "wrap_up"
        } else if days_to_event <= 7 {
            "firming_up"
        } else if days_to_event <= 21 {
            "strengthen"
        } else {
            "build"
        }
        .to_string();

        let subject_risk_level = if gap_score >= 6000 || metrics.1 >= 6500 {
            "high"
        } else if gap_score >= 3500 || metrics.1 >= 4500 {
            "medium"
        } else {
            "low"
        }
        .to_string();
        let urgency_level = if days_to_event <= 1 {
            "immediate"
        } else if days_to_event <= 3 {
            "urgent"
        } else if days_to_event <= 7 {
            "high"
        } else if days_to_event <= 21 {
            "rising"
        } else {
            "steady"
        }
        .to_string();
        let recommended_mode = match phase.as_str() {
            "build" => "concept_build",
            "strengthen" => "targeted_repair",
            "firming_up" => "mixed_retrieval",
            "wrap_up" => "exam_wrap_up",
            _ => "performance_readiness",
        }
        .to_string();
        let revision_density = match phase.as_str() {
            "build" => "light",
            "strengthen" => "medium",
            "firming_up" => "high",
            "wrap_up" => "dense",
            _ => "precision_only",
        }
        .to_string();

        let mission_priority_influence = clamp_bp(
            ((event.importance_bp as i64 * 35 / 100)
                + (event.coach_priority_weight_bp as i64 * 25 / 100)
                + (gap_score as i64 * 25 / 100)
                + proximity_pressure_bp(days_to_event) as i64 * 15 / 100)
                .clamp(0, 10_000),
        );
        let (
            explanation_weight_bp,
            retrieval_weight_bp,
            timed_drill_weight_bp,
            breadth_weight_bp,
            tone,
        ) = match phase.as_str() {
            "build" => (4200, 2800, 1200, 6400, "calm_foundation"),
            "strengthen" => (2600, 4200, 1800, 5200, "steady_pressure"),
            "firming_up" => (1800, 4400, 2800, 3800, "confident_compaction"),
            "wrap_up" => (1200, 3600, 3600, 2600, "focused_finish"),
            _ => (800, 3200, 4200, 2000, "calm_performance"),
        };
        let rationale = vec![
            format!(
                "{} day(s) remain until {}.",
                days_to_event.max(0),
                event.title
            ),
            format!(
                "Current readiness is {} with a gap score of {}.",
                readiness_score, gap_score
            ),
            format!(
                "The event matters at {} importance, so mission priority is {}.",
                event.importance_bp, mission_priority_influence
            ),
        ];

        Ok(EventReadinessSnapshot {
            phase,
            available_study_days,
            readiness_score,
            gap_score,
            subject_risk_level,
            urgency_level,
            recommended_mode,
            revision_density,
            mission_priority_influence,
            explanation_weight_bp,
            retrieval_weight_bp,
            timed_drill_weight_bp,
            breadth_weight_bp,
            tone: tone.to_string(),
            rationale,
        })
    }

    fn count_blocked_days(
        &self,
        student_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM availability_exceptions
                 WHERE student_id = ?1
                   AND availability_mode = 'blocked'
                   AND exception_date BETWEEN ?2 AND ?3",
                params![student_id, start_date, end_date],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn capture_strategy_snapshot(&self, learner_id: i64) -> EcoachResult<Value> {
        let plan_snapshot = self
            .conn
            .query_row(
                "SELECT id, current_phase, daily_budget_minutes, status
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'draft')
                 ORDER BY created_at DESC
                 LIMIT 1",
                [learner_id],
                |row| {
                    Ok(json!({
                        "plan_id": row.get::<_, i64>(0)?,
                        "current_phase": row.get::<_, String>(1)?,
                        "daily_budget_minutes": row.get::<_, i64>(2)?,
                        "status": row.get::<_, String>(3)?,
                    }))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or_else(|| json!({}));
        let momentum_snapshot = self
            .get_student_momentum(learner_id)?
            .map(|momentum| {
                json!({
                    "momentum_state": momentum.momentum_state,
                    "dropout_risk_bp": momentum.dropout_risk_bp,
                    "current_streak_days": momentum.current_streak_days,
                })
            })
            .unwrap_or_else(|| json!({}));
        Ok(json!({
            "plan": plan_snapshot,
            "momentum": momentum_snapshot,
        }))
    }

    fn apply_intensity_to_active_plan(
        &self,
        learner_id: i64,
        profile: &PreparationIntensityProfile,
    ) -> EcoachResult<()> {
        let Some((plan_id, daily_budget_minutes)) = self
            .conn
            .query_row(
                "SELECT id, daily_budget_minutes
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'draft')
                 ORDER BY created_at DESC
                 LIMIT 1",
                [learner_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        else {
            return Ok(());
        };
        let mapped_phase = map_preparation_phase_to_plan_phase(&profile.phase);
        let multiplier = match profile.revision_density.as_str() {
            "light" => 1.0,
            "medium" => 1.1,
            "high" => 1.2,
            "dense" => 1.3,
            _ => 0.95,
        };
        let new_budget = ((daily_budget_minutes as f64) * multiplier).round() as i64;
        self.conn
            .execute(
                "UPDATE coach_plans
                 SET current_phase = ?2,
                     daily_budget_minutes = ?3,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![plan_id, mapped_phase, new_budget.clamp(20, 240)],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE coach_plan_days
                 SET phase = ?2,
                     target_minutes = CAST(ROUND(target_minutes * ?3) AS INTEGER)
                 WHERE plan_id = ?1
                   AND status = 'pending'
                   AND DATE(date) >= DATE('now')
                   AND DATE(date) <= DATE('now', '+7 day')",
                params![plan_id, profile.phase, multiplier],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn record_strategy_adjustment(
        &self,
        learner_id: i64,
        reason: &str,
        source: &str,
        old_snapshot: Value,
        new_snapshot: Value,
        visible_message_student: Option<String>,
        visible_message_parent: Option<String>,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO strategy_adjustment_logs (
                    learner_id, reason, source, old_strategy_snapshot_json,
                    new_strategy_snapshot_json, visible_message_student, visible_message_parent
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    learner_id,
                    reason,
                    source,
                    old_snapshot.to_string(),
                    new_snapshot.to_string(),
                    visible_message_student,
                    visible_message_parent,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn emit_runtime_event(
        &self,
        event_type: &str,
        aggregate_kind: &str,
        aggregate_id: String,
        payload: Value,
    ) -> EcoachResult<()> {
        let now = Utc::now().to_rfc3339();
        let event_id = format!("{}-{}", event_type, Utc::now().timestamp_micros());
        let trace_id = format!("trace-{}", Utc::now().timestamp_millis());
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event_id,
                    event_type,
                    aggregate_kind,
                    aggregate_id,
                    trace_id,
                    payload.to_string(),
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn get_reminder_by_id(&self, reminder_id: i64) -> EcoachResult<Option<ReminderSchedule>> {
        self.conn
            .query_row(
                "SELECT id, learner_id, mission_id, academic_event_id, reminder_type,
                        scheduled_time, audience, status, escalation_level, metadata_json,
                        message, sent_at, acknowledged_at, created_at
                 FROM reminder_schedules
                 WHERE id = ?1",
                [reminder_id],
                map_reminder_schedule,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn build_reminder_message(
        &self,
        learner_id: i64,
        input: &ReminderScheduleInput,
    ) -> EcoachResult<String> {
        if let Some(mission_id) = input.mission_id {
            let mission = self
                .conn
                .query_row(
                    "SELECT title FROM coach_missions WHERE id = ?1 AND student_id = ?2",
                    params![mission_id, learner_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(mission_title) = mission {
                return Ok(format!("Coach session due soon: {}", mission_title));
            }
        }
        if let Some(event_id) = input.academic_event_id {
            let event = self
                .conn
                .query_row(
                    "SELECT title FROM academic_calendar_events WHERE id = ?1 AND student_id = ?2",
                    params![event_id, learner_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(event_title) = event {
                return Ok(format!("Upcoming academic priority: {}", event_title));
            }
        }
        Ok("Your coach has a time-sensitive academic reminder for you.".to_string())
    }

    fn get_engagement_event_by_id(&self, event_id: i64) -> EcoachResult<Option<EngagementEvent>> {
        self.conn
            .query_row(
                "SELECT id, learner_id, mission_id, session_id, reminder_schedule_id, academic_event_id,
                        session_state, started_at, ended_at, completion_percent, missed_reason,
                        source, created_at
                 FROM engagement_events
                 WHERE id = ?1",
                [event_id],
                map_engagement_event,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn sync_engagement_risk_profile(&self, learner_id: i64) -> EcoachResult<EngagementRiskProfile> {
        let (consecutive_misses, partials, last_state) = self
            .conn
            .query_row(
                "SELECT
                    COALESCE((
                        SELECT COUNT(*)
                        FROM (
                            SELECT session_state
                            FROM engagement_events
                            WHERE learner_id = ?1
                            ORDER BY created_at DESC
                            LIMIT 5
                        ) recent
                        WHERE session_state = 'missed'
                    ), 0),
                    COALESCE((
                        SELECT COUNT(*)
                        FROM engagement_events
                        WHERE learner_id = ?1
                          AND session_state = 'partially_completed'
                          AND DATE(created_at) >= DATE('now', '-14 day')
                    ), 0),
                    (
                        SELECT session_state
                        FROM engagement_events
                        WHERE learner_id = ?1
                        ORDER BY created_at DESC
                        LIMIT 1
                    )",
                [learner_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let risk_score_bp = clamp_bp(
            (consecutive_misses * 2500
                + partials * 900
                + if last_state.as_deref() == Some("missed") {
                    1500
                } else {
                    0
                })
            .clamp(0, 10_000),
        );
        let risk_level = if risk_score_bp >= 8500 {
            "critical"
        } else if risk_score_bp >= 6500 {
            "high"
        } else if risk_score_bp >= 3500 {
            "medium"
        } else {
            "low"
        };
        let next_recovery_action = if consecutive_misses >= 2 {
            Some("shorten_plan_and_reschedule".to_string())
        } else if partials >= 2 {
            Some("reduce_session_size".to_string())
        } else {
            Some("keep_standard_support".to_string())
        };
        self.conn
            .execute(
                "INSERT INTO engagement_risk_profiles (
                    learner_id, risk_level, risk_score_bp, consecutive_misses,
                    recent_partial_sessions, last_session_state, next_recovery_action
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(learner_id) DO UPDATE SET
                    risk_level = excluded.risk_level,
                    risk_score_bp = excluded.risk_score_bp,
                    consecutive_misses = excluded.consecutive_misses,
                    recent_partial_sessions = excluded.recent_partial_sessions,
                    last_session_state = excluded.last_session_state,
                    next_recovery_action = excluded.next_recovery_action,
                    updated_at = datetime('now')",
                params![
                    learner_id,
                    risk_level,
                    risk_score_bp,
                    consecutive_misses,
                    partials,
                    last_state,
                    next_recovery_action,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_engagement_risk_profile(learner_id)?
            .ok_or_else(|| {
                EcoachError::NotFound("Engagement risk profile missing after upsert".to_string())
            })
    }

    fn compute_parent_alert_severity(
        &self,
        learner_id: i64,
        session_state: &str,
        mission_id: Option<i64>,
        academic_event_id: Option<i64>,
        consecutive_misses: i64,
    ) -> EcoachResult<String> {
        let critical_mission = if let Some(mission_id) = mission_id {
            self.conn
                .query_row(
                    "SELECT CASE
                        WHEN activity_type IN ('repair', 'pressure_conditioning') OR target_minutes >= 25
                        THEN 1 ELSE 0 END
                     FROM coach_missions
                     WHERE id = ?1",
                    [mission_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or(0)
                == 1
        } else {
            false
        };
        let urgent_event = if let Some(event_id) = academic_event_id {
            let event_date: Option<String> = self
                .conn
                .query_row(
                    "SELECT scheduled_date FROM academic_calendar_events WHERE id = ?1 AND student_id = ?2",
                    params![event_id, learner_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            event_date
                .as_deref()
                .and_then(|raw| NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok())
                .map(|date| (date - Utc::now().date_naive()).num_days() <= 7)
                .unwrap_or(false)
        } else {
            false
        };
        let severity = if session_state == "missed"
            && (critical_mission || urgent_event || consecutive_misses >= 3)
        {
            "urgent"
        } else if session_state == "missed" && (consecutive_misses >= 2 || critical_mission) {
            "high"
        } else if session_state == "partially_completed" || session_state == "rescheduled" {
            "watch"
        } else {
            "info"
        };
        Ok(severity.to_string())
    }

    fn schedule_recovery_mission(
        &self,
        learner_id: i64,
        mission_id: Option<i64>,
        event: &EngagementEvent,
    ) -> EcoachResult<()> {
        let Some(mission_id) = mission_id else {
            return Ok(());
        };
        let mission = self
            .conn
            .query_row(
                "SELECT title, subject_id, primary_topic_id, target_minutes
                 FROM coach_missions
                 WHERE id = ?1 AND student_id = ?2",
                params![mission_id, learner_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((title, subject_id, topic_id, target_minutes)) = mission else {
            return Ok(());
        };
        self.conn
            .execute(
                "INSERT INTO coach_missions (
                    plan_day_id, student_id, title, reason, subject_id, primary_topic_id,
                    activity_type, target_minutes, status, steps_json, success_criteria_json
                 ) VALUES (NULL, ?1, ?2, ?3, ?4, ?5, 'repair', ?6, 'pending', ?7, ?8)",
                params![
                    learner_id,
                    format!("Recovery: {}", title),
                    format!(
                        "Added after a {} session to protect continuity.",
                        event.session_state
                    ),
                    subject_id,
                    topic_id,
                    target_minutes.min(18).max(10),
                    json!(["restart", "guided_repair", "confirm"]).to_string(),
                    json!({ "recover_from": mission_id }).to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn student_engagement_message(&self, session_state: &str) -> String {
        match session_state {
            "missed" => {
                "You missed a planned session, so I added a shorter recovery block.".to_string()
            }
            "partially_completed" => {
                "TodayÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢s session only partly landed, so I trimmed the next step to keep momentum."
                    .to_string()
            }
            "rescheduled" => {
                "The plan moved, so I rebalanced the next study step without dropping the goal."
                    .to_string()
            }
            _ => "The coach updated your plan based on engagement changes.".to_string(),
        }
    }

    fn parent_engagement_message(&self, session_state: &str) -> String {
        match session_state {
            "missed" => {
                "A scheduled coach session was missed, and the plan has been adjusted.".to_string()
            }
            "partially_completed" => {
                "A session was only partly completed, so the coach reduced the next load."
                    .to_string()
            }
            "rescheduled" => {
                "A session moved, and the coach has rebalanced the short-term plan.".to_string()
            }
            _ => "The coach updated the plan after an engagement change.".to_string(),
        }
    }

    fn create_parent_alerts_for_engagement(
        &self,
        learner_id: i64,
        event: &EngagementEvent,
        severity: &str,
        strategy_adjustment_id: Option<i64>,
    ) -> EcoachResult<()> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT psl.parent_account_id
                 FROM parent_student_links psl
                 LEFT JOIN parent_access_settings pas
                   ON pas.parent_id = psl.parent_account_id
                  AND pas.learner_id = psl.student_account_id
                 WHERE psl.student_account_id = ?1
                   AND COALESCE(pas.alerts_enabled, 1) = 1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([learner_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let parent_ids = self.collect_rows(rows)?;
        if parent_ids.is_empty() {
            return Ok(());
        }
        for parent_id in parent_ids {
            self.conn
                .execute(
                    "INSERT INTO parent_alert_records (
                        learner_id, parent_id, trigger_type, severity, message, action_required, metadata_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        learner_id,
                        parent_id,
                        format!("session_{}", event.session_state),
                        severity,
                        self.parent_engagement_message(&event.session_state),
                        if event.session_state == "missed" {
                            Some("Help the learner protect the recovery slot tonight.".to_string())
                        } else {
                            Some("Review the adjusted plan in Parent Overview.".to_string())
                        },
                        json!({
                            "engagement_event_id": event.id,
                            "mission_id": event.mission_id,
                            "strategy_adjustment_id": strategy_adjustment_id,
                        })
                        .to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn ensure_parent_link(&self, parent_id: i64, learner_id: i64) -> EcoachResult<()> {
        let linked = self
            .conn
            .query_row(
                "SELECT 1
                 FROM parent_student_links
                 WHERE parent_account_id = ?1 AND student_account_id = ?2",
                params![parent_id, learner_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .is_some();
        if !linked {
            return Err(EcoachError::Validation(
                "Parent is not linked to the learner".to_string(),
            ));
        }
        Ok(())
    }

    fn apply_feedback_to_active_plan(
        &self,
        learner_id: i64,
        interpreted_signal: &str,
    ) -> EcoachResult<()> {
        let adjustment_factor = match interpreted_signal {
            "fatigue_load" | "wellbeing_pressure" => 0.8,
            "schedule_conflict" | "logistics_conflict" => 0.9,
            "motivation_drop" => 0.85,
            "confidence_drop" => 0.9,
            _ => 1.0,
        };
        self.conn
            .execute(
                "UPDATE coach_missions
                 SET target_minutes = CAST(ROUND(target_minutes * ?2) AS INTEGER)
                 WHERE student_id = ?1
                   AND status = 'pending'
                   AND id IN (
                        SELECT id FROM coach_missions
                        WHERE student_id = ?1 AND status = 'pending'
                        ORDER BY created_at ASC
                        LIMIT 4
                   )",
                params![learner_id, adjustment_factor],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE coach_plans
                 SET daily_budget_minutes = CAST(ROUND(daily_budget_minutes * ?2) AS INTEGER),
                     updated_at = datetime('now')
                 WHERE student_id = ?1 AND status IN ('active', 'draft')",
                params![learner_id, adjustment_factor],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn get_parent_feedback_by_id(
        &self,
        feedback_id: i64,
    ) -> EcoachResult<Option<ParentFeedbackRecord>> {
        self.conn
            .query_row(
                "SELECT id, learner_id, parent_id, category, message, interpreted_signal, urgency,
                        suggested_support_action, visible_strategy_change, status, submitted_at, applied_at
                 FROM parent_feedback_records
                 WHERE id = ?1",
                [feedback_id],
                map_parent_feedback_record,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_parent_alert_by_id(&self, alert_id: i64) -> EcoachResult<Option<ParentAlertRecord>> {
        self.conn
            .query_row(
                "SELECT id, learner_id, parent_id, trigger_type, severity, message,
                        action_required, status, metadata_json, created_at, acknowledged_at, resolved_at
                 FROM parent_alert_records
                 WHERE id = ?1",
                [alert_id],
                map_parent_alert_record,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn refresh_title_states(&self, student_id: i64, anchor_date: &str) -> EcoachResult<()> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id, t.subject_id, t.name, s.name
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE sts.student_id = ?1
                   AND (
                        sts.mastery_score >= 6200
                        OR sts.transfer_score >= 6200
                        OR sts.speed_score >= 6200
                   )",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let candidates = self.collect_rows(rows)?;
        for (topic_id, subject_id, topic_name, _subject_name) in candidates {
            let eligibility = self.evaluate_title_eligibility(student_id, topic_id)?;
            let title_name = format!("{} Champion", topic_name);
            let existing = self
                .conn
                .query_row(
                    "SELECT id, state, next_defense_due_at
                     FROM coach_title_states
                     WHERE student_id = ?1 AND topic_id = ?2 AND title_name = ?3",
                    params![student_id, topic_id, title_name],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                        ))
                    },
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            match existing {
                None if eligibility.awardable => {
                    let next_defense_due = (NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
                        .map_err(|err| EcoachError::Validation(err.to_string()))?
                        + Duration::days(10))
                    .format("%Y-%m-%d")
                    .to_string();
                    let evidence = json!({
                        "readiness_score": eligibility.readiness_score,
                        "independent_successes": eligibility.independent_successes,
                        "timed_successes": eligibility.timed_successes,
                        "unresolved_misconceptions": eligibility.unresolved_misconceptions,
                    });
                    self.conn
                        .execute(
                            "INSERT INTO coach_title_states (
                                student_id, subject_id, topic_id, title_name, state,
                                earned_at, next_defense_due_at, coach_note, evidence_snapshot_json
                             ) VALUES (?1, ?2, ?3, ?4, 'active', datetime('now'), ?5, ?6, ?7)",
                            params![
                                student_id,
                                subject_id,
                                topic_id,
                                title_name,
                                next_defense_due,
                                "Mastery is holding strongly across more than one form.",
                                evidence.to_string(),
                            ],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    let title_id = self.conn.last_insert_rowid();
                    self.record_title_history(title_id, None, "active", "Title awarded", evidence)?;
                    self.emit_runtime_event(
                        "title_awarded",
                        "coach_title",
                        title_id.to_string(),
                        json!({
                            "titleId": title_id,
                            "titleName": title_name,
                            "topicId": topic_id,
                        }),
                    )?;
                    let _ = self.award_badge_if_due(
                        student_id,
                        Some(subject_id),
                        Some(topic_id),
                        "Title Earned",
                        "title",
                        &format!("Earned {}", title_name),
                        None,
                        Some(title_id),
                        json!({ "title_name": title_name }),
                    )?;
                }
                None if eligibility.candidate => {
                    let evidence = json!({
                        "readiness_score": eligibility.readiness_score,
                        "independent_successes": eligibility.independent_successes,
                        "timed_successes": eligibility.timed_successes,
                        "unresolved_misconceptions": eligibility.unresolved_misconceptions,
                    });
                    self.conn
                        .execute(
                            "INSERT INTO coach_title_states (
                                student_id, subject_id, topic_id, title_name, state,
                                coach_note, evidence_snapshot_json
                             ) VALUES (?1, ?2, ?3, ?4, 'candidate', ?5, ?6)",
                            params![
                                student_id,
                                subject_id,
                                topic_id,
                                title_name,
                                "The topic is close to title standard, but still needs confirmation.",
                                evidence.to_string(),
                            ],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    let title_id = self.conn.last_insert_rowid();
                    self.record_title_history(
                        title_id,
                        None,
                        "candidate",
                        "Title candidate surfaced",
                        evidence,
                    )?;
                }
                Some((title_id, state, next_due))
                    if state != "contested"
                        && state != "dormant"
                        && !eligibility.awardable
                        && !eligibility.candidate =>
                {
                    self.conn
                        .execute(
                            "UPDATE coach_title_states
                             SET state = 'contested',
                                 coach_note = ?2,
                                 updated_at = datetime('now')
                             WHERE id = ?1",
                            params![
                                title_id,
                                "Recent evidence shows the knowledge is no longer stable enough to keep this title secure.",
                            ],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    self.record_title_history(
                        title_id,
                        Some(state.as_str()),
                        "contested",
                        "Repeated slippage re-opened the title",
                        json!({ "readiness_score": eligibility.readiness_score }),
                    )?;
                    self.emit_runtime_event(
                        "title_contested",
                        "coach_title",
                        title_id.to_string(),
                        json!({
                            "titleId": title_id,
                            "titleName": title_name,
                        }),
                    )?;
                }
                Some((title_id, state, next_due))
                    if next_due.as_deref().is_some_and(|due| due <= anchor_date) =>
                {
                    if state != "defense_due" && state != "contested" {
                        self.conn
                            .execute(
                                "UPDATE coach_title_states
                                 SET state = 'defense_due', updated_at = datetime('now')
                                 WHERE id = ?1",
                                [title_id],
                            )
                            .map_err(|err| EcoachError::Storage(err.to_string()))?;
                        self.record_title_history(
                            title_id,
                            Some(state.as_str()),
                            "defense_due",
                            "Defense is now due",
                            json!({}),
                        )?;
                    }
                }
                Some((title_id, state, _)) if state == "candidate" && eligibility.awardable => {
                    let next_defense_due = (NaiveDate::parse_from_str(anchor_date, "%Y-%m-%d")
                        .map_err(|err| EcoachError::Validation(err.to_string()))?
                        + Duration::days(10))
                    .format("%Y-%m-%d")
                    .to_string();
                    self.conn
                        .execute(
                            "UPDATE coach_title_states
                             SET state = 'active',
                                 earned_at = datetime('now'),
                                 next_defense_due_at = ?2,
                                 coach_note = ?3,
                                 updated_at = datetime('now')
                             WHERE id = ?1",
                            params![
                                title_id,
                                next_defense_due,
                                "Confirmation held across forms, so the title is now active.",
                            ],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    self.record_title_history(
                        title_id,
                        Some("candidate"),
                        "active",
                        "Title confirmed",
                        json!({ "readiness_score": eligibility.readiness_score }),
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn evaluate_title_eligibility(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<TitleEligibilitySnapshot> {
        let state = self
            .conn
            .query_row(
                "SELECT mastery_score, speed_score, transfer_score, confidence_score,
                        fragility_score, pressure_collapse_index, total_attempts, correct_attempts
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let independent_successes: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1
                   AND q.topic_id = ?2
                   AND COALESCE(sqa.is_correct, 0) = 1
                   AND sqa.support_level = 'independent'",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let timed_successes: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1
                   AND q.topic_id = ?2
                   AND COALESCE(sqa.is_correct, 0) = 1
                   AND sqa.was_timed = 1",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let unresolved_misconceptions: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM student_error_profiles
                 WHERE student_id = ?1
                   AND topic_id = ?2
                   AND (
                        knowledge_gap_score >= 5000
                        OR conceptual_confusion_score >= 5000
                        OR recognition_failure_score >= 5000
                        OR execution_error_score >= 5000
                        OR pressure_breakdown_score >= 5000
                   )",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let readiness_score = clamp_bp(
            ((state.0 as f64 * 0.35)
                + (state.1 as f64 * 0.15)
                + (state.2 as f64 * 0.20)
                + (state.3 as f64 * 0.10)
                + ((10_000 - state.4) as f64 * 0.10)
                + ((10_000 - state.5) as f64 * 0.10))
                .round() as i64,
        );
        let accuracy_bp = if state.6 > 0 {
            to_bp(state.7 as f64 / state.6.max(1) as f64)
        } else {
            0
        };
        let candidate = readiness_score >= 6800
            && accuracy_bp >= 7000
            && independent_successes >= 2
            && unresolved_misconceptions <= 1;
        let awardable = readiness_score >= 7800
            && accuracy_bp >= 7800
            && independent_successes >= 2
            && timed_successes >= 1
            && unresolved_misconceptions == 0;
        Ok(TitleEligibilitySnapshot {
            readiness_score,
            independent_successes,
            timed_successes,
            unresolved_misconceptions,
            candidate,
            awardable,
        })
    }

    fn refresh_badges(&self, student_id: i64) -> EcoachResult<()> {
        let _ = self.award_fast_and_accurate_badge(student_id)?;
        let _ = self.award_discipline_badge(student_id)?;
        let _ = self.award_recovery_badge(student_id)?;
        Ok(())
    }

    fn load_titles_hall(&self, student_id: i64) -> EcoachResult<TitlesHallSnapshot> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT cts.id, cts.student_id, cts.subject_id, cts.topic_id, cts.title_name,
                        cts.state, cts.earned_at, cts.last_defended_at, cts.next_defense_due_at,
                        cts.coach_note, cts.evidence_snapshot_json, cts.reclaim_plan_json,
                        s.name, t.name
                 FROM coach_title_states cts
                 LEFT JOIN subjects s ON s.id = cts.subject_id
                 LEFT JOIN topics t ON t.id = cts.topic_id
                 WHERE cts.student_id = ?1
                 ORDER BY cts.updated_at DESC, cts.id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], map_coach_title_card)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let titles = self.collect_rows(rows)?;

        let mut badge_statement = self
            .conn
            .prepare(
                "SELECT cba.id, cba.student_id, cba.subject_id, cba.topic_id, cba.badge_name,
                        cba.badge_family, cba.reason, cba.related_session_id, cba.related_title_state_id,
                        cba.metadata_json, s.name, t.name, cba.awarded_at
                 FROM coach_badge_awards cba
                 LEFT JOIN subjects s ON s.id = cba.subject_id
                 LEFT JOIN topics t ON t.id = cba.topic_id
                 WHERE cba.student_id = ?1
                 ORDER BY cba.awarded_at DESC, cba.id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let badge_rows = badge_statement
            .query_map([student_id], map_coach_badge_award)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let badges = self.collect_rows(badge_rows)?;

        Ok(TitlesHallSnapshot {
            student_id,
            generated_at: Utc::now().to_rfc3339(),
            active_titles: titles
                .iter()
                .filter(|title| {
                    matches!(
                        title.state.as_str(),
                        "active" | "earned" | "defended" | "narrowly_defended"
                    )
                })
                .cloned()
                .collect(),
            defenses_due: titles
                .iter()
                .filter(|title| title.state == "defense_due" || title.state == "narrowly_defended")
                .cloned()
                .collect(),
            contested_titles: titles
                .iter()
                .filter(|title| title.state == "contested")
                .cloned()
                .collect(),
            reclaimed_titles: titles
                .iter()
                .filter(|title| title.state == "reclaimed")
                .cloned()
                .collect(),
            badges,
        })
    }

    fn get_title_by_id(&self, title_id: i64) -> EcoachResult<Option<CoachTitleCard>> {
        self.conn
            .query_row(
                "SELECT cts.id, cts.student_id, cts.subject_id, cts.topic_id, cts.title_name,
                        cts.state, cts.earned_at, cts.last_defended_at, cts.next_defense_due_at,
                        cts.coach_note, cts.evidence_snapshot_json, cts.reclaim_plan_json,
                        s.name, t.name
                 FROM coach_title_states cts
                 LEFT JOIN subjects s ON s.id = cts.subject_id
                 LEFT JOIN topics t ON t.id = cts.topic_id
                 WHERE cts.id = ?1",
                [title_id],
                map_coach_title_card,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn title_subject_topic(&self, title_id: i64) -> EcoachResult<(Option<i64>, Option<i64>)> {
        self.conn
            .query_row(
                "SELECT subject_id, topic_id
                 FROM coach_title_states
                 WHERE id = ?1",
                [title_id],
                |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn schedule_title_follow_up_mission(
        &self,
        title_id: i64,
        student_id: i64,
        mission_name: &str,
    ) -> EcoachResult<()> {
        let title = self.get_title_by_id(title_id)?.ok_or_else(|| {
            EcoachError::NotFound("Title not found for follow-up mission".to_string())
        })?;
        self.conn
            .execute(
                "INSERT INTO coach_missions (
                    plan_day_id, student_id, title, reason, subject_id, primary_topic_id,
                    activity_type, target_minutes, status, steps_json, success_criteria_json
                 ) VALUES (NULL, ?1, ?2, ?3, ?4, ?5, 'repair', ?6, 'pending', ?7, ?8)",
                params![
                    student_id,
                    format!("{}: {}", mission_name, title.title_name),
                    "Added by the title engine after a defense result.",
                    title.subject_id,
                    title.topic_id,
                    if title.state == "contested" { 18 } else { 12 },
                    json!(["stabilize", "verify", "prepare_recheck"]).to_string(),
                    json!({ "title_id": title_id }).to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn award_badge_if_due(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        badge_name: &str,
        badge_family: &str,
        reason: &str,
        related_session_id: Option<i64>,
        related_title_state_id: Option<i64>,
        metadata: Value,
    ) -> EcoachResult<Option<CoachBadgeAward>> {
        let recent_duplicate: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM coach_badge_awards
                 WHERE student_id = ?1
                   AND badge_name = ?2
                   AND COALESCE(topic_id, -1) = COALESCE(?3, -1)
                   AND DATETIME(awarded_at) >= DATETIME('now', '-14 day')
                 ORDER BY awarded_at DESC
                 LIMIT 1",
                params![student_id, badge_name, topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if recent_duplicate.is_some() {
            return Ok(None);
        }
        self.conn
            .execute(
                "INSERT INTO coach_badge_awards (
                    student_id, subject_id, topic_id, badge_name, badge_family, reason,
                    related_session_id, related_title_state_id, metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    student_id,
                    subject_id,
                    topic_id,
                    badge_name,
                    badge_family,
                    reason,
                    related_session_id,
                    related_title_state_id,
                    metadata.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let badge_id = self.conn.last_insert_rowid();
        self.emit_runtime_event(
            "badge_awarded",
            "learner",
            student_id.to_string(),
            json!({
                "badgeId": badge_id,
                "badgeName": badge_name,
                "badgeFamily": badge_family,
                "topicId": topic_id,
            }),
        )?;
        self.conn
            .query_row(
                "SELECT cba.id, cba.student_id, cba.subject_id, cba.topic_id, cba.badge_name,
                        cba.badge_family, cba.reason, cba.related_session_id, cba.related_title_state_id,
                        cba.metadata_json, s.name, t.name, cba.awarded_at
                 FROM coach_badge_awards cba
                 LEFT JOIN subjects s ON s.id = cba.subject_id
                 LEFT JOIN topics t ON t.id = cba.topic_id
                 WHERE cba.id = ?1",
                [badge_id],
                map_coach_badge_award,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn award_fast_and_accurate_badge(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<CoachBadgeAward>> {
        let candidate = self
            .conn
            .query_row(
                "SELECT t.subject_id, q.topic_id,
                        CAST(COALESCE(AVG(CASE WHEN sqa.is_correct = 1 THEN 10000 ELSE 0 END), 0) AS INTEGER),
                        CAST(COALESCE(AVG(sqa.response_time_ms), 30000) AS INTEGER)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 WHERE sqa.student_id = ?1
                   AND sqa.was_timed = 1
                   AND sqa.response_time_ms IS NOT NULL
                 GROUP BY q.topic_id, t.subject_id
                 HAVING COUNT(*) >= 3
                 ORDER BY 3 DESC, 4 ASC
                 LIMIT 1",
                [student_id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((subject_id, topic_id, accuracy_bp, avg_response_time_ms)) = candidate else {
            return Ok(None);
        };
        if accuracy_bp < 8000 || avg_response_time_ms > 22_000 {
            return Ok(None);
        }
        self.award_badge_if_due(
            student_id,
            subject_id,
            topic_id,
            "Fast and Accurate",
            "performance",
            "Sustained strong accuracy while staying quick under timed pressure.",
            None,
            None,
            json!({
                "accuracy_bp": accuracy_bp,
                "avg_response_time_ms": avg_response_time_ms,
            }),
        )
    }

    fn award_discipline_badge(&self, student_id: i64) -> EcoachResult<Option<CoachBadgeAward>> {
        let study_days = self.load_student_session_dates(student_id)?;
        let recent_days = study_days
            .into_iter()
            .filter(|date| *date >= Utc::now().date_naive() - Duration::days(6))
            .count();
        if recent_days < 5 {
            return Ok(None);
        }
        self.award_badge_if_due(
            student_id,
            None,
            None,
            "Weekly Discipline",
            "discipline",
            "Protected study rhythm across most days this week.",
            None,
            None,
            json!({ "study_days_last_7": recent_days }),
        )
    }

    fn award_recovery_badge(&self, student_id: i64) -> EcoachResult<Option<CoachBadgeAward>> {
        let recovered = self
            .conn
            .query_row(
                "SELECT id
                 FROM engagement_events
                 WHERE learner_id = ?1
                   AND session_state = 'completed'
                   AND EXISTS (
                        SELECT 1
                        FROM engagement_events prior
                        WHERE prior.learner_id = engagement_events.learner_id
                          AND prior.session_state = 'missed'
                          AND DATETIME(prior.created_at) >= DATETIME(engagement_events.created_at, '-3 day')
                   )
                 ORDER BY created_at DESC
                 LIMIT 1",
                [student_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(event_id) = recovered else {
            return Ok(None);
        };
        self.award_badge_if_due(
            student_id,
            None,
            None,
            "Recovery Win",
            "recovery",
            "Returned with a completed session after a recent miss.",
            None,
            None,
            json!({ "engagement_event_id": event_id }),
        )
    }

    fn record_title_history(
        &self,
        title_state_id: i64,
        previous_state: Option<&str>,
        new_state: &str,
        reason: &str,
        snapshot: Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO coach_title_history (
                    title_state_id, previous_state, new_state, reason, snapshot_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    title_state_id,
                    previous_state,
                    new_state,
                    reason,
                    snapshot.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn split_legacy_scheduled_for(scheduled_for: &str) -> (String, Option<String>) {
    if let Some((date, time)) = scheduled_for.split_once('T') {
        return (
            date.to_string(),
            Some(time.chars().take(5).collect::<String>()),
        );
    }
    if scheduled_for.len() >= 16 && scheduled_for.as_bytes().get(10) == Some(&b' ') {
        return (
            scheduled_for[..10].to_string(),
            Some(scheduled_for[11..16].to_string()),
        );
    }
    (scheduled_for.to_string(), None)
}

fn combine_date_and_time(date: &str, time: Option<&str>) -> String {
    time.map(|time| format!("{date}T{time}"))
        .unwrap_or_else(|| date.to_string())
}

fn normalize_academic_event_type(event_type: &str) -> String {
    match event_type {
        "exam" => "exam",
        "mock" => "mock",
        "class_test" => "class_test",
        "assignment" => "assignment",
        "milestone" => "milestone",
        "quiz" => "quiz",
        "final_exam" => "final_exam",
        "title_defense" => "title_defense",
        "project" => "project",
        "review_window" => "review_window",
        _ => "milestone",
    }
    .to_string()
}

fn legacy_event_importance(event_type: &str) -> BasisPoints {
    match event_type {
        "final_exam" => 9000,
        "exam" => 8400,
        "mock" => 7600,
        "class_test" => 6400,
        "assignment" => 5200,
        _ => 5000,
    }
}

fn required_readiness_for_event(event_type: &str, importance_bp: BasisPoints) -> i64 {
    let base = match event_type {
        "final_exam" => 8400,
        "exam" => 8000,
        "mock" => 7600,
        "class_test" => 7000,
        "quiz" => 6600,
        "assignment" | "project" => 6200,
        "title_defense" => 7800,
        _ => 6400,
    };
    (base + (importance_bp as i64 - 5000) / 6).clamp(0, 10_000)
}

fn proximity_pressure_bp(days_to_event: i64) -> BasisPoints {
    match days_to_event {
        i64::MIN..=1 => 10_000,
        2..=3 => 8800,
        4..=7 => 7400,
        8..=14 => 5600,
        15..=21 => 4200,
        _ => 2600,
    }
}

fn map_preparation_phase_to_plan_phase(phase: &str) -> &'static str {
    match phase {
        "build" => "foundation",
        "strengthen" => "strengthening",
        "firming_up" => "consolidation",
        "wrap_up" => "final_revision",
        "performance" => "performance",
        _ => "foundation",
    }
}

fn interpret_parent_feedback_signal(category: &str, message: &str) -> String {
    let message = message.to_ascii_lowercase();
    match category {
        "fatigue" => "fatigue_load",
        "wellbeing" => "wellbeing_pressure",
        "schedule" if message.contains("late") || message.contains("time") => "schedule_conflict",
        "logistics" => "logistics_conflict",
        "motivation" => "motivation_drop",
        "performance" => "confidence_drop",
        "behavior" if message.contains("avoid") || message.contains("resist") => {
            "avoidance_pattern"
        }
        _ if message.contains("tired") || message.contains("fatigue") => "fatigue_load",
        _ if message.contains("bus")
            || message.contains("transport")
            || message.contains("church") =>
        {
            "logistics_conflict"
        }
        _ if message.contains("worry") || message.contains("confidence") => "confidence_drop",
        _ => "support_context_update",
    }
    .to_string()
}

fn parent_support_action(signal: &str) -> String {
    match signal {
        "fatigue_load" | "wellbeing_pressure" => {
            "Protect an earlier, shorter session and lower pressure for the week.".to_string()
        }
        "schedule_conflict" | "logistics_conflict" => {
            "Confirm a more reliable study slot and preserve the recovery mission.".to_string()
        }
        "motivation_drop" => {
            "Use a short restart session and keep encouragement focused on completion.".to_string()
        }
        "confidence_drop" => {
            "Reduce pressure temporarily and support a guided confidence rebuild.".to_string()
        }
        _ => "Keep the learnerÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢s study routine visible and steady.".to_string(),
    }
}

fn strategy_change_from_feedback(signal: &str) -> String {
    match signal {
        "fatigue_load" | "wellbeing_pressure" => {
            "I reduced intensity and shortened the next few sessions.".to_string()
        }
        "schedule_conflict" | "logistics_conflict" => {
            "I tightened the plan around a more realistic study window.".to_string()
        }
        "motivation_drop" => {
            "I switched the next step into a lighter recovery-style mission.".to_string()
        }
        "confidence_drop" => {
            "I lowered timing pressure and added a steadier rebuild step.".to_string()
        }
        _ => "I updated the plan to reflect the new context.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{Duration, Utc};
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn free_now_recommendation_prioritizes_planned_mission() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_topic_state(&conn, student_id, topic_id);
        seed_baseline_session(&conn, student_id, subject_id, topic_id, "2026-03-29");
        service
            .generate_daily_climb_target(student_id, subject_id, date)
            .expect("daily target should generate");
        seed_pending_mission(&conn, student_id, subject_id, topic_id, date);

        let recommendation = service
            .recommend_free_now_session(student_id, subject_id, date, 19 * 60, 35)
            .expect("free-now recommendation should resolve");

        assert!(recommendation.available_now);
        assert_eq!(recommendation.session_type, "planned_repair");
        assert!(recommendation.suggested_duration_minutes >= 15);
        assert_eq!(
            recommendation.focus_topic_ids.first().copied(),
            Some(topic_id)
        );
    }

    #[test]
    fn daily_replan_surfaces_remaining_load_and_capacity() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_topic_state(&conn, student_id, topic_id);
        seed_baseline_session(&conn, student_id, subject_id, topic_id, "2026-03-29");
        service
            .generate_daily_climb_target(student_id, subject_id, date)
            .expect("daily target should generate");

        let replan = service
            .replan_remaining_day(student_id, subject_id, date, 19 * 60)
            .expect("daily replan should resolve");

        assert!(replan.available_now);
        assert!(replan.remaining_capacity_minutes > 0);
        assert!(replan.remaining_target_minutes > 0);
        assert!(replan.recommended_session_count >= 1);
        assert!(!replan.focus_topic_ids.is_empty());
    }

    #[test]
    fn free_now_recommendation_preempts_target_after_failed_solidification() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_topic_state(&conn, student_id, topic_id);
        seed_memory_state(
            &conn,
            student_id,
            topic_id,
            "collapsed",
            2200,
            8800,
            Some(date),
        );
        seed_baseline_session(&conn, student_id, subject_id, topic_id, "2026-03-29");
        service
            .generate_daily_climb_target(student_id, subject_id, date)
            .expect("daily target should generate");
        seed_solidification_outcome(
            &conn,
            student_id,
            subject_id,
            topic_id,
            "failed",
            "reteach_before_retry",
            2200,
            &format!("{date}T17:30:00Z"),
        );

        let recommendation = service
            .recommend_free_now_session(student_id, subject_id, date, 19 * 60, 35)
            .expect("free-now recommendation should resolve");

        assert!(recommendation.available_now);
        assert_eq!(recommendation.session_type, "comeback_reteach");
        assert_eq!(recommendation.recommended_comeback_topic_id, Some(topic_id));
        assert_eq!(
            recommendation.recent_repair_outcome.as_deref(),
            Some("failed")
        );
        assert_eq!(
            recommendation.focus_topic_ids.first().copied(),
            Some(topic_id)
        );
        assert!(recommendation.pressure_score >= 7000);
        assert!(recommendation.repair_buffer_minutes >= 12);
        assert!(recommendation.rationale.contains("did not hold"));
    }

    #[test]
    fn daily_replan_adds_comeback_buffer_after_failed_solidification() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_topic_state(&conn, student_id, topic_id);
        seed_memory_state(
            &conn,
            student_id,
            topic_id,
            "collapsed",
            2200,
            9000,
            Some(date),
        );
        seed_baseline_session(&conn, student_id, subject_id, topic_id, "2026-03-29");
        service
            .generate_daily_climb_target(student_id, subject_id, date)
            .expect("daily target should generate");

        let baseline_replan = service
            .replan_remaining_day(student_id, subject_id, date, 19 * 60)
            .expect("baseline replan should resolve");

        seed_solidification_outcome(
            &conn,
            student_id,
            subject_id,
            topic_id,
            "failed",
            "reteach_before_retry",
            1800,
            &format!("{date}T17:45:00Z"),
        );

        let replan = service
            .replan_remaining_day(student_id, subject_id, date, 19 * 60)
            .expect("daily replan should resolve");

        assert_eq!(replan.next_session_type, "comeback_reteach");
        assert_eq!(replan.recommended_comeback_topic_id, Some(topic_id));
        assert_eq!(replan.recent_repair_outcome.as_deref(), Some("failed"));
        assert!(replan.repair_buffer_minutes >= 12);
        assert!(replan.remaining_target_minutes > baseline_replan.remaining_target_minutes);
        assert!(replan.rationale.contains("did not hold"));
    }

    #[test]
    fn daily_replan_downshifts_to_retention_after_successful_solidification() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_stable_topic_state(&conn, student_id, topic_id);
        seed_memory_state(
            &conn,
            student_id,
            topic_id,
            "accessible",
            7600,
            2100,
            Some("2026-04-02"),
        );
        seed_solidification_outcome(
            &conn,
            student_id,
            subject_id,
            topic_id,
            "success",
            "stabilize_memory",
            8600,
            &format!("{date}T16:30:00Z"),
        );

        let replan = service
            .replan_remaining_day(student_id, subject_id, date, 19 * 60)
            .expect("daily replan should resolve");

        assert_eq!(replan.next_session_type, "retention_check");
        assert_eq!(replan.recommended_comeback_topic_id, Some(topic_id));
        assert_eq!(replan.recent_repair_outcome.as_deref(), Some("success"));
        assert_eq!(replan.repair_buffer_minutes, 6);
        assert_eq!(replan.remaining_target_minutes, 6);
        assert!(replan.rationale.contains("retention"));
    }

    #[test]
    fn time_orchestration_snapshot_builds_blocks_and_dispatches_triggers() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_availability(&service, student_id);
        seed_topic_state(&conn, student_id, topic_id);
        seed_baseline_session(&conn, student_id, subject_id, topic_id, "2026-03-29");

        let plan_state = service
            .sync_exam_plan_state(&ExamPlanStateInput {
                student_id,
                subject_id,
                anchor_date: "2026-03-30".to_string(),
                exam_date: "2026-04-20".to_string(),
                target_effective_minutes: 600,
                plan_mode: None,
                auto_trigger_mode: Some("hybrid".to_string()),
            })
            .expect("exam plan state should sync");
        let snapshot = service
            .build_time_orchestration_snapshot(
                student_id,
                subject_id,
                "2026-03-30",
                19 * 60,
                None,
                None,
                None,
                None,
                5,
            )
            .expect("time orchestration snapshot should build");

        assert_eq!(snapshot.plan_state.id, plan_state.id);
        assert_eq!(snapshot.plan_state.auto_trigger_mode, "hybrid");
        assert_eq!(snapshot.ledger.student_id, student_id);
        assert!(!snapshot.session_blocks.is_empty());
        assert!(!snapshot.trigger_jobs.is_empty());
        assert!(snapshot.trigger_jobs.iter().all(|job| matches!(
            job.trigger_kind.as_str(),
            "launch" | "reminder" | "hybrid_prompt"
        )));

        let due_jobs = service
            .dispatch_due_trigger_jobs("2026-03-30T20:00:00", Some(student_id), 10)
            .expect("due trigger jobs should dispatch");
        assert!(!due_jobs.is_empty());
        assert!(due_jobs.iter().all(|job| job.status == "fired"));
    }

    #[test]
    fn momentum_sync_and_comeback_flow_round_trip() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = GoalsCalendarService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        let today = Utc::now().date_naive();
        let session_dates = [today - Duration::days(2), today - Duration::days(1), today];

        for date in session_dates {
            seed_baseline_session(&conn, student_id, subject_id, topic_id, &date.to_string());
        }

        let momentum = service
            .sync_student_momentum(student_id)
            .expect("momentum should sync");
        assert_eq!(momentum.student_id, student_id);
        assert_eq!(momentum.current_streak_days, 3);
        assert_eq!(momentum.days_since_last_session, 0);
        assert_eq!(momentum.comeback_session_count, 0);
        assert!(matches!(
            momentum.momentum_state.as_str(),
            "building" | "slipping" | "strong"
        ));
        assert!(momentum.consistency_7d_bp >= 4_000);

        let flow = service
            .start_comeback_flow(student_id, "missed_days", 5)
            .expect("comeback flow should start");
        assert_eq!(flow.student_id, student_id);
        assert_eq!(flow.trigger_reason, "missed_days");
        assert_eq!(flow.status, "active");
        assert!(!flow.flow_steps.is_empty());
        let active_flow = service
            .get_active_comeback_flow(student_id)
            .expect("active flow should load")
            .expect("active flow should exist");
        assert_eq!(active_flow.id, flow.id);
        assert_eq!(active_flow.flow_steps, flow.flow_steps);

        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        conn.execute(
            "INSERT INTO revenge_queue (
                student_id, question_id, original_session_id, original_error_type,
                original_wrong_answer, attempts_to_beat, is_beaten
             ) VALUES (?1, ?2, ?3, 'misconception_triggered', 'B', 2, 0)",
            params![student_id, question_id, flow.id],
        )
        .expect("revenge queue item should insert");

        let revenge_queue = service
            .get_revenge_queue(student_id)
            .expect("revenge queue should load");
        assert_eq!(revenge_queue.len(), 1);
        assert_eq!(revenge_queue[0].question_id, question_id);
        assert_eq!(revenge_queue[0].attempts_to_beat, 2);
        assert!(revenge_queue[0].question_text.as_deref().is_some());
    }

    #[test]
    fn academic_calendar_snapshot_shapes_strategy_from_urgent_event() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let service = GoalsCalendarService::new(&conn);
        let anchor_date = "2026-03-30";
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_pending_mission(&conn, student_id, subject_id, topic_id, anchor_date);
        seed_title_ready_topic_state(&conn, student_id, topic_id);

        let event = service
            .upsert_academic_event(
                student_id,
                None,
                &AcademicCalendarEventInput {
                    title: "Math Mock".to_string(),
                    event_type: "mock".to_string(),
                    subject_id: Some(subject_id),
                    scheduled_date: "2026-04-02".to_string(),
                    start_time: Some("09:00".to_string()),
                    end_time: Some("11:00".to_string()),
                    term: Some("Term 2".to_string()),
                    academic_year: Some("2026".to_string()),
                    importance_bp: 8200,
                    scope: "focused".to_string(),
                    linked_topic_ids: vec![topic_id],
                    preparation_window_days: 14,
                    review_window_days: 5,
                    status: "scheduled".to_string(),
                    result_after_event: None,
                    coach_priority_weight_bp: 8600,
                    expected_weight_bp: 7800,
                    timed_performance_weight_bp: 7600,
                    coverage_mode: "mixed".to_string(),
                    source: Some("manual".to_string()),
                },
            )
            .expect("academic event should upsert");
        assert_eq!(event.title, "Math Mock");

        let snapshot = service
            .build_academic_calendar_snapshot(student_id, Some(anchor_date))
            .expect("calendar snapshot should build");

        assert_eq!(
            snapshot
                .prioritized_event
                .as_ref()
                .map(|item| item.title.as_str()),
            Some("Math Mock")
        );
        assert_eq!(
            snapshot.intensity.as_ref().map(|item| item.phase.as_str()),
            Some("wrap_up")
        );
        assert_eq!(
            snapshot
                .intensity
                .as_ref()
                .map(|item| item.recommended_mode.as_str()),
            Some("exam_wrap_up")
        );

        let current_phase: String = conn
            .query_row(
                "SELECT current_phase FROM coach_plans WHERE student_id = ?1 ORDER BY id DESC LIMIT 1",
                [student_id],
                |row| row.get(0),
            )
            .expect("coach plan should exist");
        assert_eq!(current_phase, "final_revision");
    }

    #[test]
    fn missed_engagement_creates_parent_alert_and_recovery_mission() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let parent_id = insert_parent_and_link(&conn, student_id);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let service = GoalsCalendarService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        seed_pending_mission(&conn, student_id, subject_id, topic_id, "2026-03-30");
        let mission_id = conn.last_insert_rowid();

        let event = service
            .record_engagement_event(
                student_id,
                &EngagementEventInput {
                    mission_id: Some(mission_id),
                    session_id: None,
                    reminder_schedule_id: None,
                    academic_event_id: None,
                    session_state: "missed".to_string(),
                    started_at: None,
                    ended_at: None,
                    completion_percent: 0,
                    missed_reason: Some("transport delay".to_string()),
                    source: Some("coach".to_string()),
                },
            )
            .expect("engagement event should record");
        assert_eq!(event.session_state, "missed");

        let risk = service
            .get_engagement_risk_profile(student_id)
            .expect("risk profile should load")
            .expect("risk profile should exist");
        assert!(matches!(
            risk.risk_level.as_str(),
            "medium" | "high" | "critical"
        ));

        let alerts = service
            .list_parent_alerts(parent_id, Some(student_id), None, 10)
            .expect("parent alerts should load");
        assert!(!alerts.is_empty());
        assert!(matches!(alerts[0].severity.as_str(), "high" | "urgent"));

        let recovery_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_missions
                 WHERE student_id = ?1 AND title LIKE 'Recovery:%'",
                [student_id],
                |row| row.get(0),
            )
            .expect("recovery mission count should load");
        assert!(recovery_count >= 1);
    }

    #[test]
    fn titles_hall_and_defense_flow_round_trip() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let service = GoalsCalendarService::new(&conn);
        let topic_id: i64 = conn
            .query_row(
                "SELECT topic_id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        seed_title_ready_topic_state(&conn, student_id, topic_id);
        seed_title_ready_attempts(&conn, student_id, topic_id);

        let hall = service
            .refresh_titles_hall(student_id, Some("2026-03-30"))
            .expect("titles hall should refresh");
        assert!(!hall.active_titles.is_empty());
        let title = hall.active_titles[0].clone();

        let brief = service
            .begin_title_defense(student_id, title.id)
            .expect("title defense should begin");
        assert_eq!(brief.title_id, title.id);
        assert!(brief.components.len() >= 3);

        let result = service
            .complete_title_defense(
                brief.run_id,
                &TitleDefenseCompletionInput {
                    recall_passed: true,
                    application_passed: true,
                    transfer_passed: true,
                    timed_challenge_passed: Some(true),
                    confidence_held: Some(true),
                    accuracy_score_bp: Some(8600),
                    notes: Some("Held under pressure".to_string()),
                    triggered_misconception: None,
                },
            )
            .expect("title defense should complete");
        assert!(matches!(
            result.new_state.as_str(),
            "defended" | "narrowly_defended"
        ));

        let history = service
            .list_title_history(title.id, 10)
            .expect("title history should load");
        assert!(history.len() >= 2);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn insert_student(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 90)",
            [student_id],
        )
        .expect("student profile should insert");
        student_id
    }

    fn insert_parent_and_link(conn: &Connection, student_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('parent', 'Grace', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("parent should insert");
        let parent_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO parent_profiles (account_id) VALUES (?1)",
            [parent_id],
        )
        .expect("parent profile should insert");
        conn.execute(
            "INSERT INTO parent_student_links (parent_account_id, student_account_id, relationship_label)
             VALUES (?1, ?2, 'parent')",
            params![parent_id, student_id],
        )
        .expect("parent link should insert");
        parent_id
    }

    fn seed_availability(service: &GoalsCalendarService<'_>, student_id: i64) {
        service
            .upsert_availability_profile(&AvailabilityProfile {
                student_id,
                timezone_name: "America/New_York".to_string(),
                preferred_daily_minutes: 90,
                ideal_session_minutes: 45,
                min_session_minutes: 15,
                max_session_minutes: 60,
                split_sessions_allowed: true,
                max_split_sessions: 2,
                min_break_minutes: 20,
                trigger_mode: "hybrid".to_string(),
                notification_lead_minutes: 10,
                weekday_capacity_weight_bp: 10_000,
                weekend_capacity_weight_bp: 11_500,
                schedule_buffer_ratio_bp: 1_500,
                fatigue_start_minute: None,
                fatigue_end_minute: None,
                thinking_idle_grace_seconds: 180,
                idle_confirmation_seconds: 120,
                abandonment_seconds: 900,
            })
            .expect("availability profile should upsert");
        service
            .replace_availability_windows(
                student_id,
                &[AvailabilityWindow {
                    weekday: 0,
                    start_minute: 18 * 60,
                    end_minute: 21 * 60,
                    is_preferred: true,
                }],
            )
            .expect("availability windows should replace");
    }

    fn seed_topic_state(conn: &Connection, student_id: i64, topic_id: i64) {
        seed_topic_state_with_scores(conn, student_id, topic_id, 4200, 9200, 7300, 9000, true);
    }

    fn seed_stable_topic_state(conn: &Connection, student_id: i64, topic_id: i64) {
        seed_topic_state_with_scores(conn, student_id, topic_id, 7600, 3600, 1800, 2500, false);
    }

    fn seed_title_ready_topic_state(conn: &Connection, student_id: i64, topic_id: i64) {
        seed_stable_topic_state(conn, student_id, topic_id);
        conn.execute(
            "UPDATE student_topic_states
             SET mastery_state = 'stable',
                 mastery_score = 8400,
                 speed_score = 7600,
                 transfer_score = 8000,
                 confidence_score = 7800,
                 fragility_score = 2200,
                 pressure_collapse_index = 1800,
                 total_attempts = 8,
                 correct_attempts = 7
             WHERE student_id = ?1 AND topic_id = ?2",
            params![student_id, topic_id],
        )
        .expect("title-ready topic state should update");
    }

    fn seed_topic_state_with_scores(
        conn: &Connection,
        student_id: i64,
        topic_id: i64,
        mastery_score: i64,
        priority_score: i64,
        fragility_score: i64,
        repair_priority: i64,
        is_urgent: bool,
    ) {
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score,
                fragility_score, pressure_collapse_index, decay_risk, memory_strength,
                evidence_count, repair_priority, is_urgent
             ) VALUES (?1, ?2, ?3, 'fragile', 8600, ?4, ?5, 5400, 6200, 3400, 3, ?6, ?7)",
            params![
                student_id,
                topic_id,
                mastery_score,
                priority_score,
                fragility_score,
                repair_priority,
                if is_urgent { 1 } else { 0 },
            ],
        )
        .expect("topic state should insert");
    }

    fn seed_memory_state(
        conn: &Connection,
        student_id: i64,
        topic_id: i64,
        memory_state: &str,
        memory_strength: i64,
        decay_risk: i64,
        review_due_date: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                student_id,
                topic_id,
                memory_state,
                memory_strength,
                decay_risk,
                review_due_date.map(|value| format!("{value}T18:00:00Z")),
            ],
        )
        .expect("memory state should insert");
    }

    fn seed_baseline_session(
        conn: &Connection,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        date: &str,
    ) {
        let topic_ids = format!("[{}]", topic_id);
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (?1, 'practice', ?2, ?3, 8, 8, 0, 'completed', ?4, ?4, 8, 5, 6250, 24000)",
            params![
                student_id,
                subject_id,
                topic_ids,
                format!("{date} 18:00:00")
            ],
        )
        .expect("baseline session should insert");
    }

    fn seed_pending_mission(
        conn: &Connection,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        date: &str,
    ) {
        conn.execute(
            "INSERT INTO coach_plans (
                student_id, exam_target, exam_date, start_date, total_days, daily_budget_minutes,
                current_phase, status, plan_data_json
             ) VALUES (?1, 'BECE', '2026-06-01', '2026-03-01', 90, 90, 'performance', 'active', '{}')",
            [student_id],
        )
        .expect("coach plan should insert");
        let plan_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO coach_plan_days (plan_id, date, phase, target_minutes, status)
             VALUES (?1, ?2, 'performance', 45, 'active')",
            params![plan_id, date],
        )
        .expect("plan day should insert");
        let plan_day_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO coach_missions (
                plan_day_id, student_id, title, reason, subject_id, primary_topic_id,
                activity_type, target_minutes, status
             ) VALUES (?1, ?2, 'Repair Window', 'daily planned repair', ?3, ?4, 'repair', 25, 'pending')",
            params![plan_day_id, student_id, subject_id, topic_id],
        )
        .expect("pending mission should insert");
    }

    fn seed_title_ready_attempts(conn: &Connection, student_id: i64, topic_id: i64) {
        let (question_id, correct_option_id): (i64, i64) = conn
            .query_row(
                "SELECT q.id,
                        MAX(CASE WHEN qo.is_correct = 1 THEN qo.id END)
                 FROM questions q
                 INNER JOIN question_options qo ON qo.question_id = q.id
                 WHERE q.topic_id = ?1
                 GROUP BY q.id
                 ORDER BY q.id ASC
                 LIMIT 1",
                [topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("question and correct option should load");
        for attempt_number in 1..=3 {
            conn.execute(
                "INSERT INTO student_question_attempts (
                    student_id, question_id, session_id, session_type, attempt_number,
                    started_at, submitted_at, response_time_ms, selected_option_id,
                    is_correct, confidence_level, support_level, was_timed, was_transfer_variant
                 ) VALUES (?1, ?2, NULL, 'practice', ?3, datetime('now'), datetime('now'), ?4, ?5, 1, 'sure', 'independent', ?6, ?7)",
                params![
                    student_id,
                    question_id,
                    attempt_number,
                    if attempt_number == 3 { 18_000 } else { 24_000 },
                    correct_option_id,
                    if attempt_number == 3 { 1 } else { 0 },
                    if attempt_number == 2 { 1 } else { 0 },
                ],
            )
            .expect("student question attempt should insert");
        }
    }

    fn seed_solidification_outcome(
        conn: &Connection,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        outcome: &str,
        next_action_hint: &str,
        accuracy_score: i64,
        occurred_at: &str,
    ) {
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, status, started_at, completed_at,
                answered_questions, correct_questions, accuracy_score
             ) VALUES (?1, 'gap_repair', ?2, ?3, 'completed', ?4, ?4, 4, 3, ?5)",
            params![
                student_id,
                subject_id,
                format!("[{}]", topic_id),
                occurred_at,
                accuracy_score,
            ],
        )
        .expect("gap repair session should insert");
        let session_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO solidification_sessions (
                student_id, topic_id, session_id, status, completed_at
             ) VALUES (?1, ?2, ?3, 'completed', ?4)",
            params![student_id, topic_id, session_id, occurred_at],
        )
        .expect("solidification session should insert");
        conn.execute(
            "INSERT INTO runtime_events (
                event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
             ) VALUES (?1, 'session.interpreted', 'session', ?2, ?3, ?4, ?5)",
            params![
                format!("session-interpreted-{}", session_id),
                session_id.to_string(),
                format!("trace-session-{}", session_id),
                json!({
                    "repair_outcome": outcome,
                    "next_action_hint": next_action_hint,
                    "topic_summaries": [
                        {
                            "topic_id": topic_id,
                            "accuracy_score": accuracy_score
                        }
                    ]
                })
                .to_string(),
                occurred_at,
            ],
        )
        .expect("session interpretation event should insert");
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}
