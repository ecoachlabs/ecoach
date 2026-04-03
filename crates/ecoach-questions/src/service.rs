use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    DuplicateCheckResult, Question, QuestionFamilySummary, QuestionIntelligenceFilter,
    QuestionIntelligenceLink, QuestionIntelligenceProfile, QuestionIntelligenceQuery,
    QuestionIntelligenceSnapshot, QuestionMisconceptionTag, QuestionOption,
    QuestionReviewActionInput, QuestionReviewQueueItem, QuestionReviewState, RelatedQuestion,
};

#[derive(Debug, Clone)]
struct QuestionIntelligenceSeed {
    question: Question,
    source_type: String,
    source_ref: Option<String>,
    primary_knowledge_role: Option<String>,
    primary_cognitive_demand: Option<String>,
    primary_solve_pattern: Option<String>,
    primary_pedagogic_function: Option<String>,
    classification_confidence: BasisPoints,
    intelligence_snapshot: Value,
}

#[derive(Debug, Clone)]
struct QuestionProfileRow {
    knowledge_role: Option<String>,
    cognitive_demand: Option<String>,
    solve_pattern: Option<String>,
    pedagogic_function: Option<String>,
    content_grain: Option<String>,
    machine_confidence_score: BasisPoints,
}

pub struct QuestionService<'a> {
    conn: &'a Connection,
}

