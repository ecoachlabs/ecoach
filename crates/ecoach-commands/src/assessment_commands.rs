use ecoach_elite::EliteService;
use ecoach_past_papers::{
    PastPaperCourseSummary, PastPaperInverseSignal, PastPaperTopicCount, PastPaperYear,
    PastPapersService, QuestionAssetMeta,
};
use ecoach_sessions::SessionService;
use ecoach_substrate::EcoachError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{dtos, dtos::SessionSnapshotDto, error::CommandError, state::AppState};

pub type PastPaperInverseSignalDto = PastPaperInverseSignal;
pub type PastPaperCourseSummaryDto = PastPaperCourseSummary;
pub type PastPaperYearDto = PastPaperYear;
pub type PastPaperTopicCountDto = PastPaperTopicCount;

// ── Admin: past paper authoring ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperOptionInput {
    /// Existing option's DB id when the admin is editing in place.
    /// When None, the option is new and gets a fresh row.
    #[serde(default)]
    pub option_id: Option<i64>,
    pub option_label: String,
    pub option_text: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperQuestionInput {
    pub question_id: Option<i64>,
    pub section_label: String,
    pub question_number: Option<String>,
    pub topic_id: i64,
    pub subtopic_id: Option<i64>,
    pub stem: String,
    /// DB-persisted format. Essay + fill-in-the-blank both land here as
    /// `short_answer` — the UI distinction travels in `primary_pedagogic_function`
    /// because the `question_format` CHECK constraint (migration 003) does not
    /// allow new values without a table rebuild.
    pub question_format: String,
    pub primary_pedagogic_function: Option<String>,
    pub explanation_text: Option<String>,
    pub difficulty_level: Option<i64>,
    pub marks: Option<i64>,
    pub options: Vec<AdminPastPaperOptionInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperSaveInput {
    pub paper_id: Option<i64>,
    pub subject_id: i64,
    pub exam_year: i64,
    pub paper_code: Option<String>,
    pub title: String,
    pub questions: Vec<AdminPastPaperQuestionInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperSaveResult {
    pub paper_id: i64,
    pub question_count: i64,
    pub question_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperListItem {
    pub paper_id: i64,
    pub subject_id: i64,
    pub subject_name: String,
    pub exam_year: i64,
    pub paper_code: Option<String>,
    pub title: String,
    pub question_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperQuestionDto {
    pub question_id: i64,
    pub section_label: String,
    pub question_number: Option<String>,
    pub topic_id: i64,
    pub subtopic_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub primary_pedagogic_function: Option<String>,
    pub explanation_text: Option<String>,
    pub difficulty_level: i64,
    pub marks: i64,
    pub options: Vec<AdminPastPaperOptionDto>,
    /// Attached images (bytes NOT included — fetch via
    /// `get_question_asset_bytes` when rendering).
    pub assets: Vec<QuestionAssetMetaDto>,
}

pub type QuestionAssetMetaDto = QuestionAssetMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAssetBytesDto {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperOptionDto {
    pub option_id: i64,
    pub option_label: String,
    pub option_text: String,
    pub is_correct: bool,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPastPaperFullDto {
    pub paper_id: i64,
    pub subject_id: i64,
    pub exam_year: i64,
    pub paper_code: Option<String>,
    pub title: String,
    pub questions: Vec<AdminPastPaperQuestionDto>,
}

pub type RecoveredTextDto = ecoach_intake::RecoveredText;
pub type RecoveredTextPageDto = ecoach_intake::RecoveredTextPage;
pub type EliteProfileDto = dtos::EliteProfileDto;
pub type EliteTopicProfileDto = dtos::EliteTopicProfileDto;
pub type EliteSessionBlueprintDto = dtos::EliteSessionBlueprintDto;
pub type EliteBlueprintReportDto = dtos::EliteBlueprintReportDto;
pub type PastPaperComebackSignalDto = dtos::PastPaperComebackSignalDto;
pub type SessionRemediationPlanDto = dtos::QuestionRemediationPlanDto;
pub type SessionEvidenceFabricDto = dtos::SessionEvidenceFabricDto;

pub fn list_inverse_pressure_families(
    state: &AppState,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperInverseSignalDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn)
            .list_inverse_pressure_families(subject_id, topic_id, limit)?)
    })
}

pub fn build_elite_session_blueprint(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteSessionBlueprintDto, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .build_session_blueprint(student_id, subject_id)?
            .into())
    })
}

