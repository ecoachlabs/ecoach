use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use ecoach_student_model::{LearnerTruthSnapshot, StudentModelService, StudentTopicState};
use ecoach_substrate::LearnerEvidenceFabric;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthDto {
    pub student_id: i64,
    pub student_name: String,
    pub overall_mastery_score: i64,
    pub overall_readiness_band: String,
    pub pending_review_count: i64,
    pub due_memory_count: i64,
    pub topic_count: usize,
    pub skill_count: usize,
    pub memory_count: usize,
    pub diagnosis_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeLearningStatsDto {
    pub streak_days: i64,
    pub accuracy_percent: i64,
    pub today_minutes: i64,
    pub week_questions: i64,
    pub total_attempts: i64,
    pub correct_attempts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentActivityHistoryItemDto {
    pub id: i64,
    pub type_key: String,
    pub label: String,
    pub subject: String,
    pub score: i64,
    pub answered_questions: i64,
    pub total_questions: i64,
    pub correct_questions: i64,
    pub occurred_at: String,
    pub status: String,
}

impl From<LearnerTruthSnapshot> for LearnerTruthDto {
    fn from(v: LearnerTruthSnapshot) -> Self {
        Self {
            student_id: v.student_id,
            student_name: v.student_name,
            overall_mastery_score: v.overall_mastery_score as i64,
            overall_readiness_band: v.overall_readiness_band,
            pending_review_count: v.pending_review_count,
            due_memory_count: v.due_memory_count,
            topic_count: v.total_topic_count,
            skill_count: v.total_skill_count,
            memory_count: v.total_memory_count,
            diagnosis_count: v.recent_diagnosis_count,
        }
    }
}

pub type LearnerEvidenceFabricDto = LearnerEvidenceFabric;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStateDto {
    pub topic_id: i64,
    pub mastery_score: i64,
    pub mastery_state: String,
    pub accuracy_score: i64,
    pub speed_score: i64,
    pub confidence_score: i64,
    pub retention_score: i64,
    pub gap_score: i64,
    pub priority_score: i64,
    pub trend_state: String,
    pub fragility_score: i64,
    pub pressure_collapse_index: i64,
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub memory_strength: i64,
}

impl From<StudentTopicState> for TopicStateDto {
    fn from(v: StudentTopicState) -> Self {
        Self {
            topic_id: v.topic_id,
            mastery_score: v.mastery_score as i64,
            mastery_state: v.mastery_state.as_str().to_string(),
            accuracy_score: v.accuracy_score as i64,
            speed_score: v.speed_score as i64,
            confidence_score: v.confidence_score as i64,
            retention_score: v.retention_score as i64,
            gap_score: v.gap_score as i64,
            priority_score: v.priority_score as i64,
            trend_state: v.trend_state,
            fragility_score: v.fragility_score as i64,
            pressure_collapse_index: v.pressure_collapse_index as i64,
            total_attempts: v.total_attempts,
            correct_attempts: v.correct_attempts,
            memory_strength: v.memory_strength as i64,
        }
    }
}

pub fn get_learner_truth(
    state: &AppState,
    student_id: i64,
) -> Result<LearnerTruthDto, CommandError> {
    state.with_connection(|conn| {
        let service = StudentModelService::new(conn);
        let snapshot = service.get_learner_truth_snapshot(student_id)?;
        Ok(LearnerTruthDto::from(snapshot))
    })
}

pub fn get_home_learning_stats(
    state: &AppState,
    student_id: i64,
) -> Result<HomeLearningStatsDto, CommandError> {
    state.with_connection(|conn| build_home_learning_stats(conn, student_id))
}

pub fn list_student_activity_history(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<StudentActivityHistoryItemDto>, CommandError> {
    state.with_connection(|conn| {
        let capped_limit = limit.clamp(1, 200);
        let mut items = load_session_activity_history(conn, student_id, capped_limit)?;
        items.extend(load_diagnostic_activity_history(conn, student_id, capped_limit)?);
        items.extend(load_game_activity_history(conn, student_id, capped_limit)?);
        items.extend(load_elite_activity_history(conn, student_id, capped_limit)?);
        items.sort_by(|left, right| {
            activity_timestamp_key(&right.occurred_at).cmp(&activity_timestamp_key(&left.occurred_at))
        });
        items.truncate(capped_limit);
        Ok(items)
    })
}

pub fn get_learner_evidence_fabric(
    state: &AppState,
    student_id: i64,
    limit_per_stream: usize,
) -> Result<LearnerEvidenceFabricDto, CommandError> {
    state.with_connection(|conn| {
        let service = StudentModelService::new(conn);
        Ok(service.get_learner_evidence_fabric(student_id, limit_per_stream)?)
    })
}

// ── idea3 instant-gratification features ──

fn build_home_learning_stats(
    conn: &Connection,
    student_id: i64,
) -> Result<HomeLearningStatsDto, CommandError> {
    let today = Utc::now().date_naive();
    let today_key = today.format("%Y-%m-%d").to_string();
    let week_start = (today - Duration::days(6)).format("%Y-%m-%d").to_string();

    let (week_questions, week_correct, today_response_ms, total_attempts, correct_attempts): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = conn
        .query_row(
            "SELECT
                COUNT(CASE WHEN date(COALESCE(submitted_at, created_at)) >= date(?2) THEN 1 END),
                COALESCE(SUM(CASE
                    WHEN date(COALESCE(submitted_at, created_at)) >= date(?2)
                     AND is_correct = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE
                    WHEN date(COALESCE(submitted_at, created_at)) = date(?3)
                    THEN COALESCE(response_time_ms, 0) ELSE 0 END), 0),
                COUNT(*),
                COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), 0)
             FROM student_question_attempts
             WHERE student_id = ?1
               AND COALESCE(skipped, 0) = 0",
            params![student_id, week_start, today_key],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(storage_error)?;

    let streak_days = current_attempt_streak_days(conn, student_id, today)?;
    let accuracy_percent = if week_questions > 0 {
        ((week_correct * 100) + (week_questions / 2)) / week_questions
    } else {
        0
    };
    let today_minutes = if today_response_ms > 0 {
        (today_response_ms + 59_999) / 60_000
    } else {
        0
    };

    Ok(HomeLearningStatsDto {
        streak_days,
        accuracy_percent,
        today_minutes,
        week_questions,
        total_attempts,
        correct_attempts,
    })
}

fn load_session_activity_history(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<Vec<StudentActivityHistoryItemDto>, CommandError> {
    let mut statement = conn
        .prepare(
            "SELECT
                s.id,
                s.session_type,
                COALESCE(sub.name, 'All Subjects'),
                COALESCE(s.answered_questions, 0),
                COALESCE(s.total_questions, s.question_count, 0),
                COALESCE(s.correct_questions, 0),
                s.accuracy_score,
                COALESCE(s.completed_at, s.last_activity_at, s.started_at, datetime('now')),
                s.status
             FROM sessions s
             LEFT JOIN subjects sub ON sub.id = s.subject_id
             WHERE s.student_id = ?1
             ORDER BY datetime(COALESCE(s.completed_at, s.last_activity_at, s.started_at)) DESC, s.id DESC
             LIMIT ?2",
        )
        .map_err(storage_error)?;

    let rows = statement
        .query_map(params![student_id, limit as i64], |row| {
            let session_type = row.get::<_, String>(1)?;
            let answered_questions = row.get::<_, i64>(3)?;
            let correct_questions = row.get::<_, i64>(5)?;
            Ok(StudentActivityHistoryItemDto {
                id: row.get(0)?,
                type_key: history_type_key_for_session(&session_type).to_string(),
                label: history_label_for_session(&session_type).to_string(),
                subject: row.get(2)?,
                answered_questions,
                total_questions: row.get(4)?,
                correct_questions,
                score: score_from_basis_points(row.get(6)?, correct_questions, answered_questions),
                occurred_at: row.get(7)?,
                status: row.get(8)?,
            })
        })
        .map_err(storage_error)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(storage_error)?);
    }
    Ok(items)
}

fn load_diagnostic_activity_history(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<Vec<StudentActivityHistoryItemDto>, CommandError> {
    let mut statement = conn
        .prepare(
            "SELECT
                di.id,
                di.session_mode,
                COALESCE(sub.name, 'All Subjects'),
                COALESCE(SUM(CASE
                    WHEN dia.submitted_at IS NOT NULL
                      OR dia.selected_option_id IS NOT NULL
                      OR COALESCE(dia.skipped, 0) = 1
                      OR COALESCE(dia.timed_out, 0) = 1
                    THEN 1 ELSE 0 END), 0),
                COUNT(dia.id),
                COALESCE(SUM(CASE WHEN dia.is_correct = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(di.completed_at, di.started_at, datetime('now')),
                di.status
             FROM diagnostic_instances di
             LEFT JOIN subjects sub ON sub.id = di.subject_id
             LEFT JOIN diagnostic_item_attempts dia ON dia.diagnostic_id = di.id
             WHERE di.student_id = ?1
             GROUP BY di.id, di.session_mode, sub.name, di.completed_at, di.started_at, di.status
             ORDER BY datetime(COALESCE(di.completed_at, di.started_at)) DESC, di.id DESC
             LIMIT ?2",
        )
        .map_err(storage_error)?;

    let rows = statement
        .query_map(params![student_id, limit as i64], |row| {
            let mode = row.get::<_, String>(1)?;
            let answered_questions = row.get::<_, i64>(3)?;
            let correct_questions = row.get::<_, i64>(5)?;
            Ok(StudentActivityHistoryItemDto {
                id: row.get(0)?,
                type_key: "diagnostic".to_string(),
                label: history_label_for_diagnostic(&mode).to_string(),
                subject: row.get(2)?,
                score: score_from_counts(correct_questions, answered_questions),
                answered_questions,
                total_questions: row.get(4)?,
                correct_questions,
                occurred_at: row.get(6)?,
                status: row.get(7)?,
            })
        })
        .map_err(storage_error)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(storage_error)?);
    }
    Ok(items)
}

fn load_game_activity_history(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<Vec<StudentActivityHistoryItemDto>, CommandError> {
    let mut statement = conn
        .prepare(
            "SELECT
                gs.id,
                gs.game_type,
                COALESCE(sub.name, 'All Subjects'),
                COALESCE(gae.answered_questions, gs.rounds_played, 0),
                COALESCE(gs.rounds_total, 0),
                COALESCE(gae.correct_questions, 0),
                COALESCE(gs.completed_at, gs.created_at, datetime('now')),
                gs.session_state
             FROM game_sessions gs
             LEFT JOIN subjects sub ON sub.id = gs.subject_id
             LEFT JOIN (
                SELECT
                    game_session_id,
                    COUNT(*) AS answered_questions,
                    COALESCE(SUM(CASE WHEN was_correct = 1 THEN 1 ELSE 0 END), 0) AS correct_questions
                FROM game_answer_events
                GROUP BY game_session_id
             ) gae ON gae.game_session_id = gs.id
             WHERE gs.student_id = ?1
             ORDER BY datetime(COALESCE(gs.completed_at, gs.created_at)) DESC, gs.id DESC
             LIMIT ?2",
        )
        .map_err(storage_error)?;

    let rows = statement
        .query_map(params![student_id, limit as i64], |row| {
            let game_type = row.get::<_, String>(1)?;
            let answered_questions = row.get::<_, i64>(3)?;
            let correct_questions = row.get::<_, i64>(5)?;
            Ok(StudentActivityHistoryItemDto {
                id: row.get(0)?,
                type_key: "games".to_string(),
                label: history_label_for_game(&game_type).to_string(),
                subject: row.get(2)?,
                score: score_from_counts(correct_questions, answered_questions),
                answered_questions,
                total_questions: row.get(4)?,
                correct_questions,
                occurred_at: row.get(6)?,
                status: row.get(7)?,
            })
        })
        .map_err(storage_error)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(storage_error)?);
    }
    Ok(items)
}

fn load_elite_activity_history(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<Vec<StudentActivityHistoryItemDto>, CommandError> {
    let mut statement = conn
        .prepare(
            "SELECT
                esr.id,
                esr.session_type,
                COALESCE(sub.name, 'All Subjects'),
                esr.metadata_json,
                esr.created_at
             FROM elite_session_records esr
             LEFT JOIN subjects sub ON sub.id = esr.subject_id
             WHERE esr.student_id = ?1
             ORDER BY datetime(esr.created_at) DESC, esr.id DESC
             LIMIT ?2",
        )
        .map_err(storage_error)?;

    let rows = statement
        .query_map(params![student_id, limit as i64], |row| {
            let session_type = row.get::<_, String>(1)?;
            let metadata_json = row.get::<_, String>(3)?;
            let metadata = serde_json::from_str::<serde_json::Value>(&metadata_json)
                .unwrap_or(serde_json::Value::Null);
            let accuracy_score = metadata
                .get("accuracy_score")
                .and_then(serde_json::Value::as_i64);
            let item_count = metadata
                .get("item_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let correct_questions = accuracy_score
                .map(|value| ((value * item_count) + 5_000) / 10_000)
                .unwrap_or(0);
            Ok(StudentActivityHistoryItemDto {
                id: row.get(0)?,
                type_key: "elite".to_string(),
                label: history_label_for_elite(&session_type),
                subject: row.get(2)?,
                score: score_from_basis_points(accuracy_score, correct_questions, item_count),
                answered_questions: item_count,
                total_questions: item_count,
                correct_questions,
                occurred_at: row.get(4)?,
                status: "completed".to_string(),
            })
        })
        .map_err(storage_error)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(storage_error)?);
    }
    Ok(items)
}

fn history_type_key_for_session(session_type: &str) -> &'static str {
    match session_type {
        "mock" | "custom_test" => "mock",
        _ => "practice",
    }
}

