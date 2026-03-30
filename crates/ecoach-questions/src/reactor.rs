use std::collections::BTreeMap;

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    GeneratedQuestionDraft, Question, QuestionFamilyChoice, QuestionFamilyGenerationPriority,
    QuestionFamilyHealth, QuestionGenerationRequest, QuestionGenerationRequestInput,
    QuestionLineageEdge, QuestionLineageGraph, QuestionLineageNode, QuestionOption,
    QuestionRemediationPlan, QuestionSlotSpec, QuestionVariantMode,
};
use crate::service::QuestionService;

pub struct QuestionReactor<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct SourceQuestion {
    id: i64,
    subject_id: i64,
    topic_id: i64,
    subtopic_id: Option<i64>,
    family_id: i64,
    stem: String,
    question_format: String,
    explanation_text: Option<String>,
    difficulty_level: i64,
    estimated_time_seconds: i64,
    marks: i64,
    source_type: String,
    primary_knowledge_role: Option<String>,
    primary_cognitive_demand: Option<String>,
    primary_solve_pattern: Option<String>,
    primary_pedagogic_function: Option<String>,
    classification_confidence: i64,
    intelligence_snapshot: String,
    primary_skill_id: Option<i64>,
    cognitive_level: Option<String>,
}

#[derive(Debug)]
struct TransformedQuestion {
    stem: String,
    explanation_text: Option<String>,
    difficulty_level: i64,
    estimated_time_seconds: i64,
    marks: i64,
    options: Vec<QuestionOption>,
    transform_summary: String,
    transform_payload: serde_json::Value,
}

#[derive(Debug, Clone)]
struct GenerationRequestRow {
    id: i64,
    subject_id: i64,
    topic_id: Option<i64>,
    family_id: i64,
    source_question_id: Option<i64>,
    request_kind: String,
    variant_mode: String,
    requested_count: i64,
    status: String,
    rationale: Option<String>,
    generated_count: i64,
}

#[derive(Debug, Clone)]
struct FractionToken {
    start: usize,
    end: usize,
    numerator: i64,
    denominator: i64,
    text: String,
}