pub fn get_elite_profile(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<EliteProfileDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .get_profile(student_id, subject_id)?
            .map(EliteProfileDto::from))
    })
}

pub fn list_elite_topic_domination(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    limit: usize,
) -> Result<Vec<EliteTopicProfileDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .list_topic_domination(student_id, subject_id, limit)?
            .into_iter()
            .map(EliteTopicProfileDto::from)
            .collect())
    })
}

pub fn build_elite_session_blueprint_report(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteBlueprintReportDto, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .build_session_blueprint_report(student_id, subject_id)?
            .into())
    })
}

pub fn list_comeback_candidate_families(
    state: &AppState,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperComebackSignalDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn)
            .list_comeback_candidate_families(subject_id, topic_id, limit)?
            .into_iter()
            .map(PastPaperComebackSignalDto::from)
            .collect())
    })
}

pub fn list_session_remediation_plans(
    state: &AppState,
    session_id: i64,
    limit: usize,
) -> Result<Vec<SessionRemediationPlanDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn)
            .list_session_remediation_plans(session_id, limit)?
            .into_iter()
            .map(SessionRemediationPlanDto::from)
            .collect())
    })
}

pub fn get_session_evidence_fabric(
    state: &AppState,
    session_id: i64,
    limit_events: usize,
) -> Result<Option<SessionEvidenceFabricDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn)
            .get_session_evidence_fabric(session_id, limit_events)?
            .map(SessionEvidenceFabricDto::from))
    })
}

// ── Elite Mode deep commands ──

pub fn score_elite_session(
    state: &AppState,
    student_id: i64,
    session_id: i64,
    session_class: &str,
) -> Result<ecoach_elite::EliteSessionScore, CommandError> {
    state.with_connection(|conn| {
        let service = EliteService::new(conn);
        let score = service.score_session(student_id, session_id, session_class)?;
        Ok(score)
    })
}

pub fn list_elite_personal_bests(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<(String, i64, String)>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).list_personal_bests(student_id, subject_id)?)
    })
}

pub fn check_elite_badges(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<String>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).check_and_award_badges(student_id, subject_id)?)
    })
}

pub fn list_elite_earned_badges(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<(String, String, String)>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).list_earned_elite_badges(student_id, subject_id)?)
    })
}

// ── Past Questions browser commands ───────────────────────────────

pub fn list_past_paper_courses(
    state: &AppState,
) -> Result<Vec<PastPaperCourseSummaryDto>, CommandError> {
    state.with_connection(|conn| Ok(PastPapersService::new(conn).list_past_paper_courses()?))
}

pub fn list_past_papers_for_subject(
    state: &AppState,
    subject_id: i64,
) -> Result<Vec<PastPaperYearDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn).list_past_papers_for_subject(subject_id)?)
    })
}

/// Per-topic question tally across every past paper for a subject.
/// Feeds the "Topic" view of the Past Questions browser.
pub fn list_past_paper_topic_counts(
    state: &AppState,
    subject_id: i64,
) -> Result<Vec<PastPaperTopicCountDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn).list_past_paper_topic_counts(subject_id)?)
    })
}

