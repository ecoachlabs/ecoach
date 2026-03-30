use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, NaiveDate};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    AvailabilityException, AvailabilityProfile, AvailabilityWindow, BeatYesterdayDailySummary,
    BeatYesterdayDailyTarget, BeatYesterdayDashboard, BeatYesterdayProfile, CalendarEvent,
    ClimbTrendPoint, DailyAvailabilitySummary, DailyReplan, FreeNowRecommendation, Goal,
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

fn to_sql_conversion_error(err: EcoachError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
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
        Ok(self.conn.last_insert_rowid())
    }

    pub fn next_event(&self, student_id: i64) -> EcoachResult<Option<CalendarEvent>> {
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

    pub fn upsert_availability_profile(&self, profile: &AvailabilityProfile) -> EcoachResult<()> {
        self.conn.execute(
            "INSERT INTO availability_profiles (
                student_id, timezone_name, preferred_daily_minutes, min_session_minutes, max_session_minutes
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(student_id) DO UPDATE SET
                timezone_name = excluded.timezone_name,
                preferred_daily_minutes = excluded.preferred_daily_minutes,
                min_session_minutes = excluded.min_session_minutes,
                max_session_minutes = excluded.max_session_minutes,
                updated_at = datetime('now')",
            params![
                profile.student_id,
                profile.timezone_name,
                profile.preferred_daily_minutes,
                profile.min_session_minutes,
                profile.max_session_minutes,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
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
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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

    fn seed_availability(service: &GoalsCalendarService<'_>, student_id: i64) {
        service
            .upsert_availability_profile(&AvailabilityProfile {
                student_id,
                timezone_name: "America/New_York".to_string(),
                preferred_daily_minutes: 90,
                min_session_minutes: 15,
                max_session_minutes: 60,
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
