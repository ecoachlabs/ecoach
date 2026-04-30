use ecoach_questions::{
    QuestionGenerationRequestInput, QuestionIntelligenceFilter, QuestionReactor,
    QuestionReviewActionInput, QuestionService, QuestionSlotSpec,
};
use ecoach_substrate::EcoachError;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::{
    dtos::{
        DuplicateCheckResultDto, GeneratedQuestionDraftDto, QuestionFamilyChoiceDto,
        QuestionFamilyHealthDto, QuestionGenerationRequestDto, QuestionIntelligenceSnapshotDto,
        QuestionLineageGraphDto, QuestionRemediationPlanDto, QuestionReviewQueueItemDto,
        RelatedQuestionDto,
    },
    error::CommandError,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionOptionInput {
    pub id: Option<i64>,
    pub option_label: String,
    pub option_text: String,
    pub is_correct: bool,
    pub misconception_id: Option<i64>,
    pub distractor_intent: Option<String>,
    pub position: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionOptionDto {
    pub id: i64,
    pub option_label: String,
    pub option_text: String,
    pub is_correct: bool,
    pub misconception_id: Option<i64>,
    pub distractor_intent: Option<String>,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionUpsertInput {
    pub question_id: Option<i64>,
    pub subject_id: i64,
    pub topic_id: i64,
    pub subtopic_id: Option<i64>,
    pub family_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub explanation_text: Option<String>,
    pub difficulty_level: i64,
    pub estimated_time_seconds: i64,
    pub marks: i64,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub exam_year: Option<i64>,
    pub primary_knowledge_role: Option<String>,
    pub primary_cognitive_demand: Option<String>,
    pub primary_solve_pattern: Option<String>,
    pub primary_pedagogic_function: Option<String>,
    pub primary_content_grain: Option<String>,
    pub cognitive_level: Option<String>,
    pub options: Vec<AdminQuestionOptionInput>,
}

impl AdminQuestionUpsertInput {
    pub fn existing(question_id: i64, subject_id: i64, topic_id: i64) -> Self {
        Self {
            question_id: Some(question_id),
            subject_id,
            topic_id,
            subtopic_id: None,
            family_id: None,
            stem: String::new(),
            question_format: "mcq".to_string(),
            explanation_text: None,
            difficulty_level: 5000,
            estimated_time_seconds: 30,
            marks: 1,
            source_type: "authored".to_string(),
            source_ref: None,
            exam_year: None,
            primary_knowledge_role: None,
            primary_cognitive_demand: None,
            primary_solve_pattern: None,
            primary_pedagogic_function: None,
            primary_content_grain: None,
            cognitive_level: None,
            options: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionUpsertResultDto {
    pub question_id: i64,
    pub stem: String,
    pub option_count: i64,
    pub review_status: String,
    pub machine_confidence_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionArchiveResultDto {
    pub question_id: i64,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionBulkActionInput {
    pub question_ids: Vec<i64>,
    pub action: String,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionBulkActionResultDto {
    pub requested_count: i64,
    pub updated_count: i64,
    pub active_count: i64,
    pub archived_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionEditorDto {
    pub question_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub subtopic_id: Option<i64>,
    pub family_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub explanation_text: Option<String>,
    pub difficulty_level: i64,
    pub estimated_time_seconds: i64,
    pub marks: i64,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub exam_year: Option<i64>,
    pub primary_knowledge_role: Option<String>,
    pub primary_cognitive_demand: Option<String>,
    pub primary_solve_pattern: Option<String>,
    pub primary_pedagogic_function: Option<String>,
    pub primary_content_grain: Option<String>,
    pub cognitive_level: Option<String>,
    pub review_status: String,
    pub machine_confidence_score: i64,
    pub is_active: bool,
    pub options: Vec<AdminQuestionOptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionListFilter {
    pub search: Option<String>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub review_status: Option<String>,
    pub source_type: Option<String>,
    #[serde(default)]
    pub active_status: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionListItemDto {
    pub question_id: i64,
    pub subject_id: i64,
    pub subject_name: String,
    pub topic_id: i64,
    pub topic_name: String,
    pub family_id: Option<i64>,
    pub family_name: Option<String>,
    pub stem: String,
    pub question_format: String,
    pub difficulty_level: i64,
    pub marks: i64,
    pub source_type: String,
    pub review_status: String,
    pub machine_confidence_score: i64,
    pub option_count: i64,
    pub attempt_count: i64,
    pub correct_count: i64,
    pub average_response_time_ms: Option<i64>,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionCountDto {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminQuestionBankStatsDto {
    pub total_questions: i64,
    pub active_questions: i64,
    pub total_options: i64,
    pub total_attempts: i64,
    pub family_count: i64,
    pub installed_pack_count: i64,
    pub source_upload_count: i64,
    pub pending_review_count: i64,
    pub approved_review_count: i64,
    pub by_format: Vec<AdminQuestionCountDto>,
    pub by_source_type: Vec<AdminQuestionCountDto>,
    pub by_review_status: Vec<AdminQuestionCountDto>,
}

pub fn choose_reactor_family(
    state: &AppState,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionFamilyChoiceDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .get_best_family_for_slot(&slot_spec)?
            .map(QuestionFamilyChoiceDto::from))
    })
}

pub fn create_question_generation_request(
    state: &AppState,
    input: QuestionGenerationRequestInput,
) -> Result<QuestionGenerationRequestDto, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        let request = reactor.create_generation_request(&input)?;
        Ok(QuestionGenerationRequestDto::from(request))
    })
}

pub fn process_question_generation_request(
    state: &AppState,
    request_id: i64,
) -> Result<Vec<GeneratedQuestionDraftDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .process_generation_request(request_id)?
            .into_iter()
            .map(GeneratedQuestionDraftDto::from)
            .collect())
    })
}

pub fn get_question_lineage(
    state: &AppState,
    question_id: i64,
) -> Result<QuestionLineageGraphDto, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(QuestionLineageGraphDto::from(
            reactor.get_question_lineage(question_id)?,
        ))
    })
}

pub fn get_question_family_health(
    state: &AppState,
    family_id: i64,
) -> Result<Option<QuestionFamilyHealthDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .get_family_health(family_id)?
            .map(QuestionFamilyHealthDto::from))
    })
}

