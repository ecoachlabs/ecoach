use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    CompileMockInput, ImprovementDelta, MockAnswerResult, MockGrade, MockReport, MockSession,
    MockSessionSummary, MockTopicScore, SubmitMockAnswerInput,
};

pub struct MockCentreService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct MockCandidate {
    question_id: i64,
    topic_id: i64,
    family_id: Option<i64>,
    estimated_time_seconds: i64,
    quality_score: i64,
    recurrence_score: i64,
    replacement_score: i64,
    exact_paper_match: bool,
}

impl<'a> MockCentreService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Compile mock ──

    pub fn compile_mock(&self, input: &CompileMockInput) -> EcoachResult<MockSession> {
        let now = Utc::now().to_rfc3339();
        let topic_ids_json = serde_json::to_string(&input.topic_ids)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;

        // Create the underlying session record
        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, question_count,
                    duration_minutes, is_timed, difficulty_preference, status, created_at, updated_at
                 ) VALUES (?1, 'mock', ?2, ?3, ?4, ?5, 1, 'exam', 'created', ?6, ?6)",
                params![
                    input.student_id,
                    input.subject_id,
                    topic_ids_json,
                    input.question_count as i64,
                    input.duration_minutes,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let session_id = self.conn.last_insert_rowid();

        // Create the mock_sessions record
        self.conn
            .execute(
                "INSERT INTO mock_sessions (
                    student_id, subject_id, session_id, status, duration_minutes,
                    question_count, paper_year, created_at
                 ) VALUES (?1, ?2, ?3, 'created', ?4, ?5, ?6, ?7)",
                params![
                    input.student_id,
                    input.subject_id,
                    session_id,
                    input.duration_minutes,
                    input.question_count as i64,
                    input.paper_year,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mock_session_id = self.conn.last_insert_rowid();

        // Select questions for the mock paper
        self.populate_mock_questions(mock_session_id, session_id, input)?;

        self.append_event(DomainEvent::new(
            "mock.compiled",
            mock_session_id.to_string(),
            json!({
                "student_id": input.student_id,
                "subject_id": input.subject_id,
                "question_count": input.question_count,
                "duration_minutes": input.duration_minutes,
            }),
        ))?;

        self.get_mock_session(mock_session_id)
    }

    fn populate_mock_questions(
        &self,
        mock_session_id: i64,
        session_id: i64,
        input: &CompileMockInput,
    ) -> EcoachResult<()> {
        let candidates = self.load_mock_candidates(input)?;
        let questions = self.select_mock_questions(&candidates, input);

        // Insert session_items for the mock
        for (display_order, (question_id, topic_id)) in questions.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO session_items (
                        session_id, question_id, display_order, source_topic_id, status, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, 'queued', datetime('now'), datetime('now'))",
                    params![session_id, question_id, display_order as i64, topic_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // Update actual question count in case fewer questions were available
        let actual_count = questions.len() as i64;
        self.conn
            .execute(
                "UPDATE sessions SET question_count = ?1, total_questions = ?1 WHERE id = ?2",
                params![actual_count, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        self.conn
            .execute(
                "UPDATE mock_sessions SET question_count = ?1 WHERE id = ?2",
                params![actual_count, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    fn load_mock_candidates(&self, input: &CompileMockInput) -> EcoachResult<Vec<MockCandidate>> {
        let paper_year_filter = input.paper_year.clone().unwrap_or_default();
        let sql = if input.topic_ids.is_empty() {
            "SELECT q.id, q.topic_id, q.family_id, q.estimated_time_seconds,
                    COALESCE(qfh.quality_score, 5500),
                    COALESCE(qfa.recurrence_score, 0),
                    COALESCE(qfa.replacement_score, 0),
                    EXISTS(
                        SELECT 1
                        FROM past_paper_question_links ppql
                        INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                        WHERE ppql.question_id = q.id
                          AND (?2 = '' OR CAST(pps.exam_year AS TEXT) = ?2)
                    ) AS exact_paper_match
             FROM questions q
             LEFT JOIN question_family_health qfh ON qfh.family_id = q.family_id
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = q.family_id
             WHERE q.subject_id = ?1 AND q.is_active = 1"
                .to_string()
        } else {
            let placeholders = input
                .topic_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "SELECT q.id, q.topic_id, q.family_id, q.estimated_time_seconds,
                        COALESCE(qfh.quality_score, 5500),
                        COALESCE(qfa.recurrence_score, 0),
                        COALESCE(qfa.replacement_score, 0),
                        EXISTS(
                            SELECT 1
                            FROM past_paper_question_links ppql
                            INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                            WHERE ppql.question_id = q.id
                              AND (?2 = '' OR CAST(pps.exam_year AS TEXT) = ?2)
                        ) AS exact_paper_match
                 FROM questions q
                 LEFT JOIN question_family_health qfh ON qfh.family_id = q.family_id
                 LEFT JOIN question_family_analytics qfa ON qfa.family_id = q.family_id
                 WHERE q.subject_id = ?1 AND q.is_active = 1 AND q.topic_id IN ({})",
                placeholders
            )
        };

        let mut params_vec: Vec<rusqlite::types::Value> =
            Vec::with_capacity(input.topic_ids.len() + 2);
        params_vec.push(input.subject_id.into());
        params_vec.push(paper_year_filter.clone().into());
        for topic_id in &input.topic_ids {
            params_vec.push((*topic_id).into());
        }

        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok(MockCandidate {
                    question_id: row.get(0)?,
                    topic_id: row.get(1)?,
                    family_id: row.get(2)?,
                    estimated_time_seconds: row.get(3)?,
                    quality_score: row.get(4)?,
                    recurrence_score: row.get(5)?,
                    replacement_score: row.get(6)?,
                    exact_paper_match: row.get::<_, i64>(7)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut candidates = Vec::new();
        for row in rows {
            candidates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(candidates)
    }

    fn select_mock_questions(
        &self,
        candidates: &[MockCandidate],
        input: &CompileMockInput,
    ) -> Vec<(i64, i64)> {
        if candidates.is_empty() {
            return Vec::new();
        }

        let mut topic_quotas = BTreeMap::new();
        let mut ordered_topics = if input.topic_ids.is_empty() {
            let mut topics = candidates
                .iter()
                .map(|item| item.topic_id)
                .collect::<Vec<_>>();
            topics.sort_unstable();
            topics.dedup();
            topics
        } else {
            input.topic_ids.clone()
        };
        ordered_topics.sort_unstable();
        ordered_topics.dedup();

        let base = (input.question_count / ordered_topics.len().max(1)) as i64;
        let remainder = input.question_count % ordered_topics.len().max(1);
        for (index, topic_id) in ordered_topics.iter().enumerate() {
            let quota = base + if index < remainder { 1 } else { 0 };
            topic_quotas.insert(*topic_id, quota.max(1));
        }

        let mut ranked = candidates.to_vec();
        ranked.sort_by(|left, right| {
            score_mock_candidate(right)
                .cmp(&score_mock_candidate(left))
                .then(right.question_id.cmp(&left.question_id))
        });

        let mut selected = Vec::new();
        let mut selected_question_ids = BTreeSet::new();
        let mut family_counts: BTreeMap<i64, i64> = BTreeMap::new();
        let family_cap = ((input.question_count as i64 + 1) / 2).max(1);

        for candidate in &ranked {
            if selected.len() >= input.question_count {
                break;
            }
            if selected_question_ids.contains(&candidate.question_id) {
                continue;
            }
            let remaining_quota = topic_quotas.get(&candidate.topic_id).copied().unwrap_or(0);
            if remaining_quota <= 0 {
                continue;
            }
            if let Some(family_id) = candidate.family_id {
                if family_counts.get(&family_id).copied().unwrap_or(0) >= family_cap {
                    continue;
                }
                *family_counts.entry(family_id).or_insert(0) += 1;
            }
            *topic_quotas.entry(candidate.topic_id).or_insert(0) -= 1;
            selected_question_ids.insert(candidate.question_id);
            selected.push((candidate.question_id, candidate.topic_id));
        }

        if selected.len() < input.question_count {
            for candidate in &ranked {
                if selected.len() >= input.question_count {
                    break;
                }
                if selected_question_ids.insert(candidate.question_id) {
                    selected.push((candidate.question_id, candidate.topic_id));
                }
            }
        }

        selected
    }

    // ── Start timed execution ──

    pub fn start_mock(&self, mock_session_id: i64) -> EcoachResult<MockSession> {
        let now = Utc::now().to_rfc3339();
        let affected = self
            .conn
            .execute(
                "UPDATE mock_sessions SET status = 'active', started_at = ?1 WHERE id = ?2 AND status = 'created'",
                params![now, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if affected == 0 {
            return Err(EcoachError::Validation(
                "mock session cannot be started (wrong state or not found)".to_string(),
            ));
        }

        // Also start the underlying session
        let session_id: i64 = self
            .conn
            .query_row(
                "SELECT session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE sessions SET status = 'active', started_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.append_event(DomainEvent::new(
            "mock.started",
            mock_session_id.to_string(),
            json!({ "started_at": now }),
        ))?;

        self.get_mock_session(mock_session_id)
    }

    // ── Submit answer ──

    pub fn submit_answer(&self, input: &SubmitMockAnswerInput) -> EcoachResult<MockAnswerResult> {
        let mock = self.get_mock_session(input.mock_session_id)?;
        if mock.status != "active" {
            return Err(EcoachError::Validation(
                "mock session is not active".to_string(),
            ));
        }

        // Check time remaining
        if let Some(remaining) = mock.time_remaining_seconds {
            if remaining <= 0 {
                self.force_time_up(input.mock_session_id)?;
                return Err(EcoachError::Validation("time is up".to_string()));
            }
        }

        // Check correctness
        let is_correct: bool = self
            .conn
            .query_row(
                "SELECT is_correct FROM question_options WHERE id = ?1 AND question_id = ?2",
                params![input.selected_option_id, input.question_id],
                |row| Ok(row.get::<_, i64>(0)? == 1),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Update session_item
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE session_items
                 SET status = 'answered', selected_option_id = ?1, is_correct = ?2,
                     answered_at = ?3, updated_at = ?3
                 WHERE session_id = ?4 AND question_id = ?5 AND status IN ('queued', 'presented')",
                params![
                    input.selected_option_id,
                    if is_correct { 1 } else { 0 },
                    now,
                    mock.session_id,
                    input.question_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Update counters on underlying session
        self.conn
            .execute(
                "UPDATE sessions SET answered_questions = answered_questions + 1,
                     correct_questions = correct_questions + CASE WHEN ?1 THEN 1 ELSE 0 END,
                     updated_at = ?2
                 WHERE id = ?3",
                params![is_correct, now, mock.session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let answered_count = mock.answered_count + 1;
        let remaining_count = mock.question_count - answered_count;

        // Check if mock is done
        if remaining_count <= 0 {
            self.complete_mock(input.mock_session_id)?;
        }

        Ok(MockAnswerResult {
            question_id: input.question_id,
            was_correct: is_correct,
            answered_count,
            remaining_count,
            time_remaining_seconds: self
                .get_mock_session(input.mock_session_id)?
                .time_remaining_seconds,
        })
    }

    // ── Timer management ──

    pub fn pause_mock(&self, mock_session_id: i64) -> EcoachResult<MockSession> {
        let mock = self.get_mock_session(mock_session_id)?;
        if mock.status != "active" {
            return Err(EcoachError::Validation("mock is not active".to_string()));
        }

        // Store remaining time so we can resume correctly
        let remaining = mock.time_remaining_seconds.unwrap_or(0);
        self.conn
            .execute(
                "UPDATE mock_sessions SET status = 'paused', time_banked_seconds = ?1 WHERE id = ?2",
                params![remaining, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE sessions SET status = 'paused', paused_at = datetime('now') WHERE id = ?1",
                [mock.session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.get_mock_session(mock_session_id)
    }

    pub fn resume_mock(&self, mock_session_id: i64) -> EcoachResult<MockSession> {
        let now = Utc::now().to_rfc3339();

        // Reset started_at to now, but with banked time factored in
        let (session_id, banked): (i64, i64) = self
            .conn
            .query_row(
                "SELECT session_id, COALESCE(time_banked_seconds, 0) FROM mock_sessions WHERE id = ?1 AND status = 'paused'",
                [mock_session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| EcoachError::Validation(format!("cannot resume: {}", e)))?;

        self.conn
            .execute(
                "UPDATE mock_sessions SET status = 'active', resumed_at = ?1, time_banked_seconds = ?2 WHERE id = ?3",
                params![now, banked, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE sessions SET status = 'active', updated_at = ?1 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.get_mock_session(mock_session_id)
    }

    fn force_time_up(&self, mock_session_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE mock_sessions SET status = 'time_up' WHERE id = ?1",
                [mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        self.complete_mock(mock_session_id)
    }

    fn complete_mock(&self, mock_session_id: i64) -> EcoachResult<()> {
        let now = Utc::now().to_rfc3339();
        let session_id: i64 = self
            .conn
            .query_row(
                "SELECT session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE mock_sessions SET status = 'completed', completed_at = ?1 WHERE id = ?2 AND status IN ('active', 'time_up')",
                params![now, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE sessions SET status = 'completed', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.append_event(DomainEvent::new(
            "mock.completed",
            mock_session_id.to_string(),
            json!({ "completed_at": now }),
        ))?;

        Ok(())
    }

    pub fn abandon_mock(&self, mock_session_id: i64) -> EcoachResult<()> {
        let session_id: i64 = self
            .conn
            .query_row(
                "SELECT session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE mock_sessions SET status = 'abandoned', completed_at = ?1 WHERE id = ?2 AND status IN ('created', 'active', 'paused')",
                params![now, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE sessions SET status = 'abandoned', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    // ── Grading / Report ──

    pub fn get_report(&self, mock_session_id: i64) -> EcoachResult<MockReport> {
        let mock = self.get_mock_session(mock_session_id)?;
        if mock.status != "completed" && mock.status != "time_up" && mock.status != "abandoned" {
            return Err(EcoachError::Validation(
                "mock has not been completed yet".to_string(),
            ));
        }

        // Get answer stats from session_items
        let (total, correct, answered): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END),
                        SUM(CASE WHEN status = 'answered' THEN 1 ELSE 0 END)
                 FROM session_items WHERE session_id = ?1",
                [mock.session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let percentage = if total > 0 {
            (correct as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let grade = MockGrade::from_percentage(percentage);
        let accuracy_bp: BasisPoints = if total > 0 {
            to_bp(correct as f64 / total as f64)
        } else {
            0
        };

        // Calculate time used
        let time_used_seconds = self.calculate_time_used(&mock)?;
        let total_time_seconds = mock.duration_minutes * 60;

        // Topic breakdown
        let topic_breakdown = self.get_topic_breakdown(mock.session_id)?;

        // Compare with last mock
        let improvement = self.calculate_improvement(
            mock.student_id,
            mock.subject_id,
            mock_session_id,
            percentage,
        )?;

        // Store the grade in mock_sessions
        self.conn
            .execute(
                "UPDATE mock_sessions SET grade = ?1, percentage = ?2 WHERE id = ?3",
                params![grade.as_str(), percentage, mock_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(MockReport {
            mock_session_id,
            student_id: mock.student_id,
            subject_id: mock.subject_id,
            grade: grade.as_str().to_string(),
            total_score: correct,
            max_score: total,
            percentage,
            accuracy_bp,
            time_used_seconds,
            total_time_seconds,
            questions_answered: answered,
            questions_correct: correct,
            questions_unanswered: total - answered,
            topic_breakdown,
            improvement_vs_last: improvement,
        })
    }

    fn get_topic_breakdown(&self, session_id: i64) -> EcoachResult<Vec<MockTopicScore>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT si.source_topic_id, COALESCE(t.name, 'Unknown'), COUNT(*),
                        SUM(CASE WHEN si.is_correct = 1 THEN 1 ELSE 0 END)
                 FROM session_items si
                 LEFT JOIN topics t ON t.id = si.source_topic_id
                 WHERE si.session_id = ?1 AND si.source_topic_id IS NOT NULL
                 GROUP BY si.source_topic_id
                 ORDER BY t.name",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| {
                let topic_id: i64 = row.get(0)?;
                let topic_name: String = row.get(1)?;
                let total: i64 = row.get(2)?;
                let correct: i64 = row.get(3)?;
                let accuracy_bp = if total > 0 {
                    to_bp(correct as f64 / total as f64)
                } else {
                    0
                };
                Ok(MockTopicScore {
                    topic_id,
                    topic_name,
                    correct,
                    total,
                    accuracy_bp,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut scores = Vec::new();
        for row in rows {
            scores.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(scores)
    }

    fn calculate_time_used(&self, mock: &MockSession) -> EcoachResult<i64> {
        if let Some(started) = &mock.started_at {
            let fallback = Utc::now().to_rfc3339();
            let end_time = mock.completed_at.as_deref().unwrap_or(&fallback);
            if let (Ok(start), Ok(end)) = (
                chrono::DateTime::parse_from_rfc3339(started),
                chrono::DateTime::parse_from_rfc3339(end_time),
            ) {
                let diff = end.signed_duration_since(start);
                return Ok(diff.num_seconds().max(0));
            }
        }
        Ok(0)
    }

    fn calculate_improvement(
        &self,
        student_id: i64,
        subject_id: i64,
        current_mock_id: i64,
        current_percentage: f64,
    ) -> EcoachResult<Option<ImprovementDelta>> {
        let previous: Option<(String, f64)> = self
            .conn
            .query_row(
                "SELECT grade, percentage FROM mock_sessions
                 WHERE student_id = ?1 AND subject_id = ?2 AND id != ?3
                   AND status = 'completed' AND grade IS NOT NULL
                 ORDER BY completed_at DESC LIMIT 1",
                params![student_id, subject_id, current_mock_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(previous.map(|(prev_grade, prev_pct)| {
            let delta = current_percentage - prev_pct;
            let direction = if delta > 1.0 {
                "improved"
            } else if delta < -1.0 {
                "declined"
            } else {
                "stable"
            };
            ImprovementDelta {
                previous_grade: prev_grade,
                previous_percentage: prev_pct,
                delta_percentage: delta,
                direction: direction.to_string(),
            }
        }))
    }

    // ── Queries ──

    pub fn get_mock_session(&self, mock_session_id: i64) -> EcoachResult<MockSession> {
        let row = self
            .conn
            .query_row(
                "SELECT ms.id, ms.student_id, ms.subject_id, ms.session_id, ms.status,
                        ms.duration_minutes, ms.question_count, ms.paper_year,
                        ms.started_at, ms.completed_at, ms.created_at,
                        COALESCE(ms.time_banked_seconds, 0), ms.resumed_at,
                        s.answered_questions
                 FROM mock_sessions ms
                 INNER JOIN sessions s ON s.id = ms.session_id
                 WHERE ms.id = ?1",
                [mock_session_id],
                |row| {
                    let status: String = row.get(4)?;
                    let duration_min: i64 = row.get(5)?;
                    let started_at: Option<String> = row.get(8)?;
                    let banked_seconds: i64 = row.get(11)?;
                    let resumed_at: Option<String> = row.get(12)?;
                    let answered: i64 = row.get(13)?;

                    // Calculate time remaining
                    let time_remaining = if status == "active" {
                        let ref_time = resumed_at.as_deref().or(started_at.as_deref());
                        if let Some(ref_str) = ref_time {
                            if let Ok(ref_dt) = chrono::DateTime::parse_from_rfc3339(ref_str) {
                                let elapsed = Utc::now()
                                    .signed_duration_since(ref_dt.with_timezone(&Utc))
                                    .num_seconds();
                                let total_allowed = if banked_seconds > 0 {
                                    banked_seconds
                                } else {
                                    duration_min * 60
                                };
                                Some((total_allowed - elapsed).max(0))
                            } else {
                                None
                            }
                        } else {
                            Some(duration_min * 60)
                        }
                    } else if status == "paused" {
                        Some(banked_seconds)
                    } else {
                        None
                    };

                    Ok(MockSession {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        subject_id: row.get(2)?,
                        session_id: row.get(3)?,
                        status,
                        duration_minutes: duration_min,
                        time_remaining_seconds: time_remaining,
                        question_count: row.get(6)?,
                        answered_count: answered,
                        paper_year: row.get(7)?,
                        started_at,
                        completed_at: row.get(9)?,
                        created_at: row.get(10)?,
                    })
                },
            )
            .map_err(|e| {
                EcoachError::NotFound(format!("mock session {} not found: {}", mock_session_id, e))
            })?;

        Ok(row)
    }

    pub fn list_mock_sessions(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<MockSessionSummary>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject_id, grade, percentage, status, paper_year, created_at
                 FROM mock_sessions
                 WHERE student_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, limit as i64], |row| {
                Ok(MockSessionSummary {
                    id: row.get(0)?,
                    subject_id: row.get(1)?,
                    grade: row.get(2)?,
                    percentage: row.get(3)?,
                    status: row.get(4)?,
                    paper_year: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(sessions)
    }

    // ── Internal ──

    fn append_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'mock', ?3, ?4, ?5, ?6)",
                params![
                    event.event_id,
                    event.event_type,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }
}

fn score_mock_candidate(candidate: &MockCandidate) -> i64 {
    let paper_match = if candidate.exact_paper_match {
        3_200
    } else {
        0
    };
    let pacing_bonus = if candidate.estimated_time_seconds <= 90 {
        800
    } else if candidate.estimated_time_seconds <= 150 {
        550
    } else {
        250
    };
    paper_match
        + (candidate.quality_score / 5)
        + (candidate.recurrence_score / 4)
        + (candidate.replacement_score / 3)
        + pacing_bonus
}
