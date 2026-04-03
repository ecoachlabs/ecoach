use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use ecoach_coach_brain::{
    CoachMissionBrief, PedagogicalRuntimeService, PlanEngine, ReadinessEngine,
};
use ecoach_questions::{
    Question, QuestionGenerationRequestInput, QuestionReactor, QuestionRemediationPlan,
    QuestionSelectionRequest, QuestionSelector, QuestionService, QuestionSlotSpec,
    QuestionVariantMode, SelectedQuestion,
};
use ecoach_substrate::{
    BasisPoints, DomainEvent, EcoachError, EcoachResult, EngineRegistry, FabricEvidenceRecord,
    FabricOrchestrationSummary, FabricSignal, clamp_bp,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    CoachMissionSessionPlan, CustomTestStartInput, FocusModeConfig, MockBlueprint,
    MockBlueprintInput, PracticeSessionStartInput, Session, SessionAnswerInput,
    SessionEvidenceFabric, SessionInterpretation, SessionItem, SessionPresenceEvent,
    SessionPresenceEventInput, SessionPresenceSnapshot, SessionSnapshot, SessionSummary,
    SessionTopicInterpretation,
};

pub struct SessionService<'a> {
    conn: &'a Connection,
}

impl<'a> SessionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn start_coach_mission_session(
        &self,
        student_id: i64,
    ) -> EcoachResult<CoachMissionSessionPlan> {
        let plan_engine = PlanEngine::new(self.conn);
        let mission = plan_engine.get_or_prepare_today_mission(student_id)?;
        if let Some(existing_session_id) = mission.session_id {
            PedagogicalRuntimeService::new(self.conn)
                .initialize_session_runtime(existing_session_id)?;
            self.ensure_presence_snapshot(existing_session_id, None)?;
            let snapshot = self
                .get_session_snapshot(existing_session_id)?
                .ok_or_else(|| {
                    EcoachError::NotFound(format!(
                        "coach mission session {} was not found",
                        existing_session_id
                    ))
                })?;
            return Ok(CoachMissionSessionPlan {
                mission_id: mission.mission_id,
                session_id: existing_session_id,
                title: mission.title,
                reason: mission.reason,
                activity_type: mission.activity_type,
                target_minutes: mission.target_minutes,
                subject_id: mission.subject_id.unwrap_or_default(),
                topic_id: mission.topic_id.unwrap_or_default(),
                question_ids: mission.question_ids,
                coverage: json!({
                    "selected_count": snapshot.items.len(),
                    "reused_existing_session": true,
                }),
                session_snapshot: snapshot,
            });
        }
        let subject_id = mission
            .subject_id
            .ok_or_else(|| EcoachError::Validation("coach mission has no subject".to_string()))?;
        let topic_id = mission
            .topic_id
            .ok_or_else(|| EcoachError::Validation("coach mission has no topic".to_string()))?;
        let target_question_count =
            desired_question_count_for_mission(&mission.activity_type, mission.target_minutes);
        let topic_ids = vec![topic_id];
        let timed = mission_uses_timed_questions(&mission.activity_type);
        let recent_ids = self.load_recently_seen_question_ids(student_id, 40)?;
        let selector = QuestionSelector::new(self.conn);
        let mut selected_questions = selector.select_questions(&QuestionSelectionRequest {
            subject_id,
            topic_ids: topic_ids.clone(),
            target_question_count,
            target_difficulty: target_difficulty_for_mission(&mission.activity_type),
            weakness_topic_ids: weakness_topic_ids_for_mission(&mission),
            recently_seen_question_ids: recent_ids,
            timed,
            diagnostic_stage: None,
            condition_type: None,
            require_confidence_prompt: false,
            require_concept_guess_prompt: false,
        })?;
        let target_topic_counts = distribute_topic_targets(&topic_ids, target_question_count);
        if selected_questions.len() < target_question_count {
            if let Err(err) = self.top_up_with_reactor(
                subject_id,
                &target_topic_counts,
                &mut selected_questions,
                target_difficulty_for_mission(&mission.activity_type),
                timed,
                choose_reactor_variant_mode(
                    timed,
                    target_difficulty_for_mission(&mission.activity_type),
                ),
                "slot_fill",
                "Fill missing coach mission items with traced family variants",
            ) {
                if selected_questions.is_empty() {
                    return Err(err);
                }
            }
        }

        if selected_questions.is_empty() {
            plan_engine.mark_mission_no_question_recovery(
                student_id,
                mission.mission_id,
                Some(topic_id),
            )?;
            return Err(EcoachError::NotFound(
                "no real questions available for coach mission".to_string(),
            ));
        }

        let question_ids = selected_questions
            .iter()
            .map(|selected| selected.question.id)
            .collect::<Vec<_>>();
        let now = Utc::now().to_rfc3339();
        let topic_ids_json = serde_json::to_string(&topic_ids)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                    duration_minutes, is_timed, difficulty_preference, status, started_at,
                    last_activity_at, coach_mode_entered_at
                 ) VALUES (?1, 'coach_mission', ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?9, ?9)",
                params![
                    student_id,
                    subject_id,
                    topic_ids_json,
                    target_question_count as i64,
                    selected_questions.len() as i64,
                    mission.target_minutes,
                    if timed { 1 } else { 0 },
                    mission.activity_type.clone(),
                    now,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let session_id = self.conn.last_insert_rowid();
        self.persist_selected_items(session_id, &selected_questions)?;
        PedagogicalRuntimeService::new(self.conn).initialize_session_runtime(session_id)?;
        self.ensure_presence_snapshot(session_id, Some(&now))?;
        self.tag_items_with_mission_intent(session_id, &mission.activity_type)?;
        plan_engine.attach_session_to_mission(mission.mission_id, session_id, &question_ids)?;
        plan_engine.start_mission(mission.mission_id)?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.coach_mission_created",
                session_id.to_string(),
                json!({
                    "student_id": student_id,
                    "mission_id": mission.mission_id,
                    "topic_id": topic_id,
                    "activity_type": mission.activity_type,
                    "question_ids": question_ids,
                }),
            ),
        )?;
        let snapshot = self
            .get_session_snapshot(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        let coverage = self.build_mission_question_coverage(subject_id, topic_id, &question_ids)?;
        Ok(CoachMissionSessionPlan {
            mission_id: mission.mission_id,
            session_id,
            title: mission.title,
            reason: mission.reason,
            activity_type: mission.activity_type,
            target_minutes: mission.target_minutes,
            subject_id,
            topic_id,
            question_ids,
            coverage,
            session_snapshot: snapshot,
        })
    }

    pub fn start_practice_session(
        &self,
        input: &PracticeSessionStartInput,
    ) -> EcoachResult<(Session, Vec<SelectedQuestion>)> {
        let selector = QuestionSelector::new(self.conn);
        let mut questions = selector.select_questions(&QuestionSelectionRequest {
            subject_id: input.subject_id,
            topic_ids: input.topic_ids.clone(),
            target_question_count: input.question_count,
            target_difficulty: None,
            weakness_topic_ids: input.topic_ids.clone(),
            recently_seen_question_ids: Vec::new(),
            timed: input.is_timed,
            diagnostic_stage: None,
            condition_type: None,
            require_confidence_prompt: false,
            require_concept_guess_prompt: false,
        })?;
        let target_topic_counts = distribute_topic_targets(&input.topic_ids, input.question_count);
        let reactor_generated_count = self.top_up_with_reactor(
            input.subject_id,
            &target_topic_counts,
            &mut questions,
            None,
            input.is_timed,
            choose_reactor_variant_mode(input.is_timed, None),
            "slot_fill",
            "Fill missing practice items with traced family variants",
        )?;
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
        PedagogicalRuntimeService::new(self.conn).initialize_session_runtime(session_id)?;
        self.ensure_presence_snapshot(session_id, Some(&now))?;
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
        if reactor_generated_count > 0 {
            self.append_runtime_event(
                "session",
                DomainEvent::new(
                    "session.reactor_top_up",
                    session_id.to_string(),
                    serde_json::json!({
                        "session_type": "practice",
                        "generated_question_count": reactor_generated_count,
                        "target_question_count": input.question_count,
                    }),
                ),
            )?;
        }

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
        let mut questions = selector.select_questions(&QuestionSelectionRequest {
            subject_id: input.subject_id,
            topic_ids: topic_scope.clone(),
            target_question_count: input.question_count,
            target_difficulty: input.target_difficulty,
            weakness_topic_ids,
            recently_seen_question_ids: Vec::new(),
            timed: input.is_timed,
            diagnostic_stage: None,
            condition_type: None,
            require_confidence_prompt: false,
            require_concept_guess_prompt: false,
        })?;
        let target_topic_counts = distribute_topic_targets(&topic_scope, input.question_count);
        let reactor_generated_count = self.top_up_with_reactor(
            input.subject_id,
            &target_topic_counts,
            &mut questions,
            input.target_difficulty,
            input.is_timed,
            choose_reactor_variant_mode(input.is_timed, input.target_difficulty),
            "slot_fill",
            "Fill missing custom-test items with traced family variants",
        )?;
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
        PedagogicalRuntimeService::new(self.conn).initialize_session_runtime(session_id)?;
        self.ensure_presence_snapshot(session_id, Some(&now))?;
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
        if reactor_generated_count > 0 {
            self.append_runtime_event(
                "session",
                DomainEvent::new(
                    "session.reactor_top_up",
                    session_id.to_string(),
                    serde_json::json!({
                        "session_type": "custom_test",
                        "generated_question_count": reactor_generated_count,
                        "target_question_count": input.question_count,
                    }),
                ),
            )?;
        }

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
            input.subject_id,
            &candidates,
            &quotas,
            input.target_difficulty,
            input.is_timed,
        );
        let quota_targets = quotas
            .iter()
            .map(|quota| (quota.topic_id, quota.target_count.max(0) as usize))
            .collect::<BTreeMap<_, _>>();
        let mut compiled_selected = compiled_questions
            .into_iter()
            .map(|question| SelectedQuestion {
                fit_score: 1.0,
                question,
            })
            .collect::<Vec<_>>();
        let reactor_generated_count = self.top_up_with_reactor(
            input.subject_id,
            &quota_targets,
            &mut compiled_selected,
            input.target_difficulty,
            input.is_timed,
            choose_reactor_variant_mode(input.is_timed, input.target_difficulty),
            "slot_fill",
            "Fill missing mock blueprint slots with traced family variants",
        )?;
        let compiled_questions = compiled_selected
            .into_iter()
            .map(|selected| selected.question)
            .collect::<Vec<_>>();
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
        if reactor_generated_count > 0 {
            self.append_runtime_event(
                "session",
                DomainEvent::new(
                    "mock.blueprint_reactor_top_up",
                    blueprint_id.to_string(),
                    json!({
                        "student_id": input.student_id,
                        "subject_id": input.subject_id,
                        "generated_question_count": reactor_generated_count,
                        "target_question_count": input.question_count,
                    }),
                ),
            )?;
        }

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
        self.ensure_presence_snapshot(session_id, Some(&now))?;

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
        PedagogicalRuntimeService::new(self.conn).initialize_session_runtime(session_id)?;

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

    pub fn get_session_presence_snapshot(
        &self,
        session_id: i64,
    ) -> EcoachResult<Option<SessionPresenceSnapshot>> {
        self.load_presence_snapshot(session_id)
    }

    pub fn list_session_presence_events(
        &self,
        session_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<SessionPresenceEvent>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, session_id, event_type, occurred_at, state_before, state_after,
                        segment_duration_ms, counted_credit_ms, metadata_json
                 FROM session_presence_events
                 WHERE session_id = ?1
                 ORDER BY occurred_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![session_id, limit.max(1) as i64], |row| {
                let metadata_json = row.get::<_, String>(8)?;
                let metadata = serde_json::from_str::<Value>(&metadata_json).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(EcoachError::Serialization(err.to_string())),
                    )
                })?;
                let occurred_at = row.get::<_, String>(3)?;
                Ok(SessionPresenceEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    event_type: row.get(2)?,
                    occurred_at: DateTime::parse_from_rfc3339(&occurred_at)
                        .map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(EcoachError::Validation(err.to_string())),
                            )
                        })?
                        .with_timezone(&Utc),
                    state_before: row.get(4)?,
                    state_after: row.get(5)?,
                    segment_duration_ms: row.get(6)?,
                    counted_credit_ms: row.get(7)?,
                    metadata_json: metadata,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        events.reverse();
        Ok(events)
    }

    pub fn record_session_presence_event(
        &self,
        session_id: i64,
        input: &SessionPresenceEventInput,
    ) -> EcoachResult<SessionPresenceSnapshot> {
        let occurred_at = input
            .occurred_at
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let metadata_json = input
            .metadata_json
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        self.record_presence_event_internal(
            session_id,
            &input.event_type,
            &occurred_at,
            metadata_json,
        )?;
        self.get_session_presence_snapshot(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} presence not found", session_id)))
    }

    pub fn manual_stop_session(
        &self,
        session_id: i64,
        reason: Option<String>,
    ) -> EcoachResult<SessionSnapshot> {
        self.ensure_session_status(session_id, &["active", "paused"])?;
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE sessions
                 SET status = 'paused', paused_at = ?1, last_activity_at = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![now, session_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.record_presence_event_internal(
            session_id,
            "manual_stop",
            &now,
            serde_json::json!({ "reason": reason }),
        )?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.manually_stopped",
                session_id.to_string(),
                serde_json::json!({ "stopped_at": now, "reason": reason }),
            ),
        )?;
        self.get_session_snapshot(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))
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
        self.record_presence_event_internal(
            session_id,
            "manual_pause",
            &now,
            serde_json::json!({}),
        )?;
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
        self.record_presence_event_internal(
            session_id,
            "session_resumed",
            &now,
            serde_json::json!({}),
        )?;
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
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        let session_meta = self.load_session_runtime_meta(session_id)?;

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
        let question = question_service
            .get_question(item.question_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("question {} not found", item.question_id))
            })?;
        let variant_flags = self.load_question_variant_flags(item.question_id)?;

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
        self.record_presence_event_internal(
            session_id,
            "meaningful_interaction",
            &answered_at,
            serde_json::json!({
                "item_id": item.id,
                "question_id": item.question_id,
                "response_time_ms": input.response_time_ms,
            }),
        )?;

        self.insert_session_attempt(&session, &session_meta, &item, &question, &option, input)?;
        self.refresh_session_progress(session_id, item.display_order)?;
        QuestionReactor::new(self.conn).record_instance_outcome(item.question_id)?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.answer_recorded",
                session_id.to_string(),
                serde_json::json!({
                    "item_id": item.id,
                    "question_id": item.question_id,
                    "topic_id": question.topic_id,
                    "selected_option_id": input.selected_option_id,
                    "is_correct": option.is_correct,
                }),
            ),
        )?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.answer_interpreted",
                session_id.to_string(),
                serde_json::json!({
                    "item_id": item.id,
                    "question_id": item.question_id,
                    "topic_id": question.topic_id,
                    "node_id": question.primary_skill_id,
                    "family_id": question.family_id,
                    "question_format": question.question_format,
                    "estimated_time_seconds": question.estimated_time_seconds,
                    "response_time_ms": input.response_time_ms,
                    "timed_session": session_meta.is_timed,
                    "session_type": session_meta.session_type,
                    "selected_option_id": input.selected_option_id,
                    "is_correct": option.is_correct,
                    "misconception_triggered": option.misconception_id.is_some(),
                    "timed_pressure_signal": session_meta.is_timed && !option.is_correct,
                    "was_transfer_variant": variant_flags.was_transfer_variant,
                    "was_mixed_context": variant_flags.was_mixed_context,
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
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        let summary = self.build_summary(session_id)?;
        let completed_at = Utc::now().to_rfc3339();
        self.record_presence_event_internal(
            session_id,
            "session_completed",
            &completed_at,
            serde_json::json!({}),
        )?;
        let (active_study_time_ms, idle_time_ms) =
            self.estimate_session_time_totals(session_id, session.started_at, &completed_at)?;
        self.conn
            .execute(
                "UPDATE sessions
                 SET status = 'completed', completed_at = ?1, last_activity_at = ?1,
                     answered_questions = ?2, correct_questions = ?3, accuracy_score = ?4,
                     active_study_time_ms = ?5, idle_time_ms = ?6,
                     coach_mode_exited_at = CASE
                        WHEN session_type = 'coach_mission' THEN ?1
                        ELSE coach_mode_exited_at
                     END,
                     updated_at = datetime('now')
                 WHERE id = ?7",
                params![
                    completed_at,
                    summary.answered_questions,
                    summary.correct_questions,
                    summary.accuracy_score,
                    active_study_time_ms,
                    idle_time_ms,
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
        if let Some(interpretation) = self.get_session_interpretation(session_id)? {
            self.append_runtime_event(
                "session",
                DomainEvent::new(
                    "session.interpreted",
                    session_id.to_string(),
                    json!({
                        "session_id": interpretation.session_id,
                        "student_id": interpretation.student_id,
                        "session_type": interpretation.session_type,
                        "status": interpretation.status,
                        "observed_at": interpretation.observed_at.to_rfc3339(),
                        "is_timed": interpretation.is_timed,
                        "answered_questions": interpretation.answered_questions,
                        "correct_questions": interpretation.correct_questions,
                        "incorrect_questions": interpretation.incorrect_questions,
                        "unanswered_questions": interpretation.unanswered_questions,
                        "accuracy_score": interpretation.accuracy_score,
                        "avg_response_time_ms": interpretation.avg_response_time_ms,
                        "flagged_count": interpretation.flagged_count,
                        "distinct_topic_count": interpretation.distinct_topic_count,
                        "misconception_hit_count": interpretation.misconception_hit_count,
                        "pressure_breakdown_count": interpretation.pressure_breakdown_count,
                        "transfer_variant_count": interpretation.transfer_variant_count,
                        "retention_check_count": interpretation.retention_check_count,
                        "mixed_context_count": interpretation.mixed_context_count,
                        "supported_answer_count": interpretation.supported_answer_count,
                        "independent_answer_count": interpretation.independent_answer_count,
                        "dominant_error_type": interpretation.dominant_error_type,
                        "interpretation_tags": interpretation.interpretation_tags,
                        "next_action_hint": interpretation.next_action_hint,
                        "topic_summaries": interpretation.topic_summaries,
                    }),
                ),
            )?;
        }
        let remediation_plans = self.list_session_remediation_plans(session_id, 3)?;
        if !remediation_plans.is_empty() {
            self.append_runtime_event(
                "session",
                DomainEvent::new(
                    "session.remediation_planned",
                    session_id.to_string(),
                    json!({
                        "session_id": session_id,
                        "plan_count": remediation_plans.len(),
                        "plans": remediation_plans,
                    }),
                ),
            )?;
        }
        if session.session_type == "coach_mission" {
            if let Some(mission_id) = self.load_mission_id_for_session(session_id)? {
                PlanEngine::new(self.conn)
                    .complete_mission_from_session(mission_id, Some(session_id))?;
            }
        }
        self.build_summary(session_id)
    }

    pub fn get_session_interpretation(
        &self,
        session_id: i64,
    ) -> EcoachResult<Option<SessionInterpretation>> {
        let session = match self.get_session(session_id)? {
            Some(session) => session,
            None => return Ok(None),
        };
        let meta = self.load_session_runtime_meta(session_id)?;
        let summary = self.build_summary(session_id)?;
        let topic_summaries = self.list_session_topic_interpretations(session_id)?;
        let metrics = self.load_attempt_metrics(session_id)?;
        let flagged_count = self.count_flagged_items(session_id)?;
        let unanswered_questions = (meta.total_questions - summary.answered_questions).max(0);
        let incorrect_questions = (summary.answered_questions - summary.correct_questions).max(0);
        let accuracy_score = summary.accuracy_score.map(|value| value as BasisPoints);
        let interpretation_tags = derive_session_interpretation_tags(
            meta.is_timed,
            accuracy_score,
            unanswered_questions,
            flagged_count,
            &metrics,
        );
        let next_action_hint = derive_next_action_hint(
            meta.is_timed,
            accuracy_score,
            unanswered_questions,
            &metrics,
        );
        let observed_at = session
            .completed_at
            .or(session.last_activity_at)
            .or_else(|| session.started_at.as_ref().map(|value| value.to_owned()))
            .unwrap_or_else(Utc::now);

        Ok(Some(SessionInterpretation {
            session_id,
            student_id: session.student_id,
            session_type: session.session_type,
            status: session.status,
            observed_at,
            is_timed: meta.is_timed,
            answered_questions: summary.answered_questions,
            correct_questions: summary.correct_questions,
            incorrect_questions,
            unanswered_questions,
            accuracy_score,
            avg_response_time_ms: metrics.avg_response_time_ms,
            flagged_count,
            distinct_topic_count: metrics
                .distinct_topic_count
                .max(topic_summaries.len() as i64),
            misconception_hit_count: metrics.misconception_hit_count,
            pressure_breakdown_count: metrics.pressure_breakdown_count,
            transfer_variant_count: metrics.transfer_variant_count,
            retention_check_count: metrics.retention_check_count,
            mixed_context_count: metrics.mixed_context_count,
            supported_answer_count: metrics.supported_answer_count,
            independent_answer_count: metrics.independent_answer_count,
            dominant_error_type: metrics.dominant_error_type,
            interpretation_tags,
            next_action_hint,
            topic_summaries,
        }))
    }

    pub fn list_session_remediation_plans(
        &self,
        session_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<QuestionRemediationPlan>> {
        let session = match self.get_session(session_id)? {
            Some(session) => session,
            None => return Ok(Vec::new()),
        };
        let subject_id = match session.subject_id {
            Some(subject_id) => subject_id,
            None => return Ok(Vec::new()),
        };
        let session_is_timed = self
            .conn
            .query_row(
                "SELECT COALESCE(is_timed, 0) FROM sessions WHERE id = ?1",
                [session_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            != 0;

        let reactor = QuestionReactor::new(self.conn);
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.topic_id, q.question_format, q.primary_cognitive_demand,
                        q.family_id, si.question_id,
                        COALESCE(si.response_time_ms, 0) AS response_time_ms
                 FROM session_items si
                 INNER JOIN questions q ON q.id = si.question_id
                 WHERE si.session_id = ?1
                   AND COALESCE(si.is_correct, 1) = 0
                 ORDER BY CASE WHEN si.flagged = 1 THEN 0 ELSE 1 END ASC,
                          CASE WHEN si.response_time_ms IS NULL THEN 1 ELSE 0 END ASC,
                          si.response_time_ms DESC,
                          si.display_order ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut observations = Vec::new();
        for row in rows {
            let (
                topic_id,
                question_format,
                primary_cognitive_demand,
                family_id,
                question_id,
                response_time_ms,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            observations.push(SessionRemediationObservation {
                topic_id,
                question_format,
                primary_cognitive_demand,
                family_id,
                question_id,
                response_time_ms,
            });
        }

        let family_ids = observations
            .iter()
            .filter_map(|item| item.family_id)
            .collect::<BTreeSet<_>>();
        let runtime_signals = self.load_family_runtime_signals(subject_id, &family_ids)?;
        let mut remediation_plans = Vec::new();
        let mut planned_families = BTreeSet::new();
        for observation in observations {
            let family_id = observation.family_id;
            if let Some(family_id) = family_id {
                if planned_families.contains(&family_id) {
                    continue;
                }
            }
            let slot_spec = QuestionSlotSpec {
                subject_id,
                topic_id: Some(observation.topic_id),
                target_cognitive_demand: observation.primary_cognitive_demand.clone(),
                target_question_format: Some(observation.question_format.clone()),
                max_generated_share: if observation.response_time_ms > 45_000 {
                    6_000
                } else {
                    7_500
                },
            };
            let base_plan = reactor
                .recommend_remediation_plan(session.student_id, &slot_spec)?
                .or_else(|| {
                    reactor
                        .list_family_generation_priorities(&slot_spec, 4)
                        .ok()
                        .and_then(|priorities| {
                            priorities.into_iter().find_map(|priority| {
                                if planned_families.contains(&priority.family_choice.family_id) {
                                    None
                                } else {
                                    Some(QuestionRemediationPlan {
                                        family_choice: priority.family_choice,
                                        variant_mode: priority.recommended_variant_mode,
                                        priority_score: priority.priority_score,
                                        source_question_id: Some(observation.question_id),
                                        request_kind: "remediation".to_string(),
                                        rationale: format!(
                                            "Session error had no direct student-history match, so the highest-pressure family for the slot was chosen. {}",
                                            priority.rationale
                                        ),
                                    })
                                }
                            })
                        })
                });
            if let Some(plan) = base_plan {
                let family_choice_id = plan.family_choice.family_id;
                let enriched = enrich_remediation_plan(
                    plan,
                    &session.session_type,
                    session_is_timed,
                    observation.response_time_ms,
                    runtime_signals.get(&family_choice_id),
                );
                planned_families.insert(enriched.family_choice.family_id);
                remediation_plans.push(enriched);
            }
            if remediation_plans.len() >= limit.max(1) {
                break;
            }
        }

        remediation_plans.sort_by(|left, right| {
            right.priority_score.cmp(&left.priority_score).then(
                left.family_choice
                    .family_name
                    .cmp(&right.family_choice.family_name),
            )
        });
        remediation_plans.truncate(limit.max(1));
        Ok(remediation_plans)
    }

    pub fn get_session_evidence_fabric(
        &self,
        session_id: i64,
        limit_events: usize,
    ) -> EcoachResult<Option<SessionEvidenceFabric>> {
        let session = match self.get_session(session_id)? {
            Some(session) => session,
            None => return Ok(None),
        };
        let interpretation = match self.get_session_interpretation(session_id)? {
            Some(interpretation) => interpretation,
            None => return Ok(None),
        };
        let remediation_plans = self.list_session_remediation_plans(session_id, 3)?;
        let observed_at = interpretation.observed_at.to_rfc3339();
        let mut signals = vec![FabricSignal {
            engine_key: "session_runtime".to_string(),
            signal_type: "session_outcome".to_string(),
            status: Some(interpretation.next_action_hint.clone()),
            score: interpretation.accuracy_score,
            topic_id: interpretation
                .topic_summaries
                .first()
                .filter(|_| interpretation.topic_summaries.len() == 1)
                .map(|topic| topic.topic_id),
            node_id: None,
            question_id: None,
            observed_at: observed_at.clone(),
            payload: json!({
                "session_type": interpretation.session_type,
                "is_timed": interpretation.is_timed,
                "answered_questions": interpretation.answered_questions,
                "correct_questions": interpretation.correct_questions,
                "incorrect_questions": interpretation.incorrect_questions,
                "unanswered_questions": interpretation.unanswered_questions,
                "avg_response_time_ms": interpretation.avg_response_time_ms,
                "flagged_count": interpretation.flagged_count,
                "distinct_topic_count": interpretation.distinct_topic_count,
                "dominant_error_type": interpretation.dominant_error_type,
                "interpretation_tags": interpretation.interpretation_tags,
            }),
        }];
        for topic in &interpretation.topic_summaries {
            signals.push(FabricSignal {
                engine_key: "session_runtime".to_string(),
                signal_type: "session_topic_outcome".to_string(),
                status: Some(
                    if topic.accuracy_score >= 7_000 {
                        "stabilize"
                    } else {
                        "repair"
                    }
                    .to_string(),
                ),
                score: Some(topic.accuracy_score),
                topic_id: Some(topic.topic_id),
                node_id: None,
                question_id: None,
                observed_at: observed_at.clone(),
                payload: json!({
                    "topic_name": topic.topic_name,
                    "attempts": topic.attempts,
                    "correct_attempts": topic.correct_attempts,
                    "avg_response_time_ms": topic.avg_response_time_ms,
                    "dominant_error_type": topic.dominant_error_type,
                }),
            });
        }

        let evidence_records =
            self.list_session_runtime_evidence(session_id, limit_events.max(1))?;
        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            session_fabric_inputs(&signals, &evidence_records),
        );

        Ok(Some(SessionEvidenceFabric {
            session_id,
            student_id: session.student_id,
            session_type: session.session_type,
            status: session.status,
            interpretation,
            remediation_plans,
            signals,
            evidence_records,
            orchestration,
        }))
    }

    pub fn get_session(&self, session_id: i64) -> EcoachResult<Option<Session>> {
        self.conn
            .query_row(
                "SELECT id, student_id, session_type, subject_id, status, active_item_index,
                        COALESCE(focus_mode, 0), focus_goal, break_schedule_json, ambient_profile,
                        started_at, paused_at, completed_at, last_activity_at
                 FROM sessions WHERE id = ?1",
                [session_id],
                |row| {
                    let break_schedule_json = row
                        .get::<_, Option<String>>(8)?
                        .map(|raw| serde_json::from_str::<Value>(&raw))
                        .transpose()
                        .map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                8,
                                rusqlite::types::Type::Text,
                                Box::new(EcoachError::Serialization(err.to_string())),
                            )
                        })?;
                    Ok(Session {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        session_type: row.get(2)?,
                        subject_id: row.get(3)?,
                        status: row.get(4)?,
                        active_item_index: row.get(5)?,
                        focus_mode: row.get::<_, i64>(6)? == 1,
                        focus_goal: row.get(7)?,
                        break_schedule_json,
                        ambient_profile: row.get(9)?,
                        started_at: parse_datetime(row.get::<_, Option<String>>(10)?),
                        paused_at: parse_datetime(row.get::<_, Option<String>>(11)?),
                        completed_at: parse_datetime(row.get::<_, Option<String>>(12)?),
                        last_activity_at: parse_datetime(row.get::<_, Option<String>>(13)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn enable_focus_mode(
        &self,
        session_id: i64,
        focus_goal: Option<String>,
        break_schedule_json: Option<Value>,
        ambient_profile: Option<String>,
    ) -> EcoachResult<Session> {
        self.ensure_session_status(session_id, &["active", "paused"])?;
        let now = Utc::now().to_rfc3339();
        let break_schedule_json = match break_schedule_json {
            Some(value) => Some(
                serde_json::to_string(&value)
                    .map_err(|err| EcoachError::Serialization(err.to_string()))?,
            ),
            None => None,
        };
        self.conn
            .execute(
                "UPDATE sessions
                 SET focus_mode = 1,
                     focus_goal = ?1,
                     break_schedule_json = ?2,
                     ambient_profile = ?3,
                     last_activity_at = COALESCE(last_activity_at, ?4),
                     updated_at = datetime('now')
                 WHERE id = ?5",
                params![
                    focus_goal,
                    break_schedule_json,
                    ambient_profile,
                    now,
                    session_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let updated_session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.focus_mode_enabled",
                session_id.to_string(),
                serde_json::json!({
                    "focus_mode": updated_session.focus_mode,
                    "focus_goal": updated_session.focus_goal.clone(),
                    "ambient_profile": updated_session.ambient_profile.clone(),
                }),
            ),
        )?;
        Ok(updated_session)
    }

    pub fn get_focus_mode_config(&self, session_id: i64) -> EcoachResult<Option<FocusModeConfig>> {
        self.conn
            .query_row(
                "SELECT id, COALESCE(focus_mode, 0), focus_goal, break_schedule_json, ambient_profile
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    let break_schedule_json = row
                        .get::<_, Option<String>>(3)?
                        .map(|raw| serde_json::from_str::<Value>(&raw))
                        .transpose()
                        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(EcoachError::Serialization(err.to_string())),
                        ))?;
                    Ok(FocusModeConfig {
                        session_id: row.get(0)?,
                        focus_mode: row.get::<_, i64>(1)? == 1,
                        focus_goal: row.get(2)?,
                        break_schedule_json,
                        ambient_profile: row.get(4)?,
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

    fn load_session_runtime_meta(&self, session_id: i64) -> EcoachResult<SessionRuntimeMeta> {
        self.conn
            .query_row(
                "SELECT session_type, COALESCE(is_timed, 0), COALESCE(total_questions, question_count, 0)
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    Ok(SessionRuntimeMeta {
                        session_type: row.get(0)?,
                        is_timed: row.get::<_, i64>(1)? == 1,
                        total_questions: row.get(2)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_flagged_items(&self, session_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM session_items WHERE session_id = ?1 AND flagged = 1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_attempt_metrics(&self, session_id: i64) -> EcoachResult<AttemptMetrics> {
        let metrics = self
            .conn
            .query_row(
                "SELECT
                    COUNT(*),
                    AVG(response_time_ms),
                    COALESCE(SUM(CASE WHEN misconception_triggered_id IS NOT NULL OR error_type = 'misconception_triggered' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN error_type = 'pressure_breakdown' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN was_transfer_variant = 1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN was_retention_check = 1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN was_mixed_context = 1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN COALESCE(support_level, 'independent') <> 'independent' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN COALESCE(support_level, 'independent') = 'independent' THEN 1 ELSE 0 END), 0),
                    COUNT(DISTINCT q.topic_id)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.session_id = ?1",
                [session_id],
                |row| {
                    Ok(AttemptMetrics {
                        attempt_count: row.get(0)?,
                        avg_response_time_ms: row
                            .get::<_, Option<f64>>(1)?
                            .map(|value| value.round() as i64),
                        misconception_hit_count: row.get(2)?,
                        pressure_breakdown_count: row.get(3)?,
                        transfer_variant_count: row.get(4)?,
                        retention_check_count: row.get(5)?,
                        mixed_context_count: row.get(6)?,
                        supported_answer_count: row.get(7)?,
                        independent_answer_count: row.get(8)?,
                        distinct_topic_count: row.get(9)?,
                        dominant_error_type: None,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or_default();

        let dominant_error_type = self
            .conn
            .query_row(
                "SELECT error_type
                 FROM student_question_attempts
                 WHERE session_id = ?1
                   AND error_type IS NOT NULL
                 GROUP BY error_type
                 ORDER BY COUNT(*) DESC, error_type ASC
                 LIMIT 1",
                [session_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(AttemptMetrics {
            dominant_error_type,
            ..metrics
        })
    }

    fn list_session_topic_interpretations(
        &self,
        session_id: i64,
    ) -> EcoachResult<Vec<SessionTopicInterpretation>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    q.topic_id,
                    t.name,
                    COUNT(*),
                    COALESCE(SUM(CASE WHEN COALESCE(sqa.is_correct, si.is_correct, 0) = 1 THEN 1 ELSE 0 END), 0),
                    AVG(COALESCE(sqa.response_time_ms, si.response_time_ms))
                 FROM session_items si
                 INNER JOIN questions q ON q.id = si.question_id
                 LEFT JOIN topics t ON t.id = q.topic_id
                 LEFT JOIN student_question_attempts sqa
                   ON sqa.session_id = si.session_id
                  AND sqa.question_id = si.question_id
                 WHERE si.session_id = ?1
                   AND COALESCE(si.selected_option_id, sqa.selected_option_id) IS NOT NULL
                 GROUP BY q.topic_id, t.name
                 ORDER BY COUNT(*) DESC, q.topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([session_id], |row| {
                let topic_id = row.get::<_, i64>(0)?;
                Ok(SessionTopicInterpretation {
                    topic_id,
                    topic_name: row
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| "Unknown".to_string()),
                    attempts: row.get(2)?,
                    correct_attempts: row.get(3)?,
                    accuracy_score: to_topic_accuracy(row.get::<_, i64>(2)?, row.get::<_, i64>(3)?),
                    avg_response_time_ms: row
                        .get::<_, Option<f64>>(4)?
                        .map(|value| value.round() as i64),
                    dominant_error_type: None,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        for item in &mut items {
            item.dominant_error_type = self.topic_dominant_error_type(session_id, item.topic_id)?;
        }
        Ok(items)
    }

    fn topic_dominant_error_type(
        &self,
        session_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT sqa.error_type
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.session_id = ?1
                   AND q.topic_id = ?2
                   AND sqa.error_type IS NOT NULL
                 GROUP BY sqa.error_type
                 ORDER BY COUNT(*) DESC, sqa.error_type ASC
                 LIMIT 1",
                params![session_id, topic_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_session_runtime_evidence(
        &self,
        session_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT event_id, event_type, payload_json, occurred_at
                 FROM runtime_events
                 WHERE aggregate_kind = 'session'
                   AND aggregate_id = ?1
                 ORDER BY occurred_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![session_id.to_string(), limit.max(1) as i64],
                |row| {
                    let payload_json: String = row.get(2)?;
                    let payload = serde_json::from_str::<Value>(&payload_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                    Ok(FabricEvidenceRecord {
                        stream: "session_runtime_events".to_string(),
                        reference_id: row.get(0)?,
                        event_type: row.get(1)?,
                        topic_id: payload.get("topic_id").and_then(|value| value.as_i64()),
                        node_id: payload.get("node_id").and_then(|value| value.as_i64()),
                        question_id: payload.get("question_id").and_then(|value| value.as_i64()),
                        occurred_at: row.get(3)?,
                        payload,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
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

    fn tag_items_with_mission_intent(
        &self,
        session_id: i64,
        activity_type: &str,
    ) -> EcoachResult<()> {
        let question_intent = mission_question_intent(activity_type);
        self.conn
            .execute(
                "UPDATE session_items
                 SET question_intent = ?1
                 WHERE session_id = ?2",
                params![question_intent, session_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn build_mission_question_coverage(
        &self,
        subject_id: i64,
        topic_id: i64,
        question_ids: &[i64],
    ) -> EcoachResult<Value> {
        let total_found: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM questions
                 WHERE is_active = 1 AND subject_id = ?1 AND topic_id = ?2",
                params![subject_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let family_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT family_id) FROM questions
                 WHERE is_active = 1 AND subject_id = ?1 AND topic_id = ?2",
                params![subject_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(json!({
            "topic_id": topic_id,
            "total_found": total_found,
            "selected_count": question_ids.len(),
            "family_count": family_count,
            "question_ids": question_ids,
        }))
    }

    fn load_mission_id_for_session(&self, session_id: i64) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id FROM coach_missions WHERE session_id = ?1 ORDER BY id DESC LIMIT 1",
                [session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn ensure_presence_snapshot(
        &self,
        session_id: i64,
        started_at_hint: Option<&str>,
    ) -> EcoachResult<()> {
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        let seeded_at = started_at_hint
            .map(str::to_string)
            .or_else(|| session.started_at.map(|value| value.to_rfc3339()))
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        self.conn
            .execute(
                "INSERT OR IGNORE INTO session_presence_snapshots (
                    session_id, current_state, current_segment_started_at, updated_at
                 ) VALUES (?1, 'launched_unengaged', ?2, ?2)",
                params![session_id, seeded_at],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_presence_snapshot(
        &self,
        session_id: i64,
    ) -> EcoachResult<Option<SessionPresenceSnapshot>> {
        self.conn
            .query_row(
                "SELECT session_id, current_state, current_segment_started_at, first_meaningful_at,
                        last_meaningful_at, idle_started_at, idle_confirmed_at,
                        interruption_started_at, gross_elapsed_ms, active_engaged_ms,
                        passive_engaged_ms, thinking_time_ms, idle_time_ms,
                        interruption_time_ms, counted_study_time_ms, abandonment_risk_bp,
                        updated_at
                 FROM session_presence_snapshots
                 WHERE session_id = ?1",
                [session_id],
                |row| {
                    let updated_at_raw = row.get::<_, String>(16)?;
                    let updated_at = DateTime::parse_from_rfc3339(&updated_at_raw)
                        .map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                16,
                                rusqlite::types::Type::Text,
                                Box::new(EcoachError::Validation(err.to_string())),
                            )
                        })?
                        .with_timezone(&Utc);
                    Ok(SessionPresenceSnapshot {
                        session_id: row.get(0)?,
                        current_state: row.get(1)?,
                        current_segment_started_at: parse_datetime(row.get::<_, Option<String>>(2)?),
                        first_meaningful_at: parse_datetime(row.get::<_, Option<String>>(3)?),
                        last_meaningful_at: parse_datetime(row.get::<_, Option<String>>(4)?),
                        idle_started_at: parse_datetime(row.get::<_, Option<String>>(5)?),
                        idle_confirmed_at: parse_datetime(row.get::<_, Option<String>>(6)?),
                        interruption_started_at: parse_datetime(row.get::<_, Option<String>>(7)?),
                        gross_elapsed_ms: row.get(8)?,
                        active_engaged_ms: row.get(9)?,
                        passive_engaged_ms: row.get(10)?,
                        thinking_time_ms: row.get(11)?,
                        idle_time_ms: row.get(12)?,
                        interruption_time_ms: row.get(13)?,
                        counted_study_time_ms: row.get(14)?,
                        abandonment_risk_bp: row.get(15)?,
                        updated_at,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn persist_presence_snapshot(&self, snapshot: &SessionPresenceSnapshot) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE session_presence_snapshots
                 SET current_state = ?2,
                     current_segment_started_at = ?3,
                     first_meaningful_at = ?4,
                     last_meaningful_at = ?5,
                     idle_started_at = ?6,
                     idle_confirmed_at = ?7,
                     interruption_started_at = ?8,
                     gross_elapsed_ms = ?9,
                     active_engaged_ms = ?10,
                     passive_engaged_ms = ?11,
                     thinking_time_ms = ?12,
                     idle_time_ms = ?13,
                     interruption_time_ms = ?14,
                     counted_study_time_ms = ?15,
                     abandonment_risk_bp = ?16,
                     updated_at = ?17
                 WHERE session_id = ?1",
                params![
                    snapshot.session_id,
                    snapshot.current_state,
                    snapshot.current_segment_started_at.map(|value| value.to_rfc3339()),
                    snapshot.first_meaningful_at.map(|value| value.to_rfc3339()),
                    snapshot.last_meaningful_at.map(|value| value.to_rfc3339()),
                    snapshot.idle_started_at.map(|value| value.to_rfc3339()),
                    snapshot.idle_confirmed_at.map(|value| value.to_rfc3339()),
                    snapshot.interruption_started_at.map(|value| value.to_rfc3339()),
                    snapshot.gross_elapsed_ms,
                    snapshot.active_engaged_ms,
                    snapshot.passive_engaged_ms,
                    snapshot.thinking_time_ms,
                    snapshot.idle_time_ms,
                    snapshot.interruption_time_ms,
                    snapshot.counted_study_time_ms,
                    snapshot.abandonment_risk_bp,
                    snapshot.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn record_presence_event_internal(
        &self,
        session_id: i64,
        event_type: &str,
        occurred_at: &str,
        metadata_json: Value,
    ) -> EcoachResult<()> {
        let session = self
            .get_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        self.ensure_presence_snapshot(session_id, None)?;
        let snapshot = self
            .load_presence_snapshot(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} presence not found", session_id)))?;
        let occurred_at_dt = DateTime::parse_from_rfc3339(occurred_at)
            .map_err(|err| EcoachError::Validation(err.to_string()))?
            .with_timezone(&Utc);
        let (next_snapshot, state_before, state_after, segment_duration_ms, counted_credit_ms) =
            self.apply_presence_transition(&session, snapshot, event_type, occurred_at_dt, &metadata_json);
        self.persist_presence_snapshot(&next_snapshot)?;
        self.conn
            .execute(
                "INSERT INTO session_presence_events (
                    session_id, event_type, occurred_at, state_before, state_after,
                    segment_duration_ms, counted_credit_ms, metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    session_id,
                    event_type,
                    occurred_at,
                    state_before,
                    state_after,
                    segment_duration_ms,
                    counted_credit_ms,
                    metadata_json.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            "session",
            DomainEvent::new(
                "session.presence_transition",
                session_id.to_string(),
                serde_json::json!({
                    "event_type": event_type,
                    "occurred_at": occurred_at,
                    "state_before": state_before,
                    "state_after": state_after,
                    "segment_duration_ms": segment_duration_ms,
                    "counted_credit_ms": counted_credit_ms,
                    "metadata": metadata_json,
                }),
            ),
        )?;
        Ok(())
    }

    fn apply_presence_transition(
        &self,
        session: &Session,
        mut snapshot: SessionPresenceSnapshot,
        event_type: &str,
        occurred_at: DateTime<Utc>,
        metadata_json: &Value,
    ) -> (
        SessionPresenceSnapshot,
        Option<String>,
        String,
        i64,
        i64,
    ) {
        let previous_state = snapshot.current_state.clone();
        let state_before = Some(previous_state.clone());
        let segment_started_at = snapshot
            .current_segment_started_at
            .or(snapshot.last_meaningful_at)
            .or(snapshot.first_meaningful_at)
            .or_else(|| session.started_at.as_ref().map(|value| value.to_owned()))
            .unwrap_or(occurred_at);
        let segment_duration_ms = (occurred_at - segment_started_at).num_milliseconds().max(0);
        let counted_credit_ms = Self::credit_presence_segment(
            &mut snapshot,
            &previous_state,
            segment_duration_ms,
        );
        let explicit_reason = metadata_json
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        let state_after = match event_type {
            "session_started" => "launched_unengaged".to_string(),
            "meaningful_interaction" | "session_resumed" | "reactivated" => {
                if snapshot.first_meaningful_at.is_none() {
                    snapshot.first_meaningful_at = Some(occurred_at);
                }
                snapshot.last_meaningful_at = Some(occurred_at);
                snapshot.idle_started_at = None;
                snapshot.idle_confirmed_at = None;
                snapshot.interruption_started_at = None;
                "active_engaged".to_string()
            }
            "passive_progress_event" => {
                if snapshot.first_meaningful_at.is_none() {
                    snapshot.first_meaningful_at = Some(occurred_at);
                }
                snapshot.last_meaningful_at = Some(occurred_at);
                snapshot.idle_started_at = None;
                snapshot.idle_confirmed_at = None;
                snapshot.interruption_started_at = None;
                "active_passive".to_string()
            }
            "thinking_mode_entered" => {
                if snapshot.first_meaningful_at.is_none() {
                    snapshot.first_meaningful_at = Some(occurred_at);
                }
                snapshot.last_meaningful_at = Some(occurred_at);
                snapshot.idle_started_at = None;
                snapshot.idle_confirmed_at = None;
                snapshot.interruption_started_at = None;
                "thinking_solving".to_string()
            }
            "quiet_started" => {
                snapshot.idle_started_at = snapshot.last_meaningful_at.or(Some(occurred_at));
                "quiet_grace".to_string()
            }
            "idle_suspected" => {
                snapshot.idle_started_at = snapshot.last_meaningful_at.or(Some(occurred_at));
                "suspected_idle".to_string()
            }
            "idle_confirmed" => {
                snapshot.idle_started_at = snapshot.last_meaningful_at.or(Some(occurred_at));
                snapshot.idle_confirmed_at = Some(occurred_at);
                "idle_confirmed".to_string()
            }
            "background_interrupted" => {
                snapshot.interruption_started_at = Some(occurred_at);
                "background_interrupted".to_string()
            }
            "manual_pause" => {
                snapshot.interruption_started_at = Some(occurred_at);
                "manually_paused".to_string()
            }
            "manual_stop" => {
                snapshot.interruption_started_at = Some(occurred_at);
                if explicit_reason
                    .as_deref()
                    .is_some_and(|value| value == "continue_later" || value == "interrupted")
                {
                    "parked_recoverable".to_string()
                } else {
                    "manually_stopped".to_string()
                }
            }
            "session_parked" => {
                snapshot.interruption_started_at = Some(occurred_at);
                "parked_recoverable".to_string()
            }
            "session_abandoned" => "abandoned".to_string(),
            "session_completed" => {
                if snapshot.first_meaningful_at.is_none() {
                    snapshot.first_meaningful_at = session.started_at.as_ref().map(|value| value.to_owned()).or(Some(occurred_at));
                }
                snapshot.last_meaningful_at = Some(occurred_at);
                "completed".to_string()
            }
            _ => snapshot.current_state.clone(),
        };
        snapshot.current_state = state_after.clone();
        snapshot.current_segment_started_at = Some(occurred_at);
        snapshot.gross_elapsed_ms = snapshot
            .first_meaningful_at
            .map(|started| (occurred_at - started).num_milliseconds().max(0))
            .unwrap_or(0);
        snapshot.abandonment_risk_bp = Self::presence_abandonment_risk_bp(&snapshot.current_state);
        snapshot.updated_at = occurred_at;
        (
            snapshot,
            state_before,
            state_after,
            segment_duration_ms,
            counted_credit_ms,
        )
    }

    fn credit_presence_segment(
        snapshot: &mut SessionPresenceSnapshot,
        state_before: &str,
        segment_duration_ms: i64,
    ) -> i64 {
        let duration_ms = segment_duration_ms.max(0);
        match state_before {
            "active_engaged" => {
                snapshot.active_engaged_ms += duration_ms;
                snapshot.counted_study_time_ms += duration_ms;
                duration_ms
            }
            "active_passive" => {
                snapshot.passive_engaged_ms += duration_ms;
                snapshot.counted_study_time_ms += duration_ms;
                duration_ms
            }
            "thinking_solving" | "quiet_grace" => {
                snapshot.thinking_time_ms += duration_ms;
                snapshot.counted_study_time_ms += duration_ms;
                duration_ms
            }
            "suspected_idle" | "idle_confirmed" => {
                snapshot.idle_time_ms += duration_ms;
                0
            }
            "background_interrupted" | "manually_paused" | "parked_recoverable"
            | "manually_stopped" | "abandoned" => {
                snapshot.interruption_time_ms += duration_ms;
                0
            }
            _ => 0,
        }
    }

    fn presence_abandonment_risk_bp(state: &str) -> BasisPoints {
        clamp_bp(match state {
            "suspected_idle" => 5_600,
            "idle_confirmed" => 7_800,
            "background_interrupted" | "manually_paused" | "parked_recoverable" => 4_200,
            "manually_stopped" => 6_400,
            "abandoned" => 9_500,
            _ => 1_200,
        })
    }

    fn estimate_session_time_totals(
        &self,
        session_id: i64,
        started_at: Option<DateTime<Utc>>,
        completed_at: &str,
    ) -> EcoachResult<(i64, i64)> {
        if let Some(snapshot) = self.load_presence_snapshot(session_id)? {
            let active_study_time_ms = snapshot.counted_study_time_ms.max(0);
            let idle_time_ms = (snapshot.idle_time_ms + snapshot.interruption_time_ms).max(0);
            return Ok((active_study_time_ms, idle_time_ms));
        }

        let active_study_time_ms: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(response_time_ms), 0)
                 FROM student_question_attempts
                 WHERE session_id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let completed_at = DateTime::parse_from_rfc3339(completed_at)
            .map_err(|err| EcoachError::Validation(err.to_string()))?
            .with_timezone(&Utc);
        let idle_time_ms = started_at
            .map(|started| {
                let elapsed_ms = (completed_at - started).num_milliseconds().max(0);
                (elapsed_ms - active_study_time_ms).max(0)
            })
            .unwrap_or(0);
        Ok((active_study_time_ms, idle_time_ms))
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
        subject_id: i64,
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
        let family_ids = candidates
            .iter()
            .filter_map(|question| question.family_id)
            .collect::<BTreeSet<_>>();
        let runtime_signals = self
            .load_family_runtime_signals(subject_id, &family_ids)
            .unwrap_or_default();
        let mut buckets = BTreeMap::<i64, Vec<RankedQuestion>>::new();

        for question in candidates {
            let quota = quota_map.get(&question.topic_id);
            let topic_priority = quota
                .map(|item| item.priority_weight as f64 / 10_000.0)
                .unwrap_or(0.5);
            let score = mock_candidate_fit(
                question,
                target_difficulty,
                is_timed,
                topic_priority,
                quota.map(|item| item.is_weak_topic).unwrap_or(false),
                question
                    .family_id
                    .and_then(|family_id| runtime_signals.get(&family_id)),
            );
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

    fn top_up_with_reactor(
        &self,
        subject_id: i64,
        target_topic_counts: &BTreeMap<i64, usize>,
        selected_questions: &mut Vec<SelectedQuestion>,
        target_difficulty: Option<BasisPoints>,
        is_timed: bool,
        variant_mode: QuestionVariantMode,
        request_kind: &str,
        rationale: &str,
    ) -> EcoachResult<usize> {
        let target_total = target_topic_counts.values().copied().sum::<usize>();
        if selected_questions.len() >= target_total || target_total == 0 {
            return Ok(0);
        }

        let reactor = QuestionReactor::new(self.conn);
        let mut current_topic_counts = BTreeMap::<i64, usize>::new();
        for selected in selected_questions.iter() {
            *current_topic_counts
                .entry(selected.question.topic_id)
                .or_default() += 1;
        }

        let mut generated_count = 0usize;
        for (topic_id, target_count) in target_topic_counts {
            let current_count = current_topic_counts.get(topic_id).copied().unwrap_or(0);
            if current_count >= *target_count {
                continue;
            }
            let deficit = *target_count - current_count;
            let seed = self.load_reactor_seed(subject_id, *topic_id)?;
            let slot_spec = QuestionSlotSpec {
                subject_id,
                topic_id: Some(*topic_id),
                target_cognitive_demand: seed
                    .as_ref()
                    .and_then(|item| item.primary_cognitive_demand.clone())
                    .or_else(|| infer_reactor_cognitive_demand(target_difficulty)),
                target_question_format: Some(
                    seed.as_ref()
                        .map(|item| item.question_format.clone())
                        .unwrap_or_else(|| "mcq".to_string()),
                ),
                max_generated_share: 8_000,
            };
            let mut family_priorities =
                reactor.list_family_generation_priorities(&slot_spec, deficit.clamp(1, 4))?;
            let runtime_signals = self.load_family_runtime_signals(
                subject_id,
                &family_priorities
                    .iter()
                    .map(|priority| priority.family_choice.family_id)
                    .collect::<BTreeSet<_>>(),
            )?;
            if family_priorities.is_empty() {
                if seed.as_ref().and_then(|item| item.family_id).is_some() {
                    if let Some(family_choice) = reactor.get_best_family_for_slot(&slot_spec)? {
                        family_priorities.push(ecoach_questions::QuestionFamilyGenerationPriority {
                            rationale: "Fallback to the strongest available seed family for this topic slot.".to_string(),
                            recommended_variant_mode: variant_mode.as_str().to_string(),
                            priority_score: family_choice.fit_score,
                            health_status: "warming".to_string(),
                            freshness_score: 0,
                            calibration_score: 0,
                            quality_score: 0,
                            recurrence_score: 0,
                            replacement_score: 0,
                            family_choice,
                        });
                    }
                }
            }
            if family_priorities.is_empty() {
                continue;
            }
            family_priorities.sort_by(|left, right| {
                enrich_generation_priority_score(
                    right.priority_score,
                    runtime_signals.get(&right.family_choice.family_id),
                    is_timed,
                )
                .cmp(&enrich_generation_priority_score(
                    left.priority_score,
                    runtime_signals.get(&left.family_choice.family_id),
                    is_timed,
                ))
                .then(right.priority_score.cmp(&left.priority_score))
                .then(
                    right
                        .family_choice
                        .fit_score
                        .cmp(&left.family_choice.fit_score),
                )
            });

            for offset in 0..deficit {
                let priority = &family_priorities[offset % family_priorities.len()];
                let family_signal = runtime_signals.get(&priority.family_choice.family_id);
                let chosen_variant_mode = resolve_runtime_generation_variant_mode(
                    variant_mode,
                    &priority.recommended_variant_mode,
                    family_signal,
                    is_timed,
                );
                let request =
                    reactor.create_generation_request(&QuestionGenerationRequestInput {
                        slot_spec: slot_spec.clone(),
                        family_id: Some(priority.family_choice.family_id),
                        source_question_id: seed.as_ref().and_then(|item| {
                            if item.family_id == Some(priority.family_choice.family_id) {
                                item.question_id
                            } else {
                                None
                            }
                        }),
                        request_kind: request_kind.to_string(),
                        variant_mode: chosen_variant_mode,
                        requested_count: 1,
                        rationale: Some(format!(
                            "{} [{}{}]",
                            rationale,
                            priority.rationale,
                            runtime_signal_rationale_suffix(family_signal)
                        )),
                    })?;
                let generated = reactor.process_generation_request(request.id)?;
                for draft in generated {
                    selected_questions.push(SelectedQuestion {
                        fit_score: mock_candidate_fit(
                            &draft.question,
                            target_difficulty,
                            is_timed,
                            0.75,
                            false,
                            draft
                                .question
                                .family_id
                                .and_then(|family_id| runtime_signals.get(&family_id)),
                        ),
                        question: draft.question,
                    });
                    *current_topic_counts.entry(*topic_id).or_default() += 1;
                    generated_count += 1;
                }
            }
        }

        if selected_questions.len() > target_total {
            selected_questions.truncate(target_total);
        }

        Ok(generated_count)
    }

    fn load_reactor_seed(
        &self,
        subject_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<ReactorSeed>> {
        self.conn
            .query_row(
                "SELECT id, family_id, question_format, primary_cognitive_demand
                 FROM questions
                 WHERE subject_id = ?1 AND topic_id = ?2 AND is_active = 1
                 ORDER BY CASE WHEN source_type = 'generated' THEN 1 ELSE 0 END ASC, id ASC
                 LIMIT 1",
                params![subject_id, topic_id],
                |row| {
                    Ok(ReactorSeed {
                        question_id: Some(row.get(0)?),
                        family_id: row.get(1)?,
                        question_format: row.get(2)?,
                        primary_cognitive_demand: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_family_runtime_signals(
        &self,
        subject_id: i64,
        family_ids: &BTreeSet<i64>,
    ) -> EcoachResult<BTreeMap<i64, FamilyRuntimeSignal>> {
        if family_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let (latest_subject_year, subject_year_span) =
            load_subject_year_bounds(self.conn, subject_id)?;
        let placeholders = family_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT qf.id,
                    COALESCE(qfh.health_status, 'warming') AS health_status,
                    COALESCE(qfh.quality_score, 5400) AS quality_score,
                    COALESCE(qfh.calibration_score, 5200) AS calibration_score,
                    COALESCE(qfa.recurrence_score, 0) AS recurrence_score,
                    COALESCE(qfa.coappearance_score, 0) AS coappearance_score,
                    COALESCE(qfa.replacement_score, 0) AS replacement_score,
                    COALESCE((
                        SELECT COUNT(DISTINCT ppql.paper_id)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions history_q ON history_q.id = ppql.question_id
                        WHERE history_q.family_id = qf.id
                    ), 0) AS paper_count,
                    (
                        SELECT MAX(pps.exam_year)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions history_q ON history_q.id = ppql.question_id
                        INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                        WHERE history_q.family_id = qf.id
                    ) AS last_seen_year
             FROM question_families qf
             LEFT JOIN question_family_health qfh ON qfh.family_id = qf.id
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
             WHERE qf.id IN ({})",
            placeholders
        );

        let params_vec = family_ids
            .iter()
            .map(|family_id| rusqlite::types::Value::from(*family_id))
            .collect::<Vec<_>>();
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                let recurrence_score = row.get::<_, i64>(4)?.clamp(0, 10_000) as BasisPoints;
                let coappearance_score = row.get::<_, i64>(5)?.clamp(0, 10_000) as BasisPoints;
                let replacement_score = row.get::<_, i64>(6)?.clamp(0, 10_000) as BasisPoints;
                let paper_count = row.get::<_, i64>(7)?;
                let last_seen_year = row.get::<_, Option<i64>>(8)?;
                Ok((
                    row.get::<_, i64>(0)?,
                    FamilyRuntimeSignal {
                        health_status: row.get::<_, String>(1)?,
                        quality_score: row.get::<_, i64>(2)?.clamp(0, 10_000) as BasisPoints,
                        calibration_score: row.get::<_, i64>(3)?.clamp(0, 10_000) as BasisPoints,
                        recurrence_score,
                        coappearance_score,
                        replacement_score,
                        paper_count,
                        last_seen_year,
                        inverse_pressure_score: composite_inverse_pressure(
                            recurrence_score,
                            coappearance_score,
                            replacement_score,
                        ),
                        comeback_score: compute_comeback_pressure(
                            recurrence_score,
                            coappearance_score,
                            replacement_score,
                            paper_count,
                            last_seen_year,
                            latest_subject_year,
                            subject_year_span,
                        ),
                    },
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut signal_map = BTreeMap::new();
        for row in rows {
            let (family_id, signal) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            signal_map.insert(family_id, signal);
        }
        Ok(signal_map)
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

    fn insert_session_attempt(
        &self,
        session: &Session,
        session_meta: &SessionRuntimeMeta,
        item: &SessionItem,
        question: &Question,
        option: &ecoach_questions::QuestionOption,
        input: &SessionAnswerInput,
    ) -> EcoachResult<()> {
        let attempt_number = self.next_attempt_number(session.student_id, item.question_id)?;
        let submitted_at = Utc::now();
        let started_at = input
            .response_time_ms
            .map(|value| submitted_at - chrono::Duration::milliseconds(value.max(0)))
            .or_else(|| session.started_at.as_ref().map(|value| value.to_owned()))
            .unwrap_or(submitted_at);
        let changed_answer_count = item
            .selected_option_id
            .filter(|selected_option_id| *selected_option_id != input.selected_option_id)
            .map(|_| 1_i64)
            .unwrap_or(0);
        let error_type = classify_session_error_type(
            option.is_correct,
            option.misconception_id,
            input.response_time_ms,
            question.estimated_time_seconds,
            session_meta.is_timed,
        );
        let variant_flags = self.load_question_variant_flags(item.question_id)?;

        self.conn
            .execute(
                "INSERT INTO student_question_attempts (
                    student_id, question_id, session_id, session_type, attempt_number,
                    started_at, submitted_at, response_time_ms, selected_option_id, is_correct,
                    confidence_level, hint_count, changed_answer_count, skipped, timed_out,
                    error_type, misconception_triggered_id, support_level, was_timed,
                    was_transfer_variant, was_retention_check, was_mixed_context, evidence_weight
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, 0, ?11, 0, 0, ?12, ?13, 'independent', ?14, ?15, 0, ?16, ?17)",
                params![
                    session.student_id,
                    item.question_id,
                    session.id,
                    session_meta.session_type.as_str(),
                    attempt_number,
                    started_at.to_rfc3339(),
                    submitted_at.to_rfc3339(),
                    input.response_time_ms,
                    input.selected_option_id,
                    if option.is_correct { 1 } else { 0 },
                    changed_answer_count,
                    error_type,
                    if option.is_correct { None::<i64> } else { option.misconception_id },
                    if session_meta.is_timed { 1 } else { 0 },
                    if variant_flags.was_transfer_variant { 1 } else { 0 },
                    if variant_flags.was_mixed_context { 1 } else { 0 },
                    session_attempt_evidence_weight(
                        option.is_correct,
                        input.response_time_ms,
                        question.estimated_time_seconds,
                        session_meta.is_timed,
                    ),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn next_attempt_number(&self, student_id: i64, question_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM student_question_attempts WHERE student_id = ?1 AND question_id = ?2",
                params![student_id, question_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count + 1)
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_question_variant_flags(&self, question_id: i64) -> EcoachResult<QuestionVariantFlags> {
        let (was_transfer_variant, was_mixed_context) = self
            .conn
            .query_row(
                "SELECT
                    EXISTS(
                        SELECT 1
                        FROM question_graph_edges
                        WHERE to_question_id = ?1
                          AND relation_type IN ('representation_shift', 'difficulty_ladder')
                    ),
                    EXISTS(
                        SELECT 1
                        FROM question_graph_edges
                        WHERE to_question_id = ?1
                          AND relation_type = 'misconception_pair'
                    )",
                [question_id],
                |row| Ok((row.get::<_, i64>(0)? == 1, row.get::<_, i64>(1)? == 1)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(QuestionVariantFlags {
            was_transfer_variant,
            was_mixed_context,
        })
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

#[derive(Debug, Clone)]
struct SessionRuntimeMeta {
    session_type: String,
    is_timed: bool,
    total_questions: i64,
}

#[derive(Debug, Clone, Copy, Default)]
struct QuestionVariantFlags {
    was_transfer_variant: bool,
    was_mixed_context: bool,
}

#[derive(Debug, Clone, Default)]
struct AttemptMetrics {
    attempt_count: i64,
    avg_response_time_ms: Option<i64>,
    misconception_hit_count: i64,
    pressure_breakdown_count: i64,
    transfer_variant_count: i64,
    retention_check_count: i64,
    mixed_context_count: i64,
    supported_answer_count: i64,
    independent_answer_count: i64,
    distinct_topic_count: i64,
    dominant_error_type: Option<String>,
}

fn to_topic_accuracy(attempts: i64, correct_attempts: i64) -> BasisPoints {
    if attempts <= 0 {
        0
    } else {
        clamp_bp(((correct_attempts as f64 / attempts as f64) * 10_000.0).round() as i64)
    }
}

fn derive_session_interpretation_tags(
    is_timed: bool,
    accuracy_score: Option<BasisPoints>,
    unanswered_questions: i64,
    flagged_count: i64,
    metrics: &AttemptMetrics,
) -> Vec<String> {
    let mut tags = Vec::new();

    if is_timed && accuracy_score.unwrap_or(0) < 6_500 {
        tags.push("timed_fragility".to_string());
    }
    if metrics.pressure_breakdown_count > 0 {
        tags.push("pressure_breakdown".to_string());
    }
    if metrics.misconception_hit_count > 0 {
        tags.push("misconception_pressure".to_string());
    }
    if metrics.retention_check_count > 0 && accuracy_score.unwrap_or(0) < 6_000 {
        tags.push("retention_fragility".to_string());
    }
    if metrics.transfer_variant_count > 0 && accuracy_score.unwrap_or(0) < 6_500 {
        tags.push("transfer_fragility".to_string());
    }
    if metrics.mixed_context_count > 0 || metrics.distinct_topic_count > 1 {
        tags.push("mixed_context".to_string());
    }
    if flagged_count > 0 {
        tags.push("review_requested".to_string());
    }
    if unanswered_questions > 0 {
        tags.push("unfinished".to_string());
    }
    if metrics.supported_answer_count > metrics.independent_answer_count
        && metrics.supported_answer_count > 0
    {
        tags.push("support_dependent".to_string());
    }
    if metrics.avg_response_time_ms.unwrap_or_default() >= 45_000
        && accuracy_score.unwrap_or(0) >= 7_000
    {
        tags.push("slow_but_secure".to_string());
    }
    if tags.is_empty() {
        tags.push(
            if accuracy_score.unwrap_or(0) >= 8_000 {
                "stable_progress"
            } else {
                "needs_reinforcement"
            }
            .to_string(),
        );
    }

    tags
}

fn derive_next_action_hint(
    is_timed: bool,
    accuracy_score: Option<BasisPoints>,
    unanswered_questions: i64,
    metrics: &AttemptMetrics,
) -> String {
    if metrics.misconception_hit_count > 0
        || metrics.dominant_error_type.as_deref() == Some("misconception_triggered")
    {
        return "repair_required".to_string();
    }
    if is_timed && (metrics.pressure_breakdown_count > 0 || accuracy_score.unwrap_or(0) < 6_500) {
        return "pressure_repair".to_string();
    }
    if unanswered_questions > 0 {
        return "resume_session".to_string();
    }
    if accuracy_score.unwrap_or(0) >= 8_500 && metrics.supported_answer_count == 0 {
        return "advance_or_mix".to_string();
    }
    if accuracy_score.unwrap_or(0) >= 6_500 {
        return "stabilize_and_review".to_string();
    }
    "rebuild_support".to_string()
}

fn classify_session_error_type(
    is_correct: bool,
    misconception_id: Option<i64>,
    response_time_ms: Option<i64>,
    estimated_time_seconds: i64,
    is_timed: bool,
) -> Option<&'static str> {
    if is_correct {
        return None;
    }
    if misconception_id.is_some() {
        return Some("misconception_triggered");
    }
    let response_time_ms = response_time_ms.unwrap_or_default();
    if is_timed && response_time_ms >= estimated_time_seconds.max(1) * 1_350 {
        Some("pressure_breakdown")
    } else if response_time_ms > 0 && response_time_ms <= estimated_time_seconds.max(1) * 600 {
        Some("carelessness")
    } else {
        Some("knowledge_gap")
    }
}

fn session_attempt_evidence_weight(
    is_correct: bool,
    response_time_ms: Option<i64>,
    estimated_time_seconds: i64,
    is_timed: bool,
) -> BasisPoints {
    let speed_component = match response_time_ms.unwrap_or_default() {
        value if value <= estimated_time_seconds.max(1) * 700 => 1_000,
        value if value <= estimated_time_seconds.max(1) * 1_100 => 700,
        _ => 300,
    };
    let timed_component = if is_timed { 600 } else { 0 };
    let correctness_component = if is_correct { 5_400 } else { 4_600 };
    clamp_bp(correctness_component + speed_component + timed_component)
}

fn session_fabric_inputs(
    signals: &[FabricSignal],
    evidence_records: &[FabricEvidenceRecord],
) -> Vec<String> {
    let mut inputs = if signals.is_empty() {
        Vec::new()
    } else {
        vec![
            "session_outcomes".to_string(),
            "mission_memory_inputs".to_string(),
        ]
    };
    if evidence_records
        .iter()
        .any(|record| record.stream == "session_runtime_events")
    {
        inputs.push("session_evidence".to_string());
    }
    inputs.sort();
    inputs.dedup();
    inputs
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

#[derive(Debug, Clone)]
struct ReactorSeed {
    question_id: Option<i64>,
    family_id: Option<i64>,
    question_format: String,
    primary_cognitive_demand: Option<String>,
}

#[derive(Debug, Clone)]
struct FamilyRuntimeSignal {
    health_status: String,
    quality_score: BasisPoints,
    calibration_score: BasisPoints,
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
    paper_count: i64,
    last_seen_year: Option<i64>,
    inverse_pressure_score: BasisPoints,
    comeback_score: BasisPoints,
}

#[derive(Debug, Clone)]
struct SessionRemediationObservation {
    topic_id: i64,
    question_format: String,
    primary_cognitive_demand: Option<String>,
    family_id: Option<i64>,
    question_id: i64,
    response_time_ms: i64,
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
    is_weak_topic: bool,
    family_signal: Option<&FamilyRuntimeSignal>,
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
    let family_presence = if question.family_id.is_some() {
        0.85
    } else {
        0.55
    };
    let family_quality = family_signal
        .map(|signal| signal.quality_score as f64 / 10_000.0)
        .unwrap_or(family_presence);
    let family_calibration = family_signal
        .map(|signal| signal.calibration_score as f64 / 10_000.0)
        .unwrap_or(0.55);
    let pressure_alignment = family_signal
        .map(|signal| {
            if is_timed {
                (0.55 * bp_ratio(signal.inverse_pressure_score)
                    + 0.45 * bp_ratio(signal.comeback_score))
                .clamp(0.0, 1.0)
            } else if is_weak_topic {
                (0.45 * bp_ratio(signal.inverse_pressure_score)
                    + 0.25 * bp_ratio(signal.comeback_score)
                    + 0.30 * health_alignment(signal.health_status.as_str(), true))
                .clamp(0.0, 1.0)
            } else {
                (0.55 * bp_ratio(signal.inverse_pressure_score)
                    + 0.20 * bp_ratio(signal.comeback_score)
                    + 0.25 * family_quality)
                    .clamp(0.0, 1.0)
            }
        })
        .unwrap_or(0.45);
    let health_fit = family_signal
        .map(|signal| health_alignment(signal.health_status.as_str(), is_weak_topic))
        .unwrap_or(0.55);

    0.28 * difficulty_fit
        + 0.18 * topic_priority
        + 0.14 * timed_fit
        + 0.06 * family_presence
        + 0.10 * family_quality
        + 0.08 * family_calibration
        + 0.10 * pressure_alignment
        + 0.06 * health_fit
}

fn load_subject_year_bounds(conn: &Connection, subject_id: i64) -> EcoachResult<(i64, i64)> {
    let latest_subject_year = conn
        .query_row(
            "SELECT MAX(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
            [subject_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?
        .unwrap_or(0);
    let earliest_subject_year = conn
        .query_row(
            "SELECT MIN(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
            [subject_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?
        .unwrap_or(latest_subject_year);

    Ok((
        latest_subject_year,
        (latest_subject_year - earliest_subject_year).max(1),
    ))
}

fn composite_inverse_pressure(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
) -> BasisPoints {
    clamp_bp(
        (0.45 * replacement_score as f64
            + 0.30 * coappearance_score as f64
            + 0.25 * recurrence_score as f64)
            .round() as i64,
    ) as BasisPoints
}

fn compute_comeback_pressure(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
    paper_count: i64,
    last_seen_year: Option<i64>,
    latest_subject_year: i64,
    subject_year_span: i64,
) -> BasisPoints {
    let dormant_years =
        latest_subject_year.saturating_sub(last_seen_year.unwrap_or(latest_subject_year));
    let dormant_score = scale_score(dormant_years, subject_year_span.max(1));
    let paper_breadth_score = scale_score(paper_count, 6);
    let historical_strength_score = clamp_bp(
        ((0.60 * recurrence_score as f64)
            + (0.25 * coappearance_score as f64)
            + (0.15 * paper_breadth_score as f64))
            .round() as i64,
    );

    clamp_bp(
        ((f64::from(historical_strength_score) * f64::from(dormant_score)) / 10_000.0).round()
            as i64
            + (i64::from(replacement_score) / 4),
    ) as BasisPoints
}

fn scale_score(value: i64, max_value: i64) -> BasisPoints {
    if value <= 0 || max_value <= 0 {
        0
    } else {
        clamp_bp(((value as f64 / max_value as f64) * 10_000.0).round() as i64) as BasisPoints
    }
}

fn bp_ratio(score: BasisPoints) -> f64 {
    (score as f64 / 10_000.0).clamp(0.0, 1.0)
}

fn health_alignment(health_status: &str, is_weak_topic: bool) -> f64 {
    if is_weak_topic {
        match health_status {
            "fragile" => 1.0,
            "warming" => 0.86,
            "missing" => 0.78,
            "active" => 0.60,
            "gold" => 0.42,
            _ => 0.68,
        }
    } else {
        match health_status {
            "gold" => 1.0,
            "active" => 0.86,
            "warming" => 0.68,
            "fragile" => 0.48,
            "missing" => 0.28,
            _ => 0.58,
        }
    }
}

fn enrich_generation_priority_score(
    base_score: BasisPoints,
    family_signal: Option<&FamilyRuntimeSignal>,
    is_timed: bool,
) -> BasisPoints {
    let Some(signal) = family_signal else {
        return base_score;
    };
    let timed_pressure_bonus = if is_timed {
        i64::from(signal.inverse_pressure_score.max(signal.comeback_score)) / 5
    } else {
        i64::from(signal.inverse_pressure_score.max(signal.comeback_score)) / 8
    };
    let health_bonus = match signal.health_status.as_str() {
        "fragile" => 900,
        "warming" => 550,
        "active" => 250,
        "gold" => 120,
        "missing" => 1_100,
        _ => 300,
    };
    clamp_bp(i64::from(base_score) + timed_pressure_bonus + health_bonus) as BasisPoints
}

fn resolve_runtime_generation_variant_mode(
    default_mode: QuestionVariantMode,
    recommended_mode: &str,
    family_signal: Option<&FamilyRuntimeSignal>,
    is_timed: bool,
) -> QuestionVariantMode {
    let resolved = resolve_generation_variant_mode(default_mode, recommended_mode);
    let Some(signal) = family_signal else {
        return resolved;
    };
    if matches!(
        resolved,
        QuestionVariantMode::MisconceptionProbe | QuestionVariantMode::Rescue
    ) {
        return resolved;
    }
    if signal.health_status == "fragile" || signal.calibration_score < 5_400 {
        return QuestionVariantMode::Rescue;
    }
    if is_timed
        && (signal.inverse_pressure_score >= 7_100 || signal.comeback_score >= 6_600)
        && signal.health_status != "missing"
    {
        return QuestionVariantMode::RepresentationShift;
    }
    resolved
}

fn runtime_signal_rationale_suffix(family_signal: Option<&FamilyRuntimeSignal>) -> String {
    let Some(signal) = family_signal else {
        return String::new();
    };
    if signal.comeback_score >= 6_600 {
        " Past-paper comeback pressure is elevated.".to_string()
    } else if signal.inverse_pressure_score >= 7_100 {
        " Past-paper inverse pressure is elevated.".to_string()
    } else {
        String::new()
    }
}

fn enrich_remediation_plan(
    mut plan: QuestionRemediationPlan,
    session_type: &str,
    is_timed: bool,
    response_time_ms: i64,
    family_signal: Option<&FamilyRuntimeSignal>,
) -> QuestionRemediationPlan {
    let mut rationale_parts = vec![plan.rationale.clone()];
    let mut priority_score = i64::from(plan.priority_score);

    if let Some(signal) = family_signal {
        let pressure_peak = signal.inverse_pressure_score.max(signal.comeback_score);
        priority_score += i64::from(pressure_peak) / if is_timed { 5 } else { 7 };
        priority_score += (10_000 - i64::from(signal.calibration_score)).clamp(0, 10_000) / 6;
        priority_score += match signal.health_status.as_str() {
            "fragile" => 1_400,
            "warming" => 700,
            "missing" => 1_800,
            "active" => 250,
            "gold" => 80,
            _ => 450,
        };

        if plan.variant_mode != QuestionVariantMode::MisconceptionProbe.as_str()
            && (signal.health_status == "fragile" || signal.calibration_score < 5_400)
        {
            plan.variant_mode = QuestionVariantMode::Rescue.as_str().to_string();
            rationale_parts.push(format!(
                "Family health is {} with low calibration, so remediation should lower load before rebuilding speed.",
                signal.health_status
            ));
        } else if plan.variant_mode != QuestionVariantMode::MisconceptionProbe.as_str()
            && plan.variant_mode != QuestionVariantMode::Rescue.as_str()
            && is_timed
            && (signal.inverse_pressure_score >= 7_000 || signal.comeback_score >= 6_500)
        {
            plan.variant_mode = QuestionVariantMode::RepresentationShift
                .as_str()
                .to_string();
            rationale_parts.push(
                "Past-paper inverse/comeback pressure is high, so rehearse this family under exam-like surface variation."
                    .to_string(),
            );
        }

        if signal.comeback_score >= 6_500 {
            rationale_parts.push(format!(
                "Dormant paper history still points to comeback risk for this family ({} papers, last seen {:?}).",
                signal.paper_count, signal.last_seen_year
            ));
        } else if signal.inverse_pressure_score >= 7_000 {
            rationale_parts.push(format!(
                "Recurring co-appearance and replacement pressure keep this family exam-relevant (recurrence {}, co-appearance {}, replacement {}).",
                signal.recurrence_score, signal.coappearance_score, signal.replacement_score
            ));
        }
    }

    if response_time_ms >= 55_000
        && plan.variant_mode != QuestionVariantMode::MisconceptionProbe.as_str()
    {
        priority_score += 900;
        plan.variant_mode = QuestionVariantMode::Rescue.as_str().to_string();
        rationale_parts.push(
            "Response speed broke down under live conditions, so the next step should reduce load before restretching."
                .to_string(),
        );
    } else if is_timed && response_time_ms >= 40_000 {
        priority_score += 450;
        rationale_parts.push(
            "Timed conditions amplified the miss, so keep pressure rehearsal in the repair path."
                .to_string(),
        );
    }

    if session_type == "mock"
        && plan.variant_mode == QuestionVariantMode::RepresentationShift.as_str()
    {
        priority_score += 300;
    }

    plan.priority_score = clamp_bp(priority_score) as BasisPoints;
    plan.rationale = rationale_parts.join(" ");
    plan
}

fn distribute_topic_targets(
    topic_ids: &[i64],
    target_question_count: usize,
) -> BTreeMap<i64, usize> {
    let mut targets = BTreeMap::new();
    if topic_ids.is_empty() || target_question_count == 0 {
        return targets;
    }

    for index in 0..target_question_count {
        let topic_id = topic_ids[index % topic_ids.len()];
        *targets.entry(topic_id).or_default() += 1;
    }

    targets
}

fn choose_reactor_variant_mode(
    is_timed: bool,
    target_difficulty: Option<BasisPoints>,
) -> QuestionVariantMode {
    match target_difficulty {
        Some(difficulty) if difficulty >= 7_500 => QuestionVariantMode::Stretch,
        Some(difficulty) if difficulty <= 3_500 => QuestionVariantMode::Rescue,
        _ if is_timed => QuestionVariantMode::RepresentationShift,
        _ => QuestionVariantMode::Isomorphic,
    }
}

fn resolve_generation_variant_mode(
    default_mode: QuestionVariantMode,
    recommended_mode: &str,
) -> QuestionVariantMode {
    match default_mode {
        QuestionVariantMode::Stretch
        | QuestionVariantMode::Rescue
        | QuestionVariantMode::MisconceptionProbe
        | QuestionVariantMode::Fission
        | QuestionVariantMode::Fusion
        | QuestionVariantMode::Adversary => default_mode,
        QuestionVariantMode::RepresentationShift | QuestionVariantMode::Isomorphic => {
            match recommended_mode {
                "representation_shift" => QuestionVariantMode::RepresentationShift,
                "misconception_probe" => QuestionVariantMode::MisconceptionProbe,
                "rescue" => QuestionVariantMode::Rescue,
                "stretch" => QuestionVariantMode::Stretch,
                _ => default_mode,
            }
        }
    }
}

fn infer_reactor_cognitive_demand(target_difficulty: Option<BasisPoints>) -> Option<String> {
    match target_difficulty {
        Some(difficulty) if difficulty >= 7_500 => Some("application".to_string()),
        Some(difficulty) if difficulty <= 3_500 => Some("recognition".to_string()),
        _ => None,
    }
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

fn desired_question_count_for_mission(activity_type: &str, target_minutes: i64) -> usize {
    let baseline = match activity_type {
        "learn" | "worked_example" => 5,
        "guided_practice" | "review" | "memory_reactivation" => 6,
        "repair" | "checkpoint" => 7,
        "speed_drill" | "pressure_conditioning" => 8,
        "mixed_test" => 10,
        _ => 6,
    };
    baseline.max((target_minutes / 4).clamp(4, 12) as usize)
}

fn target_difficulty_for_mission(activity_type: &str) -> Option<BasisPoints> {
    match activity_type {
        "learn" | "worked_example" => Some(3_600),
        "repair" | "memory_reactivation" => Some(4_200),
        "guided_practice" | "review" => Some(5_200),
        "checkpoint" => Some(6_300),
        "mixed_test" => Some(6_800),
        "speed_drill" | "pressure_conditioning" => Some(7_100),
        _ => None,
    }
}

fn mission_uses_timed_questions(activity_type: &str) -> bool {
    matches!(
        activity_type,
        "checkpoint" | "mixed_test" | "speed_drill" | "pressure_conditioning"
    )
}

fn weakness_topic_ids_for_mission(mission: &CoachMissionBrief) -> Vec<i64> {
    match mission.topic_id {
        Some(topic_id)
            if matches!(
                mission.activity_type.as_str(),
                "repair" | "worked_example" | "memory_reactivation" | "review"
            ) =>
        {
            vec![topic_id]
        }
        _ => Vec::new(),
    }
}

fn mission_question_intent(activity_type: &str) -> &'static str {
    match activity_type {
        "learn" => "discovery",
        "guided_practice" => "coverage",
        "worked_example" | "repair" => "repair",
        "review" | "memory_reactivation" => "retention",
        "checkpoint" => "confirmation",
        "speed_drill" | "pressure_conditioning" => "pressure",
        "mixed_test" => "mini_mock",
        _ => "coverage",
    }
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
    use rusqlite::{Connection, params};

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
                question_count: 3,
                is_timed: false,
            })
            .expect("practice session should start");

        assert_eq!(selected_questions.len(), 3);
        assert!(
            selected_questions
                .iter()
                .any(|selected| selected.question.id > 2),
            "reactor should top up beyond the authored sample pool"
        );
        let snapshot = session_service
            .get_session_snapshot(session.id)
            .expect("session snapshot should load")
            .expect("session should exist");
        assert_eq!(snapshot.items.len(), 3);

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
                    selected_option_id: Some(misconception_option.id),
                    answer_text: None,
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
        let interpretation = session_service
            .get_session_interpretation(session.id)
            .expect("session interpretation should load")
            .expect("session interpretation should exist");
        let session_fabric = session_service
            .get_session_evidence_fabric(session.id, 6)
            .expect("session fabric should build")
            .expect("session fabric should exist");

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
        let interpreted_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type = 'session.interpreted' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("session interpreted event count should be queryable");
        let reactor_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type = 'session.reactor_top_up' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("reactor event count should be queryable");

        assert_eq!(mission_count, 1);
        assert_eq!(plan_count, 1);
        assert_eq!(skill_state_count, 1);
        assert_eq!(memory_state_count, 1);
        assert_eq!(recheck_count, 1);
        assert!(runtime_event_count >= 6);
        assert_eq!(interpreted_event_count, 1);
        assert_eq!(reactor_event_count, 1);
        assert_eq!(interpretation.next_action_hint, "repair_required");
        assert!(
            interpretation
                .interpretation_tags
                .iter()
                .any(|tag| tag == "misconception_pressure")
        );
        assert!(
            session_fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "session_outcome")
        );
        assert!(
            session_fabric
                .evidence_records
                .iter()
                .any(|record| record.event_type == "session.interpreted")
        );
        assert!(
            session_fabric
                .orchestration
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "student_truth")
        );
    }

    #[test]
    fn session_presence_tracking_captures_idle_pause_resume_and_completion() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Yaw".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let session_service = SessionService::new(&conn);
        let (session, _) = session_service
            .start_practice_session(&PracticeSessionStartInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                is_timed: false,
            })
            .expect("practice session should start");

        let initial_presence = session_service
            .get_session_presence_snapshot(session.id)
            .expect("presence snapshot should load")
            .expect("presence snapshot should exist");
        assert_eq!(initial_presence.current_state, "launched_unengaged");

        let base = Utc::now() - Duration::minutes(15);
        let base_rfc3339 = base.to_rfc3339();
        conn.execute(
            "UPDATE sessions
             SET started_at = ?1, last_activity_at = ?1
             WHERE id = ?2",
            params![base_rfc3339.clone(), session.id],
        )
        .expect("session timestamps should update");
        conn.execute(
            "UPDATE session_presence_snapshots
             SET current_segment_started_at = ?1, updated_at = ?1
             WHERE session_id = ?2",
            params![base_rfc3339, session.id],
        )
        .expect("presence snapshot timestamps should update");

        let thinking_snapshot = session_service
            .record_session_presence_event(
                session.id,
                &SessionPresenceEventInput {
                    event_type: "thinking_mode_entered".to_string(),
                    occurred_at: Some((base + Duration::minutes(2)).to_rfc3339()),
                    metadata_json: Some(serde_json::json!({ "source": "test" })),
                },
            )
            .expect("thinking event should record");
        assert_eq!(thinking_snapshot.current_state, "thinking_solving");

        let idle_snapshot = session_service
            .record_session_presence_event(
                session.id,
                &SessionPresenceEventInput {
                    event_type: "idle_confirmed".to_string(),
                    occurred_at: Some((base + Duration::minutes(5)).to_rfc3339()),
                    metadata_json: None,
                },
            )
            .expect("idle event should record");
        assert_eq!(idle_snapshot.current_state, "idle_confirmed");
        assert!(idle_snapshot.counted_study_time_ms >= Duration::minutes(3).num_milliseconds());

        let snapshot = session_service
            .get_session_snapshot(session.id)
            .expect("session snapshot should load")
            .expect("session should exist");
        let question_service = QuestionService::new(&conn);
        let option_id = question_service
            .list_options(snapshot.items[0].question_id)
            .expect("question options should load")
            .first()
            .map(|option| option.id)
            .expect("question should have at least one option");
        session_service
            .record_answer(
                session.id,
                &SessionAnswerInput {
                    item_id: snapshot.items[0].id,
                    selected_option_id: option_id,
                    response_time_ms: Some(12_000),
                },
            )
            .expect("answer should record");

        let stopped = session_service
            .manual_stop_session(session.id, Some("continue_later".to_string()))
            .expect("session should stop manually");
        assert_eq!(stopped.session.status, "paused");
        let resumed = session_service
            .resume_session(session.id)
            .expect("session should resume");
        assert_eq!(resumed.session.status, "active");

        let summary = session_service
            .complete_session(session.id)
            .expect("session should complete");
        assert_eq!(summary.status, "completed");
        assert_eq!(summary.answered_questions, 1);

        let final_presence = session_service
            .get_session_presence_snapshot(session.id)
            .expect("final presence snapshot should load")
            .expect("final presence snapshot should exist");
        assert_eq!(final_presence.current_state, "completed");
        assert!(final_presence.counted_study_time_ms >= Duration::minutes(3).num_milliseconds());
        assert!(final_presence.idle_time_ms > 0);

        let presence_events = session_service
            .list_session_presence_events(session.id, 16)
            .expect("presence events should load");
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "thinking_mode_entered"));
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "idle_confirmed"));
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "meaningful_interaction"));
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "manual_stop"));
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "session_resumed"));
        assert!(presence_events
            .iter()
            .any(|event| event.event_type == "session_completed"));

        let (status, active_study_time_ms, idle_time_ms): (String, i64, i64) = conn
            .query_row(
                "SELECT status, COALESCE(active_study_time_ms, 0), COALESCE(idle_time_ms, 0)
                 FROM sessions
                 WHERE id = ?1",
                [session.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("session time totals should load");
        assert_eq!(status, "completed");
        assert_eq!(active_study_time_ms, final_presence.counted_study_time_ms);
        assert_eq!(
            idle_time_ms,
            final_presence.idle_time_ms + final_presence.interruption_time_ms
        );
    }

    #[test]
    fn focus_mode_configuration_round_trips_on_session() {
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
        let (session, _) = session_service
            .start_practice_session(&PracticeSessionStartInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                is_timed: false,
            })
            .expect("practice session should start");

        let updated = session_service
            .enable_focus_mode(
                session.id,
                Some("finish the last five problems".to_string()),
                Some(serde_json::json!([15, 5, 10])),
                Some("quiet_focus".to_string()),
            )
            .expect("focus mode should enable");

        assert!(updated.focus_mode);
        assert_eq!(
            updated.focus_goal.as_deref(),
            Some("finish the last five problems")
        );
        assert_eq!(updated.ambient_profile.as_deref(), Some("quiet_focus"));
        let config = session_service
            .get_focus_mode_config(session.id)
            .expect("focus mode config should load")
            .expect("focus mode config should exist");
        assert!(config.focus_mode);
        assert_eq!(
            config.focus_goal.as_deref(),
            Some("finish the last five problems")
        );
        assert_eq!(config.ambient_profile.as_deref(), Some("quiet_focus"));
        assert_eq!(
            config.break_schedule_json,
            Some(serde_json::json!([15, 5, 10]))
        );
    }

    #[test]
    fn coach_mission_session_uses_real_questions_and_links_back_to_mission() {
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
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student.id],
        )
        .expect("student profile should insert");
        conn.execute(
            "INSERT INTO diagnostic_instances (student_id, subject_id, session_mode, status, started_at, completed_at, result_json)
             VALUES (?1, ?2, 'standard', 'completed', datetime('now'), datetime('now'), '{}')",
            params![student.id, subject_id],
        )
        .expect("diagnostic instance should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, memory_strength, priority_score
             ) VALUES (?1, ?2, 4600, 7800, 5200, 4200, 9300)",
            params![student.id, topic_id],
        )
        .expect("topic state should insert");

        let plan_engine = PlanEngine::new(&conn);
        let exam_date = (Utc::now() + Duration::days(45)).date_naive().to_string();
        plan_engine
            .generate_plan(student.id, "BECE", &exam_date, 60)
            .expect("plan should generate");

        let service = SessionService::new(&conn);
        let mission_session = service
            .start_coach_mission_session(student.id)
            .expect("coach mission session should start");

        assert_eq!(
            mission_session.session_snapshot.session.session_type,
            "coach_mission"
        );
        assert!(!mission_session.question_ids.is_empty());
        assert!(!mission_session.session_snapshot.items.is_empty());
        assert_eq!(
            mission_session.coverage["selected_count"].as_i64(),
            Some(mission_session.question_ids.len() as i64)
        );

        let linked_session_id: Option<i64> = conn
            .query_row(
                "SELECT session_id FROM coach_missions WHERE id = ?1",
                [mission_session.mission_id],
                |row| row.get(0),
            )
            .expect("linked session should query");
        assert_eq!(linked_session_id, Some(mission_session.session_id));

        service
            .complete_session(mission_session.session_id)
            .expect("coach mission session should complete");
        let mission_memory_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_mission_memories WHERE mission_id = ?1",
                [mission_session.mission_id],
                |row| row.get(0),
            )
            .expect("mission memory count should query");
        assert_eq!(mission_memory_count, 1);
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
                question_count: 3,
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
        let reactor_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type = 'session.reactor_top_up' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("custom reactor event count should be queryable");

        assert_eq!(session.session_type, "custom_test");
        assert_eq!(selected_questions.len(), 3);
        assert_eq!(snapshot.items.len(), 3);
        assert_eq!(archetype, "timed_targeted");
        assert_eq!(custom_event_count, 1);
        assert_eq!(reactor_event_count, 1);
    }

    #[test]
    fn session_runtime_records_attempts_and_emits_remediation_plans() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Naa".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let service = SessionService::new(&conn);
        let (session, _) = service
            .start_practice_session(&PracticeSessionStartInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 2,
                is_timed: true,
            })
            .expect("practice session should start");
        let snapshot = service
            .get_session_snapshot(session.id)
            .expect("snapshot should load")
            .expect("session should exist");

        let question_service = QuestionService::new(&conn);
        let options = question_service
            .list_options(snapshot.items[0].question_id)
            .expect("options should load");
        let misconception_option = options
            .iter()
            .find(|option| option.misconception_id.is_some())
            .expect("misconception option should exist");

        service
            .record_answer(
                session.id,
                &SessionAnswerInput {
                    item_id: snapshot.items[0].id,
                    selected_option_id: misconception_option.id,
                    response_time_ms: Some(52_000),
                },
            )
            .expect("answer should record");
        service
            .complete_session(session.id)
            .expect("session should complete");

        let attempt_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM student_question_attempts
                 WHERE student_id = ?1 AND session_id = ?2",
                params![student.id, session.id],
                |row| row.get(0),
            )
            .expect("attempt count should query");
        let family_health_attempts: i64 = conn
            .query_row(
                "SELECT COALESCE(qfh.recent_attempts, 0)
                 FROM question_family_health qfh
                 INNER JOIN session_items si ON si.source_family_id = qfh.family_id
                 WHERE si.id = ?1",
                [snapshot.items[0].id],
                |row| row.get(0),
            )
            .expect("family health should query");
        let remediation_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events
                 WHERE event_type = 'session.remediation_planned' AND aggregate_id = ?1",
                [session.id.to_string()],
                |row| row.get(0),
            )
            .expect("remediation event count should query");
        let fabric = service
            .get_session_evidence_fabric(session.id, 10)
            .expect("fabric should build")
            .expect("fabric should exist");

        assert_eq!(attempt_count, 1);
        assert!(family_health_attempts >= 1);
        assert_eq!(remediation_event_count, 1);
        assert!(!fabric.remediation_plans.is_empty());
        assert_eq!(
            fabric.remediation_plans[0].request_kind,
            "remediation".to_string()
        );
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
                question_count: 4,
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
        let blueprint_reactor_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE event_type = 'mock.blueprint_reactor_top_up' AND aggregate_id = ?1",
                [blueprint.id.to_string()],
                |row| row.get(0),
            )
            .expect("mock blueprint reactor event count should query");

        assert_eq!(session.session_type, "mock");
        assert_eq!(selected_questions.len(), 4);
        assert_eq!(blueprint.status, "compiled");
        assert!(event_count >= 2);
        assert_eq!(blueprint_reactor_event_count, 1);
    }

    #[test]
    fn mock_candidate_fit_rewards_exam_pressure_for_timed_blueprints() {
        let question = Question {
            id: 1,
            subject_id: 1,
            topic_id: 10,
            subtopic_id: None,
            family_id: Some(100),
            stem: "Solve".to_string(),
            question_format: "mcq".to_string(),
            explanation_text: None,
            difficulty_level: 6_200,
            estimated_time_seconds: 55,
            marks: 1,
            primary_skill_id: None,
        };
        let low_pressure = FamilyRuntimeSignal {
            health_status: "active".to_string(),
            quality_score: 7_400,
            calibration_score: 7_100,
            recurrence_score: 4_200,
            coappearance_score: 3_900,
            replacement_score: 2_200,
            paper_count: 1,
            last_seen_year: Some(2025),
            inverse_pressure_score: 3_100,
            comeback_score: 2_400,
        };
        let high_pressure = FamilyRuntimeSignal {
            health_status: "active".to_string(),
            quality_score: 7_400,
            calibration_score: 7_100,
            recurrence_score: 7_800,
            coappearance_score: 7_100,
            replacement_score: 9_000,
            paper_count: 3,
            last_seen_year: Some(2022),
            inverse_pressure_score: 8_150,
            comeback_score: 7_250,
        };

        let low_score = mock_candidate_fit(
            &question,
            Some(6_500),
            true,
            0.7,
            false,
            Some(&low_pressure),
        );
        let high_score = mock_candidate_fit(
            &question,
            Some(6_500),
            true,
            0.7,
            false,
            Some(&high_pressure),
        );

        assert!(high_score > low_score);
    }

    #[test]
    fn remediation_enrichment_uses_family_pressure_and_health() {
        let base_plan = QuestionRemediationPlan {
            family_choice: ecoach_questions::QuestionFamilyChoice {
                family_id: 44,
                family_code: "ALG_CORE".to_string(),
                family_name: "Algebra Core".to_string(),
                subject_id: 1,
                topic_id: Some(10),
                total_instances: 3,
                generated_instances: 0,
                fit_score: 6_900,
            },
            variant_mode: QuestionVariantMode::RepresentationShift
                .as_str()
                .to_string(),
            priority_score: 5_800,
            source_question_id: Some(900),
            request_kind: "remediation".to_string(),
            rationale: "Base remediation plan.".to_string(),
        };
        let signal = FamilyRuntimeSignal {
            health_status: "fragile".to_string(),
            quality_score: 6_400,
            calibration_score: 4_900,
            recurrence_score: 7_300,
            coappearance_score: 6_800,
            replacement_score: 8_900,
            paper_count: 4,
            last_seen_year: Some(2022),
            inverse_pressure_score: 8_000,
            comeback_score: 7_100,
        };

        let enriched = enrich_remediation_plan(base_plan, "mock", true, 58_000, Some(&signal));

        assert_eq!(enriched.variant_mode, QuestionVariantMode::Rescue.as_str());
        assert!(enriched.priority_score > 5_800);
        assert!(
            enriched.rationale.contains("Past-paper") || enriched.rationale.contains("Dormant")
        );
        assert!(enriched.rationale.contains("fragile"));
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
