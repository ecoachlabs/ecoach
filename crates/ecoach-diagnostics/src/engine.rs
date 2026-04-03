use ecoach_questions::{
    Question, QuestionOption, QuestionSelectionRequest, QuestionSelector, QuestionService,
    SelectedQuestion,
};
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value, json};

use crate::models::{
    DiagnosticAudienceReport, DiagnosticBattery, DiagnosticCauseEvolution,
    DiagnosticConditionMetrics, DiagnosticInterventionPrescription, DiagnosticItemRoutingProfile,
    DiagnosticLearningProfile, DiagnosticLongitudinalSummary, DiagnosticMode,
    DiagnosticOverallSummary, DiagnosticPhaseCode, DiagnosticPhaseItem, DiagnosticPhasePlan,
    DiagnosticProblemCauseFixCard, DiagnosticRecommendation, DiagnosticResult,
    DiagnosticRootCauseHypothesis, DiagnosticSessionScore, DiagnosticSkillResult,
    DiagnosticSubjectBlueprint, DiagnosticTopicAnalytics, TopicDiagnosticLongitudinalSignal,
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
        self.ensure_diagnostic_group_shadow(diagnostic_id, student_id, subject_id, mode)?;
        self.ensure_subject_blueprint(subject_id)?;
        self.ensure_routing_profiles_for_scope(subject_id, &topic_ids)?;
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
        self.ensure_routing_profiles_for_scope(subject_id, &topic_ids)?;
        let selector = QuestionSelector::new(self.conn);
        selector.select_questions(&QuestionSelectionRequest {
            subject_id,
            topic_ids: topic_ids.clone(),
            target_question_count: count,
            target_difficulty: None,
            weakness_topic_ids: topic_ids,
            recently_seen_question_ids: Vec::new(),
            timed: false,
            diagnostic_stage: Some(DiagnosticPhaseCode::Baseline.as_str().to_string()),
            condition_type: Some(DiagnosticPhaseCode::Baseline.condition_type().to_string()),
            require_confidence_prompt: false,
            require_concept_guess_prompt: false,
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
        self.submit_phase_attempt_details(
            diagnostic_id,
            attempt_id,
            Some(selected_option_id),
            response_time_ms,
            confidence_level,
            changed_answer_count,
            skipped,
            timed_out,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_phase_attempt_details(
        &self,
        diagnostic_id: i64,
        attempt_id: i64,
        selected_option_id: Option<i64>,
        response_time_ms: Option<i64>,
        confidence_level: Option<&str>,
        changed_answer_count: i64,
        skipped: bool,
        timed_out: bool,
        first_focus_at: Option<&str>,
        first_input_at: Option<&str>,
        concept_guess: Option<&str>,
        final_answer: Option<&Value>,
        raw_interaction_log: Option<&Value>,
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
        let selected_option = if let Some(selected_option_id) = selected_option_id {
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
            Some(option)
        } else {
            None
        };
        let correct_options = question_service
            .list_options(question_id)?
            .into_iter()
            .filter(|option| option.is_correct)
            .collect::<Vec<_>>();
        let final_answer_json = final_answer
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let raw_interaction_log_json =
            raw_interaction_log
                .map(serde_json::to_string)
                .transpose()
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let submitted_answer_text = final_answer.and_then(extract_final_answer_text);
        if selected_option.is_none() && submitted_answer_text.is_none() && !skipped && !timed_out {
            return Err(EcoachError::Validation(
                "diagnostic attempt requires either an option selection or a final answer"
                    .to_string(),
            ));
        }
        let is_correct = if let Some(option) = selected_option.as_ref() {
            option.is_correct
        } else if skipped || timed_out {
            false
        } else if correct_options.is_empty() {
            return Err(EcoachError::Validation(format!(
                "diagnostic question {} has no grading anchor for free-response submission",
                question_id
            )));
        } else {
            submitted_answer_text
                .as_deref()
                .map(|text| answer_matches_correct_options(text, &correct_options))
                .unwrap_or(false)
        };

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
                     timed_out = ?7,
                     first_focus_at = COALESCE(?8, first_focus_at),
                     first_input_at = COALESCE(?9, first_input_at),
                     concept_guess = COALESCE(?10, concept_guess),
                     final_answer_json = COALESCE(?11, final_answer_json),
                     raw_interaction_log_json = COALESCE(?12, raw_interaction_log_json)
                 WHERE id = ?13 AND diagnostic_id = ?14",
                params![
                    response_time_ms,
                    selected_option_id,
                    if is_correct { 1 } else { 0 },
                    confidence_level,
                    changed_answer_count,
                    if skipped { 1 } else { 0 },
                    if timed_out { 1 } else { 0 },
                    first_focus_at,
                    first_input_at,
                    concept_guess,
                    final_answer_json,
                    raw_interaction_log_json,
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
        self.persist_phase_summary(diagnostic_id, completed_phase_number)?;

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
                    params![
                        diagnostic_instance_status_for_phase(phase_number),
                        diagnostic_id
                    ],
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
        self.complete_diagnostic_group_shadow(diagnostic_id)?;
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
            let hypotheses = enrich_hypotheses_with_longitudinal_context(
                hypotheses,
                &analytics,
                longitudinal_signal.as_ref(),
            );
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
                endurance_score: analytics.endurance_score,
                weakness_type: analytics.weakness_type.clone(),
                failure_stage: analytics.failure_stage.clone(),
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

        let session_scores = self.compute_session_scores(diagnostic_id)?;
        let skill_results = self.compute_skill_results(diagnostic_id, student_id)?;
        let condition_metrics =
            self.build_condition_metrics(&topic_results, &session_scores, &skill_results);
        let overall_summary = build_overall_summary(
            &readiness_band,
            &topic_results,
            &skill_results,
            recommended_next_actions.first().cloned(),
        );
        let learning_profile = self.build_learning_profile(
            diagnostic_id,
            student_id,
            subject_id,
            &topic_results,
            &skill_results,
            &condition_metrics,
        )?;
        let recommendations = build_recommendations(
            student_id,
            &topic_results,
            &skill_results,
            &condition_metrics,
            learning_profile.as_ref(),
        );
        let audience_reports = build_audience_reports(
            &overall_summary,
            &topic_results,
            &recommendations,
            &condition_metrics,
        );
        self.persist_deep_diagnostic_outputs(
            diagnostic_id,
            student_id,
            subject_id,
            &session_scores,
            &topic_results,
            &skill_results,
            &condition_metrics,
            learning_profile.as_ref(),
            &recommendations,
            &audience_reports,
        )?;
        let longitudinal_summary = build_longitudinal_summary(
            previous_context.as_ref(),
            overall_readiness,
            &topic_results,
        );
        let result = DiagnosticResult {
            overall_readiness,
            readiness_band,
            topic_results,
            recommended_next_actions: unique_actions(recommended_next_actions),
            overall_summary,
            session_scores,
            condition_metrics,
            skill_results,
            recommendations,
            learning_profile,
            audience_reports,
            longitudinal_summary,
            problem_cause_fix_cards: Vec::new(),
            intervention_prescriptions: Vec::new(),
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
                    "trend": &summary.trend,
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
        let mut result = serde_json::from_str::<DiagnosticResult>(&result_json)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        result.problem_cause_fix_cards = self.list_problem_cause_fix_cards(diagnostic_id)?;
        result.intervention_prescriptions = self.list_intervention_prescriptions(diagnostic_id)?;
        Ok(Some(result))
    }

    pub fn get_subject_blueprint(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Option<DiagnosticSubjectBlueprint>> {
        self.conn
            .query_row(
                "SELECT subject_id, blueprint_code, subject_name, session_modes_json,
                        stage_rules_json, item_family_mix_json, routing_contract_json,
                        report_contract_json
                 FROM diagnostic_subject_blueprints
                 WHERE subject_id = ?1",
                [subject_id],
                |row| {
                    let session_modes_json: String = row.get(3)?;
                    let stage_rules_json: String = row.get(4)?;
                    let item_family_mix_json: String = row.get(5)?;
                    let routing_contract_json: String = row.get(6)?;
                    let report_contract_json: String = row.get(7)?;
                    Ok(DiagnosticSubjectBlueprint {
                        subject_id: row.get(0)?,
                        blueprint_code: row.get(1)?,
                        subject_name: row.get(2)?,
                        session_modes: serde_json::from_str(&session_modes_json)
                            .unwrap_or_else(|_| json!({})),
                        stage_rules: serde_json::from_str(&stage_rules_json)
                            .unwrap_or_else(|_| json!({})),
                        item_family_mix: serde_json::from_str(&item_family_mix_json)
                            .unwrap_or_default(),
                        routing_contract: serde_json::from_str(&routing_contract_json)
                            .unwrap_or_else(|_| json!({})),
                        report_contract: serde_json::from_str(&report_contract_json)
                            .unwrap_or_else(|_| json!({})),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn list_item_routing_profiles(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticItemRoutingProfile>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT dirp.question_id, dirp.subject_id, dirp.topic_id, dirp.family_id,
                        dirp.item_family, dirp.recognition_suitable, dirp.recall_suitable,
                        dirp.transfer_suitable, dirp.timed_suitable, dirp.confidence_prompt,
                        dirp.recommended_stages_json, dirp.sibling_variant_modes_json,
                        dirp.routing_notes_json
                 FROM diagnostic_item_routing_profiles dirp
                 INNER JOIN diagnostic_item_attempts dia ON dia.question_id = dirp.question_id
                 WHERE dia.diagnostic_id = ?1
                 GROUP BY dirp.question_id, dirp.subject_id, dirp.topic_id, dirp.family_id,
                          dirp.item_family, dirp.recognition_suitable, dirp.recall_suitable,
                          dirp.transfer_suitable, dirp.timed_suitable, dirp.confidence_prompt,
                          dirp.recommended_stages_json, dirp.sibling_variant_modes_json,
                          dirp.routing_notes_json
                 ORDER BY dirp.topic_id ASC, dirp.question_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let recommended_stages_json: String = row.get(10)?;
                let sibling_variant_modes_json: String = row.get(11)?;
                let routing_notes_json: String = row.get(12)?;
                Ok(DiagnosticItemRoutingProfile {
                    question_id: row.get(0)?,
                    subject_id: row.get(1)?,
                    topic_id: row.get(2)?,
                    family_id: row.get(3)?,
                    item_family: row.get(4)?,
                    recognition_suitable: row.get::<_, i64>(5)? == 1,
                    recall_suitable: row.get::<_, i64>(6)? == 1,
                    transfer_suitable: row.get::<_, i64>(7)? == 1,
                    timed_suitable: row.get::<_, i64>(8)? == 1,
                    confidence_prompt: row.get(9)?,
                    recommended_stages: serde_json::from_str(&recommended_stages_json)
                        .unwrap_or_default(),
                    sibling_variant_modes: serde_json::from_str(&sibling_variant_modes_json)
                        .unwrap_or_default(),
                    routing_notes: serde_json::from_str(&routing_notes_json)
                        .unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(profiles)
    }

    pub fn list_problem_cause_fix_cards(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticProblemCauseFixCard>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, topic_name, problem_summary, cause_summary, fix_summary,
                        confidence_score_bp, impact_score_bp, unlock_summary, evidence_json
                 FROM diagnostic_problem_cause_fix_cards
                 WHERE diagnostic_id = ?1
                 ORDER BY impact_score_bp DESC, confidence_score_bp DESC, topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let evidence_json: String = row.get(8)?;
                Ok(DiagnosticProblemCauseFixCard {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    problem_summary: row.get(2)?,
                    cause_summary: row.get(3)?,
                    fix_summary: row.get(4)?,
                    confidence_score: row.get(5)?,
                    impact_score: row.get(6)?,
                    unlock_summary: row.get(7)?,
                    evidence: serde_json::from_str(&evidence_json).unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut cards = Vec::new();
        for row in rows {
            cards.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(cards)
    }

    pub fn list_intervention_prescriptions(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticInterventionPrescription>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, topic_name, primary_mode_code, support_mode_code,
                        recheck_mode_code, mode_chain_json, contraindications_json,
                        success_signals_json, confidence_score_bp, payload_json
                 FROM diagnostic_intervention_prescriptions
                 WHERE diagnostic_id = ?1
                 ORDER BY confidence_score_bp DESC, topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let mode_chain_json: String = row.get(5)?;
                let contraindications_json: String = row.get(6)?;
                let success_signals_json: String = row.get(7)?;
                let payload_json: String = row.get(9)?;
                Ok(DiagnosticInterventionPrescription {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    primary_mode_code: row.get(2)?,
                    support_mode_code: row.get(3)?,
                    recheck_mode_code: row.get(4)?,
                    mode_chain: serde_json::from_str(&mode_chain_json).unwrap_or_default(),
                    contraindications: serde_json::from_str(&contraindications_json)
                        .unwrap_or_default(),
                    success_signals: serde_json::from_str(&success_signals_json)
                        .unwrap_or_default(),
                    confidence_score: row.get(8)?,
                    payload: serde_json::from_str(&payload_json).unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut prescriptions = Vec::new();
        for row in rows {
            prescriptions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(prescriptions)
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
                        dta.recommended_action, dta.endurance_score_bp,
                        dta.error_distribution_json, dta.weakness_type, dta.failure_stage
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
                    endurance_score: row.get::<_, Option<BasisPoints>>(12)?.unwrap_or(0),
                    error_distribution: row
                        .get::<_, Option<String>>(13)?
                        .map(|value| serde_json::from_str::<Value>(&value))
                        .transpose()
                        .map_err(|err| {
                            rusqlite::Error::FromSqlConversionFailure(
                                13,
                                rusqlite::types::Type::Text,
                                Box::new(err),
                            )
                        })?
                        .unwrap_or_else(|| json!({})),
                    weakness_type: row.get(14)?,
                    failure_stage: row.get(15)?,
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

    pub fn list_skill_results(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticSkillResult>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT dsr.skill_key, dsr.skill_name, dsr.skill_type, dsr.topic_id, t.name,
                        dsr.baseline_score, dsr.speed_score, dsr.precision_score,
                        dsr.pressure_score, dsr.flex_score, dsr.root_cause_score,
                        dsr.endurance_score, dsr.recovery_score, dsr.mastery_score,
                        dsr.fragility_index, dsr.pressure_collapse_index,
                        dsr.recognition_gap_index, dsr.formula_recall_use_delta,
                        dsr.stability_score, dsr.mastery_state, dsr.weakness_type_primary,
                        dsr.weakness_type_secondary, dsr.recommended_intervention,
                        dsr.evidence_json
                 FROM diagnostic_skill_results dsr
                 INNER JOIN topics t ON t.id = dsr.topic_id
                 WHERE dsr.diagnostic_id = ?1
                 ORDER BY dsr.mastery_score ASC, dsr.fragility_index DESC, dsr.skill_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let evidence_json: String = row.get(23)?;
                let evidence = serde_json::from_str::<Value>(&evidence_json).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        23,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })?;
                Ok(DiagnosticSkillResult {
                    skill_key: row.get(0)?,
                    skill_name: row.get(1)?,
                    skill_type: row.get(2)?,
                    topic_id: row.get(3)?,
                    topic_name: row.get(4)?,
                    baseline_score: row.get(5)?,
                    speed_score: row.get(6)?,
                    precision_score: row.get(7)?,
                    pressure_score: row.get(8)?,
                    flex_score: row.get(9)?,
                    root_cause_score: row.get(10)?,
                    endurance_score: row.get(11)?,
                    recovery_score: row.get(12)?,
                    mastery_score: row.get(13)?,
                    fragility_index: row.get(14)?,
                    pressure_collapse_index: row.get(15)?,
                    recognition_gap_index: row.get(16)?,
                    formula_recall_use_delta: row.get(17)?,
                    stability_score: row.get(18)?,
                    mastery_state: row.get(19)?,
                    weakness_type_primary: row.get(20)?,
                    weakness_type_secondary: row.get(21)?,
                    recommended_intervention: row.get(22)?,
                    evidence,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(results)
    }

    pub fn list_recommendations(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticRecommendation>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT category, action_code, title, rationale, priority,
                        target_kind, target_ref
                 FROM diagnostic_recommendations
                 WHERE diagnostic_id = ?1
                 ORDER BY priority DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                Ok(DiagnosticRecommendation {
                    category: row.get(0)?,
                    action_code: row.get(1)?,
                    title: row.get(2)?,
                    rationale: row.get(3)?,
                    priority: row.get(4)?,
                    target_kind: row.get(5)?,
                    target_ref: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(results)
    }

    pub fn get_audience_report(
        &self,
        diagnostic_id: i64,
        audience: &str,
    ) -> EcoachResult<Option<DiagnosticAudienceReport>> {
        self.conn
            .query_row(
                "SELECT audience, headline, narrative, payload_json
                 FROM diagnostic_audience_reports
                 WHERE diagnostic_id = ?1 AND audience = ?2",
                params![diagnostic_id, audience],
                |row| {
                    let payload_json: String = row.get(3)?;
                    let payload = serde_json::from_str::<Value>(&payload_json).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;
                    Ok(DiagnosticAudienceReport {
                        audience: row.get(0)?,
                        headline: row.get(1)?,
                        narrative: row.get(2)?,
                        strengths: parse_report_list(&payload, "strengths"),
                        fragile_areas: parse_report_list(&payload, "fragile_areas"),
                        critical_areas: parse_report_list(&payload, "critical_areas"),
                        action_plan: parse_report_list(&payload, "action_plan"),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn ensure_diagnostic_group_shadow(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        subject_id: i64,
        mode: DiagnosticMode,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO diagnostic_session_groups (
                    id, student_id, subject_id, group_type, status, stages_completed_json,
                    comparison_deltas_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, 'in_progress', '{}', '{}', datetime('now'))
                 ON CONFLICT(id) DO UPDATE SET
                    student_id = excluded.student_id,
                    subject_id = excluded.subject_id,
                    group_type = excluded.group_type,
                    status = 'in_progress'",
                params![
                    diagnostic_id,
                    student_id,
                    subject_id,
                    diagnostic_group_type(mode),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn persist_phase_summary(
        &self,
        diagnostic_id: i64,
        completed_phase_number: i64,
    ) -> EcoachResult<()> {
        let phase = self
            .list_diagnostic_phases(diagnostic_id)?
            .into_iter()
            .find(|phase| phase.phase_number == completed_phase_number)
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "diagnostic {} missing phase {}",
                    diagnostic_id, completed_phase_number
                ))
            })?;
        let score = self.compute_session_score_for_phase(diagnostic_id, &phase)?;
        let phase_result_json = serde_json::to_string(&score)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE diagnostic_session_phases
                 SET phase_result_json = ?1
                 WHERE id = ?2",
                params![phase_result_json, phase.phase_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM diagnostic_phase_results
                 WHERE group_id = ?1 AND phase_type = ?2",
                params![diagnostic_id, deep_phase_type_from_code(&phase.phase_code)],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO diagnostic_phase_results (
                    group_id, phase_type, session_id, accuracy_bp, fluency_bp, precision_bp,
                    pressure_bp, flexibility_bp, stability_bp, early_segment_accuracy_bp,
                    middle_segment_accuracy_bp, final_segment_accuracy_bp, confidence_capture_json
                 ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    diagnostic_id,
                    deep_phase_type_from_code(&phase.phase_code),
                    score.raw_accuracy,
                    if phase.phase_code == "speed" {
                        score.adjusted_accuracy
                    } else {
                        0
                    },
                    if phase.phase_code == "precision" {
                        score.adjusted_accuracy
                    } else {
                        0
                    },
                    if phase.phase_code == "pressure" {
                        score.adjusted_accuracy
                    } else {
                        0
                    },
                    if phase.phase_code == "flex" {
                        score.adjusted_accuracy
                    } else {
                        0
                    },
                    score.stability_measure,
                    score.early_segment_accuracy,
                    score.middle_segment_accuracy,
                    score.final_segment_accuracy,
                    "{}",
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.sync_group_stage_progress(diagnostic_id)?;
        Ok(())
    }

    fn complete_diagnostic_group_shadow(&self, diagnostic_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE diagnostic_session_groups
                 SET status = 'completed', completed_at = datetime('now')
                 WHERE id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn sync_group_stage_progress(&self, diagnostic_id: i64) -> EcoachResult<()> {
        let stages = self
            .list_diagnostic_phases(diagnostic_id)?
            .into_iter()
            .filter(|phase| phase.status == "completed")
            .map(|phase| (phase.phase_code, json!(true)))
            .collect::<Map<String, Value>>();
        self.conn
            .execute(
                "UPDATE diagnostic_session_groups
                 SET stages_completed_json = ?1
                 WHERE id = ?2",
                params![
                    serde_json::to_string(&stages)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    diagnostic_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn compute_session_scores(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<DiagnosticSessionScore>> {
        let phases = self.list_diagnostic_phases(diagnostic_id)?;
        let mut scores = Vec::new();
        for phase in phases {
            scores.push(self.compute_session_score_for_phase(diagnostic_id, &phase)?);
        }
        Ok(scores)
    }

    fn compute_session_score_for_phase(
        &self,
        diagnostic_id: i64,
        phase: &DiagnosticPhasePlan,
    ) -> EcoachResult<DiagnosticSessionScore> {
        let attempts = self.load_phase_attempt_snapshots(phase.phase_id)?;
        let answered = attempts
            .iter()
            .filter(|item| item.is_correct.is_some())
            .count();
        let correct = attempts
            .iter()
            .filter(|item| item.is_correct == Some(true))
            .count();
        let timeout_count = attempts.iter().filter(|item| item.timed_out).count();
        let careless_count = attempts.iter().filter(|item| item.looks_careless()).count();
        let misread_count = attempts.iter().filter(|item| item.looks_misread()).count();
        let median_response_time_ms = median_i64(
            attempts
                .iter()
                .filter_map(|item| item.response_time_ms)
                .collect::<Vec<_>>(),
        );
        let segment_scores = segment_accuracy_profile(&attempts);
        let raw_accuracy = accuracy_bp(correct, answered);
        let timeout_rate = accuracy_bp(timeout_count, attempts.len());
        let careless_error_rate = accuracy_bp(careless_count, attempts.len());
        let misread_rate = accuracy_bp(misread_count, attempts.len());
        let pressure_volatility =
            segment_volatility_bp(segment_scores.0, segment_scores.1, segment_scores.2);
        let adjusted_accuracy = raw_accuracy
            .saturating_sub(timeout_rate / 3)
            .saturating_sub(careless_error_rate / 4)
            .saturating_sub(misread_rate / 5);
        let stability_measure = 10_000u16.saturating_sub(pressure_volatility);

        if phase.status == "completed" {
            let phase_result_json = serde_json::to_string(&json!({
                "raw_accuracy": raw_accuracy,
                "adjusted_accuracy": adjusted_accuracy,
                "median_response_time_ms": median_response_time_ms,
                "stability_measure": stability_measure,
                "careless_error_rate": careless_error_rate,
                "timeout_rate": timeout_rate,
                "misread_rate": misread_rate,
                "pressure_volatility": pressure_volatility,
                "early_segment_accuracy": segment_scores.0,
                "middle_segment_accuracy": segment_scores.1,
                "final_segment_accuracy": segment_scores.2,
            }))
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
            self.conn
                .execute(
                    "UPDATE diagnostic_session_phases
                     SET phase_result_json = ?1
                     WHERE diagnostic_id = ?2 AND id = ?3",
                    params![phase_result_json, diagnostic_id, phase.phase_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(DiagnosticSessionScore {
            phase_code: phase.phase_code.clone(),
            phase_title: phase.phase_title.clone(),
            raw_accuracy,
            adjusted_accuracy,
            median_response_time_ms,
            stability_measure,
            careless_error_rate,
            timeout_rate,
            misread_rate,
            pressure_volatility,
            early_segment_accuracy: segment_scores.0,
            middle_segment_accuracy: segment_scores.1,
            final_segment_accuracy: segment_scores.2,
        })
    }

    fn load_phase_attempt_snapshots(
        &self,
        phase_id: i64,
    ) -> EcoachResult<Vec<PhaseAttemptSnapshot>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT dia.display_order, dia.is_correct, dia.response_time_ms,
                        COALESCE(dia.changed_answer_count, 0), COALESCE(dia.skipped, 0),
                        COALESCE(dia.timed_out, 0), COALESCE(q.estimated_time_seconds, 30),
                        qo.distractor_intent, dia.confidence_level
                 FROM diagnostic_item_attempts dia
                 INNER JOIN questions q ON q.id = dia.question_id
                 LEFT JOIN question_options qo ON qo.id = dia.selected_option_id
                 WHERE dia.phase_id = ?1
                 ORDER BY dia.display_order ASC, dia.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([phase_id], |row| {
                Ok(PhaseAttemptSnapshot {
                    display_order: row.get(0)?,
                    is_correct: row.get::<_, Option<i64>>(1)?.map(|value| value == 1),
                    response_time_ms: row.get(2)?,
                    changed_answer_count: row.get(3)?,
                    skipped: row.get::<_, i64>(4)? == 1,
                    timed_out: row.get::<_, i64>(5)? == 1,
                    estimated_time_ms: row.get::<_, i64>(6)? * 1_000,
                    distractor_intent: row.get(7)?,
                    confidence_level: row.get(8)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut attempts = Vec::new();
        for row in rows {
            attempts.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(attempts)
    }

    fn compute_skill_results(
        &self,
        diagnostic_id: i64,
        student_id: i64,
    ) -> EcoachResult<Vec<DiagnosticSkillResult>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    CASE
                        WHEN q.micro_skill_id IS NOT NULL THEN 'micro_skill'
                        WHEN q.primary_skill_id IS NOT NULL THEN 'academic_node'
                        ELSE 'derived'
                    END,
                    COALESCE(q.micro_skill_id, q.primary_skill_id, q.id),
                    CASE
                        WHEN q.micro_skill_id IS NOT NULL THEN 'micro:' || q.micro_skill_id
                        WHEN q.primary_skill_id IS NOT NULL THEN 'node:' || q.primary_skill_id
                        ELSE 'question:' || q.id
                    END,
                    COALESCE(ms.skill_name, an.short_label, an.canonical_title,
                             q.primary_knowledge_role, q.primary_cognitive_demand, 'diagnostic_skill'),
                    COALESCE(q.primary_knowledge_role, an.node_type, q.primary_cognitive_demand, 'derived'),
                    q.topic_id,
                    t.name,
                    COALESCE(dsp.phase_code, lower(replace(dsp.phase_type, ' ', '_'))),
                    dia.is_correct,
                    q.family_id
                 FROM diagnostic_item_attempts dia
                 INNER JOIN diagnostic_session_phases dsp ON dsp.id = dia.phase_id
                 INNER JOIN questions q ON q.id = dia.question_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 LEFT JOIN micro_skills ms ON ms.id = q.micro_skill_id
                 LEFT JOIN academic_nodes an ON an.id = q.primary_skill_id
                 WHERE dia.diagnostic_id = ?1
                   AND dia.is_correct IS NOT NULL
                 ORDER BY q.topic_id ASC, skill_name ASC, dsp.phase_number ASC, dia.display_order ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                Ok(SkillAttemptSnapshot {
                    skill_kind: row.get(0)?,
                    skill_id: row.get(1)?,
                    skill_key: row.get(2)?,
                    skill_name: row.get(3)?,
                    skill_type: row.get(4)?,
                    topic_id: row.get(5)?,
                    topic_name: row.get(6)?,
                    phase_code: row.get(7)?,
                    is_correct: row.get::<_, Option<i64>>(8)?.unwrap_or(0) == 1,
                    family_id: row.get(9)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut grouped: std::collections::BTreeMap<String, SkillAggregate> =
            std::collections::BTreeMap::new();
        for row in rows {
            let item = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            grouped
                .entry(item.skill_key.clone())
                .and_modify(|aggregate| aggregate.push(item.clone()))
                .or_insert_with(|| SkillAggregate::from_attempt(item));
        }

        let mut results = grouped
            .into_values()
            .map(|aggregate| aggregate.into_result(student_id))
            .collect::<Vec<_>>();
        results.sort_by(|left, right| {
            left.mastery_score
                .cmp(&right.mastery_score)
                .then_with(|| right.fragility_index.cmp(&left.fragility_index))
                .then_with(|| left.skill_name.cmp(&right.skill_name))
        });
        Ok(results)
    }

    fn build_condition_metrics(
        &self,
        topic_results: &[TopicDiagnosticResult],
        session_scores: &[DiagnosticSessionScore],
        skill_results: &[DiagnosticSkillResult],
    ) -> DiagnosticConditionMetrics {
        let fragility_index = average_bp(
            skill_results
                .iter()
                .map(|item| item.fragility_index)
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|| {
            average_bp(
                topic_results
                    .iter()
                    .map(|item| item.mastery_score.saturating_sub(item.flexibility_score))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or(0)
        });
        let pressure_collapse_index = average_bp(
            skill_results
                .iter()
                .map(|item| item.pressure_collapse_index)
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|| {
            average_bp(
                topic_results
                    .iter()
                    .map(|item| item.mastery_score.saturating_sub(item.pressure_score))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or(0)
        });
        let recognition_gap_index = average_bp(
            skill_results
                .iter()
                .map(|item| item.recognition_gap_index)
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|| {
            average_bp(
                topic_results
                    .iter()
                    .map(|item| item.mastery_score.saturating_sub(item.flexibility_score))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or(0)
        });
        let formula_recall_use_delta = average_bp(
            skill_results
                .iter()
                .filter(|item| item.formula_recall_use_delta > 0)
                .map(|item| item.formula_recall_use_delta)
                .collect::<Vec<_>>(),
        )
        .unwrap_or(0);
        let endurance_drop = session_scores
            .iter()
            .find(|score| score.phase_code == "endurance")
            .map(|score| {
                score
                    .early_segment_accuracy
                    .unwrap_or(score.raw_accuracy)
                    .saturating_sub(score.final_segment_accuracy.unwrap_or(score.raw_accuracy))
            })
            .unwrap_or(0);
        let early_late_delta = endurance_drop;
        let confidence_correctness_delta = average_bp(
            topic_results
                .iter()
                .map(|item| item.mastery_score.saturating_sub(item.stability_score))
                .collect::<Vec<_>>(),
        )
        .unwrap_or(0);

        DiagnosticConditionMetrics {
            fragility_index,
            pressure_collapse_index,
            recognition_gap_index,
            formula_recall_use_delta,
            early_late_delta,
            confidence_correctness_delta,
            endurance_drop,
        }
    }

    fn build_learning_profile(
        &self,
        diagnostic_id: i64,
        _student_id: i64,
        _subject_id: i64,
        topic_results: &[TopicDiagnosticResult],
        skill_results: &[DiagnosticSkillResult],
        condition_metrics: &DiagnosticConditionMetrics,
    ) -> EcoachResult<Option<DiagnosticLearningProfile>> {
        if topic_results.is_empty() && skill_results.is_empty() {
            return Ok(None);
        }
        let profile_type = classify_learning_profile(skill_results, condition_metrics).to_string();
        let confidence_score =
            10_000u16.saturating_sub(condition_metrics.confidence_correctness_delta);
        Ok(Some(DiagnosticLearningProfile {
            profile_type,
            confidence_score,
            evidence: json!({
                "diagnostic_id": diagnostic_id,
                "topic_count": topic_results.len(),
                "skill_count": skill_results.len(),
                "fragility_index": condition_metrics.fragility_index,
                "pressure_collapse_index": condition_metrics.pressure_collapse_index,
                "recognition_gap_index": condition_metrics.recognition_gap_index,
                "formula_recall_use_delta": condition_metrics.formula_recall_use_delta,
            }),
        }))
    }

    #[allow(clippy::too_many_arguments)]
    fn persist_deep_diagnostic_outputs(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        subject_id: i64,
        session_scores: &[DiagnosticSessionScore],
        topic_results: &[TopicDiagnosticResult],
        skill_results: &[DiagnosticSkillResult],
        condition_metrics: &DiagnosticConditionMetrics,
        learning_profile: Option<&DiagnosticLearningProfile>,
        recommendations: &[DiagnosticRecommendation],
        audience_reports: &[DiagnosticAudienceReport],
    ) -> EcoachResult<()> {
        let stages_completed = session_scores
            .iter()
            .map(|score| (score.phase_code.clone(), json!(true)))
            .collect::<Map<String, Value>>();
        let comparison_deltas_json = serde_json::to_string(&json!({
            "fragility_index": condition_metrics.fragility_index,
            "pressure_collapse_index": condition_metrics.pressure_collapse_index,
            "recognition_gap_index": condition_metrics.recognition_gap_index,
            "formula_recall_use_delta": condition_metrics.formula_recall_use_delta,
            "early_late_delta": condition_metrics.early_late_delta,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE diagnostic_session_groups
                 SET status = 'completed',
                     stages_completed_json = ?1,
                     profile_type = ?2,
                     comparison_deltas_json = ?3,
                     completed_at = datetime('now')
                 WHERE id = ?4",
                params![
                    serde_json::to_string(&stages_completed)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    learning_profile.map(|profile| profile.profile_type.as_str()),
                    comparison_deltas_json,
                    diagnostic_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "DELETE FROM diagnostic_phase_results WHERE group_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for score in session_scores {
            self.conn
                .execute(
                    "INSERT INTO diagnostic_phase_results (
                        group_id, phase_type, session_id, accuracy_bp, fluency_bp, precision_bp,
                        pressure_bp, flexibility_bp, stability_bp, early_segment_accuracy_bp,
                        middle_segment_accuracy_bp, final_segment_accuracy_bp, confidence_capture_json
                     ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        diagnostic_id,
                        deep_phase_type_from_code(&score.phase_code),
                        score.raw_accuracy,
                        if score.phase_code == "speed" { score.adjusted_accuracy } else { 0 },
                        if score.phase_code == "precision" { score.adjusted_accuracy } else { 0 },
                        if score.phase_code == "pressure" { score.adjusted_accuracy } else { 0 },
                        if score.phase_code == "flex" { score.adjusted_accuracy } else { 0 },
                        score.stability_measure,
                        score.early_segment_accuracy,
                        score.middle_segment_accuracy,
                        score.final_segment_accuracy,
                        "{}",
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "DELETE FROM diagnostic_deltas WHERE group_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO diagnostic_deltas (
                    group_id, student_id, topic_id, speed_accuracy_delta_bp, calm_pressure_delta_bp,
                    direct_variant_delta_bp, recall_application_delta_bp, formula_recall_use_delta_bp,
                    early_late_delta_bp
                 ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    diagnostic_id,
                    student_id,
                    session_delta(session_scores, "baseline", "speed"),
                    condition_metrics.pressure_collapse_index,
                    condition_metrics.recognition_gap_index,
                    condition_metrics.recognition_gap_index,
                    condition_metrics.formula_recall_use_delta,
                    condition_metrics.early_late_delta,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "DELETE FROM diagnostic_learning_dimensions WHERE group_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO diagnostic_learning_dimensions (
                    group_id, student_id, coverage_bp, accuracy_bp, recall_strength_bp,
                    recognition_vs_production_bp, reasoning_depth_bp, misconception_density_bp,
                    speed_bp, pressure_response_bp, transfer_ability_bp, stability_bp,
                    confidence_calibration_bp, fatigue_pattern_bp
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    diagnostic_id,
                    student_id,
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.mastery_score)
                            .collect()
                    )
                    .unwrap_or(0),
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.precision_score)
                            .collect()
                    )
                    .unwrap_or(0),
                    session_score_value(session_scores, "baseline"),
                    10_000i64 - condition_metrics.recognition_gap_index as i64,
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.precision_score)
                            .collect()
                    )
                    .unwrap_or(0),
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.endurance_score.saturating_sub(item.mastery_score))
                            .collect()
                    )
                    .unwrap_or(0),
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.fluency_score)
                            .collect()
                    )
                    .unwrap_or(0),
                    10_000i64 - condition_metrics.pressure_collapse_index as i64,
                    10_000i64 - condition_metrics.recognition_gap_index as i64,
                    average_bp(
                        topic_results
                            .iter()
                            .map(|item| item.stability_score)
                            .collect()
                    )
                    .unwrap_or(0),
                    10_000i64 - condition_metrics.confidence_correctness_delta as i64,
                    condition_metrics.endurance_drop,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "DELETE FROM diagnostic_skill_results WHERE diagnostic_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for skill in skill_results {
            self.conn
                .execute(
                    "INSERT INTO diagnostic_skill_results (
                        diagnostic_id, student_id, skill_kind, skill_id, skill_key, skill_name,
                        skill_type, topic_id, baseline_score, speed_score, precision_score,
                        pressure_score, flex_score, root_cause_score, endurance_score, recovery_score,
                        mastery_score, fragility_index, pressure_collapse_index,
                        recognition_gap_index, formula_recall_use_delta, stability_score,
                        mastery_state, weakness_type_primary, weakness_type_secondary,
                        recommended_intervention, evidence_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
                    params![
                        diagnostic_id,
                        student_id,
                        skill_kind_from_key(&skill.skill_key),
                        skill_id_from_key(&skill.skill_key),
                        skill.skill_key,
                        skill.skill_name,
                        skill.skill_type,
                        skill.topic_id,
                        skill.baseline_score,
                        skill.speed_score,
                        skill.precision_score,
                        skill.pressure_score,
                        skill.flex_score,
                        skill.root_cause_score,
                        skill.endurance_score,
                        skill.recovery_score,
                        skill.mastery_score,
                        skill.fragility_index,
                        skill.pressure_collapse_index,
                        skill.recognition_gap_index,
                        skill.formula_recall_use_delta,
                        skill.stability_score,
                        skill.mastery_state,
                        skill.weakness_type_primary,
                        skill.weakness_type_secondary,
                        skill.recommended_intervention,
                        serde_json::to_string(&skill.evidence)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "DELETE FROM diagnostic_recommendations WHERE diagnostic_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for recommendation in recommendations {
            self.conn
                .execute(
                    "INSERT INTO diagnostic_recommendations (
                        diagnostic_id, student_id, category, action_code, title, rationale,
                        priority, target_kind, target_ref, payload_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '{}')",
                    params![
                        diagnostic_id,
                        student_id,
                        recommendation.category,
                        recommendation.action_code,
                        recommendation.title,
                        recommendation.rationale,
                        recommendation.priority,
                        recommendation.target_kind,
                        recommendation.target_ref,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "DELETE FROM diagnostic_audience_reports WHERE diagnostic_id = ?1",
                [diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for report in audience_reports {
            let payload_json = serde_json::to_string(&json!({
                "strengths": report.strengths,
                "fragile_areas": report.fragile_areas,
                "critical_areas": report.critical_areas,
                "action_plan": report.action_plan,
            }))
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
            self.conn
                .execute(
                    "INSERT INTO diagnostic_audience_reports (
                        diagnostic_id, audience, headline, narrative, payload_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        diagnostic_id,
                        report.audience,
                        report.headline,
                        report.narrative,
                        payload_json,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        if let Some(profile) = learning_profile {
            self.conn
                .execute(
                    "INSERT INTO student_learning_profiles (
                        student_id, subject_id, profile_type, confidence_bp, evidence_json,
                        diagnostic_group_id, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
                     ON CONFLICT(student_id, subject_id) DO UPDATE SET
                        profile_type = excluded.profile_type,
                        confidence_bp = excluded.confidence_bp,
                        evidence_json = excluded.evidence_json,
                        diagnostic_group_id = excluded.diagnostic_group_id,
                        updated_at = datetime('now')",
                    params![
                        student_id,
                        subject_id,
                        profile.profile_type,
                        profile.confidence_score,
                        serde_json::to_string(&profile.evidence)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        diagnostic_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
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
        let endurance = self.topic_phase_accuracy(diagnostic_id, topic_id, "endurance")?;
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
        let endurance_score = if endurance > 0 {
            endurance
        } else {
            stability_score
        };
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
        let weakness_type = Some(primary_weakness_type(
            mastery_score,
            fluency_score,
            pressure_score,
            flexibility_score,
            endurance_score,
            &error_profile,
        ));
        let failure_stage = Some(primary_failure_stage(&error_profile).to_string());
        let error_distribution = json!({
            "knowledge_gap": error_profile.knowledge_gap_score,
            "conceptual_confusion": error_profile.conceptual_confusion_score,
            "recognition_failure": error_profile.recognition_failure_score,
            "pressure_breakdown": error_profile.pressure_breakdown_score,
            "speed_error": error_profile.speed_error_score,
            "misconception_signals": error_profile.misconception_signal_count,
            "high_confidence_wrong": error_profile.high_confidence_wrong_count,
            "low_confidence_correct": error_profile.low_confidence_correct_count,
        });

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
            endurance_score,
            error_distribution,
            weakness_type,
            failure_stage,
        })
    }

    fn persist_topic_analytics(&self, analytics: &DiagnosticTopicAnalytics) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO diagnostic_topic_analytics (
                    diagnostic_id, topic_id, mastery_score, fluency_score, precision_score,
                    pressure_score, flexibility_score, stability_score, classification,
                    confidence_score, recommended_action, endurance_score_bp,
                    error_distribution_json, weakness_type, failure_stage
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
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
                    endurance_score_bp = excluded.endurance_score_bp,
                    error_distribution_json = excluded.error_distribution_json,
                    weakness_type = excluded.weakness_type,
                    failure_stage = excluded.failure_stage,
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
                    analytics.endurance_score,
                    serde_json::to_string(&analytics.error_distribution)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    analytics.weakness_type,
                    analytics.failure_stage,
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
                .or_else(|| {
                    self.load_diagnostic_overall_readiness(diagnostic_id)
                        .ok()
                        .flatten()
                })
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
            _diagnostic_id,
            _topic_id,
            _topic_name,
            mastery_score,
            pressure_score,
            flexibility_score,
            classification,
        )) = row
        else {
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
                |row| {
                    Ok((
                        Some(row.get::<_, String>(0)?),
                        Some(row.get::<_, BasisPoints>(1)?),
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((None, None));

        Ok(Some(HistoricalTopicSnapshot {
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
                   AND dia.is_correct IS NOT NULL",
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
            diagnostic_stage: Some(template.code.as_str().to_string()),
            condition_type: Some(template.code.condition_type().to_string()),
            require_confidence_prompt: matches!(
                template.code,
                DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery
            ),
            require_concept_guess_prompt: matches!(
                template.code,
                DiagnosticPhaseCode::Flex
                    | DiagnosticPhaseCode::RootCause
                    | DiagnosticPhaseCode::Recovery
            ),
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
            diagnostic_stage: Some(template.code.as_str().to_string()),
            condition_type: Some(template.code.condition_type().to_string()),
            require_confidence_prompt: matches!(
                template.code,
                DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery
            ),
            require_concept_guess_prompt: matches!(
                template.code,
                DiagnosticPhaseCode::Flex
                    | DiagnosticPhaseCode::RootCause
                    | DiagnosticPhaseCode::Recovery
            ),
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
            "confidence_prompt": matches!(template.code, DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery),
            "concept_guess_prompt": matches!(template.code, DiagnosticPhaseCode::Flex | DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery),
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
                DiagnosticPhaseCode::Recovery => 2,
                _ => 3,
            })
            .map(|(topic_id, _)| topic_id)
            .collect::<Vec<_>>();
        if !topic_ids.is_empty() {
            return Ok(topic_ids);
        }

        let fallback = match phase_code {
            DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery => self
                .load_root_cause_topic_ids(
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
            "confidence_prompt": matches!(template.code, DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery),
            "concept_guess_prompt": matches!(template.code, DiagnosticPhaseCode::Flex | DiagnosticPhaseCode::RootCause | DiagnosticPhaseCode::Recovery),
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
            self.upsert_item_routing_profile(&selected.question)?;
        }

        Ok(())
    }

    fn ensure_subject_blueprint(&self, subject_id: i64) -> EcoachResult<()> {
        let subject_name: String = self
            .conn
            .query_row(
                "SELECT name FROM subjects WHERE id = ?1",
                [subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT COALESCE(qip.primary_pedagogic_function, qip.primary_cognitive_demand, q.question_format) AS item_family,
                        COUNT(*) AS item_count
                 FROM questions q
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                 WHERE q.subject_id = ?1 AND q.is_active = 1
                 GROUP BY item_family
                 ORDER BY item_count DESC, item_family ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| {
                Ok(json!({
                    "item_family": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut item_family_mix = Vec::new();
        for row in rows {
            item_family_mix.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let build_mode_json = |mode: DiagnosticMode, duration_minutes: i64| {
            let phases = diagnostic_phase_templates(mode)
                .into_iter()
                .map(|template| {
                    json!({
                        "phase_code": template.code.as_str(),
                        "question_count": template.question_count,
                        "timed": template.timed,
                        "time_limit_seconds": template.time_limit_seconds,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "duration_minutes": duration_minutes,
                "phase_count": phases.len(),
                "phases": phases,
            })
        };

        let session_modes_json = json!({
            "quick": build_mode_json(DiagnosticMode::Quick, 35),
            "standard": build_mode_json(DiagnosticMode::Standard, 60),
            "deep": build_mode_json(DiagnosticMode::Deep, 90),
        });
        let stage_rules_json = json!({
            "baseline": "scan broadly across the subject before deepening",
            "adaptive_zoom": "retarget weak topics as evidence accumulates",
            "condition_testing": "compare calm, timed, and endurance conditions",
            "stability_recheck": "confirm whether success survives transfer and repetition",
            "confidence_snapshot": "capture confidence and recognition signals before final classification"
        });
        let routing_contract_json = json!({
            "confidence_prompt": "sure_not_sure_guessed",
            "supports_transfer_checks": true,
            "supports_timed_mirrors": true,
            "supports_root_cause_branching": true,
            "uses_question_intelligence": true,
        });
        let report_contract_json = json!({
            "sections": [
                "overall_dashboard",
                "condition_deltas",
                "topic_cards",
                "problem_cause_fix_cards",
                "intervention_prescriptions"
            ],
            "audiences": ["student", "parent", "teacher"],
            "exports": ["json", "printable_summary"]
        });

        self.conn
            .execute(
                "INSERT INTO diagnostic_subject_blueprints (
                    subject_id, blueprint_code, subject_name, session_modes_json,
                    stage_rules_json, item_family_mix_json, routing_contract_json,
                    report_contract_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
                 ON CONFLICT(subject_id) DO UPDATE SET
                    blueprint_code = excluded.blueprint_code,
                    subject_name = excluded.subject_name,
                    session_modes_json = excluded.session_modes_json,
                    stage_rules_json = excluded.stage_rules_json,
                    item_family_mix_json = excluded.item_family_mix_json,
                    routing_contract_json = excluded.routing_contract_json,
                    report_contract_json = excluded.report_contract_json,
                    updated_at = datetime('now')",
                params![
                    subject_id,
                    format!("dna_subject_{}", subject_id),
                    subject_name,
                    serde_json::to_string(&session_modes_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&stage_rules_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&item_family_mix)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&routing_contract_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&report_contract_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn ensure_routing_profiles_for_scope(
        &self,
        subject_id: i64,
        topic_ids: &[i64],
    ) -> EcoachResult<()> {
        if topic_ids.is_empty() {
            return Ok(());
        }

        let question_service = QuestionService::new(self.conn);
        for question in question_service.list_questions_for_scope(subject_id, topic_ids)? {
            self.upsert_item_routing_profile(&question)?;
        }
        Ok(())
    }

    fn upsert_item_routing_profile(&self, question: &Question) -> EcoachResult<()> {
        let question_service = QuestionService::new(self.conn);
        let intelligence = question_service.get_question_intelligence(question.id)?;
        let cognitive_demand = intelligence
            .as_ref()
            .and_then(|item| item.cognitive_demand.clone());
        let pedagogic_function = intelligence
            .as_ref()
            .and_then(|item| item.pedagogic_function.clone());
        let family_id = intelligence
            .as_ref()
            .and_then(|item| item.family.as_ref().and_then(|family| family.family_id));
        let item_family = pedagogic_function
            .clone()
            .or(cognitive_demand.clone())
            .unwrap_or_else(|| question.question_format.clone());
        let recognition_suitable = matches!(cognitive_demand.as_deref(), Some("recognition"))
            || matches!(
                question.question_format.as_str(),
                "mcq" | "true_false" | "matching"
            );
        let recall_suitable = matches!(
            cognitive_demand.as_deref(),
            Some("recall") | Some("reasoning") | Some("application")
        ) || intelligence
            .as_ref()
            .and_then(|item| item.knowledge_role.as_deref())
            == Some("formula_recall");
        let transfer_suitable = matches!(pedagogic_function.as_deref(), Some("transfer_check"))
            || matches!(
                cognitive_demand.as_deref(),
                Some("application") | Some("reasoning")
            );
        let timed_suitable = question.estimated_time_seconds <= 60 || recognition_suitable;

        let mut recommended_stages = vec!["baseline".to_string()];
        if timed_suitable {
            recommended_stages.push("speed".to_string());
            recommended_stages.push("pressure".to_string());
        }
        if transfer_suitable {
            recommended_stages.push("flex".to_string());
        }
        if intelligence
            .as_ref()
            .map(|item| !item.misconceptions.is_empty())
            .unwrap_or(false)
            || item_family.contains("misconception")
        {
            recommended_stages.push("root_cause".to_string());
        }
        let mut stage_seen = std::collections::BTreeSet::new();
        recommended_stages.retain(|item| stage_seen.insert(item.clone()));

        let mut sibling_variant_modes = vec!["isomorphic".to_string(), "rescue".to_string()];
        if transfer_suitable {
            sibling_variant_modes.push("representation_shift".to_string());
        }
        if intelligence
            .as_ref()
            .map(|item| !item.misconceptions.is_empty())
            .unwrap_or(false)
        {
            sibling_variant_modes.push("misconception_probe".to_string());
        }
        if !timed_suitable {
            sibling_variant_modes.push("stretch".to_string());
        }
        let mut mode_seen = std::collections::BTreeSet::new();
        sibling_variant_modes.retain(|item| mode_seen.insert(item.clone()));

        let routing_notes = json!({
            "question_format": question.question_format,
            "cognitive_demand": cognitive_demand,
            "pedagogic_function": pedagogic_function,
            "family_code": intelligence
                .as_ref()
                .and_then(|item| item.family.as_ref().and_then(|family| family.family_code.clone())),
            "misconception_codes": intelligence
                .as_ref()
                .map(|item| {
                    item.misconceptions
                        .iter()
                        .map(|misconception| misconception.misconception_code.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        });

        self.conn
            .execute(
                "INSERT INTO diagnostic_item_routing_profiles (
                    question_id, subject_id, topic_id, family_id, item_family,
                    recognition_suitable, recall_suitable, transfer_suitable, timed_suitable,
                    confidence_prompt, recommended_stages_json, sibling_variant_modes_json,
                    routing_notes_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))
                 ON CONFLICT(question_id) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    family_id = excluded.family_id,
                    item_family = excluded.item_family,
                    recognition_suitable = excluded.recognition_suitable,
                    recall_suitable = excluded.recall_suitable,
                    transfer_suitable = excluded.transfer_suitable,
                    timed_suitable = excluded.timed_suitable,
                    confidence_prompt = excluded.confidence_prompt,
                    recommended_stages_json = excluded.recommended_stages_json,
                    sibling_variant_modes_json = excluded.sibling_variant_modes_json,
                    routing_notes_json = excluded.routing_notes_json,
                    updated_at = datetime('now')",
                params![
                    question.id,
                    question.subject_id,
                    question.topic_id,
                    family_id,
                    item_family,
                    recognition_suitable as i64,
                    recall_suitable as i64,
                    transfer_suitable as i64,
                    timed_suitable as i64,
                    "sure_not_sure_guessed",
                    serde_json::to_string(&recommended_stages)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&sibling_variant_modes)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&routing_notes)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
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

#[derive(Debug)]
struct PhaseAttemptSnapshot {
    display_order: i64,
    is_correct: Option<bool>,
    response_time_ms: Option<i64>,
    changed_answer_count: i64,
    skipped: bool,
    timed_out: bool,
    estimated_time_ms: i64,
    distractor_intent: Option<String>,
    confidence_level: Option<String>,
}

impl PhaseAttemptSnapshot {
    fn looks_careless(&self) -> bool {
        self.is_correct == Some(false)
            && !self.timed_out
            && (self.changed_answer_count > 0
                || self
                    .response_time_ms
                    .map(|value| value < self.estimated_time_ms / 2)
                    .unwrap_or(false))
    }

    fn looks_misread(&self) -> bool {
        self.is_correct == Some(false)
            && self
                .distractor_intent
                .as_deref()
                .map(|intent| {
                    let normalized = intent.to_ascii_lowercase();
                    normalized.contains("misread")
                        || normalized.contains("sign")
                        || normalized.contains("unit")
                })
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
struct SkillAttemptSnapshot {
    skill_kind: String,
    skill_id: i64,
    skill_key: String,
    skill_name: String,
    skill_type: String,
    topic_id: i64,
    topic_name: String,
    phase_code: String,
    is_correct: bool,
    family_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct SkillAggregate {
    skill_kind: String,
    skill_id: i64,
    skill_key: String,
    skill_name: String,
    skill_type: String,
    topic_id: i64,
    topic_name: String,
    families: Vec<i64>,
    phase_totals: std::collections::BTreeMap<String, (usize, usize)>,
}

impl SkillAggregate {
    fn from_attempt(attempt: SkillAttemptSnapshot) -> Self {
        let mut aggregate = Self {
            skill_kind: attempt.skill_kind.clone(),
            skill_id: attempt.skill_id,
            skill_key: attempt.skill_key.clone(),
            skill_name: attempt.skill_name.clone(),
            skill_type: attempt.skill_type.clone(),
            topic_id: attempt.topic_id,
            topic_name: attempt.topic_name.clone(),
            families: attempt.family_id.into_iter().collect(),
            phase_totals: std::collections::BTreeMap::new(),
        };
        aggregate.push(attempt);
        aggregate
    }

    fn push(&mut self, attempt: SkillAttemptSnapshot) {
        if let Some(family_id) = attempt.family_id {
            if !self.families.contains(&family_id) {
                self.families.push(family_id);
            }
        }
        let entry = self
            .phase_totals
            .entry(attempt.phase_code)
            .or_insert((0usize, 0usize));
        entry.0 += 1;
        if attempt.is_correct {
            entry.1 += 1;
        }
    }

    fn into_result(self, _student_id: i64) -> DiagnosticSkillResult {
        let baseline_score = phase_accuracy_from_totals(&self.phase_totals, "baseline");
        let speed_score = phase_accuracy_from_totals(&self.phase_totals, "speed");
        let precision_score = phase_accuracy_from_totals(&self.phase_totals, "precision");
        let pressure_score = phase_accuracy_from_totals(&self.phase_totals, "pressure");
        let flex_score = phase_accuracy_from_totals(&self.phase_totals, "flex");
        let root_cause_score = phase_accuracy_from_totals(&self.phase_totals, "root_cause");
        let endurance_score = phase_accuracy_from_totals(&self.phase_totals, "endurance");
        let recovery_score = phase_accuracy_from_totals(&self.phase_totals, "recovery");
        let effective_precision = if precision_score > 0 {
            precision_score
        } else {
            baseline_score
        };
        let effective_flex = if flex_score > 0 {
            flex_score
        } else {
            baseline_score
        };
        let effective_pressure = if pressure_score > 0 {
            pressure_score
        } else {
            baseline_score
        };
        let effective_root_cause = if root_cause_score > 0 {
            root_cause_score
        } else {
            baseline_score
        };
        let mastery_score = weighted_mastery_score(
            baseline_score,
            speed_score,
            effective_precision,
            effective_pressure,
            effective_flex,
            effective_root_cause,
        );
        let fragility_index = average_bp(vec![
            baseline_score.saturating_sub(speed_score),
            baseline_score.saturating_sub(effective_pressure),
            baseline_score.saturating_sub(effective_flex),
        ])
        .unwrap_or(0);
        let pressure_collapse_index = baseline_score.saturating_sub(effective_pressure);
        let recognition_gap_index = baseline_score.saturating_sub(effective_flex);
        let formula_recall_use_delta =
            formula_recall_use_delta(&self.skill_type, effective_root_cause, effective_precision);
        let stability_score = 10_000u16.saturating_sub(fragility_index);
        let weakness_type_primary = primary_skill_weakness_type(
            mastery_score,
            speed_score,
            effective_precision,
            effective_pressure,
            effective_flex,
            endurance_score,
            formula_recall_use_delta,
        )
        .to_string();
        let weakness_type_secondary = secondary_skill_weakness_type(
            &weakness_type_primary,
            speed_score,
            effective_pressure,
            effective_flex,
            endurance_score,
        )
        .map(str::to_string);
        let mastery_state = classify_skill_mastery_state(
            mastery_score,
            fragility_index,
            pressure_collapse_index,
            recognition_gap_index,
        )
        .to_string();
        let recommended_intervention =
            recommendation_for_weakness_type(&weakness_type_primary).to_string();

        DiagnosticSkillResult {
            skill_key: self.skill_key,
            skill_name: self.skill_name,
            skill_type: self.skill_type,
            topic_id: self.topic_id,
            topic_name: self.topic_name,
            baseline_score,
            speed_score,
            precision_score: effective_precision,
            pressure_score: effective_pressure,
            flex_score: effective_flex,
            root_cause_score: effective_root_cause,
            endurance_score,
            recovery_score,
            mastery_score,
            fragility_index,
            pressure_collapse_index,
            recognition_gap_index,
            formula_recall_use_delta,
            stability_score,
            mastery_state,
            weakness_type_primary,
            weakness_type_secondary,
            recommended_intervention,
            evidence: json!({
                "skill_kind": self.skill_kind,
                "skill_id": self.skill_id,
                "family_count": self.families.len(),
                "phase_totals": self.phase_totals,
            }),
        }
    }
}

fn build_topic_longitudinal_signal(
    previous_context: Option<&PreviousDiagnosticContext>,
    previous_topic: Option<&HistoricalTopicSnapshot>,
    analytics: &DiagnosticTopicAnalytics,
    hypotheses: &[DiagnosticRootCauseHypothesis],
) -> Option<TopicDiagnosticLongitudinalSignal> {
    let previous_context = previous_context?;
    let previous_mastery_score = previous_topic.map(|item| item.mastery_score);
    let previous_classification = previous_topic.map(|item| item.classification.clone());
    let mastery_delta =
        previous_topic.map(|item| analytics.mastery_score as i64 - item.mastery_score as i64);
    let pressure_delta =
        previous_topic.map(|item| analytics.pressure_score as i64 - item.pressure_score as i64);
    let flexibility_delta = previous_topic
        .map(|item| analytics.flexibility_score as i64 - item.flexibility_score as i64);
    let cause_evolution =
        build_cause_evolution(analytics, hypotheses, previous_context, previous_topic);
    let trend = topic_trend_label(
        previous_topic,
        mastery_delta,
        pressure_delta,
        flexibility_delta,
        cause_evolution.as_ref(),
    )
    .to_string();

    Some(TopicDiagnosticLongitudinalSignal {
        previous_diagnostic_id: Some(previous_context.diagnostic_id),
        previous_completed_at: previous_context.completed_at.clone(),
        previous_classification,
        previous_mastery_score,
        mastery_delta,
        pressure_delta,
        flexibility_delta,
        trend,
        cause_evolution,
    })
}

fn build_cause_evolution(
    analytics: &DiagnosticTopicAnalytics,
    hypotheses: &[DiagnosticRootCauseHypothesis],
    previous_context: &PreviousDiagnosticContext,
    previous_topic: Option<&HistoricalTopicSnapshot>,
) -> Option<DiagnosticCauseEvolution> {
    let current_top = top_hypothesis_snapshot(hypotheses);
    let current_code = current_top.as_ref().map(|item| item.0.clone());
    let current_confidence = current_top.as_ref().map(|item| item.1 as i64);
    let previous_code = previous_topic.and_then(|item| item.top_hypothesis_code.clone());
    let previous_confidence = previous_topic
        .and_then(|item| item.top_hypothesis_confidence)
        .map(|value| value as i64);

    let (evolution_status, recurrence_count) =
        match (previous_code.as_deref(), current_code.as_deref()) {
            (Some(previous), Some(current)) if previous == current => {
                let confidence_delta =
                    current_confidence.unwrap_or(0) - previous_confidence.unwrap_or(0);
                if confidence_delta >= 500 {
                    ("repeated_intensifying", 2)
                } else if confidence_delta <= -500 {
                    ("repeated_softening", 2)
                } else {
                    ("repeated", 2)
                }
            }
            (Some(_), Some(_)) => ("shifted", 1),
            (None, Some(_)) => ("new", 1),
            (Some(_), None) => ("resolved", 0),
            (None, None) => return None,
        };

    let confidence_delta = match (current_confidence, previous_confidence) {
        (Some(current), Some(previous)) => Some(current - previous),
        _ => None,
    };
    let summary = summarize_cause_evolution(
        &analytics.topic_name,
        previous_context,
        previous_topic,
        analytics.diagnostic_id,
        current_code.as_deref(),
        evolution_status,
        confidence_delta,
    );

    Some(DiagnosticCauseEvolution {
        topic_id: analytics.topic_id,
        topic_name: analytics.topic_name.clone(),
        current_hypothesis_code: current_code,
        previous_hypothesis_code: previous_code,
        evolution_status: evolution_status.to_string(),
        recurrence_count,
        confidence_delta,
        summary,
    })
}

fn top_hypothesis_snapshot(
    hypotheses: &[DiagnosticRootCauseHypothesis],
) -> Option<(String, BasisPoints)> {
    hypotheses
        .iter()
        .max_by(|left, right| {
            left.confidence_score
                .cmp(&right.confidence_score)
                .then_with(|| left.hypothesis_code.cmp(&right.hypothesis_code))
        })
        .map(|item| (item.hypothesis_code.clone(), item.confidence_score))
}

fn summarize_cause_evolution(
    topic_name: &str,
    previous_context: &PreviousDiagnosticContext,
    previous_topic: Option<&HistoricalTopicSnapshot>,
    current_diagnostic_id: i64,
    current_code: Option<&str>,
    evolution_status: &str,
    confidence_delta: Option<i64>,
) -> String {
    let previous_code = previous_topic.and_then(|item| item.top_hypothesis_code.as_deref());
    match evolution_status {
        "repeated" | "repeated_intensifying" | "repeated_softening" => format!(
            "{} still shows {} across diagnostics {} and {}{}.",
            topic_name,
            current_code.unwrap_or("unknown_cause").replace('_', " "),
            previous_context.diagnostic_id,
            current_diagnostic_id,
            confidence_delta
                .map(|delta| format!(" ({:+} bp confidence)", delta))
                .unwrap_or_default(),
        ),
        "shifted" => format!(
            "{} shifted from {} to {} between repeated diagnostics.",
            topic_name,
            previous_code.unwrap_or("unknown_cause").replace('_', " "),
            current_code.unwrap_or("unknown_cause").replace('_', " "),
        ),
        "new" => format!(
            "{} now shows {} and the previous diagnostic had no stored dominant cause.",
            topic_name,
            current_code.unwrap_or("unknown_cause").replace('_', " "),
        ),
        "resolved" => format!(
            "{} no longer shows the previous dominant cause {}.",
            topic_name,
            previous_code.unwrap_or("unknown_cause").replace('_', " "),
        ),
        _ => format!(
            "{} shows mixed cause movement across diagnostics.",
            topic_name
        ),
    }
}

fn topic_trend_label(
    previous_topic: Option<&HistoricalTopicSnapshot>,
    mastery_delta: Option<i64>,
    pressure_delta: Option<i64>,
    flexibility_delta: Option<i64>,
    cause_evolution: Option<&DiagnosticCauseEvolution>,
) -> &'static str {
    let Some(previous_topic) = previous_topic else {
        return "first_measured";
    };
    let mastery_delta = mastery_delta.unwrap_or(0);
    let pressure_delta = pressure_delta.unwrap_or(0);
    let flexibility_delta = flexibility_delta.unwrap_or(0);

    if mastery_delta >= 700 && pressure_delta >= 0 && flexibility_delta >= 0 {
        "improving"
    } else if mastery_delta <= -700 {
        "declining"
    } else if pressure_delta <= -700 || flexibility_delta <= -700 {
        "destabilizing"
    } else if cause_evolution
        .map(|item| item.evolution_status.starts_with("repeated"))
        .unwrap_or(false)
        && mastery_delta.abs() < 400
    {
        "stalled"
    } else if previous_topic.classification == "fragile_under_pressure"
        && pressure_delta >= 500
        && mastery_delta >= 0
    {
        "recovering"
    } else {
        "stable"
    }
}

fn build_longitudinal_summary(
    previous_context: Option<&PreviousDiagnosticContext>,
    overall_readiness: BasisPoints,
    topic_results: &[TopicDiagnosticResult],
) -> Option<DiagnosticLongitudinalSummary> {
    let previous_context = previous_context?;
    let mut improved_topic_count = 0usize;
    let mut declined_topic_count = 0usize;
    let mut stable_topic_count = 0usize;
    let mut persistent_cause_count = 0usize;
    let mut shifted_cause_count = 0usize;
    let mut new_cause_count = 0usize;
    let mut top_regressions = Vec::<(i64, String)>::new();
    let mut cause_evolution = Vec::new();

    for topic in topic_results {
        if let Some(signal) = topic.longitudinal_signal.as_ref() {
            match signal.trend.as_str() {
                "improving" | "recovering" => improved_topic_count += 1,
                "declining" | "destabilizing" => declined_topic_count += 1,
                _ => stable_topic_count += 1,
            }
            if let Some(delta) = signal.mastery_delta.filter(|delta| *delta < 0) {
                top_regressions.push((
                    delta,
                    format!(
                        "{} ({:+} bp mastery, {})",
                        topic.topic_name, delta, signal.trend
                    ),
                ));
            }
            if let Some(evolution) = signal.cause_evolution.clone() {
                match evolution.evolution_status.as_str() {
                    "repeated" | "repeated_intensifying" | "repeated_softening" => {
                        persistent_cause_count += 1
                    }
                    "shifted" => shifted_cause_count += 1,
                    "new" => new_cause_count += 1,
                    _ => {}
                }
                cause_evolution.push(evolution);
            }
        }
    }

    top_regressions.sort_by(|left, right| left.0.cmp(&right.0));
    let overall_readiness_delta = previous_context
        .overall_readiness
        .map(|previous| overall_readiness as i64 - previous as i64);
    let trend = overall_longitudinal_trend(
        overall_readiness_delta,
        improved_topic_count,
        declined_topic_count,
        persistent_cause_count,
    )
    .to_string();

    Some(DiagnosticLongitudinalSummary {
        previous_diagnostic_id: Some(previous_context.diagnostic_id),
        previous_completed_at: previous_context.completed_at.clone(),
        overall_readiness_delta,
        trend,
        improved_topic_count,
        declined_topic_count,
        stable_topic_count,
        persistent_cause_count,
        shifted_cause_count,
        new_cause_count,
        top_regressions: top_regressions
            .into_iter()
            .take(3)
            .map(|(_, summary)| summary)
            .collect(),
        cause_evolution,
    })
}

fn overall_longitudinal_trend(
    overall_readiness_delta: Option<i64>,
    improved_topic_count: usize,
    declined_topic_count: usize,
    persistent_cause_count: usize,
) -> &'static str {
    match overall_readiness_delta.unwrap_or(0) {
        600..=i64::MAX if declined_topic_count == 0 => "improving",
        i64::MIN..=-600 => "declining",
        _ if declined_topic_count > improved_topic_count => "declining",
        _ if persistent_cause_count > 0 && improved_topic_count == 0 => "stalled",
        _ if improved_topic_count > declined_topic_count => "recovering",
        _ => "mixed",
    }
}

fn enrich_hypotheses_with_longitudinal_context(
    hypotheses: Vec<DiagnosticRootCauseHypothesis>,
    analytics: &DiagnosticTopicAnalytics,
    longitudinal_signal: Option<&TopicDiagnosticLongitudinalSignal>,
) -> Vec<DiagnosticRootCauseHypothesis> {
    let top_hypothesis_code = top_hypothesis_snapshot(&hypotheses).map(|item| item.0);
    hypotheses
        .into_iter()
        .map(|mut hypothesis| {
            let mut evidence = match hypothesis.evidence {
                Value::Object(map) => map,
                other => {
                    let mut map = Map::new();
                    map.insert("base_evidence".to_string(), other);
                    map
                }
            };
            evidence.insert(
                "diagnostic_snapshot".to_string(),
                json!({
                    "diagnostic_id": analytics.diagnostic_id,
                    "topic_id": analytics.topic_id,
                    "classification": analytics.classification,
                    "mastery_score": analytics.mastery_score,
                    "pressure_score": analytics.pressure_score,
                    "flexibility_score": analytics.flexibility_score,
                    "recommended_action": analytics.recommended_action,
                }),
            );
            if let Some(signal) = longitudinal_signal {
                evidence.insert(
                    "longitudinal".to_string(),
                    json!({
                        "previous_diagnostic_id": signal.previous_diagnostic_id,
                        "previous_completed_at": &signal.previous_completed_at,
                        "previous_classification": &signal.previous_classification,
                        "previous_mastery_score": signal.previous_mastery_score,
                        "mastery_delta": signal.mastery_delta,
                        "pressure_delta": signal.pressure_delta,
                        "flexibility_delta": signal.flexibility_delta,
                        "trend": &signal.trend,
                    }),
                );
                if top_hypothesis_code
                    .as_deref()
                    .map(|code| code == hypothesis.hypothesis_code)
                    .unwrap_or(false)
                {
                    evidence.insert(
                        "cause_evolution".to_string(),
                        json!(&signal.cause_evolution),
                    );
                    evidence.insert(
                        "durable_signal".to_string(),
                        json!(
                            signal
                                .cause_evolution
                                .as_ref()
                                .map(|item| item.recurrence_count >= 2
                                    || item.evolution_status == "shifted")
                                .unwrap_or(false)
                        ),
                    );
                }
            }
            hypothesis.evidence = Value::Object(evidence);
            hypothesis
        })
        .collect()
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
        DiagnosticPhaseCode::Endurance => "fatigue_probe_on_inconsistent_topics",
        DiagnosticPhaseCode::Recovery => "recovery_probe_after_error_heavy_topics",
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
        DiagnosticPhaseCode::Endurance => {
            inaccuracy + signal.avg_response_time_ms.min(90_000) / 25 + signal.high_conf_wrong * 900
        }
        DiagnosticPhaseCode::Recovery => {
            inaccuracy + signal.high_conf_wrong * 1_100 + signal.low_conf_correct * 650
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
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Endurance,
                question_count: 8,
                time_limit_seconds: Some(30),
                timed: true,
                evidence_weight: 9_000,
            },
            DiagnosticPhaseTemplate {
                code: DiagnosticPhaseCode::Recovery,
                question_count: 6,
                time_limit_seconds: Some(35),
                timed: false,
                evidence_weight: 8_800,
            },
        ],
    }
}

fn diagnostic_instance_status_for_phase(phase_number: i64) -> &'static str {
    match phase_number {
        1 => "phase_1",
        2 => "phase_2",
        3 => "phase_3",
        4 => "phase_4",
        _ => "phase_5",
    }
}

fn diagnostic_group_type(mode: DiagnosticMode) -> &'static str {
    match mode {
        DiagnosticMode::Quick => "light",
        DiagnosticMode::Standard => "standard",
        DiagnosticMode::Deep => "deep",
    }
}

fn deep_phase_type_from_code(phase_code: &str) -> &'static str {
    match phase_code {
        "baseline" => "baseline",
        "speed" => "speed",
        "precision" => "precision",
        "pressure" => "pressure",
        "flex" => "flexibility",
        "root_cause" => "root_cause",
        "endurance" => "endurance",
        "recovery" => "recovery",
        _ => "baseline",
    }
}

fn parse_report_list(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn accuracy_bp(count: usize, total: usize) -> BasisPoints {
    if total == 0 {
        0
    } else {
        ((count as f64 / total as f64) * 10_000.0).round() as BasisPoints
    }
}

fn median_i64(mut values: Vec<i64>) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        Some((values[middle - 1] + values[middle]) / 2)
    } else {
        Some(values[middle])
    }
}

fn average_bp(values: Vec<BasisPoints>) -> Option<BasisPoints> {
    if values.is_empty() {
        return None;
    }
    Some(
        (values.iter().map(|value| *value as i64).sum::<i64>() / values.len() as i64)
            .clamp(0, 10_000) as BasisPoints,
    )
}

fn segment_accuracy_profile(
    attempts: &[PhaseAttemptSnapshot],
) -> (
    Option<BasisPoints>,
    Option<BasisPoints>,
    Option<BasisPoints>,
) {
    if attempts.is_empty() {
        return (None, None, None);
    }
    let chunk_size = ((attempts.len() as f64) / 3.0).ceil() as usize;
    let chunk_accuracy = |chunk: &[PhaseAttemptSnapshot]| -> Option<BasisPoints> {
        if chunk.is_empty() {
            return None;
        }
        let answered = chunk
            .iter()
            .filter(|item| item.is_correct.is_some())
            .count();
        let correct = chunk
            .iter()
            .filter(|item| item.is_correct == Some(true))
            .count();
        Some(accuracy_bp(correct, answered))
    };
    let early = chunk_accuracy(&attempts[..attempts.len().min(chunk_size)]);
    let middle_start = attempts.len().min(chunk_size);
    let middle_end = attempts.len().min(chunk_size * 2);
    let middle = chunk_accuracy(&attempts[middle_start..middle_end]);
    let final_segment = chunk_accuracy(&attempts[middle_end..]);
    (early, middle, final_segment)
}

fn segment_volatility_bp(
    early: Option<BasisPoints>,
    middle: Option<BasisPoints>,
    late: Option<BasisPoints>,
) -> BasisPoints {
    let values = [early, middle, late]
        .into_iter()
        .flatten()
        .map(|value| value as i64)
        .collect::<Vec<_>>();
    if values.len() < 2 {
        return 0;
    }
    let min = *values.iter().min().unwrap_or(&0);
    let max = *values.iter().max().unwrap_or(&0);
    (max - min).clamp(0, 10_000) as BasisPoints
}

fn phase_accuracy_from_totals(
    phase_totals: &std::collections::BTreeMap<String, (usize, usize)>,
    phase_code: &str,
) -> BasisPoints {
    phase_totals
        .get(phase_code)
        .map(|(total, correct)| accuracy_bp(*correct, *total))
        .unwrap_or(0)
}

fn weighted_mastery_score(
    baseline: BasisPoints,
    speed: BasisPoints,
    precision: BasisPoints,
    pressure: BasisPoints,
    flex: BasisPoints,
    root_cause: BasisPoints,
) -> BasisPoints {
    (((baseline as i64 * 30)
        + (precision as i64 * 20)
        + (flex as i64 * 15)
        + (root_cause as i64 * 15)
        + (speed as i64 * 10)
        + (pressure as i64 * 10))
        / 100)
        .clamp(0, 10_000) as BasisPoints
}

fn formula_recall_use_delta(
    skill_type: &str,
    root_cause_score: BasisPoints,
    precision_score: BasisPoints,
) -> BasisPoints {
    if skill_type.contains("formula") {
        root_cause_score.saturating_sub(precision_score)
    } else {
        0
    }
}

fn primary_skill_weakness_type(
    mastery_score: BasisPoints,
    speed_score: BasisPoints,
    precision_score: BasisPoints,
    pressure_score: BasisPoints,
    flex_score: BasisPoints,
    endurance_score: BasisPoints,
    formula_delta: BasisPoints,
) -> &'static str {
    if mastery_score < 4_500 {
        "knowledge"
    } else if pressure_score + 1_500 < mastery_score {
        "pressure"
    } else if flex_score + 1_500 < mastery_score {
        "recognition"
    } else if speed_score + 1_500 < precision_score {
        "retrieval"
    } else if formula_delta >= 1_500 {
        "formula"
    } else if endurance_score > 0 && endurance_score + 1_200 < mastery_score {
        "endurance"
    } else {
        "execution"
    }
}

fn secondary_skill_weakness_type(
    primary: &str,
    speed_score: BasisPoints,
    pressure_score: BasisPoints,
    flex_score: BasisPoints,
    endurance_score: BasisPoints,
) -> Option<&'static str> {
    if primary != "pressure" && pressure_score < 5_500 {
        Some("pressure")
    } else if primary != "recognition" && flex_score < 5_500 {
        Some("recognition")
    } else if primary != "retrieval" && speed_score < 5_500 {
        Some("retrieval")
    } else if primary != "endurance" && endurance_score > 0 && endurance_score < 5_500 {
        Some("endurance")
    } else {
        None
    }
}

fn classify_skill_mastery_state(
    mastery_score: BasisPoints,
    fragility_index: BasisPoints,
    pressure_collapse_index: BasisPoints,
    recognition_gap_index: BasisPoints,
) -> &'static str {
    if mastery_score < 5_000 {
        "critical"
    } else if fragility_index >= 3_000
        || pressure_collapse_index >= 2_000
        || recognition_gap_index >= 2_000
    {
        "fragile"
    } else if mastery_score >= 8_000 {
        "strong"
    } else {
        "firming"
    }
}

fn recommendation_for_weakness_type(weakness_type: &str) -> &'static str {
    match weakness_type {
        "knowledge" => "teach_mode_rebuild",
        "retrieval" => "rapid_recall_drills",
        "recognition" => "recognition_drills",
        "formula" => "formula_selection_training",
        "pressure" => "pressure_ladder_mode",
        "endurance" => "stamina_builder",
        _ => "step_check_precision_drills",
    }
}

fn classify_learning_profile(
    skill_results: &[DiagnosticSkillResult],
    condition_metrics: &DiagnosticConditionMetrics,
) -> &'static str {
    if condition_metrics.pressure_collapse_index >= 2_500 {
        "pressure_collapser"
    } else if condition_metrics.formula_recall_use_delta >= 1_500 {
        "formula_memorizer"
    } else if condition_metrics.recognition_gap_index >= 2_000 {
        "recognition_gap"
    } else if condition_metrics.fragility_index >= 2_500 {
        "fragile_knower"
    } else if skill_results
        .iter()
        .any(|item| item.weakness_type_primary == "retrieval" && item.precision_score >= 6_500)
    {
        "careful_thinker"
    } else if skill_results
        .iter()
        .any(|item| item.speed_score >= 7_500 && item.pressure_score + 1_500 < item.mastery_score)
    {
        "sprinter"
    } else {
        "balanced"
    }
}

fn build_overall_summary(
    readiness_band: &str,
    topic_results: &[TopicDiagnosticResult],
    skill_results: &[DiagnosticSkillResult],
    top_recommended_action: Option<String>,
) -> DiagnosticOverallSummary {
    let strong_zones = topic_results
        .iter()
        .filter(|topic| topic.classification == "secure" || topic.mastery_score >= 8_000)
        .map(|topic| topic.topic_name.clone())
        .collect::<Vec<_>>();
    let firming_zones = topic_results
        .iter()
        .filter(|topic| topic.mastery_score >= 6_500 && topic.mastery_score < 8_000)
        .map(|topic| topic.topic_name.clone())
        .collect::<Vec<_>>();
    let fragile_zones = topic_results
        .iter()
        .filter(|topic| {
            topic.classification.contains("fragile")
                || topic.classification.contains("pressure")
                || topic.classification.contains("distorted")
        })
        .map(|topic| topic.topic_name.clone())
        .collect::<Vec<_>>();
    let mut critical_zones = skill_results
        .iter()
        .filter(|skill| skill.mastery_state == "critical")
        .take(5)
        .map(|skill| skill.skill_name.clone())
        .collect::<Vec<_>>();
    if critical_zones.is_empty() {
        critical_zones = topic_results
            .iter()
            .filter(|topic| topic.mastery_score < 5_000)
            .map(|topic| topic.topic_name.clone())
            .collect::<Vec<_>>();
    }

    DiagnosticOverallSummary {
        mastery_level: readiness_band.to_string(),
        strong_zones,
        firming_zones,
        fragile_zones,
        critical_zones,
        top_recommended_action,
    }
}

fn build_recommendations(
    _student_id: i64,
    topic_results: &[TopicDiagnosticResult],
    skill_results: &[DiagnosticSkillResult],
    condition_metrics: &DiagnosticConditionMetrics,
    learning_profile: Option<&DiagnosticLearningProfile>,
) -> Vec<DiagnosticRecommendation> {
    let mut recommendations = Vec::new();
    for skill in skill_results.iter().take(4) {
        recommendations.push(DiagnosticRecommendation {
            category: "immediate_focus".to_string(),
            action_code: skill.recommended_intervention.clone(),
            title: format!("Repair {}", skill.skill_name),
            rationale: format!(
                "{} is {} with {} as the main weakness.",
                skill.skill_name, skill.mastery_state, skill.weakness_type_primary
            ),
            priority: (10_000 - skill.mastery_score as i64).max(1),
            target_kind: Some("skill".to_string()),
            target_ref: Some(skill.skill_key.clone()),
        });
    }
    for topic in topic_results
        .iter()
        .filter(|topic| topic.mastery_score >= 8_000)
        .take(2)
    {
        recommendations.push(DiagnosticRecommendation {
            category: "maintenance".to_string(),
            action_code: "maintenance_only".to_string(),
            title: format!("Maintain {}", topic.topic_name),
            rationale: format!(
                "{} is strong and only needs light spaced review.",
                topic.topic_name
            ),
            priority: 500,
            target_kind: Some("topic".to_string()),
            target_ref: Some(format!("topic:{}", topic.topic_id)),
        });
    }
    if condition_metrics.pressure_collapse_index >= 2_000 {
        recommendations.push(DiagnosticRecommendation {
            category: "conditioning".to_string(),
            action_code: "pressure_ladder_mode".to_string(),
            title: "Condition Pressure Response".to_string(),
            rationale: "Performance drops meaningfully under pressure conditions.".to_string(),
            priority: 7_500,
            target_kind: Some("condition".to_string()),
            target_ref: Some("pressure".to_string()),
        });
    }
    if let Some(profile) = learning_profile {
        recommendations.push(DiagnosticRecommendation {
            category: "profile_route".to_string(),
            action_code: profile.profile_type.clone(),
            title: format!("Route {}", profile.profile_type.replace('_', " ")),
            rationale: "Overall diagnostic pattern suggests a specific learning profile route."
                .to_string(),
            priority: 6_500,
            target_kind: Some("profile".to_string()),
            target_ref: Some(profile.profile_type.clone()),
        });
    }
    recommendations.sort_by(|left, right| right.priority.cmp(&left.priority));
    recommendations
}

fn build_audience_reports(
    summary: &DiagnosticOverallSummary,
    topic_results: &[TopicDiagnosticResult],
    recommendations: &[DiagnosticRecommendation],
    condition_metrics: &DiagnosticConditionMetrics,
) -> Vec<DiagnosticAudienceReport> {
    let strengths = summary.strong_zones.clone();
    let fragile_areas = summary.fragile_zones.clone();
    let critical_areas = summary.critical_zones.clone();
    let action_plan = recommendations
        .iter()
        .take(4)
        .map(|item| item.title.clone())
        .collect::<Vec<_>>();
    let top_topic = topic_results
        .iter()
        .min_by_key(|topic| topic.mastery_score)
        .map(|topic| topic.topic_name.clone())
        .unwrap_or_else(|| "the weakest area".to_string());

    vec![
        DiagnosticAudienceReport {
            audience: "student".to_string(),
            headline: "Your diagnostic map".to_string(),
            narrative: format!(
                "You know some areas well, but {} becomes less reliable when speed or pressure increases. Focus next on {}.",
                top_topic,
                action_plan
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "steady repair practice".to_string())
            ),
            strengths: strengths.clone(),
            fragile_areas: fragile_areas.clone(),
            critical_areas: critical_areas.clone(),
            action_plan: action_plan.clone(),
        },
        DiagnosticAudienceReport {
            audience: "teacher".to_string(),
            headline: "Condition-aware diagnostic summary".to_string(),
            narrative: format!(
                "Baseline understanding is mixed, with fragility index {} and pressure collapse index {}. The most urgent repair target is {}.",
                condition_metrics.fragility_index,
                condition_metrics.pressure_collapse_index,
                top_topic
            ),
            strengths: strengths.clone(),
            fragile_areas: fragile_areas.clone(),
            critical_areas: critical_areas.clone(),
            action_plan: action_plan.clone(),
        },
        DiagnosticAudienceReport {
            audience: "parent".to_string(),
            headline: "How your child is doing".to_string(),
            narrative: format!(
                "Your child shows real strengths, but some areas are still fragile and can break when time becomes tight. The next priority is {}.",
                action_plan
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "guided practice".to_string())
            ),
            strengths,
            fragile_areas,
            critical_areas,
            action_plan,
        },
    ]
}

fn primary_weakness_type(
    mastery_score: BasisPoints,
    fluency_score: BasisPoints,
    pressure_score: BasisPoints,
    flexibility_score: BasisPoints,
    endurance_score: BasisPoints,
    error_profile: &ErrorProfileSnapshot,
) -> String {
    if error_profile.high_confidence_wrong_count > 0 {
        "confidence_control".to_string()
    } else if mastery_score < 4_500 || error_profile.knowledge_gap_score >= 5_000 {
        "knowledge".to_string()
    } else if pressure_score + 1_500 < mastery_score
        || error_profile.pressure_breakdown_score >= 5_000
    {
        "pressure".to_string()
    } else if flexibility_score + 1_500 < mastery_score
        || error_profile.recognition_failure_score >= 5_000
    {
        "recognition".to_string()
    } else if fluency_score + 1_500 < mastery_score || error_profile.speed_error_score >= 5_000 {
        "retrieval".to_string()
    } else if endurance_score > 0 && endurance_score + 1_200 < mastery_score {
        "endurance".to_string()
    } else if error_profile.conceptual_confusion_score >= 5_000
        || error_profile.misconception_signal_count > 0
    {
        "conceptual".to_string()
    } else {
        "execution".to_string()
    }
}

fn primary_failure_stage(error_profile: &ErrorProfileSnapshot) -> &'static str {
    let mut stages = [
        ("foundation_level", error_profile.knowledge_gap_score as i64),
        (
            "concept_level",
            error_profile.conceptual_confusion_score as i64
                + error_profile.misconception_signal_count * 400,
        ),
        (
            "recognition_level",
            error_profile.recognition_failure_score as i64,
        ),
        ("execution_level", error_profile.speed_error_score as i64),
        (
            "pressure_level",
            error_profile.pressure_breakdown_score as i64,
        ),
    ];
    stages.sort_by(|left, right| right.1.cmp(&left.1));
    stages.first().map(|item| item.0).unwrap_or("concept_level")
}

fn session_delta(
    session_scores: &[DiagnosticSessionScore],
    left_phase: &str,
    right_phase: &str,
) -> BasisPoints {
    let left = session_score_value(session_scores, left_phase);
    let right = session_score_value(session_scores, right_phase);
    left.saturating_sub(right)
}

fn session_score_value(session_scores: &[DiagnosticSessionScore], phase_code: &str) -> BasisPoints {
    session_scores
        .iter()
        .find(|score| score.phase_code == phase_code)
        .map(|score| score.adjusted_accuracy)
        .unwrap_or(0)
}

fn skill_kind_from_key(skill_key: &str) -> &'static str {
    if skill_key.starts_with("micro:") {
        "micro_skill"
    } else if skill_key.starts_with("node:") {
        "academic_node"
    } else {
        "derived"
    }
}

fn skill_id_from_key(skill_key: &str) -> Option<i64> {
    skill_key
        .split(':')
        .nth(1)
        .and_then(|value| value.parse::<i64>().ok())
}

fn extract_final_answer_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(items) => {
            let joined = items
                .iter()
                .filter_map(extract_final_answer_text)
                .collect::<Vec<_>>()
                .join(" ");
            let trimmed = joined.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Value::Object(map) => {
            for key in ["answer", "text", "value", "response", "final_answer"] {
                if let Some(value) = map.get(key).and_then(extract_final_answer_text) {
                    return Some(value);
                }
            }
            None
        }
        Value::Null => None,
    }
}

fn answer_matches_correct_options(
    submitted_answer: &str,
    correct_options: &[QuestionOption],
) -> bool {
    let normalized_submitted = normalize_answer_text(submitted_answer);
    if normalized_submitted.is_empty() {
        return false;
    }

    correct_options.iter().any(|option| {
        let normalized_option_text = normalize_answer_text(&option.option_text);
        let normalized_option_label = normalize_answer_text(&option.option_label);
        normalized_submitted == normalized_option_text
            || normalized_submitted == normalized_option_label
            || numeric_answer_value(&normalized_submitted)
                .zip(numeric_answer_value(&normalized_option_text))
                .map(|(submitted, correct)| (submitted - correct).abs() < 0.000_001)
                .unwrap_or(false)
    })
}

fn normalize_answer_text(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    lowered
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '-') {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn numeric_answer_value(text: &str) -> Option<f64> {
    let compact = text.replace(',', "");
    compact.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_questions::QuestionService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

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
    fn constructed_response_diagnostic_attempts_can_submit_without_option_ids() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status)
             VALUES ('student', 'Ama', 'hash', 'salt', 'active')",
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
        conn.execute(
            "INSERT INTO topics (subject_id, code, name) VALUES (?1, 'DNA34_FR', 'Diagnostic Free Response')",
            [subject_id],
        )
        .expect("topic should insert");
        let topic_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO questions (
                subject_id, topic_id, stem, question_format, explanation_text,
                difficulty_level, estimated_time_seconds, marks, is_active
             ) VALUES (?1, ?2, 'Simplify 6/9 as a fraction.', 'short_answer', 'Reduce to lowest terms.', 5200, 40, 1, 1)",
            params![subject_id, topic_id],
        )
        .expect("question should insert");
        let question_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO question_options (
                question_id, option_label, option_text, is_correct, position
             ) VALUES (?1, 'A', '2/3', 1, 1)",
            [question_id],
        )
        .expect("correct answer anchor should insert");

        let engine = DiagnosticEngine::new(&conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Quick,
            )
            .expect("diagnostic battery should build");
        let first_phase = battery.phases.first().expect("phase should exist");
        let first_item = engine
            .list_phase_items(battery.diagnostic_id, first_phase.phase_number)
            .expect("phase items should load")
            .into_iter()
            .next()
            .expect("constructed response item should exist");

        engine
            .submit_phase_attempt_details(
                battery.diagnostic_id,
                first_item.attempt_id,
                None,
                Some(12_000),
                Some("sure"),
                0,
                false,
                false,
                None,
                None,
                Some("reduced_fraction"),
                Some(&serde_json::json!({ "text": "2/3" })),
                Some(&serde_json::json!({ "typed": true })),
            )
            .expect("constructed response submission should succeed");

        let stored_attempt = conn
            .query_row(
                "SELECT selected_option_id, is_correct, final_answer_json
                 FROM diagnostic_item_attempts
                 WHERE id = ?1",
                [first_item.attempt_id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .expect("attempt should persist");

        assert_eq!(stored_attempt.0, None);
        assert_eq!(stored_attempt.1, Some(1));
        assert!(
            stored_attempt
                .2
                .as_deref()
                .unwrap_or_default()
                .contains("2/3")
        );
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

    #[test]
    fn complete_diagnostic_persists_repeated_cause_evolution() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Naa', 'hash', 'salt', 'active')",
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
        let (previous_diagnostic_id, previous_result) =
            run_pressure_heavy_diagnostic(&conn, student_id, subject_id, topic_id);
        let previous_target = previous_result
            .topic_results
            .iter()
            .find(|item| item.topic_id == topic_id)
            .expect("previous topic result should exist");
        assert!(previous_target.longitudinal_signal.is_none());

        let (diagnostic_id, result) =
            run_pressure_heavy_diagnostic(&conn, student_id, subject_id, topic_id);
        let target_topic = result
            .topic_results
            .iter()
            .find(|item| item.topic_id == topic_id)
            .expect("target topic result should exist");
        let longitudinal_signal = target_topic
            .longitudinal_signal
            .as_ref()
            .expect("longitudinal signal should be present");
        let cause_evolution = longitudinal_signal
            .cause_evolution
            .as_ref()
            .expect("cause evolution should be present");
        let persisted_summary = DiagnosticEngine::new(&conn)
            .get_longitudinal_summary(diagnostic_id)
            .expect("summary should load")
            .expect("summary should exist");
        let persisted_evolution = DiagnosticEngine::new(&conn)
            .list_cause_evolution(diagnostic_id)
            .expect("cause evolution should load");
        let current_hypothesis_code = cause_evolution
            .current_hypothesis_code
            .as_deref()
            .expect("current hypothesis code should exist");
        let persisted_hypotheses = DiagnosticEngine::new(&conn)
            .list_root_cause_hypotheses(diagnostic_id, Some(topic_id))
            .expect("hypotheses should load");
        let top_hypothesis = persisted_hypotheses
            .iter()
            .find(|item| item.hypothesis_code == current_hypothesis_code)
            .expect("current dominant hypothesis should exist");

        assert_eq!(
            longitudinal_signal.previous_diagnostic_id,
            Some(previous_diagnostic_id)
        );
        assert!(cause_evolution.evolution_status.starts_with("repeated"));
        assert_eq!(
            cause_evolution.previous_hypothesis_code.as_deref(),
            Some(current_hypothesis_code)
        );
        assert_eq!(
            cause_evolution.current_hypothesis_code.as_deref(),
            Some(current_hypothesis_code)
        );
        assert_eq!(
            persisted_summary.previous_diagnostic_id,
            Some(previous_diagnostic_id)
        );
        assert!(
            persisted_evolution
                .iter()
                .any(|item| item.topic_id == topic_id
                    && item.evolution_status.starts_with("repeated"))
        );
        assert_eq!(
            top_hypothesis
                .evidence
                .get("cause_evolution")
                .and_then(|item| item.get("evolution_status"))
                .and_then(Value::as_str),
            Some(cause_evolution.evolution_status.as_str())
        );
        assert_eq!(
            top_hypothesis
                .evidence
                .get("durable_signal")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn complete_diagnostic_marks_shifted_cause_when_root_cause_changes() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status) VALUES ('student', 'Akosua', 'hash', 'salt', 'active')",
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
        let previous_diagnostic_id = insert_completed_diagnostic_history(
            &conn,
            student_id,
            subject_id,
            topic_id,
            "2026-03-10T08:00:00Z",
            3400,
            3200,
            3300,
            "at_risk",
            "teach_then_guided_practice",
            "foundation_gap",
            7600,
        );

        let (diagnostic_id, result) =
            run_pressure_heavy_diagnostic(&conn, student_id, subject_id, topic_id);
        let target_topic = result
            .topic_results
            .iter()
            .find(|item| item.topic_id == topic_id)
            .expect("target topic result should exist");
        let cause_evolution = target_topic
            .longitudinal_signal
            .as_ref()
            .and_then(|item| item.cause_evolution.as_ref())
            .expect("cause evolution should be present");
        let persisted_evolution = DiagnosticEngine::new(&conn)
            .list_cause_evolution(diagnostic_id)
            .expect("cause evolution should load");

        assert_eq!(
            cause_evolution.previous_hypothesis_code.as_deref(),
            Some("foundation_gap")
        );
        assert_ne!(
            cause_evolution.current_hypothesis_code.as_deref(),
            Some("foundation_gap")
        );
        assert_eq!(cause_evolution.evolution_status, "shifted");
        assert!(
            cause_evolution
                .summary
                .contains("shifted from foundation gap")
        );
        assert!(persisted_evolution.iter().any(|item| {
            item.topic_id == topic_id
                && item.evolution_status == "shifted"
                && item.previous_hypothesis_code.as_deref() == Some("foundation_gap")
        }));
        assert_eq!(
            result
                .longitudinal_summary
                .as_ref()
                .and_then(|item| item.previous_diagnostic_id),
            Some(previous_diagnostic_id)
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

    fn run_pressure_heavy_diagnostic(
        conn: &Connection,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
    ) -> (i64, DiagnosticResult) {
        let engine = DiagnosticEngine::new(conn);
        let battery = engine
            .start_diagnostic_battery(
                student_id,
                subject_id,
                vec![topic_id],
                DiagnosticMode::Standard,
            )
            .expect("diagnostic battery should build");
        let question_service = QuestionService::new(conn);

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
                    params![
                        battery.diagnostic_id,
                        phase.phase_id,
                        first_item.question_id
                    ],
                    |row| row.get(0),
                )
                .expect("attempt should exist");
            let selected_option_id = match phase.phase_code.as_str() {
                "pressure" => wrong_option_id,
                _ => correct_option_id,
            };
            engine
                .submit_phase_attempt(
                    battery.diagnostic_id,
                    attempt_id,
                    selected_option_id,
                    Some(12_000),
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
        (battery.diagnostic_id, result)
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_completed_diagnostic_history(
        conn: &Connection,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        completed_at: &str,
        mastery_score: i64,
        pressure_score: i64,
        flexibility_score: i64,
        classification: &str,
        recommended_action: &str,
        hypothesis_code: &str,
        confidence_score: i64,
    ) -> i64 {
        conn.execute(
            "INSERT INTO diagnostic_instances (
                student_id, subject_id, session_mode, status, started_at, completed_at
             ) VALUES (?1, ?2, 'standard', 'completed', ?3, ?3)",
            params![student_id, subject_id, completed_at],
        )
        .expect("diagnostic instance should insert");
        let diagnostic_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO diagnostic_topic_analytics (
                diagnostic_id, topic_id, mastery_score, fluency_score, precision_score,
                pressure_score, flexibility_score, stability_score, classification,
                confidence_score, recommended_action
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                diagnostic_id,
                topic_id,
                mastery_score,
                mastery_score,
                mastery_score,
                pressure_score,
                flexibility_score,
                ((mastery_score + pressure_score + flexibility_score) / 3),
                classification,
                confidence_score,
                recommended_action,
            ],
        )
        .expect("topic analytics should insert");
        conn.execute(
            "INSERT INTO diagnostic_root_cause_hypotheses (
                diagnostic_id, topic_id, hypothesis_code, confidence_score, recommended_action, evidence_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, '{}')",
            params![
                diagnostic_id,
                topic_id,
                hypothesis_code,
                confidence_score,
                recommended_action,
            ],
        )
        .expect("root cause hypothesis should insert");
        diagnostic_id
    }
}
