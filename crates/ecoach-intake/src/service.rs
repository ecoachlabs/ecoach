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
            let sample_text = read_text_sample(&path, &file.file_kind, exists);
            let analysis = analyze_bundle_file(file, inferred_mime, exists, sample_text.as_deref());
            self.insert_insight(bundle_id, "file_reconstruction", &analysis.payload)?;
            analyses.push(analysis);
        }

        let document_groups = reconstruct_document_groups(&analyses);
        let reconstructed_document_count = document_groups.len() as i64;
        let bundle_overview = summarize_bundle(&analyses, document_groups.len());
        let final_status = if !bundle_overview.review_reasons.is_empty() {
            "review_required"
        } else {
            "completed"
        };
        let bundle_reconstruction_payload = json!({
            "bundle_kind": &bundle_overview.bundle_kind,
            "document_groups": &document_groups,
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
            "question_like_file_count": bundle_overview.question_like_file_count,
            "answer_like_file_count": bundle_overview.answer_like_file_count,
            "ocr_candidate_file_count": bundle_overview.ocr_candidate_file_count,
            "layout_recovered_file_count": bundle_overview.layout_recovered_file_count,
            "estimated_question_count": bundle_overview.estimated_question_count,
            "estimated_answer_count": bundle_overview.estimated_answer_count,
            "reconstructed_document_count": reconstructed_document_count,
            "bundle_kind": &bundle_overview.bundle_kind,
            "detected_document_roles": &bundle_overview.document_roles,
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
        let mut question_like_file_count = 0i64;
        let mut answer_like_file_count = 0i64;
        let mut ocr_candidate_file_count = 0i64;
        let mut layout_recovered_file_count = 0i64;
        let mut estimated_question_count = 0i64;
        let mut estimated_answer_count = 0i64;
        let mut reconstructed_document_count = 0i64;
        let mut bundle_kind = "unknown".to_string();
        let mut detected_document_roles = BTreeSet::new();
        let mut review_reasons = BTreeSet::new();
        for insight in &insights {
            if insight.insight_type == "bundle_overview" {
                for subject in collect_string_values(&insight.payload, "detected_subjects") {
                    detected_subjects.insert(subject);
                }
                for year in collect_i64_values(&insight.payload, "detected_exam_years") {
                    detected_exam_years.insert(year);
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
                if let Some(kind) = insight.payload.get("bundle_kind").and_then(Value::as_str) {
                    bundle_kind = kind.to_string();
                }
                for role in collect_string_values(&insight.payload, "detected_document_roles") {
                    detected_document_roles.insert(role);
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
            question_like_file_count,
            answer_like_file_count,
            ocr_candidate_file_count,
            layout_recovered_file_count,
            estimated_question_count,
            estimated_answer_count,
            reconstructed_document_count,
            bundle_kind,
            detected_document_roles: detected_document_roles.into_iter().collect(),
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
    document_key: String,
    detected_subjects: Vec<String>,
    detected_exam_years: Vec<i64>,
    question_like: bool,
    answer_like: bool,
    ocr_candidate: bool,
    layout_recovered: bool,
    estimated_question_count: i64,
    estimated_answer_count: i64,
    layout_kind: String,
    layout_confidence_score: i64,
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
    preview_lines: Vec<String>,
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

struct BundleOverviewSummary {
    detected_subjects: Vec<String>,
    detected_exam_years: Vec<i64>,
    question_like_file_count: i64,
    answer_like_file_count: i64,
    ocr_candidate_file_count: i64,
    layout_recovered_file_count: i64,
    estimated_question_count: i64,
    estimated_answer_count: i64,
    bundle_kind: String,
    document_roles: Vec<String>,
    review_reasons: Vec<String>,
}

fn read_text_sample(path: &Path, file_kind: &str, exists: bool) -> Option<String> {
    if !exists || !is_text_like(file_kind) {
        return None;
    }

    fs::read_to_string(path).ok()
}

fn analyze_bundle_file(
    file: &BundleFile,
    inferred_mime: Option<&str>,
    exists: bool,
    sample_text: Option<&str>,
) -> FileReconstruction {
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

    let text_profile = build_text_profile(sample_text, &file.file_kind);
    let layout = build_layout_signals(sample_text);
    let document_role =
        detect_document_role(&file.file_name, sample_text, &file.file_kind, &layout);
    let question_like = matches!(
        document_role.as_str(),
        "question_paper" | "worksheet" | "mixed_assessment"
    ) || is_question_like(&file.file_name, sample_text)
        || layout.question_prompt_count > 0;
    let answer_like = matches!(
        document_role.as_str(),
        "mark_scheme" | "answer_sheet" | "mixed_assessment" | "student_work"
    ) || layout.answer_key_line_count > 0;
    let ocr_candidate =
        matches!(file.file_kind.as_str(), "image" | "pdf") && sample_text.is_none() && exists;
    let layout_recovered = sample_text.is_some() && layout.confidence_score >= 20;
    let estimated_question_count = if question_like {
        layout.question_prompt_count.max(1)
    } else {
        layout.question_prompt_count
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
        &text_profile,
        &layout,
        &detected_subjects,
    );
    let content_signals =
        build_content_signals(&document_role, question_like, answer_like, &layout);
    let document_key = build_document_key(
        file.id,
        &file.file_name,
        &detected_subjects,
        &detected_exam_years,
    );
    let ocr_strategy = if ocr_candidate {
        match file.file_kind.as_str() {
            "image" => "image_ocr",
            "pdf" => "pdf_ocr",
            _ => "ocr_recovery",
        }
    } else if sample_text.is_some() {
        "native_text"
    } else if file.file_kind == "archive" {
        "archive_unpack"
    } else {
        "manual_review"
    };
    let ocr_status = if ocr_candidate {
        "required"
    } else if sample_text.is_some() {
        "not_needed"
    } else if exists {
        "unavailable"
    } else {
        "missing_file"
    };
    let ocr_confidence_score = if sample_text.is_some() {
        95
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
            "preview_lines": &text_profile.preview_lines,
        },
        "ocr_recovery": {
            "required": ocr_candidate,
            "status": ocr_status,
            "strategy": ocr_strategy,
            "confidence_score": ocr_confidence_score,
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
        "review_reasons": &review_reasons,
    });

    FileReconstruction {
        file_id: file.id,
        file_name: file.file_name.clone(),
        document_role,
        document_key,
        detected_subjects,
        detected_exam_years,
        question_like,
        answer_like,
        ocr_candidate,
        layout_recovered,
        estimated_question_count,
        estimated_answer_count,
        layout_kind: layout.layout_kind,
        layout_confidence_score: layout.confidence_score,
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
    let mut document_roles = BTreeSet::new();
    let mut review_reasons = BTreeSet::new();
    let mut question_like_file_count = 0i64;
    let mut answer_like_file_count = 0i64;
    let mut ocr_candidate_file_count = 0i64;
    let mut layout_recovered_file_count = 0i64;
    let mut estimated_question_count = 0i64;
    let mut estimated_answer_count = 0i64;

    for analysis in analyses {
        for subject in &analysis.detected_subjects {
            detected_subjects.insert(subject.clone());
        }
        for year in &analysis.detected_exam_years {
            detected_exam_years.insert(*year);
        }
        document_roles.insert(analysis.document_role.clone());
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
        if analysis.layout_recovered {
            layout_recovered_file_count += 1;
        }
        estimated_question_count += analysis.estimated_question_count;
        estimated_answer_count += analysis.estimated_answer_count;
    }

    BundleOverviewSummary {
        detected_subjects: detected_subjects.into_iter().collect(),
        detected_exam_years: detected_exam_years.into_iter().collect(),
        question_like_file_count,
        answer_like_file_count,
        ocr_candidate_file_count,
        layout_recovered_file_count,
        estimated_question_count,
        estimated_answer_count,
        bundle_kind: classify_bundle_kind(analyses, reconstructed_document_count),
        document_roles: document_roles.into_iter().collect(),
        review_reasons: review_reasons.into_iter().collect(),
    }
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
            let mut review_reasons = BTreeSet::new();
            let mut estimated_question_count = 0i64;
            let mut estimated_answer_count = 0i64;
            let mut ocr_candidate_file_count = 0i64;

            for member in &members {
                roles.insert(member.document_role.clone());
                for subject in &member.detected_subjects {
                    subjects.insert(subject.clone());
                }
                for year in &member.detected_exam_years {
                    years.insert(*year);
                }
                for reason in &member.review_reasons {
                    review_reasons.insert(reason.clone());
                }
                estimated_question_count += member.estimated_question_count;
                estimated_answer_count += member.estimated_answer_count;
                if member.ocr_candidate {
                    ocr_candidate_file_count += 1;
                }
            }

            let member_files = members
                .iter()
                .map(|member| {
                    json!({
                        "file_id": member.file_id,
                        "file_name": member.file_name,
                        "document_role": member.document_role,
                        "layout_kind": member.layout_kind,
                        "layout_confidence_score": member.layout_confidence_score,
                        "ocr_candidate": member.ocr_candidate,
                    })
                })
                .collect::<Vec<_>>();
            let role_list = roles.into_iter().collect::<Vec<_>>();
            let reason_list = review_reasons.into_iter().collect::<Vec<_>>();
            json!({
                "document_key": document_key,
                "canonical_label": document_key.replace("__", " "),
                "document_kind": derive_document_group_kind(&role_list),
                "alignment_status": derive_group_alignment_status(&role_list, &reason_list),
                "member_count": members.len(),
                "member_files": member_files,
                "roles": role_list,
                "detected_subjects": subjects.into_iter().collect::<Vec<_>>(),
                "detected_exam_years": years.into_iter().collect::<Vec<_>>(),
                "estimated_question_count": estimated_question_count,
                "estimated_answer_count": estimated_answer_count,
                "ocr_candidate_file_count": ocr_candidate_file_count,
                "review_reasons": reason_list,
            })
        })
        .collect()
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
    if roles.contains("study_notes") || roles.contains("text_reference") {
        return "study_bundle".to_string();
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
    } else if role_set.contains("worksheet") {
        "worksheet_set"
    } else if role_set.contains("study_notes") {
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

fn build_text_profile(sample_text: Option<&str>, file_kind: &str) -> TextProfile {
    let Some(text) = sample_text else {
        let source = match file_kind {
            "image" | "pdf" => "no_embedded_text",
            "archive" => "archive_binary",
            "unknown" => "unsupported_binary",
            _ => "missing_text",
        };
        return TextProfile {
            source: source.to_string(),
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
        source: "native_text".to_string(),
        line_count,
        non_empty_line_count,
        character_count,
        word_count,
        preview_lines,
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
    if layout.question_prompt_count > 0 && layout.answer_key_line_count > 0 {
        return "mixed_assessment".to_string();
    }
    if has_answer_name_signal
        || lowered_text.contains("mark scheme")
        || lowered_text.contains("correct answer")
        || layout.answer_key_line_count >= 2
    {
        return "mark_scheme".to_string();
    }
    if has_worksheet_signal {
        return "worksheet".to_string();
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
    text_profile: &TextProfile,
    layout: &LayoutSignals,
    detected_subjects: &[String],
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
    remainder.len() <= 16
        && (matches!(
            lowered_remainder.as_str(),
            "a" | "b" | "c" | "d" | "true" | "false"
        ) || lowered_remainder.starts_with("option ")
            || lowered_remainder.starts_with("accept ")
            || lowered_remainder.starts_with("award "))
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
        if mime_type.starts_with("text/") || mime_type == "application/json" {
            return "text".to_string();
        }
    }

    match extension_of(file_name).as_deref() {
        Some("pdf") => "pdf".to_string(),
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
        ("social", "social_studies"),
        ("biology", "biology"),
        ("chemistry", "chemistry"),
        ("physics", "physics"),
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
