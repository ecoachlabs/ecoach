use ecoach_questions::{QuestionSelectionRequest, QuestionSelector, SelectedQuestion};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    DiagnosticBattery, DiagnosticMode, DiagnosticPhaseCode, DiagnosticPhaseItem,
    DiagnosticPhasePlan, DiagnosticResult, TopicDiagnosticResult, WrongAnswerDiagnosis,
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
                    phase_id: row.get(0)?,
                    question_id: row.get(1)?,
                    display_order: row.get(2)?,
                    condition_type: row.get(3)?,
                    stem: row.get(4)?,
                    question_format: row.get(5)?,
                    topic_id: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
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
                "SELECT id, phase_number
                 FROM diagnostic_session_phases
                 WHERE diagnostic_id = ?1 AND phase_number > ?2 AND status = 'pending'
                 ORDER BY phase_number ASC
                 LIMIT 1",
                params![diagnostic_id, completed_phase_number],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((phase_id, phase_number)) = next_phase {
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
        let subject_id: i64 = self
            .conn
            .query_row(
                "SELECT subject_id FROM diagnostic_instances WHERE id = ?1",
                [diagnostic_id],
                |row| row.get(0),
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

        let mut topic_results = Vec::new();
        for row in rows {
            let (topic_id, topic_name) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let accuracy_bp = self.topic_accuracy_in_diagnostic(diagnostic_id, topic_id)?;
            topic_results.push(TopicDiagnosticResult {
                topic_id,
                topic_name,
                mastery_score: accuracy_bp,
                fluency_score: accuracy_bp,
                precision_score: accuracy_bp,
                pressure_score: accuracy_bp,
                flexibility_score: accuracy_bp,
                stability_score: accuracy_bp,
                classification: if accuracy_bp >= 8000 {
                    "secure".to_string()
                } else {
                    "needs_repair".to_string()
                },
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

        let result = DiagnosticResult {
            overall_readiness,
            readiness_band,
            topic_results,
            recommended_next_actions: vec![
                "generate_plan".to_string(),
                "open_repair_queue".to_string(),
            ],
        };

        let serialized = serde_json::to_string(&result)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE diagnostic_instances SET status = 'completed', completed_at = datetime('now'), result_json = ?1 WHERE id = ?2",
                params![serialized, diagnostic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(result)
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

    fn topic_accuracy_in_diagnostic(
        &self,
        diagnostic_id: i64,
        topic_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let (count, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(d.is_correct), 0)
                 FROM diagnostic_item_attempts d
                 JOIN questions q ON q.id = d.question_id
                 WHERE d.diagnostic_id = ?1 AND q.topic_id = ?2",
                params![diagnostic_id, topic_id],
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
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
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