fn history_label_for_session(session_type: &str) -> &'static str {
    match session_type {
        "coach_mission" => "Coach Mission",
        "mock" => "Mock Session",
        "custom_test" => "Custom Test",
        "gap_repair" => "Solidify Session",
        _ => "Practice Session",
    }
}

fn history_label_for_diagnostic(mode: &str) -> &'static str {
    match mode {
        "pressure" => "Pressure Diagnostic",
        "root_cause" => "Root-Cause Diagnostic",
        _ => "Diagnostic Scan",
    }
}

fn history_label_for_game(game_type: &str) -> &'static str {
    match game_type {
        "mindstack" => "MindStack",
        "tug_of_war" => "Tug of War",
        "traps" => "Traps",
        _ => "Game Session",
    }
}

fn history_label_for_elite(session_type: &str) -> String {
    match session_type {
        "elite_sprint" => "Elite Sprint".to_string(),
        "precision_lab" => "Precision Lab".to_string(),
        "depth_lab" => "Depth Lab".to_string(),
        "endurance_track" => "Endurance Track".to_string(),
        "apex_mock" => "Apex Mock".to_string(),
        "trapsense" => "TrapSense".to_string(),
        "perfect_run" => "Perfect Run".to_string(),
        _ => titleize_history_key(session_type),
    }
}

