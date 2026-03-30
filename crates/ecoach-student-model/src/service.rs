use std::{collections::BTreeSet, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use ecoach_questions::{Question, QuestionService};
use ecoach_substrate::{
    BasisPoints, DomainEvent, EcoachError, EcoachResult, EngineRegistry, FabricEvidenceRecord,
    FabricOrchestrationSummary, FabricSignal, LearnerEvidenceFabric, clamp_bp, ema_update, from_bp,
    to_bp,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    AnswerProcessingResult, AnswerSubmission, ErrorType, LearnerTruthDiagnosisSummary,
    LearnerTruthMemorySummary, LearnerTruthSkillSummary, LearnerTruthSnapshot,
    LearnerTruthTopicSummary, MasteryState, MemoryDecayUpdate, MemoryRecheckItem,
    StudentTopicState,
};

pub struct StudentModelService<'a> {
    conn: &'a Connection,
}

impl<'a> StudentModelService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn process_answer(
        &self,
        student_id: i64,
        submission: &AnswerSubmission,
    ) -> EcoachResult<AnswerProcessingResult> {
        let question_service = QuestionService::new(self.conn);
        let question = question_service
            .get_question(submission.question_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("question {} not found", submission.question_id))
            })?;
        let selected_option = question_service
            .get_option(submission.selected_option_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "option {} not found",
                    submission.selected_option_id
                ))
            })?;

        let mut topic_state = self.get_or_create_topic_state(student_id, question.topic_id)?;
        let is_correct = selected_option.is_correct;
        let error_type = if is_correct {
            None
        } else {
            Some(classify_error(submission, &selected_option, &topic_state))
        };
        let evidence_weight = compute_evidence_weight(submission, is_correct);
        self.write_attempt(
            student_id,
            submission,
            question.topic_id,
            is_correct,
            error_type,
            selected_option.misconception_id,
            evidence_weight,
        )?;
        topic_state = self.update_topic_state(topic_state, submission, is_correct)?;
        if let Some(error_type) = error_type {
            self.update_error_profile(student_id, question.topic_id, error_type)?;
        }
        self.update_skill_states(student_id, &question, is_correct)?;
        self.update_memory_state(
            student_id,
            &question,
            submission,
            is_correct,
            evidence_weight,
        )?;
        let wrong_answer_diagnosis = if let Some(error_type) = error_type {
            Some(self.store_wrong_answer_diagnosis(
                student_id,
                &question,
                submission,
                selected_option.misconception_id,
                error_type,
                &topic_state,
            )?)
        } else {
            None
        };
        topic_state = self.recompute_topic_truth(student_id, question.topic_id)?;

        let event = DomainEvent::new(
            "answer.processed",
            student_id.to_string(),
            serde_json::json!({
                "question_id": question.id,
                "topic_id": question.topic_id,
                "is_correct": is_correct,
                "error_type": error_type.map(ErrorType::as_str),
                "mastery_score": topic_state.mastery_score,
                "gap_score": topic_state.gap_score,
            }),
        );
        self.append_runtime_event("learner_truth", event)?;

        Ok(AnswerProcessingResult {
            is_correct,
            error_type,
            diagnosis_summary: wrong_answer_diagnosis
                .as_ref()
                .map(|item| item.diagnosis_summary.clone()),
            recommended_action: wrong_answer_diagnosis
                .as_ref()
                .map(|item| item.recommended_action.clone()),
            explanation: question.explanation_text,
            selected_option_text: selected_option.option_text,
            correct_option_text: question_service.get_correct_option_text(question.id)?,
            updated_mastery: topic_state.mastery_score,
            updated_gap: topic_state.gap_score,
            misconception_info: selected_option.distractor_intent,
        })
    }

    pub fn get_learner_truth_snapshot(
        &self,
        student_id: i64,
    ) -> EcoachResult<LearnerTruthSnapshot> {
        let student_name: String = self
            .conn
            .query_row(
                "SELECT display_name FROM accounts WHERE id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let topic_summaries = self.list_topic_summaries(student_id, 5)?;
        let skill_summaries = self.list_skill_summaries(student_id, 5)?;
        let memory_summaries = self.list_memory_summaries(student_id, 5)?;
        let recent_diagnoses = self.list_recent_diagnosis_summaries(student_id, 5)?;

        let overall_mastery_score = if topic_summaries.is_empty() {
            0
        } else {
            (topic_summaries
                .iter()
                .map(|item| item.mastery_score as i64)
                .sum::<i64>()
                / topic_summaries.len() as i64) as BasisPoints
        };
        let pending_review_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM coach_mission_memories WHERE student_id = ?1 AND review_status = 'pending'",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let due_memory_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM memory_states
                 WHERE student_id = ?1
                   AND review_due_at IS NOT NULL
                   AND review_due_at <= ?2",
                params![student_id, Utc::now().to_rfc3339()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(LearnerTruthSnapshot {
            student_id,
            student_name,
            overall_mastery_score,
            overall_readiness_band: learner_truth_readiness_band(overall_mastery_score).to_string(),
            pending_review_count,
            due_memory_count,
            topic_summaries,
            skill_summaries,
            memory_summaries,
            recent_diagnoses,
        })
    }

    pub fn get_learner_evidence_fabric(
        &self,
        student_id: i64,
        limit_per_stream: usize,
    ) -> EcoachResult<LearnerEvidenceFabric> {
        let snapshot = self.get_learner_truth_snapshot(student_id)?;
        let per_stream_limit = limit_per_stream.max(1);
        let mut signals = Vec::new();

        for topic in &snapshot.topic_summaries {
            signals.push(FabricSignal {
                engine_key: "student_truth".to_string(),
                signal_type: "topic_truth".to_string(),
                status: Some(topic.mastery_state.clone()),
                score: Some(topic.mastery_score),
                topic_id: Some(topic.topic_id),
                node_id: None,
                question_id: None,
                observed_at: topic
                    .next_review_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339()),
                payload: json!({
                    "topic_name": topic.topic_name,
                    "gap_score": topic.gap_score,
                    "priority_score": topic.priority_score,
                    "memory_strength": topic.memory_strength,
                    "next_review_at": topic.next_review_at.map(|value| value.to_rfc3339()),
                }),
            });
        }

        for skill in &snapshot.skill_summaries {
            signals.push(FabricSignal {
                engine_key: "student_truth".to_string(),
                signal_type: "skill_truth".to_string(),
                status: Some(skill.state.clone()),
                score: Some(skill.mastery_score),
                topic_id: None,
                node_id: Some(skill.node_id),
                question_id: None,
                observed_at: Utc::now().to_rfc3339(),
                payload: json!({
                    "title": skill.title,
                    "gap_score": skill.gap_score,
                    "priority_score": skill.priority_score,
                }),
            });
        }

        for memory in &snapshot.memory_summaries {
            signals.push(FabricSignal {
                engine_key: "student_truth".to_string(),
                signal_type: "memory_truth".to_string(),
                status: Some(memory.memory_state.clone()),
                score: Some(memory.memory_strength),
                topic_id: memory.topic_id,
                node_id: memory.node_id,
                question_id: None,
                observed_at: memory
                    .review_due_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339()),
                payload: json!({
                    "topic_name": memory.topic_name,
                    "node_title": memory.node_title,
                    "recall_fluency": memory.recall_fluency,
                    "decay_risk": memory.decay_risk,
                    "review_due_at": memory.review_due_at.map(|value| value.to_rfc3339()),
                }),
            });
        }

        for diagnosis in &snapshot.recent_diagnoses {
            signals.push(FabricSignal {
                engine_key: "diagnostics".to_string(),
                signal_type: "diagnosis_claim".to_string(),
                status: Some(diagnosis.severity.clone()),
                score: Some(severity_basis_points(&diagnosis.severity)),
                topic_id: Some(diagnosis.topic_id),
                node_id: None,
                question_id: None,
                observed_at: diagnosis.created_at.clone(),
                payload: json!({
                    "topic_name": diagnosis.topic_name,
                    "primary_diagnosis": diagnosis.primary_diagnosis,
                    "recommended_action": diagnosis.recommended_action,
                    "diagnosis_id": diagnosis.diagnosis_id,
                }),
            });
        }

        signals.extend(self.list_recent_mission_signals(student_id, per_stream_limit)?);
        signals.extend(self.list_recent_session_signals(student_id, per_stream_limit)?);

        let mut evidence_records = Vec::new();
        evidence_records.extend(self.list_recent_attempt_evidence(student_id, per_stream_limit)?);
        evidence_records.extend(self.list_recent_memory_evidence(student_id, per_stream_limit)?);
        evidence_records.extend(self.list_recent_diagnosis_evidence(student_id, per_stream_limit)?);
        evidence_records.extend(self.list_recent_mission_evidence(student_id, per_stream_limit)?);
        evidence_records.extend(self.list_recent_session_evidence(student_id, per_stream_limit)?);
        evidence_records.extend(self.list_recent_runtime_evidence(student_id, per_stream_limit)?);

        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            learner_fabric_inputs(&signals, &evidence_records),
        );

        Ok(LearnerEvidenceFabric {
            student_id: snapshot.student_id,
            student_name: snapshot.student_name,
            overall_mastery_score: snapshot.overall_mastery_score,
            overall_readiness_band: snapshot.overall_readiness_band,
            pending_review_count: snapshot.pending_review_count,
            due_memory_count: snapshot.due_memory_count,
            signals,
            evidence_records,
            orchestration,
        })
    }

    pub fn run_memory_decay_scan(
        &self,
        student_id: Option<i64>,
        as_of: DateTime<Utc>,
        limit: usize,
    ) -> EcoachResult<Vec<MemoryDecayUpdate>> {
        let scan_limit = limit.max(1);
        let mut candidates = Vec::new();
        if let Some(student_id) = student_id {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                            decay_risk, review_due_at
                     FROM memory_states
                     WHERE student_id = ?1
                       AND review_due_at IS NOT NULL
                       AND review_due_at <= ?2
                     ORDER BY review_due_at ASC, id ASC
                     LIMIT ?3",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map(
                    params![student_id, as_of.to_rfc3339(), scan_limit as i64],
                    |row| {
                        Ok(DecayCandidate {
                            id: row.get(0)?,
                            student_id: row.get(1)?,
                            topic_id: row.get(2)?,
                            node_id: row.get(3)?,
                            memory_state: row.get(4)?,
                            memory_strength: row.get(5)?,
                            decay_risk: row.get(6)?,
                            review_due_at: parse_datetime(row.get::<_, Option<String>>(7)?),
                        })
                    },
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                candidates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        } else {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                            decay_risk, review_due_at
                     FROM memory_states
                     WHERE review_due_at IS NOT NULL
                       AND review_due_at <= ?1
                     ORDER BY review_due_at ASC, id ASC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map(params![as_of.to_rfc3339(), scan_limit as i64], |row| {
                    Ok(DecayCandidate {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        node_id: row.get(3)?,
                        memory_state: row.get(4)?,
                        memory_strength: row.get(5)?,
                        decay_risk: row.get(6)?,
                        review_due_at: parse_datetime(row.get::<_, Option<String>>(7)?),
                    })
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                candidates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }

        let mut updates = Vec::new();
        let mut affected_topics = BTreeSet::new();
        for candidate in candidates {
            let Some(review_due_at) = candidate.review_due_at else {
                continue;
            };

            let overdue_days = (as_of - review_due_at).num_hours().max(0) / 24;
            let decay_penalty = 450 + overdue_days * 425 + (candidate.decay_risk as i64 / 10);
            let next_strength =
                clamp_bp(candidate.memory_strength as i64 - decay_penalty.clamp(0, 4500));
            let next_decay_risk = clamp_bp(candidate.decay_risk as i64 + 300 + overdue_days * 250);
            let next_state =
                resolve_decay_memory_state(next_strength, next_decay_risk, overdue_days);

            self.conn
                .execute(
                    "UPDATE memory_states
                     SET memory_state = ?1,
                         memory_strength = ?2,
                         decay_risk = ?3,
                         updated_at = datetime('now')
                     WHERE id = ?4",
                    params![next_state, next_strength, next_decay_risk, candidate.id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            if overdue_days >= 1 {
                self.conn
                    .execute(
                        "UPDATE recheck_schedules
                         SET status = 'missed'
                         WHERE student_id = ?1
                           AND (?2 IS NULL OR node_id = ?2)
                           AND status = 'pending'
                           AND due_at < ?3",
                        params![
                            candidate.student_id,
                            candidate.node_id,
                            (as_of - Duration::hours(24)).to_rfc3339(),
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }

            self.append_runtime_event(
                "learner_truth",
                DomainEvent::new(
                    "memory.decay_scanned",
                    candidate.student_id.to_string(),
                    json!({
                        "memory_state_id": candidate.id,
                        "topic_id": candidate.topic_id,
                        "node_id": candidate.node_id,
                        "previous_state": candidate.memory_state,
                        "next_state": next_state,
                        "overdue_days": overdue_days,
                    }),
                ),
            )?;

            updates.push(MemoryDecayUpdate {
                memory_state_id: candidate.id,
                student_id: candidate.student_id,
                topic_id: candidate.topic_id,
                node_id: candidate.node_id,
                previous_state: candidate.memory_state,
                next_state: next_state.to_string(),
                previous_strength: candidate.memory_strength,
                next_strength,
                decay_risk: next_decay_risk,
                review_due_at: Some(review_due_at),
                overdue_days,
            });

            if let Some(topic_id) = candidate.topic_id {
                affected_topics.insert((candidate.student_id, topic_id));
            }
        }

        for (student_id, topic_id) in affected_topics {
            let _ = self.recompute_topic_truth(student_id, topic_id)?;
        }

        Ok(updates)
    }

    pub fn list_due_rechecks(
        &self,
        student_id: i64,
        as_of: DateTime<Utc>,
        limit: usize,
    ) -> EcoachResult<Vec<MemoryRecheckItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT rs.id, rs.student_id, ms.topic_id, t.name, rs.node_id, an.canonical_title,
                        rs.due_at, rs.schedule_type, rs.status, ms.memory_state, ms.decay_risk
                 FROM recheck_schedules rs
                 LEFT JOIN memory_states ms
                    ON ms.student_id = rs.student_id
                   AND ((ms.node_id IS NOT NULL AND ms.node_id = rs.node_id)
                        OR (ms.node_id IS NULL AND rs.node_id IS NULL))
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 LEFT JOIN academic_nodes an ON an.id = rs.node_id
                 WHERE rs.student_id = ?1
                   AND rs.status = 'pending'
                   AND rs.due_at <= ?2
                 ORDER BY rs.due_at ASC, rs.id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, as_of.to_rfc3339(), limit.max(1) as i64],
                |row| {
                    let due_at = row.get::<_, String>(6)?;
                    let due_at = DateTime::<Utc>::from_str(&due_at).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                    Ok(MemoryRecheckItem {
                        schedule_id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        topic_name: row.get(3)?,
                        node_id: row.get(4)?,
                        node_title: row.get(5)?,
                        due_at: due_at.with_timezone(&Utc),
                        schedule_type: row.get(7)?,
                        status: row.get(8)?,
                        memory_state: row.get(9)?,
                        decay_risk: row.get(10)?,
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

    pub fn get_or_create_topic_state(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<StudentTopicState> {
        if let Some(state) = self.find_topic_state(student_id, topic_id)? {
            return Ok(state);
        }

        self.conn
            .execute(
                "INSERT INTO student_topic_states (student_id, topic_id) VALUES (?1, ?2)",
                params![student_id, topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.find_topic_state(student_id, topic_id)?
            .ok_or_else(|| EcoachError::Storage("topic state was not created".to_string()))
    }

    fn find_topic_state(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<StudentTopicState>> {
        self.conn
            .query_row(
                "SELECT id, student_id, topic_id, mastery_score, mastery_state, accuracy_score, speed_score,
                        confidence_score, retention_score, transfer_score, consistency_score, gap_score,
                        priority_score, trend_state, fragility_score, pressure_collapse_index,
                        total_attempts, correct_attempts, evidence_count, last_seen_at, last_correct_at,
                        memory_strength, next_review_at, version
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok(StudentTopicState {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        mastery_score: row.get(3)?,
                        mastery_state: parse_mastery_state(row.get::<_, String>(4)?)?,
                        accuracy_score: row.get(5)?,
                        speed_score: row.get(6)?,
                        confidence_score: row.get(7)?,
                        retention_score: row.get(8)?,
                        transfer_score: row.get(9)?,
                        consistency_score: row.get(10)?,
                        gap_score: row.get(11)?,
                        priority_score: row.get(12)?,
                        trend_state: row.get(13)?,
                        fragility_score: row.get(14)?,
                        pressure_collapse_index: row.get(15)?,
                        total_attempts: row.get(16)?,
                        correct_attempts: row.get(17)?,
                        evidence_count: row.get(18)?,
                        last_seen_at: parse_datetime(row.get::<_, Option<String>>(19)?),
                        last_correct_at: parse_datetime(row.get::<_, Option<String>>(20)?),
                        memory_strength: row.get(21)?,
                        next_review_at: parse_datetime(row.get::<_, Option<String>>(22)?),
                        version: row.get(23)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn update_topic_state(
        &self,
        mut state: StudentTopicState,
        submission: &AnswerSubmission,
        is_correct: bool,
    ) -> EcoachResult<StudentTopicState> {
        let alpha = 0.3;
        let correctness_bp = if is_correct { 10_000 } else { 0 };
        state.accuracy_score = ema_update(state.accuracy_score, correctness_bp, alpha);

        if let Some(response_time_ms) = submission.response_time_ms {
            let expected_ms = 30_000f64;
            let speed = (expected_ms / (response_time_ms.max(1) as f64)).clamp(0.0, 1.0);
            state.speed_score = ema_update(state.speed_score, to_bp(speed), alpha);
        }

        if let Some(confidence_level) = &submission.confidence_level {
            let confidence = match confidence_level.as_str() {
                "sure" => 9000,
                "not_sure" => 5500,
                "guessed" => 2500,
                _ => 5000,
            };
            state.confidence_score = ema_update(state.confidence_score, confidence, alpha);
        }

        if submission.was_transfer_variant {
            state.transfer_score = ema_update(state.transfer_score, correctness_bp, alpha);
        }
        if submission.was_retention_check {
            state.retention_score = ema_update(state.retention_score, correctness_bp, alpha);
        }

        state.total_attempts += 1;
        if is_correct {
            state.correct_attempts += 1;
            state.last_correct_at = Some(Utc::now());
        }
        state.evidence_count += 1;
        state.last_seen_at = Some(Utc::now());

        state.consistency_score = to_bp(
            (state.correct_attempts as f64 / state.total_attempts.max(1) as f64).clamp(0.0, 1.0),
        );
        state.mastery_score = compute_mastery(&state);
        state.gap_score = clamp_bp(10_000 - state.mastery_score as i64);
        state.priority_score = compute_priority_from_state(&state);
        state.mastery_state = resolve_mastery_state(&state);
        state.trend_state = compute_trend_label(&state);
        state.version += 1;

        self.conn
            .execute(
                "UPDATE student_topic_states
                 SET mastery_score = ?1, mastery_state = ?2, accuracy_score = ?3, speed_score = ?4,
                     confidence_score = ?5, retention_score = ?6, transfer_score = ?7,
                     consistency_score = ?8, gap_score = ?9, priority_score = ?10,
                     trend_state = ?11, total_attempts = ?12, correct_attempts = ?13,
                     evidence_count = ?14, last_seen_at = ?15, last_correct_at = ?16,
                     version = ?17, updated_at = datetime('now')
                 WHERE id = ?18",
                params![
                    state.mastery_score,
                    state.mastery_state.as_str(),
                    state.accuracy_score,
                    state.speed_score,
                    state.confidence_score,
                    state.retention_score,
                    state.transfer_score,
                    state.consistency_score,
                    state.gap_score,
                    state.priority_score,
                    state.trend_state,
                    state.total_attempts,
                    state.correct_attempts,
                    state.evidence_count,
                    state.last_seen_at.map(|dt| dt.to_rfc3339()),
                    state.last_correct_at.map(|dt| dt.to_rfc3339()),
                    state.version,
                    state.id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(state)
    }

    fn write_attempt(
        &self,
        student_id: i64,
        submission: &AnswerSubmission,
        topic_id: i64,
        is_correct: bool,
        error_type: Option<ErrorType>,
        misconception_id: Option<i64>,
        evidence_weight: BasisPoints,
    ) -> EcoachResult<()> {
        let attempt_number = self.next_attempt_number(student_id, submission.question_id)?;
        self.conn
            .execute(
                "INSERT INTO student_question_attempts (
                    student_id, question_id, session_id, session_type, attempt_number,
                    started_at, submitted_at, response_time_ms, selected_option_id, is_correct,
                    confidence_level, hint_count, changed_answer_count, skipped, timed_out,
                    error_type, misconception_triggered_id, support_level, was_timed,
                    was_transfer_variant, was_retention_check, was_mixed_context, evidence_weight
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
                params![
                    student_id,
                    submission.question_id,
                    submission.session_id,
                    submission.session_type,
                    attempt_number,
                    submission.started_at.to_rfc3339(),
                    submission.submitted_at.to_rfc3339(),
                    submission.response_time_ms,
                    submission.selected_option_id,
                    bool_to_i64(is_correct),
                    submission.confidence_level,
                    submission.hint_count,
                    submission.changed_answer_count,
                    bool_to_i64(submission.skipped),
                    bool_to_i64(submission.timed_out),
                    error_type.map(ErrorType::as_str),
                    misconception_id,
                    submission.support_level.as_deref().unwrap_or("independent"),
                    bool_to_i64(submission.was_timed),
                    bool_to_i64(submission.was_transfer_variant),
                    bool_to_i64(submission.was_retention_check),
                    bool_to_i64(submission.was_mixed_context),
                    evidence_weight,
                ],
            )
            .map_err(|err| EcoachError::Storage(format!("failed to write attempt for topic {}: {}", topic_id, err)))?;
        Ok(())
    }

    fn next_attempt_number(&self, student_id: i64, question_id: i64) -> EcoachResult<i64> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM student_question_attempts WHERE student_id = ?1 AND question_id = ?2",
                params![student_id, question_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(count + 1)
    }

    fn update_error_profile(
        &self,
        student_id: i64,
        topic_id: i64,
        error_type: ErrorType,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO student_error_profiles (student_id, topic_id) VALUES (?1, ?2)",
                params![student_id, topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let column = match error_type {
            ErrorType::KnowledgeGap => "knowledge_gap_score",
            ErrorType::ConceptualConfusion => "conceptual_confusion_score",
            ErrorType::RecognitionFailure => "recognition_failure_score",
            ErrorType::ExecutionError => "execution_error_score",
            ErrorType::Carelessness => "carelessness_score",
            ErrorType::PressureBreakdown => "pressure_breakdown_score",
            ErrorType::ExpressionWeakness => "expression_weakness_score",
            ErrorType::SpeedError => "speed_error_score",
            ErrorType::GuessingDetected | ErrorType::MisconceptionTriggered => {
                "conceptual_confusion_score"
            }
        };

        let sql = format!(
            "UPDATE student_error_profiles
             SET {column} = MIN({column} + 500, 10000), updated_at = datetime('now')
             WHERE student_id = ?1 AND topic_id = ?2"
        );

        self.conn
            .execute(&sql, params![student_id, topic_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn update_memory_state(
        &self,
        student_id: i64,
        question: &Question,
        submission: &AnswerSubmission,
        is_correct: bool,
        evidence_weight: BasisPoints,
    ) -> EcoachResult<()> {
        let recall_mode = if submission.was_retention_check {
            "retention_check"
        } else if submission.was_transfer_variant {
            "transfer"
        } else {
            "standard"
        };
        let cue_level = submission.support_level.as_deref().unwrap_or("independent");
        let delay_bucket = if submission.was_retention_check {
            "scheduled"
        } else if submission.was_transfer_variant {
            "transfer"
        } else {
            "immediate"
        };
        let fluency_score = compute_recall_fluency(submission.response_time_ms);
        let now = Utc::now();

        self.conn
            .execute(
                "INSERT INTO memory_evidence_events (
                    student_id, node_id, topic_id, recall_mode, cue_level, delay_bucket,
                    interference_detected, was_correct, confidence_level, evidence_weight
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9)",
                params![
                    student_id,
                    question.primary_skill_id,
                    question.topic_id,
                    recall_mode,
                    cue_level,
                    delay_bucket,
                    bool_to_i64(is_correct),
                    submission.confidence_level,
                    evidence_weight,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let existing =
            self.find_memory_state(student_id, question.topic_id, question.primary_skill_id)?;
        let prior_strength = existing
            .as_ref()
            .map(|state| state.memory_strength)
            .unwrap_or(0);
        let prior_fluency = existing
            .as_ref()
            .map(|state| state.recall_fluency)
            .unwrap_or(0);
        let strength_target = if is_correct { 10_000 } else { 0 };
        let updated_strength = ema_update(prior_strength, strength_target, 0.25);
        let updated_fluency = ema_update(prior_fluency, fluency_score, 0.25);
        let decay_risk = compute_decay_risk(updated_strength, updated_fluency, is_correct);
        let memory_state = resolve_memory_state(updated_strength, decay_risk, is_correct);
        let review_due_at = compute_review_due_at(
            now,
            updated_strength,
            updated_fluency,
            is_correct,
            submission.confidence_level.as_deref(),
        );

        if let Some(existing) = existing {
            self.conn
                .execute(
                    "UPDATE memory_states
                     SET memory_state = ?1,
                         memory_strength = ?2,
                         recall_fluency = ?3,
                         decay_risk = ?4,
                         review_due_at = ?5,
                         last_recalled_at = ?6,
                         updated_at = datetime('now')
                     WHERE id = ?7",
                    params![
                        memory_state,
                        updated_strength,
                        updated_fluency,
                        decay_risk,
                        review_due_at.to_rfc3339(),
                        now.to_rfc3339(),
                        existing.id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO memory_states (
                        student_id, topic_id, node_id, memory_state, memory_strength, recall_fluency,
                        decay_risk, review_due_at, last_recalled_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        student_id,
                        question.topic_id,
                        question.primary_skill_id,
                        memory_state,
                        updated_strength,
                        updated_fluency,
                        decay_risk,
                        review_due_at.to_rfc3339(),
                        now.to_rfc3339(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "UPDATE recheck_schedules
                 SET status = 'cancelled', completed_at = ?1
                 WHERE student_id = ?2
                   AND (node_id = ?3 OR (node_id IS NULL AND ?3 IS NULL))
                   AND status = 'pending'",
                params![now.to_rfc3339(), student_id, question.primary_skill_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO recheck_schedules (student_id, node_id, due_at, schedule_type, status)
                 VALUES (?1, ?2, ?3, 'spaced_review', 'pending')",
                params![
                    student_id,
                    question.primary_skill_id,
                    review_due_at.to_rfc3339()
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_runtime_event(
            "memory",
            DomainEvent::new(
                "memory.updated",
                student_id.to_string(),
                serde_json::json!({
                    "topic_id": question.topic_id,
                    "node_id": question.primary_skill_id,
                    "memory_state": memory_state,
                    "memory_strength": updated_strength,
                    "decay_risk": decay_risk,
                    "review_due_at": review_due_at.to_rfc3339(),
                }),
            ),
        )?;
        Ok(())
    }

    fn recompute_topic_truth(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<StudentTopicState> {
        let mut state = self.get_or_create_topic_state(student_id, topic_id)?;
        let now = Utc::now().to_rfc3339();
        let (
            memory_item_count,
            fragile_item_count,
            collapsed_item_count,
            due_item_count,
            average_memory_strength,
            average_decay_risk,
            next_review_at,
        ): (i64, i64, i64, i64, i64, i64, Option<String>) = self
            .conn
            .query_row(
                "SELECT
                    COUNT(*),
                    COALESCE(SUM(CASE WHEN memory_state IN ('fragile', 'at_risk', 'fading', 'rebuilding') THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN memory_state = 'collapsed' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN review_due_at IS NOT NULL AND review_due_at <= ?3 THEN 1 ELSE 0 END), 0),
                    CAST(COALESCE(AVG(memory_strength), 0) AS INTEGER),
                    CAST(COALESCE(AVG(decay_risk), 0) AS INTEGER),
                    MIN(review_due_at)
                 FROM memory_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id, now],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (knowledge_gap_score, conceptual_confusion_score, recognition_failure_score, execution_error_score): (i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT knowledge_gap_score, conceptual_confusion_score, recognition_failure_score, execution_error_score
                 FROM student_error_profiles
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0, 0, 0));

        state.memory_strength = clamp_bp(average_memory_strength);
        state.next_review_at = parse_datetime(next_review_at.clone());

        let structural_fragility = if memory_item_count > 0 {
            ((fragile_item_count * 5000) + (collapsed_item_count * 9000) + (due_item_count * 2500))
                / memory_item_count.max(1)
        } else {
            0
        };
        state.fragility_score = clamp_bp(
            ((structural_fragility as f64 * 0.65) + (average_decay_risk as f64 * 0.35)).round()
                as i64,
        );
        state.priority_score = compute_priority_from_state(&state);
        state.trend_state = compute_trend_label(&state);

        let repair_priority = clamp_bp(
            ((state.gap_score as f64 * 0.50)
                + (knowledge_gap_score as f64 * 0.20)
                + (conceptual_confusion_score as f64 * 0.15)
                + (recognition_failure_score as f64 * 0.075)
                + (execution_error_score as f64 * 0.075)
                + (collapsed_item_count.min(3) as f64 * 250.0)
                + (due_item_count.min(4) as f64 * 150.0))
                .round() as i64,
        );
        let is_urgent = collapsed_item_count > 0 || due_item_count > 0 || repair_priority >= 7000;

        self.conn
            .execute(
                "UPDATE student_topic_states
                 SET memory_strength = ?1,
                     next_review_at = ?2,
                     fragility_score = ?3,
                     decay_risk = ?4,
                     priority_score = ?5,
                     repair_priority = ?6,
                     is_urgent = ?7,
                     last_decline_at = CASE
                         WHEN ?7 = 1 AND (?4 >= 7000 OR ?3 >= 6500) THEN datetime('now')
                         ELSE last_decline_at
                     END,
                     updated_at = datetime('now')
                 WHERE id = ?8",
                params![
                    state.memory_strength,
                    next_review_at,
                    state.fragility_score,
                    clamp_bp(average_decay_risk),
                    state.priority_score,
                    repair_priority,
                    bool_to_i64(is_urgent),
                    state.id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.sync_active_gap_repair_plans(student_id, topic_id, repair_priority)?;
        self.append_runtime_event(
            "learner_truth",
            DomainEvent::new(
                "learner_truth.topic_recomputed",
                student_id.to_string(),
                json!({
                    "topic_id": topic_id,
                    "memory_strength": state.memory_strength,
                    "fragility_score": state.fragility_score,
                    "priority_score": state.priority_score,
                    "repair_priority": repair_priority,
                    "due_item_count": due_item_count,
                    "collapsed_item_count": collapsed_item_count,
                    "next_review_at": next_review_at,
                }),
            ),
        )?;

        self.find_topic_state(student_id, topic_id)?.ok_or_else(|| {
            EcoachError::Storage("topic state disappeared after recompute".to_string())
        })
    }

    fn sync_active_gap_repair_plans(
        &self,
        student_id: i64,
        topic_id: i64,
        repair_priority: BasisPoints,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE gap_repair_plans
                 SET priority_score = ?1,
                     updated_at = datetime('now')
                 WHERE student_id = ?2
                   AND topic_id = ?3
                   AND status = 'active'",
                params![repair_priority, student_id, topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn update_skill_states(
        &self,
        student_id: i64,
        question: &Question,
        is_correct: bool,
    ) -> EcoachResult<()> {
        let skill_ids = self.load_question_skill_ids(question)?;
        let now = Utc::now().to_rfc3339();

        for node_id in skill_ids {
            let existing = self.find_skill_state(student_id, node_id)?;
            let total_attempts = existing
                .as_ref()
                .map(|state| state.total_attempts)
                .unwrap_or(0)
                + 1;
            let mut correct_attempts = existing
                .as_ref()
                .map(|state| state.correct_attempts)
                .unwrap_or(0);
            if is_correct {
                correct_attempts += 1;
            }
            let evidence_count = existing
                .as_ref()
                .map(|state| state.evidence_count)
                .unwrap_or(0)
                + 1;
            let mastery_score =
                to_bp((correct_attempts as f64 / total_attempts.max(1) as f64).clamp(0.0, 1.0));
            let gap_score = clamp_bp(10_000 - mastery_score as i64);
            let priority_score = gap_score;
            let state = resolve_skill_state(mastery_score);

            if let Some(existing) = existing {
                self.conn
                    .execute(
                        "UPDATE student_skill_states
                         SET mastery_score = ?1,
                             gap_score = ?2,
                             priority_score = ?3,
                             evidence_count = ?4,
                             total_attempts = ?5,
                             correct_attempts = ?6,
                             last_seen_at = ?7,
                             last_correct_at = CASE WHEN ?8 = 1 THEN ?7 ELSE last_correct_at END,
                             state = ?9,
                             updated_at = datetime('now')
                         WHERE id = ?10",
                        params![
                            mastery_score,
                            gap_score,
                            priority_score,
                            evidence_count,
                            total_attempts,
                            correct_attempts,
                            now,
                            bool_to_i64(is_correct),
                            state,
                            existing.id,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            } else {
                self.conn
                    .execute(
                        "INSERT INTO student_skill_states (
                            student_id, node_id, mastery_score, gap_score, priority_score,
                            evidence_count, total_attempts, correct_attempts, last_seen_at,
                            last_correct_at, state
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            student_id,
                            node_id,
                            mastery_score,
                            gap_score,
                            priority_score,
                            evidence_count,
                            total_attempts,
                            correct_attempts,
                            now,
                            if is_correct { Some(now.clone()) } else { None },
                            state,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }

            self.append_runtime_event(
                "skill_truth",
                DomainEvent::new(
                    "skill_truth.updated",
                    student_id.to_string(),
                    serde_json::json!({
                        "node_id": node_id,
                        "mastery_score": mastery_score,
                        "gap_score": gap_score,
                        "state": state,
                    }),
                ),
            )?;
        }

        Ok(())
    }

    fn store_wrong_answer_diagnosis(
        &self,
        student_id: i64,
        question: &Question,
        submission: &AnswerSubmission,
        misconception_id: Option<i64>,
        error_type: ErrorType,
        topic_state: &StudentTopicState,
    ) -> EcoachResult<WrongAnswerDiagnosisRecord> {
        let primary_diagnosis = primary_diagnosis_label(error_type);
        let secondary_diagnosis = secondary_diagnosis_label(submission, topic_state);
        let severity = diagnosis_severity(error_type, submission, topic_state);
        let recommended_action = recommended_action_label(error_type);
        let confidence_score = diagnosis_confidence_score(error_type, misconception_id);
        let diagnosis_summary = diagnosis_summary_text(
            error_type,
            question.topic_id,
            submission,
            topic_state,
            misconception_id,
        );

        self.conn
            .execute(
                "INSERT INTO wrong_answer_diagnoses (
                    student_id, question_id, topic_id, session_id, misconception_id, error_type,
                    primary_diagnosis, secondary_diagnosis, severity, diagnosis_summary,
                    recommended_action, confidence_score
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    student_id,
                    question.id,
                    question.topic_id,
                    submission.session_id,
                    misconception_id,
                    error_type.as_str(),
                    primary_diagnosis,
                    secondary_diagnosis,
                    severity,
                    diagnosis_summary,
                    recommended_action,
                    confidence_score,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_runtime_event(
            "wrong_answer",
            DomainEvent::new(
                "wrong_answer.diagnosed",
                student_id.to_string(),
                serde_json::json!({
                    "question_id": question.id,
                    "topic_id": question.topic_id,
                    "error_type": error_type.as_str(),
                    "primary_diagnosis": primary_diagnosis,
                    "severity": severity,
                    "recommended_action": recommended_action,
                }),
            ),
        )?;

        Ok(WrongAnswerDiagnosisRecord {
            diagnosis_summary,
            recommended_action: recommended_action.to_string(),
        })
    }

    fn load_question_skill_ids(&self, question: &Question) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT node_id
                 FROM question_skill_links
                 WHERE question_id = ?1
                 ORDER BY is_primary DESC, contribution_weight DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([question.id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut node_ids = Vec::new();
        for row in rows {
            node_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        if node_ids.is_empty() {
            if let Some(primary_skill_id) = question.primary_skill_id {
                node_ids.push(primary_skill_id);
            }
        }

        Ok(node_ids)
    }

    fn find_skill_state(
        &self,
        student_id: i64,
        node_id: i64,
    ) -> EcoachResult<Option<SkillStateRecord>> {
        self.conn
            .query_row(
                "SELECT id, evidence_count, total_attempts, correct_attempts
                 FROM student_skill_states
                 WHERE student_id = ?1 AND node_id = ?2",
                params![student_id, node_id],
                |row| {
                    Ok(SkillStateRecord {
                        id: row.get(0)?,
                        evidence_count: row.get(1)?,
                        total_attempts: row.get(2)?,
                        correct_attempts: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn find_memory_state(
        &self,
        student_id: i64,
        topic_id: i64,
        node_id: Option<i64>,
    ) -> EcoachResult<Option<MemoryStateRecord>> {
        self.conn
            .query_row(
                "SELECT id, memory_strength, recall_fluency
                 FROM memory_states
                 WHERE student_id = ?1
                   AND topic_id = ?2
                   AND (node_id = ?3 OR (node_id IS NULL AND ?3 IS NULL))
                 LIMIT 1",
                params![student_id, topic_id, node_id],
                |row| {
                    Ok(MemoryStateRecord {
                        id: row.get(0)?,
                        memory_strength: row.get(1)?,
                        recall_fluency: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_topic_summaries(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LearnerTruthTopicSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    sts.topic_id,
                    t.name,
                    sts.mastery_score,
                    sts.mastery_state,
                    sts.gap_score,
                    sts.priority_score,
                    sts.memory_strength,
                    sts.next_review_at
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC, sts.mastery_score ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LearnerTruthTopicSummary {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    mastery_score: row.get(2)?,
                    mastery_state: row.get(3)?,
                    gap_score: row.get(4)?,
                    priority_score: row.get(5)?,
                    memory_strength: row.get(6)?,
                    next_review_at: parse_datetime(row.get::<_, Option<String>>(7)?),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_skill_summaries(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LearnerTruthSkillSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    sss.node_id,
                    an.canonical_title,
                    sss.mastery_score,
                    sss.gap_score,
                    sss.priority_score,
                    sss.state
                 FROM student_skill_states sss
                 INNER JOIN academic_nodes an ON an.id = sss.node_id
                 WHERE sss.student_id = ?1
                 ORDER BY sss.priority_score DESC, sss.gap_score DESC, sss.mastery_score ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LearnerTruthSkillSummary {
                    node_id: row.get(0)?,
                    title: row.get(1)?,
                    mastery_score: row.get(2)?,
                    gap_score: row.get(3)?,
                    priority_score: row.get(4)?,
                    state: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_memory_summaries(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LearnerTruthMemorySummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    ms.topic_id,
                    t.name,
                    ms.node_id,
                    an.canonical_title,
                    ms.memory_state,
                    ms.memory_strength,
                    ms.recall_fluency,
                    ms.decay_risk,
                    ms.review_due_at
                 FROM memory_states ms
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 LEFT JOIN academic_nodes an ON an.id = ms.node_id
                 WHERE ms.student_id = ?1
                 ORDER BY
                    CASE WHEN ms.review_due_at IS NULL THEN 1 ELSE 0 END,
                    ms.review_due_at ASC,
                    ms.decay_risk DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LearnerTruthMemorySummary {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    node_id: row.get(2)?,
                    node_title: row.get(3)?,
                    memory_state: row.get(4)?,
                    memory_strength: row.get(5)?,
                    recall_fluency: row.get(6)?,
                    decay_risk: row.get(7)?,
                    review_due_at: parse_datetime(row.get::<_, Option<String>>(8)?),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_diagnosis_summaries(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LearnerTruthDiagnosisSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    wad.id,
                    wad.topic_id,
                    t.name,
                    wad.primary_diagnosis,
                    wad.severity,
                    wad.recommended_action,
                    wad.created_at
                 FROM wrong_answer_diagnoses wad
                 INNER JOIN topics t ON t.id = wad.topic_id
                 WHERE wad.student_id = ?1
                 ORDER BY wad.created_at DESC, wad.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LearnerTruthDiagnosisSummary {
                    diagnosis_id: row.get(0)?,
                    topic_id: row.get(1)?,
                    topic_name: row.get(2)?,
                    primary_diagnosis: row.get(3)?,
                    severity: row.get(4)?,
                    recommended_action: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_mission_signals(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricSignal>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT cmm.id, cmm.topic_id, t.name, cmm.mission_status, cmm.accuracy_score,
                        cmm.review_status, cmm.next_action_type, cmm.review_due_at, cmm.created_at
                 FROM coach_mission_memories cmm
                 LEFT JOIN topics t ON t.id = cmm.topic_id
                 WHERE cmm.student_id = ?1
                 ORDER BY cmm.created_at DESC, cmm.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let accuracy_score: Option<BasisPoints> = row.get(4)?;
                let review_due_at: Option<String> = row.get(7)?;
                let created_at: String = row.get(8)?;
                Ok(FabricSignal {
                    engine_key: "coach_brain".to_string(),
                    signal_type: "mission_memory".to_string(),
                    status: Some(row.get::<_, String>(3)?),
                    score: accuracy_score,
                    topic_id: row.get(1)?,
                    node_id: None,
                    question_id: None,
                    observed_at: review_due_at.unwrap_or(created_at),
                    payload: json!({
                        "mission_memory_id": row.get::<_, i64>(0)?,
                        "topic_name": row.get::<_, Option<String>>(2)?,
                        "review_status": row.get::<_, String>(5)?,
                        "next_action_type": row.get::<_, String>(6)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_session_signals(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricSignal>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    s.id,
                    s.session_type,
                    s.status,
                    s.accuracy_score,
                    COALESCE(s.completed_at, s.last_activity_at, s.started_at),
                    (
                        SELECT re.payload_json
                        FROM runtime_events re
                        WHERE re.aggregate_kind = 'session'
                          AND re.aggregate_id = CAST(s.id AS TEXT)
                          AND re.event_type = 'session.interpreted'
                        ORDER BY re.occurred_at DESC, re.id DESC
                        LIMIT 1
                    )
                 FROM sessions s
                 WHERE s.student_id = ?1
                   AND s.status = 'completed'
                 ORDER BY COALESCE(s.completed_at, s.last_activity_at, s.started_at) DESC, s.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let payload_json: Option<String> = row.get(5)?;
                let payload = payload_json
                    .as_deref()
                    .map(|value| {
                        serde_json::from_str::<serde_json::Value>(value).map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                5,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })
                    })
                    .transpose()?
                    .unwrap_or_else(|| json!({}));
                let session_id = row.get::<_, i64>(0)?;
                let session_type = row.get::<_, String>(1)?;
                let status = row.get::<_, String>(2)?;
                let accuracy_score = row.get::<_, Option<BasisPoints>>(3)?;
                let observed_at = row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                let next_action_hint = payload
                    .get("next_action_hint")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .or_else(|| Some(status.clone()))
                    .unwrap_or_else(|| "stabilize_and_review".to_string());
                Ok(FabricSignal {
                    engine_key: "session_runtime".to_string(),
                    signal_type: "session_interpretation".to_string(),
                    status: Some(next_action_hint),
                    score: accuracy_score,
                    topic_id: payload
                        .get("topic_summaries")
                        .and_then(|value| value.as_array())
                        .and_then(|items| {
                            if items.len() == 1 {
                                items[0].get("topic_id").and_then(|value| value.as_i64())
                            } else {
                                None
                            }
                        }),
                    node_id: None,
                    question_id: None,
                    observed_at,
                    payload: json!({
                        "session_id": session_id,
                        "session_type": session_type,
                        "status": status,
                        "accuracy_score": accuracy_score,
                        "interpretation": payload,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let signal = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(signal.clone());

            if let Some(topic_summaries) = signal
                .payload
                .get("interpretation")
                .and_then(|value| value.get("topic_summaries"))
                .and_then(|value| value.as_array())
            {
                for topic in topic_summaries {
                    let Some(topic_id) = topic.get("topic_id").and_then(|value| value.as_i64())
                    else {
                        continue;
                    };
                    items.push(FabricSignal {
                        engine_key: "session_runtime".to_string(),
                        signal_type: "session_topic_outcome".to_string(),
                        status: Some(
                            topic
                                .get("dominant_error_type")
                                .and_then(|value| value.as_str())
                                .map(str::to_string)
                                .unwrap_or_else(|| {
                                    if topic
                                        .get("accuracy_score")
                                        .and_then(|value| value.as_u64())
                                        .unwrap_or_default()
                                        >= 7_000
                                    {
                                        "stabilize".to_string()
                                    } else {
                                        "repair".to_string()
                                    }
                                }),
                        ),
                        score: topic
                            .get("accuracy_score")
                            .and_then(|value| value.as_u64())
                            .map(|value| value as BasisPoints),
                        topic_id: Some(topic_id),
                        node_id: None,
                        question_id: None,
                        observed_at: signal.observed_at.clone(),
                        payload: topic.clone(),
                    });
                }
            }
        }
        Ok(items)
    }

    fn list_recent_attempt_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT a.id, a.question_id, q.topic_id, q.primary_skill_id, q.stem, t.name,
                        a.is_correct, a.error_type, a.confidence_level, a.evidence_weight,
                        COALESCE(a.submitted_at, a.created_at)
                 FROM student_question_attempts a
                 INNER JOIN questions q ON q.id = a.question_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 WHERE a.student_id = ?1
                 ORDER BY COALESCE(a.submitted_at, a.created_at) DESC, a.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(FabricEvidenceRecord {
                    stream: "question_attempts".to_string(),
                    reference_id: row.get::<_, i64>(0)?.to_string(),
                    event_type: "answer_submitted".to_string(),
                    topic_id: row.get(2)?,
                    node_id: row.get(3)?,
                    question_id: row.get(1)?,
                    occurred_at: row.get(10)?,
                    payload: json!({
                        "stem": row.get::<_, String>(4)?,
                        "topic_name": row.get::<_, String>(5)?,
                        "is_correct": row.get::<_, i64>(6)? == 1,
                        "error_type": row.get::<_, Option<String>>(7)?,
                        "confidence_level": row.get::<_, Option<String>>(8)?,
                        "evidence_weight": row.get::<_, BasisPoints>(9)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_session_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    s.id,
                    s.session_type,
                    s.status,
                    s.accuracy_score,
                    COALESCE(s.completed_at, s.last_activity_at, s.started_at),
                    (
                        SELECT re.payload_json
                        FROM runtime_events re
                        WHERE re.aggregate_kind = 'session'
                          AND re.aggregate_id = CAST(s.id AS TEXT)
                          AND re.event_type = 'session.interpreted'
                        ORDER BY re.occurred_at DESC, re.id DESC
                        LIMIT 1
                    )
                 FROM sessions s
                 WHERE s.student_id = ?1
                   AND s.status = 'completed'
                 ORDER BY COALESCE(s.completed_at, s.last_activity_at, s.started_at) DESC, s.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let payload_json: Option<String> = row.get(5)?;
                let interpretation = payload_json
                    .as_deref()
                    .map(|value| {
                        serde_json::from_str::<serde_json::Value>(value).map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                5,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })
                    })
                    .transpose()?
                    .unwrap_or_else(|| json!({}));
                let session_type = row.get::<_, String>(1)?;
                let status = row.get::<_, String>(2)?;
                let accuracy_score = row.get::<_, Option<BasisPoints>>(3)?;
                let occurred_at = row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                Ok(FabricEvidenceRecord {
                    stream: "session_outcomes".to_string(),
                    reference_id: row.get::<_, i64>(0)?.to_string(),
                    event_type: "session_completed".to_string(),
                    topic_id: interpretation
                        .get("topic_summaries")
                        .and_then(|value| value.as_array())
                        .and_then(|items| {
                            if items.len() == 1 {
                                items[0].get("topic_id").and_then(|value| value.as_i64())
                            } else {
                                None
                            }
                        }),
                    node_id: None,
                    question_id: None,
                    occurred_at,
                    payload: json!({
                        "session_type": session_type,
                        "status": status,
                        "accuracy_score": accuracy_score,
                        "interpretation": interpretation,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_memory_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT me.id, me.topic_id, me.node_id, t.name, an.canonical_title, me.recall_mode,
                        me.cue_level, me.delay_bucket, me.was_correct, me.evidence_weight, me.created_at
                 FROM memory_evidence_events me
                 LEFT JOIN topics t ON t.id = me.topic_id
                 LEFT JOIN academic_nodes an ON an.id = me.node_id
                 WHERE me.student_id = ?1
                 ORDER BY me.created_at DESC, me.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(FabricEvidenceRecord {
                    stream: "memory_events".to_string(),
                    reference_id: row.get::<_, i64>(0)?.to_string(),
                    event_type: "memory_observed".to_string(),
                    topic_id: row.get(1)?,
                    node_id: row.get(2)?,
                    question_id: None,
                    occurred_at: row.get(10)?,
                    payload: json!({
                        "topic_name": row.get::<_, Option<String>>(3)?,
                        "node_title": row.get::<_, Option<String>>(4)?,
                        "recall_mode": row.get::<_, Option<String>>(5)?,
                        "cue_level": row.get::<_, Option<String>>(6)?,
                        "delay_bucket": row.get::<_, Option<String>>(7)?,
                        "was_correct": row.get::<_, i64>(8)? == 1,
                        "evidence_weight": row.get::<_, BasisPoints>(9)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_diagnosis_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT wad.id, wad.question_id, wad.topic_id, wad.primary_diagnosis, wad.severity,
                        wad.recommended_action, wad.confidence_score, wad.created_at
                 FROM wrong_answer_diagnoses wad
                 WHERE wad.student_id = ?1
                 ORDER BY wad.created_at DESC, wad.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(FabricEvidenceRecord {
                    stream: "wrong_answer_diagnoses".to_string(),
                    reference_id: row.get::<_, i64>(0)?.to_string(),
                    event_type: "diagnosis_recorded".to_string(),
                    topic_id: row.get(2)?,
                    node_id: None,
                    question_id: row.get(1)?,
                    occurred_at: row.get(7)?,
                    payload: json!({
                        "primary_diagnosis": row.get::<_, String>(3)?,
                        "severity": row.get::<_, String>(4)?,
                        "recommended_action": row.get::<_, String>(5)?,
                        "confidence_score": row.get::<_, BasisPoints>(6)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_mission_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, mission_id, topic_id, mission_status, accuracy_score, review_status,
                        next_action_type, summary_json, created_at
                 FROM coach_mission_memories
                 WHERE student_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let summary_json: String = row.get(7)?;
                let summary =
                    serde_json::from_str::<serde_json::Value>(&summary_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                Ok(FabricEvidenceRecord {
                    stream: "coach_mission_memories".to_string(),
                    reference_id: row.get::<_, i64>(0)?.to_string(),
                    event_type: "mission_memory_recorded".to_string(),
                    topic_id: row.get(2)?,
                    node_id: None,
                    question_id: None,
                    occurred_at: row.get(8)?,
                    payload: json!({
                        "mission_id": row.get::<_, i64>(1)?,
                        "mission_status": row.get::<_, String>(3)?,
                        "accuracy_score": row.get::<_, Option<BasisPoints>>(4)?,
                        "review_status": row.get::<_, String>(5)?,
                        "next_action_type": row.get::<_, String>(6)?,
                        "summary": summary,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_recent_runtime_evidence(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FabricEvidenceRecord>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT event_id, event_type, payload_json, occurred_at
                 FROM runtime_events
                 WHERE aggregate_kind = 'learner_truth' AND aggregate_id = ?1
                 ORDER BY occurred_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id.to_string(), limit as i64], |row| {
                let payload_json: String = row.get(2)?;
                let payload =
                    serde_json::from_str::<serde_json::Value>(&payload_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                Ok(FabricEvidenceRecord {
                    stream: "runtime_events".to_string(),
                    reference_id: row.get(0)?,
                    event_type: row.get(1)?,
                    topic_id: payload.get("topic_id").and_then(|value| value.as_i64()),
                    node_id: payload.get("node_id").and_then(|value| value.as_i64()),
                    question_id: payload.get("question_id").and_then(|value| value.as_i64()),
                    occurred_at: row.get(3)?,
                    payload,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
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

struct MemoryStateRecord {
    id: i64,
    memory_strength: BasisPoints,
    recall_fluency: BasisPoints,
}

struct DecayCandidate {
    id: i64,
    student_id: i64,
    topic_id: Option<i64>,
    node_id: Option<i64>,
    memory_state: String,
    memory_strength: BasisPoints,
    decay_risk: BasisPoints,
    review_due_at: Option<DateTime<Utc>>,
}

struct SkillStateRecord {
    id: i64,
    evidence_count: i64,
    total_attempts: i64,
    correct_attempts: i64,
}

struct WrongAnswerDiagnosisRecord {
    diagnosis_summary: String,
    recommended_action: String,
}

pub fn compute_mastery(state: &StudentTopicState) -> BasisPoints {
    let accuracy = from_bp(state.accuracy_score);
    let retention = from_bp(state.retention_score);
    let transfer = from_bp(state.transfer_score);
    let speed = from_bp(state.speed_score);
    let confidence = from_bp(state.confidence_score);
    let consistency = from_bp(state.consistency_score);

    to_bp(
        0.35 * accuracy
            + 0.20 * retention
            + 0.15 * transfer
            + 0.15 * speed
            + 0.10 * confidence
            + 0.05 * consistency,
    )
}

pub fn compute_priority_from_state(state: &StudentTopicState) -> BasisPoints {
    let gap = from_bp(state.gap_score);
    let trend_risk = match state.trend_state.as_str() {
        "critical" => 1.0,
        "declining" => 0.8,
        "fragile" => 0.6,
        "stable" => 0.3,
        _ => 0.2,
    };
    let forgetting_risk = from_bp(state.memory_strength)
        .mul_add(-1.0, 1.0)
        .clamp(0.0, 1.0);
    let fragility = from_bp(state.fragility_score);
    to_bp(0.45 * gap + 0.25 * trend_risk + 0.15 * forgetting_risk + 0.15 * fragility)
}

pub fn resolve_mastery_state(state: &StudentTopicState) -> MasteryState {
    let m = from_bp(state.mastery_score);
    let f = from_bp(state.fragility_score);
    let e = state.evidence_count;

    match state.mastery_state {
        MasteryState::Unseen => {
            if e >= 1 {
                MasteryState::Exposed
            } else {
                MasteryState::Unseen
            }
        }
        MasteryState::Exposed => {
            if e >= 3 && m >= 0.25 {
                MasteryState::Emerging
            } else {
                MasteryState::Exposed
            }
        }
        MasteryState::Emerging => {
            if m >= 0.45 && e >= 8 {
                MasteryState::Partial
            } else if m < 0.15 && e >= 5 {
                MasteryState::Exposed
            } else {
                MasteryState::Emerging
            }
        }
        MasteryState::Partial => {
            if m >= 0.60 && f < 0.30 && e >= 15 {
                MasteryState::Fragile
            } else if m < 0.35 {
                MasteryState::Emerging
            } else {
                MasteryState::Partial
            }
        }
        MasteryState::Fragile => {
            if m >= 0.72 && f < 0.20 && e >= 25 {
                MasteryState::Stable
            } else if m < 0.50 || f > 0.50 {
                MasteryState::Partial
            } else {
                MasteryState::Fragile
            }
        }
        MasteryState::Stable => {
            if m >= 0.82
                && f < 0.15
                && from_bp(state.transfer_score) >= 0.65
                && from_bp(state.retention_score) >= 0.70
            {
                MasteryState::Robust
            } else if m < 0.60 || f > 0.35 {
                MasteryState::Fragile
            } else {
                MasteryState::Stable
            }
        }
        MasteryState::Robust => {
            if m >= 0.90
                && from_bp(state.pressure_collapse_index) < 0.15
                && from_bp(state.retention_score) >= 0.80
            {
                MasteryState::ExamReady
            } else if m < 0.72 || f > 0.25 {
                MasteryState::Stable
            } else {
                MasteryState::Robust
            }
        }
        MasteryState::ExamReady => {
            if m < 0.80 || f > 0.20 {
                MasteryState::Robust
            } else {
                MasteryState::ExamReady
            }
        }
    }
}

pub fn classify_error(
    submission: &AnswerSubmission,
    selected_option: &ecoach_questions::QuestionOption,
    student_state: &StudentTopicState,
) -> ErrorType {
    if selected_option.misconception_id.is_some() {
        return ErrorType::MisconceptionTriggered;
    }

    let guess_likelihood = if submission.confidence_level.as_deref() == Some("guessed") {
        0.8
    } else {
        0.2
    };
    if guess_likelihood > 0.7 {
        return ErrorType::GuessingDetected;
    }

    if submission.was_timed && student_state.accuracy_score > 6000 {
        return ErrorType::PressureBreakdown;
    }

    if student_state.mastery_score > 6500 && submission.hint_count == 0 {
        return ErrorType::Carelessness;
    }

    if submission.was_transfer_variant
        && !submission.was_timed
        && student_state.accuracy_score > 5000
    {
        return ErrorType::RecognitionFailure;
    }

    if student_state.mastery_score < 3000 {
        return ErrorType::KnowledgeGap;
    }

    ErrorType::ConceptualConfusion
}

pub fn compute_evidence_weight(submission: &AnswerSubmission, is_correct: bool) -> BasisPoints {
    let mut weight = 1.0f64;
    if submission.hint_count > 0 {
        weight *= 0.5_f64.powi(submission.hint_count.min(3) as i32);
    }
    match submission.support_level.as_deref() {
        Some("guided") => weight *= 0.7,
        Some("heavily_guided") => weight *= 0.4,
        _ => {}
    }
    if submission.was_transfer_variant {
        weight *= 1.3;
    }
    if submission.was_retention_check {
        weight *= 1.5;
    }
    if is_correct && submission.confidence_level.as_deref() == Some("guessed") {
        weight *= 0.5;
    }
    to_bp((weight.clamp(0.0, 2.0)) / 2.0)
}

fn compute_recall_fluency(response_time_ms: Option<i64>) -> BasisPoints {
    let expected_ms = 30_000f64;
    let actual_ms = response_time_ms.unwrap_or(30_000).max(1) as f64;
    to_bp((expected_ms / actual_ms).clamp(0.0, 1.0))
}

fn compute_decay_risk(
    memory_strength: BasisPoints,
    recall_fluency: BasisPoints,
    is_correct: bool,
) -> BasisPoints {
    let correctness_penalty = if is_correct { 0.0 } else { 0.35 };
    let strength_gap = 1.0 - from_bp(memory_strength);
    let fluency_gap = 1.0 - from_bp(recall_fluency);
    to_bp((0.65 * strength_gap + 0.35 * fluency_gap + correctness_penalty).clamp(0.0, 1.0))
}

fn resolve_memory_state(
    memory_strength: BasisPoints,
    decay_risk: BasisPoints,
    is_correct: bool,
) -> &'static str {
    let strength = from_bp(memory_strength);
    let risk = from_bp(decay_risk);

    if !is_correct && strength < 0.20 {
        "collapsed"
    } else if !is_correct {
        "rebuilding"
    } else if strength >= 0.85 && risk < 0.15 {
        "locked_in"
    } else if strength >= 0.70 && risk < 0.25 {
        "confirmed"
    } else if strength >= 0.55 {
        "anchoring"
    } else if strength >= 0.40 {
        "accessible"
    } else if strength >= 0.25 {
        "encoded"
    } else {
        "seen"
    }
}

fn compute_review_due_at(
    now: DateTime<Utc>,
    memory_strength: BasisPoints,
    recall_fluency: BasisPoints,
    is_correct: bool,
    confidence_level: Option<&str>,
) -> DateTime<Utc> {
    let interval = if !is_correct {
        Duration::hours(12)
    } else if confidence_level == Some("guessed") {
        Duration::days(1)
    } else if from_bp(memory_strength) >= 0.80 && from_bp(recall_fluency) >= 0.70 {
        Duration::days(7)
    } else if from_bp(memory_strength) >= 0.60 {
        Duration::days(4)
    } else {
        Duration::days(2)
    };

    now + interval
}

fn resolve_skill_state(mastery_score: BasisPoints) -> &'static str {
    match mastery_score {
        0..=1999 => "unseen",
        2000..=4499 => "emerging",
        4500..=6999 => "functional",
        7000..=8499 => "stable",
        _ => "exam_ready",
    }
}

fn learner_truth_readiness_band(score: BasisPoints) -> &'static str {
    match score {
        0..=2499 => "critical",
        2500..=4499 => "fragile",
        4500..=6499 => "developing",
        6500..=7999 => "progressing",
        8000..=8999 => "near_ready",
        _ => "exam_ready",
    }
}

fn severity_basis_points(severity: &str) -> BasisPoints {
    match severity {
        "high" => 9000,
        "medium" => 6500,
        "low" => 3500,
        _ => 5000,
    }
}

fn resolve_decay_memory_state(
    memory_strength: BasisPoints,
    decay_risk: BasisPoints,
    overdue_days: i64,
) -> &'static str {
    if overdue_days >= 5 || (memory_strength <= 1200 && decay_risk >= 8500) {
        "collapsed"
    } else if overdue_days >= 3 || decay_risk >= 7500 {
        "fading"
    } else if overdue_days >= 1 || decay_risk >= 6000 {
        "at_risk"
    } else if memory_strength >= 7000 && decay_risk <= 2500 {
        "confirmed"
    } else {
        "accessible"
    }
}

fn primary_diagnosis_label(error_type: ErrorType) -> &'static str {
    match error_type {
        ErrorType::KnowledgeGap => "missing_foundation",
        ErrorType::ConceptualConfusion => "concept_misread",
        ErrorType::RecognitionFailure => "pattern_mismatch",
        ErrorType::ExecutionError => "execution_break",
        ErrorType::Carelessness => "accuracy_slip",
        ErrorType::PressureBreakdown => "timed_pressure_drop",
        ErrorType::ExpressionWeakness => "expression_gap",
        ErrorType::SpeedError => "slow_retrieval",
        ErrorType::GuessingDetected => "guess_without_anchor",
        ErrorType::MisconceptionTriggered => "misconception_trigger",
    }
}

fn secondary_diagnosis_label(
    submission: &AnswerSubmission,
    topic_state: &StudentTopicState,
) -> Option<&'static str> {
    if submission.was_timed {
        Some("timed_condition")
    } else if submission.was_transfer_variant {
        Some("transfer_condition")
    } else if submission.confidence_level.as_deref() == Some("sure")
        && topic_state.mastery_score < 5000
    {
        Some("overconfidence_signal")
    } else {
        None
    }
}

fn diagnosis_severity(
    error_type: ErrorType,
    submission: &AnswerSubmission,
    topic_state: &StudentTopicState,
) -> &'static str {
    if matches!(
        error_type,
        ErrorType::MisconceptionTriggered | ErrorType::PressureBreakdown
    ) {
        "high"
    } else if submission.confidence_level.as_deref() == Some("sure")
        || topic_state.mastery_score < 3500
    {
        "high"
    } else if topic_state.mastery_score < 5500 {
        "medium"
    } else {
        "low"
    }
}

fn recommended_action_label(error_type: ErrorType) -> &'static str {
    match error_type {
        ErrorType::KnowledgeGap => "teach_then_guided_practice",
        ErrorType::ConceptualConfusion => "contrast_review_then_check",
        ErrorType::RecognitionFailure => "pattern_rebuild_pack",
        ErrorType::ExecutionError => "worked_example_then_retry",
        ErrorType::Carelessness => "slow_check_checkpoint",
        ErrorType::PressureBreakdown => "timed_repair_checkpoint",
        ErrorType::ExpressionWeakness => "structured_response_repair",
        ErrorType::SpeedError => "fluency_ladder",
        ErrorType::GuessingDetected => "confidence_repair_drill",
        ErrorType::MisconceptionTriggered => "misconception_repair_pack",
    }
}

fn diagnosis_confidence_score(error_type: ErrorType, misconception_id: Option<i64>) -> BasisPoints {
    match error_type {
        ErrorType::MisconceptionTriggered if misconception_id.is_some() => 9000,
        ErrorType::GuessingDetected => 7800,
        ErrorType::PressureBreakdown => 7600,
        ErrorType::KnowledgeGap => 7200,
        _ => 6800,
    }
}

fn diagnosis_summary_text(
    error_type: ErrorType,
    topic_id: i64,
    _submission: &AnswerSubmission,
    _topic_state: &StudentTopicState,
    misconception_id: Option<i64>,
) -> String {
    match error_type {
        ErrorType::KnowledgeGap => format!(
            "The learner likely lacks enough foundation on topic {} to solve this independently.",
            topic_id
        ),
        ErrorType::ConceptualConfusion => format!(
            "The learner appears to mix up the core meaning in topic {} and needs a contrast-style repair.",
            topic_id
        ),
        ErrorType::RecognitionFailure => format!(
            "The learner knows some of topic {} but did not recognize the pattern in this variant.",
            topic_id
        ),
        ErrorType::ExecutionError => format!(
            "The learner likely knew the path in topic {} but broke down during execution.",
            topic_id
        ),
        ErrorType::Carelessness => format!(
            "The learner’s knowledge on topic {} is stronger than this answer suggests; accuracy controls likely slipped.",
            topic_id
        ),
        ErrorType::PressureBreakdown => format!(
            "Timed pressure seems to have lowered performance on topic {} despite some existing mastery.",
            topic_id
        ),
        ErrorType::ExpressionWeakness => format!(
            "The learner likely needs help expressing the correct idea clearly for topic {}.",
            topic_id
        ),
        ErrorType::SpeedError => format!(
            "Retrieval speed on topic {} is too slow for fluent performance right now.",
            topic_id
        ),
        ErrorType::GuessingDetected => format!(
            "The response on topic {} looks weakly anchored and likely came from guessing rather than reliable recall.",
            topic_id
        ),
        ErrorType::MisconceptionTriggered => {
            if misconception_id.is_some() {
                format!(
                    "A known misconception was triggered on topic {}, so this needs direct misconception repair.",
                    topic_id
                )
            } else {
                format!(
                    "The answer pattern on topic {} looks misconception-driven and needs targeted repair.",
                    topic_id
                )
            }
        }
    }
}

fn compute_trend_label(state: &StudentTopicState) -> String {
    let accuracy = from_bp(state.accuracy_score);
    let gap = from_bp(state.gap_score);
    if gap > 0.75 {
        "critical".to_string()
    } else if accuracy < 0.35 {
        "declining".to_string()
    } else if accuracy < 0.55 {
        "fragile".to_string()
    } else if accuracy > 0.75 {
        "improving".to_string()
    } else {
        "stable".to_string()
    }
}

fn parse_mastery_state(value: String) -> rusqlite::Result<MasteryState> {
    let state = match value.as_str() {
        "unseen" => MasteryState::Unseen,
        "exposed" => MasteryState::Exposed,
        "emerging" => MasteryState::Emerging,
        "partial" => MasteryState::Partial,
        "fragile" => MasteryState::Fragile,
        "stable" => MasteryState::Stable,
        "robust" => MasteryState::Robust,
        "exam_ready" => MasteryState::ExamReady,
        other => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(EcoachError::Serialization(format!(
                    "unknown mastery state: {}",
                    other
                ))),
            ));
        }
    };
    Ok(state)
}

fn parse_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
    value
        .and_then(|raw| DateTime::<Utc>::from_str(&raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn learner_fabric_inputs(
    signals: &[FabricSignal],
    evidence_records: &[FabricEvidenceRecord],
) -> Vec<String> {
    let mut inputs = BTreeSet::new();
    inputs.insert("learner_evidence_fabric".to_string());

    for signal in signals {
        match signal.signal_type.as_str() {
            "topic_truth" => {
                inputs.insert("topic_truth".to_string());
            }
            "skill_truth" => {
                inputs.insert("skill_truth".to_string());
            }
            "memory_truth" => {
                inputs.insert("memory_truth".to_string());
            }
            "diagnosis_claim" => {
                inputs.insert("diagnosis_claims".to_string());
            }
            "mission_memory" => {
                inputs.insert("mission_memory".to_string());
            }
            "session_interpretation" | "session_topic_outcome" => {
                inputs.insert("session_outcomes".to_string());
            }
            _ => {}
        }
    }

    for record in evidence_records {
        match record.stream.as_str() {
            "question_attempts" => {
                inputs.insert("answer_submissions".to_string());
            }
            "memory_events" => {
                inputs.insert("memory_evidence".to_string());
            }
            "session_outcomes" => {
                inputs.insert("session_outcomes".to_string());
                inputs.insert("session_evidence".to_string());
                inputs.insert("mission_memory_inputs".to_string());
            }
            "wrong_answer_diagnoses" => {
                inputs.insert("diagnosis_claims".to_string());
            }
            "coach_mission_memories" => {
                inputs.insert("mission_memory".to_string());
            }
            _ => {}
        }
    }

    inputs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{Duration, Utc};
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};
    use serde_json::json;

    use super::*;

    #[test]
    fn mastery_formula_weights_dimensions() {
        let state = StudentTopicState {
            id: 1,
            student_id: 1,
            topic_id: 1,
            mastery_score: 0,
            mastery_state: MasteryState::Partial,
            accuracy_score: 8000,
            speed_score: 6000,
            confidence_score: 7000,
            retention_score: 5000,
            transfer_score: 5500,
            consistency_score: 6500,
            gap_score: 0,
            priority_score: 0,
            trend_state: "stable".to_string(),
            fragility_score: 1500,
            pressure_collapse_index: 500,
            total_attempts: 10,
            correct_attempts: 7,
            evidence_count: 10,
            last_seen_at: None,
            last_correct_at: None,
            memory_strength: 5000,
            next_review_at: None,
            version: 1,
        };

        let mastery = compute_mastery(&state);
        assert!(mastery > 6000);
        assert!(mastery < 8000);
    }

    #[test]
    fn mastery_state_promotes_when_evidence_is_strong() {
        let state = StudentTopicState {
            id: 1,
            student_id: 1,
            topic_id: 1,
            mastery_score: 9100,
            mastery_state: MasteryState::Robust,
            accuracy_score: 9200,
            speed_score: 8800,
            confidence_score: 8500,
            retention_score: 8300,
            transfer_score: 8100,
            consistency_score: 8700,
            gap_score: 900,
            priority_score: 2000,
            trend_state: "improving".to_string(),
            fragility_score: 500,
            pressure_collapse_index: 500,
            total_attempts: 40,
            correct_attempts: 36,
            evidence_count: 40,
            last_seen_at: None,
            last_correct_at: None,
            memory_strength: 8200,
            next_review_at: None,
            version: 1,
        };

        assert_eq!(resolve_mastery_state(&state), MasteryState::ExamReady);
    }

    #[test]
    fn learner_truth_readiness_band_matches_score_bands() {
        assert_eq!(learner_truth_readiness_band(1500), "critical");
        assert_eq!(learner_truth_readiness_band(4200), "fragile");
        assert_eq!(learner_truth_readiness_band(6100), "developing");
        assert_eq!(learner_truth_readiness_band(7600), "progressing");
        assert_eq!(learner_truth_readiness_band(8600), "near_ready");
        assert_eq!(learner_truth_readiness_band(9300), "exam_ready");
    }

    #[test]
    fn learner_evidence_fabric_collects_core_truth_streams() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        let (subject_id, topic_id): (i64, i64) = conn
            .query_row(
                "SELECT subject_id, topic_id FROM questions WHERE id = ?1",
                [question_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("question scope should resolve");
        let wrong_option_id: i64 = conn
            .query_row(
                "SELECT id FROM question_options
                 WHERE question_id = ?1 AND is_correct = 0
                 ORDER BY id ASC
                 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .expect("wrong option should exist");
        conn.execute(
            "INSERT INTO sessions (
                id, student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, last_activity_at, answered_questions, correct_questions, accuracy_score
             ) VALUES (?1, ?2, 'practice', ?3, ?4, 1, 1, 1, 'completed', ?5, ?5, 1, 0, 0)",
            params![
                11_i64,
                1_i64,
                subject_id,
                serde_json::to_string(&vec![topic_id]).expect("topic ids json"),
                Utc::now().to_rfc3339(),
            ],
        )
        .expect("session should seed");
        conn.execute(
            "INSERT INTO runtime_events (
                event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
             ) VALUES (?1, 'session.interpreted', 'session', '11', ?2, ?3, ?4)",
            params![
                "session-interpreted-1",
                "trace-session-1",
                json!({
                    "session_id": 11,
                    "student_id": 1,
                    "session_type": "practice",
                    "status": "completed",
                    "next_action_hint": "repair_required",
                    "interpretation_tags": ["timed_fragility", "misconception_pressure"],
                    "topic_summaries": [{
                        "topic_id": topic_id,
                        "topic_name": "Fractions",
                        "attempts": 1,
                        "correct_attempts": 0,
                        "accuracy_score": 0,
                        "avg_response_time_ms": 18000,
                        "dominant_error_type": "misconception_triggered"
                    }]
                })
                .to_string(),
                Utc::now().to_rfc3339(),
            ],
        )
        .expect("session interpretation event should seed");

        let service = StudentModelService::new(&conn);
        service
            .process_answer(
                1,
                &AnswerSubmission {
                    question_id,
                    selected_option_id: wrong_option_id,
                    session_id: Some(11),
                    session_type: Some("practice".to_string()),
                    started_at: Utc::now() - Duration::seconds(30),
                    submitted_at: Utc::now(),
                    response_time_ms: Some(18_000),
                    confidence_level: Some("sure".to_string()),
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
            .expect("answer should be processed");

        let fabric = service
            .get_learner_evidence_fabric(1, 4)
            .expect("fabric should build");

        assert_eq!(fabric.student_id, 1);
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "topic_truth")
        );
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "skill_truth")
        );
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "memory_truth")
        );
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "diagnosis_claim")
        );
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "session_interpretation")
        );
        assert!(
            fabric
                .signals
                .iter()
                .any(|signal| signal.signal_type == "session_topic_outcome")
        );
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.stream == "question_attempts")
        );
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.stream == "memory_events")
        );
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.stream == "wrong_answer_diagnoses")
        );
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.stream == "runtime_events")
        );
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.stream == "session_outcomes")
        );
        assert!(
            fabric
                .orchestration
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "coach_brain")
        );
        assert!(
            fabric
                .orchestration
                .available_inputs
                .iter()
                .any(|input| input == "session_outcomes")
        );
    }

    #[test]
    fn memory_decay_scan_marks_overdue_memory_and_due_rechecks() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        let correct_option_id: i64 = conn
            .query_row(
                "SELECT id FROM question_options
                 WHERE question_id = ?1 AND is_correct = 1
                 ORDER BY id ASC
                 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .expect("correct option should exist");

        let service = StudentModelService::new(&conn);
        service
            .process_answer(
                1,
                &AnswerSubmission {
                    question_id,
                    selected_option_id: correct_option_id,
                    session_id: None,
                    session_type: Some("practice".to_string()),
                    started_at: Utc::now() - Duration::seconds(20),
                    submitted_at: Utc::now(),
                    response_time_ms: Some(12_000),
                    confidence_level: Some("sure".to_string()),
                    hint_count: 0,
                    changed_answer_count: 0,
                    skipped: false,
                    timed_out: false,
                    support_level: Some("independent".to_string()),
                    was_timed: false,
                    was_transfer_variant: false,
                    was_retention_check: true,
                    was_mixed_context: false,
                },
            )
            .expect("answer should be processed");

        let two_days_ago = (Utc::now() - Duration::days(2)).to_rfc3339();
        conn.execute(
            "UPDATE memory_states
             SET review_due_at = ?1, memory_state = 'confirmed', memory_strength = 8200, decay_risk = 1800",
            [two_days_ago.as_str()],
        )
        .expect("memory state should update");
        conn.execute(
            "UPDATE recheck_schedules
             SET due_at = ?1, status = 'pending'",
            [two_days_ago.as_str()],
        )
        .expect("recheck schedule should update");

        let due_items = service
            .list_due_rechecks(1, Utc::now(), 5)
            .expect("due rechecks should list");
        assert!(!due_items.is_empty());

        let decay_updates = service
            .run_memory_decay_scan(Some(1), Utc::now(), 5)
            .expect("decay scan should succeed");
        assert!(!decay_updates.is_empty());
        assert!(decay_updates[0].overdue_days >= 1);
        assert_ne!(decay_updates[0].previous_state, decay_updates[0].next_state);

        let missed_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM recheck_schedules WHERE student_id = 1 AND status = 'missed'",
                [],
                |row| row.get(0),
            )
            .expect("missed count should query");
        assert!(missed_count >= 1);
    }

    #[test]
    fn process_answer_recomputes_topic_truth_and_syncs_active_gap_plan_priority() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT topic_id FROM questions WHERE id = ?1",
                [question_id],
                |row| row.get(0),
            )
            .expect("topic should resolve");
        let correct_option_id: i64 = conn
            .query_row(
                "SELECT id FROM question_options
                 WHERE question_id = ?1 AND is_correct = 1
                 ORDER BY id ASC
                 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .expect("correct option should exist");

        conn.execute(
            "INSERT INTO gap_repair_plans (id, student_id, topic_id, status, priority_score)
             VALUES (77, 1, ?1, 'active', 0)",
            [topic_id],
        )
        .expect("active gap plan should seed");

        let service = StudentModelService::new(&conn);
        service
            .process_answer(
                1,
                &AnswerSubmission {
                    question_id,
                    selected_option_id: correct_option_id,
                    session_id: None,
                    session_type: Some("practice".to_string()),
                    started_at: Utc::now() - Duration::seconds(15),
                    submitted_at: Utc::now(),
                    response_time_ms: Some(9_000),
                    confidence_level: Some("sure".to_string()),
                    hint_count: 0,
                    changed_answer_count: 0,
                    skipped: false,
                    timed_out: false,
                    support_level: Some("independent".to_string()),
                    was_timed: false,
                    was_transfer_variant: false,
                    was_retention_check: true,
                    was_mixed_context: false,
                },
            )
            .expect("answer should be processed");

        let (memory_strength, next_review_at, fragility_score, repair_priority): (
            i64,
            Option<String>,
            i64,
            i64,
        ) = conn
            .query_row(
                "SELECT memory_strength, next_review_at, fragility_score, repair_priority
                 FROM student_topic_states
                 WHERE student_id = 1 AND topic_id = ?1",
                [topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("topic truth should query");
        assert!(memory_strength > 0);
        assert!(next_review_at.is_some());
        assert!(fragility_score >= 0);
        assert!(repair_priority > 0);

        let plan_priority: i64 = conn
            .query_row(
                "SELECT priority_score FROM gap_repair_plans WHERE id = 77",
                [],
                |row| row.get(0),
            )
            .expect("gap plan priority should query");
        assert_eq!(plan_priority, repair_priority);

        let recompute_events: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events
                 WHERE event_type = 'learner_truth.topic_recomputed'",
                [],
                |row| row.get(0),
            )
            .expect("recompute event count should query");
        assert!(recompute_events >= 1);
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ada Student', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates directory should exist")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}