/// Start a practice session that spans every past-paper question
/// tagged to a given topic across the entire subject, filtered by
/// question format. `format` is one of:
///   - "objective" — mcq + true/false only (default for the Topic view)
///   - "essay"     — short-answer / numeric / fill-blank / etc.
///   - "all"       — both buckets mixed (retained for flexibility; the
///                    UI deliberately never requests this)
pub fn start_past_paper_topic_session(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    topic_id: i64,
    format: String,
    is_timed: bool,
) -> Result<SessionSnapshotDto, CommandError> {
    let format_filter = match format.as_str() {
        "objective" | "essay" | "all" => format.as_str(),
        other => {
            return Err(CommandError {
                code: "bad_request".to_string(),
                message: format!(
                    "invalid format '{}'; expected one of objective|essay|all",
                    other
                ),
            });
        }
    };

    state.with_connection(|conn| {
        let past_papers = PastPapersService::new(conn);
        let question_ids =
            past_papers.list_subject_topic_past_question_ids(subject_id, topic_id, format_filter)?;
        if question_ids.is_empty() {
            return Err(CommandError {
                code: "not_found".to_string(),
                message: format!(
                    "no {} past-paper questions found for subject {} / topic {}",
                    format_filter, subject_id, topic_id
                ),
            });
        }

        let sessions = SessionService::new(conn);
        let focus = format!(
            "past_paper_topic:{}:{}:{}",
            subject_id, topic_id, format_filter
        );
        let (session, _) = sessions.start_session_from_question_ids(
            student_id,
            subject_id,
            &question_ids,
            is_timed,
            &focus,
        )?;
        let snapshot = sessions
            .get_session_snapshot(session.id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("session {} snapshot was not created", session.id),
            })?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

/// Start a practice session over a single paper-section. Loads the
/// ordered question ids from past_paper_question_links and hands them
/// to SessionService::start_session_from_question_ids. Returns the
/// same SessionSnapshotDto as start_practice_session so the frontend
/// can route into the existing session view.
pub fn start_past_paper_section(
    state: &AppState,
    student_id: i64,
    paper_id: i64,
    section_label: String,
    is_timed: bool,
) -> Result<SessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let past_papers = PastPapersService::new(conn);
        let question_ids = past_papers.list_section_question_ids(paper_id, &section_label)?;

        // Look up subject from the paper so the session is tagged correctly.
        let paper = past_papers.get_paper_set(paper_id)?.ok_or(CommandError {
            code: "not_found".to_string(),
            message: format!("past paper {} not found", paper_id),
        })?;

        let sessions = SessionService::new(conn);
        let focus = format!("past_paper_section:{}:{}", paper_id, section_label);
        let (session, _) = sessions.start_session_from_question_ids(
            student_id,
            paper.subject_id,
            &question_ids,
            is_timed,
            &focus,
        )?;
        let snapshot = sessions
            .get_session_snapshot(session.id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("session {} snapshot was not created", session.id),
            })?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

// ── Admin: past paper authoring ───────────────────────────────────

/// Create or update a past paper and all its questions in one
/// transaction. Questions are stored in the shared `questions` table
/// (source_type = 'past_question'); links carry the section label and
/// question number. Editing a paper regenerates all links — questions
/// themselves are updated in place when their id is supplied.
pub fn admin_save_past_paper(
    state: &AppState,
    input: AdminPastPaperSaveInput,
) -> Result<AdminPastPaperSaveResult, CommandError> {
    validate_past_paper_input(&input)?;

    state.with_connection(|conn| {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let outcome = (|| -> Result<AdminPastPaperSaveResult, EcoachError> {
            let papers = PastPapersService::new(conn);

            // 1. Upsert paper set.
            let paper_id = match input.paper_id {
                Some(id) => {
                    papers.update_paper_set(
                        id,
                        input.exam_year,
                        input.paper_code.as_deref(),
                        &input.title,
                    )?;
                    // paper_code lives in the row — update separately because
                    // create_paper_set API doesn't accept it. Inline the update:
                    conn.execute(
                        "UPDATE past_paper_sets SET paper_code = ?2 WHERE id = ?1",
                        params![id, input.paper_code.as_deref()],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    id
                }
                None => {
                    let new_id = papers.create_paper_set(
                        input.subject_id,
                        input.exam_year,
                        &input.title,
                    )?;
                    if let Some(code) = input.paper_code.as_deref() {
                        conn.execute(
                            "UPDATE past_paper_sets SET paper_code = ?2 WHERE id = ?1",
                            params![new_id, code],
                        )
                        .map_err(|err| EcoachError::Storage(err.to_string()))?;
                    }
                    new_id
                }
            };

            // 2. Drop existing links; we rebuild from the incoming list.
            papers.delete_paper_question_links(paper_id)?;

            // 3. For each question: insert or update, then replace options.
            let mut question_ids: Vec<i64> = Vec::with_capacity(input.questions.len());
            for question in input.questions.iter() {
                let question_id = upsert_past_paper_question(
                    conn,
                    input.subject_id,
                    input.exam_year,
                    &input.title,
                    question,
                )?;
                papers.link_question_to_paper(
                    paper_id,
                    question_id,
                    Some(question.section_label.as_str()),
                    question.question_number.as_deref(),
                )?;
                question_ids.push(question_id);
            }

            Ok(AdminPastPaperSaveResult {
                paper_id,
                question_count: question_ids.len() as i64,
                question_ids,
            })
        })();

        match outcome {
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

pub fn admin_list_past_papers(
    state: &AppState,
    subject_id: Option<i64>,
) -> Result<Vec<AdminPastPaperListItem>, CommandError> {
    state.with_connection(|conn| {
        let sql = "SELECT pps.id, pps.subject_id, s.name, pps.exam_year, pps.paper_code,
                          pps.title,
                          (SELECT COUNT(*) FROM past_paper_question_links WHERE paper_id = pps.id)
                            AS question_count,
                          pps.created_at
                   FROM past_paper_sets pps
                   INNER JOIN subjects s ON s.id = pps.subject_id
                   WHERE (?1 IS NULL OR pps.subject_id = ?1)
                   ORDER BY pps.exam_year DESC, s.display_order ASC, pps.id DESC";
        let mut stmt = conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([subject_id], |row| {
                Ok(AdminPastPaperListItem {
                    paper_id: row.get(0)?,
                    subject_id: row.get(1)?,
                    subject_name: row.get(2)?,
                    exam_year: row.get(3)?,
                    paper_code: row.get(4)?,
                    title: row.get(5)?,
                    question_count: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    })
}

pub fn admin_get_past_paper(
    state: &AppState,
    paper_id: i64,
) -> Result<AdminPastPaperFullDto, CommandError> {
    state.with_connection(|conn| {
        let (subject_id, exam_year, paper_code, title): (i64, i64, Option<String>, String) = conn
            .query_row(
                "SELECT subject_id, exam_year, paper_code, title
                 FROM past_paper_sets WHERE id = ?1",
                [paper_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    EcoachError::NotFound(format!("past paper {} not found", paper_id))
                }
                other => EcoachError::Storage(other.to_string()),
            })?;

        // Questions in paper order.
        let mut q_stmt = conn
            .prepare(
                "SELECT q.id, q.topic_id, q.subtopic_id, q.stem, q.question_format,
                        q.explanation_text, q.difficulty_level, q.marks,
                        ppql.section_label, ppql.question_number,
                        q.primary_pedagogic_function
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE ppql.paper_id = ?1
                 ORDER BY COALESCE(ppql.section_label, ''),
                          ppql.question_number, ppql.id",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let q_rows = q_stmt
            .query_map([paper_id], |row| {
                Ok(AdminPastPaperQuestionDto {
                    question_id: row.get(0)?,
                    topic_id: row.get(1)?,
                    subtopic_id: row.get(2)?,
                    stem: row.get(3)?,
                    question_format: row.get(4)?,
                    explanation_text: row.get(5)?,
                    difficulty_level: row.get(6)?,
                    marks: row.get(7)?,
                    section_label: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    question_number: row.get(9)?,
                    primary_pedagogic_function: row.get(10)?,
                    options: Vec::new(),
                    assets: Vec::new(),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions: Vec<AdminPastPaperQuestionDto> = Vec::new();
        for row in q_rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        // Load options per question.
        let mut o_stmt = conn
            .prepare(
                "SELECT id, option_label, option_text, is_correct, position
                 FROM question_options
                 WHERE question_id = ?1
                 ORDER BY position ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for q in questions.iter_mut() {
            let rows = o_stmt
                .query_map([q.question_id], |row| {
                    Ok(AdminPastPaperOptionDto {
                        option_id: row.get(0)?,
                        option_label: row.get(1)?,
                        option_text: row.get(2)?,
                        is_correct: row.get::<_, i64>(3)? == 1,
                        position: row.get(4)?,
                    })
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                q.options
                    .push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }

        // Attach asset metadata in one batched query so the editor can
        // render thumbnails for every question without N+1 round-trips.
        // Bytes are NOT included — the editor fetches those lazily.
        let question_ids: Vec<i64> = questions.iter().map(|q| q.question_id).collect();
        let assets =
            PastPapersService::new(conn).list_question_assets_for_questions(&question_ids)?;
        let mut assets_by_question: std::collections::HashMap<i64, Vec<QuestionAssetMetaDto>> =
            std::collections::HashMap::new();
        for asset in assets {
            assets_by_question
                .entry(asset.question_id)
                .or_default()
                .push(asset);
        }
        for q in questions.iter_mut() {
            if let Some(list) = assets_by_question.remove(&q.question_id) {
                q.assets = list;
            }
        }

        Ok(AdminPastPaperFullDto {
            paper_id,
            subject_id,
            exam_year,
            paper_code,
            title,
            questions,
        })
    })
}

// ── Asset attach / delete / stream ───────────────────────────────

pub fn admin_attach_question_asset(
    state: &AppState,
    question_id: i64,
    scope: String,
    scope_ref: Option<i64>,
    mime_type: String,
    file_bytes: Vec<u8>,
    alt_text: Option<String>,
) -> Result<QuestionAssetMetaDto, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn).attach_question_asset(
            question_id,
            &scope,
            scope_ref,
            &mime_type,
            &file_bytes,
            alt_text.as_deref(),
        )?)
    })
}

pub fn admin_delete_question_asset(
    state: &AppState,
    asset_id: i64,
) -> Result<(), CommandError> {
    state.with_connection(|conn| Ok(PastPapersService::new(conn).delete_question_asset(asset_id)?))
}

pub fn list_question_assets(
    state: &AppState,
    question_id: i64,
) -> Result<Vec<QuestionAssetMetaDto>, CommandError> {
    state.with_connection(|conn| Ok(PastPapersService::new(conn).list_question_assets(question_id)?))
}

pub fn get_question_asset_bytes(
    state: &AppState,
    asset_id: i64,
) -> Result<QuestionAssetBytesDto, CommandError> {
    state.with_connection(|conn| {
        let (mime_type, bytes) = PastPapersService::new(conn)
            .get_question_asset_bytes(asset_id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("asset {} not found", asset_id),
            })?;
        Ok(QuestionAssetBytesDto { mime_type, bytes })
    })
}

pub fn admin_delete_past_paper(state: &AppState, paper_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| Ok(PastPapersService::new(conn).delete_paper_set(paper_id)?))
}

/// Pure file → text over the intake adapter chain. The frontend sends
/// a byte array (so the user can pick a file with a normal HTML
/// `<input type="file">` without needing a native dialog plugin), we
/// persist to a temp file, run the extractors, return, then unlink.
pub fn admin_extract_past_paper_text(
    _state: &AppState,
    file_name: String,
    mime_type: Option<String>,
    file_bytes: Vec<u8>,
) -> Result<RecoveredTextDto, CommandError> {
    use std::env::temp_dir;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    if file_bytes.is_empty() {
        return Err(CommandError {
            code: "validation".to_string(),
            message: "uploaded file is empty".to_string(),
        });
    }

    let stem = std::path::Path::new(&file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("upload");
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut temp_path = temp_dir();
    temp_path.push(format!("ecoach_pp_{}_{}.{}", stem, unique, extension));

    fs::write(&temp_path, &file_bytes).map_err(|err| CommandError {
        code: "io_error".to_string(),
        message: format!("could not write temp file: {}", err),
    })?;

    let recovered =
        ecoach_intake::extract_text_from_file(&temp_path, &file_name, mime_type.as_deref());

    // Best-effort cleanup — don't leak temp files even on error paths.
    let _ = fs::remove_file(&temp_path);

    Ok(recovered)
}

// ── helpers ──────────────────────────────────────────────────────

fn validate_past_paper_input(input: &AdminPastPaperSaveInput) -> Result<(), CommandError> {
    if input.title.trim().is_empty() {
        return Err(CommandError {
            code: "validation".to_string(),
            message: "title is required".to_string(),
        });
    }
    if input.exam_year < 1980 || input.exam_year > 2100 {
        return Err(CommandError {
            code: "validation".to_string(),
            message: format!("exam_year {} is outside the allowed range", input.exam_year),
        });
    }
    if input.subject_id <= 0 {
        return Err(CommandError {
            code: "validation".to_string(),
            message: "subject_id is required".to_string(),
        });
    }
    for (idx, q) in input.questions.iter().enumerate() {
        if q.stem.trim().is_empty() {
            return Err(CommandError {
                code: "validation".to_string(),
                message: format!("question #{} has an empty stem", idx + 1),
            });
        }
        if q.question_format == "mcq" {
            if q.options.is_empty() {
                return Err(CommandError {
                    code: "validation".to_string(),
                    message: format!("MCQ question #{} needs at least one option", idx + 1),
                });
            }
            if !q.options.iter().any(|o| o.is_correct) {
                return Err(CommandError {
                    code: "validation".to_string(),
                    message: format!("MCQ question #{} needs a correct option", idx + 1),
                });
            }
        }
    }
    Ok(())
}

fn upsert_past_paper_question(
    conn: &rusqlite::Connection,
    subject_id: i64,
    exam_year: i64,
    paper_title: &str,
    question: &AdminPastPaperQuestionInput,
) -> Result<i64, EcoachError> {
    let difficulty = question.difficulty_level.unwrap_or(5000);
    let marks = question.marks.unwrap_or_else(|| {
        match question.question_format.as_str() {
            "mcq" | "true_false" => 1,
            _ => 4,
        }
    });
    let estimated_time = match question.question_format.as_str() {
        "mcq" | "true_false" => 45_i64,
        "numeric" | "short_answer" => 90,
        _ => 180,
    };

    let question_id = if let Some(id) = question.question_id {
        let changed = conn
            .execute(
                "UPDATE questions
                 SET subject_id = ?1, topic_id = ?2, subtopic_id = ?3,
                     stem = ?4, question_format = ?5, explanation_text = ?6,
                     difficulty_level = ?7, estimated_time_seconds = ?8, marks = ?9,
                     source_type = 'past_question', source_ref = ?10, exam_year = ?11,
                     primary_pedagogic_function = ?12,
                     updated_at = datetime('now')
                 WHERE id = ?13",
                params![
                    subject_id,
                    question.topic_id,
                    question.subtopic_id,
                    question.stem.trim(),
                    question.question_format,
                    question.explanation_text,
                    difficulty,
                    estimated_time,
                    marks,
                    paper_title,
                    exam_year,
                    question.primary_pedagogic_function,
                    id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if changed == 0 {
            return Err(EcoachError::NotFound(format!("question {} not found", id)));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO questions (
                subject_id, topic_id, subtopic_id, stem, question_format,
                explanation_text, difficulty_level, estimated_time_seconds, marks,
                source_type, source_ref, exam_year,
                primary_pedagogic_function,
                classification_confidence, intelligence_snapshot, is_active,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                'past_question', ?10, ?11,
                ?12,
                0, '{}', 1,
                datetime('now'), datetime('now')
             )",
            params![
                subject_id,
                question.topic_id,
                question.subtopic_id,
                question.stem.trim(),
                question.question_format,
                question.explanation_text,
                difficulty,
                estimated_time,
                marks,
                paper_title,
                exam_year,
                question.primary_pedagogic_function,
            ],
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
        conn.last_insert_rowid()
    };

    // Stable-id upsert for options.
    //
    // The naive approach (DELETE + INSERT all options) regenerates
    // every option_id on save — which would orphan any image assets
    // attached via scope='option', scope_ref=option_id. Instead we:
    //   1. Load the current option ids for this question.
    //   2. For each incoming option with an id that matches an
    //      existing row, UPDATE in place.
    //   3. For incoming options without an id (new rows), INSERT.
    //   4. DELETE rows whose ids aren't present in the incoming list.
    //
    // Images (question_assets with scope='option') keyed to surviving
    // option_ids stay intact. Options that the admin removed get their
    // scoped assets cascade-orphaned — we also clean those up in
    // step 4 so the DB stays tidy.
    let existing_option_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM question_options WHERE question_id = ?1")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([question_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        ids
    };
    let existing_set: std::collections::HashSet<i64> =
        existing_option_ids.iter().copied().collect();
    let mut retained_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();

    for (idx, option) in question.options.iter().enumerate() {
        let position = (idx + 1) as i64;
        match option.option_id {
            Some(id) if existing_set.contains(&id) => {
                conn.execute(
                    "UPDATE question_options
                     SET option_label = ?1, option_text = ?2, is_correct = ?3, position = ?4
                     WHERE id = ?5",
                    params![
                        option.option_label.trim(),
                        option.option_text.trim(),
                        if option.is_correct { 1 } else { 0 },
                        position,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
                retained_ids.insert(id);
            }
            _ => {
                // Either id is None or the claimed id no longer exists
                // on this question (stale client state). Insert fresh.
                conn.execute(
                    "INSERT INTO question_options (
                        question_id, option_label, option_text, is_correct, position
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        question_id,
                        option.option_label.trim(),
                        option.option_text.trim(),
                        if option.is_correct { 1 } else { 0 },
                        position,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
                retained_ids.insert(conn.last_insert_rowid());
            }
        }
    }

    // Prune any options the admin removed — also drops orphaned
    // option-scoped assets so the DB doesn't leak blobs.
    let to_delete: Vec<i64> = existing_option_ids
        .into_iter()
        .filter(|id| !retained_ids.contains(id))
        .collect();
    for dead_id in to_delete {
        conn.execute(
            "DELETE FROM question_assets
             WHERE question_id = ?1 AND scope = 'option' AND scope_ref = ?2",
            params![question_id, dead_id],
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
        conn.execute("DELETE FROM question_options WHERE id = ?1", [dead_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
    }

    Ok(question_id)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_identity::CreateAccountInput;
    use ecoach_questions::QuestionService;
    use ecoach_sessions::{PracticeSessionStartInput, SessionAnswerInput, SessionService};
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{identity_commands, state::AppState};

    use super::*;

    #[test]
    fn assessment_commands_surface_comeback_candidates_and_session_evidence() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                PackService::new(conn).install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");

        let account = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Afia".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("account should create");

        let (subject_id, _topic_id, session_id) = state
            .with_connection(|conn| {
                let question_id: i64 = conn.query_row(
                    "SELECT id FROM questions WHERE family_id IS NOT NULL ORDER BY id ASC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(storage_error)?;
                let subject_id: i64 = conn
                    .query_row(
                        "SELECT subject_id FROM questions WHERE id = ?1",
                        [question_id],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;
                let topic_id: i64 = conn
                    .query_row(
                        "SELECT COALESCE(qf.topic_id, q.topic_id)
                         FROM questions q
                         LEFT JOIN question_families qf ON qf.id = q.family_id
                         WHERE q.id = ?1",
                        [question_id],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;

                let papers = PastPapersService::new(conn);
                let early_paper = papers.create_paper_set(subject_id, 2021, "Past Paper 2021")?;
                let late_paper = papers.create_paper_set(subject_id, 2023, "Past Paper 2023")?;
                papers.link_question_to_paper(early_paper, question_id, None, Some("1"))?;
                papers.link_question_to_paper(late_paper, question_id, None, Some("2"))?;
                papers.recompute_family_analytics(subject_id)?;

                let sessions = SessionService::new(conn);
                let (session, _) = sessions.start_practice_session(&PracticeSessionStartInput {
                    student_id: account.id,
                    subject_id,
                    topic_ids: vec![topic_id],
                    family_ids: Vec::new(),
                    question_count: 2,
                    is_timed: true,
                })?;
                let snapshot = sessions
                    .get_session_snapshot(session.id)?
                    .expect("session snapshot should exist");
                let options =
                    QuestionService::new(conn).list_options(snapshot.items[0].question_id)?;
                let misconception_option = options
                    .iter()
                    .find(|option| option.misconception_id.is_some())
                    .or_else(|| options.iter().find(|option| !option.is_correct))
                    .expect("answer option should exist");
                sessions.record_answer(
                    session.id,
                    &SessionAnswerInput {
                        item_id: snapshot.items[0].id,
                        selected_option_id: misconception_option.id,
                        response_time_ms: Some(48_000),
                    },
                )?;
                sessions.complete_session(session.id)?;

                Ok::<(i64, i64, i64), crate::CommandError>((subject_id, topic_id, session.id))
            })
            .expect("seed data should prepare");

        let comeback = list_comeback_candidate_families(&state, subject_id, None, 5)
            .expect("comeback candidates should load");
        let remediation = list_session_remediation_plans(&state, session_id, 3)
            .expect("session remediation plans should load");
        let fabric = get_session_evidence_fabric(&state, session_id, 10)
            .expect("session evidence fabric should load")
            .expect("session evidence fabric should exist");

        assert!(!comeback.is_empty());
        assert!(comeback[0].comeback_score >= 0);
        assert!(!remediation.is_empty());
        assert_eq!(fabric.session_id, session_id);
        assert!(!fabric.remediation_plans.is_empty());
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.event_type == "session.remediation_planned")
        );
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

    fn storage_error(err: impl ToString) -> crate::CommandError {
        crate::CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        }
    }
}