pub fn list_related_questions(
    state: &AppState,
    question_id: i64,
    relation_type: Option<String>,
    limit: usize,
) -> Result<Vec<RelatedQuestionDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .list_related_questions(question_id, relation_type.as_deref(), limit)?
            .into_iter()
            .map(RelatedQuestionDto::from)
            .collect())
    })
}

pub fn detect_near_duplicate(
    state: &AppState,
    stem: String,
    family_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<DuplicateCheckResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(DuplicateCheckResultDto::from(
            service.detect_near_duplicate(&stem, family_id, topic_id)?,
        ))
    })
}

pub fn recommend_question_remediation_plan(
    state: &AppState,
    student_id: i64,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionRemediationPlanDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .recommend_remediation_plan(student_id, &slot_spec)?
            .map(QuestionRemediationPlanDto::from))
    })
}

pub fn get_question_intelligence(
    state: &AppState,
    question_id: i64,
) -> Result<Option<QuestionIntelligenceSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .get_question_intelligence(question_id)?
            .map(QuestionIntelligenceSnapshotDto::from))
    })
}

pub fn classify_question_intelligence(
    state: &AppState,
    question_id: i64,
    reclassify: bool,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(QuestionIntelligenceSnapshotDto::from(
            service.classify_question(question_id, reclassify)?,
        ))
    })
}

pub fn find_questions_by_intelligence_filter(
    state: &AppState,
    filter: QuestionIntelligenceFilter,
) -> Result<Vec<QuestionIntelligenceSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .find_questions_by_intelligence_filter(&filter)?
            .into_iter()
            .map(QuestionIntelligenceSnapshotDto::from)
            .collect())
    })
}

pub fn list_question_review_queue(
    state: &AppState,
    review_status: Option<String>,
    limit: usize,
) -> Result<Vec<QuestionReviewQueueItemDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .list_question_review_queue(review_status.as_deref(), limit)?
            .into_iter()
            .map(QuestionReviewQueueItemDto::from)
            .collect())
    })
}

pub fn review_question_intelligence(
    state: &AppState,
    question_id: i64,
    input: QuestionReviewActionInput,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(QuestionIntelligenceSnapshotDto::from(
            service.review_question_intelligence(question_id, &input)?,
        ))
    })
}

pub fn queue_question_reclassification(
    state: &AppState,
    question_id: i64,
    trigger_reason: String,
    requested_by: Option<String>,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        service
            .queue_question_reclassification(question_id, &trigger_reason, requested_by.as_deref())
            .map_err(CommandError::from)
    })
}

pub fn get_admin_question_bank_stats(
    state: &AppState,
) -> Result<AdminQuestionBankStatsDto, CommandError> {
    state.with_connection(|conn| {
        let total_questions = query_i64(conn, "SELECT COUNT(*) FROM questions", [])?;
        let active_questions = query_i64(
            conn,
            "SELECT COUNT(*) FROM questions WHERE is_active = 1",
            [],
        )?;
        let total_options = query_i64(conn, "SELECT COUNT(*) FROM question_options", [])?;
        let total_attempts = query_i64(conn, "SELECT COUNT(*) FROM student_question_attempts", [])?;
        let family_count = query_i64(conn, "SELECT COUNT(*) FROM question_families", [])?;
        let installed_pack_count = query_i64(
            conn,
            "SELECT COUNT(*) FROM content_packs WHERE status IN ('installed', 'active')",
            [],
        )?;
        let source_upload_count =
            query_i64(conn, "SELECT COUNT(*) FROM curriculum_source_uploads", [])?;
        let pending_review_count = query_i64(
            conn,
            "SELECT COUNT(*)
             FROM question_intelligence_profiles
             WHERE review_status <> 'approved'",
            [],
        )?;
        let approved_review_count = query_i64(
            conn,
            "SELECT COUNT(*)
             FROM question_intelligence_profiles
             WHERE review_status = 'approved'",
            [],
        )?;

        Ok(AdminQuestionBankStatsDto {
            total_questions,
            active_questions,
            total_options,
            total_attempts,
            family_count,
            installed_pack_count,
            source_upload_count,
            pending_review_count,
            approved_review_count,
            by_format: query_breakdown(
                conn,
                "SELECT question_format, COUNT(*)
                 FROM questions
                 WHERE is_active = 1
                 GROUP BY question_format
                 ORDER BY COUNT(*) DESC, question_format ASC",
            )?,
            by_source_type: query_breakdown(
                conn,
                "SELECT COALESCE(source_type, 'unknown'), COUNT(*)
                 FROM questions
                 WHERE is_active = 1
                 GROUP BY COALESCE(source_type, 'unknown')
                 ORDER BY COUNT(*) DESC, source_type ASC",
            )?,
            by_review_status: query_breakdown(
                conn,
                "SELECT COALESCE(qip.review_status, 'unclassified'), COUNT(*)
                 FROM questions q
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                 WHERE q.is_active = 1
                 GROUP BY COALESCE(qip.review_status, 'unclassified')
                 ORDER BY COUNT(*) DESC, COALESCE(qip.review_status, 'unclassified') ASC",
            )?,
        })
    })
}