fn titleize_history_key(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn score_from_basis_points(
    accuracy_score: Option<i64>,
    correct_questions: i64,
    answered_questions: i64,
) -> i64 {
    accuracy_score
        .map(|value| ((value + 50) / 100).clamp(0, 100))
        .unwrap_or_else(|| score_from_counts(correct_questions, answered_questions))
}

fn score_from_counts(correct_questions: i64, answered_questions: i64) -> i64 {
    if answered_questions > 0 {
        ((correct_questions * 100) + (answered_questions / 2)) / answered_questions
    } else {
        0
    }
}

fn activity_timestamp_key(raw: &str) -> i64 {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return parsed.timestamp();
    }
    if let Ok(parsed) = NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc).timestamp();
    }
    if let Ok(parsed) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        if let Some(at_midnight) = parsed.and_hms_opt(0, 0, 0) {
            return DateTime::<Utc>::from_naive_utc_and_offset(at_midnight, Utc).timestamp();
        }
    }
    0
}

fn current_attempt_streak_days(
    conn: &Connection,
    student_id: i64,
    today: NaiveDate,
) -> Result<i64, CommandError> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT date(COALESCE(submitted_at, created_at)) AS attempt_day
             FROM student_question_attempts
             WHERE student_id = ?1
               AND COALESCE(skipped, 0) = 0
             ORDER BY attempt_day DESC
             LIMIT 60",
        )
        .map_err(storage_error)?;
    let rows = stmt
        .query_map(params![student_id], |row| row.get::<_, String>(0))
        .map_err(storage_error)?;

    let mut expected = today;
    let mut streak = 0;
    for row in rows {
        let raw_day = row.map_err(storage_error)?;
        let Ok(day) = NaiveDate::parse_from_str(&raw_day, "%Y-%m-%d") else {
            continue;
        };
        if day == expected {
            streak += 1;
            expected = expected - Duration::days(1);
            continue;
        }
        if streak == 0 && day == today - Duration::days(1) {
            streak += 1;
            expected = today - Duration::days(2);
            continue;
        }
        if day < expected {
            break;
        }
    }
    Ok(streak)
}

