use ecoach_questions::{
    QuestionSelectionRequest, QuestionSelector, QuestionService, SelectedQuestion,
};
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value, json};

use crate::models::{
    DiagnosticBattery, DiagnosticCauseEvolution, DiagnosticLongitudinalSummary, DiagnosticMode,
    DiagnosticPhaseCode, DiagnosticPhaseItem, DiagnosticPhasePlan, DiagnosticResult,
    DiagnosticRootCauseHypothesis, DiagnosticTopicAnalytics, TopicDiagnosticLongitudinalSignal,
    TopicDiagnosticResult, WrongAnswerDiagnosis,
};

pub struct DiagnosticEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Copy)]
struct DiagnosticPhaseTemplate {
    code: DiagnosticPhaseCode,
    question_count: usize,
    time_limit_seconds: Option<i64>,
    timed: bool,
    evidence_weight: i64,
}

#[derive(Debug, Clone)]
struct PreviousDiagnosticContext {
    diagnostic_id: i64,
    completed_at: Option<String>,
    overall_readiness: Option<BasisPoints>,
}

#[derive(Debug, Clone)]
struct HistoricalTopicSnapshot {
    diagnostic_id: i64,
    topic_id: i64,
    topic_name: String,
    mastery_score: BasisPoints,
    pressure_score: BasisPoints,
    flexibility_score: BasisPoints,
    classification: String,
    top_hypothesis_code: Option<String>,
    top_hypothesis_confidence: Option<BasisPoints>,
}