pub fn list_admin_questions(
    state: &AppState,
    filter: AdminQuestionListFilter,
) -> Result<Vec<AdminQuestionListItemDto>, CommandError> {
    state.with_connection(|conn| {
        let search = filter.search.as_ref().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("%{}%", trimmed))
            }
        });
        let active_status = match filter.active_status.as_deref() {
            Some("archived") => "archived",
            Some("all") => "all",
            _ => "active",
        };

        let mut statement = conn
            .prepare(
                "SELECT q.id, q.subject_id, s.name, q.topic_id, t.name, q.family_id,
                        qf.family_name, q.stem, q.question_format, q.difficulty_level,
                        q.marks, COALESCE(q.source_type, 'authored'),
                        COALESCE(qip.review_status, 'unclassified'),
                        COALESCE(qip.machine_confidence_bp, q.classification_confidence, 0),
                        (SELECT COUNT(*) FROM question_options qo WHERE qo.question_id = q.id),
                        (SELECT COUNT(*) FROM student_question_attempts a WHERE a.question_id = q.id),
                        (SELECT COUNT(*) FROM student_question_attempts a WHERE a.question_id = q.id AND a.is_correct = 1),
                        (SELECT CAST(ROUND(AVG(a.response_time_ms)) AS INTEGER)
                         FROM student_question_attempts a
                         WHERE a.question_id = q.id AND a.response_time_ms IS NOT NULL),
                        q.is_active, q.updated_at
                 FROM questions q
                 INNER JOIN subjects s ON s.id = q.subject_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 LEFT JOIN question_families qf ON qf.id = q.family_id
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                  WHERE (?1 IS NULL OR q.subject_id = ?1)
                    AND (?2 IS NULL OR q.topic_id = ?2)
                    AND (?3 IS NULL OR COALESCE(qip.review_status, 'unclassified') = ?3)
                    AND (?4 IS NULL OR COALESCE(q.source_type, 'authored') = ?4)
                    AND (
                         ?5 = 'all'
                         OR (?5 = 'archived' AND q.is_active = 0)
                         OR (?5 = 'active' AND q.is_active = 1)
                    )
                    AND (
                         ?6 IS NULL
                         OR q.stem LIKE ?6
                         OR t.name LIKE ?6
                         OR s.name LIKE ?6
                         OR qf.family_name LIKE ?6
                    )
                  ORDER BY q.updated_at DESC, q.id DESC
                  LIMIT ?7",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![
                    filter.subject_id,
                    filter.topic_id,
                    filter.review_status,
                    filter.source_type,
                    active_status,
                    search,
                    filter.limit.max(1) as i64,
                ],
                |row| {
                    Ok(AdminQuestionListItemDto {
                        question_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        subject_name: row.get(2)?,
                        topic_id: row.get(3)?,
                        topic_name: row.get(4)?,
                        family_id: row.get(5)?,
                        family_name: row.get(6)?,
                        stem: row.get(7)?,
                        question_format: row.get(8)?,
                        difficulty_level: row.get(9)?,
                        marks: row.get(10)?,
                        source_type: row.get(11)?,
                        review_status: row.get(12)?,
                        machine_confidence_score: row.get::<_, i64>(13)?.clamp(0, 10_000),
                        option_count: row.get(14)?,
                        attempt_count: row.get(15)?,
                        correct_count: row.get(16)?,
                        average_response_time_ms: row.get(17)?,
                        is_active: row.get::<_, i64>(18)? == 1,
                        updated_at: row.get(19)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    })
}

pub fn get_admin_question_editor(
    state: &AppState,
    question_id: i64,
) -> Result<AdminQuestionEditorDto, CommandError> {
    read_admin_question_editor(state, question_id, false)
}

pub fn get_admin_question_editor_any(
    state: &AppState,
    question_id: i64,
) -> Result<AdminQuestionEditorDto, CommandError> {
    read_admin_question_editor(state, question_id, true)
}