fn storage_error(err: rusqlite::Error) -> CommandError {
    CommandError {
        code: "storage_error".to_string(),
        message: err.to_string(),
    }
}

/// Academic MRI / instant scan: run diagnostic and return instant insight summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicScanResult {
    pub student_id: i64,
    pub overall_readiness_band: String,
    pub strongest_topic: Option<String>,
    pub weakest_topic: Option<String>,
    pub top_score_blockers: Vec<String>,
    pub readiness_score: i64,
    pub study_days_last_14: i64,
    pub recommended_next_action: String,
}

pub fn get_academic_scan(
    state: &AppState,
    student_id: i64,
) -> Result<AcademicScanResult, CommandError> {
    state.with_connection(|conn| {
        let truth = StudentModelService::new(conn).get_learner_truth_snapshot(student_id)?;

        let strongest = truth
            .topic_summaries
            .iter()
            .max_by_key(|t| t.mastery_score)
            .map(|t| t.topic_name.clone());
        let weakest = truth
            .topic_summaries
            .iter()
            .min_by_key(|t| t.mastery_score)
            .map(|t| t.topic_name.clone());

        // Top score blockers from recent diagnoses
        let blockers: Vec<String> = truth
            .recent_diagnoses
            .iter()
            .take(3)
            .map(|d| d.primary_diagnosis.clone())
            .collect();

        let recommended = if truth.pending_review_count > 0 {
            "Complete pending reviews before new content".into()
        } else if truth.overall_mastery_score < 5000 {
            "Start with a focused repair session on your weakest topic".into()
        } else {
            "Continue your study path and take a mini mock this week".into()
        };

        // Study consistency
        let study_days: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_consistency
                 WHERE student_id = ?1 AND study_date >= date('now', '-14 days')",
                [student_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(AcademicScanResult {
            student_id,
            overall_readiness_band: truth.overall_readiness_band,
            strongest_topic: strongest,
            weakest_topic: weakest,
            top_score_blockers: blockers,
            readiness_score: truth.overall_mastery_score as i64,
            study_days_last_14: study_days,
            recommended_next_action: recommended,
        })
    })
}

/// "What changed this week" summary for parents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyChangeDto {
    pub student_id: i64,
    pub student_name: String,
    pub improvements: Vec<String>,
    pub concerns: Vec<String>,
    pub recommended_actions: Vec<String>,
}