impl<'a> QuestionReactor<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_best_family_for_slot(
        &self,
        slot_spec: &QuestionSlotSpec,
    ) -> EcoachResult<Option<QuestionFamilyChoice>> {
        Ok(self
            .list_family_generation_priorities(slot_spec, 1)?
            .into_iter()
            .next()
            .map(|priority| priority.family_choice))
    }

    pub fn list_family_generation_priorities(
        &self,
        slot_spec: &QuestionSlotSpec,
        limit: usize,
    ) -> EcoachResult<Vec<QuestionFamilyGenerationPriority>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT qf.id, qf.family_code, qf.family_name, qf.subject_id, qf.topic_id,
                        COUNT(q.id) AS total_instances,
                        COALESCE(SUM(CASE WHEN q.source_type = 'generated' THEN 1 ELSE 0 END), 0) AS generated_instances,
                        COALESCE(MAX(CASE
                            WHEN ?3 = '' THEN 0
                            WHEN q.primary_cognitive_demand = ?3 THEN 1
                            ELSE 0
                        END), 0) AS cognitive_match,
                        COALESCE(MAX(CASE
                            WHEN ?4 = '' THEN 0
                            WHEN q.question_format = ?4 THEN 1
                            ELSE 0
                        END), 0) AS format_match,
                        COALESCE(qfh.freshness_score, 0) AS freshness_score,
                        COALESCE(qfh.calibration_score, 0) AS calibration_score,
                        COALESCE(qfh.quality_score, 0) AS quality_score,
                        COALESCE(qfh.health_status, 'warming') AS health_status,
                        COALESCE(qfh.recent_attempts, 0) AS recent_attempts,
                        COALESCE(qfh.misconception_hit_count, 0) AS misconception_hit_count,
                        COALESCE(qfa.recurrence_score, 0) AS recurrence_score,
                        COALESCE(qfa.replacement_score, 0) AS replacement_score
                 FROM question_families qf
                 LEFT JOIN questions q ON q.family_id = qf.id AND q.is_active = 1
                 LEFT JOIN question_family_health qfh ON qfh.family_id = qf.id
                 LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
                 WHERE qf.subject_id = ?1
                   AND (?2 = 0 OR qf.topic_id = ?5 OR qf.topic_id IS NULL)
                 GROUP BY qf.id, qf.family_code, qf.family_name, qf.subject_id, qf.topic_id,
                          qfh.freshness_score, qfh.calibration_score, qfh.quality_score,
                          qfh.health_status, qfh.recent_attempts, qfh.misconception_hit_count,
                          qfa.recurrence_score, qfa.replacement_score",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![
                    slot_spec.subject_id,
                    if slot_spec.topic_id.is_some() { 1 } else { 0 },
                    slot_spec
                        .target_cognitive_demand
                        .clone()
                        .unwrap_or_default(),
                    slot_spec.target_question_format.clone().unwrap_or_default(),
                    slot_spec.topic_id.unwrap_or_default(),
                ],
                |row| {
                    let total_instances = row.get::<_, i64>(5)?;
                    let generated_instances = row.get::<_, i64>(6)?;
                    let cognitive_match = row.get::<_, i64>(7)?;
                    let format_match = row.get::<_, i64>(8)?;
                    let freshness_score = row.get::<_, i64>(9)?.clamp(0, 10_000) as BasisPoints;
                    let calibration_score = row.get::<_, i64>(10)?.clamp(0, 10_000) as BasisPoints;
                    let quality_score = row.get::<_, i64>(11)?.clamp(0, 10_000) as BasisPoints;
                    let health_status = row.get::<_, String>(12)?;
                    let recent_attempts = row.get::<_, i64>(13)?;
                    let misconception_hit_count = row.get::<_, i64>(14)?;
                    let recurrence_score = row.get::<_, i64>(15)?.clamp(0, 10_000) as BasisPoints;
                    let replacement_score = row.get::<_, i64>(16)?.clamp(0, 10_000) as BasisPoints;
                    let family_choice = QuestionFamilyChoice {
                        family_id: row.get(0)?,
                        family_code: row.get(1)?,
                        family_name: row.get(2)?,
                        subject_id: row.get(3)?,
                        topic_id: row.get(4)?,
                        total_instances,
                        generated_instances,
                        fit_score: compute_family_fit_score(
                            total_instances,
                            generated_instances,
                            cognitive_match,
                            format_match,
                            slot_spec.max_generated_share,
                        ),
                    };

                    Ok(QuestionFamilyGenerationPriority {
                        rationale: generation_priority_rationale(
                            &health_status,
                            calibration_score,
                            replacement_score,
                            recurrence_score,
                            recent_attempts,
                            misconception_hit_count,
                        ),
                        recommended_variant_mode: recommended_generation_variant(
                            &health_status,
                            calibration_score,
                            quality_score,
                            replacement_score,
                            misconception_hit_count,
                        )
                        .to_string(),
                        priority_score: compute_generation_priority_score(
                            &family_choice,
                            freshness_score,
                            calibration_score,
                            quality_score,
                            recurrence_score,
                            replacement_score,
                            recent_attempts,
                            slot_spec.max_generated_share,
                        ),
                        family_choice,
                        health_status,
                        freshness_score,
                        calibration_score,
                        quality_score,
                        recurrence_score,
                        replacement_score,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut priorities = Vec::new();
        for row in rows {
            priorities.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        priorities.sort_by(|left, right| {
            right
                .priority_score
                .cmp(&left.priority_score)
                .then(
                    right
                        .family_choice
                        .fit_score
                        .cmp(&left.family_choice.fit_score),
                )
                .then(right.replacement_score.cmp(&left.replacement_score))
                .then(
                    left.family_choice
                        .family_name
                        .cmp(&right.family_choice.family_name),
                )
        });
        priorities.truncate(limit.max(1));
        Ok(priorities)
    }

    pub fn recommend_remediation_plan(
        &self,
        student_id: i64,
        slot_spec: &QuestionSlotSpec,
    ) -> EcoachResult<Option<QuestionRemediationPlan>> {
        let candidate = self
            .conn
            .query_row(
                "SELECT
                    q.family_id,
                    COUNT(*) AS attempts,
                    COALESCE(SUM(CASE WHEN sqa.is_correct = 0 THEN 1 ELSE 0 END), 0) AS wrong_attempts,
                    COALESCE(SUM(CASE WHEN sqa.misconception_triggered_id IS NOT NULL THEN 1 ELSE 0 END), 0) AS misconception_hits,
                    CAST(COALESCE(AVG(COALESCE(sqa.response_time_ms, 0)), 0) AS INTEGER) AS avg_response_time_ms,
                    MAX(CASE WHEN sqa.is_correct = 0 THEN sqa.question_id ELSE NULL END) AS latest_wrong_question_id
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1
                   AND q.subject_id = ?2
                   AND q.family_id IS NOT NULL
                   AND (?3 = 0 OR q.topic_id = ?4)
                 GROUP BY q.family_id
                 ORDER BY wrong_attempts DESC, misconception_hits DESC, avg_response_time_ms DESC, attempts DESC
                 LIMIT 1",
                params![
                    student_id,
                    slot_spec.subject_id,
                    if slot_spec.topic_id.is_some() { 1 } else { 0 },
                    slot_spec.topic_id.unwrap_or_default(),
                ],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((
            family_id,
            attempts,
            wrong_attempts,
            misconception_hits,
            avg_response_time_ms,
            latest_wrong_question_id,
        )) = candidate
        {
            if let Some(family_choice) = self.load_family_choice_for_slot(family_id, slot_spec)? {
                let (variant_mode, rationale) = if misconception_hits > 0 {
                    (
                        QuestionVariantMode::MisconceptionProbe,
                        format!(
                            "Student has {} misconception-linked misses in this family; probe the distractor boundary directly.",
                            misconception_hits
                        ),
                    )
                } else if wrong_attempts >= 2 || avg_response_time_ms >= 45_000 {
                    (
                        QuestionVariantMode::Rescue,
                        format!(
                            "Student has {} misses across {} attempts in this family, so reduce load and rebuild the core pattern.",
                            wrong_attempts, attempts
                        ),
                    )
                } else {
                    (
                        QuestionVariantMode::RepresentationShift,
                        "Student has touched this family already; keep the logic but shift the surface form to verify transfer."
                            .to_string(),
                    )
                };

                let speed_pressure = if avg_response_time_ms >= 60_000 {
                    1_600
                } else if avg_response_time_ms >= 40_000 {
                    900
                } else {
                    300
                };
                let priority_score = clamp_bp(
                    3_200
                        + wrong_attempts * 1_100
                        + misconception_hits * 1_500
                        + speed_pressure
                        + ((10_000 - i64::from(family_choice.fit_score)) / 6),
                ) as BasisPoints;

                return Ok(Some(QuestionRemediationPlan {
                    family_choice,
                    variant_mode: variant_mode.as_str().to_string(),
                    priority_score,
                    source_question_id: latest_wrong_question_id,
                    request_kind: "remediation".to_string(),
                    rationale,
                }));
            }
        }

        let fallback_family = self.get_best_family_for_slot(slot_spec)?;
        Ok(fallback_family.map(|family_choice| QuestionRemediationPlan {
            priority_score: family_choice.fit_score,
            family_choice,
            variant_mode: QuestionVariantMode::Rescue.as_str().to_string(),
            source_question_id: None,
            request_kind: "remediation".to_string(),
            rationale:
                "No student-specific failure trace exists yet, so start with the best family fit and a lower-load repair variant."
                    .to_string(),
        }))
    }

    pub fn create_generation_request(
        &self,
        input: &QuestionGenerationRequestInput,
    ) -> EcoachResult<QuestionGenerationRequest> {
        if input.requested_count == 0 {
            return Err(EcoachError::Validation(
                "generation request must ask for at least one question".to_string(),
            ));
        }

        let family_id = match input.family_id {
            Some(family_id) => family_id,
            None => {
                self.get_best_family_for_slot(&input.slot_spec)?
                    .ok_or_else(|| {
                        EcoachError::NotFound(
                            "no question family matched the requested slot".to_string(),
                        )
                    })?
                    .family_id
            }
        };

        let constraints_json = serde_json::to_string(&input.slot_spec)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO question_generation_requests (
                    subject_id, topic_id, family_id, source_question_id, request_kind, variant_mode,
                    requested_count, status, constraints_json, rationale
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8, ?9)",
                params![
                    input.slot_spec.subject_id,
                    input.slot_spec.topic_id,
                    family_id,
                    input.source_question_id,
                    input.request_kind,
                    input.variant_mode.as_str(),
                    input.requested_count as i64,
                    constraints_json,
                    input.rationale,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.load_generation_request(self.conn.last_insert_rowid())?
            .ok_or_else(|| {
                EcoachError::NotFound("created generation request was missing".to_string())
            })
    }

    pub fn process_generation_request(
        &self,
        request_id: i64,
    ) -> EcoachResult<Vec<GeneratedQuestionDraft>> {
        let request = self
            .load_generation_request_row(request_id)?
            .ok_or_else(|| EcoachError::NotFound("generation request was not found".to_string()))?;

        if request.status == "completed" {
            return self.load_generated_drafts_for_request(request_id);
        }

        self.set_generation_request_status(request_id, "processing", request.generated_count)?;

        let variant_mode = parse_variant_mode(&request.variant_mode)?;
        let mut generated = Vec::new();
        for offset in 0..request.requested_count.max(1) {
            let source_question =
                self.select_source_question(request.family_id, request.source_question_id, offset)?;
            let source_options = self.list_options(source_question.id)?;
            let (transformed, similarity_score) = self.prepare_candidate_for_insert(
                &request,
                &source_question,
                &source_options,
                variant_mode,
                offset,
            )?;
            let generated_question_id = self.insert_generated_question(
                &request,
                &source_question,
                variant_mode,
                &transformed,
            )?;
            self.copy_skill_links(source_question.id, generated_question_id)?;
            self.copy_intelligence_links(source_question.id, generated_question_id)?;
            self.insert_generated_options(generated_question_id, &transformed.options)?;
            self.insert_lineage_records(
                request.family_id,
                source_question.id,
                generated_question_id,
                variant_mode,
                &transformed.transform_summary,
                &transformed.options,
            )?;
            self.insert_question_graph_edges(
                &source_question,
                generated_question_id,
                variant_mode,
                similarity_score,
            )?;
            self.insert_transform_log(
                request.id,
                request.family_id,
                source_question.id,
                generated_question_id,
                variant_mode,
                &transformed.transform_summary,
                transformed.transform_payload.clone(),
            )?;

            let question = self.get_question(generated_question_id)?.ok_or_else(|| {
                EcoachError::NotFound("generated question could not be reloaded".to_string())
            })?;
            let options = self.list_options(generated_question_id)?;
            generated.push(GeneratedQuestionDraft {
                request_id: request.id,
                source_question_id: source_question.id,
                question,
                options,
                variant_mode: variant_mode.as_str().to_string(),
                transform_summary: transformed.transform_summary.clone(),
            });
        }

        self.set_generation_request_status(
            request_id,
            "completed",
            request.generated_count + generated.len() as i64,
        )?;
        self.refresh_family_health(request.family_id)?;

        Ok(generated)
    }

    pub fn get_question_lineage(&self, question_id: i64) -> EcoachResult<QuestionLineageGraph> {
        self.ensure_lineage_node(question_id, None, None, None, None)?;

        let focus_node_id = self
            .conn
            .query_row(
                "SELECT id FROM question_lineage_nodes WHERE question_id = ?1",
                [question_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut node_statement = self
            .conn
            .prepare(
                "SELECT question_id, family_id, lineage_key, node_role, origin_kind, fingerprint_text
                 FROM question_lineage_nodes
                 WHERE id = ?1
                    OR id IN (
                        SELECT from_node_id FROM question_lineage_edges WHERE to_node_id = ?1
                        UNION
                        SELECT to_node_id FROM question_lineage_edges WHERE from_node_id = ?1
                    )
                 ORDER BY question_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let node_rows = node_statement
            .query_map([focus_node_id], |row| {
                Ok(QuestionLineageNode {
                    question_id: row.get(0)?,
                    family_id: row.get(1)?,
                    lineage_key: row.get(2)?,
                    node_role: row.get(3)?,
                    origin_kind: row.get(4)?,
                    fingerprint_text: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut nodes = Vec::new();
        for row in node_rows {
            nodes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut edge_statement = self
            .conn
            .prepare(
                "SELECT parent.question_id, child.question_id, qle.relation_type, qle.transform_mode, qle.rationale
                 FROM question_lineage_edges qle
                 INNER JOIN question_lineage_nodes parent ON parent.id = qle.from_node_id
                 INNER JOIN question_lineage_nodes child ON child.id = qle.to_node_id
                 WHERE qle.from_node_id = ?1 OR qle.to_node_id = ?1
                 ORDER BY qle.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let edge_rows = edge_statement
            .query_map([focus_node_id], |row| {
                Ok(QuestionLineageEdge {
                    from_question_id: row.get(0)?,
                    to_question_id: row.get(1)?,
                    relation_type: row.get(2)?,
                    transform_mode: row.get(3)?,
                    rationale: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut edges = Vec::new();
        for row in edge_rows {
            edges.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(QuestionLineageGraph {
            focus_question_id: question_id,
            nodes,
            edges,
        })
    }

    pub fn get_family_health(&self, family_id: i64) -> EcoachResult<Option<QuestionFamilyHealth>> {
        self.refresh_family_health(family_id)?;

        self.conn
            .query_row(
                "SELECT family_id, total_instances, generated_instances, active_instances,
                        recent_attempts, recent_correct_attempts, avg_response_time_ms,
                        misconception_hit_count, freshness_score, calibration_score,
                        quality_score, health_status, last_generated_at
                 FROM question_family_health
                 WHERE family_id = ?1",
                [family_id],
                |row| {
                    Ok(QuestionFamilyHealth {
                        family_id: row.get(0)?,
                        total_instances: row.get(1)?,
                        generated_instances: row.get(2)?,
                        active_instances: row.get(3)?,
                        recent_attempts: row.get(4)?,
                        recent_correct_attempts: row.get(5)?,
                        avg_response_time_ms: row.get(6)?,
                        misconception_hit_count: row.get(7)?,
                        freshness_score: row.get::<_, i64>(8)?.clamp(0, 10_000) as BasisPoints,
                        calibration_score: row.get::<_, i64>(9)?.clamp(0, 10_000) as BasisPoints,
                        quality_score: row.get::<_, i64>(10)?.clamp(0, 10_000) as BasisPoints,
                        health_status: row.get(11)?,
                        last_generated_at: row.get(12)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn record_instance_outcome(&self, question_id: i64) -> EcoachResult<()> {
        let family_id = self
            .conn
            .query_row(
                "SELECT family_id FROM questions WHERE id = ?1",
                [question_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten();

        if let Some(family_id) = family_id {
            self.refresh_family_health(family_id)?;
        }

        Ok(())
    }

    fn load_family_choice_for_slot(
        &self,
        family_id: i64,
        slot_spec: &QuestionSlotSpec,
    ) -> EcoachResult<Option<QuestionFamilyChoice>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT qf.id, qf.family_code, qf.family_name, qf.subject_id, qf.topic_id,
                        COUNT(q.id) AS total_instances,
                        COALESCE(SUM(CASE WHEN q.source_type = 'generated' THEN 1 ELSE 0 END), 0) AS generated_instances,
                        COALESCE(MAX(CASE
                            WHEN ?3 = '' THEN 0
                            WHEN q.primary_cognitive_demand = ?3 THEN 1
                            ELSE 0
                        END), 0) AS cognitive_match,
                        COALESCE(MAX(CASE
                            WHEN ?4 = '' THEN 0
                            WHEN q.question_format = ?4 THEN 1
                            ELSE 0
                        END), 0) AS format_match
                 FROM question_families qf
                 LEFT JOIN questions q ON q.family_id = qf.id AND q.is_active = 1
                 WHERE qf.id = ?1
                   AND (?2 = 0 OR qf.topic_id = ?5 OR qf.topic_id IS NULL)
                 GROUP BY qf.id, qf.family_code, qf.family_name, qf.subject_id, qf.topic_id",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        statement
            .query_row(
                params![
                    family_id,
                    if slot_spec.topic_id.is_some() { 1 } else { 0 },
                    slot_spec
                        .target_cognitive_demand
                        .clone()
                        .unwrap_or_default(),
                    slot_spec.target_question_format.clone().unwrap_or_default(),
                    slot_spec.topic_id.unwrap_or_default(),
                ],
                |row| {
                    let total_instances = row.get::<_, i64>(5)?;
                    let generated_instances = row.get::<_, i64>(6)?;
                    let cognitive_match = row.get::<_, i64>(7)?;
                    let format_match = row.get::<_, i64>(8)?;
                    let fit_score = compute_family_fit_score(
                        total_instances,
                        generated_instances,
                        cognitive_match,
                        format_match,
                        slot_spec.max_generated_share,
                    );

                    Ok(QuestionFamilyChoice {
                        family_id: row.get(0)?,
                        family_code: row.get(1)?,
                        family_name: row.get(2)?,
                        subject_id: row.get(3)?,
                        topic_id: row.get(4)?,
                        total_instances,
                        generated_instances,
                        fit_score,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

fn map_source_question(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceQuestion> {
    Ok(SourceQuestion {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        topic_id: row.get(2)?,
        subtopic_id: row.get(3)?,
        family_id: row.get(4)?,
        stem: row.get(5)?,
        question_format: row.get(6)?,
        explanation_text: row.get(7)?,
        difficulty_level: row.get(8)?,
        estimated_time_seconds: row.get(9)?,
        marks: row.get(10)?,
        source_type: row.get(11)?,
        primary_knowledge_role: row.get(12)?,
        primary_cognitive_demand: row.get(13)?,
        primary_solve_pattern: row.get(14)?,
        primary_pedagogic_function: row.get(15)?,
        classification_confidence: row.get(16)?,
        intelligence_snapshot: row.get(17)?,
        primary_skill_id: row.get(18)?,
        cognitive_level: row.get(19)?,
    })
}

fn parse_variant_mode(value: &str) -> EcoachResult<QuestionVariantMode> {
    match value {
        "isomorphic" => Ok(QuestionVariantMode::Isomorphic),
        "representation_shift" => Ok(QuestionVariantMode::RepresentationShift),
        "misconception_probe" => Ok(QuestionVariantMode::MisconceptionProbe),
        "rescue" => Ok(QuestionVariantMode::Rescue),
        "stretch" => Ok(QuestionVariantMode::Stretch),
        other => Err(EcoachError::Validation(format!(
            "unknown reactor variant mode: {}",
            other
        ))),
    }
}

fn compute_family_fit_score(
    total_instances: i64,
    generated_instances: i64,
    cognitive_match: i64,
    format_match: i64,
    max_generated_share: BasisPoints,
) -> BasisPoints {
    let generated_share = if total_instances > 0 {
        (generated_instances * 10_000) / total_instances
    } else {
        0
    };
    let generated_penalty = if generated_share > i64::from(max_generated_share) {
        ((generated_share - i64::from(max_generated_share)) / 3).min(3_000)
    } else {
        0
    };
    clamp_bp(
        3_500 + total_instances.min(4) * 700 + cognitive_match * 2_200 + format_match * 1_200
            - generated_penalty,
    ) as BasisPoints
}

fn compute_generation_priority_score(
    family_choice: &QuestionFamilyChoice,
    freshness_score: BasisPoints,
    calibration_score: BasisPoints,
    quality_score: BasisPoints,
    recurrence_score: BasisPoints,
    replacement_score: BasisPoints,
    recent_attempts: i64,
    max_generated_share: BasisPoints,
) -> BasisPoints {
    let generated_share = if family_choice.total_instances > 0 {
        ((family_choice.generated_instances * 10_000) / family_choice.total_instances)
            .clamp(0, 10_000)
    } else {
        0
    };
    let generation_headroom = i64::from(max_generated_share)
        .saturating_sub(generated_share)
        .clamp(0, 10_000);
    let calibration_gap = (10_000 - i64::from(calibration_score)).clamp(0, 10_000);
    let quality_gap = (10_000 - i64::from(quality_score)).clamp(0, 10_000);
    let freshness_gap = (10_000 - i64::from(freshness_score)).clamp(0, 10_000);
    let inventory_pressure = match family_choice.total_instances {
        count if count <= 0 => 9_200,
        1 => 8_200,
        2 => 7_000,
        3 => 5_600,
        _ => 3_200,
    };
    let evidence_pressure = if recent_attempts <= 0 {
        5_200
    } else if recent_attempts == 1 {
        3_600
    } else {
        1_200
    };

    clamp_bp(
        ((0.30 * family_choice.fit_score as f64)
            + (0.22 * replacement_score as f64)
            + (0.14 * recurrence_score as f64)
            + (0.14 * calibration_gap as f64)
            + (0.10 * inventory_pressure as f64)
            + (0.05 * freshness_gap as f64)
            + (0.03 * quality_gap as f64)
            + (0.02 * generation_headroom as f64))
            .round() as i64
            + evidence_pressure,
    ) as BasisPoints
}

fn recommended_generation_variant(
    health_status: &str,
    calibration_score: BasisPoints,
    quality_score: BasisPoints,
    replacement_score: BasisPoints,
    misconception_hit_count: i64,
) -> &'static str {
    if misconception_hit_count > 0 || health_status == "fragile" {
        QuestionVariantMode::MisconceptionProbe.as_str()
    } else if calibration_score < 5_800 {
        QuestionVariantMode::RepresentationShift.as_str()
    } else if quality_score < 6_200 {
        QuestionVariantMode::Rescue.as_str()
    } else if replacement_score >= 7_200 {
        QuestionVariantMode::Stretch.as_str()
    } else {
        QuestionVariantMode::Isomorphic.as_str()
    }
}

fn generation_priority_rationale(
    health_status: &str,
    calibration_score: BasisPoints,
    replacement_score: BasisPoints,
    recurrence_score: BasisPoints,
    recent_attempts: i64,
    misconception_hit_count: i64,
) -> String {
    if misconception_hit_count > 0 {
        "Student evidence shows misconception hits in this family, so probe variants should be kept ready.".to_string()
    } else if health_status == "fragile" {
        "Family health is fragile, so new variants should prioritize repair and calibration."
            .to_string()
    } else if recent_attempts < 2 || calibration_score < 5_800 {
        "Family has thin calibration evidence, so it should be expanded with fresh traced variants."
            .to_string()
    } else if replacement_score >= 7_200 && recurrence_score >= 5_000 {
        "Past-paper pressure suggests this family is a comeback risk and should stay warm."
            .to_string()
    } else {
        "Family is still useful for coverage, but it does not carry the strongest urgency signal."
            .to_string()
    }
}

fn variant_multiplier(variant_mode: QuestionVariantMode, offset: i64) -> i64 {
    match variant_mode {
        QuestionVariantMode::Rescue => 2 + offset.rem_euclid(2),
        QuestionVariantMode::Isomorphic => 3 + offset.rem_euclid(2),
        QuestionVariantMode::RepresentationShift => 4 + offset.rem_euclid(2),
        QuestionVariantMode::MisconceptionProbe => 3,
        QuestionVariantMode::Stretch => 5 + offset.rem_euclid(2),
    }
}

fn variant_mode_graph_relation(variant_mode: QuestionVariantMode) -> &'static str {
    match variant_mode {
        QuestionVariantMode::RepresentationShift => "representation_shift",
        QuestionVariantMode::Isomorphic => "isomorphic_cluster",
        QuestionVariantMode::Stretch | QuestionVariantMode::Rescue => "difficulty_ladder",
        QuestionVariantMode::MisconceptionProbe => "misconception_pair",
    }
}

fn rewrite_stem_for_mode(stem: &str, variant_mode: QuestionVariantMode, ordinal: i64) -> String {
    match variant_mode {
        QuestionVariantMode::RepresentationShift => stem
            .replace(
                "Which fraction is equivalent to",
                "Choose the fraction that names the same value as",
            )
            .replace("written in simplest form", "written in lowest terms"),
        QuestionVariantMode::MisconceptionProbe => {
            if stem.contains("equivalent to") {
                format!("{} Choose carefully so the value stays unchanged.", stem)
            } else {
                format!(
                    "{} Watch the numerator and denominator roles carefully.",
                    stem
                )
            }
        }
        QuestionVariantMode::Rescue => stem
            .replace(
                "Which fraction is equivalent to",
                "Pick the fraction that means the same thing as",
            )
            .replace("written in simplest form", "reduced as much as possible"),
        QuestionVariantMode::Stretch => format!("Think one step deeper: {}", stem),
        QuestionVariantMode::Isomorphic => {
            if stem.contains("Which fraction is") || stem.contains("What is") {
                stem.to_string()
            } else {
                format!("{} (variant {})", stem, ordinal)
            }
        }
    }
}

fn fallback_variant_stem(stem: &str, variant_mode: QuestionVariantMode, ordinal: i64) -> String {
    match variant_mode {
        QuestionVariantMode::Stretch => format!("Think one step deeper: {}", stem),
        QuestionVariantMode::Rescue => format!("Start with the core idea: {}", stem),
        QuestionVariantMode::MisconceptionProbe => {
            format!("Be careful with the trap here: {}", stem)
        }
        QuestionVariantMode::RepresentationShift => {
            format!("Choose the matching idea in a new wording: {}", stem)
        }
        QuestionVariantMode::Isomorphic => format!("{} (family variant {})", stem, ordinal),
    }
}

fn rewrite_explanation_for_mode(explanation: &str, variant_mode: QuestionVariantMode) -> String {
    match variant_mode {
        QuestionVariantMode::Stretch => {
            format!(
                "Keep the same rule, but be more deliberate: {}",
                explanation
            )
        }
        QuestionVariantMode::Rescue => format!("Anchor the core idea first: {}", explanation),
        _ => explanation.to_string(),
    }
}

fn rewrite_distractor_intent(
    intent: Option<&str>,
    variant_mode: QuestionVariantMode,
) -> Option<String> {
    match (intent, variant_mode) {
        (Some(intent), QuestionVariantMode::MisconceptionProbe) => Some(format!(
            "{} Probe the misconception directly.",
            intent.trim()
        )),
        (Some(intent), _) => Some(intent.to_string()),
        (None, QuestionVariantMode::MisconceptionProbe) => {
            Some("Probe the common misconception more explicitly.".to_string())
        }
        (None, _) => None,
    }
}

fn fingerprint_for(stem: &str, options: &[QuestionOption]) -> String {
    let stem_normalized = normalize_fingerprint_fragment(stem);
    let option_part = options
        .iter()
        .map(|option| normalize_fingerprint_fragment(&option.option_text))
        .collect::<Vec<_>>()
        .join("|");
    format!("{}::{}", stem_normalized, option_part)
}

fn normalize_fingerprint_fragment(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_ascii_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn find_fraction_tokens(text: &str) -> Vec<FractionToken> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }

        let numerator_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'/' {
            continue;
        }
        let slash_index = index;
        index += 1;
        let denominator_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if denominator_start == index {
            continue;
        }

        let numerator_text = &text[numerator_start..slash_index];
        let denominator_text = &text[denominator_start..index];
        let numerator = numerator_text.parse::<i64>();
        let denominator = denominator_text.parse::<i64>();
        if let (Ok(numerator), Ok(denominator)) = (numerator, denominator) {
            tokens.push(FractionToken {
                start: numerator_start,
                end: index,
                numerator,
                denominator,
                text: text[numerator_start..index].to_string(),
            });
        }
    }

    tokens
}

fn replace_fractions(text: &str, replacements: &BTreeMap<String, String>) -> String {
    let tokens = find_fraction_tokens(text);
    if tokens.is_empty() {
        return text.to_string();
    }

    let mut rebuilt = String::new();
    let mut cursor = 0;
    for token in tokens {
        rebuilt.push_str(&text[cursor..token.start]);
        if let Some(replacement) = replacements.get(&token.text) {
            rebuilt.push_str(replacement);
        } else {
            rebuilt.push_str(&token.text);
        }
        cursor = token.end;
    }
    rebuilt.push_str(&text[cursor..]);
    rebuilt
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn reactor_generates_variants_with_lineage_and_health() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        let reactor = QuestionReactor::new(&conn);
        let family = reactor
            .get_best_family_for_slot(&QuestionSlotSpec {
                subject_id,
                topic_id: Some(topic_id),
                target_cognitive_demand: Some("recognition".to_string()),
                target_question_format: Some("mcq".to_string()),
                max_generated_share: 7_000,
            })
            .expect("family choice should work")
            .expect("family should be chosen");

        let request = reactor
            .create_generation_request(&QuestionGenerationRequestInput {
                slot_spec: QuestionSlotSpec {
                    subject_id,
                    topic_id: Some(topic_id),
                    target_cognitive_demand: Some("recognition".to_string()),
                    target_question_format: Some("mcq".to_string()),
                    max_generated_share: 7_000,
                },
                family_id: Some(family.family_id),
                source_question_id: None,
                request_kind: "variant".to_string(),
                variant_mode: QuestionVariantMode::RepresentationShift,
                requested_count: 1,
                rationale: Some("Need a fresh but same-skill mock slot".to_string()),
            })
            .expect("generation request should be created");

        let generated = reactor
            .process_generation_request(request.id)
            .expect("generation request should process");

        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].question.family_id, Some(family.family_id));
        assert!(generated[0].question.stem.contains("same value as"));
        assert_eq!(
            generated[0]
                .options
                .iter()
                .filter(|option| option.is_correct)
                .count(),
            1
        );

        let lineage = reactor
            .get_question_lineage(generated[0].question.id)
            .expect("lineage should load");
        assert_eq!(lineage.nodes.len(), 2);
        assert_eq!(lineage.edges.len(), 1);
        assert_eq!(lineage.edges[0].relation_type, "representation_shift");

        let transform_log_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM question_transform_log WHERE request_id = ?1",
                [request.id],
                |row| row.get(0),
            )
            .expect("transform log count should be queryable");
        let graph_edge_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM question_graph_edges WHERE to_question_id = ?1",
                [generated[0].question.id],
                |row| row.get(0),
            )
            .expect("graph edge count should be queryable");
        let candidate_score_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM reactor_candidate_scores WHERE request_id = ?1 AND decision = 'accepted'",
                [request.id],
                |row| row.get(0),
            )
            .expect("candidate score count should be queryable");
        assert_eq!(transform_log_count, 1);
        assert!(graph_edge_count >= 2);
        assert_eq!(candidate_score_count, 1);

        let health = reactor
            .get_family_health(family.family_id)
            .expect("family health should load")
            .expect("family health should exist");
        assert_eq!(health.total_instances, 3);
        assert_eq!(health.generated_instances, 1);
        assert!(health.freshness_score >= 4_000);
    }

    #[test]
    fn record_instance_outcome_refreshes_family_health_from_attempts() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let family_id: i64 = conn
            .query_row(
                "SELECT id FROM question_families WHERE family_code = 'FRA_EQUIV_01' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("family should exist");
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions WHERE family_id = ?1 ORDER BY id ASC LIMIT 1",
                [family_id],
                |row| row.get(0),
            )
            .expect("question should exist");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt)
             VALUES ('student', 'Ada', 'hash', 'salt')",
            [],
        )
        .expect("student account should insert");
        let student_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, started_at, submitted_at, response_time_ms,
                selected_option_id, is_correct, misconception_triggered_id
             ) VALUES (?1, ?2, datetime('now'), datetime('now'), 24000, NULL, 1, NULL)",
            params![student_id, question_id],
        )
        .expect("attempt should insert");

        let reactor = QuestionReactor::new(&conn);
        reactor
            .record_instance_outcome(question_id)
            .expect("family health should refresh");

        let health = reactor
            .get_family_health(family_id)
            .expect("family health should load")
            .expect("family health should exist");
        assert_eq!(health.recent_attempts, 1);
        assert_eq!(health.recent_correct_attempts, 1);
        assert!(health.quality_score >= 6_000);
    }

    #[test]
    fn remediation_plan_prefers_misconception_probe_when_student_keeps_hitting_traps() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions WHERE topic_id = ?1 ORDER BY id ASC LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("question should exist");
        let misconception_id: i64 = conn
            .query_row(
                "SELECT misconception_id
                 FROM question_options
                 WHERE question_id = ?1 AND misconception_id IS NOT NULL
                 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .expect("misconception option should exist");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt)
             VALUES ('student', 'Remi', 'hash', 'salt')",
            [],
        )
        .expect("student account should insert");
        let student_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, started_at, submitted_at, response_time_ms,
                selected_option_id, is_correct, misconception_triggered_id
             ) VALUES (?1, ?2, datetime('now'), datetime('now'), 47000, NULL, 0, ?3)",
            params![student_id, question_id, misconception_id],
        )
        .expect("attempt should insert");

        let reactor = QuestionReactor::new(&conn);
        let plan = reactor
            .recommend_remediation_plan(
                student_id,
                &QuestionSlotSpec {
                    subject_id,
                    topic_id: Some(topic_id),
                    target_cognitive_demand: Some("recognition".to_string()),
                    target_question_format: Some("mcq".to_string()),
                    max_generated_share: 7_000,
                },
            )
            .expect("remediation plan should compute")
            .expect("remediation plan should exist");

        assert_eq!(
            plan.variant_mode,
            QuestionVariantMode::MisconceptionProbe.as_str()
        );
        assert_eq!(plan.request_kind, "remediation");
        assert_eq!(plan.source_question_id, Some(question_id));
        assert!(plan.rationale.contains("misconception"));
    }

    #[test]
    fn generation_priorities_surface_fragile_comeback_families_first() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("topic should exist");

        conn.execute(
            "INSERT INTO question_families (id, subject_id, topic_id, family_code, family_name)
             VALUES (9001, ?1, ?2, 'FRA_REMEDY', 'Fraction Repair Family')",
            params![subject_id, topic_id],
        )
        .expect("repair family should insert");
        conn.execute(
            "INSERT INTO questions (
                id, subject_id, topic_id, family_id, stem, question_format, explanation_text,
                difficulty_level, estimated_time_seconds, marks, is_active, source_type,
                primary_cognitive_demand
             ) VALUES (
                900100, ?1, ?2, 9001, 'Which fraction equals one half?', 'mcq',
                'Repair explanation', 4200, 75, 1, 1, 'authored', 'recognition'
             )",
            params![subject_id, topic_id],
        )
        .expect("repair question should insert");
        conn.execute(
            "INSERT INTO question_family_health (
                family_id, total_instances, generated_instances, active_instances, recent_attempts,
                recent_correct_attempts, avg_response_time_ms, misconception_hit_count,
                freshness_score, calibration_score, quality_score, health_status
             ) VALUES (9001, 1, 0, 1, 1, 0, 62000, 1, 3000, 2800, 4200, 'fragile')",
            [],
        )
        .expect("repair health should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (
                family_id, recurrence_score, coappearance_score, replacement_score
             ) VALUES (9001, 7600, 5400, 8800)",
            [],
        )
        .expect("repair analytics should insert");

        let reactor = QuestionReactor::new(&conn);
        let priorities = reactor
            .list_family_generation_priorities(
                &QuestionSlotSpec {
                    subject_id,
                    topic_id: Some(topic_id),
                    target_cognitive_demand: Some("recognition".to_string()),
                    target_question_format: Some("mcq".to_string()),
                    max_generated_share: 7_500,
                },
                3,
            )
            .expect("generation priorities should compute");

        assert!(!priorities.is_empty());
        assert_eq!(priorities[0].family_choice.family_id, 9001);
        assert_eq!(
            priorities[0].recommended_variant_mode,
            QuestionVariantMode::MisconceptionProbe.as_str()
        );
        assert!(priorities[0].priority_score >= priorities[0].family_choice.fit_score);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn install_sample_pack(conn: &Connection) {
        PackService::new(conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
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

impl<'a> QuestionReactor<'a> {
    fn get_question(&self, question_id: i64) -> EcoachResult<Option<Question>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks, primary_skill_id
                 FROM questions
                 WHERE id = ?1 AND is_active = 1",
                [question_id],
                |row| {
                    Ok(Question {
                        id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        subtopic_id: row.get(3)?,
                        family_id: row.get(4)?,
                        stem: row.get(5)?,
                        question_format: row.get(6)?,
                        explanation_text: row.get(7)?,
                        difficulty_level: row.get::<_, i64>(8)?.clamp(0, 10_000) as BasisPoints,
                        estimated_time_seconds: row.get(9)?,
                        marks: row.get(10)?,
                        primary_skill_id: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_options(&self, question_id: i64) -> EcoachResult<Vec<QuestionOption>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                 FROM question_options
                 WHERE question_id = ?1
                 ORDER BY position ASC, option_label ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionOption {
                    id: row.get(0)?,
                    question_id: row.get(1)?,
                    option_label: row.get(2)?,
                    option_text: row.get(3)?,
                    is_correct: row.get::<_, i64>(4)? == 1,
                    misconception_id: row.get(5)?,
                    distractor_intent: row.get(6)?,
                    position: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(options)
    }

    fn load_generation_request(
        &self,
        request_id: i64,
    ) -> EcoachResult<Option<QuestionGenerationRequest>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, family_id, source_question_id, request_kind,
                        variant_mode, requested_count, status, rationale, generated_count
                 FROM question_generation_requests
                 WHERE id = ?1",
                [request_id],
                |row| {
                    Ok(QuestionGenerationRequest {
                        id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        family_id: row.get(3)?,
                        source_question_id: row.get(4)?,
                        request_kind: row.get(5)?,
                        variant_mode: row.get(6)?,
                        requested_count: row.get(7)?,
                        status: row.get(8)?,
                        rationale: row.get(9)?,
                        generated_count: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_generation_request_row(
        &self,
        request_id: i64,
    ) -> EcoachResult<Option<GenerationRequestRow>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, family_id, source_question_id, request_kind,
                        variant_mode, requested_count, status, rationale, generated_count
                 FROM question_generation_requests
                 WHERE id = ?1",
                [request_id],
                |row| {
                    Ok(GenerationRequestRow {
                        id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        family_id: row.get(3)?,
                        source_question_id: row.get(4)?,
                        request_kind: row.get(5)?,
                        variant_mode: row.get(6)?,
                        requested_count: row.get(7)?,
                        status: row.get(8)?,
                        rationale: row.get(9)?,
                        generated_count: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_generated_drafts_for_request(
        &self,
        request_id: i64,
    ) -> EcoachResult<Vec<GeneratedQuestionDraft>> {
        let request = self
            .load_generation_request(request_id)?
            .ok_or_else(|| EcoachError::NotFound("generation request was not found".to_string()))?;
        let mut statement = self
            .conn
            .prepare(
                "SELECT generated_question_id, source_question_id, variant_mode, transform_summary
                 FROM question_transform_log
                 WHERE request_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([request_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut drafts = Vec::new();
        for row in rows {
            let (question_id, source_question_id, variant_mode, transform_summary) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let question = self.get_question(question_id)?.ok_or_else(|| {
                EcoachError::NotFound("generated question was not found".to_string())
            })?;
            drafts.push(GeneratedQuestionDraft {
                request_id: request.id,
                source_question_id,
                options: self.list_options(question_id)?,
                question,
                variant_mode,
                transform_summary,
            });
        }
        Ok(drafts)
    }

    fn set_generation_request_status(
        &self,
        request_id: i64,
        status: &str,
        generated_count: i64,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE question_generation_requests
                 SET status = ?2,
                     generated_count = ?3,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![request_id, status, generated_count],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn select_source_question(
        &self,
        family_id: i64,
        explicit_question_id: Option<i64>,
        offset: i64,
    ) -> EcoachResult<SourceQuestion> {
        if let Some(question_id) = explicit_question_id {
            return self.load_source_question(question_id)?.ok_or_else(|| {
                EcoachError::NotFound("requested source question was not found".to_string())
            });
        }

        let authored_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM questions
                 WHERE family_id = ?1 AND is_active = 1 AND source_type != 'generated'",
                [family_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let authored_offset = if authored_count > 0 {
            offset.rem_euclid(authored_count)
        } else {
            0
        };

        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks,
                        source_type, primary_knowledge_role, primary_cognitive_demand,
                        primary_solve_pattern, primary_pedagogic_function, classification_confidence,
                        intelligence_snapshot, primary_skill_id, cognitive_level
                 FROM questions
                 WHERE family_id = ?1 AND is_active = 1
                 ORDER BY CASE WHEN source_type = 'generated' THEN 1 ELSE 0 END ASC, id ASC
                 LIMIT 1 OFFSET ?2",
                params![family_id, authored_offset],
                map_source_question,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .ok_or_else(|| EcoachError::NotFound("no source question was available for the family".to_string()))
    }

    fn load_source_question(&self, question_id: i64) -> EcoachResult<Option<SourceQuestion>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks,
                        source_type, primary_knowledge_role, primary_cognitive_demand,
                        primary_solve_pattern, primary_pedagogic_function, classification_confidence,
                        intelligence_snapshot, primary_skill_id, cognitive_level
                 FROM questions
                 WHERE id = ?1 AND is_active = 1",
                [question_id],
                map_source_question,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn transform_question(
        &self,
        source_question: &SourceQuestion,
        source_options: &[QuestionOption],
        variant_mode: QuestionVariantMode,
        offset: i64,
    ) -> EcoachResult<TransformedQuestion> {
        let multiplier = variant_multiplier(variant_mode, offset);
        let transform_seed = source_question.id + offset + multiplier;

        let mut fraction_mapping = BTreeMap::new();
        for text in std::iter::once(source_question.stem.as_str())
            .chain(source_question.explanation_text.iter().map(String::as_str))
            .chain(
                source_options
                    .iter()
                    .map(|option| option.option_text.as_str()),
            )
        {
            for token in find_fraction_tokens(text) {
                fraction_mapping
                    .entry(token.text.clone())
                    .or_insert_with(|| {
                        format!(
                            "{}/{}",
                            token.numerator * multiplier,
                            token.denominator * multiplier
                        )
                    });
            }
        }

        let mut stem = replace_fractions(&source_question.stem, &fraction_mapping);
        stem = rewrite_stem_for_mode(&stem, variant_mode, offset + 1);
        if stem == source_question.stem {
            stem = fallback_variant_stem(&source_question.stem, variant_mode, offset + 1);
        }

        let explanation_text = source_question.explanation_text.as_ref().map(|text| {
            rewrite_explanation_for_mode(&replace_fractions(text, &fraction_mapping), variant_mode)
        });

        let mut options = source_options
            .iter()
            .map(|option| QuestionOption {
                id: 0,
                question_id: 0,
                option_label: String::new(),
                option_text: replace_fractions(&option.option_text, &fraction_mapping),
                is_correct: option.is_correct,
                misconception_id: option.misconception_id,
                distractor_intent: rewrite_distractor_intent(
                    option.distractor_intent.as_deref(),
                    variant_mode,
                ),
                position: 0,
            })
            .collect::<Vec<_>>();

        if variant_mode == QuestionVariantMode::MisconceptionProbe {
            options.sort_by_key(|option| {
                (
                    if option.misconception_id.is_some() {
                        0
                    } else {
                        1
                    },
                    if option.is_correct { 1 } else { 0 },
                    option.option_text.clone(),
                )
            });
        }

        for (index, option) in options.iter_mut().enumerate() {
            option.position = index as i64;
            option.option_label = char::from(b'A' + index as u8).to_string();
        }

        let difficulty_level = match variant_mode {
            QuestionVariantMode::Rescue => (source_question.difficulty_level - 1_300).max(1_200),
            QuestionVariantMode::Stretch => (source_question.difficulty_level + 1_500).min(9_500),
            QuestionVariantMode::RepresentationShift => {
                (source_question.difficulty_level + 600).min(9_000)
            }
            QuestionVariantMode::MisconceptionProbe => {
                (source_question.difficulty_level + 900).min(9_000)
            }
            QuestionVariantMode::Isomorphic => source_question.difficulty_level,
        };
        let estimated_time_seconds = match variant_mode {
            QuestionVariantMode::Rescue => (source_question.estimated_time_seconds - 10).max(20),
            QuestionVariantMode::Stretch => source_question.estimated_time_seconds + 20,
            QuestionVariantMode::MisconceptionProbe => source_question.estimated_time_seconds + 10,
            _ => source_question.estimated_time_seconds + 5,
        };
        let marks = match variant_mode {
            QuestionVariantMode::Stretch => source_question.marks + 1,
            _ => source_question.marks,
        };
        let transform_summary = match variant_mode {
            QuestionVariantMode::Isomorphic => {
                format!(
                    "Built an isomorphic family variant using multiplier {}",
                    multiplier
                )
            }
            QuestionVariantMode::RepresentationShift => format!(
                "Shifted the same family into a different wording/representation with multiplier {}",
                multiplier
            ),
            QuestionVariantMode::MisconceptionProbe => {
                "Reordered and sharpened the distractors to probe the known misconception"
                    .to_string()
            }
            QuestionVariantMode::Rescue => {
                "Lowered the cognitive load while keeping the same family logic".to_string()
            }
            QuestionVariantMode::Stretch => {
                "Raised the demand with a denser but still family-faithful variant".to_string()
            }
        };

        Ok(TransformedQuestion {
            stem,
            explanation_text,
            difficulty_level,
            estimated_time_seconds,
            marks,
            options,
            transform_summary,
            transform_payload: json!({
                "variant_mode": variant_mode.as_str(),
                "multiplier": multiplier,
                "transform_seed": transform_seed,
                "fraction_mapping": fraction_mapping,
            }),
        })
    }

    fn prepare_candidate_for_insert(
        &self,
        request: &GenerationRequestRow,
        source_question: &SourceQuestion,
        source_options: &[QuestionOption],
        variant_mode: QuestionVariantMode,
        offset: i64,
    ) -> EcoachResult<(TransformedQuestion, BasisPoints)> {
        let question_service = QuestionService::new(self.conn);
        let mut best_candidate = None;

        for attempt in 0..4 {
            let candidate_offset = offset + attempt;
            let transformed = self.transform_question(
                source_question,
                source_options,
                variant_mode,
                candidate_offset,
            )?;
            let fingerprint = fingerprint_for(&transformed.stem, &transformed.options);
            let duplicate = self.detect_duplicate_candidate(
                &question_service,
                &transformed.stem,
                &fingerprint,
                source_question.family_id,
                source_question.topic_id,
            )?;
            let anti_repeat_penalty = if duplicate.is_exact_duplicate {
                5_000
            } else if duplicate.is_near_duplicate {
                2_500
            } else {
                0
            };
            let novelty_score = 10_000u16.saturating_sub(anti_repeat_penalty as u16);
            let decision_score = novelty_score.saturating_sub((anti_repeat_penalty / 2) as u16);
            let decision = if duplicate.is_exact_duplicate {
                "rejected_exact_duplicate"
            } else if duplicate.is_near_duplicate {
                "rejected_near_duplicate"
            } else {
                "accepted"
            };

            self.record_candidate_score(
                request.id,
                source_question.id,
                &transformed.stem,
                &fingerprint,
                duplicate.matched_question_id,
                duplicate.similarity_score,
                novelty_score,
                anti_repeat_penalty as BasisPoints,
                decision_score,
                decision,
            )?;

            if !duplicate.is_exact_duplicate && !duplicate.is_near_duplicate {
                return Ok((transformed, duplicate.similarity_score));
            }

            if best_candidate.is_none() && !duplicate.is_exact_duplicate {
                best_candidate = Some((transformed, duplicate.similarity_score));
            }
        }

        best_candidate.ok_or_else(|| {
            EcoachError::Validation(
                "reactor could not produce a non-duplicate candidate for the request".to_string(),
            )
        })
    }

    fn insert_generated_question(
        &self,
        request: &GenerationRequestRow,
        source_question: &SourceQuestion,
        variant_mode: QuestionVariantMode,
        transformed: &TransformedQuestion,
    ) -> EcoachResult<i64> {
        let source_snapshot =
            serde_json::from_str::<serde_json::Value>(&source_question.intelligence_snapshot)
                .unwrap_or_else(
                    |_| json!({ "raw_snapshot": source_question.intelligence_snapshot }),
                );
        let merged_snapshot = json!({
            "source_snapshot": source_snapshot,
            "reactor": {
                "request_id": request.id,
                "request_kind": request.request_kind,
                "subject_id": request.subject_id,
                "topic_id": request.topic_id,
                "variant_mode": variant_mode.as_str(),
                "rationale": request.rationale,
                "source_question_id": source_question.id,
            }
        });

        self.conn
            .execute(
                "INSERT INTO questions (
                    subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                    explanation_text, difficulty_level, estimated_time_seconds, marks,
                    source_type, source_ref, primary_knowledge_role, primary_cognitive_demand,
                    primary_solve_pattern, primary_pedagogic_function, classification_confidence,
                    intelligence_snapshot, primary_skill_id, cognitive_level, pack_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'generated', ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, NULL)",
                params![
                    source_question.subject_id,
                    source_question.topic_id,
                    source_question.subtopic_id,
                    request.family_id,
                    transformed.stem,
                    source_question.question_format,
                    transformed.explanation_text,
                    transformed.difficulty_level,
                    transformed.estimated_time_seconds,
                    transformed.marks,
                    format!("reactor-request:{}:source:{}", request.id, source_question.id),
                    source_question.primary_knowledge_role,
                    source_question.primary_cognitive_demand,
                    source_question.primary_solve_pattern,
                    source_question.primary_pedagogic_function,
                    source_question.classification_confidence,
                    serde_json::to_string(&merged_snapshot)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    source_question.primary_skill_id,
                    source_question.cognitive_level,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    fn detect_duplicate_candidate(
        &self,
        question_service: &QuestionService<'_>,
        stem: &str,
        fingerprint: &str,
        family_id: i64,
        topic_id: i64,
    ) -> EcoachResult<crate::models::DuplicateCheckResult> {
        let exact_match = self
            .conn
            .query_row(
                "SELECT qln.question_id
                 FROM question_lineage_nodes qln
                 WHERE qln.family_id = ?1 AND qln.fingerprint_text = ?2
                 LIMIT 1",
                params![family_id, fingerprint],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(question_id) = exact_match {
            return Ok(crate::models::DuplicateCheckResult {
                matched_question_id: Some(question_id),
                similarity_score: 10_000,
                is_exact_duplicate: true,
                is_near_duplicate: true,
            });
        }

        question_service.detect_near_duplicate(stem, Some(family_id), Some(topic_id))
    }

    fn record_candidate_score(
        &self,
        request_id: i64,
        source_question_id: i64,
        candidate_stem: &str,
        candidate_fingerprint: &str,
        matched_question_id: Option<i64>,
        similarity_score: BasisPoints,
        novelty_score: BasisPoints,
        anti_repeat_penalty: BasisPoints,
        decision_score: BasisPoints,
        decision: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO reactor_candidate_scores (
                    request_id, source_question_id, candidate_stem, candidate_fingerprint,
                    matched_question_id, similarity_score, novelty_score, anti_repeat_penalty,
                    decision_score, decision
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    request_id,
                    source_question_id,
                    candidate_stem,
                    candidate_fingerprint,
                    matched_question_id,
                    similarity_score,
                    novelty_score,
                    anti_repeat_penalty,
                    decision_score,
                    decision,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_generated_options(
        &self,
        question_id: i64,
        options: &[QuestionOption],
    ) -> EcoachResult<()> {
        for option in options {
            self.conn
                .execute(
                    "INSERT INTO question_options (
                        question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        question_id,
                        option.option_label,
                        option.option_text,
                        if option.is_correct { 1 } else { 0 },
                        option.misconception_id,
                        option.distractor_intent,
                        option.position,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn copy_skill_links(
        &self,
        source_question_id: i64,
        generated_question_id: i64,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_skill_links (question_id, node_id, contribution_weight, is_primary)
                 SELECT ?1, node_id, contribution_weight, is_primary
                 FROM question_skill_links
                 WHERE question_id = ?2",
                params![generated_question_id, source_question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn copy_intelligence_links(
        &self,
        source_question_id: i64,
        generated_question_id: i64,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_intelligence_links (
                    question_id, axis_code, concept_code, confidence_score, is_primary
                 )
                 SELECT ?1, axis_code, concept_code, confidence_score, is_primary
                 FROM question_intelligence_links
                 WHERE question_id = ?2",
                params![generated_question_id, source_question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_lineage_records(
        &self,
        family_id: i64,
        source_question_id: i64,
        generated_question_id: i64,
        variant_mode: QuestionVariantMode,
        transform_summary: &str,
        generated_options: &[QuestionOption],
    ) -> EcoachResult<()> {
        let source_node_id =
            self.ensure_lineage_node(source_question_id, Some(family_id), None, None, None)?;
        let child_question = self
            .get_question(generated_question_id)?
            .ok_or_else(|| EcoachError::NotFound("generated question was not found".to_string()))?;
        let child_node_id = self.ensure_lineage_node(
            generated_question_id,
            Some(family_id),
            Some("variant"),
            Some("generated"),
            Some(fingerprint_for(&child_question.stem, generated_options)),
        )?;

        self.conn
            .execute(
                "INSERT OR IGNORE INTO question_lineage_edges (
                    from_node_id, to_node_id, relation_type, transform_mode, rationale
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    source_node_id,
                    child_node_id,
                    variant_mode.relation_type(),
                    variant_mode.as_str(),
                    transform_summary,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn ensure_lineage_node(
        &self,
        question_id: i64,
        family_id_override: Option<i64>,
        node_role_override: Option<&str>,
        origin_kind_override: Option<&str>,
        fingerprint_override: Option<String>,
    ) -> EcoachResult<i64> {
        if let Some(node_id) = self
            .conn
            .query_row(
                "SELECT id FROM question_lineage_nodes WHERE question_id = ?1",
                [question_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok(node_id);
        }

        let source_question = self.load_source_question(question_id)?.ok_or_else(|| {
            EcoachError::NotFound("question for lineage node was not found".to_string())
        })?;
        let options = self.list_options(question_id)?;
        let fingerprint = fingerprint_override
            .unwrap_or_else(|| fingerprint_for(&source_question.stem, &options));
        let node_role = node_role_override.unwrap_or_else(|| {
            if source_question.source_type == "generated" {
                "variant"
            } else {
                "seed"
            }
        });
        let origin_kind = origin_kind_override.unwrap_or(&source_question.source_type);

        self.conn
            .execute(
                "INSERT INTO question_lineage_nodes (
                    question_id, family_id, lineage_key, node_role, origin_kind, fingerprint_text
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    question_id,
                    family_id_override.or(Some(source_question.family_id)),
                    format!(
                        "family-{}-question-{}",
                        source_question.family_id, question_id
                    ),
                    node_role,
                    origin_kind,
                    fingerprint,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    fn insert_transform_log(
        &self,
        request_id: i64,
        family_id: i64,
        source_question_id: i64,
        generated_question_id: i64,
        variant_mode: QuestionVariantMode,
        transform_summary: &str,
        transform_payload: serde_json::Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_transform_log (
                    request_id, family_id, source_question_id, generated_question_id,
                    variant_mode, transform_summary, transform_payload
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    request_id,
                    family_id,
                    source_question_id,
                    generated_question_id,
                    variant_mode.as_str(),
                    transform_summary,
                    serde_json::to_string(&transform_payload)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_question_graph_edges(
        &self,
        source_question: &SourceQuestion,
        generated_question_id: i64,
        variant_mode: QuestionVariantMode,
        similarity_score: BasisPoints,
    ) -> EcoachResult<()> {
        self.insert_question_graph_edge(
            source_question.id,
            generated_question_id,
            "same_family",
            similarity_score,
            Some("Questions belong to the same family cluster"),
        )?;

        if source_question.primary_skill_id.is_some() {
            self.insert_question_graph_edge(
                source_question.id,
                generated_question_id,
                "same_skill",
                similarity_score,
                Some("Questions share the same primary skill target"),
            )?;
        }

        self.insert_question_graph_edge(
            source_question.id,
            generated_question_id,
            variant_mode_graph_relation(variant_mode),
            similarity_score,
            Some("Generated by the local question reactor"),
        )?;

        if matches!(
            variant_mode,
            QuestionVariantMode::Stretch | QuestionVariantMode::Rescue
        ) {
            self.insert_question_graph_edge(
                source_question.id,
                generated_question_id,
                "difficulty_ladder",
                similarity_score,
                Some("Variant sits on a different difficulty rung"),
            )?;
        }

        if matches!(variant_mode, QuestionVariantMode::MisconceptionProbe) {
            self.insert_question_graph_edge(
                source_question.id,
                generated_question_id,
                "misconception_pair",
                similarity_score,
                Some("Variant explicitly probes the same misconception"),
            )?;
        }

        Ok(())
    }

    fn insert_question_graph_edge(
        &self,
        from_question_id: i64,
        to_question_id: i64,
        relation_type: &str,
        similarity_score: BasisPoints,
        rationale: Option<&str>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO question_graph_edges (
                    from_question_id, to_question_id, relation_type, similarity_score, rationale
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    from_question_id,
                    to_question_id,
                    relation_type,
                    similarity_score,
                    rationale,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn refresh_family_health(&self, family_id: i64) -> EcoachResult<()> {
        let family_exists = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM question_families WHERE id = ?1)",
                [family_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if family_exists == 0 {
            return Ok(());
        }

        let (total_instances, generated_instances, active_instances, last_generated_at): (
            i64,
            i64,
            i64,
            Option<String>,
        ) = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN source_type = 'generated' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN is_active = 1 THEN 1 ELSE 0 END), 0),
                        MAX(CASE WHEN source_type = 'generated' THEN created_at ELSE NULL END)
                 FROM questions
                 WHERE family_id = ?1",
                [family_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (recent_attempts, recent_correct_attempts, avg_response_time_ms, misconception_hit_count): (
            i64,
            i64,
            i64,
            i64,
        ) = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN sqa.is_correct = 1 THEN 1 ELSE 0 END), 0),
                        CAST(COALESCE(AVG(COALESCE(sqa.response_time_ms, 0)), 0) AS INTEGER),
                        COALESCE(SUM(CASE WHEN sqa.misconception_triggered_id IS NOT NULL THEN 1 ELSE 0 END), 0)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE q.family_id = ?1",
                [family_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let accuracy_bp = if recent_attempts > 0 {
            ((recent_correct_attempts * 10_000) / recent_attempts).clamp(0, 10_000)
        } else {
            0
        };
        let response_component = if recent_attempts == 0 {
            4_000
        } else if avg_response_time_ms <= 25_000 {
            8_200
        } else if avg_response_time_ms <= 45_000 {
            7_000
        } else if avg_response_time_ms <= 75_000 {
            5_500
        } else {
            3_800
        };
        let freshness_score = clamp_bp(
            2_800
                + active_instances * 500
                + generated_instances * 1_100
                + total_instances.min(6) * 250,
        );
        let calibration_score = if recent_attempts == 0 {
            clamp_bp(3_500 + total_instances * 250 + generated_instances * 200)
        } else {
            clamp_bp(accuracy_bp / 2 + response_component / 5 + (recent_attempts.min(8) * 700))
        };
        let quality_score = clamp_bp(
            ((accuracy_bp * 7) / 10) + ((response_component * 3) / 10)
                - (misconception_hit_count.min(8) * 180),
        );
        let health_status = if total_instances == 0 {
            "missing"
        } else if recent_attempts < 2 {
            "warming"
        } else if quality_score >= 7_200 && calibration_score >= 6_600 {
            "gold"
        } else if quality_score >= 5_400 {
            "active"
        } else {
            "fragile"
        };

        self.conn
            .execute(
                "INSERT INTO question_family_health (
                    family_id, total_instances, generated_instances, active_instances,
                    recent_attempts, recent_correct_attempts, avg_response_time_ms,
                    misconception_hit_count, freshness_score, calibration_score,
                    quality_score, health_status, last_generated_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))
                 ON CONFLICT(family_id) DO UPDATE SET
                    total_instances = excluded.total_instances,
                    generated_instances = excluded.generated_instances,
                    active_instances = excluded.active_instances,
                    recent_attempts = excluded.recent_attempts,
                    recent_correct_attempts = excluded.recent_correct_attempts,
                    avg_response_time_ms = excluded.avg_response_time_ms,
                    misconception_hit_count = excluded.misconception_hit_count,
                    freshness_score = excluded.freshness_score,
                    calibration_score = excluded.calibration_score,
                    quality_score = excluded.quality_score,
                    health_status = excluded.health_status,
                    last_generated_at = excluded.last_generated_at,
                    updated_at = excluded.updated_at",
                params![
                    family_id,
                    total_instances,
                    generated_instances,
                    active_instances,
                    recent_attempts,
                    recent_correct_attempts,
                    avg_response_time_ms,
                    misconception_hit_count,
                    freshness_score,
                    calibration_score,
                    quality_score,
                    health_status,
                    last_generated_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }
}