impl<'a> DiagnosticEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn start_diagnostic(
        &self,
        student_id: i64,
        subject_id: i64,
        mode: DiagnosticMode,
    ) -> EcoachResult<i64> {
        let topic_ids = self.load_subject_topic_ids(subject_id)?;
        let battery = self.start_diagnostic_battery(student_id, subject_id, topic_ids, mode)?;
        Ok(battery.diagnostic_id)
    }

    pub fn start_diagnostic_battery(
        &self,
        student_id: i64,
        subject_id: i64,
        mut topic_ids: Vec<i64>,
        mode: DiagnosticMode,
    ) -> EcoachResult<DiagnosticBattery> {
        if topic_ids.is_empty() {
            topic_ids = self.load_subject_topic_ids(subject_id)?;
        }

        self.conn
            .execute(
                "INSERT INTO diagnostic_instances (student_id, subject_id, session_mode, status, started_at)
                 VALUES (?1, ?2, ?3, 'phase_1', datetime('now'))",
                params![student_id, subject_id, mode.as_str()],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let diagnostic_id = self.conn.last_insert_rowid();
        let phase_templates = diagnostic_phase_templates(mode);
        let root_cause_topic_ids =
            self.load_root_cause_topic_ids(student_id, subject_id, &topic_ids)?;
        let mut recently_seen_question_ids = Vec::new();

        for (index, template) in phase_templates.iter().enumerate() {
            let phase_topic_ids = if matches!(template.code, DiagnosticPhaseCode::RootCause)
                && !root_cause_topic_ids.is_empty()
            {
                root_cause_topic_ids.clone()
            } else {
                topic_ids.clone()
            };

            let selected_questions = self.select_phase_questions(
                subject_id,
                &phase_topic_ids,
                template,
                &recently_seen_question_ids,
            )?;

            recently_seen_question_ids.extend(
                selected_questions
                    .iter()
                    .map(|selected| selected.question.id),
            );

            let phase_id = self.insert_phase_row(
                diagnostic_id,
                (index + 1) as i64,
                template,
                selected_questions.len() as i64,
                index == 0,
            )?;
            self.insert_phase_items(diagnostic_id, phase_id, template, &selected_questions)?;
        }

        self.get_diagnostic_battery(diagnostic_id)
    }

    pub fn get_diagnostic_battery(&self, diagnostic_id: i64) -> EcoachResult<DiagnosticBattery> {
        let (student_id, subject_id, session_mode, status): (i64, i64, String, String) = self
            .conn
            .query_row(
                "SELECT student_id, subject_id, session_mode, status
                 FROM diagnostic_instances
                 WHERE id = ?1",
                [diagnostic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(DiagnosticBattery {
            diagnostic_id,
            student_id,
            subject_id,
            session_mode,
            status,
            phases: self.list_diagnostic_phases(diagnostic_id)?,
        })
    }

    pub fn get_phase_one_items(
        &self,
        subject_id: i64,
        topic_ids: Vec<i64>,
        count: usize,
    ) -> EcoachResult<Vec<SelectedQuestion>> {
        let selector = QuestionSelector::new(self.conn);
        selector.select_questions(&QuestionSelectionRequest {
            subject_id,
            topic_ids: topic_ids.clone(),
            target_question_count: count,
            target_difficulty: None,
            weakness_topic_ids: topic_ids,
            recently_seen_question_ids: Vec::new(),
            timed: false,
        })
    }

    pub fn list_phase_items(
        &self,
        diagnostic_id: i64,
        phase_number: i64,
    ) -> EcoachResult<Vec<DiagnosticPhaseItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    dia.id,
                    dia.phase_id,
                    dia.question_id,
                    dia.display_order,
                    dia.condition_type,
                    q.stem,
                    q.question_format,
                    q.topic_id
                 FROM diagnostic_item_attempts dia
                 INNER JOIN diagnostic_session_phases dsp ON dsp.id = dia.phase_id
                 INNER JOIN questions q ON q.id = dia.question_id
                 WHERE dia.diagnostic_id = ?1 AND dsp.phase_number = ?2
                 ORDER BY dia.display_order ASC, dia.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![diagnostic_id, phase_number], |row| {
                Ok(DiagnosticPhaseItem {
                    attempt_id: row.get(0)?,
                    phase_id: row.get(1)?,
                    question_id: row.get(2)?,
                    display_order: row.get(3)?,
                    condition_type: row.get(4)?,
                    stem: row.get(5)?,
                    question_format: row.get(6)?,
                    topic_id: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn submit_phase_attempt(
        &self,
        diagnostic_id: i64,
        attempt_id: i64,
        selected_option_id: i64,
        response_time_ms: Option<i64>,
        confidence_level: Option<&str>,
        changed_answer_count: i64,
        skipped: bool,
        timed_out: bool,
    ) -> EcoachResult<()> {
        let question_id: i64 = self
            .conn
            .query_row(
                "SELECT question_id FROM diagnostic_item_attempts
                 WHERE id = ?1 AND diagnostic_id = ?2",
                params![attempt_id, diagnostic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let question_service = QuestionService::new(self.conn);
        let option = question_service
            .get_option(selected_option_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "diagnostic option {} not found",
                    selected_option_id
                ))
            })?;
        if option.question_id != question_id {
            return Err(EcoachError::Validation(format!(
                "option {} does not belong to diagnostic question {}",
                selected_option_id, question_id
            )));
        }

        self.conn
            .execute(
                "UPDATE diagnostic_item_attempts
                 SET submitted_at = datetime('now'),
                     response_time_ms = ?1,
                     selected_option_id = ?2,
                     is_correct = ?3,
                     confidence_level = ?4,
                     changed_answer_count = ?5,
                     skipped = ?6,
                     timed_out = ?7
                 WHERE id = ?8 AND diagnostic_id = ?9",
                params![
                    response_time_ms,
                    selected_option_id,
                    if option.is_correct { 1 } else { 0 },
                    confidence_level,
                    changed_answer_count,
                    if skipped { 1 } else { 0 },
                    if timed_out { 1 } else { 0 },
                    attempt_id,
                    diagnostic_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn advance_phase(
        &self,
        diagnostic_id: i64,
        completed_phase_number: i64,
    ) -> EcoachResult<Option<DiagnosticPhasePlan>> {
        let completed_rows = self
            .conn
            .execute(
                "UPDATE diagnostic_session_phases
                 SET status = 'completed', completed_at = datetime('now')
                 WHERE diagnostic_id = ?1 AND phase_number = ?2",
                params![diagnostic_id, completed_phase_number],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if completed_rows == 0 {
            return Err(EcoachError::Validation(format!(
                "diagnostic {} has no phase {} to advance",
                diagnostic_id, completed_phase_number
            )));
        }

        let next_phase = self
            .conn
            .query_row(
                "SELECT id, phase_number, COALESCE(phase_code, lower(replace(phase_type, ' ', '_'))),
                        question_count, time_limit_seconds,
                        COALESCE(json_extract(condition_profile_json, '$.timed'), 0)
                 FROM diagnostic_session_phases
                 WHERE diagnostic_id = ?1 AND phase_number > ?2 AND status = 'pending'
                 ORDER BY phase_number ASC
                 LIMIT 1",
                params![diagnostic_id, completed_phase_number],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, i64>(5)? == 1,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((
            phase_id,
            phase_number,
            phase_code,
            question_count,
            time_limit_seconds,
            timed,
        )) = next_phase
        {
            self.retarget_pending_phase(
                diagnostic_id,
                phase_id,
                &phase_code,
                question_count,
                time_limit_seconds,
                timed,
            )?;
            self.conn
                .execute(
                    "UPDATE diagnostic_session_phases
                     SET status = 'active',
                         started_at = COALESCE(started_at, datetime('now'))
                     WHERE id = ?1",
                    [phase_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn
                .execute(
                    "UPDATE diagnostic_instances SET status = ?1 WHERE id = ?2",
                    params![format!("phase_{}", phase_number), diagnostic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let phases = self.list_diagnostic_phases(diagnostic_id)?;
            return Ok(phases.into_iter().find(|phase| phase.phase_id == phase_id));
        }

        self.conn
            .execute(
                "UPDATE diagnostic_instances
                 SET status = 'completed', completed_at = datetime('now')
                 WHERE id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(None)
    }

    pub fn complete_diagnostic(&self, diagnostic_id: i64) -> EcoachResult<DiagnosticResult> {
        let (student_id, subject_id): (i64, i64) = self
            .conn
            .query_row(
                "SELECT student_id, subject_id FROM diagnostic_instances WHERE id = ?1",
                [diagnostic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT t.id, t.name
                 FROM questions q
                 JOIN topics t ON t.id = q.topic_id
                 WHERE q.subject_id = ?1 AND q.is_active = 1
                 ORDER BY t.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([subject_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let previous_context =
            self.load_previous_completed_diagnostic_context(student_id, subject_id, diagnostic_id)?;
        let mut topic_results = Vec::new();
        let mut recommended_next_actions = Vec::<String>::new();
        for row in rows {
            let (topic_id, topic_name) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let analytics =
                self.compute_topic_analytics(diagnostic_id, student_id, topic_id, &topic_name)?;
            self.persist_topic_analytics(&analytics)?;
            let previous_topic = if let Some(context) = previous_context.as_ref() {
                self.load_historical_topic_snapshot(context.diagnostic_id, topic_id)?
            } else {
                None
            };
            let hypotheses = self.build_root_cause_hypotheses(
                diagnostic_id,
                student_id,
                topic_id,
                &topic_name,
                &analytics,
            )?;
            let longitudinal_signal = build_topic_longitudinal_signal(
                previous_context.as_ref(),
                previous_topic.as_ref(),
                &analytics,
                &hypotheses,
            );
            let hypotheses =
                enrich_hypotheses_with_longitudinal_context(hypotheses, &analytics, longitudinal_signal.as_ref());
            self.persist_root_cause_hypotheses(diagnostic_id, topic_id, &hypotheses)?;
            recommended_next_actions.push(analytics.recommended_action.clone());
            for hypothesis in &hypotheses {
                recommended_next_actions.push(hypothesis.recommended_action.clone());
            }
            topic_results.push(TopicDiagnosticResult {
                topic_id,
                topic_name,
                mastery_score: analytics.mastery_score,
                fluency_score: analytics.fluency_score,
                precision_score: analytics.precision_score,
                pressure_score: analytics.pressure_score,
                flexibility_score: analytics.flexibility_score,
                stability_score: analytics.stability_score,
                classification: analytics.classification,
                longitudinal_signal,
            });
        }

        let overall_readiness = if topic_results.is_empty() {
            0
        } else {
            (topic_results
                .iter()
                .map(|result| result.mastery_score as i64)
                .sum::<i64>()
                / topic_results.len() as i64) as BasisPoints
        };

        let readiness_band = match overall_readiness {
            8500..=10000 => "Exam Ready",
            7000..=8499 => "Strong",
            5500..=6999 => "Building",
            4000..=5499 => "At Risk",
            _ => "Not Ready",
        }
        .to_string();

        let longitudinal_summary =
            build_longitudinal_summary(previous_context.as_ref(), overall_readiness, &topic_results);
        let result = DiagnosticResult {
            overall_readiness,
            readiness_band,
            topic_results,
            recommended_next_actions: unique_actions(recommended_next_actions),
            longitudinal_summary,
        };

        let serialized = serde_json::to_string(&result)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE diagnostic_instances SET status = 'completed', completed_at = datetime('now'), result_json = ?1 WHERE id = ?2",
                params![serialized, diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(summary) = result.longitudinal_summary.as_ref() {
            self.append_runtime_event(DomainEvent::new(
                "diagnostic.longitudinal_compared",
                diagnostic_id.to_string(),
                json!({
                    "diagnostic_id": diagnostic_id,
                    "student_id": student_id,
                    "subject_id": subject_id,
                    "previous_diagnostic_id": summary.previous_diagnostic_id,
                    "overall_readiness_delta": summary.overall_readiness_delta,
                    "trend": summary.trend,
                    "improved_topic_count": summary.improved_topic_count,
                    "declined_topic_count": summary.declined_topic_count,
                    "persistent_cause_count": summary.persistent_cause_count,
                    "shifted_cause_count": summary.shifted_cause_count,
                    "new_cause_count": summary.new_cause_count,
                }),
            ))?;
        }

        Ok(result)
    }

    pub fn get_diagnostic_result(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Option<DiagnosticResult>> {
        let result_json = self
            .conn
            .query_row(
                "SELECT result_json
                 FROM diagnostic_instances
                 WHERE id = ?1",
                [diagnostic_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten();
        let Some(result_json) = result_json else {
            return Ok(None);
        };
        let result = serde_json::from_str::<DiagnosticResult>(&result_json)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        Ok(Some(result))
    }

    pub fn get_longitudinal_summary(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Option<DiagnosticLongitudinalSummary>> {
        Ok(self
            .get_diagnostic_result(diagnostic_id)?
            .and_then(|result| result.longitudinal_summary))
    }

    pub fn list_cause_evolution(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticCauseEvolution>> {
        Ok(self
            .get_longitudinal_summary(diagnostic_id)?
            .map(|summary| summary.cause_evolution)
            .unwrap_or_default())
    }

    pub fn list_topic_analytics(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticTopicAnalytics>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT dta.diagnostic_id, dta.topic_id, t.name, dta.mastery_score, dta.fluency_score,
                        dta.precision_score, dta.pressure_score, dta.flexibility_score,
                        dta.stability_score, dta.classification, dta.confidence_score,
                        dta.recommended_action
                 FROM diagnostic_topic_analytics dta
                 INNER JOIN topics t ON t.id = dta.topic_id
                 WHERE dta.diagnostic_id = ?1
                 ORDER BY dta.confidence_score DESC, t.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                Ok(DiagnosticTopicAnalytics {
                    diagnostic_id: row.get(0)?,
                    topic_id: row.get(1)?,
                    topic_name: row.get(2)?,
                    mastery_score: row.get(3)?,
                    fluency_score: row.get(4)?,
                    precision_score: row.get(5)?,
                    pressure_score: row.get(6)?,
                    flexibility_score: row.get(7)?,
                    stability_score: row.get(8)?,
                    classification: row.get(9)?,
                    confidence_score: row.get(10)?,
                    recommended_action: row.get(11)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn list_root_cause_hypotheses(
        &self,
        diagnostic_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<DiagnosticRootCauseHypothesis>> {
        let sql = if topic_id.is_some() {
            "SELECT drch.id, drch.diagnostic_id, drch.topic_id, t.name, drch.hypothesis_code,
                    drch.confidence_score, drch.recommended_action, drch.evidence_json, drch.created_at
             FROM diagnostic_root_cause_hypotheses drch
             INNER JOIN topics t ON t.id = drch.topic_id
             WHERE drch.diagnostic_id = ?1 AND drch.topic_id = ?2
             ORDER BY drch.confidence_score DESC, drch.id ASC"
        } else {
            "SELECT drch.id, drch.diagnostic_id, drch.topic_id, t.name, drch.hypothesis_code,
                    drch.confidence_score, drch.recommended_action, drch.evidence_json, drch.created_at
             FROM diagnostic_root_cause_hypotheses drch
             INNER JOIN topics t ON t.id = drch.topic_id
             WHERE drch.diagnostic_id = ?1
             ORDER BY drch.confidence_score DESC, drch.id ASC"
        };
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = if let Some(topic_id) = topic_id {
            statement.query_map(params![diagnostic_id, topic_id], map_root_cause_hypothesis)
        } else {
            statement.query_map([diagnostic_id], map_root_cause_hypothesis)
        }
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn compute_topic_analytics(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        topic_id: i64,
        topic_name: &str,
    ) -> EcoachResult<DiagnosticTopicAnalytics> {
        let baseline = self.topic_phase_accuracy(diagnostic_id, topic_id, "baseline")?;
        let speed_accuracy = self.topic_phase_accuracy(diagnostic_id, topic_id, "speed")?;
        let speed_latency = self.topic_phase_latency(diagnostic_id, topic_id, "speed")?;
        let precision = self.topic_phase_accuracy(diagnostic_id, topic_id, "precision")?;
        let pressure = self.topic_phase_accuracy(diagnostic_id, topic_id, "pressure")?;
        let flex = self.topic_phase_accuracy(diagnostic_id, topic_id, "flex")?;
        let root_cause = self.topic_phase_accuracy(diagnostic_id, topic_id, "root_cause")?;
        let mut error_profile = self.load_error_profile(student_id, topic_id)?;
        let confidence_signals = self.load_confidence_signals(diagnostic_id, topic_id)?;
        error_profile.high_confidence_wrong_count = confidence_signals.high_confidence_wrong_count;
        error_profile.low_confidence_correct_count =
            confidence_signals.low_confidence_correct_count;

        let fluency_score = if speed_accuracy > 0 {
            let latency_bonus = latency_to_fluency(speed_latency);
            ((speed_accuracy as i64 * 7 + latency_bonus as i64 * 3) / 10) as BasisPoints
        } else {
            baseline
        };
        let flexibility_score = if flex > 0 { flex } else { baseline };
        let pressure_score = if pressure > 0 { pressure } else { baseline };
        let precision_score = if precision > 0 { precision } else { baseline };
        let stability_score = ((baseline as i64
            + precision_score as i64
            + pressure_score as i64
            + flexibility_score as i64)
            / 4) as BasisPoints;
        let mastery_score =
            ((baseline as i64 + root_cause as i64 + precision_score as i64) / 3) as BasisPoints;
        let classification = classify_topic_analytics(
            mastery_score,
            fluency_score,
            precision_score,
            pressure_score,
            flexibility_score,
            &error_profile,
        )
        .to_string();
        let confidence_score = analytics_confidence_score(
            mastery_score,
            pressure_score,
            flexibility_score,
            &error_profile,
        );
        let recommended_action = analytics_recommended_action(
            &classification,
            pressure_score,
            flexibility_score,
            &error_profile,
        )
        .to_string();

        Ok(DiagnosticTopicAnalytics {
            diagnostic_id,
            topic_id,
            topic_name: topic_name.to_string(),
            mastery_score,
            fluency_score,
            precision_score,
            pressure_score,
            flexibility_score,
            stability_score,
            classification,
            confidence_score,
            recommended_action,
        })
    }

    fn persist_topic_analytics(&self, analytics: &DiagnosticTopicAnalytics) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO diagnostic_topic_analytics (
                    diagnostic_id, topic_id, mastery_score, fluency_score, precision_score,
                    pressure_score, flexibility_score, stability_score, classification,
                    confidence_score, recommended_action
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(diagnostic_id, topic_id) DO UPDATE SET
                    mastery_score = excluded.mastery_score,
                    fluency_score = excluded.fluency_score,
                    precision_score = excluded.precision_score,
                    pressure_score = excluded.pressure_score,
                    flexibility_score = excluded.flexibility_score,
                    stability_score = excluded.stability_score,
                    classification = excluded.classification,
                    confidence_score = excluded.confidence_score,
                    recommended_action = excluded.recommended_action,
                    updated_at = datetime('now')",
                params![
                    analytics.diagnostic_id,
                    analytics.topic_id,
                    analytics.mastery_score,
                    analytics.fluency_score,
                    analytics.precision_score,
                    analytics.pressure_score,
                    analytics.flexibility_score,
                    analytics.stability_score,
                    analytics.classification,
                    analytics.confidence_score,
                    analytics.recommended_action,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn build_root_cause_hypotheses(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        topic_id: i64,
        topic_name: &str,
        analytics: &DiagnosticTopicAnalytics,
    ) -> EcoachResult<Vec<DiagnosticRootCauseHypothesis>> {
        let error_profile = self.load_error_profile(student_id, topic_id)?;
        let pressure_drop = analytics
            .mastery_score
            .saturating_sub(analytics.pressure_score);
        let flex_drop = analytics
            .mastery_score
            .saturating_sub(analytics.flexibility_score);
        let mut hypotheses = Vec::new();

        if pressure_drop >= 1_500 {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "timed_pressure_breakdown",
                (7_000 + pressure_drop as i64 / 2).min(9_500) as BasisPoints,
                "timed_repair_checkpoint",
                json!({
                    "pressure_drop": pressure_drop,
                    "pressure_score": analytics.pressure_score,
                    "mastery_score": analytics.mastery_score,
                }),
            ));
        }

        if flex_drop >= 1_500 {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "transfer_fragility",
                (6_800 + flex_drop as i64 / 2).min(9_300) as BasisPoints,
                "mixed_context_repair",
                json!({
                    "flex_drop": flex_drop,
                    "flexibility_score": analytics.flexibility_score,
                }),
            ));
        }

        if analytics.fluency_score + 1_500 < analytics.precision_score {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "retrieval_latency_gap",
                7_400,
                "fluency_ladder",
                json!({
                    "fluency_score": analytics.fluency_score,
                    "precision_score": analytics.precision_score,
                }),
            ));
        }

        if error_profile.conceptual_confusion_score >= 5_000
            || error_profile.misconception_signal_count > 0
        {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "misconception_root_cause",
                8_200,
                "misconception_repair_pack",
                json!({
                    "conceptual_confusion_score": error_profile.conceptual_confusion_score,
                    "misconception_signal_count": error_profile.misconception_signal_count,
                }),
            ));
        } else if error_profile.knowledge_gap_score >= 5_000 || analytics.mastery_score < 4_500 {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "foundation_gap",
                7_600,
                "teach_then_guided_practice",
                json!({
                    "knowledge_gap_score": error_profile.knowledge_gap_score,
                    "mastery_score": analytics.mastery_score,
                }),
            ));
        }

        let confidence_signals = self.load_confidence_signals(diagnostic_id, topic_id)?;
        if confidence_signals.high_confidence_wrong_count > 0
            || confidence_signals.low_confidence_correct_count >= 2
        {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "confidence_distortion",
                (6_900
                    + (confidence_signals.high_confidence_wrong_count * 800)
                    + (confidence_signals.low_confidence_correct_count * 350))
                    .min(9_300) as BasisPoints,
                "confidence_reflection_check",
                json!({
                    "high_confidence_wrong_count": confidence_signals.high_confidence_wrong_count,
                    "low_confidence_correct_count": confidence_signals.low_confidence_correct_count,
                }),
            ));
        }

        if hypotheses.is_empty() {
            hypotheses.push(build_hypothesis(
                diagnostic_id,
                topic_id,
                topic_name,
                "confidence_gap",
                6_500,
                analytics.recommended_action.as_str(),
                json!({
                    "classification": analytics.classification,
                    "confidence_score": analytics.confidence_score,
                }),
            ));
        }

        Ok(hypotheses)
    }

    fn persist_root_cause_hypotheses(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
        hypotheses: &[DiagnosticRootCauseHypothesis],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM diagnostic_root_cause_hypotheses
                 WHERE diagnostic_id = ?1 AND topic_id = ?2",
                params![diagnostic_id, topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for hypothesis in hypotheses {
            self.conn
                .execute(
                    "INSERT INTO diagnostic_root_cause_hypotheses (
                        diagnostic_id, topic_id, hypothesis_code, confidence_score,
                        recommended_action, evidence_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        hypothesis.diagnostic_id,
                        hypothesis.topic_id,
                        hypothesis.hypothesis_code,
                        hypothesis.confidence_score,
                        hypothesis.recommended_action,
                        serde_json::to_string(&hypothesis.evidence)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    pub fn list_recent_wrong_answer_diagnoses(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<WrongAnswerDiagnosis>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, question_id, topic_id, error_type, primary_diagnosis,
                        secondary_diagnosis, severity, diagnosis_summary, recommended_action,
                        confidence_score, created_at
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(WrongAnswerDiagnosis {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    question_id: row.get(2)?,
                    topic_id: row.get(3)?,
                    error_type: row.get(4)?,
                    primary_diagnosis: row.get(5)?,
                    secondary_diagnosis: row.get(6)?,
                    severity: row.get(7)?,
                    diagnosis_summary: row.get(8)?,
                    recommended_action: row.get(9)?,
                    confidence_score: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut diagnoses = Vec::new();
        for row in rows {
            diagnoses.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(diagnoses)
    }

    fn load_previous_completed_diagnostic_context(
        &self,
        student_id: i64,
        subject_id: i64,
        current_diagnostic_id: i64,
    ) -> EcoachResult<Option<PreviousDiagnosticContext>> {
        let row = self
            .conn
            .query_row(
                "SELECT id, completed_at, result_json
                 FROM diagnostic_instances
                 WHERE student_id = ?1
                   AND subject_id = ?2
                   AND status = 'completed'
                   AND id <> ?3
                 ORDER BY COALESCE(completed_at, started_at) DESC, id DESC
                 LIMIT 1",
                params![student_id, subject_id, current_diagnostic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some((diagnostic_id, completed_at, result_json)) = row else {
            return Ok(None);
        };
        let overall_readiness = if let Some(result_json) = result_json.as_deref() {
            serde_json::from_str::<DiagnosticResult>(result_json)
                .ok()
                .map(|result| result.overall_readiness)
                .or_else(|| self.load_diagnostic_overall_readiness(diagnostic_id).ok().flatten())
        } else {
            self.load_diagnostic_overall_readiness(diagnostic_id)?
        };

        Ok(Some(PreviousDiagnosticContext {
            diagnostic_id,
            completed_at,
            overall_readiness,
        }))
    }

    fn load_diagnostic_overall_readiness(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Option<BasisPoints>> {
        self.conn
            .query_row(
                "SELECT CAST(ROUND(AVG(mastery_score)) AS INTEGER)
                 FROM diagnostic_topic_analytics
                 WHERE diagnostic_id = ?1",
                [diagnostic_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .map(|value| value.flatten().map(|score| score as BasisPoints))
    }

    fn load_historical_topic_snapshot(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<HistoricalTopicSnapshot>> {
        let row = self
            .conn
            .query_row(
                "SELECT dta.diagnostic_id, dta.topic_id, t.name, dta.mastery_score,
                        dta.pressure_score, dta.flexibility_score, dta.classification
                 FROM diagnostic_topic_analytics dta
                 INNER JOIN topics t ON t.id = dta.topic_id
                 WHERE dta.diagnostic_id = ?1 AND dta.topic_id = ?2",
                params![diagnostic_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, BasisPoints>(3)?,
                        row.get::<_, BasisPoints>(4)?,
                        row.get::<_, BasisPoints>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some((
            diagnostic_id,
            topic_id,
            topic_name,
            mastery_score,
            pressure_score,
            flexibility_score,
            classification,
        )) = row else {
            return Ok(None);
        };

        let top_hypothesis = self
            .conn
            .query_row(
                "SELECT hypothesis_code, confidence_score
                 FROM diagnostic_root_cause_hypotheses
                 WHERE diagnostic_id = ?1 AND topic_id = ?2
                 ORDER BY confidence_score DESC, id ASC
                 LIMIT 1",
                params![diagnostic_id, topic_id],
                |row| Ok((Some(row.get::<_, String>(0)?), Some(row.get::<_, BasisPoints>(1)?))),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((None, None));

        Ok(Some(HistoricalTopicSnapshot {
            diagnostic_id,
            topic_id,
            topic_name,
            mastery_score,
            pressure_score,
            flexibility_score,
            classification,
            top_hypothesis_code: top_hypothesis.0,
            top_hypothesis_confidence: top_hypothesis.1,
        }))
    }

    fn append_runtime_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'diagnostic', ?3, ?4, ?5, ?6)",
                params![
                    event.event_id,
                    event.event_type,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn topic_phase_accuracy(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
        phase_code: &str,
    ) -> EcoachResult<BasisPoints> {
        let (count, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(dia.is_correct), 0)
                 FROM diagnostic_item_attempts dia
                 INNER JOIN diagnostic_session_phases dsp ON dsp.id = dia.phase_id
                 INNER JOIN questions q ON q.id = dia.question_id
                 WHERE dia.diagnostic_id = ?1
                   AND q.topic_id = ?2
                   AND COALESCE(dsp.phase_code, lower(replace(dsp.phase_type, ' ', '_'))) = ?3
                   AND dia.selected_option_id IS NOT NULL",
                params![diagnostic_id, topic_id, phase_code],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0));
        if count == 0 {
            return Ok(0);
        }
        Ok(((correct as f64 / count as f64) * 10_000.0).round() as BasisPoints)
    }

    fn topic_phase_latency(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
        phase_code: &str,
    ) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT AVG(dia.response_time_ms)
                 FROM diagnostic_item_attempts dia
                 INNER JOIN diagnostic_session_phases dsp ON dsp.id = dia.phase_id
                 INNER JOIN questions q ON q.id = dia.question_id
                 WHERE dia.diagnostic_id = ?1
                   AND q.topic_id = ?2
                   AND COALESCE(dsp.phase_code, lower(replace(dsp.phase_type, ' ', '_'))) = ?3
                   AND dia.response_time_ms IS NOT NULL",
                params![diagnostic_id, topic_id, phase_code],
                |row| row.get::<_, Option<f64>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .map(|value| value.flatten().map(|latency| latency.round() as i64))
    }

    fn load_error_profile(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<ErrorProfileSnapshot> {
        let profile = self
            .conn
            .query_row(
                "SELECT knowledge_gap_score, conceptual_confusion_score, recognition_failure_score,
                        pressure_breakdown_score, speed_error_score
                 FROM student_error_profiles
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok(ErrorProfileSnapshot {
                        knowledge_gap_score: row.get(0)?,
                        conceptual_confusion_score: row.get(1)?,
                        recognition_failure_score: row.get(2)?,
                        pressure_breakdown_score: row.get(3)?,
                        speed_error_score: row.get(4)?,
                        misconception_signal_count: 0,
                        high_confidence_wrong_count: 0,
                        low_confidence_correct_count: 0,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or_default();
        let misconception_signal_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1 AND topic_id = ?2
                   AND (error_type = 'misconception_triggered' OR primary_diagnosis = 'misconception_trigger')",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(ErrorProfileSnapshot {
            misconception_signal_count,
            ..profile
        })
    }

    fn load_confidence_signals(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
    ) -> EcoachResult<ConfidenceSignalSnapshot> {
        self.conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN dia.is_correct = 0 AND dia.confidence_level = 'sure' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN dia.is_correct = 1 AND (dia.confidence_level = 'guessed' OR dia.confidence_level = 'not_sure') THEN 1 ELSE 0 END), 0)
                 FROM diagnostic_item_attempts dia
                 INNER JOIN questions q ON q.id = dia.question_id
                 WHERE dia.diagnostic_id = ?1 AND q.topic_id = ?2",
                params![diagnostic_id, topic_id],
                |row| {
                    Ok(ConfidenceSignalSnapshot {
                        high_confidence_wrong_count: row.get(0)?,
                        low_confidence_correct_count: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .map(|value| value.unwrap_or_default())
    }

    fn load_subject_topic_ids(&self, subject_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id FROM topics WHERE subject_id = ?1 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_ids = Vec::new();
        for row in rows {
            topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(topic_ids)
    }

    fn load_root_cause_topic_ids(
        &self,
        student_id: i64,
        subject_id: i64,
        fallback_topic_ids: &[i64],
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2
                 ORDER BY sts.mastery_score ASC, sts.fragility_score DESC, sts.priority_score DESC, sts.topic_id ASC
                 LIMIT 3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topic_ids = Vec::new();
        for row in rows {
            topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        if topic_ids.is_empty() {
            Ok(fallback_topic_ids.iter().copied().take(3).collect())
        } else {
            Ok(topic_ids)
        }
    }

    fn select_phase_questions(
        &self,
        subject_id: i64,
        topic_ids: &[i64],
        template: &DiagnosticPhaseTemplate,
        recently_seen_question_ids: &[i64],
    ) -> EcoachResult<Vec<SelectedQuestion>> {
        let selector = QuestionSelector::new(self.conn);
        let primary_selection = selector.select_questions(&QuestionSelectionRequest {
            subject_id,
            topic_ids: topic_ids.to_vec(),
            target_question_count: template.question_count,
            target_difficulty: None,
            weakness_topic_ids: topic_ids.to_vec(),
            recently_seen_question_ids: recently_seen_question_ids.to_vec(),
            timed: template.timed,
        })?;

        if !primary_selection.is_empty() || recently_seen_question_ids.is_empty() {
            return Ok(primary_selection);
        }

        selector.select_questions(&QuestionSelectionRequest {
            subject_id,
            topic_ids: topic_ids.to_vec(),
            target_question_count: template.question_count,
            target_difficulty: None,
            weakness_topic_ids: topic_ids.to_vec(),
            recently_seen_question_ids: Vec::new(),
            timed: template.timed,
        })
    }

    fn retarget_pending_phase(
        &self,
        diagnostic_id: i64,
        phase_id: i64,
        phase_code: &str,
        question_count: i64,
        time_limit_seconds: Option<i64>,
        timed: bool,
    ) -> EcoachResult<()> {
        let (student_id, subject_id): (i64, i64) = self
            .conn
            .query_row(
                "SELECT student_id, subject_id FROM diagnostic_instances WHERE id = ?1",
                [diagnostic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(mut template) = template_for_phase_code(phase_code) else {
            return Ok(());
        };
        template.question_count = question_count.max(1) as usize;
        template.time_limit_seconds = time_limit_seconds;
        template.timed = timed;

        let branch_topics =
            self.load_branch_topic_ids(diagnostic_id, student_id, subject_id, template.code)?;
        let seen_question_ids = self.load_diagnostic_seen_question_ids(diagnostic_id)?;
        let selected_questions =
            self.select_phase_questions(subject_id, &branch_topics, &template, &seen_question_ids)?;

        self.conn
            .execute(
                "DELETE FROM diagnostic_item_attempts WHERE diagnostic_id = ?1 AND phase_id = ?2",
                params![diagnostic_id, phase_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.insert_phase_items(diagnostic_id, phase_id, &template, &selected_questions)?;

        let condition_profile_json = serde_json::to_string(&json!({
            "phase_code": template.code.as_str(),
            "timed": template.timed,
            "time_limit_seconds": template.time_limit_seconds,
            "condition_type": template.code.condition_type(),
            "confidence_prompt": matches!(template.code, DiagnosticPhaseCode::RootCause),
            "concept_guess_prompt": matches!(template.code, DiagnosticPhaseCode::Flex | DiagnosticPhaseCode::RootCause),
            "branch_topic_ids": branch_topics,
            "branch_reason": branch_reason_for_phase(template.code),
            "adaptive_retargeted": true,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE diagnostic_session_phases
                 SET question_count = ?1,
                     condition_profile_json = ?2
                 WHERE id = ?3",
                params![
                    selected_questions.len() as i64,
                    condition_profile_json,
                    phase_id
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn load_branch_topic_ids(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        subject_id: i64,
        phase_code: DiagnosticPhaseCode,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.topic_id,
                        COUNT(*) AS total_items,
                        COALESCE(SUM(CASE WHEN dia.is_correct = 1 THEN 1 ELSE 0 END), 0) AS correct_items,
                        COALESCE(SUM(CASE WHEN dia.is_correct = 0 AND dia.confidence_level = 'sure' THEN 1 ELSE 0 END), 0) AS high_conf_wrong,
                        COALESCE(SUM(CASE WHEN dia.is_correct = 1 AND (dia.confidence_level = 'guessed' OR dia.confidence_level = 'not_sure') THEN 1 ELSE 0 END), 0) AS low_conf_correct,
                        CAST(COALESCE(AVG(COALESCE(dia.response_time_ms, 0)), 0) AS INTEGER) AS avg_response_time_ms
                 FROM diagnostic_item_attempts dia
                 INNER JOIN questions q ON q.id = dia.question_id
                 INNER JOIN diagnostic_session_phases dsp ON dsp.id = dia.phase_id
                 WHERE dia.diagnostic_id = ?1
                   AND dsp.status = 'completed'
                 GROUP BY q.topic_id
                 ORDER BY q.topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                Ok(TopicBranchSignal {
                    topic_id: row.get(0)?,
                    total_items: row.get(1)?,
                    correct_items: row.get(2)?,
                    high_conf_wrong: row.get(3)?,
                    low_conf_correct: row.get(4)?,
                    avg_response_time_ms: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut scored = Vec::new();
        for row in rows {
            let signal = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let score = branch_priority_for_phase(&signal, phase_code);
            scored.push((signal.topic_id, score));
        }
        scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        let mut topic_ids = scored
            .into_iter()
            .take(match phase_code {
                DiagnosticPhaseCode::RootCause => 2,
                _ => 3,
            })
            .map(|(topic_id, _)| topic_id)
            .collect::<Vec<_>>();
        if !topic_ids.is_empty() {
            return Ok(topic_ids);
        }

        let fallback = match phase_code {
            DiagnosticPhaseCode::RootCause => self.load_root_cause_topic_ids(
                student_id,
                subject_id,
                &self.load_subject_topic_ids(subject_id)?,
            )?,
            _ => self.load_subject_topic_ids(subject_id)?,
        };
        topic_ids.extend(fallback.into_iter().take(3));
        Ok(topic_ids)
    }

    fn load_diagnostic_seen_question_ids(&self, diagnostic_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT question_id
                 FROM diagnostic_item_attempts
                 WHERE diagnostic_id = ?1
                 ORDER BY question_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut question_ids = Vec::new();
        for row in rows {
            question_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(question_ids)
    }

    fn insert_phase_row(
        &self,
        diagnostic_id: i64,
        phase_number: i64,
        template: &DiagnosticPhaseTemplate,
        question_count: i64,
        is_active: bool,
    ) -> EcoachResult<i64> {
        let condition_profile_json = serde_json::to_string(&json!({
            "phase_code": template.code.as_str(),
            "timed": template.timed,
            "time_limit_seconds": template.time_limit_seconds,
            "condition_type": template.code.condition_type(),
            "confidence_prompt": matches!(template.code, DiagnosticPhaseCode::RootCause),
            "concept_guess_prompt": matches!(template.code, DiagnosticPhaseCode::Flex | DiagnosticPhaseCode::RootCause),
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO diagnostic_session_phases (
                    diagnostic_id, phase_number, phase_type, status, question_count, started_at,
                    phase_result_json, phase_code, phase_title, condition_profile_json, time_limit_seconds
                 ) VALUES (?1, ?2, ?3, ?4, ?5, CASE WHEN ?6 = 1 THEN datetime('now') ELSE NULL END, '{}', ?7, ?8, ?9, ?10)",
                params![
                    diagnostic_id,
                    phase_number,
                    template.code.storage_phase_type(),
                    if is_active { "active" } else { "pending" },
                    question_count,
                    is_active as i64,
                    template.code.as_str(),
                    template.code.title(),
                    condition_profile_json,
                    template.time_limit_seconds,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    fn insert_phase_items(
        &self,
        diagnostic_id: i64,
        phase_id: i64,
        template: &DiagnosticPhaseTemplate,
        selected_questions: &[SelectedQuestion],
    ) -> EcoachResult<()> {
        for (index, selected) in selected_questions.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO diagnostic_item_attempts (
                        diagnostic_id, phase_id, question_id, display_order, condition_type, evidence_weight
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        diagnostic_id,
                        phase_id,
                        selected.question.id,
                        index as i64,
                        template.code.condition_type(),
                        template.evidence_weight,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn list_diagnostic_phases(&self, diagnostic_id: i64) -> EcoachResult<Vec<DiagnosticPhasePlan>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    phase_number,
                    COALESCE(phase_code, lower(replace(phase_type, ' ', '_'))),
                    COALESCE(phase_title, phase_type),
                    phase_type,
                    status,
                    question_count,
                    time_limit_seconds,
                    json_extract(condition_profile_json, '$.condition_type')
                 FROM diagnostic_session_phases
                 WHERE diagnostic_id = ?1
                 ORDER BY phase_number ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([diagnostic_id], |row| {
                Ok(DiagnosticPhasePlan {
                    phase_id: row.get(0)?,
                    phase_number: row.get(1)?,
                    phase_code: row.get(2)?,
                    phase_title: row.get(3)?,
                    phase_type: row.get(4)?,
                    status: row.get(5)?,
                    question_count: row.get(6)?,
                    time_limit_seconds: row.get(7)?,
                    condition_type: row
                        .get::<_, Option<String>>(8)?
                        .unwrap_or_else(|| "normal".to_string()),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut phases = Vec::new();
        for row in rows {
            phases.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(phases)
    }
}

#[derive(Debug, Default)]
struct ErrorProfileSnapshot {
    knowledge_gap_score: BasisPoints,
    conceptual_confusion_score: BasisPoints,
    recognition_failure_score: BasisPoints,
    pressure_breakdown_score: BasisPoints,
    speed_error_score: BasisPoints,
    misconception_signal_count: i64,
    high_confidence_wrong_count: i64,
    low_confidence_correct_count: i64,
}

#[derive(Debug, Default)]
struct ConfidenceSignalSnapshot {
    high_confidence_wrong_count: i64,
    low_confidence_correct_count: i64,
}

#[derive(Debug)]
struct TopicBranchSignal {
    topic_id: i64,
    total_items: i64,
    correct_items: i64,
    high_conf_wrong: i64,
    low_conf_correct: i64,
    avg_response_time_ms: i64,
}

fn latency_to_fluency(latency_ms: Option<i64>) -> BasisPoints {
    let latency_ms = latency_ms.unwrap_or(30_000).max(1) as f64;
    ((30_000.0 / latency_ms).clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

fn classify_topic_analytics(
    mastery_score: BasisPoints,
    fluency_score: BasisPoints,
    precision_score: BasisPoints,
    pressure_score: BasisPoints,
    flexibility_score: BasisPoints,
    error_profile: &ErrorProfileSnapshot,
) -> &'static str {
    if error_profile.high_confidence_wrong_count > 0 {
        "confidence_distorted"
    } else if pressure_score + 1_500 < mastery_score
        || error_profile.pressure_breakdown_score >= 5_000
    {
        "fragile_under_pressure"
    } else if flexibility_score + 1_500 < mastery_score
        || error_profile.recognition_failure_score >= 5_000
    {
        "transfer_fragile"
    } else if fluency_score + 1_500 < precision_score || error_profile.speed_error_score >= 5_000 {
        "slow_but_right"
    } else if error_profile.conceptual_confusion_score >= 5_000
        || error_profile.misconception_signal_count > 0
    {
        "misconception_prone"
    } else if mastery_score >= 8_000 && pressure_score >= 7_000 && flexibility_score >= 7_000 {
        "secure"
    } else {
        "needs_repair"
    }
}

fn analytics_confidence_score(
    mastery_score: BasisPoints,
    pressure_score: BasisPoints,
    flexibility_score: BasisPoints,
    error_profile: &ErrorProfileSnapshot,
) -> BasisPoints {
    let evidence = mastery_score as i64
        + pressure_score as i64
        + flexibility_score as i64
        + error_profile.conceptual_confusion_score as i64
        + error_profile.knowledge_gap_score as i64
        + (error_profile.high_confidence_wrong_count.min(3) * 600)
        + (error_profile.low_confidence_correct_count.min(3) * 400);
    ((evidence / 5).clamp(4_500, 9_200)) as BasisPoints
}

fn analytics_recommended_action(
    classification: &str,
    pressure_score: BasisPoints,
    flexibility_score: BasisPoints,
    error_profile: &ErrorProfileSnapshot,
) -> &'static str {
    if classification == "confidence_distorted" {
        "confidence_reflection_check"
    } else if classification == "fragile_under_pressure"
        || error_profile.pressure_breakdown_score >= 5_000
    {
        "timed_repair_checkpoint"
    } else if classification == "transfer_fragile" || flexibility_score < 5_000 {
        "mixed_context_repair"
    } else if classification == "slow_but_right" {
        "fluency_ladder"
    } else if classification == "misconception_prone" {
        "misconception_repair_pack"
    } else if error_profile.knowledge_gap_score >= 5_000 || pressure_score < 4_500 {
        "teach_then_guided_practice"
    } else {
        "generate_plan"
    }
}

fn build_hypothesis(
    diagnostic_id: i64,
    topic_id: i64,
    topic_name: &str,
    hypothesis_code: &str,
    confidence_score: BasisPoints,
    recommended_action: &str,
    evidence: Value,
) -> DiagnosticRootCauseHypothesis {
    DiagnosticRootCauseHypothesis {
        id: 0,
        diagnostic_id,
        topic_id,
        topic_name: topic_name.to_string(),
        hypothesis_code: hypothesis_code.to_string(),
        confidence_score,
        recommended_action: recommended_action.to_string(),
        evidence,
        created_at: String::new(),
    }
}

fn template_for_phase_code(phase_code: &str) -> Option<DiagnosticPhaseTemplate> {
    diagnostic_phase_templates(DiagnosticMode::Deep)
        .into_iter()
        .find(|template| template.code.as_str() == phase_code)
}

fn branch_reason_for_phase(phase_code: DiagnosticPhaseCode) -> &'static str {
    match phase_code {
        DiagnosticPhaseCode::Speed => "adaptive_zoom_on_weak_or_slow_topics",
        DiagnosticPhaseCode::Precision => "precision_probe_on_error_prone_topics",
        DiagnosticPhaseCode::Pressure => "condition_testing_on_pressure_risk_topics",
        DiagnosticPhaseCode::Flex => "stability_recheck_on_fragile_topics",
        DiagnosticPhaseCode::RootCause => "root_cause_probe_on_confidence_or_accuracy_mismatch",
        DiagnosticPhaseCode::Baseline => "broad_scan",
    }
}

fn branch_priority_for_phase(signal: &TopicBranchSignal, phase_code: DiagnosticPhaseCode) -> i64 {
    let accuracy_bp = if signal.total_items > 0 {
        (signal.correct_items * 10_000) / signal.total_items
    } else {
        0
    };
    let inaccuracy = 10_000 - accuracy_bp;
    match phase_code {
        DiagnosticPhaseCode::Speed => {
            inaccuracy
                + signal.avg_response_time_ms.min(90_000) / 15
                + signal.low_conf_correct * 350
        }
        DiagnosticPhaseCode::Precision => {
            inaccuracy + signal.high_conf_wrong * 1_200 + signal.low_conf_correct * 500
        }
        DiagnosticPhaseCode::Pressure => {
            inaccuracy
                + signal.high_conf_wrong * 1_300
                + signal.avg_response_time_ms.min(90_000) / 18
        }
        DiagnosticPhaseCode::Flex => {
            inaccuracy + signal.low_conf_correct * 700 + signal.high_conf_wrong * 800
        }
        DiagnosticPhaseCode::RootCause => {
            inaccuracy + signal.high_conf_wrong * 1_500 + signal.low_conf_correct * 900
        }
        DiagnosticPhaseCode::Baseline => inaccuracy,
    }
}

fn unique_actions(actions: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for action in actions {
        if !action.is_empty() && seen.insert(action.clone()) {
            out.push(action);
        }
    }
    if out.is_empty() {
        out.push("generate_plan".to_string());
    }
    out
}

fn map_root_cause_hypothesis(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<DiagnosticRootCauseHypothesis> {
    let evidence_json: String = row.get(7)?;
    let evidence = serde_json::from_str::<Value>(&evidence_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(DiagnosticRootCauseHypothesis {
        id: row.get(0)?,
        diagnostic_id: row.get(1)?,
        topic_id: row.get(2)?,
        topic_name: row.get(3)?,
        hypothesis_code: row.get(4)?,
        confidence_score: row.get(5)?,
        recommended_action: row.get(6)?,
        evidence,
        created_at: row.get(8)?,
    })
}

fn diagnostic_phase_templates(mode: DiagnosticMode) -> Vec<DiagnosticPhaseTemplate> {
    match mode {
        DiagnosticMode::Quick => vec![
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Baseline,
                question_count: 6,
                time_limit_seconds: Some(75),
                timed: false,
                evidence_weight: 10_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Speed,
                question_count: 4,
                time_limit_seconds: Some(25),
                timed: true,
                evidence_weight: 8_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Pressure,
                question_count: 4,
                time_limit_seconds: Some(18),
                timed: true,
                evidence_weight: 9_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Flex,
                question_count: 4,
                time_limit_seconds: Some(60),
                timed: false,
                evidence_weight: 9_500,
            },
        ],
        DiagnosticMode::Standard => vec![
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Baseline,
                question_count: 8,
                time_limit_seconds: Some(90),
                timed: false,
                evidence_weight: 10_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Speed,
                question_count: 6,
                time_limit_seconds: Some(25),
                timed: true,
                evidence_weight: 8_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Precision,
                question_count: 6,
                time_limit_seconds: Some(120),
                timed: false,
                evidence_weight: 9_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Pressure,
                question_count: 6,
                time_limit_seconds: Some(18),
                timed: true,
                evidence_weight: 9_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Flex,
                question_count: 6,
                time_limit_seconds: Some(75),
                timed: false,
                evidence_weight: 9_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::RootCause,
                question_count: 4,
                time_limit_seconds: Some(45),
                timed: false,
                evidence_weight: 10_000,
            },
        ],
        DiagnosticMode::Deep => vec![
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Baseline,
                question_count: 10,
                time_limit_seconds: Some(90),
                timed: false,
                evidence_weight: 10_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Speed,
                question_count: 8,
                time_limit_seconds: Some(22),
                timed: true,
                evidence_weight: 8_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Precision,
                question_count: 8,
                time_limit_seconds: Some(135),
                timed: false,
                evidence_weight: 9_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Pressure,
                question_count: 8,
                time_limit_seconds: Some(15),
                timed: true,
                evidence_weight: 9_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Flex,
                question_count: 8,
                time_limit_seconds: Some(75),
                timed: false,
                evidence_weight: 9_500,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::RootCause,
                question_count: 6,
                time_limit_seconds: Some(45),
                timed: false,
                evidence_weight: 10_000,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_questions::QuestionService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn builds_persisted_diagnostic_battery_with_phase_items() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Ada', 'hash', 'salt', 'active')",
            [],
        )
        .expect("student should be insertable");
        let student_id = conn.last_insert_rowid();
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        let engine = DiagnosticEngine::new(&conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Standard,
            )
            .expect("diagnostic battery should build");
        let phase_one_items = engine
            .list_phase_items(battery.diagnostic_id, 1)
            .expect("phase items should be retrievable");

        assert_eq!(battery.session_mode, "standard");
        assert_eq!(battery.phases.len(), 6);
        assert_eq!(battery.phases[0].phase_code, "baseline");
        assert_eq!(battery.phases[0].status, "active");
        assert!(!phase_one_items.is_empty());
    }

    #[test]
    fn complete_diagnostic_persists_topic_analytics_and_hypotheses() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Kojo', 'hash', 'salt', 'active')",
            [],
        )
        .expect("student should be insertable");
        let student_id = conn.last_insert_rowid();
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_error_profiles (student_id, topic_id, conceptual_confusion_score, pressure_breakdown_score, speed_error_score)
             VALUES (?1, ?2, 6500, 5200, 4800)",
            rusqlite::params![student_id, topic_id],
        )
        .expect("error profile should insert");

        let engine = DiagnosticEngine::new(&conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Standard,
            )
            .expect("diagnostic battery should build");
        let question_service = QuestionService::new(&conn);

        for phase in &battery.phases {
            let items = engine
                .list_phase_items(battery.diagnostic_id, phase.phase_number)
                .expect("phase items should load");
            let Some(first_item) = items.first() else {
                continue;
            };
            let options = question_service
                .list_options(first_item.question_id)
                .expect("options should load");
            let correct_option_id = options
                .iter()
                .find(|option| option.is_correct)
                .map(|option| option.id)
                .expect("correct option should exist");
            let wrong_option_id = options
                .iter()
                .find(|option| !option.is_correct)
                .map(|option| option.id)
                .expect("wrong option should exist");
            let attempt_id: i64 = conn
                .query_row(
                    "SELECT id FROM diagnostic_item_attempts
                     WHERE diagnostic_id = ?1 AND phase_id = ?2 AND question_id = ?3
                     LIMIT 1",
                    rusqlite::params![
                        battery.diagnostic_id,
                        phase.phase_id,
                        first_item.question_id
                    ],
                    |row| row.get(0),
                )
                .expect("attempt should exist");
            let (selected_option_id, response_time_ms) = match phase.phase_code.as_str() {
                "baseline" => (correct_option_id, Some(14_000)),
                "speed" => (correct_option_id, Some(52_000)),
                "pressure" => (wrong_option_id, Some(18_000)),
                "flex" | "root_cause" => (wrong_option_id, Some(20_000)),
                _ => (correct_option_id, Some(16_000)),
            };
            engine
                .submit_phase_attempt(
                    battery.diagnostic_id,
                    attempt_id,
                    selected_option_id,
                    response_time_ms,
                    Some("not_sure"),
                    0,
                    false,
                    false,
                )
                .expect("phase attempt should submit");
        }

        let result = engine
            .complete_diagnostic(battery.diagnostic_id)
            .expect("diagnostic should complete");
        let analytics = engine
            .list_topic_analytics(battery.diagnostic_id)
            .expect("analytics should load");
        let hypotheses = engine
            .list_root_cause_hypotheses(battery.diagnostic_id, Some(topic_id))
            .expect("hypotheses should load");

        assert!(!result.topic_results.is_empty());
        assert!(!analytics.is_empty());
        assert!(!hypotheses.is_empty());
    }

    #[test]
    fn advancing_phase_retargets_next_phase_with_branch_metadata() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Yaw', 'hash', 'salt', 'active')",
            [],
        )
        .expect("student should be insertable");
        let student_id = conn.last_insert_rowid();
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        let engine = DiagnosticEngine::new(&conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Standard,
            )
            .expect("diagnostic battery should build");
        let first_phase_items = engine
            .list_phase_items(battery.diagnostic_id, 1)
            .expect("phase items should load");
        let question_service = QuestionService::new(&conn);
        let wrong_option_id = question_service
            .list_options(first_phase_items[0].question_id)
            .expect("options should load")
            .into_iter()
            .find(|option| !option.is_correct)
            .map(|option| option.id)
            .expect("wrong option should exist");
        let attempt_id: i64 = conn
            .query_row(
                "SELECT id FROM diagnostic_item_attempts WHERE diagnostic_id = ?1 AND phase_id = ?2 LIMIT 1",
                params![battery.diagnostic_id, battery.phases[0].phase_id],
                |row| row.get(0),
            )
            .expect("attempt should exist");
        engine
            .submit_phase_attempt(
                battery.diagnostic_id,
                attempt_id,
                wrong_option_id,
                Some(30_000),
                Some("sure"),
                0,
                false,
                false,
            )
            .expect("baseline attempt should submit");

        let next_phase = engine
            .advance_phase(battery.diagnostic_id, 1)
            .expect("phase should advance")
            .expect("next phase should exist");
        let condition_profile: String = conn
            .query_row(
                "SELECT condition_profile_json FROM diagnostic_session_phases WHERE id = ?1",
                [next_phase.phase_id],
                |row| row.get(0),
            )
            .expect("condition profile should exist");

        assert_eq!(next_phase.status, "active");
        assert!(condition_profile.contains("\"adaptive_retargeted\":true"));
        assert!(condition_profile.contains("\"branch_topic_ids\""));
    }

    #[test]
    fn complete_diagnostic_detects_confidence_distortion_root_cause() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Efua', 'hash', 'salt', 'active')",
            [],
        )
        .expect("student should be insertable");
        let student_id = conn.last_insert_rowid();
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("math subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        let engine = DiagnosticEngine::new(&conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Standard,
            )
            .expect("diagnostic battery should build");
        let question_service = QuestionService::new(&conn);

        for phase in &battery.phases {
            let items = engine
                .list_phase_items(battery.diagnostic_id, phase.phase_number)
                .expect("phase items should load");
            let Some(first_item) = items.first() else {
                continue;
            };
            let wrong_option_id = question_service
                .list_options(first_item.question_id)
                .expect("options should load")
                .into_iter()
                .find(|option| !option.is_correct)
                .map(|option| option.id)
                .expect("wrong option should exist");
            let attempt_id: i64 = conn
                .query_row(
                    "SELECT id FROM diagnostic_item_attempts
                     WHERE diagnostic_id = ?1 AND phase_id = ?2 AND question_id = ?3
                     LIMIT 1",
                    params![
                        battery.diagnostic_id,
                        phase.phase_id,
                        first_item.question_id
                    ],
                    |row| row.get(0),
                )
                .expect("attempt should exist");
            engine
                .submit_phase_attempt(
                    battery.diagnostic_id,
                    attempt_id,
                    wrong_option_id,
                    Some(18_000),
                    Some("sure"),
                    0,
                    false,
                    false,
                )
                .expect("phase attempt should submit");
        }

        let result = engine
            .complete_diagnostic(battery.diagnostic_id)
            .expect("diagnostic should complete");
        let analytics = engine
            .list_topic_analytics(battery.diagnostic_id)
            .expect("analytics should load");
        let hypotheses = engine
            .list_root_cause_hypotheses(battery.diagnostic_id, Some(topic_id))
            .expect("hypotheses should load");

        assert!(
            result
                .recommended_next_actions
                .iter()
                .any(|action| action == "confidence_reflection_check")
        );
        assert!(
            analytics
                .iter()
                .any(|item| item.classification == "confidence_distorted")
        );
        assert!(
            hypotheses
                .iter()
                .any(|item| item.hypothesis_code == "confidence_distortion")
        );
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
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
