use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamStrategySessionInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub session_id: Option<i64>,
    pub strategy_type: String,
    pub questions_attempted: Option<i64>,
    pub questions_skipped: Option<i64>,
    pub questions_returned_to: Option<i64>,
    pub marks_gained_from_return: Option<i64>,
    pub time_wasted_seconds: Option<i64>,
    pub optimal_time_used_bp: Option<BasisPoints>,
    pub insights: Option<Value>,
    pub total_exam_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamStrategyProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub total_exam_minutes: i64,
    pub section_time_allocation: Value,
    pub skip_return_effectiveness_bp: BasisPoints,
    pub recheck_value_bp: BasisPoints,
    pub rushing_error_rate_bp: BasisPoints,
    pub overthinking_time_loss_bp: BasisPoints,
    pub optimal_pace_seconds_per_question: Option<i64>,
    pub best_section_order: Vec<String>,
    pub mark_maximization_strategy: Option<String>,
    pub when_blank_strategy: Option<String>,
    pub elimination_effectiveness_bp: BasisPoints,
    pub session_count: i64,
    pub updated_at: Option<String>,
}

pub struct ExamStrategyService<'a> {
    conn: &'a Connection,
}

impl<'a> ExamStrategyService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn record_exam_strategy_session(
        &self,
        input: ExamStrategySessionInput,
    ) -> EcoachResult<ExamStrategyProfile> {
        self.validate_strategy_type(&input.strategy_type)?;
        let session_context = input
            .session_id
            .map(|session_id| self.load_session_context(session_id))
            .transpose()?;

        let questions_attempted = input
            .questions_attempted
            .or_else(|| {
                session_context
                    .as_ref()
                    .map(|ctx| ctx.answered_questions.max(ctx.total_questions))
            })
            .unwrap_or(0);
        let questions_skipped = input
            .questions_skipped
            .or_else(|| {
                session_context
                    .as_ref()
                    .map(|ctx| (ctx.total_questions - ctx.answered_questions).max(0))
            })
            .unwrap_or(0);
        let questions_returned_to = input.questions_returned_to.unwrap_or(0);
        let marks_gained_from_return = input.marks_gained_from_return.unwrap_or(0);
        let time_wasted_seconds = input
            .time_wasted_seconds
            .or_else(|| {
                session_context
                    .as_ref()
                    .map(|ctx| ctx.estimated_wasted_seconds)
            })
            .unwrap_or(0);
        let optimal_time_used_bp = input.optimal_time_used_bp.unwrap_or_else(|| {
            self.estimate_optimal_time_used_bp(
                input
                    .total_exam_minutes
                    .or_else(|| {
                        session_context
                            .as_ref()
                            .map(|ctx| ctx.duration_minutes.unwrap_or(120))
                    })
                    .unwrap_or(120),
                questions_attempted,
                time_wasted_seconds,
            )
        });
        let total_exam_minutes = input
            .total_exam_minutes
            .or_else(|| {
                session_context
                    .as_ref()
                    .and_then(|ctx| ctx.duration_minutes)
            })
            .unwrap_or(120)
            .max(1);
        let insights_json = serde_json::to_string(&input.insights.unwrap_or_else(|| json!({})))
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        let strategy_score_bp = self.estimate_strategy_score(
            questions_attempted,
            questions_skipped,
            questions_returned_to,
            marks_gained_from_return,
            time_wasted_seconds,
            optimal_time_used_bp,
        );

        self.conn
            .execute(
                "INSERT INTO exam_strategy_sessions (
                    student_id, session_id, strategy_type, subject_id,
                    questions_attempted, questions_skipped, questions_returned_to,
                    marks_gained_from_return, time_wasted_seconds, optimal_time_used_bp,
                    strategy_score_bp, insights_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    input.student_id,
                    input.session_id,
                    normalize_strategy_type(&input.strategy_type),
                    input.subject_id,
                    questions_attempted,
                    questions_skipped,
                    questions_returned_to,
                    marks_gained_from_return,
                    time_wasted_seconds,
                    optimal_time_used_bp as i64,
                    strategy_score_bp as i64,
                    insights_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let profile =
            self.rebuild_profile(input.student_id, input.subject_id, Some(total_exam_minutes))?;

        self.persist_profile(&profile)?;
        Ok(profile)
    }

    pub fn get_exam_strategy_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<ExamStrategyProfile>> {
        let session_count = self.count_strategy_sessions(student_id, subject_id)?;
        self.conn
            .query_row(
                "SELECT total_exam_minutes, section_time_allocation_json,
                        skip_return_effectiveness_bp, recheck_value_bp,
                        rushing_error_rate_bp, overthinking_time_loss_bp,
                        optimal_pace_seconds_per_question, best_section_order_json,
                        mark_maximization_strategy, when_blank_strategy,
                        elimination_effectiveness_bp, updated_at
                 FROM exam_strategy_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| {
                    let section_time_allocation_json: String = row.get(1)?;
                    let best_section_order_json: Option<String> = row.get(7)?;
                    Ok(ExamStrategyProfile {
                        student_id,
                        subject_id,
                        total_exam_minutes: row.get(0)?,
                        section_time_allocation: serde_json::from_str(
                            &section_time_allocation_json,
                        )
                        .unwrap_or_else(|_| json!({})),
                        skip_return_effectiveness_bp: clamp_bp(row.get::<_, i64>(2)?),
                        recheck_value_bp: clamp_bp(row.get::<_, i64>(3)?),
                        rushing_error_rate_bp: clamp_bp(row.get::<_, i64>(4)?),
                        overthinking_time_loss_bp: clamp_bp(row.get::<_, i64>(5)?),
                        optimal_pace_seconds_per_question: row.get(6)?,
                        best_section_order: serde_json::from_str(
                            best_section_order_json.as_deref().unwrap_or("[]"),
                        )
                        .unwrap_or_default(),
                        mark_maximization_strategy: row.get(8)?,
                        when_blank_strategy: row.get(9)?,
                        elimination_effectiveness_bp: clamp_bp(row.get::<_, i64>(10)?),
                        session_count,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn rebuild_profile(
        &self,
        student_id: i64,
        subject_id: i64,
        total_exam_minutes_override: Option<i64>,
    ) -> EcoachResult<ExamStrategyProfile> {
        let sessions = self.load_strategy_sessions(student_id, subject_id)?;
        let session_count = sessions.len() as i64;
        let total_questions_attempted: i64 =
            sessions.iter().map(|item| item.questions_attempted).sum();
        let total_questions_skipped: i64 = sessions.iter().map(|item| item.questions_skipped).sum();
        let total_questions_returned: i64 =
            sessions.iter().map(|item| item.questions_returned_to).sum();
        let total_marks_gained_from_return: i64 = sessions
            .iter()
            .map(|item| item.marks_gained_from_return)
            .sum();
        let total_time_wasted_seconds: i64 =
            sessions.iter().map(|item| item.time_wasted_seconds).sum();
        let total_exam_minutes = total_exam_minutes_override
            .or_else(|| {
                sessions
                    .iter()
                    .filter_map(|item| item.total_exam_minutes)
                    .max()
            })
            .unwrap_or(120)
            .max(1);
        let _average_optimal_time_used_bp = if session_count > 0 {
            clamp_bp(
                sessions
                    .iter()
                    .map(|item| item.optimal_time_used_bp as i64)
                    .sum::<i64>()
                    / session_count,
            )
        } else {
            5000
        };

        let skip_return_effectiveness_bp = if total_questions_skipped > 0 {
            clamp_bp(
                5000 + (total_questions_returned * 2200 / total_questions_skipped)
                    + (total_marks_gained_from_return * 900 / total_questions_skipped.max(1)),
            )
        } else {
            5000
        };
        let recheck_value_bp = if total_questions_returned > 0 {
            clamp_bp(5000 + (total_marks_gained_from_return * 2600 / total_questions_returned))
        } else {
            4500
        };
        let rushing_error_rate_bp = if total_questions_attempted + total_questions_skipped > 0 {
            clamp_bp(
                (total_questions_skipped * 10_000)
                    / (total_questions_attempted + total_questions_skipped),
            )
        } else {
            0
        };
        let overthinking_time_loss_bp = clamp_bp(
            (total_time_wasted_seconds * 10_000) / (total_exam_minutes * 60 * session_count.max(1)),
        );
        let optimal_pace_seconds_per_question = if total_questions_attempted > 0 {
            Some((total_exam_minutes * 60) / total_questions_attempted.max(1))
        } else {
            None
        };
        let elimination_effectiveness_bp = clamp_bp(
            4500 + (total_marks_gained_from_return * 2000 / total_questions_returned.max(1))
                + (session_count * 120).min(1200),
        );

        let section_time_allocation = build_section_time_allocation(
            total_exam_minutes,
            skip_return_effectiveness_bp,
            recheck_value_bp,
            rushing_error_rate_bp,
            overthinking_time_loss_bp,
        );
        let best_section_order = build_section_order(
            skip_return_effectiveness_bp,
            recheck_value_bp,
            rushing_error_rate_bp,
            overthinking_time_loss_bp,
        );
        let mark_maximization_strategy = Some(build_mark_maximization_strategy(
            skip_return_effectiveness_bp,
            rushing_error_rate_bp,
            overthinking_time_loss_bp,
        ));
        let when_blank_strategy = Some(build_when_blank_strategy(
            rushing_error_rate_bp,
            overthinking_time_loss_bp,
        ));

        Ok(ExamStrategyProfile {
            student_id,
            subject_id,
            total_exam_minutes,
            section_time_allocation,
            skip_return_effectiveness_bp,
            recheck_value_bp,
            rushing_error_rate_bp,
            overthinking_time_loss_bp,
            optimal_pace_seconds_per_question,
            best_section_order,
            mark_maximization_strategy,
            when_blank_strategy,
            elimination_effectiveness_bp,
            session_count,
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        })
    }

    fn persist_profile(&self, profile: &ExamStrategyProfile) -> EcoachResult<()> {
        let section_time_allocation_json = serde_json::to_string(&profile.section_time_allocation)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let best_section_order_json = serde_json::to_string(&profile.best_section_order)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO exam_strategy_profiles (
                    student_id, subject_id, total_exam_minutes, section_time_allocation_json,
                    skip_return_effectiveness_bp, recheck_value_bp, rushing_error_rate_bp,
                    overthinking_time_loss_bp, optimal_pace_seconds_per_question,
                    best_section_order_json, mark_maximization_strategy, when_blank_strategy,
                    elimination_effectiveness_bp, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))
                 ON CONFLICT(student_id, subject_id) DO UPDATE SET
                    total_exam_minutes = ?3,
                    section_time_allocation_json = ?4,
                    skip_return_effectiveness_bp = ?5,
                    recheck_value_bp = ?6,
                    rushing_error_rate_bp = ?7,
                    overthinking_time_loss_bp = ?8,
                    optimal_pace_seconds_per_question = ?9,
                    best_section_order_json = ?10,
                    mark_maximization_strategy = ?11,
                    when_blank_strategy = ?12,
                    elimination_effectiveness_bp = ?13,
                    updated_at = datetime('now')",
                params![
                    profile.student_id,
                    profile.subject_id,
                    profile.total_exam_minutes,
                    section_time_allocation_json,
                    profile.skip_return_effectiveness_bp as i64,
                    profile.recheck_value_bp as i64,
                    profile.rushing_error_rate_bp as i64,
                    profile.overthinking_time_loss_bp as i64,
                    profile.optimal_pace_seconds_per_question,
                    best_section_order_json,
                    profile.mark_maximization_strategy,
                    profile.when_blank_strategy,
                    profile.elimination_effectiveness_bp as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_session_context(&self, session_id: i64) -> EcoachResult<LinkedSessionContext> {
        self.conn
            .query_row(
                "SELECT duration_minutes, total_questions, answered_questions, avg_response_time_ms
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    let duration_minutes: Option<i64> = row.get(0)?;
                    let total_questions: i64 = row.get(1)?;
                    let answered_questions: i64 = row.get(2)?;
                    let avg_response_time_ms: Option<i64> = row.get(3)?;
                    let estimated_wasted_seconds =
                        if let (Some(duration_minutes), Some(avg_response_time_ms)) =
                            (duration_minutes, avg_response_time_ms)
                        {
                            let expected_seconds =
                                answered_questions.max(1) * avg_response_time_ms.max(0) / 1000;
                            (duration_minutes * 60 - expected_seconds).max(0)
                        } else {
                            0
                        };
                    Ok(LinkedSessionContext {
                        duration_minutes,
                        total_questions,
                        answered_questions,
                        estimated_wasted_seconds,
                    })
                },
            )
            .map_err(|err| EcoachError::NotFound(format!("session {session_id}: {err}")))
    }

    fn load_strategy_sessions(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<StrategySessionRow>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT es.session_id, es.questions_attempted, es.questions_skipped,
                        es.questions_returned_to, es.marks_gained_from_return,
                        es.time_wasted_seconds, es.optimal_time_used_bp,
                        COALESCE(s.duration_minutes, 0)
                 FROM exam_strategy_sessions es
                 LEFT JOIN sessions s ON s.id = es.session_id
                 WHERE es.student_id = ?1 AND es.subject_id = ?2
                 ORDER BY es.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok(StrategySessionRow {
                    session_id: row.get(0)?,
                    questions_attempted: row.get(1)?,
                    questions_skipped: row.get(2)?,
                    questions_returned_to: row.get(3)?,
                    marks_gained_from_return: row.get(4)?,
                    time_wasted_seconds: row.get(5)?,
                    optimal_time_used_bp: clamp_bp(row.get::<_, i64>(6)?),
                    total_exam_minutes: match row.get::<_, i64>(7)? {
                        0 => None,
                        minutes => Some(minutes),
                    },
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(sessions)
    }

    fn count_strategy_sessions(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM exam_strategy_sessions
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn estimate_optimal_time_used_bp(
        &self,
        total_exam_minutes: i64,
        questions_attempted: i64,
        time_wasted_seconds: i64,
    ) -> BasisPoints {
        if questions_attempted <= 0 {
            return 5000;
        }

        let total_seconds = (total_exam_minutes.max(1) * 60).max(1);
        let used_seconds = total_seconds.saturating_sub(time_wasted_seconds.max(0));
        clamp_bp((used_seconds * 10_000) / total_seconds)
    }

    fn estimate_strategy_score(
        &self,
        questions_attempted: i64,
        questions_skipped: i64,
        questions_returned_to: i64,
        marks_gained_from_return: i64,
        time_wasted_seconds: i64,
        optimal_time_used_bp: BasisPoints,
    ) -> BasisPoints {
        let attempted_component = if questions_attempted > 0 {
            (questions_attempted * 6000) / (questions_attempted + questions_skipped).max(1)
        } else {
            0
        };
        let return_component = if questions_returned_to > 0 {
            (marks_gained_from_return * 2500) / questions_returned_to.max(1)
        } else {
            0
        };
        let waste_penalty = (time_wasted_seconds * 12).min(3500);
        clamp_bp(
            attempted_component + return_component + optimal_time_used_bp as i64 - waste_penalty,
        )
    }

    fn validate_strategy_type(&self, strategy_type: &str) -> EcoachResult<()> {
        match normalize_strategy_type(strategy_type).as_str() {
            "time_distribution"
            | "skip_return"
            | "elimination_practice"
            | "mark_maximization"
            | "section_pacing"
            | "pressure_management"
            | "answer_prioritization"
            | "full_strategy_drill" => Ok(()),
            _ => Err(EcoachError::Validation(format!(
                "unsupported exam strategy type: {}",
                strategy_type
            ))),
        }
    }
}

#[derive(Debug, Clone)]
struct LinkedSessionContext {
    duration_minutes: Option<i64>,
    total_questions: i64,
    answered_questions: i64,
    estimated_wasted_seconds: i64,
}

#[derive(Debug, Clone)]
struct StrategySessionRow {
    session_id: Option<i64>,
    questions_attempted: i64,
    questions_skipped: i64,
    questions_returned_to: i64,
    marks_gained_from_return: i64,
    time_wasted_seconds: i64,
    optimal_time_used_bp: BasisPoints,
    total_exam_minutes: Option<i64>,
}

fn normalize_strategy_type(strategy_type: &str) -> String {
    match strategy_type.trim().to_ascii_lowercase().as_str() {
        "time_distribution" => "time_distribution".to_string(),
        "skip_return" => "skip_return".to_string(),
        "elimination_practice" => "elimination_practice".to_string(),
        "mark_maximization" => "mark_maximization".to_string(),
        "section_pacing" => "section_pacing".to_string(),
        "pressure_management" => "pressure_management".to_string(),
        "answer_prioritization" => "answer_prioritization".to_string(),
        "full_strategy_drill" => "full_strategy_drill".to_string(),
        other => other.to_string(),
    }
}

fn build_section_time_allocation(
    total_exam_minutes: i64,
    skip_return_effectiveness_bp: BasisPoints,
    recheck_value_bp: BasisPoints,
    rushing_error_rate_bp: BasisPoints,
    overthinking_time_loss_bp: BasisPoints,
) -> Value {
    let total = total_exam_minutes.max(1);
    let review_share = if skip_return_effectiveness_bp >= 6500 || recheck_value_bp >= 6500 {
        0.28
    } else {
        0.20
    };
    let buffer_share = if overthinking_time_loss_bp >= 5500 {
        0.18
    } else if rushing_error_rate_bp >= 5500 {
        0.12
    } else {
        0.15
    };
    let first_pass_share: f64 = (1.0_f64 - review_share - buffer_share).max(0.45).min(0.72);
    let first_pass_minutes = (total as f64 * first_pass_share).round() as i64;
    let review_minutes = (total as f64 * review_share).round() as i64;
    let buffer_minutes = (total - first_pass_minutes - review_minutes).max(0);

    json!({
        "first_pass_minutes": first_pass_minutes,
        "review_minutes": review_minutes,
        "buffer_minutes": buffer_minutes,
        "notes": if skip_return_effectiveness_bp >= 6500 {
            "spend time marking easy marks early and return to the skipped items"
        } else if overthinking_time_loss_bp >= 5500 {
            "protect against overthinking with a strict first-pass timer"
        } else {
            "keep a short review reserve for second-pass checking"
        }
    })
}

fn build_section_order(
    skip_return_effectiveness_bp: BasisPoints,
    recheck_value_bp: BasisPoints,
    rushing_error_rate_bp: BasisPoints,
    overthinking_time_loss_bp: BasisPoints,
) -> Vec<String> {
    if skip_return_effectiveness_bp >= 6500 || recheck_value_bp >= 6500 {
        vec![
            "easy_marks_first".to_string(),
            "skip_and_return".to_string(),
            "review_high_value_items".to_string(),
        ]
    } else if overthinking_time_loss_bp >= 5500 {
        vec![
            "time_boxed_first_pass".to_string(),
            "secure_certain_marks".to_string(),
            "rapid_review".to_string(),
        ]
    } else if rushing_error_rate_bp >= 5500 {
        vec![
            "slow_down_on_instruction_items".to_string(),
            "answer_stability_check".to_string(),
            "final_sweep".to_string(),
        ]
    } else {
        vec![
            "first_pass".to_string(),
            "return_to_harder_items".to_string(),
            "final_review".to_string(),
        ]
    }
}

fn build_mark_maximization_strategy(
    skip_return_effectiveness_bp: BasisPoints,
    rushing_error_rate_bp: BasisPoints,
    overthinking_time_loss_bp: BasisPoints,
) -> String {
    if skip_return_effectiveness_bp >= 6500 {
        "Secure the guaranteed marks first, skip the time traps, then return for the medium-value items."
            .to_string()
    } else if overthinking_time_loss_bp >= 5500 {
        "Limit each decision to one clean pass, write the best known method, and do not over-check early."
            .to_string()
    } else if rushing_error_rate_bp >= 5500 {
        "Slow down on instruction-heavy questions, preserve working, and protect against avoidable slips."
            .to_string()
    } else {
        "Balance quick retrieval with a short review pass to capture the highest-value marks."
            .to_string()
    }
}

fn build_when_blank_strategy(
    rushing_error_rate_bp: BasisPoints,
    overthinking_time_loss_bp: BasisPoints,
) -> String {
    if overthinking_time_loss_bp >= 5500 {
        "Write the nearest valid method, label assumptions, and move on before the timer eats the question."
            .to_string()
    } else if rushing_error_rate_bp >= 5500 {
        "Pause, eliminate obvious distractors, and record the safest partial method before revisiting."
            .to_string()
    } else {
        "Use elimination, write the known first step, and leave a visible path for a later return."
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn exam_strategy_service_records_session_and_builds_profile() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let session_id = seed_session(&conn, subject_id);

        let service = ExamStrategyService::new(&conn);
        let profile = service
            .record_exam_strategy_session(ExamStrategySessionInput {
                student_id: 1,
                subject_id,
                session_id: Some(session_id),
                strategy_type: "time_distribution".into(),
                questions_attempted: Some(10),
                questions_skipped: Some(2),
                questions_returned_to: Some(2),
                marks_gained_from_return: Some(3),
                time_wasted_seconds: Some(180),
                optimal_time_used_bp: None,
                insights: Some(json!({"notes": "steady"})),
                total_exam_minutes: Some(90),
            })
            .expect("strategy session should record");

        assert_eq!(profile.student_id, 1);
        assert_eq!(profile.subject_id, subject_id);
        assert_eq!(profile.total_exam_minutes, 90);
        assert!(!profile.best_section_order.is_empty());

        let loaded = service
            .get_exam_strategy_profile(1, subject_id)
            .expect("profile should load")
            .expect("profile should exist");
        assert_eq!(loaded.session_count, 1);
        assert_eq!(loaded.total_exam_minutes, 90);
        assert!(loaded.skip_return_effectiveness_bp > 0);
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (1, '[\"MATH\"]', 60)",
            [],
        )
        .expect("student profile should insert");
    }

    fn seed_session(conn: &Connection, subject_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, duration_minutes,
                is_timed, status, started_at, total_questions, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (1, 'mock', ?1, '[1]', 12, 90, 1, 'active', datetime('now'), 12, 10, 7, 7000, 42000)",
            [subject_id],
        )
        .expect("session should insert");
        conn.last_insert_rowid()
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
