use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use ecoach_coach_brain::ReadinessEngine;
use ecoach_questions::{
    Question, QuestionSelectionRequest, QuestionSelector, QuestionService, SelectedQuestion,
};
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    CustomTestStartInput, MockBlueprint, MockBlueprintInput, PracticeSessionStartInput, Session,
    SessionAnswerInput, SessionItem, SessionSnapshot, SessionSummary,
};

pub struct SessionService<'a> {
    conn: &'a Connection,
}

impl<'a> SessionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn start_practice_session(
        &self,
        input: &PracticeSessionStartInput,
    ) -> EcoachResult<(Session, Vec<SelectedQuestion>)> {
        let selector = QuestionSelector::new(self.conn);
        let questions = selector.select_questions(&QuestionSelectionRequest {
            subject_id: input.subject_id,
            topic_ids: input.topic_ids.clone(),
            target_question_count: input.question_count,
            target_difficulty: None,
            weakness_topic_ids: input.topic_ids.clone(),
            recently_seen_question_ids: Vec::new(),
            timed: input.is_timed,
        })?;
        if questions.is_empty() {
            return Err(EcoachError::NotFound(
                "no questions available for requested practice session".to_string(),
            ));
        }

        let topic_ids_json = serde_json::to_string(&input.topic_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                    is_timed, status, started_at, last_activity_at
                 ) VALUES (?1, 'practice', ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?7)",
                params![
                    input.student_id,
                    input.subject_id,
                    topic_ids_json,
                    input.question_count as i64,
                    questions.len() as i64,
                    if input.is_timed { 1 } else { 0 },
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let session_id = self.conn.last_insert_rowid();
        self.persist_selected_items(session_id, &questions)?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.created",
                session_id.to_string(),
                serde_json::json!({
                    "student_id": input.student_id,
                    "session_type": "practice",
                    "subject_id": input.subject_id,
                    "question_count": questions.len(),
                }),
            ),
        )?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.started",
                session_id.to_string(),
                serde_json::json!({
                    "student_id": input.student_id,
                    "started_at": now,
                }),
            ),
        )?;

        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        Ok((session, questions))
    }

    pub fn start_custom_test(
        &self,
        input: &CustomTestStartInput,
    ) -> EcoachResult<(Session, Vec<SelectedQuestion>)> {
        let topic_scope = self.resolve_custom_test_topic_scope(input)?;
        let weakness_topic_ids = if input.weakness_bias {
            let weak_topics = self.load_weakness_topic_ids(input.student_id, input.subject_id)?;
            if weak_topics.is_empty() {
                topic_scope.clone()
            } else {
                weak_topics
            }
        } else {
            topic_scope.clone()
        };

        let selector = QuestionSelector::new(self.conn);
        let questions = selector.select_questions(&QuestionSelectionRequest {
            subject_id: input.subject_id,
            topic_ids: topic_scope.clone(),
            target_question_count: input.question_count,
            target_difficulty: input.target_difficulty,
            weakness_topic_ids,
            recently_seen_question_ids: Vec::new(),
            timed: input.is_timed,
        })?;
        if questions.is_empty() {
            return Err(EcoachError::NotFound(
                "no questions available for requested custom test".to_string(),
            ));
        }

        let archetype = self.resolve_custom_test_archetype(input, &topic_scope);
        let topic_ids_json = serde_json::to_string(&topic_scope)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                    duration_minutes, is_timed, difficulty_preference, status, started_at, last_activity_at
                 ) VALUES (?1, 'custom_test', ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    input.student_id,
                    input.subject_id,
                    topic_ids_json,
                    input.question_count as i64,
                    questions.len() as i64,
                    input.duration_minutes,
                    if input.is_timed { 1 } else { 0 },
                    archetype,
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let session_id = self.conn.last_insert_rowid();
        self.persist_selected_items(session_id, &questions)?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "custom_test.composed",
                session_id.to_string(),
                serde_json::json!({
                    "student_id": input.student_id,
                    "subject_id": input.subject_id,
                    "topic_ids": topic_scope,
                    "archetype": archetype,
                    "question_count": questions.len(),
                    "timed": input.is_timed,
                    "target_difficulty": input.target_difficulty,
                }),
            ),
        )?;

        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        Ok((session, questions))
    }

    pub fn generate_mock_blueprint(
        &self,
        input: &MockBlueprintInput,
    ) -> EcoachResult<MockBlueprint> {
        let topic_scope = self.resolve_mock_topic_scope(input)?;
        let readiness = ReadinessEngine::new(self.conn)
            .build_subject_readiness(input.student_id, input.subject_id)?;
        let question_service = QuestionService::new(self.conn);
        let recent_ids = self.load_recently_seen_question_ids(input.student_id, 40)?;
        let mut candidates =
            question_service.list_questions_for_scope(input.subject_id, &topic_scope)?;
        if !recent_ids.is_empty() {
            candidates.retain(|question| !recent_ids.contains(&question.id));
        }
        if candidates.is_empty() {
            candidates =
                question_service.list_questions_for_scope(input.subject_id, &topic_scope)?;
        }
        if candidates.is_empty() {
            return Err(EcoachError::NotFound(
                "no questions available to compile a mock blueprint".to_string(),
            ));
        }

        let quotas = self.build_mock_topic_quotas(
            input.student_id,
            &topic_scope,
            input.question_count,
            &readiness,
        )?;
        let compiled_questions = self.compile_mock_questions(
            &candidates,
            &quotas,
            input.target_difficulty,
            input.is_timed,
        );
        if compiled_questions.is_empty() {
            return Err(EcoachError::Validation(
                "mock blueprint compilation produced no questions".to_string(),
            ));
        }

        let blueprint_type =
            if input.is_timed && readiness.recommended_mock_blueprint == "balanced_mock" {
                "pressure_mock".to_string()
            } else {
                readiness.recommended_mock_blueprint.clone()
            };
        let compiled_question_ids = compiled_questions
            .iter()
            .map(|question| question.id)
            .collect::<Vec<_>>();
        let quota_json = json!({
            "is_timed": input.is_timed,
            "target_difficulty": input.target_difficulty,
            "topics": quotas.iter().map(|quota| {
                json!({
                    "topic_id": quota.topic_id,
                    "target_count": quota.target_count,
                    "priority_weight": quota.priority_weight,
                    "is_weak_topic": quota.is_weak_topic,
                })
            }).collect::<Vec<_>>(),
        });
        let coverage_json = self.build_mock_coverage(&quotas, &compiled_questions, &readiness)?;

        self.conn
            .execute(
                "INSERT INTO mock_blueprints (
                    student_id, subject_id, title, blueprint_type, duration_minutes, question_count,
                    readiness_score, readiness_band, coverage_json, quota_json,
                    compiled_question_ids_json, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'compiled')",
                params![
                    input.student_id,
                    input.subject_id,
                    format!(
                        "{} Mock Blueprint",
                        title_for_mock_blueprint(&blueprint_type)
                    ),
                    blueprint_type,
                    input.duration_minutes,
                    compiled_questions.len() as i64,
                    readiness.readiness_score,
                    readiness.readiness_band,
                    serde_json::to_string(&coverage_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&quota_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&compiled_question_ids)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let blueprint_id = self.conn.last_insert_rowid();

        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "mock.blueprint_compiled",
                blueprint_id.to_string(),
                json!({
                    "student_id": input.student_id,
                    "subject_id": input.subject_id,
                    "blueprint_type": blueprint_type,
                    "compiled_question_count": compiled_question_ids.len(),
                    "readiness_score": readiness.readiness_score,
                }),
            ),
        )?;

        self.get_mock_blueprint(blueprint_id)?
            .ok_or_else(|| EcoachError::NotFound("mock blueprint was not created".to_string()))
    }

    pub fn start_mock_session(
        &self,
        blueprint_id: i64,
    ) -> EcoachResult<(Session, Vec<SelectedQuestion>)> {
        let blueprint = self.get_mock_blueprint(blueprint_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("mock blueprint {} not found", blueprint_id))
        })?;
        if blueprint.compiled_question_ids.is_empty() {
            return Err(EcoachError::Validation(
                "mock blueprint has no compiled questions".to_string(),
            ));
        }
        let topic_ids = blueprint
            .coverage
            .get("topic_scope")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
            .unwrap_or_default();
        let is_timed = blueprint
            .quotas
            .get("is_timed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                    duration_minutes, is_timed, difficulty_preference, status, started_at, last_activity_at
                 ) VALUES (?1, 'mock', ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9)",
                params![
                    blueprint.student_id,
                    blueprint.subject_id,
                    serde_json::to_string(&topic_ids)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    blueprint.question_count,
                    blueprint.compiled_question_ids.len() as i64,
                    blueprint.duration_minutes,
                    if is_timed { 1 } else { 0 },
                    blueprint.blueprint_type,
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let session_id = self.conn.last_insert_rowid();

        let question_service = QuestionService::new(self.conn);
        let mut selected_questions = Vec::new();
        for (index, question_id) in blueprint.compiled_question_ids.iter().enumerate() {
            let question = question_service
                .get_question(*question_id)?
                .ok_or_else(|| {
                    EcoachError::NotFound(format!("question {} missing", question_id))
                })?;
            self.conn
                .execute(
                    "INSERT INTO session_items (
                        session_id, question_id, display_order, source_family_id, source_topic_id, status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 'queued')",
                    params![
                        session_id,
                        question.id,
                        (index + 1) as i64,
                        question.family_id,
                        question.topic_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            selected_questions.push(SelectedQuestion {
                question,
                fit_score: 1.0,
            });
        }

        self.conn
            .execute(
                "UPDATE mock_blueprints
                 SET status = 'used', updated_at = datetime('now')
                 WHERE id = ?1",
                [blueprint_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "mock.session_created",
                session_id.to_string(),
                json!({
                    "blueprint_id": blueprint_id,
                    "student_id": blueprint.student_id,
                    "subject_id": blueprint.subject_id,
                    "question_count": blueprint.compiled_question_ids.len(),
                }),
            ),
        )?;

        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        Ok((session, selected_questions))
    }

    pub fn get_session_snapshot(&self, session_id: i64) -> EcoachResult<Option<SessionSnapshot>> {
        let Some(session) = self.get_session(session_id)? else {
            return Ok(None);
        };
        let items = self.list_session_items(session_id)?;
        Ok(Some(SessionSnapshot { session, items }))
    }

    pub fn pause_session(&self, session_id: i64) -> EcoachResult<Session> {
        self.ensure_session_status(session_id, &["active"])?;
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE sessions
                 SET status = 'paused', paused_at = ?1, last_activity_at = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.paused",
                session_id.to_string(),
                serde_json::json!({ "paused_at": now }),
            ),
        )?;
        self.get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))
    }

    pub fn resume_session(&self, session_id: i64) -> EcoachResult<SessionSnapshot> {
        self.ensure_session_status(session_id, &["paused", "active"])?;
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE sessions
                 SET status = 'active', paused_at = NULL, last_activity_at = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.resumed",
                session_id.to_string(),
                serde_json::json!({ "resumed_at": now }),
            ),
        )?;
        self.get_session_snapshot(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))
    }

    pub fn record_answer(
        &self,
        session_id: i64,
        input: &SessionAnswerInput,
    ) -> EcoachResult<SessionItem> {
        self.ensure_session_status(session_id, &["active"])?;

        let item = self.get_session_item(input.item_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("session item {} not found", input.item_id))
        })?;
        if item.session_id != session_id {
            return Err(EcoachError::Validation(format!(
                "session item {} does not belong to session {}",
                input.item_id, session_id
            )));
        }

        let question_service = QuestionService::new(self.conn);
        let option = question_service
            .get_option(input.selected_option_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("option {} not found", input.selected_option_id))
            })?;
        if option.question_id != item.question_id {
            return Err(EcoachError::Validation(format!(
                "option {} does not belong to question {}",
                input.selected_option_id, item.question_id
            )));
        }

        let answered_at = Utc::now().to_rfc3339();
        let answer_state_json = serde_json::json!({
            "selected_option_id": input.selected_option_id,
            "response_time_ms": input.response_time_ms,
        })
        .to_string();

        self.conn
            .execute(
                "UPDATE session_items
                 SET status = 'answered',
                     selected_option_id = ?1,
                     answer_state_json = ?2,
                     answered_at = ?3,
                     response_time_ms = ?4,
                     is_correct = ?5,
                     updated_at = datetime('now')
                 WHERE id = ?6",
                params![
                    input.selected_option_id,
                    answer_state_json,
                    answered_at,
                    input.response_time_ms,
                    if option.is_correct { 1 } else { 0 },
                    input.item_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.refresh_session_progress(session_id, item.display_order)?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.answer_recorded",
                session_id.to_string(),
                serde_json::json!({
                    "item_id": item.id,
                    "question_id": item.question_id,
                    "selected_option_id": input.selected_option_id,
                    "is_correct": option.is_correct,
                }),
            ),
        )?;

        self.get_session_item(input.item_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("session item {} not found", input.item_id))
        })
    }

    pub fn flag_session_item(
        &self,
        session_id: i64,
        item_id: i64,
        flagged: bool,
    ) -> EcoachResult<SessionItem> {
        self.ensure_session_status(session_id, &["active", "paused"])?;
        let item = self
            .get_session_item(item_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session item {} not found", item_id)))?;
        if item.session_id != session_id {
            return Err(EcoachError::Validation(format!(
                "session item {} does not belong to session {}",
                item_id, session_id
            )));
        }

        self.conn
            .execute(
                "UPDATE session_items
                 SET flagged = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![if flagged { 1 } else { 0 }, item_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.question_flagged",
                session_id.to_string(),
                serde_json::json!({
                    "item_id": item_id,
                    "flagged": flagged,
                }),
            ),
        )?;

        self.get_session_item(item_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session item {} not found", item_id)))
    }

    pub fn complete_session(&self, session_id: i64) -> EcoachResult<SessionSummary> {
        let summary = self.build_summary(session_id)?;
        let completed_at = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE sessions
                 SET status = 'completed', completed_at = ?1, last_activity_at = ?1,
                     answered_questions = ?2, correct_questions = ?3, accuracy_score = ?4,
                     updated_at = datetime('now')
                 WHERE id = ?5",
                params![
                    completed_at,
                    summary.answered_questions,
                    summary.correct_questions,
                    summary.accuracy_score,
                    session_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.submitted",
                session_id.to_string(),
                serde_json::json!({
                    "answered_questions": summary.answered_questions,
                    "correct_questions": summary.correct_questions,
                    "accuracy_score": summary.accuracy_score,
                }),
            ),
        )?;
        self.build_summary(session_id)
    }

    pub fn get_session(&self, session_id: i64) -> EcoachResult<Option<Session>> {
        self.conn
            .query_row(
                "SELECT id, student_id, session_type, subject_id, status, active_item_index,
                        started_at, paused_at, completed_at, last_activity_at
                 FROM sessions WHERE id = ?1",
                [session_id],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        session_type: row.get(2)?,
                        subject_id: row.get(3)?,
                        status: row.get(4)?,
                        active_item_index: row.get(5)?,
                        started_at: parse_datetime(row.get::<_, Option<String>>(6)?),
                        paused_at: parse_datetime(row.get::<_, Option<String>>(7)?),
                        completed_at: parse_datetime(row.get::<_, Option<String>>(8)?),
                        last_activity_at: parse_datetime(row.get::<_, Option<String>>(9)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn build_summary(&self, session_id: i64) -> EcoachResult<SessionSummary> {
        let (mut answered_questions, mut correct_questions): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), 0)
                 FROM session_items
                 WHERE session_id = ?1 AND selected_option_id IS NOT NULL",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if answered_questions == 0 {
            (answered_questions, correct_questions) = self
                .conn
                .query_row(
                    "SELECT COUNT(*), COALESCE(SUM(is_correct), 0)
                     FROM student_question_attempts
                     WHERE session_id = ?1",
                    [session_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        let status: String = self
            .conn
            .query_row(
                "SELECT status FROM sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let accuracy_score = if answered_questions > 0 {
            Some(((correct_questions as f64 / answered_questions as f64) * 10_000.0).round() as i64)
        } else {
            None
        };

        Ok(SessionSummary {
            session_id,
            accuracy_score,
            answered_questions,
            correct_questions,
            status,
        })
    }

    fn persist_selected_items(
        &self,
        session_id: i64,
        questions: &[SelectedQuestion],
    ) -> EcoachResult<()> {
        for (index, selected) in questions.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO session_items (
                        session_id, question_id, display_order, source_family_id, source_topic_id, status
                    ) VALUES (?1, ?2, ?3, ?4, ?5, 'queued')",
                    params![
                        session_id,
                        selected.question.id,
                        (index + 1) as i64,
                        selected.question.family_id,
                        selected.question.topic_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn resolve_custom_test_topic_scope(
        &self,
        input: &CustomTestStartInput,
    ) -> EcoachResult<Vec<i64>> {
        if !input.topic_ids.is_empty() {
            return Ok(input.topic_ids.clone());
        }

        let weak_topics = if input.weakness_bias {
            self.load_weakness_topic_ids(input.student_id, input.subject_id)?
        } else {
            Vec::new()
        };
        if !weak_topics.is_empty() {
            return Ok(weak_topics);
        }

        self.load_default_subject_topics(input.subject_id)
    }

    fn load_weakness_topic_ids(&self, student_id: i64, subject_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id
                 FROM student_topic_states sts
                 JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC
                 LIMIT 5",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_ids = Vec::new();
        for row in rows {
            topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(topic_ids)
    }

    fn load_default_subject_topics(&self, subject_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT topic_id
                 FROM questions
                 WHERE subject_id = ?1
                 ORDER BY topic_id ASC
                 LIMIT 5",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_ids = Vec::new();
        for row in rows {
            topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        if topic_ids.is_empty() {
            return Err(EcoachError::NotFound(format!(
                "no topics with questions found for subject {}",
                subject_id
            )));
        }
        Ok(topic_ids)
    }

    fn resolve_custom_test_archetype(
        &self,
        input: &CustomTestStartInput,
        topic_scope: &[i64],
    ) -> &'static str {
        if input.is_timed && input.question_count >= 20 {
            "pressure_mock"
        } else if input.is_timed {
            "timed_targeted"
        } else if input.weakness_bias && topic_scope.len() <= 2 {
            "repair_check"
        } else {
            "mixed_mastery_check"
        }
    }

    fn resolve_mock_topic_scope(&self, input: &MockBlueprintInput) -> EcoachResult<Vec<i64>> {
        if !input.topic_ids.is_empty() {
            return Ok(input.topic_ids.clone());
        }

        let weak_topics = if input.weakness_bias {
            self.load_weakness_topic_ids(input.student_id, input.subject_id)?
        } else {
            Vec::new()
        };
        if !weak_topics.is_empty() {
            return Ok(weak_topics);
        }

        self.load_default_subject_topics(input.subject_id)
    }

    fn load_recently_seen_question_ids(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT question_id
                 FROM student_question_attempts
                 WHERE student_id = ?1
                 ORDER BY id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit.max(1) as i64], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for row in rows {
            let question_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if seen.insert(question_id) {
                out.push(question_id);
            }
        }
        Ok(out)
    }

    fn build_mock_topic_quotas(
        &self,
        student_id: i64,
        topic_scope: &[i64],
        target_question_count: usize,
        readiness: &ecoach_coach_brain::StudentReadinessSnapshot,
    ) -> EcoachResult<Vec<MockTopicQuota>> {
        let mut weights = self.load_topic_priority_weights(student_id, topic_scope)?;
        let weak_topic_ids = readiness
            .topic_slices
            .iter()
            .filter(|slice| slice.topic_readiness_score < 5_500)
            .map(|slice| slice.topic_id)
            .collect::<BTreeSet<_>>();
        for quota in &mut weights {
            if weak_topic_ids.contains(&quota.topic_id) {
                quota.priority_weight += 2_000;
                quota.is_weak_topic = true;
            }
        }
        weights.sort_by(|left, right| right.priority_weight.cmp(&left.priority_weight));

        if weights.is_empty() {
            return Ok(Vec::new());
        }

        let mut remaining = target_question_count as i64;
        for quota in &mut weights {
            if remaining <= 0 {
                break;
            }
            quota.target_count = 1;
            remaining -= 1;
        }

        while remaining > 0 {
            for quota in &mut weights {
                if remaining <= 0 {
                    break;
                }
                quota.target_count += 1;
                remaining -= 1;
            }
        }

        Ok(weights)
    }

    fn load_topic_priority_weights(
        &self,
        student_id: i64,
        topic_scope: &[i64],
    ) -> EcoachResult<Vec<MockTopicQuota>> {
        let mut topic_names = BTreeMap::new();
        let placeholders = topic_scope
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let topic_sql = format!(
            "SELECT id, name FROM topics WHERE id IN ({}) ORDER BY id ASC",
            placeholders
        );
        let mut params_vec = topic_scope
            .iter()
            .map(|topic_id| rusqlite::types::Value::from(*topic_id))
            .collect::<Vec<_>>();
        let mut topic_statement = self
            .conn
            .prepare(&topic_sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let topic_rows = topic_statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in topic_rows {
            let (topic_id, topic_name) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            topic_names.insert(topic_id, topic_name);
        }

        params_vec.insert(0, rusqlite::types::Value::from(student_id));
        let sql = format!(
            "SELECT topic_id, COALESCE(priority_score, 0)
             FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN ({})
             ORDER BY priority_score DESC, gap_score DESC",
            placeholders
        );
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, BasisPoints>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut weight_map = BTreeMap::new();
        for row in rows {
            let (topic_id, priority_weight) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            weight_map.insert(topic_id, priority_weight);
        }

        Ok(topic_scope
            .iter()
            .map(|topic_id| MockTopicQuota {
                topic_id: *topic_id,
                topic_name: topic_names
                    .get(topic_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Topic {}", topic_id)),
                target_count: 0,
                priority_weight: *weight_map.get(topic_id).unwrap_or(&5_000),
                is_weak_topic: false,
            })
            .collect())
    }

    fn compile_mock_questions(
        &self,
        candidates: &[Question],
        quotas: &[MockTopicQuota],
        target_difficulty: Option<BasisPoints>,
        is_timed: bool,
    ) -> Vec<Question> {
        let mut selected = Vec::new();
        let mut selected_ids = BTreeSet::new();
        let mut selected_families = BTreeSet::new();
        let quota_map = quotas
            .iter()
            .map(|quota| (quota.topic_id, quota.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut buckets = BTreeMap::<i64, Vec<RankedQuestion>>::new();

        for question in candidates {
            let quota = quota_map.get(&question.topic_id);
            let topic_priority = quota
                .map(|item| item.priority_weight as f64 / 10_000.0)
                .unwrap_or(0.5);
            let score = mock_candidate_fit(question, target_difficulty, is_timed, topic_priority);
            buckets
                .entry(question.topic_id)
                .or_default()
                .push(RankedQuestion {
                    question: question.clone(),
                    score,
                });
        }
        for bucket in buckets.values_mut() {
            bucket.sort_by(|left, right| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        for quota in quotas {
            if let Some(bucket) = buckets.get_mut(&quota.topic_id) {
                for _ in 0..quota.target_count {
                    if let Some(question) =
                        take_ranked_question(bucket, &selected_ids, &selected_families)
                    {
                        selected_ids.insert(question.id);
                        if let Some(family_id) = question.family_id {
                            selected_families.insert(family_id);
                        }
                        selected.push(question);
                    }
                }
            }
        }

        let target_count = quotas.iter().map(|quota| quota.target_count).sum::<i64>() as usize;
        if selected.len() < target_count {
            let mut remaining = buckets
                .values()
                .flat_map(|bucket| bucket.iter().cloned())
                .collect::<Vec<_>>();
            remaining.sort_by(|left, right| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for ranked in remaining {
                if selected.len() >= target_count {
                    break;
                }
                if selected_ids.insert(ranked.question.id) {
                    if let Some(family_id) = ranked.question.family_id {
                        selected_families.insert(family_id);
                    }
                    selected.push(ranked.question);
                }
            }
        }

        selected
    }

    fn build_mock_coverage(
        &self,
        quotas: &[MockTopicQuota],
        compiled_questions: &[Question],
        readiness: &ecoach_coach_brain::StudentReadinessSnapshot,
    ) -> EcoachResult<Value> {
        let mut topic_counts = BTreeMap::<i64, i64>::new();
        let mut topic_families = BTreeMap::<i64, BTreeSet<i64>>::new();
        for question in compiled_questions {
            *topic_counts.entry(question.topic_id).or_default() += 1;
            if let Some(family_id) = question.family_id {
                topic_families
                    .entry(question.topic_id)
                    .or_default()
                    .insert(family_id);
            }
        }

        Ok(json!({
            "topic_scope": quotas.iter().map(|quota| quota.topic_id).collect::<Vec<_>>(),
            "compiled_question_count": compiled_questions.len(),
            "distinct_family_count": compiled_questions.iter().filter_map(|question| question.family_id).collect::<BTreeSet<_>>().len(),
            "readiness_score": readiness.readiness_score,
            "topics": quotas.iter().map(|quota| {
                json!({
                    "topic_id": quota.topic_id,
                    "topic_name": quota.topic_name,
                    "target_count": quota.target_count,
                    "compiled_count": topic_counts.get(&quota.topic_id).copied().unwrap_or(0),
                    "distinct_family_count": topic_families.get(&quota.topic_id).map(|families| families.len()).unwrap_or(0),
                })
            }).collect::<Vec<_>>(),
        }))
    }

    fn get_mock_blueprint(&self, blueprint_id: i64) -> EcoachResult<Option<MockBlueprint>> {
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id, title, blueprint_type, duration_minutes,
                        question_count, readiness_score, readiness_band, coverage_json, quota_json,
                        compiled_question_ids_json, status
                 FROM mock_blueprints
                 WHERE id = ?1",
                [blueprint_id],
                map_mock_blueprint,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_session_items(&self, session_id: i64) -> EcoachResult<Vec<SessionItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, session_id, question_id, display_order, source_family_id, source_topic_id,
                        status, selected_option_id, flagged, response_time_ms, is_correct
                 FROM session_items
                 WHERE session_id = ?1
                 ORDER BY display_order ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([session_id], map_session_item)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn get_session_item(&self, item_id: i64) -> EcoachResult<Option<SessionItem>> {
        self.conn
            .query_row(
                "SELECT id, session_id, question_id, display_order, source_family_id, source_topic_id,
                        status, selected_option_id, flagged, response_time_ms, is_correct
                 FROM session_items
                 WHERE id = ?1",
                [item_id],
                map_session_item,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn refresh_session_progress(
        &self,
        session_id: i64,
        active_item_index: i64,
    ) -> EcoachResult<()> {
        let (answered_questions, correct_questions, avg_response_time_ms): (i64, i64, Option<f64>) =
            self.conn
                .query_row(
                    "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), 0),
                        AVG(response_time_ms)
                 FROM session_items
                 WHERE session_id = ?1 AND selected_option_id IS NOT NULL",
                    [session_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let accuracy_score = if answered_questions > 0 {
            Some(((correct_questions as f64 / answered_questions as f64) * 10_000.0).round() as i64)
        } else {
            None
        };
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "UPDATE sessions
                 SET answered_questions = ?1,
                     correct_questions = ?2,
                     accuracy_score = ?3,
                     avg_response_time_ms = ?4,
                     active_item_index = MAX(active_item_index, ?5),
                     last_activity_at = ?6,
                     updated_at = datetime('now')
                 WHERE id = ?7",
                params![
                    answered_questions,
                    correct_questions,
                    accuracy_score,
                    avg_response_time_ms.map(|value| value.round() as i64),
                    active_item_index,
                    now,
                    session_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn ensure_session_status(
        &self,
        session_id: i64,
        allowed_statuses: &[&str],
    ) -> EcoachResult<()> {
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        if allowed_statuses.contains(&session.status.as_str()) {
            Ok(())
        } else {
            Err(EcoachError::Validation(format!(
                "session {} is in status {} but expected one of {:?}",
                session_id, session.status, allowed_statuses
            )))
        }
    }

    fn append_runtime_event(&self, aggregate_kind: &str, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.event_id,
                    event.event_type,
                    aggregate_kind,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn parse_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
    value
        .and_then(|raw| DateTime::<Utc>::from_str(&raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn map_session_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionItem> {
    Ok(SessionItem {
        id: row.get(0)?,
        session_id: row.get(1)?,
        question_id: row.get(2)?,
        display_order: row.get(3)?,
        source_family_id: row.get(4)?,
        source_topic_id: row.get(5)?,
        status: row.get(6)?,
        selected_option_id: row.get(7)?,
        flagged: row.get::<_, i64>(8)? == 1,
        response_time_ms: row.get(9)?,
        is_correct: row.get::<_, Option<i64>>(10)?.map(|value| value == 1),
    })
}

#[derive(Debug, Clone)]
struct MockTopicQuota {
    topic_id: i64,
    topic_name: String,
    target_count: i64,
    priority_weight: BasisPoints,
    is_weak_topic: bool,
}

#[derive(Debug, Clone)]
struct RankedQuestion {
    question: Question,
    score: f64,
}

fn map_mock_blueprint(row: &rusqlite::Row<'_>) -> rusqlite::Result<MockBlueprint> {
    let coverage_json: String = row.get(9)?;
    let quota_json: String = row.get(10)?;
    let compiled_question_ids_json: String = row.get(11)?;
    Ok(MockBlueprint {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        title: row.get(3)?,
        blueprint_type: row.get(4)?,
        duration_minutes: row.get(5)?,
        question_count: row.get(6)?,
        readiness_score: row.get(7)?,
        readiness_band: row.get(8)?,
        coverage: parse_json_value(9, &coverage_json)?,
        quotas: parse_json_value(10, &quota_json)?,
        compiled_question_ids: parse_i64_json_array(11, &compiled_question_ids_json)?,
        status: row.get(12)?,
    })
}

fn parse_json_value(column_index: usize, raw: &str) -> rusqlite::Result<Value> {
    serde_json::from_str::<Value>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

fn parse_i64_json_array(column_index: usize, raw: &str) -> rusqlite::Result<Vec<i64>> {
    serde_json::from_str::<Vec<i64>>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

fn mock_candidate_fit(
    question: &Question,
    target_difficulty: Option<BasisPoints>,
    is_timed: bool,
    topic_priority: f64,
) -> f64 {
    let difficulty_fit = target_difficulty
        .map(|target| {
            1.0 - ((question.difficulty_level as f64 - target as f64).abs() / 10_000.0).min(1.0)
        })
        .unwrap_or(0.75);
    let timed_fit = if is_timed && question.estimated_time_seconds <= 60 {
        0.95
    } else if is_timed {
        0.6
    } else {
        0.8
    };
    let family_bonus = if question.family_id.is_some() {
        0.85
    } else {
        0.55
    };
    0.35 * difficulty_fit + 0.25 * topic_priority + 0.20 * timed_fit + 0.20 * family_bonus
}

fn take_ranked_question(
    bucket: &mut Vec<RankedQuestion>,
    selected_ids: &BTreeSet<i64>,
    selected_families: &BTreeSet<i64>,
) -> Option<Question> {
    if let Some(index) = bucket.iter().position(|candidate| {
        !selected_ids.contains(&candidate.question.id)
            && candidate
                .question
                .family_id
                .map(|family_id| !selected_families.contains(&family_id))
                .unwrap_or(true)
    }) {
        return Some(bucket.remove(index).question);
    }
    bucket
        .iter()
        .position(|candidate| !selected_ids.contains(&candidate.question.id))
        .map(|index| bucket.remove(index).question)
}

fn title_for_mock_blueprint(blueprint_type: &str) -> &'static str {
    match blueprint_type {
        "repair_mock" => "Repair",
        "recovery_mock" => "Recovery",
        "coverage_mock" => "Coverage",
        "pressure_mock" => "Pressure",
        _ => "Balanced",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{Duration, Utc};
    use ecoach_coach_brain::PlanEngine;
    use ecoach_content::PackService;
    use ecoach_identity::{CreateAccountInput, IdentityService};
    use ecoach_questions::QuestionService;
    use ecoach_storage::run_runtime_migrations;
    use ecoach_student_model::{AnswerSubmission, ErrorType, StudentModelService};
    use ecoach_substrate::{AccountType, EntitlementTier};
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn practice_session_flow_drives_attempts_and_mission_generation() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Ama".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let session_service = SessionService::new(&conn);
        let (session, selected_questions) = session_service
            .start_practice_session(&PracticeSessionStartInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                is_timed: false,
            })
            .expect("practice session should start");

        assert_eq!(selected_questions.len(), 2);
        let snapshot = session_service
            .get_session_snapshot(session.id)
            .expect("session snapshot should load")
            .expect("session should exist");
        assert_eq!(snapshot.items.len(), 2);

        let paused = session_service
            .pause_session(session.id)
            .expect("session should pause");
        assert_eq!(paused.status, "paused");
        let resumed = session_service
            .resume_session(session.id)
            .expect("session should resume");
        assert_eq!(resumed.session.status, "active");

        let question_service = QuestionService::new(&conn);
        let options = question_service
            .list_options(snapshot.items[0].question_id)
            .expect("question options should be queryable");
        let misconception_option = options
            .iter()
            .find(|option| option.misconception_id.is_some())
            .expect("sample pack should include a misconception option");
        let recorded_item = session_service
            .record_answer(
                session.id,
                &SessionAnswerInput {
                    item_id: snapshot.items[0].id,
                    selected_option_id: misconception_option.id,
                    response_time_ms: Some(18_000),
                },
            )
            .expect("session runtime answer should persist");
        assert_eq!(recorded_item.is_correct, Some(false));
        let flagged_item = session_service
            .flag_session_item(session.id, snapshot.items[1].id, true)
            .expect("session item should be flaggable");
        assert!(flagged_item.flagged);

        let student_model = StudentModelService::new(&conn);
        let now = Utc::now();
        let result = student_model
            .process_answer(
                student.id,
                &AnswerSubmission {
                    question_id: snapshot.items[0].question_id,
                    selected_option_id: misconception_option.id,
                    session_id: Some(session.id),
                    session_type: Some("practice".to_string()),
                    started_at: now - Duration::seconds(18),
                    submitted_at: now,
                    response_time_ms: Some(18_000),
                    confidence_level: Some("not_sure".to_string()),
                    hint_count: 0,
                    changed_answer_count: 0,
                    skipped: false,
                    timed_out: false,
                    support_level: Some("independent".to_string()),
                    was_timed: false,
                    was_transfer_variant: false,
                    was_retention_check: false,
                    was_mixed_context: false,
                },
            )
            .expect("answer processing should succeed");

        assert!(!result.is_correct);
        assert_eq!(result.error_type, Some(ErrorType::MisconceptionTriggered));

        let summary = session_service
            .complete_session(session.id)
            .expect("session summary should be generated");
        assert_eq!(summary.answered_questions, 1);
        assert_eq!(summary.correct_questions, 0);
        assert_eq!(summary.status, "completed");

        let plan_engine = PlanEngine::new(&conn);
        let exam_date = (Utc::now() + Duration::days(60)).date_naive().to_string();
        let plan_id = plan_engine
            .generate_plan(student.id, "BECE", &exam_date, 45)
            .expect("plan should be generated");
        let mission_id = plan_engine
            .generate_today_mission(student.id)
            .expect("today mission should be generated");

        let mission_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_missions WHERE id = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .expect("mission count should be queryable");
        let plan_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_plans WHERE id = ?1",
                [plan_id],
                |row| row.get(0),
            )
            .expect("plan count should be queryable");
        let skill_state_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM student_skill_states WHERE student_id = ?1",
                [student.id],
                |row| row.get(0),
            )
            .expect("skill state count should be queryable");
        let memory_state_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memory_states WHERE student_id = ?1",
                [student.id],
                |row| row.get(0),
            )
            .expect("memory state count should be queryable");
        let recheck_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM recheck_schedules WHERE student_id = ?1",
                [student.id],
                |row| row.get(0),
            )
            .expect("recheck schedule count should be queryable");
        let runtime_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE aggregate_kind = 'session' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("runtime event count should be queryable");

        assert_eq!(mission_count, 1);
        assert_eq!(plan_count, 1);
        assert_eq!(skill_state_count, 1);
        assert_eq!(memory_state_count, 1);
        assert_eq!(recheck_count, 1);
        assert!(runtime_event_count >= 6);
    }

    #[test]
    fn custom_test_composes_a_targeted_runtime_session() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kojo".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let session_service = SessionService::new(&conn);
        let (session, selected_questions) = session_service
            .start_custom_test(&CustomTestStartInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                duration_minutes: Some(15),
                is_timed: true,
                target_difficulty: Some(6500),
                weakness_bias: true,
            })
            .expect("custom test should compose successfully");

        let snapshot = session_service
            .get_session_snapshot(session.id)
            .expect("snapshot should load")
            .expect("session should exist");
        let archetype: String = conn
            .query_row(
                "SELECT difficulty_preference FROM sessions WHERE id = ?1",
                [session.id],
                |row| row.get(0),
            )
            .expect("custom archetype should be stored");
        let custom_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type = 'custom_test.composed' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("custom test event count should be queryable");

        assert_eq!(session.session_type, "custom_test");
        assert_eq!(selected_questions.len(), 2);
        assert_eq!(snapshot.items.len(), 2);
        assert_eq!(archetype, "timed_targeted");
        assert_eq!(custom_event_count, 1);
    }

    #[test]
    fn mock_blueprint_compiles_and_starts_mock_session() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Esi".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student.id],
        )
        .expect("student profile should insert");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, memory_strength, priority_score
             ) VALUES (?1, ?2, 4200, 7800, 6200, 3500, 9300)",
            params![student.id, topic_id],
        )
        .expect("topic state should insert");

        let service = SessionService::new(&conn);
        let blueprint = service
            .generate_mock_blueprint(&MockBlueprintInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                duration_minutes: Some(20),
                is_timed: true,
                target_difficulty: Some(6500),
                weakness_bias: true,
            })
            .expect("mock blueprint should compile");
        let (session, selected_questions) = service
            .start_mock_session(blueprint.id)
            .expect("mock session should start");

        let event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type IN ('mock.blueprint_compiled', 'mock.session_created')",
                [],
                |row| row.get(0),
            )
            .expect("mock events should query");

        assert_eq!(session.session_type, "mock");
        assert_eq!(selected_questions.len(), 2);
        assert_eq!(blueprint.status, "compiled");
        assert!(event_count >= 2);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn install_sample_pack(conn: &Connection) {
        let service = PackService::new(conn);
        service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
    }

    fn load_fraction_scope(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT s.id, t.id
             FROM subjects s
             JOIN topics t ON t.subject_id = s.id
             WHERE s.code = 'MATH' AND t.code = 'FRA'
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("fractions topic should exist")
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