fn read_admin_question_editor(
    state: &AppState,
    question_id: i64,
    include_archived: bool,
) -> Result<AdminQuestionEditorDto, CommandError> {
    state.with_connection(|conn| {
        let mut editor = conn
            .query_row(
                "SELECT q.id, q.subject_id, q.topic_id, q.subtopic_id, q.family_id,
                        q.stem, q.question_format, q.explanation_text, q.difficulty_level,
                        q.estimated_time_seconds, q.marks, COALESCE(q.source_type, 'authored'),
                        q.source_ref, q.exam_year,
                        COALESCE(qip.primary_knowledge_role, q.primary_knowledge_role),
                        COALESCE(qip.primary_cognitive_demand, q.primary_cognitive_demand),
                        COALESCE(qip.primary_solve_pattern, q.primary_solve_pattern),
                        COALESCE(qip.primary_pedagogic_function, q.primary_pedagogic_function),
                        qip.primary_content_grain,
                        q.cognitive_level,
                        COALESCE(qip.review_status, 'unclassified'),
                        COALESCE(qip.machine_confidence_bp, q.classification_confidence, 0),
                        q.is_active
                 FROM questions q
                 LEFT JOIN question_intelligence_profiles qip ON qip.question_id = q.id
                 WHERE q.id = ?1 AND (?2 = 1 OR q.is_active = 1)",
                params![question_id, if include_archived { 1_i64 } else { 0_i64 }],
                |row| {
                    Ok(AdminQuestionEditorDto {
                        question_id: row.get(0)?,
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
                        source_ref: row.get(12)?,
                        exam_year: row.get(13)?,
                        primary_knowledge_role: row.get(14)?,
                        primary_cognitive_demand: row.get(15)?,
                        primary_solve_pattern: row.get(16)?,
                        primary_pedagogic_function: row.get(17)?,
                        primary_content_grain: row.get(18)?,
                        cognitive_level: row.get(19)?,
                        review_status: row.get(20)?,
                        machine_confidence_score: row.get::<_, i64>(21)?.clamp(0, 10_000),
                        is_active: row.get::<_, i64>(22)? == 1,
                        options: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("question {} was not found", question_id))
            })?;

        editor.options = read_admin_question_options(conn, question_id)?;
        Ok(editor)
    })
}

pub fn list_admin_question_options(
    state: &AppState,
    question_id: i64,
) -> Result<Vec<AdminQuestionOptionDto>, CommandError> {
    state.with_connection(|conn| {
        let exists = conn
            .query_row(
                "SELECT 1 FROM questions WHERE id = ?1 AND is_active = 1",
                [question_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists.is_none() {
            return Err(
                EcoachError::NotFound(format!("question {} was not found", question_id)).into(),
            );
        }

        read_admin_question_options(conn, question_id).map_err(CommandError::from)
    })
}

pub fn upsert_admin_question(
    state: &AppState,
    input: AdminQuestionUpsertInput,
) -> Result<AdminQuestionUpsertResultDto, CommandError> {
    validate_admin_question_input(&input)?;

    state.with_connection(|conn| {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let result = (|| -> Result<i64, EcoachError> {
            let question_id = if let Some(question_id) = input.question_id {
                let changed = conn
                    .execute(
                        "UPDATE questions
                         SET subject_id = ?1,
                             topic_id = ?2,
                             subtopic_id = ?3,
                             family_id = ?4,
                             stem = ?5,
                             question_format = ?6,
                             explanation_text = ?7,
                             difficulty_level = ?8,
                             estimated_time_seconds = ?9,
                             marks = ?10,
                             source_type = ?11,
                             source_ref = ?12,
                             exam_year = ?13,
                             primary_knowledge_role = ?14,
                             primary_cognitive_demand = ?15,
                             primary_solve_pattern = ?16,
                             primary_pedagogic_function = ?17,
                             cognitive_level = ?18,
                             classification_confidence = ?19,
                             classification_confidence_bp = ?19,
                             updated_at = datetime('now')
                         WHERE id = ?20 AND is_active = 1",
                        params![
                            input.subject_id,
                            input.topic_id,
                            input.subtopic_id,
                            input.family_id,
                            input.stem.trim(),
                            input.question_format,
                            input.explanation_text,
                            input.difficulty_level,
                            input.estimated_time_seconds,
                            input.marks,
                            input.source_type,
                            input.source_ref,
                            input.exam_year,
                            input.primary_knowledge_role,
                            input.primary_cognitive_demand,
                            input.primary_solve_pattern,
                            input.primary_pedagogic_function,
                            input.cognitive_level,
                            0_i64,
                            question_id,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if changed == 0 {
                    return Err(EcoachError::NotFound(format!(
                        "question {} was not found",
                        question_id
                    )));
                }
                question_id
            } else {
                conn.execute(
                    "INSERT INTO questions (
                        subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks,
                        source_type, source_ref, exam_year, primary_knowledge_role,
                        primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function, cognitive_level,
                        classification_confidence, classification_confidence_bp,
                        intelligence_snapshot, is_active, created_at, updated_at
                     ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6,
                        ?7, ?8, ?9, ?10,
                        ?11, ?12, ?13, ?14,
                        ?15, ?16,
                        ?17, ?18,
                        0, 0, '{}', 1, datetime('now'), datetime('now')
                     )",
                    params![
                        input.subject_id,
                        input.topic_id,
                        input.subtopic_id,
                        input.family_id,
                        input.stem.trim(),
                        input.question_format,
                        input.explanation_text,
                        input.difficulty_level,
                        input.estimated_time_seconds,
                        input.marks,
                        input.source_type,
                        input.source_ref,
                        input.exam_year,
                        input.primary_knowledge_role,
                        input.primary_cognitive_demand,
                        input.primary_solve_pattern,
                        input.primary_pedagogic_function,
                        input.cognitive_level,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
                conn.last_insert_rowid()
            };

            conn.execute(
                "DELETE FROM question_options WHERE question_id = ?1",
                [question_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

            for (index, option) in input.options.iter().enumerate() {
                conn.execute(
                    "INSERT INTO question_options (
                        question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        question_id,
                        option.option_label.trim(),
                        option.option_text.trim(),
                        if option.is_correct { 1 } else { 0 },
                        option.misconception_id,
                        option.distractor_intent,
                        option.position.unwrap_or((index + 1) as i64),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }

            Ok(question_id)
        })();

        match result {
            Ok(question_id) => {
                conn.execute_batch("COMMIT")
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;

                let service = QuestionService::new(conn);
                let mut snapshot = service.classify_question(question_id, true)?;
                if let Some(content_grain) = input.primary_content_grain.as_deref() {
                    conn.execute(
                        "UPDATE question_intelligence_profiles
                         SET primary_content_grain = ?2, updated_at = datetime('now')
                         WHERE question_id = ?1",
                        params![question_id, content_grain],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    if let Some(updated_snapshot) =
                        service.get_question_intelligence(question_id)?
                    {
                        snapshot = updated_snapshot;
                    }
                }
                let option_count = conn
                    .query_row(
                        "SELECT COUNT(*) FROM question_options WHERE question_id = ?1",
                        [question_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;

                Ok(AdminQuestionUpsertResultDto {
                    question_id,
                    stem: snapshot.question.stem,
                    option_count,
                    review_status: snapshot.review.review_status,
                    machine_confidence_score: snapshot.machine_confidence_score as i64,
                })
            }
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(err.into())
            }
        }
    })
}

pub fn archive_admin_question(
    state: &AppState,
    question_id: i64,
) -> Result<AdminQuestionArchiveResultDto, CommandError> {
    set_admin_question_active(state, question_id, false)
}

pub fn restore_admin_question(
    state: &AppState,
    question_id: i64,
) -> Result<AdminQuestionArchiveResultDto, CommandError> {
    set_admin_question_active(state, question_id, true)
}

pub fn bulk_update_admin_questions(
    state: &AppState,
    input: AdminQuestionBulkActionInput,
) -> Result<AdminQuestionBulkActionResultDto, CommandError> {
    let action = input.action.trim().to_ascii_lowercase();
    if !matches!(action.as_str(), "archive" | "restore" | "move") {
        return Err(EcoachError::Validation(format!(
            "unsupported bulk question action: {}",
            input.action
        ))
        .into());
    }

    let mut question_ids: Vec<i64> = input
        .question_ids
        .into_iter()
        .filter(|question_id| *question_id > 0)
        .collect();
    question_ids.sort_unstable();
    question_ids.dedup();

    if question_ids.is_empty() {
        return Err(
            EcoachError::Validation("at least one question is required".to_string()).into(),
        );
    }

    if action == "move" && input.topic_id.is_none() {
        return Err(
            EcoachError::Validation("topic_id is required for bulk move".to_string()).into(),
        );
    }

    state.with_connection(|conn| {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let result = (|| -> Result<AdminQuestionBulkActionResultDto, EcoachError> {
            let move_target = if action == "move" {
                let topic_id = input.topic_id.expect("validated topic_id");
                let topic_subject_id = conn
                    .query_row(
                        "SELECT subject_id FROM topics WHERE id = ?1",
                        [topic_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .ok_or_else(|| {
                        EcoachError::NotFound(format!("topic {} was not found", topic_id))
                    })?;

                if let Some(subject_id) = input.subject_id {
                    if subject_id != topic_subject_id {
                        return Err(EcoachError::Validation(format!(
                            "topic {} does not belong to subject {}",
                            topic_id, subject_id
                        )));
                    }
                }

                Some((topic_subject_id, topic_id))
            } else {
                None
            };

            let mut updated_count = 0_i64;
            for question_id in &question_ids {
                let changed = match (action.as_str(), move_target) {
                    ("archive", _) => conn.execute(
                        "UPDATE questions
                         SET is_active = 0, updated_at = datetime('now')
                         WHERE id = ?1",
                        [question_id],
                    ),
                    ("restore", _) => conn.execute(
                        "UPDATE questions
                         SET is_active = 1, updated_at = datetime('now')
                         WHERE id = ?1",
                        [question_id],
                    ),
                    ("move", Some((subject_id, topic_id))) => conn.execute(
                        "UPDATE questions
                         SET subject_id = ?2,
                             topic_id = ?3,
                             subtopic_id = NULL,
                             updated_at = datetime('now')
                         WHERE id = ?1",
                        params![question_id, subject_id, topic_id],
                    ),
                    _ => unreachable!("bulk question action was validated"),
                }
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

                if changed == 0 {
                    return Err(EcoachError::NotFound(format!(
                        "question {} was not found",
                        question_id
                    )));
                }
                updated_count += changed as i64;
            }

            let mut active_count = 0_i64;
            let mut archived_count = 0_i64;
            for question_id in &question_ids {
                let is_active = conn
                    .query_row(
                        "SELECT is_active FROM questions WHERE id = ?1",
                        [question_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .ok_or_else(|| {
                        EcoachError::NotFound(format!("question {} was not found", question_id))
                    })?;
                if is_active == 1 {
                    active_count += 1;
                } else {
                    archived_count += 1;
                }
            }

            Ok(AdminQuestionBulkActionResultDto {
                requested_count: question_ids.len() as i64,
                updated_count,
                active_count,
                archived_count,
            })
        })();

        match result {
            Ok(result) => {
                conn.execute_batch("COMMIT")
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                Ok(result)
            }
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(err.into())
            }
        }
    })
}

fn set_admin_question_active(
    state: &AppState,
    question_id: i64,
    is_active: bool,
) -> Result<AdminQuestionArchiveResultDto, CommandError> {
    state.with_connection(|conn| {
        let changed = conn
            .execute(
                "UPDATE questions
                 SET is_active = ?2, updated_at = datetime('now')
                 WHERE id = ?1",
                params![question_id, if is_active { 1_i64 } else { 0_i64 }],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if changed == 0 {
            return Err(
                EcoachError::NotFound(format!("question {} was not found", question_id)).into(),
            );
        }

        conn.query_row(
            "SELECT id, is_active, updated_at FROM questions WHERE id = ?1",
            [question_id],
            |row| {
                Ok(AdminQuestionArchiveResultDto {
                    question_id: row.get(0)?,
                    is_active: row.get::<_, i64>(1)? == 1,
                    updated_at: row.get(2)?,
                })
            },
        )
        .map_err(|err| EcoachError::Storage(err.to_string()).into())
    })
}

fn validate_admin_question_input(input: &AdminQuestionUpsertInput) -> Result<(), CommandError> {
    if input.stem.trim().is_empty() {
        return Err(EcoachError::Validation("question stem is required".to_string()).into());
    }

    if !matches!(
        input.question_format.as_str(),
        "mcq" | "short_answer" | "numeric" | "true_false" | "matching" | "ordering"
    ) {
        return Err(EcoachError::Validation(format!(
            "unsupported question format: {}",
            input.question_format
        ))
        .into());
    }

    if !matches!(
        input.source_type.as_str(),
        "past_question" | "authored" | "generated" | "teacher_upload"
    ) {
        return Err(EcoachError::Validation(format!(
            "unsupported source type: {}",
            input.source_type
        ))
        .into());
    }

    if input.question_format == "mcq" {
        if input.options.len() < 2 {
            return Err(EcoachError::Validation(
                "mcq questions need at least two options".to_string(),
            )
            .into());
        }

        let correct_count = input
            .options
            .iter()
            .filter(|option| option.is_correct)
            .count();
        if correct_count != 1 {
            return Err(EcoachError::Validation(
                "mcq questions need exactly one correct option".to_string(),
            )
            .into());
        }
    }

    for option in &input.options {
        if option.option_label.trim().is_empty() || option.option_text.trim().is_empty() {
            return Err(
                EcoachError::Validation("answer options need labels and text".to_string()).into(),
            );
        }
    }

    Ok(())
}

fn query_i64<P: rusqlite::Params>(
    conn: &rusqlite::Connection,
    sql: &str,
    params: P,
) -> Result<i64, EcoachError> {
    conn.query_row(sql, params, |row| row.get::<_, i64>(0))
        .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn query_breakdown(
    conn: &rusqlite::Connection,
    sql: &str,
) -> Result<Vec<AdminQuestionCountDto>, EcoachError> {
    let mut statement = conn
        .prepare(sql)
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok(AdminQuestionCountDto {
                label: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(items)
}

fn read_admin_question_options(
    conn: &rusqlite::Connection,
    question_id: i64,
) -> Result<Vec<AdminQuestionOptionDto>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT id, option_label, option_text, is_correct, misconception_id,
                    distractor_intent, position
             FROM question_options
             WHERE question_id = ?1
             ORDER BY position ASC, id ASC",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map([question_id], |row| {
            Ok(AdminQuestionOptionDto {
                id: row.get(0)?,
                option_label: row.get(1)?,
                option_text: row.get(2)?,
                is_correct: row.get::<_, i64>(3)? == 1,
                misconception_id: row.get(4)?,
                distractor_intent: row.get(5)?,
                position: row.get(6)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut options = Vec::new();
    for row in rows {
        options.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(options)
}

#[cfg(test)]
mod admin_tests {
    use super::*;

    fn seed_minimal_curriculum(state: &AppState) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO curriculum_versions (
                        id, name, country, version_label, status
                     ) VALUES (1, 'Admin Test Curriculum', 'GH', 'v1', 'published')",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO subjects (
                        id, curriculum_version_id, code, name
                     ) VALUES (1, 1, 'MTH', 'Mathematics')",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO topics (
                        id, subject_id, code, name, node_type
                     ) VALUES (2, 1, 'ALG', 'Algebra', 'topic')",
                    [],
                )
                .unwrap();
                Ok::<_, CommandError>(())
            })
            .unwrap();
    }

    fn admin_mcq_input(stem: &str, topic_id: i64) -> AdminQuestionUpsertInput {
        AdminQuestionUpsertInput {
            question_id: None,
            subject_id: 1,
            topic_id,
            subtopic_id: None,
            family_id: None,
            stem: stem.to_string(),
            question_format: "mcq".to_string(),
            explanation_text: None,
            difficulty_level: 4000,
            estimated_time_seconds: 45,
            marks: 1,
            source_type: "authored".to_string(),
            source_ref: None,
            exam_year: None,
            primary_knowledge_role: None,
            primary_cognitive_demand: None,
            primary_solve_pattern: None,
            primary_pedagogic_function: None,
            primary_content_grain: None,
            cognitive_level: None,
            options: vec![
                AdminQuestionOptionInput {
                    id: None,
                    option_label: "A".to_string(),
                    option_text: "Correct".to_string(),
                    is_correct: true,
                    misconception_id: None,
                    distractor_intent: None,
                    position: Some(1),
                },
                AdminQuestionOptionInput {
                    id: None,
                    option_label: "B".to_string(),
                    option_text: "Distractor".to_string(),
                    is_correct: false,
                    misconception_id: None,
                    distractor_intent: None,
                    position: Some(2),
                },
            ],
        }
    }

    #[test]
    fn admin_question_upsert_creates_and_updates_options() {
        let state = AppState::in_memory().expect("in-memory state should build");
        seed_minimal_curriculum(&state);

        let created = upsert_admin_question(
            &state,
            AdminQuestionUpsertInput {
                question_id: None,
                subject_id: 1,
                topic_id: 2,
                subtopic_id: None,
                family_id: None,
                stem: "What is 2 + 3?".to_string(),
                question_format: "mcq".to_string(),
                explanation_text: Some("Add the two numbers.".to_string()),
                difficulty_level: 2500,
                estimated_time_seconds: 30,
                marks: 1,
                source_type: "authored".to_string(),
                source_ref: Some("admin-test".to_string()),
                exam_year: None,
                primary_knowledge_role: Some("procedure".to_string()),
                primary_cognitive_demand: Some("application".to_string()),
                primary_solve_pattern: Some("direct_retrieval".to_string()),
                primary_pedagogic_function: Some("foundation_check".to_string()),
                primary_content_grain: Some("topic".to_string()),
                cognitive_level: Some("application".to_string()),
                options: vec![
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "A".to_string(),
                        option_text: "5".to_string(),
                        is_correct: true,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(1),
                    },
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "B".to_string(),
                        option_text: "6".to_string(),
                        is_correct: false,
                        misconception_id: None,
                        distractor_intent: Some("one_more_than_sum".to_string()),
                        position: Some(2),
                    },
                ],
            },
        )
        .expect("admin question should create");

        assert!(created.question_id > 0);
        assert_eq!(created.option_count, 2);

        let editor = get_admin_question_editor(&state, created.question_id)
            .expect("admin edit snapshot should load");
        assert_eq!(editor.question_id, created.question_id);
        assert_eq!(
            editor.explanation_text.as_deref(),
            Some("Add the two numbers.")
        );
        assert_eq!(editor.estimated_time_seconds, 30);
        assert_eq!(editor.source_ref.as_deref(), Some("admin-test"));
        assert_eq!(editor.primary_knowledge_role.as_deref(), Some("procedure"));
        assert_eq!(editor.options.len(), 2);
        assert!(editor.options[0].is_correct);

        let updated = upsert_admin_question(
            &state,
            AdminQuestionUpsertInput {
                question_id: Some(created.question_id),
                stem: "What is 4 + 3?".to_string(),
                options: vec![
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "A".to_string(),
                        option_text: "7".to_string(),
                        is_correct: true,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(1),
                    },
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "B".to_string(),
                        option_text: "8".to_string(),
                        is_correct: false,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(2),
                    },
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "C".to_string(),
                        option_text: "1".to_string(),
                        is_correct: false,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(3),
                    },
                ],
                ..AdminQuestionUpsertInput::existing(created.question_id, 1, 2)
            },
        )
        .expect("admin question should update");

        assert_eq!(updated.question_id, created.question_id);
        assert_eq!(updated.option_count, 3);
        assert_eq!(updated.stem, "What is 4 + 3?");

        let options = list_admin_question_options(&state, created.question_id)
            .expect("admin options should load");
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].option_text, "7");
        assert!(options[0].is_correct);
        assert!(!options[1].is_correct);
    }

    #[test]
    fn admin_question_bank_stats_and_list_include_learning_data() {
        let state = AppState::in_memory().expect("in-memory state should build");
        seed_minimal_curriculum(&state);

        let question = upsert_admin_question(
            &state,
            AdminQuestionUpsertInput {
                question_id: None,
                subject_id: 1,
                topic_id: 2,
                subtopic_id: None,
                family_id: None,
                stem: "Which value equals 10?".to_string(),
                question_format: "mcq".to_string(),
                explanation_text: None,
                difficulty_level: 4000,
                estimated_time_seconds: 45,
                marks: 1,
                source_type: "authored".to_string(),
                source_ref: None,
                exam_year: None,
                primary_knowledge_role: Some("definition".to_string()),
                primary_cognitive_demand: Some("recognition".to_string()),
                primary_solve_pattern: Some("direct_retrieval".to_string()),
                primary_pedagogic_function: Some("foundation_check".to_string()),
                primary_content_grain: Some("topic".to_string()),
                cognitive_level: None,
                options: vec![
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "A".to_string(),
                        option_text: "10".to_string(),
                        is_correct: true,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(1),
                    },
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "B".to_string(),
                        option_text: "12".to_string(),
                        is_correct: false,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(2),
                    },
                ],
            },
        )
        .expect("question should create");

        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO accounts (
                        id, account_type, display_name, pin_hash, pin_salt, entitlement_tier
                     ) VALUES (99, 'student', 'Learner', 'hash', 'salt', 'standard')",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO student_question_attempts (
                        student_id, question_id, started_at, submitted_at,
                        response_time_ms, is_correct, confidence_level
                     ) VALUES
                        (99, ?1, datetime('now'), datetime('now'), 1100, 1, 'sure'),
                        (99, ?1, datetime('now'), datetime('now'), 2100, 0, 'guessed')",
                    [question.question_id],
                )
                .unwrap();
                Ok::<_, CommandError>(())
            })
            .unwrap();

        let stats = get_admin_question_bank_stats(&state).expect("stats should load");
        assert_eq!(stats.total_questions, 1);
        assert_eq!(stats.active_questions, 1);
        assert_eq!(stats.total_options, 2);
        assert_eq!(stats.total_attempts, 2);

        let list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("value".to_string()),
                subject_id: Some(1),
                topic_id: None,
                review_status: None,
                source_type: None,
                active_status: None,
                limit: 20,
            },
        )
        .expect("question list should load");

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].question_id, question.question_id);
        assert_eq!(list[0].attempt_count, 2);
        assert_eq!(list[0].correct_count, 1);
        assert_eq!(list[0].average_response_time_ms, Some(1600));
    }

    #[test]
    fn admin_question_archive_removes_from_active_bank_and_restore_returns_it() {
        let state = AppState::in_memory().expect("in-memory state should build");
        seed_minimal_curriculum(&state);

        let question = upsert_admin_question(
            &state,
            AdminQuestionUpsertInput {
                question_id: None,
                subject_id: 1,
                topic_id: 2,
                subtopic_id: None,
                family_id: None,
                stem: "Archive me from the active bank.".to_string(),
                question_format: "mcq".to_string(),
                explanation_text: None,
                difficulty_level: 4000,
                estimated_time_seconds: 45,
                marks: 1,
                source_type: "authored".to_string(),
                source_ref: None,
                exam_year: None,
                primary_knowledge_role: None,
                primary_cognitive_demand: None,
                primary_solve_pattern: None,
                primary_pedagogic_function: None,
                primary_content_grain: None,
                cognitive_level: None,
                options: vec![
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "A".to_string(),
                        option_text: "Keep".to_string(),
                        is_correct: true,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(1),
                    },
                    AdminQuestionOptionInput {
                        id: None,
                        option_label: "B".to_string(),
                        option_text: "Drop".to_string(),
                        is_correct: false,
                        misconception_id: None,
                        distractor_intent: None,
                        position: Some(2),
                    },
                ],
            },
        )
        .expect("question should create");

        let archived =
            archive_admin_question(&state, question.question_id).expect("question should archive");
        assert_eq!(archived.question_id, question.question_id);
        assert!(!archived.is_active);

        let stats = get_admin_question_bank_stats(&state).expect("stats should load");
        assert_eq!(stats.total_questions, 1);
        assert_eq!(stats.active_questions, 0);

        let list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("Archive me".to_string()),
                subject_id: None,
                topic_id: None,
                review_status: None,
                source_type: None,
                active_status: None,
                limit: 20,
            },
        )
        .expect("question list should load");
        assert!(list.is_empty());
        assert!(get_admin_question_editor(&state, question.question_id).is_err());

        let archived_list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("Archive me".to_string()),
                subject_id: None,
                topic_id: None,
                review_status: None,
                source_type: None,
                active_status: Some("archived".to_string()),
                limit: 20,
            },
        )
        .expect("archived question list should load");
        assert_eq!(archived_list.len(), 1);
        assert_eq!(archived_list[0].question_id, question.question_id);
        assert!(!archived_list[0].is_active);

        let restored =
            restore_admin_question(&state, question.question_id).expect("question should restore");
        assert_eq!(restored.question_id, question.question_id);
        assert!(restored.is_active);

        let restored_list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("Archive me".to_string()),
                subject_id: None,
                topic_id: None,
                review_status: None,
                source_type: None,
                active_status: None,
                limit: 20,
            },
        )
        .expect("question list should load after restore");
        assert_eq!(restored_list.len(), 1);
        assert_eq!(restored_list[0].question_id, question.question_id);
    }

    #[test]
    fn admin_question_editor_any_loads_archived_questions_for_cms_restore() {
        let state = AppState::in_memory().expect("in-memory state should build");
        seed_minimal_curriculum(&state);

        let question = upsert_admin_question(
            &state,
            admin_mcq_input("Archived content editor record.", 2),
        )
        .expect("question should create");

        archive_admin_question(&state, question.question_id).expect("question should archive");

        assert!(get_admin_question_editor(&state, question.question_id).is_err());

        let archived_editor = get_admin_question_editor_any(&state, question.question_id)
            .expect("archived editor snapshot should load for CMS");
        assert_eq!(archived_editor.question_id, question.question_id);
        assert!(!archived_editor.is_active);
        assert_eq!(archived_editor.stem, "Archived content editor record.");
        assert_eq!(archived_editor.options.len(), 2);

        restore_admin_question(&state, question.question_id).expect("question should restore");
        let active_editor = get_admin_question_editor(&state, question.question_id)
            .expect("active editor snapshot should load after restore");
        assert!(active_editor.is_active);
    }

    #[test]
    fn admin_question_bulk_actions_archive_restore_and_move() {
        let state = AppState::in_memory().expect("in-memory state should build");
        seed_minimal_curriculum(&state);
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO topics (
                        id, subject_id, code, name, node_type
                     ) VALUES (3, 1, 'GEO', 'Geometry', 'topic')",
                    [],
                )
                .unwrap();
                Ok::<_, CommandError>(())
            })
            .unwrap();

        let first = upsert_admin_question(&state, admin_mcq_input("Bulk organize alpha.", 2))
            .expect("first question should create");
        let second = upsert_admin_question(&state, admin_mcq_input("Bulk organize beta.", 2))
            .expect("second question should create");

        let archived = bulk_update_admin_questions(
            &state,
            AdminQuestionBulkActionInput {
                question_ids: vec![first.question_id],
                action: "archive".to_string(),
                subject_id: None,
                topic_id: None,
            },
        )
        .expect("bulk archive should work");

        assert_eq!(archived.requested_count, 1);
        assert_eq!(archived.updated_count, 1);
        assert_eq!(archived.active_count, 0);
        assert_eq!(archived.archived_count, 1);

        let active_list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("Bulk organize".to_string()),
                subject_id: None,
                topic_id: None,
                review_status: None,
                source_type: None,
                active_status: None,
                limit: 20,
            },
        )
        .expect("active question list should load");
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].question_id, second.question_id);

        let restored = bulk_update_admin_questions(
            &state,
            AdminQuestionBulkActionInput {
                question_ids: vec![first.question_id],
                action: "restore".to_string(),
                subject_id: None,
                topic_id: None,
            },
        )
        .expect("bulk restore should work");
        assert_eq!(restored.active_count, 1);
        assert_eq!(restored.archived_count, 0);

        let moved = bulk_update_admin_questions(
            &state,
            AdminQuestionBulkActionInput {
                question_ids: vec![first.question_id, second.question_id],
                action: "move".to_string(),
                subject_id: Some(1),
                topic_id: Some(3),
            },
        )
        .expect("bulk move should work");
        assert_eq!(moved.requested_count, 2);
        assert_eq!(moved.updated_count, 2);
        assert_eq!(moved.active_count, 2);
        assert_eq!(moved.archived_count, 0);

        let geometry_list = list_admin_questions(
            &state,
            AdminQuestionListFilter {
                search: Some("Bulk organize".to_string()),
                subject_id: Some(1),
                topic_id: Some(3),
                review_status: None,
                source_type: None,
                active_status: None,
                limit: 20,
            },
        )
        .expect("moved question list should load");
        assert_eq!(geometry_list.len(), 2);
        assert!(
            geometry_list
                .iter()
                .all(|question| question.topic_name == "Geometry")
        );
    }
}
