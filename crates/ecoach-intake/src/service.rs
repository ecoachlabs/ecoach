use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    AcquisitionEvidenceCandidate, AcquisitionJobReport, BundleFile, BundleProcessReport,
    ContentAcquisitionJob, ExtractedInsight, SubmissionBundle,
};

pub struct IntakeService<'a> {
    conn: &'a Connection,
}

impl<'a> IntakeService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_bundle(&self, student_id: i64, title: &str) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO submission_bundles (student_id, title) VALUES (?1, ?2)",
                params![student_id, title],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_bundle_file(
        &self,
        bundle_id: i64,
        file_name: &str,
        file_path: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO bundle_files (bundle_id, file_name, file_path, mime_type)
                 VALUES (?1, ?2, ?3, ?4)",
                params![bundle_id, file_name, file_path, infer_mime_type(file_name)],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_bundle_files(&self, bundle_id: i64) -> EcoachResult<Vec<BundleFile>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, bundle_id, file_name, file_path, mime_type
                 FROM bundle_files
                 WHERE bundle_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([bundle_id], |row| {
                let file_name: String = row.get(2)?;
                let mime_type: Option<String> = row.get(4)?;
                let file_kind = infer_file_kind(&file_name, mime_type.as_deref());
                Ok(BundleFile {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    file_name,
                    file_path: row.get(3)?,
                    mime_type,
                    file_kind,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn reconstruct_bundle(&self, bundle_id: i64) -> EcoachResult<BundleProcessReport> {
        let _bundle = self
            .get_bundle(bundle_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("bundle {} not found", bundle_id)))?;
        let files = self.list_bundle_files(bundle_id)?;
        if files.is_empty() {
            return Err(EcoachError::Validation(
                "cannot reconstruct a bundle without files".to_string(),
            ));
        }

        self.update_bundle_status(bundle_id, "processing")?;
        self.conn
            .execute(
                "DELETE FROM extracted_insights WHERE bundle_id = ?1",
                [bundle_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut analyses = Vec::new();

        for file in &files {
            let inferred_mime = infer_mime_type(&file.file_name);
            self.conn
                .execute(
                    "UPDATE bundle_files
                     SET mime_type = COALESCE(mime_type, ?1)
                     WHERE id = ?2",
                    params![inferred_mime, file.id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let path = PathBuf::from(&file.file_path);
            let exists = path.exists();
            let text_recovery = recover_text_sample(&path, &file.file_kind, exists);
            let analysis = analyze_bundle_file(file, inferred_mime, exists, &text_recovery);
            self.insert_insight(bundle_id, "file_reconstruction", &analysis.payload)?;
            analyses.push(analysis);
        }

        let document_groups = reconstruct_document_groups(&analyses);
        let reconstructed_document_count = document_groups.len() as i64;
        let unresolved_alignment_count = count_unresolved_alignment_documents(&document_groups);
        let bundle_overview = summarize_bundle(&analyses, document_groups.len());
        let final_status = if !bundle_overview.review_reasons.is_empty() {
            "review_required"
        } else {
            "completed"
        };
        let bundle_reconstruction_payload = json!({
            "bundle_kind": &bundle_overview.bundle_kind,
            "document_groups": &document_groups,
            "detected_topics": &bundle_overview.detected_topics,
            "detected_dates": &bundle_overview.detected_dates,
            "weakness_signals": &bundle_overview.weakness_signals,
            "recommended_actions": &bundle_overview.recommended_actions,
            "quality_signals": {
                "reconstruction_confidence_score": bundle_overview.reconstruction_confidence_score,
                "review_priority": &bundle_overview.review_priority,
                "paired_assessment_document_count": bundle_overview.paired_assessment_document_count,
                "ocr_recovered_file_count": bundle_overview.ocr_recovered_file_count,
                "needs_confirmation": bundle_overview.needs_confirmation,
                "unresolved_alignment_count": unresolved_alignment_count,
            },
            "files_requiring_review": analyses
                .iter()
                .filter(|analysis| !analysis.review_reasons.is_empty())
                .map(|analysis| {
                    json!({
                        "file_id": analysis.file_id,
                        "file_name": &analysis.file_name,
                        "document_role": &analysis.document_role,
                        "review_reasons": &analysis.review_reasons,
                    })
                })
                .collect::<Vec<_>>(),
            "requires_review": !bundle_overview.review_reasons.is_empty(),
        });
        self.insert_insight(
            bundle_id,
            "bundle_reconstruction",
            &bundle_reconstruction_payload,
        )?;
        let overview_payload = json!({
            "file_count": files.len(),
            "detected_topics": &bundle_overview.detected_topics,
            "detected_dates": &bundle_overview.detected_dates,
            "question_like_file_count": bundle_overview.question_like_file_count,
            "answer_like_file_count": bundle_overview.answer_like_file_count,
            "ocr_candidate_file_count": bundle_overview.ocr_candidate_file_count,
            "ocr_recovered_file_count": bundle_overview.ocr_recovered_file_count,
            "layout_recovered_file_count": bundle_overview.layout_recovered_file_count,
            "estimated_question_count": bundle_overview.estimated_question_count,
            "estimated_answer_count": bundle_overview.estimated_answer_count,
            "reconstructed_document_count": reconstructed_document_count,
            "paired_assessment_document_count": bundle_overview.paired_assessment_document_count,
            "reconstruction_confidence_score": bundle_overview.reconstruction_confidence_score,
            "extracted_question_block_count": bundle_overview.extracted_question_block_count,
            "score_signal_count": bundle_overview.score_signal_count,
            "remark_signal_count": bundle_overview.remark_signal_count,
            "needs_confirmation": bundle_overview.needs_confirmation,
            "unresolved_alignment_count": unresolved_alignment_count,
            "review_priority": &bundle_overview.review_priority,
            "bundle_kind": &bundle_overview.bundle_kind,
            "detected_document_roles": &bundle_overview.document_roles,
            "weakness_signals": &bundle_overview.weakness_signals,
            "recommended_actions": &bundle_overview.recommended_actions,
            "detected_subjects": &bundle_overview.detected_subjects,
            "detected_exam_years": &bundle_overview.detected_exam_years,
            "review_reasons": &bundle_overview.review_reasons,
            "requires_review": !bundle_overview.review_reasons.is_empty(),
        });
        self.insert_insight(bundle_id, "bundle_overview", &overview_payload)?;
        self.update_bundle_status(bundle_id, final_status)?;

        self.get_bundle_report(bundle_id)
    }

    pub fn list_bundle_insights(&self, bundle_id: i64) -> EcoachResult<Vec<ExtractedInsight>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, bundle_id, insight_type, payload_json, created_at
                 FROM extracted_insights
                 WHERE bundle_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([bundle_id], |row| {
                let payload_json: String = row.get(3)?;
                let payload = serde_json::from_str::<Value>(&payload_json).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })?;
                Ok(ExtractedInsight {
                    id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    insight_type: row.get(2)?,
                    payload,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn get_bundle_report(&self, bundle_id: i64) -> EcoachResult<BundleProcessReport> {
        let bundle = self
            .get_bundle(bundle_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("bundle {} not found", bundle_id)))?;
        let files = self.list_bundle_files(bundle_id)?;
        let insights = self.list_bundle_insights(bundle_id)?;

        let mut detected_subjects = BTreeSet::new();
        let mut detected_exam_years = BTreeSet::new();
        let mut detected_topics = BTreeSet::new();
        let mut detected_dates = BTreeSet::new();
        let mut question_like_file_count = 0i64;
        let mut answer_like_file_count = 0i64;
        let mut ocr_candidate_file_count = 0i64;
        let mut ocr_recovered_file_count = 0i64;
        let mut layout_recovered_file_count = 0i64;
        let mut estimated_question_count = 0i64;
        let mut estimated_answer_count = 0i64;
        let mut reconstructed_document_count = 0i64;
        let mut paired_assessment_document_count = 0i64;
        let mut reconstruction_confidence_score = 0i64;
        let mut extracted_question_block_count = 0i64;
        let mut score_signal_count = 0i64;
        let mut remark_signal_count = 0i64;
        let mut needs_confirmation = false;
        let mut unresolved_alignment_count = 0i64;
        let mut review_priority = "low".to_string();
        let mut bundle_kind = "unknown".to_string();
        let mut detected_document_roles = BTreeSet::new();
        let mut weakness_signals = BTreeSet::new();
        let mut recommended_actions = BTreeSet::new();
        let mut review_reasons = BTreeSet::new();
        for insight in &insights {
            if insight.insight_type == "bundle_overview" {
                for subject in collect_string_values(&insight.payload, "detected_subjects") {
                    detected_subjects.insert(subject);
                }
                for year in collect_i64_values(&insight.payload, "detected_exam_years") {
                    detected_exam_years.insert(year);
                }
                for topic in collect_string_values(&insight.payload, "detected_topics") {
                    detected_topics.insert(topic);
                }
                for date in collect_string_values(&insight.payload, "detected_dates") {
                    detected_dates.insert(date);
                }
                if let Some(count) = insight
                    .payload
                    .get("question_like_file_count")
                    .and_then(Value::as_i64)
                {
                    question_like_file_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("answer_like_file_count")
                    .and_then(Value::as_i64)
                {
                    answer_like_file_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("ocr_candidate_file_count")
                    .and_then(Value::as_i64)
                {
                    ocr_candidate_file_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("ocr_recovered_file_count")
                    .and_then(Value::as_i64)
                {
                    ocr_recovered_file_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("layout_recovered_file_count")
                    .and_then(Value::as_i64)
                {
                    layout_recovered_file_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("estimated_question_count")
                    .and_then(Value::as_i64)
                {
                    estimated_question_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("estimated_answer_count")
                    .and_then(Value::as_i64)
                {
                    estimated_answer_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("reconstructed_document_count")
                    .and_then(Value::as_i64)
                {
                    reconstructed_document_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("paired_assessment_document_count")
                    .and_then(Value::as_i64)
                {
                    paired_assessment_document_count = count;
                }
                if let Some(score) = insight
                    .payload
                    .get("reconstruction_confidence_score")
                    .and_then(Value::as_i64)
                {
                    reconstruction_confidence_score = score;
                }
                if let Some(count) = insight
                    .payload
                    .get("extracted_question_block_count")
                    .and_then(Value::as_i64)
                {
                    extracted_question_block_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("score_signal_count")
                    .and_then(Value::as_i64)
                {
                    score_signal_count = count;
                }
                if let Some(count) = insight
                    .payload
                    .get("remark_signal_count")
                    .and_then(Value::as_i64)
                {
                    remark_signal_count = count;
                }
                if let Some(value) = insight
                    .payload
                    .get("needs_confirmation")
                    .and_then(Value::as_bool)
                {
                    needs_confirmation = value;
                }
                if let Some(count) = insight
                    .payload
                    .get("unresolved_alignment_count")
                    .and_then(Value::as_i64)
                {
                    unresolved_alignment_count = count;
                }
                if let Some(priority) = insight
                    .payload
                    .get("review_priority")
                    .and_then(Value::as_str)
                {
                    review_priority = priority.to_string();
                }
                if let Some(kind) = insight.payload.get("bundle_kind").and_then(Value::as_str) {
                    bundle_kind = kind.to_string();
                }
                for role in collect_string_values(&insight.payload, "detected_document_roles") {
                    detected_document_roles.insert(role);
                }
                for weakness in collect_string_values(&insight.payload, "weakness_signals") {
                    weakness_signals.insert(weakness);
                }
                for action in collect_string_values(&insight.payload, "recommended_actions") {
                    recommended_actions.insert(action);
                }
                for reason in collect_string_values(&insight.payload, "review_reasons") {
                    review_reasons.insert(reason);
                }
            }
        }

        Ok(BundleProcessReport {
            bundle,
            files,
            insights,
            detected_subjects: detected_subjects.into_iter().collect(),
            detected_exam_years: detected_exam_years.into_iter().collect(),
            detected_topics: detected_topics.into_iter().collect(),
            detected_dates: detected_dates.into_iter().collect(),
            question_like_file_count,
            answer_like_file_count,
            ocr_candidate_file_count,
            ocr_recovered_file_count,
            layout_recovered_file_count,
            estimated_question_count,
            estimated_answer_count,
            reconstructed_document_count,
            paired_assessment_document_count,
            reconstruction_confidence_score,
            extracted_question_block_count,
            score_signal_count,
            remark_signal_count,
            needs_confirmation,
            unresolved_alignment_count,
            review_priority,
            bundle_kind,
            detected_document_roles: detected_document_roles.into_iter().collect(),
            weakness_signals: weakness_signals.into_iter().collect(),
            recommended_actions: recommended_actions.into_iter().collect(),
            review_reasons: review_reasons.into_iter().collect(),
        })
    }

    pub fn create_acquisition_job(
        &self,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        intent_type: &str,
        query_text: &str,
        source_scope: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO content_acquisition_jobs (
                    subject_id, topic_id, intent_type, query_text, source_scope
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![subject_id, topic_id, intent_type, query_text, source_scope],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_acquisition_candidate(
        &self,
        job_id: i64,
        source_label: &str,
        source_url: Option<&str>,
        source_kind: &str,
        title: Option<&str>,
        snippet: Option<&str>,
        extracted_payload: &Value,
        quality_score: i64,
        freshness_score: i64,
    ) -> EcoachResult<i64> {
        let payload_json = serde_json::to_string(extracted_payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO acquisition_evidence_candidates (
                    job_id, source_label, source_url, source_kind, title, snippet,
                    extracted_payload_json, quality_score, freshness_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    job_id,
                    source_label,
                    source_url,
                    source_kind,
                    title,
                    snippet,
                    payload_json,
                    quality_score.clamp(0, 10_000),
                    freshness_score.clamp(0, 10_000),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE content_acquisition_jobs
                 SET status = 'running', updated_at = datetime('now')
                 WHERE id = ?1 AND status = 'queued'",
                [job_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn complete_acquisition_job(
        &self,
        job_id: i64,
        status: &str,
        result_summary: &Value,
    ) -> EcoachResult<()> {
        let result_summary_json = serde_json::to_string(result_summary)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE content_acquisition_jobs
                 SET status = ?1,
                     result_summary_json = ?2,
                     updated_at = datetime('now'),
                     completed_at = CASE WHEN ?1 IN ('completed', 'failed', 'review_required') THEN datetime('now') ELSE completed_at END
                 WHERE id = ?3",
                params![status, result_summary_json, job_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn list_acquisition_jobs(
        &self,
        status: Option<&str>,
    ) -> EcoachResult<Vec<ContentAcquisitionJob>> {
        let mut jobs = Vec::new();
        if let Some(status) = status {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, subject_id, topic_id, intent_type, query_text, source_scope,
                            status, result_summary_json
                     FROM content_acquisition_jobs
                     WHERE status = ?1
                     ORDER BY created_at DESC, id DESC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([status], map_acquisition_job)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                jobs.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        } else {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, subject_id, topic_id, intent_type, query_text, source_scope,
                            status, result_summary_json
                     FROM content_acquisition_jobs
                     ORDER BY created_at DESC, id DESC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([], map_acquisition_job)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                jobs.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }
        Ok(jobs)
    }

    pub fn get_acquisition_job_report(
        &self,
        job_id: i64,
    ) -> EcoachResult<Option<AcquisitionJobReport>> {
        let job = self
            .conn
            .query_row(
                "SELECT id, subject_id, topic_id, intent_type, query_text, source_scope,
                        status, result_summary_json
                 FROM content_acquisition_jobs
                 WHERE id = ?1",
                [job_id],
                map_acquisition_job,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some(job) = job else {
            return Ok(None);
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, job_id, source_label, source_url, source_kind, title, snippet,
                        extracted_payload_json, quality_score, freshness_score, review_status
                 FROM acquisition_evidence_candidates
                 WHERE job_id = ?1
                 ORDER BY quality_score DESC, freshness_score DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([job_id], map_acquisition_candidate)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut candidates = Vec::new();
        for row in rows {
            candidates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(Some(AcquisitionJobReport { job, candidates }))
    }

    fn get_bundle(&self, bundle_id: i64) -> EcoachResult<Option<SubmissionBundle>> {
        self.conn
            .query_row(
                "SELECT id, student_id, title, status
                 FROM submission_bundles
                 WHERE id = ?1",
                [bundle_id],
                |row| {
                    Ok(SubmissionBundle {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        title: row.get(2)?,
                        status: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn update_bundle_status(&self, bundle_id: i64, status: &str) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE submission_bundles
                 SET status = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![status, bundle_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_insight(
        &self,
        bundle_id: i64,
        insight_type: &str,
        payload: &Value,
    ) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO extracted_insights (bundle_id, insight_type, payload_json)
                 VALUES (?1, ?2, ?3)",
                params![bundle_id, insight_type, payload_json],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

struct FileReconstruction {
    file_id: i64,
    file_name: String,
    document_role: String,
    document_origin: String,
    document_key: String,
    detected_subjects: Vec<String>,
    detected_exam_years: Vec<i64>,
    detected_topics: Vec<String>,
    detected_dates: Vec<String>,
    question_like: bool,
    answer_like: bool,
    ocr_candidate: bool,
    ocr_recovered: bool,
    layout_recovered: bool,
    estimated_question_count: i64,
    estimated_answer_count: i64,
    extracted_question_block_count: i64,
    score_signal_count: i64,
    remark_signal_count: i64,
    needs_confirmation: bool,
    layout_kind: String,
    layout_confidence_score: i64,
    reconstruction_confidence_score: i64,
    review_priority: String,
    weakness_signals: Vec<String>,
    recommended_actions: Vec<String>,
    review_reasons: Vec<String>,
    payload: Value,
}

#[derive(Default)]
struct TextProfile {
    source: String,
    line_count: i64,
    non_empty_line_count: i64,
    character_count: i64,
    word_count: i64,
    page_count: i64,
    block_count: i64,
    recovery_confidence_score: i64,
    preview_lines: Vec<String>,
    page_previews: Vec<String>,
}

#[derive(Default)]
struct TextRecovery {
    source: String,
    text: Option<String>,
    page_count: i64,
    block_count: i64,
    confidence_score: i64,
    page_previews: Vec<String>,
    page_summaries: Vec<Value>,
    recovered_from_ocr: bool,
}

#[derive(Default)]
struct LayoutSignals {
    layout_kind: String,
    confidence_score: i64,
    heading_count: i64,
    question_prompt_count: i64,
    choice_option_count: i64,
    answer_key_line_count: i64,
    instruction_line_count: i64,
    formula_line_count: i64,
    table_like_line_count: i64,
    diagram_signal_count: i64,
    mark_allocation_count: i64,
    question_candidates: Vec<String>,
    answer_candidates: Vec<String>,
    instruction_candidates: Vec<String>,
    formula_candidates: Vec<String>,
}

#[derive(Default)]
struct DocumentIntelligence {
    document_origin: String,
    detected_dates: Vec<String>,
    detected_topics: Vec<String>,
    question_blocks: Vec<Value>,
    score_signals: Vec<Value>,
    remark_signals: Vec<String>,
    glossary_terms: Vec<String>,
    question_patterns: Vec<String>,
    weakness_signals: Vec<String>,
    coach_actions: Vec<String>,
    student_model_updates: Vec<String>,
}

struct BundleOverviewSummary {
    detected_subjects: Vec<String>,
    detected_exam_years: Vec<i64>,
    detected_topics: Vec<String>,
    detected_dates: Vec<String>,
    question_like_file_count: i64,
    answer_like_file_count: i64,
    ocr_candidate_file_count: i64,
    ocr_recovered_file_count: i64,
    layout_recovered_file_count: i64,
    estimated_question_count: i64,
    estimated_answer_count: i64,
    paired_assessment_document_count: i64,
    reconstruction_confidence_score: i64,
    extracted_question_block_count: i64,
    score_signal_count: i64,
    remark_signal_count: i64,
    needs_confirmation: bool,
    review_priority: String,
    bundle_kind: String,
    document_roles: Vec<String>,
    weakness_signals: Vec<String>,
    recommended_actions: Vec<String>,
    review_reasons: Vec<String>,
}

fn recover_text_sample(path: &Path, file_kind: &str, exists: bool) -> TextRecovery {
    if !exists {
        return TextRecovery {
            source: "missing_file".to_string(),
            ..TextRecovery::default()
        };
    }

    if is_text_like(file_kind) {
        if let Ok(text) = fs::read_to_string(path) {
            return TextRecovery {
                source: "native_text".to_string(),
                text: Some(text),
                page_count: 1,
                confidence_score: 95,
                ..TextRecovery::default()
            };
        }
    }

    if let Some(recovery) = recover_sidecar_text(path, file_kind) {
        return recovery;
    }

    let source = match file_kind {
        "image" => "image_requires_ocr",
        "pdf" => "pdf_requires_ocr",
        "document" => "document_requires_extraction",
        "spreadsheet" => "spreadsheet_requires_extraction",
        "archive" => "archive_binary",
        "unknown" => "unsupported_binary",
        _ => "missing_text",
    };

    TextRecovery {
        source: source.to_string(),
        ..TextRecovery::default()
    }
}

fn analyze_bundle_file(
    file: &BundleFile,
    inferred_mime: Option<&str>,
    exists: bool,
    text_recovery: &TextRecovery,
) -> FileReconstruction {
    let sample_text = text_recovery.text.as_deref();
    let mut years = detect_exam_years(&file.file_name);
    if let Some(text) = sample_text {
        years.extend(detect_exam_years(text));
    }
    let detected_exam_years = unique_years(years);

    let mut subjects = detect_subject_hints(&file.file_name);
    if let Some(text) = sample_text {
        subjects.extend(detect_subject_hints(text));
    }
    let detected_subjects = unique_strings(subjects);

    let text_profile = build_text_profile(text_recovery, &file.file_kind);
    let layout = build_layout_signals(sample_text);
    let document_role =
        detect_document_role(&file.file_name, sample_text, &file.file_kind, &layout);
    let document_intelligence =
        mine_document_intelligence(&file.file_name, sample_text, &document_role, &layout);
    let question_like = matches!(
        document_role.as_str(),
        "question_paper" | "worksheet" | "mixed_assessment"
    ) || is_question_like(&file.file_name, sample_text)
        || layout.question_prompt_count > 0;
    let answer_like = matches!(
        document_role.as_str(),
        "mark_scheme" | "answer_sheet" | "mixed_assessment" | "student_work" | "corrected_script"
    ) || layout.answer_key_line_count > 0;
    let ocr_candidate = matches!(
        file.file_kind.as_str(),
        "image" | "pdf" | "document" | "spreadsheet"
    ) && sample_text.is_none()
        && exists;
    let ocr_recovered = text_recovery.recovered_from_ocr;
    let layout_recovered = sample_text.is_some()
        && (layout.confidence_score >= 20 || text_recovery.recovered_from_ocr);
    let estimated_question_count = if question_like {
        layout
            .question_prompt_count
            .max(document_intelligence.question_blocks.len() as i64)
            .max(1)
    } else {
        layout
            .question_prompt_count
            .max(document_intelligence.question_blocks.len() as i64)
    };
    let estimated_answer_count = if answer_like {
        layout.answer_key_line_count.max(1)
    } else {
        layout.answer_key_line_count
    };
    let review_reasons = detect_review_reasons(
        exists,
        &file.file_kind,
        sample_text,
        &document_role,
        question_like,
        answer_like,
        ocr_candidate,
        ocr_recovered,
        &text_profile,
        &layout,
        &detected_subjects,
        &document_intelligence,
    );
    let needs_confirmation = !review_reasons.is_empty()
        || (ocr_recovered && text_recovery.confidence_score < 70)
        || document_role == "unknown";
    let content_signals =
        build_content_signals(&document_role, question_like, answer_like, &layout);
    let reconstruction_confidence_score = score_reconstruction_confidence(
        exists,
        sample_text.is_some(),
        &detected_subjects,
        &detected_exam_years,
        &document_role,
        ocr_candidate,
        &layout,
        &review_reasons,
    );
    let review_priority = review_priority_from_reasons(&review_reasons);
    let document_key = build_document_key(
        file.id,
        &file.file_name,
        &detected_subjects,
        &detected_exam_years,
    );
    let ocr_strategy = if ocr_recovered {
        text_recovery.source.as_str()
    } else if ocr_candidate {
        match file.file_kind.as_str() {
            "image" => "image_ocr",
            "pdf" => "pdf_ocr",
            "document" => "document_text_recovery",
            "spreadsheet" => "spreadsheet_text_recovery",
            _ => "ocr_recovery",
        }
    } else if sample_text.is_some() {
        text_recovery.source.as_str()
    } else if file.file_kind == "archive" {
        "archive_unpack"
    } else {
        "manual_review"
    };
    let ocr_status = if ocr_recovered {
        "recovered"
    } else if ocr_candidate {
        "required"
    } else if sample_text.is_some() {
        "not_needed"
    } else if exists {
        "unavailable"
    } else {
        "missing_file"
    };
    let ocr_confidence_score = if ocr_recovered {
        text_recovery.confidence_score
    } else if sample_text.is_some() {
        text_recovery.confidence_score.max(95)
    } else if ocr_candidate {
        25
    } else if exists {
        40
    } else {
        0
    };
    let payload = json!({
        "file_id": file.id,
        "file_name": &file.file_name,
        "file_path": &file.file_path,
        "file_kind": &file.file_kind,
        "mime_type": inferred_mime,
        "exists": exists,
        "document_role": &document_role,
        "document_origin": &document_intelligence.document_origin,
        "document_key": &document_key,
        "question_like": question_like,
        "answer_like": answer_like,
        "detected_subjects": &detected_subjects,
        "detected_exam_years": &detected_exam_years,
        "text_recovery": {
            "source": &text_profile.source,
            "line_count": text_profile.line_count,
            "non_empty_line_count": text_profile.non_empty_line_count,
            "character_count": text_profile.character_count,
            "word_count": text_profile.word_count,
            "page_count": text_profile.page_count,
            "block_count": text_profile.block_count,
            "recovery_confidence_score": text_profile.recovery_confidence_score,
            "preview_lines": &text_profile.preview_lines,
            "page_previews": &text_profile.page_previews,
        },
        "ocr_recovery": {
            "required": ocr_candidate || ocr_recovered,
            "status": ocr_status,
            "strategy": ocr_strategy,
            "confidence_score": ocr_confidence_score,
            "recovered_from_ocr": ocr_recovered,
        },
        "page_recovery": {
            "page_count": text_recovery.page_count,
            "pages": &text_recovery.page_summaries,
        },
        "layout_recovery": {
            "status": if layout_recovered { "recovered" } else if ocr_candidate { "pending_ocr" } else { "limited" },
            "kind": &layout.layout_kind,
            "confidence_score": layout.confidence_score,
            "heading_count": layout.heading_count,
            "question_prompt_count": layout.question_prompt_count,
            "choice_option_count": layout.choice_option_count,
            "answer_key_line_count": layout.answer_key_line_count,
            "instruction_line_count": layout.instruction_line_count,
            "formula_line_count": layout.formula_line_count,
            "table_like_line_count": layout.table_like_line_count,
            "diagram_signal_count": layout.diagram_signal_count,
            "mark_allocation_count": layout.mark_allocation_count,
        },
        "extraction": {
            "estimated_question_count": estimated_question_count,
            "estimated_answer_count": estimated_answer_count,
            "question_candidates": &layout.question_candidates,
            "answer_candidates": &layout.answer_candidates,
            "instruction_lines": &layout.instruction_candidates,
            "formula_candidates": &layout.formula_candidates,
            "content_signals": &content_signals,
        },
        "document_intelligence": {
            "detected_dates": &document_intelligence.detected_dates,
            "detected_topics": &document_intelligence.detected_topics,
            "question_blocks": &document_intelligence.question_blocks,
            "score_signals": &document_intelligence.score_signals,
            "remark_signals": &document_intelligence.remark_signals,
            "glossary_terms": &document_intelligence.glossary_terms,
            "question_patterns": &document_intelligence.question_patterns,
            "weakness_signals": &document_intelligence.weakness_signals,
            "coach_actions": &document_intelligence.coach_actions,
            "student_model_updates": &document_intelligence.student_model_updates,
        },
        "quality_signals": {
            "reconstruction_confidence_score": reconstruction_confidence_score,
            "review_priority": &review_priority,
            "has_text_recovery": sample_text.is_some(),
            "has_subject_signal": !detected_subjects.is_empty(),
            "has_exam_year_signal": !detected_exam_years.is_empty(),
            "layout_confidence_score": layout.confidence_score,
            "needs_confirmation": needs_confirmation,
        },
        "review_reasons": &review_reasons,
    });

    FileReconstruction {
        file_id: file.id,
        file_name: file.file_name.clone(),
        document_role,
        document_origin: document_intelligence.document_origin,
        document_key,
        detected_subjects,
        detected_exam_years,
        detected_topics: document_intelligence.detected_topics,
        detected_dates: document_intelligence.detected_dates,
        question_like,
        answer_like,
        ocr_candidate,
        ocr_recovered,
        layout_recovered,
        estimated_question_count,
        estimated_answer_count,
        extracted_question_block_count: document_intelligence.question_blocks.len() as i64,
        score_signal_count: document_intelligence.score_signals.len() as i64,
        remark_signal_count: document_intelligence.remark_signals.len() as i64,
        needs_confirmation,
        layout_kind: layout.layout_kind,
        layout_confidence_score: layout.confidence_score,
        reconstruction_confidence_score,
        review_priority,
        weakness_signals: document_intelligence.weakness_signals,
        recommended_actions: document_intelligence.coach_actions,
        review_reasons,
        payload,
    }
}

fn summarize_bundle(
    analyses: &[FileReconstruction],
    reconstructed_document_count: usize,
) -> BundleOverviewSummary {
    let mut detected_subjects = BTreeSet::new();
    let mut detected_exam_years = BTreeSet::new();
    let mut detected_topics = BTreeSet::new();
    let mut detected_dates = BTreeSet::new();
    let mut document_roles = BTreeSet::new();
    let mut weakness_signals = BTreeSet::new();
    let mut recommended_actions = BTreeSet::new();
    let mut review_reasons = BTreeSet::new();
    let mut question_like_file_count = 0i64;
    let mut answer_like_file_count = 0i64;
    let mut ocr_candidate_file_count = 0i64;
    let mut ocr_recovered_file_count = 0i64;
    let mut layout_recovered_file_count = 0i64;
    let mut estimated_question_count = 0i64;
    let mut estimated_answer_count = 0i64;
    let mut extracted_question_block_count = 0i64;
    let mut score_signal_count = 0i64;
    let mut remark_signal_count = 0i64;
    let mut reconstruction_confidence_total = 0i64;

    for analysis in analyses {
        for subject in &analysis.detected_subjects {
            detected_subjects.insert(subject.clone());
        }
        for year in &analysis.detected_exam_years {
            detected_exam_years.insert(*year);
        }
        for topic in &analysis.detected_topics {
            detected_topics.insert(topic.clone());
        }
        for date in &analysis.detected_dates {
            detected_dates.insert(date.clone());
        }
        document_roles.insert(analysis.document_role.clone());
        for weakness in &analysis.weakness_signals {
            weakness_signals.insert(weakness.clone());
        }
        for action in &analysis.recommended_actions {
            recommended_actions.insert(action.clone());
        }
        for reason in &analysis.review_reasons {
            review_reasons.insert(reason.clone());
        }
        if analysis.question_like {
            question_like_file_count += 1;
        }
        if analysis.answer_like {
            answer_like_file_count += 1;
        }
        if analysis.ocr_candidate {
            ocr_candidate_file_count += 1;
        }
        if analysis.ocr_recovered {
            ocr_recovered_file_count += 1;
        }
        if analysis.layout_recovered {
            layout_recovered_file_count += 1;
        }
        estimated_question_count += analysis.estimated_question_count;
        estimated_answer_count += analysis.estimated_answer_count;
        extracted_question_block_count += analysis.extracted_question_block_count;
        score_signal_count += analysis.score_signal_count;
        remark_signal_count += analysis.remark_signal_count;
        if analysis.needs_confirmation {
            // A single uncertain file should keep the whole bundle cautious.
            review_reasons.insert("needs_confirmation".to_string());
        }
        reconstruction_confidence_total += analysis.reconstruction_confidence_score;
    }

    let paired_assessment_document_count = count_paired_assessment_documents(analyses);
    let reconstruction_confidence_score = if analyses.is_empty() {
        0
    } else {
        let avg = reconstruction_confidence_total / analyses.len() as i64;
        let pairing_bonus = (paired_assessment_document_count * 8).min(16);
        (avg + pairing_bonus).clamp(0, 100)
    };
    let review_priority = bundle_review_priority(analyses, &review_reasons);

    BundleOverviewSummary {
        detected_subjects: detected_subjects.into_iter().collect(),
        detected_exam_years: detected_exam_years.into_iter().collect(),
        detected_topics: detected_topics.into_iter().collect(),
        detected_dates: detected_dates.into_iter().collect(),
        question_like_file_count,
        answer_like_file_count,
        ocr_candidate_file_count,
        ocr_recovered_file_count,
        layout_recovered_file_count,
        estimated_question_count,
        estimated_answer_count,
        paired_assessment_document_count,
        reconstruction_confidence_score,
        extracted_question_block_count,
        score_signal_count,
        remark_signal_count,
        needs_confirmation: !review_reasons.is_empty(),
        review_priority,
        bundle_kind: classify_bundle_kind(analyses, reconstructed_document_count),
        document_roles: document_roles.into_iter().collect(),
        weakness_signals: weakness_signals.into_iter().collect(),
        recommended_actions: recommended_actions.into_iter().collect(),
        review_reasons: review_reasons.into_iter().collect(),
    }
}

fn count_paired_assessment_documents(analyses: &[FileReconstruction]) -> i64 {
    let mut grouped_roles: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for analysis in analyses {
        grouped_roles
            .entry(&analysis.document_key)
            .or_default()
            .insert(analysis.document_role.as_str());
    }

    grouped_roles
        .values()
        .filter(|roles| roles.contains("question_paper") && roles.contains("mark_scheme"))
        .count() as i64
}

fn bundle_review_priority(
    analyses: &[FileReconstruction],
    review_reasons: &BTreeSet<String>,
) -> String {
    if analyses
        .iter()
        .any(|analysis| analysis.review_priority == "high")
    {
        return "high".to_string();
    }
    if !review_reasons.is_empty()
        || analyses
            .iter()
            .any(|analysis| analysis.review_priority == "medium")
    {
        return "medium".to_string();
    }

    "low".to_string()
}

fn score_reconstruction_confidence(
    exists: bool,
    has_text_recovery: bool,
    detected_subjects: &[String],
    detected_exam_years: &[i64],
    document_role: &str,
    ocr_candidate: bool,
    layout: &LayoutSignals,
    review_reasons: &[String],
) -> i64 {
    let mut score = 0i64;
    if exists {
        score += 25;
    }
    if has_text_recovery {
        score += 25;
    }
    score += (layout.confidence_score / 2).min(25);
    if !detected_subjects.is_empty() {
        score += 10;
    }
    if !detected_exam_years.is_empty() {
        score += 5;
    }
    if document_role != "unknown" {
        score += 10;
    }
    if ocr_candidate {
        score -= 15;
    }
    score -= (review_reasons.len() as i64 * 10).min(35);
    score.clamp(0, 100)
}

fn review_priority_from_reasons(review_reasons: &[String]) -> String {
    if review_reasons.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "missing_file" | "ocr_required" | "archive_unpack_required" | "ambiguous_document_role"
        )
    }) {
        return "high".to_string();
    }
    if !review_reasons.is_empty() {
        return "medium".to_string();
    }

    "low".to_string()
}

fn reconstruct_document_groups(analyses: &[FileReconstruction]) -> Vec<Value> {
    let mut grouped: BTreeMap<String, Vec<&FileReconstruction>> = BTreeMap::new();
    for analysis in analyses {
        grouped
            .entry(analysis.document_key.clone())
            .or_default()
            .push(analysis);
    }

    grouped
        .into_iter()
        .map(|(document_key, members)| {
            let mut roles = BTreeSet::new();
            let mut subjects = BTreeSet::new();
            let mut years = BTreeSet::new();
            let mut topics = BTreeSet::new();
            let mut dates = BTreeSet::new();
            let mut origins = BTreeSet::new();
            let mut weakness_signals = BTreeSet::new();
            let mut recommended_actions = BTreeSet::new();
            let mut review_reasons = BTreeSet::new();
            let mut estimated_question_count = 0i64;
            let mut estimated_answer_count = 0i64;
            let mut ocr_candidate_file_count = 0i64;
            let mut ocr_recovered_file_count = 0i64;
            let mut extracted_question_block_count = 0i64;
            let mut score_signal_count = 0i64;
            let mut remark_signal_count = 0i64;
            let mut confidence_total = 0i64;

            for member in &members {
                roles.insert(member.document_role.clone());
                origins.insert(member.document_origin.clone());
                for subject in &member.detected_subjects {
                    subjects.insert(subject.clone());
                }
                for year in &member.detected_exam_years {
                    years.insert(*year);
                }
                for topic in &member.detected_topics {
                    topics.insert(topic.clone());
                }
                for date in &member.detected_dates {
                    dates.insert(date.clone());
                }
                for weakness in &member.weakness_signals {
                    weakness_signals.insert(weakness.clone());
                }
                for action in &member.recommended_actions {
                    recommended_actions.insert(action.clone());
                }
                for reason in &member.review_reasons {
                    review_reasons.insert(reason.clone());
                }
                estimated_question_count += member.estimated_question_count;
                estimated_answer_count += member.estimated_answer_count;
                if member.ocr_candidate {
                    ocr_candidate_file_count += 1;
                }
                if member.ocr_recovered {
                    ocr_recovered_file_count += 1;
                }
                extracted_question_block_count += member.extracted_question_block_count;
                score_signal_count += member.score_signal_count;
                remark_signal_count += member.remark_signal_count;
                confidence_total += member.reconstruction_confidence_score;
            }

            let member_files = members
                .iter()
                .map(|member| {
                    json!({
                        "file_id": member.file_id,
                        "file_name": member.file_name,
                        "document_role": member.document_role,
                        "document_origin": member.document_origin,
                        "layout_kind": member.layout_kind,
                        "layout_confidence_score": member.layout_confidence_score,
                        "reconstruction_confidence_score": member.reconstruction_confidence_score,
                        "review_priority": member.review_priority,
                        "needs_confirmation": member.needs_confirmation,
                        "ocr_candidate": member.ocr_candidate,
                        "ocr_recovered": member.ocr_recovered,
                    })
                })
                .collect::<Vec<_>>();
            let role_list = roles.into_iter().collect::<Vec<_>>();
            let reason_list = review_reasons.into_iter().collect::<Vec<_>>();
            let confidence_score = if members.is_empty() {
                0
            } else {
                let avg = confidence_total / members.len() as i64;
                let pairing_bonus = if role_list.iter().any(|role| role == "question_paper")
                    && role_list.iter().any(|role| role == "mark_scheme")
                {
                    10
                } else {
                    0
                };
                (avg + pairing_bonus).clamp(0, 100)
            };
            let review_priority = review_priority_from_reasons(&reason_list);
            let needs_confirmation =
                !reason_list.is_empty() || members.iter().any(|member| member.needs_confirmation);
            json!({
                "document_key": document_key,
                "canonical_label": document_key.replace("__", " "),
                "document_kind": derive_document_group_kind(&role_list),
                "alignment_status": derive_group_alignment_status(&role_list, &reason_list),
                "confidence_score": confidence_score,
                "review_priority": review_priority,
                "needs_confirmation": needs_confirmation,
                "member_count": members.len(),
                "member_files": member_files,
                "roles": role_list,
                "document_origins": origins.into_iter().collect::<Vec<_>>(),
                "detected_subjects": subjects.into_iter().collect::<Vec<_>>(),
                "detected_exam_years": years.into_iter().collect::<Vec<_>>(),
                "detected_topics": topics.into_iter().collect::<Vec<_>>(),
                "detected_dates": dates.into_iter().collect::<Vec<_>>(),
                "estimated_question_count": estimated_question_count,
                "estimated_answer_count": estimated_answer_count,
                "ocr_candidate_file_count": ocr_candidate_file_count,
                "ocr_recovered_file_count": ocr_recovered_file_count,
                "extracted_question_block_count": extracted_question_block_count,
                "score_signal_count": score_signal_count,
                "remark_signal_count": remark_signal_count,
                "weakness_signals": weakness_signals.into_iter().collect::<Vec<_>>(),
                "recommended_actions": recommended_actions.into_iter().collect::<Vec<_>>(),
                "review_reasons": reason_list,
            })
        })
        .collect()
}

fn count_unresolved_alignment_documents(document_groups: &[Value]) -> i64 {
    document_groups
        .iter()
        .filter(|group| {
            matches!(
                group.get("alignment_status").and_then(Value::as_str),
                Some("question_without_answers")
                    | Some("answers_without_questions")
                    | Some("review_required")
            )
        })
        .count() as i64
}

fn classify_bundle_kind(
    analyses: &[FileReconstruction],
    reconstructed_document_count: usize,
) -> String {
    let roles = analyses
        .iter()
        .map(|analysis| analysis.document_role.as_str())
        .collect::<BTreeSet<_>>();

    if roles.contains("question_paper")
        || roles.contains("mark_scheme")
        || roles.contains("answer_sheet")
        || roles.contains("mixed_assessment")
    {
        return "assessment_bundle".to_string();
    }
    if roles.contains("worksheet") {
        return "worksheet_bundle".to_string();
    }
    if roles.contains("study_notes")
        || roles.contains("teacher_handout")
        || roles.contains("text_reference")
    {
        return "study_bundle".to_string();
    }
    if roles.contains("report_card") || roles.contains("corrected_script") {
        return "performance_evidence_bundle".to_string();
    }
    if roles.contains("image_capture") || roles.contains("pdf_scan") {
        return if reconstructed_document_count > 1 {
            "scan_bundle".to_string()
        } else {
            "single_scan_bundle".to_string()
        };
    }

    "mixed_bundle".to_string()
}

fn derive_document_group_kind(roles: &[String]) -> &'static str {
    let role_set = roles.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if role_set.contains("question_paper") && role_set.contains("mark_scheme") {
        "question_and_mark_scheme"
    } else if role_set.contains("question_paper") {
        "question_set"
    } else if role_set.contains("mark_scheme") || role_set.contains("answer_sheet") {
        "answer_material"
    } else if role_set.contains("report_card") || role_set.contains("corrected_script") {
        "performance_evidence"
    } else if role_set.contains("worksheet") {
        "worksheet_set"
    } else if role_set.contains("study_notes") || role_set.contains("teacher_handout") {
        "study_notes"
    } else if role_set.contains("image_capture") || role_set.contains("pdf_scan") {
        "scan_set"
    } else {
        "mixed_set"
    }
}

fn derive_group_alignment_status(roles: &[String], review_reasons: &[String]) -> &'static str {
    let role_set = roles.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if role_set.contains("question_paper") && role_set.contains("mark_scheme") {
        "paired_question_and_mark_scheme"
    } else if role_set.contains("question_paper") {
        "question_without_answers"
    } else if role_set.contains("mark_scheme") {
        "answers_without_questions"
    } else if role_set.contains("report_card") || role_set.contains("corrected_script") {
        "performance_signal"
    } else if !review_reasons.is_empty() {
        "review_required"
    } else {
        "standalone"
    }
}

fn collect_string_values(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(|value| value.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn collect_i64_values(payload: &Value, key: &str) -> Vec<i64> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

fn build_text_profile(text_recovery: &TextRecovery, file_kind: &str) -> TextProfile {
    let Some(text) = text_recovery.text.as_deref() else {
        let source = if text_recovery.source.is_empty() {
            match file_kind {
                "image" | "pdf" => "no_embedded_text",
                "document" => "document_requires_extraction",
                "spreadsheet" => "spreadsheet_requires_extraction",
                "archive" => "archive_binary",
                "unknown" => "unsupported_binary",
                _ => "missing_text",
            }
        } else {
            text_recovery.source.as_str()
        };
        return TextProfile {
            source: source.to_string(),
            page_count: text_recovery.page_count,
            block_count: text_recovery.block_count,
            recovery_confidence_score: text_recovery.confidence_score,
            page_previews: text_recovery.page_previews.clone(),
            ..TextProfile::default()
        };
    };

    let preview_lines = text
        .lines()
        .filter_map(|line| trim_display_line(line, 140))
        .take(5)
        .collect::<Vec<_>>();
    let line_count = text.lines().count() as i64;
    let non_empty_line_count = text.lines().filter(|line| !line.trim().is_empty()).count() as i64;
    let character_count = text.chars().count() as i64;
    let word_count = text.split_whitespace().count() as i64;

    TextProfile {
        source: text_recovery.source.clone(),
        line_count,
        non_empty_line_count,
        character_count,
        word_count,
        page_count: text_recovery.page_count.max(1),
        block_count: text_recovery.block_count.max(non_empty_line_count),
        recovery_confidence_score: text_recovery.confidence_score,
        preview_lines,
        page_previews: text_recovery.page_previews.clone(),
    }
}

fn recover_sidecar_text(path: &Path, file_kind: &str) -> Option<TextRecovery> {
    if !matches!(file_kind, "image" | "pdf" | "document" | "spreadsheet") {
        return None;
    }

    for candidate in candidate_recovery_paths(path) {
        if !candidate.exists() {
            continue;
        }

        if extension_of(candidate.to_string_lossy().as_ref()).as_deref() == Some("json") {
            if let Some(recovery) = read_json_sidecar_recovery(&candidate) {
                return Some(recovery);
            }
            continue;
        }

        if let Ok(text) = fs::read_to_string(&candidate) {
            if text.trim().is_empty() {
                continue;
            }
            let block_count = text.lines().filter(|line| !line.trim().is_empty()).count() as i64;
            return Some(TextRecovery {
                source: classify_sidecar_source(&candidate),
                page_count: estimate_page_count(&text),
                block_count,
                confidence_score: 72,
                page_previews: text
                    .split("\u{c}")
                    .filter_map(|page| trim_display_line(page, 120))
                    .take(3)
                    .collect(),
                page_summaries: vec![json!({
                    "page_number": 1,
                    "label": infer_page_label(&text, "text_sidecar"),
                    "confidence_score": 72,
                    "preview": trim_display_line(&text, 120),
                })],
                text: Some(text),
                recovered_from_ocr: true,
            });
        }
    }

    None
}

fn candidate_recovery_paths(path: &Path) -> Vec<PathBuf> {
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(file_name);

    let mut candidates = Vec::new();
    for suffix in [
        format!("{file_name}.ocr.json"),
        format!("{stem}.ocr.json"),
        format!("{file_name}.ocr.txt"),
        format!("{stem}.ocr.txt"),
        format!("{stem}.txt"),
        format!("{stem}.md"),
        format!("{stem}.json"),
    ] {
        let candidate = parent.join(suffix);
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn read_json_sidecar_recovery(path: &Path) -> Option<TextRecovery> {
    let raw = fs::read_to_string(path).ok()?;
    let payload = serde_json::from_str::<Value>(&raw).ok()?;

    let mut page_previews = Vec::new();
    let mut page_summaries = Vec::new();
    let mut page_texts = Vec::new();
    let mut block_count = 0i64;
    let mut confidence_values = Vec::new();

    if let Some(value) = extract_confidence_from_value(&payload) {
        confidence_values.push(value);
    }

    if let Some(pages) = payload.get("pages").and_then(Value::as_array) {
        for (index, page) in pages.iter().enumerate() {
            let page_confidence = extract_confidence_from_value(page).unwrap_or(78);
            confidence_values.push(page_confidence);
            if let Some(blocks) = page.get("blocks").and_then(Value::as_array) {
                block_count += blocks.len() as i64;
            }
            if let Some(text) = payload_text(page) {
                if let Some(preview) = trim_display_line(&text, 120) {
                    page_previews.push(format!("Page {}: {}", index + 1, preview));
                }
                page_summaries.push(json!({
                    "page_number": page.get("page_number").and_then(Value::as_i64).unwrap_or((index + 1) as i64),
                    "label": infer_page_label(&text, "ocr_page"),
                    "confidence_score": page_confidence,
                    "preview": trim_display_line(&text, 120),
                }));
                page_texts.push(text);
            }
        }
    } else if let Some(blocks) = payload.get("blocks").and_then(Value::as_array) {
        block_count = blocks.len() as i64;
        let text = blocks
            .iter()
            .filter_map(payload_text)
            .collect::<Vec<_>>()
            .join("\n");
        if !text.trim().is_empty() {
            if let Some(preview) = trim_display_line(&text, 120) {
                page_previews.push(preview);
            }
            page_summaries.push(json!({
                "page_number": 1,
                "label": infer_page_label(&text, "ocr_blocks"),
                "confidence_score": extract_confidence_from_value(&payload).unwrap_or(78),
                "preview": trim_display_line(&text, 120),
            }));
            page_texts.push(text);
        }
    }

    let text = payload
        .get("text")
        .and_then(Value::as_str)
        .map(|value| value.to_string())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            if page_texts.is_empty() {
                None
            } else {
                Some(page_texts.join("\n\n"))
            }
        })?;

    if block_count == 0 {
        block_count = text.lines().filter(|line| !line.trim().is_empty()).count() as i64;
    }

    let page_count = payload
        .get("pages")
        .and_then(Value::as_array)
        .map(|pages| pages.len() as i64)
        .unwrap_or_else(|| estimate_page_count(&text));
    let confidence_score = if confidence_values.is_empty() {
        78
    } else {
        (confidence_values.iter().sum::<i64>() / confidence_values.len() as i64).clamp(0, 100)
    };

    Some(TextRecovery {
        source: payload
            .get("source")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
            .unwrap_or_else(|| classify_sidecar_source(path)),
        text: Some(text),
        page_count,
        block_count,
        confidence_score,
        page_previews,
        page_summaries,
        recovered_from_ocr: true,
    })
}

fn payload_text(payload: &Value) -> Option<String> {
    if let Some(text) = payload.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(text) = payload.get("content").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(lines) = payload.get("lines").and_then(Value::as_array) {
        let text = lines
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        if !text.trim().is_empty() {
            return Some(text);
        }
    }
    if let Some(blocks) = payload.get("blocks").and_then(Value::as_array) {
        let text = blocks
            .iter()
            .filter_map(payload_text)
            .collect::<Vec<_>>()
            .join("\n");
        if !text.trim().is_empty() {
            return Some(text);
        }
    }
    None
}

fn extract_confidence_from_value(payload: &Value) -> Option<i64> {
    for key in [
        "confidence_score",
        "confidence",
        "avg_confidence",
        "average_confidence",
    ] {
        if let Some(value) = payload.get(key) {
            if let Some(score) = value.as_i64() {
                return Some(score.clamp(0, 100));
            }
            if let Some(score) = value.as_f64() {
                let normalized = if score <= 1.0 { score * 100.0 } else { score };
                return Some(normalized.round() as i64).map(|value| value.clamp(0, 100));
            }
        }
    }
    None
}

fn classify_sidecar_source(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name.contains(".ocr.") {
        if file_name.ends_with(".json") {
            "ocr_sidecar_json".to_string()
        } else {
            "ocr_sidecar_text".to_string()
        }
    } else if file_name.ends_with(".json") {
        "derived_sidecar_json".to_string()
    } else {
        "derived_sidecar_text".to_string()
    }
}

fn estimate_page_count(text: &str) -> i64 {
    let form_feed_pages = text.split('\u{c}').count() as i64;
    let page_markers = text
        .lines()
        .filter_map(|line| trim_display_line(line, 80))
        .filter(|line| line.to_ascii_lowercase().starts_with("page "))
        .count() as i64;
    form_feed_pages.max(page_markers).max(1)
}

fn infer_page_label(text: &str, fallback: &str) -> String {
    let lowered = text.to_ascii_lowercase();
    if contains_any(
        &lowered,
        &["mark scheme", "correct answer", "award ", "memo"],
    ) {
        "answer_page".to_string()
    } else if contains_any(
        &lowered,
        &["score:", "grade", "report card", "teacher comment"],
    ) {
        "score_summary_page".to_string()
    } else if starts_with_numbered_prompt(text)
        || contains_any(&lowered, &["question", "choose", "solve"])
    {
        "question_page".to_string()
    } else if contains_any(&lowered, &["topic:", "notes", "summary", "handout"]) {
        "notes_page".to_string()
    } else {
        fallback.to_string()
    }
}

fn build_layout_signals(sample_text: Option<&str>) -> LayoutSignals {
    let Some(text) = sample_text else {
        return LayoutSignals {
            layout_kind: "unrecovered".to_string(),
            ..LayoutSignals::default()
        };
    };

    let mut signals = LayoutSignals::default();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 180) else {
            continue;
        };

        if is_heading_line(&trimmed) {
            signals.heading_count += 1;
        }
        if starts_with_numbered_prompt(&trimmed) {
            signals.question_prompt_count += 1;
            if signals.question_candidates.len() < 3 {
                signals.question_candidates.push(trimmed.clone());
            }
        }
        if is_choice_line(&trimmed) {
            signals.choice_option_count += 1;
        }
        if is_answer_key_line(&trimmed) {
            signals.answer_key_line_count += 1;
            if signals.answer_candidates.len() < 3 {
                signals.answer_candidates.push(trimmed.clone());
            }
        }
        if is_instruction_line(&trimmed) {
            signals.instruction_line_count += 1;
            if signals.instruction_candidates.len() < 4 {
                signals.instruction_candidates.push(trimmed.clone());
            }
        }
        if is_formula_line(&trimmed) {
            signals.formula_line_count += 1;
            if signals.formula_candidates.len() < 3 {
                signals.formula_candidates.push(trimmed.clone());
            }
        }
        if trimmed.contains('|') || trimmed.matches(',').count() >= 3 || trimmed.contains('\t') {
            signals.table_like_line_count += 1;
        }
        if contains_diagram_signal(&trimmed) {
            signals.diagram_signal_count += 1;
        }
        if trimmed.to_ascii_lowercase().contains("mark") {
            signals.mark_allocation_count += 1;
        }
    }

    signals.confidence_score = ((signals.question_prompt_count * 18)
        + (signals.answer_key_line_count * 16)
        + (signals.choice_option_count * 5)
        + (signals.heading_count * 6)
        + (signals.instruction_line_count * 4)
        + (signals.formula_line_count * 4)
        + (signals.table_like_line_count * 5))
        .min(100);
    signals.layout_kind =
        if signals.answer_key_line_count >= 2 && signals.question_prompt_count == 0 {
            "answer_key".to_string()
        } else if signals.question_prompt_count >= 2 && signals.choice_option_count >= 2 {
            "multiple_choice_sheet".to_string()
        } else if signals.question_prompt_count >= 1 {
            "question_sheet".to_string()
        } else if signals.table_like_line_count >= 2 {
            "tabular".to_string()
        } else if signals.heading_count >= 2 || signals.instruction_line_count >= 2 {
            "structured_notes".to_string()
        } else {
            "plain_text".to_string()
        };

    signals
}

fn detect_document_role(
    file_name: &str,
    sample_text: Option<&str>,
    file_kind: &str,
    layout: &LayoutSignals,
) -> String {
    let lowered_name = file_name.to_ascii_lowercase();
    let lowered_text = sample_text.unwrap_or_default().to_ascii_lowercase();
    let score_signal_count = extract_score_signals(sample_text).len();
    let remark_signal_count = extract_remark_lines(sample_text).len();
    let has_answer_name_signal = [
        "markscheme",
        "mark scheme",
        "marking scheme",
        "answer key",
        "solutions",
        "solution",
        "memo",
    ]
    .iter()
    .any(|needle| lowered_name.contains(needle));
    let has_note_name_signal = ["notes", "note", "summary", "revision", "lesson"]
        .iter()
        .any(|needle| lowered_name.contains(needle));
    let has_question_name_signal = ["question", "paper", "exam", "mock", "test"]
        .iter()
        .any(|needle| lowered_name.contains(needle));
    let has_worksheet_signal = ["worksheet", "exercise"]
        .iter()
        .any(|needle| lowered_name.contains(needle));
    let has_answer_sheet_signal = ["answer sheet", "answer booklet"]
        .iter()
        .any(|needle| lowered_name.contains(needle));
    let has_teacher_handout_signal = [
        "handout",
        "teacher",
        "revision guide",
        "booklet",
        "lesson note",
        "class note",
    ]
    .iter()
    .any(|needle| lowered_name.contains(needle));
    let has_report_card_signal = ["report card", "terminal report", "progress report"]
        .iter()
        .any(|needle| lowered_name.contains(needle))
        || lowered_text.contains("report card");
    let has_corrected_script_signal = [
        "corrected script",
        "marked assignment",
        "marked test",
        "correction sheet",
    ]
    .iter()
    .any(|needle| lowered_name.contains(needle));
    let has_student_work_signal = lowered_name.contains("student")
        && (lowered_name.contains("answer") || lowered_name.contains("work"));

    if file_kind == "archive" {
        return "archive".to_string();
    }
    if file_kind == "image" && sample_text.is_none() {
        return "image_capture".to_string();
    }
    if file_kind == "pdf" && sample_text.is_none() {
        return "pdf_scan".to_string();
    }
    if has_answer_sheet_signal {
        return "answer_sheet".to_string();
    }
    if has_student_work_signal {
        return "student_work".to_string();
    }
    if has_report_card_signal || (score_signal_count >= 2 && remark_signal_count >= 1) {
        return "report_card".to_string();
    }
    if has_corrected_script_signal
        || (remark_signal_count >= 1
            && layout.question_prompt_count >= 1
            && score_signal_count >= 1)
    {
        return "corrected_script".to_string();
    }
    if has_answer_name_signal
        || lowered_text.contains("mark scheme")
        || lowered_text.contains("correct answer")
        || layout.answer_key_line_count >= 2
    {
        return "mark_scheme".to_string();
    }
    if layout.question_prompt_count > 0 && layout.answer_key_line_count > 0 {
        return "mixed_assessment".to_string();
    }
    if has_worksheet_signal {
        return "worksheet".to_string();
    }
    if has_teacher_handout_signal {
        return "teacher_handout".to_string();
    }
    if has_note_name_signal
        || (layout.heading_count >= 2
            && layout.question_prompt_count == 0
            && layout.answer_key_line_count == 0)
    {
        return "study_notes".to_string();
    }
    if has_question_name_signal
        || is_question_like(file_name, sample_text)
        || layout.question_prompt_count > 0
    {
        return "question_paper".to_string();
    }
    if file_kind == "text" {
        return "text_reference".to_string();
    }

    "unknown".to_string()
}

#[allow(clippy::too_many_arguments)]
fn detect_review_reasons(
    exists: bool,
    file_kind: &str,
    sample_text: Option<&str>,
    document_role: &str,
    question_like: bool,
    answer_like: bool,
    ocr_candidate: bool,
    ocr_recovered: bool,
    text_profile: &TextProfile,
    layout: &LayoutSignals,
    detected_subjects: &[String],
    document_intelligence: &DocumentIntelligence,
) -> Vec<String> {
    let mut review_reasons = BTreeSet::new();
    if !exists {
        review_reasons.insert("missing_file".to_string());
    }
    if file_kind == "unknown" {
        review_reasons.insert("unknown_file_kind".to_string());
    }
    if ocr_candidate {
        review_reasons.insert("ocr_required".to_string());
    }
    if ocr_recovered && text_profile.recovery_confidence_score < 55 {
        review_reasons.insert("low_ocr_confidence".to_string());
    }
    if file_kind == "archive" {
        review_reasons.insert("archive_unpack_required".to_string());
    }
    if sample_text.is_some() && text_profile.non_empty_line_count == 0 {
        review_reasons.insert("empty_text_payload".to_string());
    }
    if question_like && sample_text.is_some() && layout.question_prompt_count == 0 {
        review_reasons.insert("weak_question_layout".to_string());
    }
    if answer_like && sample_text.is_some() && layout.answer_key_line_count == 0 {
        review_reasons.insert("weak_answer_layout".to_string());
    }
    if detected_subjects.is_empty()
        && matches!(
            document_role,
            "question_paper" | "mark_scheme" | "worksheet"
        )
    {
        review_reasons.insert("subject_not_detected".to_string());
    }
    if document_role == "unknown" {
        review_reasons.insert("ambiguous_document_role".to_string());
    }
    if document_role == "report_card" && document_intelligence.score_signals.is_empty() {
        review_reasons.insert("report_scores_not_detected".to_string());
    }
    if question_like
        && document_intelligence.question_blocks.is_empty()
        && layout.question_prompt_count > 0
    {
        review_reasons.insert("question_blocks_not_recovered".to_string());
    }

    review_reasons.into_iter().collect()
}

fn build_content_signals(
    document_role: &str,
    question_like: bool,
    answer_like: bool,
    layout: &LayoutSignals,
) -> Vec<String> {
    let mut signals = BTreeSet::new();
    signals.insert(document_role.to_string());
    if question_like {
        signals.insert("question_like".to_string());
    }
    if answer_like {
        signals.insert("answer_like".to_string());
    }
    if layout.choice_option_count >= 2 {
        signals.insert("multiple_choice".to_string());
    }
    if layout.formula_line_count > 0 {
        signals.insert("formulae".to_string());
    }
    if layout.diagram_signal_count > 0 {
        signals.insert("diagram_reference".to_string());
    }
    if layout.mark_allocation_count > 0 {
        signals.insert("mark_allocations".to_string());
    }
    if layout.table_like_line_count > 0 {
        signals.insert("tabular_content".to_string());
    }
    if layout.instruction_line_count > 0 {
        signals.insert("instructional_text".to_string());
    }
    signals.into_iter().collect()
}

fn mine_document_intelligence(
    file_name: &str,
    sample_text: Option<&str>,
    document_role: &str,
    layout: &LayoutSignals,
) -> DocumentIntelligence {
    let detected_dates = extract_date_hints(sample_text);
    let detected_topics = extract_topic_hints(sample_text);
    let question_blocks = extract_question_blocks(sample_text);
    let score_signals = extract_score_signals(sample_text);
    let remark_signals = extract_remark_lines(sample_text);
    let glossary_terms = extract_glossary_terms(sample_text, layout);
    let question_patterns = derive_question_patterns(document_role, layout, &question_blocks);
    let weakness_signals =
        derive_weakness_signals(&score_signals, &remark_signals, &detected_topics);
    let coach_actions = derive_coach_actions(
        document_role,
        &detect_document_origin(file_name, sample_text, document_role),
        &question_blocks,
        &score_signals,
        &remark_signals,
        &detected_topics,
        &glossary_terms,
    );
    let student_model_updates = derive_student_model_updates(
        document_role,
        &score_signals,
        &remark_signals,
        &detected_topics,
        &glossary_terms,
    );
    let document_origin = detect_document_origin(file_name, sample_text, document_role);

    DocumentIntelligence {
        document_origin,
        detected_dates,
        detected_topics,
        question_blocks,
        score_signals,
        remark_signals,
        glossary_terms,
        question_patterns,
        weakness_signals,
        coach_actions,
        student_model_updates,
    }
}

fn detect_document_origin(
    file_name: &str,
    sample_text: Option<&str>,
    document_role: &str,
) -> String {
    let lowered_name = file_name.to_ascii_lowercase();
    let lowered_text = sample_text.unwrap_or_default().to_ascii_lowercase();

    if contains_any(
        &lowered_name,
        &[
            "teacher",
            "handout",
            "revision guide",
            "lesson note",
            "class note",
            "booklet",
            "prep instruction",
            "topics list",
        ],
    ) || contains_any(
        &lowered_text,
        &[
            "teacher comment",
            "teacher note",
            "teacher instruction",
            "revision guide",
        ],
    ) {
        return "teacher_provided".to_string();
    }
    if document_role == "student_work"
        || contains_any(
            &lowered_name,
            &[
                "assignment",
                "homework",
                "draft",
                "worked solution",
                "my notes",
            ],
        )
    {
        return "student_generated".to_string();
    }
    if contains_any(
        &lowered_name,
        &[
            "waec",
            "bece",
            "neco",
            "report card",
            "terminal report",
            "official",
        ],
    ) || contains_any(
        &lowered_text,
        &[
            "west african examinations council",
            "official",
            "report card",
        ],
    ) {
        return "official".to_string();
    }
    if sample_text.is_some() {
        return "extracted".to_string();
    }
    "unknown".to_string()
}

fn extract_date_hints(sample_text: Option<&str>) -> Vec<String> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let month_names = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let mut dates = BTreeSet::new();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 80) else {
            continue;
        };
        let lowered = trimmed.to_ascii_lowercase();
        if month_names.iter().any(|month| lowered.contains(month))
            || trimmed.contains('/')
            || trimmed.contains('-')
        {
            if detect_exam_years(&trimmed).len() > 0
                || lowered.contains("date")
                || month_names.iter().any(|month| lowered.contains(month))
            {
                dates.insert(trimmed);
            }
        }
    }
    dates.into_iter().take(6).collect()
}

fn extract_topic_hints(sample_text: Option<&str>) -> Vec<String> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let mut topics = BTreeSet::new();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 120) else {
            continue;
        };
        let lowered = trimmed.to_ascii_lowercase();
        if let Some((_, value)) = trimmed.split_once(':') {
            if contains_any(
                &lowered,
                &[
                    "topic",
                    "topics",
                    "chapter",
                    "unit",
                    "theme",
                    "objective",
                    "focus",
                ],
            ) {
                for candidate in split_topic_candidates(value) {
                    topics.insert(candidate);
                }
                continue;
            }
        }
        for marker in [
            "weak in",
            "difficulty with",
            "confusion between",
            "focus on",
        ] {
            if let Some(index) = lowered.find(marker) {
                let value = &trimmed[(index + marker.len())..];
                for candidate in split_topic_candidates(value) {
                    topics.insert(candidate);
                }
            }
        }
        if is_heading_line(&trimmed)
            && trimmed.split_whitespace().count() <= 6
            && !contains_any(
                &lowered,
                &[
                    "mark scheme",
                    "report card",
                    "question paper",
                    "answer sheet",
                    "mathematics",
                    "science",
                    "english",
                ],
            )
        {
            topics.insert(trimmed.to_ascii_lowercase());
        }
    }
    topics.into_iter().take(12).collect()
}

fn split_topic_candidates(value: &str) -> Vec<String> {
    value
        .split([',', ';', '.'])
        .flat_map(|segment| segment.split(" and "))
        .filter_map(|segment| trim_display_line(segment, 60))
        .map(|segment| segment.trim_matches('.').to_ascii_lowercase())
        .filter(|segment| segment.split_whitespace().count() <= 6)
        .filter(|segment| segment.len() >= 3)
        .filter(|segment| {
            !contains_any(
                segment,
                &[
                    "student",
                    "teacher",
                    "remark",
                    "score",
                    "urgent intervention",
                    "needs improvement",
                ],
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extract_question_blocks(sample_text: Option<&str>) -> Vec<Value> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let mut blocks = Vec::new();
    let mut current_prompt: Option<String> = None;
    let mut current_options = Vec::new();
    let mut current_answer_hints = Vec::new();
    let mut current_marks: Option<String> = None;

    let flush_block = |blocks: &mut Vec<Value>,
                       prompt: &mut Option<String>,
                       options: &mut Vec<String>,
                       answer_hints: &mut Vec<String>,
                       marks: &mut Option<String>| {
        if let Some(prompt_value) = prompt.take() {
            blocks.push(json!({
                "prompt": prompt_value,
                "options": options.clone(),
                "answer_hints": answer_hints.clone(),
                "marks": marks.clone(),
            }));
        }
        options.clear();
        answer_hints.clear();
        *marks = None;
    };

    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 180) else {
            continue;
        };

        if starts_with_numbered_prompt(&trimmed) {
            flush_block(
                &mut blocks,
                &mut current_prompt,
                &mut current_options,
                &mut current_answer_hints,
                &mut current_marks,
            );
            current_prompt = Some(trimmed);
            continue;
        }

        if current_prompt.is_none() {
            continue;
        }

        if is_choice_line(&trimmed) {
            push_unique_limited(&mut current_options, trimmed, 6);
            continue;
        }
        if is_answer_key_line(&trimmed) {
            push_unique_limited(&mut current_answer_hints, trimmed, 4);
            continue;
        }
        if trimmed.to_ascii_lowercase().contains("mark") && current_marks.is_none() {
            current_marks = Some(trimmed);
            continue;
        }
        if !is_instruction_line(&trimmed) {
            if let Some(prompt) = current_prompt.as_mut() {
                if prompt.len() < 220 {
                    prompt.push(' ');
                    prompt.push_str(&trimmed);
                }
            }
        }
    }

    flush_block(
        &mut blocks,
        &mut current_prompt,
        &mut current_options,
        &mut current_answer_hints,
        &mut current_marks,
    );

    blocks.into_iter().take(8).collect()
}

fn extract_score_signals(sample_text: Option<&str>) -> Vec<Value> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let mut score_signals = Vec::new();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 120) else {
            continue;
        };
        if let Some(signal) = parse_score_signal(&trimmed) {
            score_signals.push(signal);
        }
    }
    score_signals
}

fn parse_score_signal(line: &str) -> Option<Value> {
    let lowered = line.to_ascii_lowercase();
    if !contains_any(
        &lowered,
        &[
            "score", "mark", "grade", "%", "/", "total", "average", "result",
        ],
    ) && detect_subject_hints(line).is_empty()
    {
        return None;
    }

    let fraction = extract_first_fraction(line);
    let percentage = extract_first_percentage(line).or_else(|| {
        fraction.map(|(earned, total)| {
            if total <= 0 {
                0
            } else {
                ((earned * 100) / total).clamp(0, 100)
            }
        })
    });
    let grade = extract_grade_hint(line);
    if fraction.is_none() && percentage.is_none() && grade.is_none() {
        return None;
    }

    let label = detect_subject_hints(line)
        .into_iter()
        .next()
        .unwrap_or_else(|| "overall".to_string());
    Some(json!({
        "label": label,
        "raw": line,
        "percentage_score": percentage,
        "points_earned": fraction.map(|value| value.0),
        "points_total": fraction.map(|value| value.1),
        "grade": grade,
    }))
}

fn extract_first_fraction(line: &str) -> Option<(i64, i64)> {
    for token in line.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '/');
        let Some((left, right)) = cleaned.split_once('/') else {
            continue;
        };
        let (Ok(earned), Ok(total)) = (left.parse::<i64>(), right.parse::<i64>()) else {
            continue;
        };
        if total > 0 && earned >= 0 && total <= 500 {
            return Some((earned, total));
        }
    }
    None
}

fn extract_first_percentage(line: &str) -> Option<i64> {
    for token in line.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != '%');
        if let Some(value) = cleaned.strip_suffix('%') {
            if let Ok(score) = value.parse::<i64>() {
                return Some(score.clamp(0, 100));
            }
        }
    }
    None
}

fn extract_grade_hint(line: &str) -> Option<String> {
    let lowered = line.to_ascii_lowercase();
    let Some(index) = lowered.find("grade") else {
        return None;
    };
    let tail = line[(index + "grade".len())..]
        .trim_start_matches([' ', ':', '-'])
        .split_whitespace()
        .next()?;
    Some(
        tail.trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
            .to_string(),
    )
}

fn extract_remark_lines(sample_text: Option<&str>) -> Vec<String> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let mut remarks = Vec::new();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 140) else {
            continue;
        };
        let lowered = trimmed.to_ascii_lowercase();
        if contains_any(
            &lowered,
            &[
                "remark",
                "comment",
                "needs improvement",
                "weak",
                "poor",
                "confusion",
                "excellent",
                "good effort",
                "urgent intervention",
            ],
        ) {
            push_unique_limited(&mut remarks, trimmed, 8);
        }
    }
    remarks
}

fn extract_glossary_terms(sample_text: Option<&str>, layout: &LayoutSignals) -> Vec<String> {
    let Some(text) = sample_text else {
        return Vec::new();
    };

    let mut terms = BTreeSet::new();
    for line in text.lines() {
        let Some(trimmed) = trim_display_line(line, 80) else {
            continue;
        };
        let lowered = trimmed.to_ascii_lowercase();
        if let Some((_, value)) = trimmed.split_once(':') {
            if contains_any(&lowered, &["definition", "term", "formula", "concept"]) {
                for candidate in split_topic_candidates(value) {
                    terms.insert(candidate);
                }
            }
        }
    }
    if layout.formula_line_count > 0 {
        for candidate in &layout.formula_candidates {
            terms.insert(candidate.to_ascii_lowercase());
        }
    }
    terms.into_iter().take(18).collect()
}

fn derive_question_patterns(
    document_role: &str,
    layout: &LayoutSignals,
    question_blocks: &[Value],
) -> Vec<String> {
    let mut patterns = BTreeSet::new();
    if layout.choice_option_count >= 2
        || question_blocks.iter().any(|block| {
            block
                .get("options")
                .and_then(Value::as_array)
                .map(|options| !options.is_empty())
                .unwrap_or(false)
        })
    {
        patterns.insert("multiple_choice".to_string());
    }
    if layout.answer_key_line_count > 0 || document_role == "mark_scheme" {
        patterns.insert("answer_key".to_string());
    }
    if layout.diagram_signal_count > 0 {
        patterns.insert("diagram_reference".to_string());
    }
    if question_blocks.iter().any(|block| {
        block
            .get("marks")
            .and_then(Value::as_str)
            .map(|value| !value.is_empty())
            .unwrap_or(false)
    }) {
        patterns.insert("structured_response".to_string());
    }
    patterns.into_iter().collect()
}

fn derive_weakness_signals(
    score_signals: &[Value],
    remark_signals: &[String],
    detected_topics: &[String],
) -> Vec<String> {
    let mut weaknesses = BTreeSet::new();
    for signal in score_signals {
        if let Some(score) = signal.get("percentage_score").and_then(Value::as_i64) {
            if score < 35 {
                weaknesses.insert("critical_score".to_string());
            } else if score < 50 {
                weaknesses.insert("low_score".to_string());
            }
        }
    }
    if remark_signals.iter().any(|remark| {
        let lowered = remark.to_ascii_lowercase();
        contains_any(
            &lowered,
            &[
                "weak",
                "needs improvement",
                "poor",
                "confusion",
                "urgent intervention",
            ],
        )
    }) {
        weaknesses.insert("teacher_concern".to_string());
    }
    if !detected_topics.is_empty() && !remark_signals.is_empty() {
        weaknesses.insert("topic_intervention_candidate".to_string());
    }
    weaknesses.into_iter().collect()
}

fn derive_coach_actions(
    document_role: &str,
    document_origin: &str,
    question_blocks: &[Value],
    score_signals: &[Value],
    remark_signals: &[String],
    detected_topics: &[String],
    glossary_terms: &[String],
) -> Vec<String> {
    let mut actions = BTreeSet::new();
    if !question_blocks.is_empty() {
        actions.insert("build_personalized_test".to_string());
    }
    if !glossary_terms.is_empty() {
        actions.insert("create_glossary_review".to_string());
    }
    if !detected_topics.is_empty() {
        actions.insert("attach_to_campaign".to_string());
    }
    if score_signals.iter().any(|signal| {
        signal
            .get("percentage_score")
            .and_then(Value::as_i64)
            .map(|score| score < 50)
            .unwrap_or(false)
    }) || !remark_signals.is_empty()
    {
        actions.insert("schedule_intervention".to_string());
    }
    if matches!(document_role, "report_card" | "corrected_script") {
        actions.insert("create_goal".to_string());
    }
    if matches!(document_role, "mark_scheme" | "corrected_script") {
        actions.insert("add_to_weakness_map".to_string());
    }
    if document_origin == "teacher_provided" {
        actions.insert("teacher_aligned_recommendation".to_string());
    }
    if document_role == "report_card" {
        actions.insert("notify_parent".to_string());
    }
    actions.into_iter().collect()
}

fn derive_student_model_updates(
    document_role: &str,
    score_signals: &[Value],
    remark_signals: &[String],
    detected_topics: &[String],
    glossary_terms: &[String],
) -> Vec<String> {
    let mut updates = BTreeSet::new();
    if !detected_topics.is_empty() {
        updates.insert("topic_coverage".to_string());
    }
    if !glossary_terms.is_empty() {
        updates.insert("resource_availability".to_string());
    }
    if !score_signals.is_empty() {
        updates.insert("assessment_evidence".to_string());
    }
    if !remark_signals.is_empty() {
        updates.insert("teacher_expectations".to_string());
    }
    if matches!(
        document_role,
        "report_card" | "corrected_script" | "mark_scheme"
    ) {
        updates.insert("known_weaknesses".to_string());
    }
    updates.into_iter().collect()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn push_unique_limited(items: &mut Vec<String>, value: String, max_len: usize) {
    if items.iter().any(|existing| existing == &value) || items.len() >= max_len {
        return;
    }
    items.push(value);
}

fn build_document_key(
    file_id: i64,
    file_name: &str,
    detected_subjects: &[String],
    detected_exam_years: &[i64],
) -> String {
    let mut tokens = BTreeSet::new();
    for subject in detected_subjects {
        tokens.insert(subject.clone());
    }
    for year in detected_exam_years {
        tokens.insert(year.to_string());
    }
    for token in tokenize_document_key(file_name) {
        tokens.insert(token);
    }

    if tokens.is_empty() {
        format!("file__{}", file_id)
    } else {
        tokens.into_iter().collect::<Vec<_>>().join("__")
    }
}

fn tokenize_document_key(file_name: &str) -> Vec<String> {
    let lowered_stem = normalized_stem(file_name);
    let mut tokens = BTreeSet::new();
    for marker in ["paper1", "paper2", "paper3", "paper4"] {
        if lowered_stem.contains(marker)
            || lowered_stem.contains(&marker.replace("paper", "paper "))
        {
            tokens.insert(marker.to_string());
        }
    }

    for token in lowered_stem
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        if matches!(
            token,
            "question"
                | "questions"
                | "paper"
                | "exam"
                | "mock"
                | "test"
                | "mark"
                | "marking"
                | "scheme"
                | "answer"
                | "answers"
                | "solution"
                | "solutions"
                | "file"
                | "upload"
                | "scan"
                | "image"
                | "copy"
        ) {
            continue;
        }
        if token.len() <= 1 {
            continue;
        }
        tokens.insert(token.to_string());
    }

    tokens.into_iter().collect()
}

fn normalized_stem(file_name: &str) -> String {
    Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(file_name)
        .to_ascii_lowercase()
}

fn unique_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_years(values: Vec<i64>) -> Vec<i64> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn trim_display_line(line: &str, max_len: usize) -> Option<String> {
    let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    let mut trimmed = collapsed;
    if trimmed.chars().count() > max_len {
        trimmed = trimmed.chars().take(max_len).collect();
    }
    Some(trimmed)
}

fn starts_with_numbered_prompt(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("question ") {
        return true;
    }
    if lowered.starts_with("q.") || lowered.starts_with("q ") {
        return true;
    }

    let digit_count = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digit_count == 0 {
        return false;
    }

    matches!(
        trimmed.chars().nth(digit_count),
        Some('.') | Some(')') | Some(':')
    )
}

fn is_choice_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let lowered = trimmed.to_ascii_lowercase();
    [
        "a)", "b)", "c)", "d)", "(a)", "(b)", "(c)", "(d)", "a.", "b.", "c.", "d.", "i)", "ii)",
        "iii)", "(i)", "(ii)", "(iii)",
    ]
    .iter()
    .any(|prefix| lowered.starts_with(prefix))
}

fn is_answer_key_line(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    if lowered.starts_with("answer:")
        || lowered.starts_with("ans:")
        || lowered.contains("correct answer")
        || lowered.contains("mark scheme")
    {
        return true;
    }

    let trimmed = line.trim_start();
    let digit_count = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digit_count == 0 {
        return false;
    }
    let Some(separator) = trimmed.chars().nth(digit_count) else {
        return false;
    };
    if !matches!(separator, '.' | ')' | ':' | '-') {
        return false;
    }
    let remainder = trimmed[(digit_count + separator.len_utf8())..].trim();
    let lowered_remainder = remainder.to_ascii_lowercase();
    matches!(
        lowered_remainder.as_str(),
        "a" | "b" | "c" | "d" | "true" | "false"
    ) || lowered_remainder.starts_with("option ")
        || lowered_remainder.starts_with("accept ")
        || lowered_remainder.starts_with("award ")
        || (remainder.len() <= 16 && lowered_remainder.starts_with("mark "))
}

fn is_instruction_line(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    [
        "answer",
        "choose",
        "write",
        "solve",
        "state",
        "explain",
        "calculate",
        "read",
        "attempt",
        "tick",
    ]
    .iter()
    .any(|prefix| lowered.starts_with(prefix))
}

fn is_heading_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || trimmed.to_ascii_lowercase().starts_with("section ") {
        return true;
    }

    let alpha_count = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count();
    alpha_count >= 5
        && trimmed
            .chars()
            .all(|ch| !ch.is_ascii_lowercase() || ch.is_whitespace())
        && trimmed.split_whitespace().count() <= 10
}

fn is_formula_line(line: &str) -> bool {
    let operator_count = line
        .chars()
        .filter(|ch| matches!(ch, '=' | '+' | '-' | '*' | '/' | '×' | '÷'))
        .count();
    operator_count >= 1 && line.chars().any(|ch| ch.is_ascii_alphabetic())
}

fn contains_diagram_signal(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    [
        "diagram",
        "figure",
        "graph",
        "table",
        "chart",
        "illustration",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn map_acquisition_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentAcquisitionJob> {
    let result_summary_json: String = row.get(7)?;
    let result_summary = serde_json::from_str::<Value>(&result_summary_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(ContentAcquisitionJob {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        topic_id: row.get(2)?,
        intent_type: row.get(3)?,
        query_text: row.get(4)?,
        source_scope: row.get(5)?,
        status: row.get(6)?,
        result_summary,
    })
}

fn map_acquisition_candidate(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AcquisitionEvidenceCandidate> {
    let payload_json: String = row.get(7)?;
    let extracted_payload = serde_json::from_str::<Value>(&payload_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(AcquisitionEvidenceCandidate {
        id: row.get(0)?,
        job_id: row.get(1)?,
        source_label: row.get(2)?,
        source_url: row.get(3)?,
        source_kind: row.get(4)?,
        title: row.get(5)?,
        snippet: row.get(6)?,
        extracted_payload,
        quality_score: row.get(8)?,
        freshness_score: row.get(9)?,
        review_status: row.get(10)?,
    })
}

fn infer_mime_type(file_name: &str) -> Option<&'static str> {
    match extension_of(file_name).as_deref() {
        Some("pdf") => Some("application/pdf"),
        Some("txt") => Some("text/plain"),
        Some("md") => Some("text/markdown"),
        Some("json") => Some("application/json"),
        Some("csv") => Some("text/csv"),
        Some("doc") => Some("application/msword"),
        Some("docx") => {
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        }
        Some("xls") => Some("application/vnd.ms-excel"),
        Some("xlsx") => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        Some("png") => Some("image/png"),
        Some("jpg" | "jpeg") => Some("image/jpeg"),
        Some("webp") => Some("image/webp"),
        _ => None,
    }
}

fn infer_file_kind(file_name: &str, mime_type: Option<&str>) -> String {
    if let Some(mime_type) = mime_type {
        if mime_type.starts_with("image/") {
            return "image".to_string();
        }
        if mime_type == "application/pdf" {
            return "pdf".to_string();
        }
        if matches!(
            mime_type,
            "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ) {
            return "document".to_string();
        }
        if matches!(
            mime_type,
            "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        ) {
            return "spreadsheet".to_string();
        }
        if mime_type.starts_with("text/") || mime_type == "application/json" {
            return "text".to_string();
        }
    }

    match extension_of(file_name).as_deref() {
        Some("pdf") => "pdf".to_string(),
        Some("doc" | "docx") => "document".to_string(),
        Some("xls" | "xlsx") => "spreadsheet".to_string(),
        Some("png" | "jpg" | "jpeg" | "webp") => "image".to_string(),
        Some("txt" | "md" | "json" | "csv") => "text".to_string(),
        Some("zip") => "archive".to_string(),
        _ => "unknown".to_string(),
    }
}

fn is_text_like(file_kind: &str) -> bool {
    matches!(file_kind, "text")
}

fn extension_of(file_name: &str) -> Option<String> {
    Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
}

fn detect_subject_hints(text: &str) -> Vec<String> {
    let lowered = text.to_ascii_lowercase();
    let mut subjects = BTreeSet::new();
    for (needle, label) in [
        ("mathematics", "mathematics"),
        ("math", "mathematics"),
        ("english", "english"),
        ("science", "science"),
        ("integrated science", "science"),
        ("social", "social_studies"),
        ("social studies", "social_studies"),
        ("biology", "biology"),
        ("chemistry", "chemistry"),
        ("physics", "physics"),
        ("economics", "economics"),
        ("geography", "geography"),
        ("history", "history"),
        ("literature", "literature"),
        ("government", "government"),
        ("civic", "civic_education"),
        ("ict", "ict"),
        ("computer", "ict"),
        ("agric", "agriculture"),
        ("agriculture", "agriculture"),
    ] {
        if lowered.contains(needle) {
            subjects.insert(label.to_string());
        }
    }
    subjects.into_iter().collect()
}

fn detect_exam_years(text: &str) -> Vec<i64> {
    let mut years = BTreeSet::new();
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if token.len() == 4 {
            if let Ok(year) = token.parse::<i64>() {
                if (1990..=2100).contains(&year) {
                    years.insert(year);
                }
            }
        }
    }
    years.into_iter().collect()
}

fn is_question_like(file_name: &str, text: Option<&str>) -> bool {
    let lowered_name = file_name.to_ascii_lowercase();
    if lowered_name.contains("question")
        || lowered_name.contains("mock")
        || lowered_name.contains("past")
        || lowered_name.contains("exam")
    {
        return true;
    }

    if let Some(text) = text {
        let question_mark_count = text.matches('?').count();
        let numbered_line_count = text
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("1.")
                    || trimmed.starts_with("2.")
                    || trimmed.starts_with("a)")
                    || trimmed.starts_with("b)")
            })
            .count();
        return question_mark_count >= 2 || numbered_line_count >= 2;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;
    use std::{
        env, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn reconstruct_bundle_groups_paired_assessment_documents_with_confidence() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);

        let temp_dir = test_temp_dir("intake_pairing");
        let question_path = temp_dir.join("Mathematics 2024 Paper 1 Questions.txt");
        let scheme_path = temp_dir.join("Mathematics 2024 Paper 1 Mark Scheme.txt");
        fs::write(
            &question_path,
            "MATHEMATICS PAPER 1\nAnswer all questions.\n1. What is 2 + 2?\na) 3\nb) 4\n2. Solve for x if x + 2 = 5.\n",
        )
        .expect("question file should write");
        fs::write(
            &scheme_path,
            "MARK SCHEME\n1. B\n2. x = 3\nAward 1 mark for each correct answer.\n",
        )
        .expect("scheme file should write");

        let service = IntakeService::new(&conn);
        let bundle_id = service
            .create_bundle(1, "Mock upload")
            .expect("bundle should create");
        service
            .add_bundle_file(
                bundle_id,
                file_name(&question_path),
                question_path.to_string_lossy().as_ref(),
            )
            .expect("question file should insert");
        service
            .add_bundle_file(
                bundle_id,
                file_name(&scheme_path),
                scheme_path.to_string_lossy().as_ref(),
            )
            .expect("scheme file should insert");

        let report = service
            .reconstruct_bundle(bundle_id)
            .expect("bundle should reconstruct");

        assert_eq!(report.bundle.status, "completed");
        assert_eq!(report.bundle_kind, "assessment_bundle");
        assert_eq!(report.reconstructed_document_count, 1);
        assert_eq!(report.paired_assessment_document_count, 1);
        assert!(report.reconstruction_confidence_score >= 75);
        assert_eq!(report.review_priority, "low");
        assert!(
            report
                .detected_subjects
                .contains(&"mathematics".to_string())
        );
        assert!(report.estimated_question_count >= 2);
        assert!(report.answer_like_file_count >= 1);

        let bundle_reconstruction = report
            .insights
            .iter()
            .find(|insight| insight.insight_type == "bundle_reconstruction")
            .expect("bundle reconstruction insight should exist");
        let groups = bundle_reconstruction
            .payload
            .get("document_groups")
            .and_then(Value::as_array)
            .expect("document groups should be present");
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].get("alignment_status").and_then(Value::as_str),
            Some("paired_question_and_mark_scheme")
        );
        assert!(
            groups[0]
                .get("confidence_score")
                .and_then(Value::as_i64)
                .expect("confidence score should exist")
                >= 75
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn reconstruct_bundle_marks_ocr_candidates_high_priority() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);

        let temp_dir = test_temp_dir("intake_ocr");
        let scan_path = temp_dir.join("Biology leaf diagram scan.png");
        fs::write(&scan_path, [137, 80, 78, 71]).expect("scan file should write");

        let service = IntakeService::new(&conn);
        let bundle_id = service
            .create_bundle(1, "Scan upload")
            .expect("bundle should create");
        service
            .add_bundle_file(
                bundle_id,
                file_name(&scan_path),
                scan_path.to_string_lossy().as_ref(),
            )
            .expect("scan file should insert");

        let report = service
            .reconstruct_bundle(bundle_id)
            .expect("bundle should reconstruct");

        assert_eq!(report.bundle.status, "review_required");
        assert_eq!(report.bundle_kind, "single_scan_bundle");
        assert_eq!(report.ocr_candidate_file_count, 1);
        assert_eq!(report.review_priority, "high");
        assert!(report.reconstruction_confidence_score <= 40);
        assert!(report.review_reasons.contains(&"ocr_required".to_string()));

        let file_reconstruction = report
            .insights
            .iter()
            .find(|insight| insight.insight_type == "file_reconstruction")
            .expect("file reconstruction insight should exist");
        assert_eq!(
            file_reconstruction
                .payload
                .pointer("/ocr_recovery/required")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            file_reconstruction
                .payload
                .pointer("/quality_signals/review_priority")
                .and_then(Value::as_str),
            Some("high")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn reconstruct_bundle_recovers_sidecar_ocr_and_mines_findings() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);

        let temp_dir = test_temp_dir("intake_ocr_sidecar");
        let scan_path = temp_dir.join("Science corrected script scan.png");
        let sidecar_path = temp_dir.join("Science corrected script scan.ocr.json");
        fs::write(&scan_path, [137, 80, 78, 71]).expect("scan file should write");
        fs::write(
            &sidecar_path,
            serde_json::to_string_pretty(&json!({
                "source": "ocr_sidecar_json",
                "pages": [{
                    "page_number": 1,
                    "confidence_score": 84,
                    "blocks": [
                        { "kind": "heading", "text": "INTEGRATED SCIENCE CORRECTED SCRIPT" },
                        { "kind": "topic", "text": "Topic: Osmosis and Diffusion" },
                        { "kind": "question", "text": "1. Define osmosis. (2 marks)" },
                        { "kind": "answer", "text": "1. award 1 mark for movement of water molecules" },
                        { "kind": "score", "text": "Score: 12/20" },
                        { "kind": "comment", "text": "Teacher comment: Weak in osmosis and diffusion definitions." }
                    ]
                }]
            }))
            .expect("sidecar payload should serialize"),
        )
        .expect("sidecar should write");

        let service = IntakeService::new(&conn);
        let bundle_id = service
            .create_bundle(1, "OCR sidecar upload")
            .expect("bundle should create");
        service
            .add_bundle_file(
                bundle_id,
                file_name(&scan_path),
                scan_path.to_string_lossy().as_ref(),
            )
            .expect("scan file should insert");

        let report = service
            .reconstruct_bundle(bundle_id)
            .expect("bundle should reconstruct");

        assert_eq!(report.bundle.status, "completed");
        assert_eq!(report.bundle_kind, "performance_evidence_bundle");
        assert_eq!(report.ocr_candidate_file_count, 0);
        assert_eq!(report.ocr_recovered_file_count, 1);
        assert!(!report.needs_confirmation);
        assert_eq!(report.unresolved_alignment_count, 0);
        assert!(report.layout_recovered_file_count >= 1);
        assert!(report.extracted_question_block_count >= 1);
        assert!(report.score_signal_count >= 1);
        assert!(report.remark_signal_count >= 1);
        assert!(
            report
                .detected_topics
                .iter()
                .any(|topic| topic == "osmosis" || topic == "diffusion")
        );
        assert!(
            report
                .recommended_actions
                .contains(&"schedule_intervention".to_string())
        );
        assert!(
            report
                .recommended_actions
                .contains(&"build_personalized_test".to_string())
        );

        let file_reconstruction = report
            .insights
            .iter()
            .find(|insight| insight.insight_type == "file_reconstruction")
            .expect("file reconstruction insight should exist");
        assert_eq!(
            file_reconstruction
                .payload
                .get("document_role")
                .and_then(Value::as_str),
            Some("corrected_script")
        );
        assert_eq!(
            file_reconstruction
                .payload
                .pointer("/text_recovery/source")
                .and_then(Value::as_str),
            Some("ocr_sidecar_json")
        );
        assert_eq!(
            file_reconstruction
                .payload
                .pointer("/ocr_recovery/status")
                .and_then(Value::as_str),
            Some("recovered")
        );
        assert_eq!(
            file_reconstruction
                .payload
                .pointer("/page_recovery/pages/0/label")
                .and_then(Value::as_str),
            Some("answer_page")
        );
        assert!(
            file_reconstruction
                .payload
                .pointer("/document_intelligence/question_blocks")
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
                .unwrap_or(false)
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn reconstruct_bundle_mines_report_card_actions_and_topics() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);

        let temp_dir = test_temp_dir("intake_report_card");
        let report_path = temp_dir.join("Term 2 Report Card.txt");
        fs::write(
            &report_path,
            "REPORT CARD\nDate: 12 March 2026\nMathematics Score: 42%\nEnglish Score: 71%\nTeacher Comment: Weak in algebra and comprehension. Needs urgent intervention.\n",
        )
        .expect("report card should write");

        let service = IntakeService::new(&conn);
        let bundle_id = service
            .create_bundle(1, "Report card upload")
            .expect("bundle should create");
        service
            .add_bundle_file(
                bundle_id,
                file_name(&report_path),
                report_path.to_string_lossy().as_ref(),
            )
            .expect("report card should insert");

        let report = service
            .reconstruct_bundle(bundle_id)
            .expect("bundle should reconstruct");

        assert_eq!(report.bundle.status, "completed");
        assert_eq!(report.bundle_kind, "performance_evidence_bundle");
        assert!(report.score_signal_count >= 2);
        assert!(report.remark_signal_count >= 1);
        assert!(!report.needs_confirmation);
        assert!(
            report
                .detected_dates
                .iter()
                .any(|date| date.contains("March 2026"))
        );
        assert!(report.detected_topics.contains(&"algebra".to_string()));
        assert!(
            report
                .detected_topics
                .contains(&"comprehension".to_string())
        );
        assert!(report.weakness_signals.contains(&"low_score".to_string()));
        assert!(
            report
                .recommended_actions
                .contains(&"create_goal".to_string())
        );
        assert!(
            report
                .recommended_actions
                .contains(&"notify_parent".to_string())
        );

        let file_reconstruction = report
            .insights
            .iter()
            .find(|insight| insight.insight_type == "file_reconstruction")
            .expect("file reconstruction insight should exist");
        assert_eq!(
            file_reconstruction
                .payload
                .get("document_role")
                .and_then(Value::as_str),
            Some("report_card")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ama', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (1, '[\"mathematics\"]', 60)",
            [],
        )
        .expect("student profile should insert");
    }

    fn test_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = env::temp_dir().join(format!("{}_{}_{}", prefix, process::id(), unique));
        fs::create_dir_all(&dir).expect("temp dir should create");
        dir
    }

    fn file_name(path: &Path) -> &str {
        path.file_name()
            .and_then(|value| value.to_str())
            .expect("file name should be unicode")
    }
}