impl<'a> QuestionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_question(&self, question_id: i64) -> EcoachResult<Option<Question>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks, primary_skill_id
                 FROM questions
                 WHERE id = ?1 AND is_active = 1",
                [question_id],
                map_question,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn list_questions_for_scope(
        &self,
        subject_id: i64,
        topic_ids: &[i64],
    ) -> EcoachResult<Vec<Question>> {
        if topic_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = topic_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                    explanation_text, difficulty_level, estimated_time_seconds, marks, primary_skill_id
             FROM questions
             WHERE is_active = 1 AND subject_id = ?1 AND topic_id IN ({})
             ORDER BY updated_at DESC, id DESC",
            placeholders
        );
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::with_capacity(topic_ids.len() + 1);
        params_vec.push(subject_id.into());
        for topic_id in topic_ids {
            params_vec.push((*topic_id).into());
        }

        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), map_question)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    }

    pub fn get_question_profile(
        &self,
        question_id: i64,
    ) -> EcoachResult<Option<QuestionIntelligenceProfile>> {
        let Some(question) = self.get_question(question_id)? else {
            return Ok(None);
        };
        let links = self.list_intelligence_links(question_id)?;
        Ok(Some(QuestionIntelligenceProfile { question, links }))
    }

    pub fn list_intelligence_links(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionIntelligenceLink>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT qil.axis_code, qil.concept_code, qit.display_name,
                        qil.confidence_score, qil.is_primary
                 FROM question_intelligence_links qil
                 INNER JOIN question_intelligence_taxonomy qit
                    ON qit.axis_code = qil.axis_code
                   AND qit.concept_code = qil.concept_code
                 WHERE qil.question_id = ?1
                 ORDER BY qil.axis_code ASC, qil.is_primary DESC, qil.confidence_score DESC, qit.display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionIntelligenceLink {
                    axis_code: row.get(0)?,
                    concept_code: row.get(1)?,
                    display_name: row.get(2)?,
                    confidence_score: row.get::<_, i64>(3)?.clamp(0, 10_000) as u16,
                    is_primary: row.get::<_, i64>(4)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut links = Vec::new();
        for row in rows {
            links.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(links)
    }

    pub fn find_questions_by_intelligence(
        &self,
        query: &QuestionIntelligenceQuery,
    ) -> EcoachResult<Vec<Question>> {
        let topic_filter_enabled = query.topic_id.is_some();
        let subject_filter_enabled = query.subject_id.is_some();
        let limit = query.limit.max(1) as i64;
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id, q.subject_id, q.topic_id, q.subtopic_id, q.family_id, q.stem,
                        q.question_format, q.explanation_text, q.difficulty_level,
                        q.estimated_time_seconds, q.marks, q.primary_skill_id
                 FROM question_intelligence_links qil
                 INNER JOIN questions q ON q.id = qil.question_id
                 WHERE qil.axis_code = ?1
                   AND qil.concept_code = ?2
                   AND q.is_active = 1
                   AND (?3 = 0 OR q.subject_id = ?4)
                   AND (?5 = 0 OR q.topic_id = ?6)
                 ORDER BY qil.is_primary DESC, qil.confidence_score DESC, q.id ASC
                 LIMIT ?7",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![
                    query.axis_code,
                    query.concept_code,
                    if subject_filter_enabled { 1 } else { 0 },
                    query.subject_id.unwrap_or_default(),
                    if topic_filter_enabled { 1 } else { 0 },
                    query.topic_id.unwrap_or_default(),
                    limit,
                ],
                map_question,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    }

    pub fn list_options(&self, question_id: i64) -> EcoachResult<Vec<QuestionOption>> {
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

    pub fn get_option(&self, option_id: i64) -> EcoachResult<Option<QuestionOption>> {
        self.conn
            .query_row(
                "SELECT id, question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                 FROM question_options WHERE id = ?1",
                [option_id],
                |row| {
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
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_correct_option_text(&self, question_id: i64) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT option_text FROM question_options WHERE question_id = ?1 AND is_correct = 1 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn list_related_questions(
        &self,
        question_id: i64,
        relation_type: Option<&str>,
        limit: usize,
    ) -> EcoachResult<Vec<RelatedQuestion>> {
        let limit = limit.max(1) as i64;
        let relation_filter_enabled = relation_type.is_some();
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id, q.subject_id, q.topic_id, q.subtopic_id, q.family_id, q.stem,
                        q.question_format, q.explanation_text, q.difficulty_level,
                        q.estimated_time_seconds, q.marks, q.primary_skill_id,
                        qge.from_question_id, qge.to_question_id, qge.relation_type,
                        qge.similarity_score, qge.rationale
                 FROM question_graph_edges qge
                 INNER JOIN questions q
                    ON q.id = CASE
                        WHEN qge.from_question_id = ?1 THEN qge.to_question_id
                        ELSE qge.from_question_id
                    END
                 WHERE (qge.from_question_id = ?1 OR qge.to_question_id = ?1)
                   AND (?2 = 0 OR qge.relation_type = ?3)
                   AND q.is_active = 1
                 ORDER BY qge.similarity_score DESC, q.id ASC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![
                    question_id,
                    if relation_filter_enabled { 1 } else { 0 },
                    relation_type.unwrap_or_default(),
                    limit,
                ],
                |row| {
                    Ok(RelatedQuestion {
                        question: Question {
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
                            primary_skill_id: row.get(11)?,
                        },
                        edge: crate::models::QuestionGraphEdge {
                            from_question_id: row.get(12)?,
                            to_question_id: row.get(13)?,
                            relation_type: row.get(14)?,
                            similarity_score: row.get::<_, i64>(15)?.clamp(0, 10_000) as u16,
                            rationale: row.get(16)?,
                        },
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut related = Vec::new();
        for row in rows {
            related.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(related)
    }

    pub fn detect_near_duplicate(
        &self,
        stem: &str,
        family_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<DuplicateCheckResult> {
        let normalized_candidate = normalize_text(stem);
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id, q.stem
                 FROM questions q
                 WHERE q.is_active = 1
                   AND (?1 = 0 OR q.family_id = ?2)
                   AND (?3 = 0 OR q.topic_id = ?4)
                 ORDER BY q.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![
                    if family_id.is_some() { 1 } else { 0 },
                    family_id.unwrap_or_default(),
                    if topic_id.is_some() { 1 } else { 0 },
                    topic_id.unwrap_or_default(),
                ],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut best_match = None;
        let mut best_score = 0u16;
        for row in rows {
            let (question_id, existing_stem) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let normalized_existing = normalize_text(&existing_stem);
            let similarity_score = similarity_bp(&normalized_candidate, &normalized_existing);
            if similarity_score > best_score {
                best_score = similarity_score;
                best_match = Some(question_id);
            }
        }

        Ok(DuplicateCheckResult {
            matched_question_id: best_match,
            similarity_score: best_score,
            is_exact_duplicate: best_score >= 10_000,
            is_near_duplicate: best_score >= 9_200,
        })
    }

    pub fn insert_question(
        &self,
        subject_id: i64,
        topic_id: i64,
        stem: &str,
        question_format: &str,
        difficulty_level: i64,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO questions (
                    subject_id, topic_id, stem, question_format, difficulty_level, estimated_time_seconds
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 30)",
                params![subject_id, topic_id, stem, question_format, difficulty_level],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn classify_question(
        &self,
        question_id: i64,
        reclassify: bool,
    ) -> EcoachResult<QuestionIntelligenceSnapshot> {
        let seed = self
            .load_intelligence_seed(question_id)?
            .ok_or_else(|| EcoachError::NotFound("question was not found".to_string()))?;
        if reclassify {
            self.conn
                .execute(
                    "UPDATE question_reclassification_queue
                     SET status = 'completed', processed_at = datetime('now')
                     WHERE question_id = ?1 AND status IN ('queued', 'processing')",
                    [question_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        let knowledge_role = seed.primary_knowledge_role.clone().or_else(|| {
            infer_knowledge_role(&seed.question.stem, seed.question.question_format.as_str())
        });
        let cognitive_demand = seed.primary_cognitive_demand.clone().or_else(|| {
            infer_cognitive_demand(&seed.question.stem, seed.question.question_format.as_str())
        });
        let solve_pattern = seed
            .primary_solve_pattern
            .clone()
            .or_else(|| infer_solve_pattern(&seed.question.stem));
        let pedagogic_function = seed
            .primary_pedagogic_function
            .clone()
            .or_else(|| infer_pedagogic_function(&seed.question.stem, &knowledge_role));
        let content_grain = infer_content_grain(seed.question.primary_skill_id);
        let inferred_misconceptions =
            self.infer_misconceptions(question_id, &seed.question.stem, &knowledge_role)?;
        let machine_confidence_score = self.compute_machine_confidence(
            seed.classification_confidence,
            &knowledge_role,
            &cognitive_demand,
            &solve_pattern,
            &pedagogic_function,
            &inferred_misconceptions,
            seed.source_type.as_str(),
        );
        let family = self.ensure_family_assignment(
            &seed,
            knowledge_role.as_deref(),
            solve_pattern.as_deref(),
            pedagogic_function.as_deref(),
        )?;
        let review = self.determine_review_state(
            seed.source_type.as_str(),
            seed.source_ref.as_deref(),
            machine_confidence_score,
            family.as_ref(),
            &knowledge_role,
            &cognitive_demand,
            &solve_pattern,
            &pedagogic_function,
        );

        for axis_code in [
            "knowledge_role",
            "cognitive_demand",
            "solve_pattern",
            "pedagogic_function",
            "content_grain",
            "question_family",
            "misconception_exposure",
        ] {
            self.conn
                .execute(
                    "DELETE FROM question_intelligence_links
                     WHERE question_id = ?1 AND axis_code = ?2 AND source = 'system'",
                    params![question_id, axis_code],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.upsert_axis_link(
            question_id,
            "knowledge_role",
            knowledge_role.as_deref(),
            "system",
            true,
        )?;
        self.upsert_axis_link(
            question_id,
            "cognitive_demand",
            cognitive_demand.as_deref(),
            "system",
            true,
        )?;
        self.upsert_axis_link(
            question_id,
            "solve_pattern",
            solve_pattern.as_deref(),
            "system",
            true,
        )?;
        self.upsert_axis_link(
            question_id,
            "pedagogic_function",
            pedagogic_function.as_deref(),
            "system",
            true,
        )?;
        self.upsert_axis_link(
            question_id,
            "content_grain",
            content_grain.as_deref(),
            "system",
            true,
        )?;
        self.upsert_axis_link(
            question_id,
            "question_family",
            family.as_ref().and_then(|item| item.family_code.as_deref()),
            "system",
            true,
        )?;
        for misconception in &inferred_misconceptions {
            self.upsert_axis_link(
                question_id,
                "misconception_exposure",
                Some(misconception.misconception_code.as_str()),
                misconception.source.as_str(),
                false,
            )?;
        }
        self.replace_question_misconceptions(question_id, &inferred_misconceptions)?;

        if let Some(family) = &family {
            if let Some(family_id) = family.family_id {
                self.conn
                    .execute(
                        "UPDATE questions SET family_id = ?2, updated_at = datetime('now') WHERE id = ?1",
                        params![question_id, family_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.ensure_family_member(
                    family_id,
                    question_id,
                    if family.similarity_score >= 8_500 {
                        "canonical"
                    } else {
                        "variant"
                    },
                    family.similarity_score,
                )?;
            }
        }

        let snapshot_json = json!({
            "knowledge_role": knowledge_role,
            "cognitive_demand": cognitive_demand,
            "solve_pattern": solve_pattern,
            "pedagogic_function": pedagogic_function,
            "content_grain": content_grain,
            "misconceptions": inferred_misconceptions,
            "family": family,
            "review": review,
            "seed_snapshot": seed.intelligence_snapshot,
        });
        self.upsert_question_profile(
            question_id,
            review.classification_source.as_str(),
            machine_confidence_score,
            knowledge_role.as_deref(),
            cognitive_demand.as_deref(),
            solve_pattern.as_deref(),
            pedagogic_function.as_deref(),
            content_grain.as_deref(),
            family.as_ref().and_then(|item| item.family_id),
            &review,
            &snapshot_json,
        )?;
        self.conn
            .execute(
                "UPDATE questions
                 SET primary_knowledge_role = COALESCE(?2, primary_knowledge_role),
                     primary_cognitive_demand = COALESCE(?3, primary_cognitive_demand),
                     primary_solve_pattern = COALESCE(?4, primary_solve_pattern),
                     primary_pedagogic_function = COALESCE(?5, primary_pedagogic_function),
                     classification_confidence = ?6,
                     intelligence_snapshot = ?7,
                     classification_source = ?8,
                     classification_confidence_bp = ?6,
                     review_status = ?9,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    question_id,
                    knowledge_role,
                    cognitive_demand,
                    solve_pattern,
                    pedagogic_function,
                    machine_confidence_score as i64,
                    snapshot_json.to_string(),
                    review.classification_source,
                    review.review_status,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.get_question_intelligence(question_id)?.ok_or_else(|| {
            EcoachError::NotFound("question intelligence profile missing".to_string())
        })
    }

    pub fn get_question_intelligence(
        &self,
        question_id: i64,
    ) -> EcoachResult<Option<QuestionIntelligenceSnapshot>> {
        let Some(question) = self.get_question(question_id)? else {
            return Ok(None);
        };
        let links = self.list_intelligence_links(question_id)?;
        let family = self.load_family_summary(question_id)?;
        let misconceptions = self.list_question_misconceptions(question_id)?;
        let review = self.load_review_state(question_id)?;
        let snapshot = self
            .conn
            .query_row(
                "SELECT snapshot_json FROM question_intelligence_profiles WHERE question_id = ?1",
                [question_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .map(|raw| serde_json::from_str::<Value>(&raw).unwrap_or(Value::Null))
            .unwrap_or(Value::Null);
        let profile_row = self.load_profile_row(question_id)?;

        Ok(Some(QuestionIntelligenceSnapshot {
            question,
            knowledge_role: profile_row.knowledge_role,
            cognitive_demand: profile_row.cognitive_demand,
            solve_pattern: profile_row.solve_pattern,
            pedagogic_function: profile_row.pedagogic_function,
            content_grain: profile_row.content_grain,
            machine_confidence_score: profile_row.machine_confidence_score,
            family,
            misconceptions,
            review,
            links,
            snapshot,
        }))
    }

    pub fn find_questions_by_intelligence_filter(
        &self,
        filter: &QuestionIntelligenceFilter,
    ) -> EcoachResult<Vec<QuestionIntelligenceSnapshot>> {
        let limit = filter.limit.max(1) as i64;
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id
                 FROM questions q
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                 LEFT JOIN question_misconceptions qm ON qm.question_id = q.id
                 LEFT JOIN question_family_members qfm ON qfm.question_id = q.id
                 LEFT JOIN question_intelligence_links qil ON qil.question_id = q.id
                 WHERE q.is_active = 1
                   AND (?1 IS NULL OR q.subject_id = ?1)
                   AND (?2 IS NULL OR q.topic_id = ?2)
                   AND (?3 IS NULL OR q.family_id = ?3)
                   AND (?4 IS NULL OR qm.misconception_code = ?4)
                   AND (?5 IS NULL OR qip.review_status = ?5)
                   AND (?6 = 0 OR qip.review_status = 'approved')
                   AND (?7 IS NULL OR qil.axis_code = ?7)
                   AND (?8 IS NULL OR qil.concept_code = ?8)
                 GROUP BY q.id
                 ORDER BY COALESCE(qip.machine_confidence_bp, 0) DESC, q.id ASC
                 LIMIT ?9",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![
                    filter.subject_id,
                    filter.topic_id,
                    filter.family_id,
                    filter.misconception_code,
                    filter.review_status,
                    if filter.reviewed_only { 1 } else { 0 },
                    filter.axis_code,
                    filter.concept_code,
                    limit,
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut seen_families = std::collections::BTreeSet::new();
        let mut results = Vec::new();
        for row in rows {
            let question_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(snapshot) = self.get_question_intelligence(question_id)? {
                if filter.exclude_family_duplicates {
                    if let Some(family_id) =
                        snapshot.family.as_ref().and_then(|item| item.family_id)
                    {
                        if !seen_families.insert(family_id) {
                            continue;
                        }
                    }
                }
                results.push(snapshot);
            }
        }
        Ok(results)
    }

    pub fn list_question_review_queue(
        &self,
        review_status: Option<&str>,
        limit: usize,
    ) -> EcoachResult<Vec<QuestionReviewQueueItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id, q.stem, q.topic_id, qip.machine_confidence_bp,
                        qip.review_status, qip.review_reason
                 FROM question_intelligence_profiles qip
                 INNER JOIN questions q ON q.id = qip.question_id
                 WHERE q.is_active = 1
                   AND qip.review_status <> 'approved'
                   AND (?1 IS NULL OR qip.review_status = ?1)
                 ORDER BY qip.machine_confidence_bp ASC, q.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![review_status, limit.max(1) as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            let (question_id, stem, topic_id, confidence, status, reason) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(QuestionReviewQueueItem {
                question_id,
                stem,
                topic_id,
                machine_confidence_score: confidence.clamp(0, 10_000) as BasisPoints,
                review_status: status,
                review_reason: reason,
                family_candidate: self.load_family_summary(question_id)?,
                misconception_candidates: self.list_question_misconceptions(question_id)?,
            });
        }
        Ok(items)
    }

    pub fn review_question_intelligence(
        &self,
        question_id: i64,
        input: &QuestionReviewActionInput,
    ) -> EcoachResult<QuestionIntelligenceSnapshot> {
        let before_snapshot = self
            .get_question_intelligence(question_id)?
            .ok_or_else(|| {
                EcoachError::NotFound("question intelligence was not found".to_string())
            })?;
        let previous_status = before_snapshot.review.review_status.clone();
        self.apply_review_overrides(question_id, input)?;
        let reviewed = self
            .get_question_intelligence(question_id)?
            .ok_or_else(|| {
                EcoachError::NotFound("question intelligence was not found".to_string())
            })?;
        self.conn
            .execute(
                "INSERT INTO question_intelligence_reviews (
                    question_id, reviewer_id, action_code, previous_review_status,
                    new_review_status, note, previous_snapshot_json, new_snapshot_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
                params![
                    question_id,
                    input.reviewer_id,
                    input.action_code,
                    previous_status,
                    reviewed.review.review_status,
                    input.note,
                    serde_json::to_string(&before_snapshot)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&reviewed)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if input.request_reclassification || input.action_code == "send_for_reclassification" {
            self.queue_question_reclassification(
                question_id,
                "manual_review",
                Some(&input.reviewer_id),
            )?;
        }

        Ok(reviewed)
    }

    fn load_intelligence_seed(
        &self,
        question_id: i64,
    ) -> EcoachResult<Option<QuestionIntelligenceSeed>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks,
                        primary_skill_id, COALESCE(source_type, 'authored'), source_ref,
                        primary_knowledge_role, primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function,
                        COALESCE(classification_confidence_bp, classification_confidence, 0),
                        intelligence_snapshot
                 FROM questions
                 WHERE id = ?1 AND is_active = 1",
                [question_id],
                |row| {
                    let raw_snapshot = row.get::<_, String>(19)?;
                    Ok(QuestionIntelligenceSeed {
                        question: Question {
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
                            primary_skill_id: row.get(11)?,
                        },
                        source_type: row.get(12)?,
                        source_ref: row.get(13)?,
                        primary_knowledge_role: row.get(14)?,
                        primary_cognitive_demand: row.get(15)?,
                        primary_solve_pattern: row.get(16)?,
                        primary_pedagogic_function: row.get(17)?,
                        classification_confidence: row.get::<_, i64>(18)?.clamp(0, 10_000)
                            as BasisPoints,
                        intelligence_snapshot: serde_json::from_str(&raw_snapshot)
                            .unwrap_or(Value::Null),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn infer_misconceptions(
        &self,
        question_id: i64,
        stem: &str,
        knowledge_role: &Option<String>,
    ) -> EcoachResult<Vec<QuestionMisconceptionTag>> {
        let mut ranked = std::collections::BTreeMap::<String, QuestionMisconceptionTag>::new();

        let mut option_statement = self
            .conn
            .prepare(
                "SELECT mp.title, mp.severity
                 FROM question_options qo
                 INNER JOIN misconception_patterns mp ON mp.id = qo.misconception_id
                 WHERE qo.question_id = ?1
                 ORDER BY mp.severity DESC, mp.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let option_rows = option_statement
            .query_map([question_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in option_rows {
            let (title, severity) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let code = normalize_axis_value("misconception_exposure", &title);
            upsert_misconception_candidate(
                &mut ranked,
                code,
                clamp_bp(severity + 1_000),
                "pack".to_string(),
            );
        }

        let mut link_statement = self
            .conn
            .prepare(
                "SELECT concept_code, confidence_score, source
                 FROM question_intelligence_links
                 WHERE question_id = ?1 AND axis_code = 'misconception_exposure'
                 ORDER BY confidence_score DESC, concept_code ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let link_rows = link_statement
            .query_map([question_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in link_rows {
            let (code, confidence, source) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            upsert_misconception_candidate(
                &mut ranked,
                normalize_axis_value("misconception_exposure", &code),
                clamp_bp(confidence),
                source,
            );
        }

        if ranked.is_empty() {
            upsert_misconception_candidate(
                &mut ranked,
                infer_default_misconception(stem, knowledge_role.as_deref()),
                5_600,
                "system".to_string(),
            );
        }

        let mut results = ranked.into_values().collect::<Vec<_>>();
        results.sort_by(|left, right| {
            right
                .confidence_score
                .cmp(&left.confidence_score)
                .then(left.misconception_code.cmp(&right.misconception_code))
        });
        results.truncate(6);
        Ok(results)
    }

    fn compute_machine_confidence(
        &self,
        seed_confidence: BasisPoints,
        knowledge_role: &Option<String>,
        cognitive_demand: &Option<String>,
        solve_pattern: &Option<String>,
        pedagogic_function: &Option<String>,
        misconceptions: &[QuestionMisconceptionTag],
        source_type: &str,
    ) -> BasisPoints {
        let mut score = i64::from(seed_confidence.max(4_500));
        if knowledge_role.is_some() {
            score += 700;
        }
        if cognitive_demand.is_some() {
            score += 600;
        }
        if solve_pattern.is_some() {
            score += 600;
        }
        if pedagogic_function.is_some() {
            score += 550;
        }
        if !misconceptions.is_empty() {
            score += 400;
        }
        match source_type {
            "past_question" => score += 300,
            "generated" => score -= 250,
            "teacher_upload" => score -= 150,
            _ => {}
        }
        clamp_bp(score)
    }

    fn ensure_family_assignment(
        &self,
        seed: &QuestionIntelligenceSeed,
        knowledge_role: Option<&str>,
        solve_pattern: Option<&str>,
        pedagogic_function: Option<&str>,
    ) -> EcoachResult<Option<QuestionFamilySummary>> {
        if let Some(family_id) = seed.question.family_id {
            let family = self
                .conn
                .query_row(
                    "SELECT id, family_code, family_name, family_type
                     FROM question_families
                     WHERE id = ?1",
                    [family_id],
                    |row| {
                        Ok(QuestionFamilySummary {
                            family_id: Some(row.get(0)?),
                            family_code: row.get(1)?,
                            family_name: row.get(2)?,
                            family_type: row.get(3)?,
                            similarity_score: 10_000,
                        })
                    },
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            return Ok(family);
        }

        let topic_code = self
            .conn
            .query_row(
                "SELECT code FROM topics WHERE id = ?1",
                [seed.question.topic_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten()
            .unwrap_or_else(|| "QGEN".to_string());

        let normalized_stem = normalize_text(&seed.question.stem);
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, family_code, family_name, family_type, canonical_pattern
                 FROM question_families
                 WHERE subject_id = ?1
                   AND (topic_id = ?2 OR topic_id IS NULL)
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![seed.question.subject_id, seed.question.topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut best_match = None;
        let mut best_score = 0;
        for row in rows {
            let (family_id, family_code, family_name, family_type, canonical_pattern) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let reference = canonical_pattern.unwrap_or_else(|| family_name.clone());
            let mut score = i64::from(similarity_bp(&normalized_stem, &normalize_text(&reference)));
            if matches!(
                (knowledge_role, family_type.as_str()),
                (Some("worked_example"), "worked_example_template")
            ) {
                score += 500;
            }
            if matches!(
                (pedagogic_function, family_type.as_str()),
                (Some("misconception_diagnosis"), "misconception_cluster")
            ) {
                score += 500;
            }
            if matches!(solve_pattern, Some("multi_step_reasoning" | "proof_chain"))
                && family_type == "exam_structure"
            {
                score += 350;
            }
            let clamped_score = clamp_bp(score);
            if clamped_score > best_score {
                best_score = clamped_score;
                best_match = Some(QuestionFamilySummary {
                    family_id: Some(family_id),
                    family_code: Some(family_code),
                    family_name: Some(family_name),
                    family_type: Some(family_type),
                    similarity_score: clamped_score,
                });
            }
        }

        if best_score >= 6_400 {
            return Ok(best_match);
        }

        let family_code = build_family_code(&topic_code, &seed.question.stem, seed.question.id);
        let family_name = summarize_stem(&seed.question.stem);
        let family_type = derive_family_type(knowledge_role, pedagogic_function);
        self.conn
            .execute(
                "INSERT INTO question_families (
                    family_code, family_name, subject_id, topic_id, subtopic_id, family_type,
                    canonical_pattern, description, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))",
                params![
                    family_code,
                    family_name,
                    seed.question.subject_id,
                    seed.question.topic_id,
                    seed.question.subtopic_id,
                    family_type,
                    seed.question.stem,
                    format!(
                        "Auto-derived family for {}",
                        summarize_stem(&seed.question.stem)
                    ),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let family_id = self.conn.last_insert_rowid();
        Ok(Some(QuestionFamilySummary {
            family_id: Some(family_id),
            family_code: Some(build_family_code(
                &topic_code,
                &seed.question.stem,
                seed.question.id,
            )),
            family_name: Some(summarize_stem(&seed.question.stem)),
            family_type: Some(family_type.to_string()),
            similarity_score: 7_600,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    fn determine_review_state(
        &self,
        source_type: &str,
        source_ref: Option<&str>,
        machine_confidence_score: BasisPoints,
        family: Option<&QuestionFamilySummary>,
        knowledge_role: &Option<String>,
        cognitive_demand: &Option<String>,
        solve_pattern: &Option<String>,
        pedagogic_function: &Option<String>,
    ) -> QuestionReviewState {
        let classification_source =
            derive_classification_source(source_type, source_ref).to_string();
        let (review_status, review_reason) = if family.is_none() {
            (
                "family_unresolved".to_string(),
                Some("Question family could not be resolved confidently.".to_string()),
            )
        } else if knowledge_role.is_none()
            || cognitive_demand.is_none()
            || solve_pattern.is_none()
            || pedagogic_function.is_none()
        {
            (
                "taxonomy_gap".to_string(),
                Some("One or more intelligence axes are still missing.".to_string()),
            )
        } else if machine_confidence_score < 6_000 {
            (
                "needs_review".to_string(),
                Some("Machine confidence is below the review threshold.".to_string()),
            )
        } else if source_type == "generated" && machine_confidence_score < 7_400 {
            (
                "needs_review".to_string(),
                Some(
                    "Generated questions stay in review until a stronger pattern match exists."
                        .to_string(),
                ),
            )
        } else if source_type == "teacher_upload" {
            (
                "pending".to_string(),
                Some("Teacher-uploaded questions should be reviewed before approval.".to_string()),
            )
        } else {
            ("approved".to_string(), None)
        };

        QuestionReviewState {
            needs_review: review_status != "approved",
            review_status,
            review_reason,
            reviewer_id: None,
            reviewed_at: None,
            classification_source,
            taxonomy_version: "qi_taxonomy_v1".to_string(),
            classification_version: "qi_engine_v1".to_string(),
        }
    }

    pub fn queue_question_reclassification(
        &self,
        question_id: i64,
        trigger_reason: &str,
        requested_by: Option<&str>,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO question_reclassification_queue (
                    question_id, trigger_reason, status, requested_by, created_at
                 ) VALUES (?1, ?2, 'queued', ?3, datetime('now'))",
                params![question_id, trigger_reason, requested_by],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE question_intelligence_profiles
                 SET needs_reclassification = 1,
                     updated_at = datetime('now')
                 WHERE question_id = ?1",
                [question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn upsert_axis_link(
        &self,
        question_id: i64,
        axis_code: &str,
        concept_code: Option<&str>,
        source: &str,
        is_primary: bool,
    ) -> EcoachResult<()> {
        let Some(concept_code) = concept_code else {
            return Ok(());
        };
        let normalized = normalize_axis_value(axis_code, concept_code);
        self.conn
            .execute(
                "INSERT INTO question_intelligence_taxonomy (
                    axis_code, concept_code, display_name, description
                 ) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(axis_code, concept_code) DO UPDATE SET
                    display_name = excluded.display_name,
                    updated_at = datetime('now')",
                params![
                    axis_code,
                    normalized,
                    taxonomy_display_name(&normalized),
                    format!(
                        "Generated by the idea22 question-intelligence engine for {}.",
                        axis_code
                    ),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if is_primary {
            self.conn
                .execute(
                    "UPDATE question_intelligence_links
                     SET is_primary = 0
                     WHERE question_id = ?1 AND axis_code = ?2 AND source = ?3",
                    params![question_id, axis_code, source],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        self.conn
            .execute(
                "INSERT INTO question_intelligence_links (
                    question_id, axis_code, concept_code, confidence_score, is_primary, source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(question_id, axis_code, concept_code) DO UPDATE SET
                    confidence_score = excluded.confidence_score,
                    is_primary = excluded.is_primary,
                    source = excluded.source",
                params![
                    question_id,
                    axis_code,
                    normalized,
                    if is_primary { 8_500 } else { 6_800 },
                    if is_primary { 1 } else { 0 },
                    source,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn replace_question_misconceptions(
        &self,
        question_id: i64,
        misconceptions: &[QuestionMisconceptionTag],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM question_misconceptions
                 WHERE question_id = ?1 AND source IN ('pack', 'system')",
                [question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for misconception in misconceptions {
            self.conn
                .execute(
                    "INSERT INTO question_misconceptions (
                        question_id, misconception_code, confidence_score_bp, source
                     ) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(question_id, misconception_code) DO UPDATE SET
                        confidence_score_bp = excluded.confidence_score_bp,
                        source = excluded.source",
                    params![
                        question_id,
                        normalize_axis_value(
                            "misconception_exposure",
                            &misconception.misconception_code
                        ),
                        misconception.confidence_score as i64,
                        misconception.source,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn ensure_family_member(
        &self,
        family_id: i64,
        question_id: i64,
        role_in_family: &str,
        similarity_score: BasisPoints,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_family_members (
                    question_family_id, question_id, role_in_family, similarity_score_bp
                 ) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(question_family_id, question_id) DO UPDATE SET
                    role_in_family = excluded.role_in_family,
                    similarity_score_bp = excluded.similarity_score_bp",
                params![
                    family_id,
                    question_id,
                    role_in_family,
                    similarity_score as i64
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn upsert_question_profile(
        &self,
        question_id: i64,
        classification_source: &str,
        machine_confidence_score: BasisPoints,
        knowledge_role: Option<&str>,
        cognitive_demand: Option<&str>,
        solve_pattern: Option<&str>,
        pedagogic_function: Option<&str>,
        content_grain: Option<&str>,
        family_id: Option<i64>,
        review: &QuestionReviewState,
        snapshot_json: &Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_intelligence_profiles (
                    question_id, taxonomy_version, classification_version, family_engine_version,
                    classification_source, machine_confidence_bp, primary_knowledge_role,
                    primary_cognitive_demand, primary_solve_pattern, primary_pedagogic_function,
                    primary_content_grain, question_family_id, review_status, review_reason,
                    reviewer_id, reviewed_at, needs_reclassification, snapshot_json, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, 'qi_family_v1', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                    ?14, ?15, 0, ?16, datetime('now'), datetime('now')
                 )
                 ON CONFLICT(question_id) DO UPDATE SET
                    taxonomy_version = excluded.taxonomy_version,
                    classification_version = excluded.classification_version,
                    family_engine_version = excluded.family_engine_version,
                    classification_source = excluded.classification_source,
                    machine_confidence_bp = excluded.machine_confidence_bp,
                    primary_knowledge_role = excluded.primary_knowledge_role,
                    primary_cognitive_demand = excluded.primary_cognitive_demand,
                    primary_solve_pattern = excluded.primary_solve_pattern,
                    primary_pedagogic_function = excluded.primary_pedagogic_function,
                    primary_content_grain = excluded.primary_content_grain,
                    question_family_id = excluded.question_family_id,
                    review_status = excluded.review_status,
                    review_reason = excluded.review_reason,
                    reviewer_id = excluded.reviewer_id,
                    reviewed_at = excluded.reviewed_at,
                    needs_reclassification = excluded.needs_reclassification,
                    snapshot_json = excluded.snapshot_json,
                    updated_at = datetime('now')",
                params![
                    question_id,
                    review.taxonomy_version,
                    review.classification_version,
                    classification_source,
                    machine_confidence_score as i64,
                    knowledge_role,
                    cognitive_demand,
                    solve_pattern,
                    pedagogic_function,
                    content_grain,
                    family_id,
                    review.review_status,
                    review.review_reason,
                    review.reviewer_id,
                    review.reviewed_at,
                    serde_json::to_string(snapshot_json)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_family_summary(&self, question_id: i64) -> EcoachResult<Option<QuestionFamilySummary>> {
        Ok(self
            .conn
            .query_row(
                "SELECT qf.id, qf.family_code, qf.family_name, qf.family_type,
                        COALESCE(qfm.similarity_score_bp, 0)
                 FROM questions q
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                 LEFT JOIN question_families qf
                    ON qf.id = COALESCE(qip.question_family_id, q.family_id)
                 LEFT JOIN question_family_members qfm
                    ON qfm.question_id = q.id
                   AND qfm.question_family_id = qf.id
                 WHERE q.id = ?1",
                [question_id],
                |row| {
                    let family_id = row.get::<_, Option<i64>>(0)?;
                    let Some(id) = family_id else {
                        return Ok(None);
                    };
                    Ok(Some(QuestionFamilySummary {
                        family_id: Some(id),
                        family_code: row.get(1)?,
                        family_name: row.get(2)?,
                        family_type: row.get(3)?,
                        similarity_score: row.get::<_, i64>(4)?.clamp(0, 10_000) as BasisPoints,
                    }))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten())
    }

    fn list_question_misconceptions(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionMisconceptionTag>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT misconception_code, confidence_score_bp, source
                 FROM question_misconceptions
                 WHERE question_id = ?1
                 ORDER BY confidence_score_bp DESC, misconception_code ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionMisconceptionTag {
                    misconception_code: row.get(0)?,
                    confidence_score: row.get::<_, i64>(1)?.clamp(0, 10_000) as BasisPoints,
                    source: row.get(2)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn load_review_state(&self, question_id: i64) -> EcoachResult<QuestionReviewState> {
        let review = self
            .conn
            .query_row(
                "SELECT review_status, review_reason, reviewer_id, reviewed_at,
                        classification_source, taxonomy_version, classification_version
                 FROM question_intelligence_profiles
                 WHERE question_id = ?1",
                [question_id],
                |row| {
                    let review_status = row.get::<_, String>(0)?;
                    Ok(QuestionReviewState {
                        needs_review: review_status != "approved",
                        review_status,
                        review_reason: row.get(1)?,
                        reviewer_id: row.get(2)?,
                        reviewed_at: row.get(3)?,
                        classification_source: row.get(4)?,
                        taxonomy_version: row.get(5)?,
                        classification_version: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(review) = review {
            return Ok(review);
        }

        let fallback = self
            .conn
            .query_row(
                "SELECT COALESCE(review_status, 'pending'),
                        COALESCE(classification_source, COALESCE(source_type, 'authored')),
                        source_ref
                 FROM questions
                 WHERE id = ?1",
                [question_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let (review_status, classification_source, source_ref) =
            fallback.unwrap_or_else(|| ("pending".to_string(), "rules".to_string(), None));
        Ok(QuestionReviewState {
            needs_review: review_status != "approved",
            review_status,
            review_reason: None,
            reviewer_id: None,
            reviewed_at: None,
            classification_source: if classification_source == "generated" {
                derive_classification_source("generated", source_ref.as_deref()).to_string()
            } else {
                classification_source
            },
            taxonomy_version: "qi_taxonomy_v1".to_string(),
            classification_version: "qi_engine_v1".to_string(),
        })
    }

    fn load_profile_row(&self, question_id: i64) -> EcoachResult<QuestionProfileRow> {
        let profile = self
            .conn
            .query_row(
                "SELECT primary_knowledge_role, primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function, primary_content_grain, machine_confidence_bp
                 FROM question_intelligence_profiles
                 WHERE question_id = ?1",
                [question_id],
                |row| {
                    Ok(QuestionProfileRow {
                        knowledge_role: row.get(0)?,
                        cognitive_demand: row.get(1)?,
                        solve_pattern: row.get(2)?,
                        pedagogic_function: row.get(3)?,
                        content_grain: row.get(4)?,
                        machine_confidence_score: row.get::<_, i64>(5)?.clamp(0, 10_000)
                            as BasisPoints,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(profile) = profile {
            return Ok(profile);
        }
        self.conn
            .query_row(
                "SELECT primary_knowledge_role, primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function, primary_content_type,
                        COALESCE(classification_confidence_bp, classification_confidence, 0)
                 FROM questions
                 WHERE id = ?1",
                [question_id],
                |row| {
                    Ok(QuestionProfileRow {
                        knowledge_role: row.get(0)?,
                        cognitive_demand: row.get(1)?,
                        solve_pattern: row.get(2)?,
                        pedagogic_function: row.get(3)?,
                        content_grain: row.get(4)?,
                        machine_confidence_score: row.get::<_, i64>(5)?.clamp(0, 10_000)
                            as BasisPoints,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn apply_review_overrides(
        &self,
        question_id: i64,
        input: &QuestionReviewActionInput,
    ) -> EcoachResult<()> {
        for axis_code in [
            "knowledge_role",
            "cognitive_demand",
            "solve_pattern",
            "pedagogic_function",
            "content_grain",
            "question_family",
            "misconception_exposure",
        ] {
            self.conn
                .execute(
                    "DELETE FROM question_intelligence_links
                     WHERE question_id = ?1 AND axis_code = ?2 AND source = 'review'",
                    params![question_id, axis_code],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        if input.primary_knowledge_role.is_some() {
            self.upsert_axis_link(
                question_id,
                "knowledge_role",
                input.primary_knowledge_role.as_deref(),
                "review",
                true,
            )?;
        }
        if input.primary_cognitive_demand.is_some() {
            self.upsert_axis_link(
                question_id,
                "cognitive_demand",
                input.primary_cognitive_demand.as_deref(),
                "review",
                true,
            )?;
        }
        if input.primary_solve_pattern.is_some() {
            self.upsert_axis_link(
                question_id,
                "solve_pattern",
                input.primary_solve_pattern.as_deref(),
                "review",
                true,
            )?;
        }
        if input.primary_pedagogic_function.is_some() {
            self.upsert_axis_link(
                question_id,
                "pedagogic_function",
                input.primary_pedagogic_function.as_deref(),
                "review",
                true,
            )?;
        }
        if input.primary_content_grain.is_some() {
            self.upsert_axis_link(
                question_id,
                "content_grain",
                input.primary_content_grain.as_deref(),
                "review",
                true,
            )?;
        }

        self.conn
            .execute(
                "DELETE FROM question_misconceptions
                 WHERE question_id = ?1 AND source = 'review'",
                [question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for misconception_code in &input.misconception_codes {
            let normalized = normalize_axis_value("misconception_exposure", misconception_code);
            self.upsert_axis_link(
                question_id,
                "misconception_exposure",
                Some(&normalized),
                "review",
                false,
            )?;
            self.conn
                .execute(
                    "INSERT INTO question_misconceptions (
                        question_id, misconception_code, confidence_score_bp, source
                     ) VALUES (?1, ?2, 9500, 'review')
                     ON CONFLICT(question_id, misconception_code) DO UPDATE SET
                        confidence_score_bp = 9500,
                        source = 'review'",
                    params![question_id, normalized],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        let family_id = if let Some(family_id) = input.family_id {
            let family_code: Option<String> = self
                .conn
                .query_row(
                    "SELECT family_code FROM question_families WHERE id = ?1",
                    [family_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.upsert_axis_link(
                question_id,
                "question_family",
                family_code.as_deref(),
                "review",
                true,
            )?;
            self.conn
                .execute(
                    "UPDATE questions SET family_id = ?2, updated_at = datetime('now') WHERE id = ?1",
                    params![question_id, family_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.ensure_family_member(family_id, question_id, "canonical", 9_500)?;
            Some(family_id)
        } else {
            self.conn
                .query_row(
                    "SELECT COALESCE(qip.question_family_id, q.family_id)
                     FROM questions q
                     LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                     WHERE q.id = ?1",
                    [question_id],
                    |row| row.get::<_, Option<i64>>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .flatten()
        };

        let current_profile = self.load_profile_row(question_id)?;
        let knowledge_role = input
            .primary_knowledge_role
            .clone()
            .or(current_profile.knowledge_role);
        let cognitive_demand = input
            .primary_cognitive_demand
            .clone()
            .or(current_profile.cognitive_demand);
        let solve_pattern = input
            .primary_solve_pattern
            .clone()
            .or(current_profile.solve_pattern);
        let pedagogic_function = input
            .primary_pedagogic_function
            .clone()
            .or(current_profile.pedagogic_function);
        let content_grain = input
            .primary_content_grain
            .clone()
            .or(current_profile.content_grain);
        let review_status = input
            .review_status
            .clone()
            .unwrap_or_else(|| review_status_for_action(&input.action_code).to_string());
        let machine_confidence_score = if input.action_code == "approve" {
            current_profile.machine_confidence_score.max(9_000)
        } else {
            current_profile.machine_confidence_score
        };
        let snapshot_json = json!({
            "review_override": {
                "action_code": input.action_code,
                "reviewer_id": input.reviewer_id,
                "knowledge_role": knowledge_role,
                "cognitive_demand": cognitive_demand,
                "solve_pattern": solve_pattern,
                "pedagogic_function": pedagogic_function,
                "content_grain": content_grain,
                "family_id": family_id,
                "misconceptions": input.misconception_codes,
                "note": input.note,
            }
        });

        self.conn
            .execute(
                "UPDATE questions
                 SET primary_knowledge_role = COALESCE(?2, primary_knowledge_role),
                     primary_cognitive_demand = COALESCE(?3, primary_cognitive_demand),
                     primary_solve_pattern = COALESCE(?4, primary_solve_pattern),
                     primary_pedagogic_function = COALESCE(?5, primary_pedagogic_function),
                     classification_source = 'review',
                     review_status = ?6,
                     human_verified = 1,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    question_id,
                    knowledge_role,
                    cognitive_demand,
                    solve_pattern,
                    pedagogic_function,
                    review_status,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.upsert_question_profile(
            question_id,
            "review",
            machine_confidence_score,
            knowledge_role.as_deref(),
            cognitive_demand.as_deref(),
            solve_pattern.as_deref(),
            pedagogic_function.as_deref(),
            content_grain.as_deref(),
            family_id,
            &QuestionReviewState {
                needs_review: review_status != "approved",
                review_status,
                review_reason: input.note.clone(),
                reviewer_id: Some(input.reviewer_id.clone()),
                reviewed_at: Some("now".to_string()),
                classification_source: "review".to_string(),
                taxonomy_version: "qi_taxonomy_v1".to_string(),
                classification_version: "qi_engine_v1".to_string(),
            },
            &snapshot_json,
        )?;
        self.conn
            .execute(
                "UPDATE question_intelligence_profiles
                 SET reviewer_id = ?2,
                     reviewed_at = datetime('now'),
                     review_reason = ?3,
                     needs_reclassification = ?4,
                     updated_at = datetime('now')
                 WHERE question_id = ?1",
                params![
                    question_id,
                    input.reviewer_id,
                    input.note,
                    if input.request_reclassification { 1 } else { 0 },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn map_question(row: &rusqlite::Row<'_>) -> rusqlite::Result<Question> {
    Ok(Question {
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
        primary_skill_id: row.get(11)?,
    })
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_ascii_whitespace() {
                Some(' ')
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn similarity_bp(left: &str, right: &str) -> u16 {
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    if left == right {
        return 10_000;
    }

    let left_tokens = left
        .split_whitespace()
        .collect::<std::collections::BTreeSet<_>>();
    let right_tokens = right
        .split_whitespace()
        .collect::<std::collections::BTreeSet<_>>();
    let intersection = left_tokens.intersection(&right_tokens).count() as f64;
    let union = left_tokens.union(&right_tokens).count() as f64;
    if union == 0.0 {
        0
    } else {
        ((intersection / union) * 10_000.0)
            .round()
            .clamp(0.0, 10_000.0) as u16
    }
}

fn infer_knowledge_role(stem: &str, question_format: &str) -> Option<String> {
    let normalized = normalize_text(stem);
    if contains_any(&normalized, &["define", "definition", "means"]) {
        Some("definition".to_string())
    } else if contains_any(&normalized, &["compare", "difference", "similar"]) {
        Some("comparison".to_string())
    } else if contains_any(&normalized, &["explain", "why", "reason"]) {
        Some("explanation".to_string())
    } else if contains_any(&normalized, &["formula", "equation"]) {
        Some("formula_recall".to_string())
    } else if contains_any(&normalized, &["calculate", "solve", "work out", "find"]) {
        Some("application".to_string())
    } else if question_format == "mcq" {
        Some("key_concept".to_string())
    } else {
        Some("procedure".to_string())
    }
}

fn infer_cognitive_demand(stem: &str, question_format: &str) -> Option<String> {
    let normalized = normalize_text(stem);
    if contains_any(&normalized, &["why", "justify", "prove", "show that"]) {
        Some("justification".to_string())
    } else if contains_any(&normalized, &["analyze", "compare", "contrast"]) {
        Some("analysis".to_string())
    } else if contains_any(&normalized, &["explain", "interpret"]) {
        Some("comprehension".to_string())
    } else if contains_any(&normalized, &["calculate", "solve", "work out", "find"]) {
        Some("application".to_string())
    } else if question_format == "mcq" {
        Some("recognition".to_string())
    } else {
        Some("recall".to_string())
    }
}

fn infer_solve_pattern(stem: &str) -> Option<String> {
    let normalized = normalize_text(stem);
    if contains_any(&normalized, &["table", "graph", "chart"]) {
        Some("graph_or_table_reading".to_string())
    } else if contains_any(&normalized, &["prove", "justify", "show that"]) {
        Some("proof_chain".to_string())
    } else if contains_any(&normalized, &["pattern", "sequence", "equivalent"]) {
        Some("pattern_spotting".to_string())
    } else if contains_any(&normalized, &["calculate", "solve", "work out"]) {
        Some("substitute_and_solve".to_string())
    } else if contains_any(&normalized, &["explain", "reason"]) {
        Some("multi_step_reasoning".to_string())
    } else {
        Some("direct_retrieval".to_string())
    }
}

fn infer_pedagogic_function(stem: &str, knowledge_role: &Option<String>) -> Option<String> {
    let normalized = normalize_text(stem);
    if contains_any(&normalized, &["trap", "mistake", "wrong"]) {
        Some("misconception_diagnosis".to_string())
    } else if contains_any(&normalized, &["speed", "quickly", "under time"]) {
        Some("speed_build".to_string())
    } else if matches!(knowledge_role.as_deref(), Some("comparison")) {
        Some("classification".to_string())
    } else if contains_any(&normalized, &["apply", "context", "scenario"]) {
        Some("transfer_check".to_string())
    } else if contains_any(&normalized, &["exam", "paper", "past question"]) {
        Some("exam_pattern_familiarization".to_string())
    } else {
        Some("foundation_check".to_string())
    }
}

fn infer_content_grain(primary_skill_id: Option<i64>) -> Option<String> {
    Some(if primary_skill_id.is_some() {
        "skill".to_string()
    } else {
        "topic".to_string()
    })
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn normalize_axis_value(axis_code: &str, value: &str) -> String {
    let normalized = normalize_taxonomy_value(value);
    match axis_code {
        "knowledge_role" => match normalized.as_str() {
            "concept_check" | "concept" => "key_concept".to_string(),
            "formula" | "formula_use" => "formula_recall".to_string(),
            "worked_pattern" | "workedexample" | "worked_example_template" => {
                "worked_example".to_string()
            }
            "concept_explanation" => "explanation".to_string(),
            "procedure_check" => "procedure".to_string(),
            "application_scenario" => "application".to_string(),
            _ => normalized,
        },
        "cognitive_demand" => match normalized.as_str() {
            "understanding" => "comprehension".to_string(),
            "knowledge" => "recall".to_string(),
            "reasoning" | "problem_solving" => "analysis".to_string(),
            _ => normalized,
        },
        "solve_pattern" => match normalized.as_str() {
            "single_step_identification" => "direct_retrieval".to_string(),
            "worked_pattern" => "pattern_spotting".to_string(),
            "calculation" => "substitute_and_solve".to_string(),
            "short_reasoning" => "multi_step_reasoning".to_string(),
            _ => normalized,
        },
        "pedagogic_function" => match normalized.as_str() {
            "coverage_seed" | "concept_check" => "foundation_check".to_string(),
            "repair" => "misconception_diagnosis".to_string(),
            "exam_drill" => "exam_pattern_familiarization".to_string(),
            _ => normalized,
        },
        "content_grain" => match normalized.as_str() {
            "node" => "concept".to_string(),
            "subtopic" => "micro_concept".to_string(),
            _ => normalized,
        },
        "misconception_exposure" => {
            if normalized.contains("inverse") {
                "wrong_inverse_operation".to_string()
            } else if normalized.contains("formula") {
                "formula_selection_error".to_string()
            } else if normalized.contains("label")
                || normalized.contains("term")
                || normalized.contains("confusion")
            {
                "concept_label_confusion".to_string()
            } else if normalized.contains("process") && normalized.contains("definition") {
                "process_vs_definition_confusion".to_string()
            } else if normalized.contains("surface") || normalized.contains("pattern") {
                "surface_pattern_copy".to_string()
            } else {
                normalized
            }
        }
        _ => normalized,
    }
}

fn normalize_taxonomy_value(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn taxonomy_display_name(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_default_misconception(stem: &str, knowledge_role: Option<&str>) -> String {
    let normalized = normalize_text(stem);
    if contains_any(&normalized, &["formula", "equation"]) {
        "formula_selection_error".to_string()
    } else if contains_any(&normalized, &["define", "definition"]) {
        "process_vs_definition_confusion".to_string()
    } else if matches!(knowledge_role, Some("comparison")) {
        "concept_label_confusion".to_string()
    } else {
        "surface_pattern_copy".to_string()
    }
}

fn derive_family_type(
    knowledge_role: Option<&str>,
    pedagogic_function: Option<&str>,
) -> &'static str {
    if pedagogic_function == Some("misconception_diagnosis") {
        "misconception_cluster"
    } else if knowledge_role == Some("worked_example") {
        "worked_example_template"
    } else if pedagogic_function == Some("exam_pattern_familiarization") {
        "exam_structure"
    } else {
        "recurring_pattern"
    }
}

fn build_family_code(topic_code: &str, stem: &str, question_id: i64) -> String {
    let stem_slug = normalize_taxonomy_value(stem)
        .split('_')
        .take(4)
        .collect::<Vec<_>>()
        .join("_");
    format!(
        "{}_{}_{}",
        normalize_taxonomy_value(topic_code).to_ascii_uppercase(),
        stem_slug.to_ascii_uppercase(),
        question_id
    )
}

fn summarize_stem(stem: &str) -> String {
    let trimmed = stem.trim();
    let summary = trimmed
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    if summary.len() < trimmed.len() {
        format!("{}...", summary)
    } else {
        summary
    }
}

fn derive_classification_source(source_type: &str, source_ref: Option<&str>) -> &'static str {
    match source_type {
        "past_question" => "pack",
        "teacher_upload" => "hybrid",
        "generated" => {
            if source_ref.unwrap_or_default().contains("reactor-request") {
                "reactor"
            } else {
                "foundry"
            }
        }
        _ => "rules",
    }
}

fn review_status_for_action(action_code: &str) -> &'static str {
    match action_code {
        "approve" => "approved",
        "reject" => "rejected",
        "override" => "overridden",
        "mark_taxonomy_gap" => "taxonomy_gap",
        "mark_family_unresolved" => "family_unresolved",
        "send_for_reclassification" => "needs_review",
        _ => "pending",
    }
}

fn upsert_misconception_candidate(
    ranked: &mut std::collections::BTreeMap<String, QuestionMisconceptionTag>,
    code: String,
    confidence_score: BasisPoints,
    source: String,
) {
    match ranked.get(&code) {
        Some(existing) if existing.confidence_score >= confidence_score => {}
        _ => {
            ranked.insert(
                code.clone(),
                QuestionMisconceptionTag {
                    misconception_code: code,
                    confidence_score,
                    source,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        QuestionGenerationRequestInput, QuestionReactor, QuestionSlotSpec, QuestionVariantMode,
    };
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn question_profile_returns_normalized_intelligence_links() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");

        let profile = service
            .get_question_profile(question_id)
            .expect("question profile should load")
            .expect("question profile should exist");

        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "knowledge_role" && link.is_primary)
        );
        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "question_family")
        );
        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "misconception_exposure")
        );
    }

    #[test]
    fn can_find_questions_by_normalized_intelligence_axis() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");

        let questions = service
            .find_questions_by_intelligence(&QuestionIntelligenceQuery {
                axis_code: "knowledge_role".to_string(),
                concept_code: "worked_example".to_string(),
                subject_id: Some(subject_id),
                topic_id: None,
                limit: 10,
            })
            .expect("intelligence query should work");

        assert_eq!(questions.len(), 1);
        assert!(questions[0].stem.contains("simplest form"));
    }

    #[test]
    fn detects_near_duplicate_stems_within_family_scope() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let family_id: i64 = conn
            .query_row(
                "SELECT id FROM question_families WHERE family_code = 'FRA_EQUIV_01'",
                [],
                |row| row.get(0),
            )
            .expect("family should exist");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics WHERE code = 'FRA'", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");

        let duplicate = service
            .detect_near_duplicate(
                "Which fraction is equivalent to 1/2?",
                Some(family_id),
                Some(topic_id),
            )
            .expect("duplicate check should work");

        assert!(duplicate.matched_question_id.is_some());
        assert!(duplicate.is_exact_duplicate);
        assert!(duplicate.is_near_duplicate);
    }

    #[test]
    fn related_questions_surface_reactor_graph_edges() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

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
        let request = reactor
            .create_generation_request(&QuestionGenerationRequestInput {
                slot_spec: QuestionSlotSpec {
                    subject_id,
                    topic_id: Some(topic_id),
                    target_cognitive_demand: Some("recognition".to_string()),
                    target_question_format: Some("mcq".to_string()),
                    max_generated_share: 8_000,
                },
                family_id: None,
                source_question_id: None,
                request_kind: "variant".to_string(),
                variant_mode: QuestionVariantMode::RepresentationShift,
                requested_count: 1,
                rationale: Some("Need a fresh related variant".to_string()),
            })
            .expect("request should create");
        let generated = reactor
            .process_generation_request(request.id)
            .expect("request should process");

        let service = QuestionService::new(&conn);
        let related = service
            .list_related_questions(generated[0].source_question_id, None, 10)
            .expect("related questions should load");

        assert!(related.iter().any(|item| {
            item.question.id == generated[0].question.id
                && item.edge.relation_type == "representation_shift"
        }));
        assert!(
            related
                .iter()
                .any(|item| item.edge.relation_type == "same_family")
        );
    }

    #[test]
    fn question_intelligence_snapshot_is_materialized_for_pack_questions() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");

        let snapshot = service
            .get_question_intelligence(question_id)
            .expect("snapshot should load")
            .expect("snapshot should exist");

        assert!(snapshot.machine_confidence_score > 0);
        assert!(snapshot.family.is_some());
        assert_eq!(snapshot.review.review_status, "approved");
        assert!(
            snapshot
                .links
                .iter()
                .any(|link| link.axis_code == "question_family")
        );
        assert!(!snapshot.misconceptions.is_empty());
    }

    #[test]
    fn review_queue_round_trip_can_approve_generated_question() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let (subject_id, topic_id): (i64, i64) = conn
            .query_row(
                "SELECT s.id, t.id
                 FROM subjects s
                 INNER JOIN topics t ON t.subject_id = s.id
                 WHERE s.code = 'MATH' AND t.code = 'FRA'
                 LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("subject/topic should exist");

        conn.execute(
            "INSERT INTO questions (
                subject_id, topic_id, stem, question_format, difficulty_level,
                estimated_time_seconds, marks, source_type, classification_confidence,
                intelligence_snapshot
             ) VALUES (
                ?1, ?2, 'Choose the safest generated fraction shortcut.', 'mcq',
                4100, 30, 1, 'generated', 1800, '{}'
             )",
            params![subject_id, topic_id],
        )
        .expect("generated question should insert");
        let question_id = conn.last_insert_rowid();
        for (label, option_text, is_correct, position) in [
            ("A", "Match the denominators before comparing.", 1, 1),
            ("B", "Pick the answer that looks most familiar.", 0, 2),
        ] {
            conn.execute(
                "INSERT INTO question_options (
                    question_id, option_label, option_text, is_correct, position
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![question_id, label, option_text, is_correct, position],
            )
            .expect("option should insert");
        }

        let pending = service
            .classify_question(question_id, false)
            .expect("generated question should classify");
        assert_ne!(pending.review.review_status, "approved");

        let queue = service
            .list_question_review_queue(None, 20)
            .expect("review queue should load");
        assert!(queue.iter().any(|item| item.question_id == question_id));

        let reviewed = service
            .review_question_intelligence(
                question_id,
                &QuestionReviewActionInput {
                    reviewer_id: "qa-reviewer".to_string(),
                    action_code: "approve".to_string(),
                    review_status: None,
                    note: Some("Pattern checked and approved.".to_string()),
                    primary_knowledge_role: Some("application".to_string()),
                    primary_cognitive_demand: Some("application".to_string()),
                    primary_solve_pattern: Some("substitute_and_solve".to_string()),
                    primary_pedagogic_function: Some("foundation_check".to_string()),
                    primary_content_grain: Some("topic".to_string()),
                    family_id: pending.family.as_ref().and_then(|family| family.family_id),
                    misconception_codes: vec!["surface_pattern_copy".to_string()],
                    request_reclassification: false,
                },
            )
            .expect("manual review should succeed");
        assert_eq!(reviewed.review.review_status, "approved");

        let queue_after = service
            .list_question_review_queue(None, 20)
            .expect("review queue should reload");
        assert!(
            !queue_after
                .iter()
                .any(|item| item.question_id == question_id)
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