pub fn get_weekly_change_summary(
    state: &AppState,
    student_id: i64,
) -> Result<WeeklyChangeDto, CommandError> {
    state.with_connection(|conn| {
        let truth = StudentModelService::new(conn).get_learner_truth_snapshot(student_id)?;

        let mut improvements = Vec::new();
        let mut concerns = Vec::new();

        for topic in &truth.topic_summaries {
            if topic.mastery_state == "robust" || topic.mastery_state == "exam_ready" {
                improvements.push(format!("{} is improving", topic.topic_name));
            } else if topic.mastery_state == "fragile" || topic.mastery_state == "exposed" {
                concerns.push(format!("{} needs attention", topic.topic_name));
            }
        }

        // Check consistency
        let study_days: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_consistency
                 WHERE student_id = ?1 AND study_date >= date('now', '-7 days')",
                [student_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if study_days >= 5 {
            improvements.push("Strong study consistency this week".into());
        } else if study_days <= 2 {
            concerns.push("Study activity has been low this week".into());
        }

        let mut actions = Vec::new();
        if !concerns.is_empty() {
            actions.push("Focus on the areas flagged as needing attention".into());
        }
        if study_days <= 2 {
            actions.push("Encourage at least one short session today".into());
        }
        if improvements.is_empty() && concerns.is_empty() {
            actions.push("Maintain current study pace".into());
        }

        Ok(WeeklyChangeDto {
            student_id,
            student_name: truth.student_name,
            improvements,
            concerns,
            recommended_actions: actions,
        })
    })
}

/// Attention-needed queue for parents: which children need help most.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionNeededItem {
    pub student_id: i64,
    pub student_name: String,
    pub urgency: String,
    pub reason: String,
    pub recommended_action: String,
}

pub fn get_attention_needed_queue(
    state: &AppState,
    parent_id: i64,
) -> Result<Vec<AttentionNeededItem>, CommandError> {
    state.with_connection(|conn| {
        let identity = ecoach_identity::IdentityService::new(conn);
        let students = identity.get_linked_students(parent_id)?;

        let mut items = Vec::new();
        for student in &students {
            // Check for risk flags
            let risk_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM risk_flags
                     WHERE student_id = ?1 AND status = 'active'",
                    [student.id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            // Check inactivity
            let days_since_activity: i64 = conn
                .query_row(
                    "SELECT COALESCE(CAST(julianday('now') - julianday(MAX(study_date)) AS INTEGER), 99)
                     FROM study_consistency WHERE student_id = ?1",
                    [student.id],
                    |row| row.get(0),
                )
                .unwrap_or(99);

            if risk_count > 0 {
                items.push(AttentionNeededItem {
                    student_id: student.id,
                    student_name: student.display_name.clone(),
                    urgency: "high".into(),
                    reason: format!("{} active risk flag(s) detected", risk_count),
                    recommended_action: "Review risk details and focus on weak topics".into(),
                });
            } else if days_since_activity >= 5 {
                items.push(AttentionNeededItem {
                    student_id: student.id,
                    student_name: student.display_name.clone(),
                    urgency: "medium".into(),
                    reason: format!("No study activity for {} days", days_since_activity),
                    recommended_action: "Encourage a short return session to rebuild momentum".into(),
                });
            }
        }

        items.sort_by(|a, b| {
            let urgency_order = |u: &str| match u {
                "high" => 0,
                "medium" => 1,
                _ => 2,
            };
            urgency_order(&a.urgency).cmp(&urgency_order(&b.urgency))
        });

        Ok(items)
    })
}
