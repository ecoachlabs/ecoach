use std::{
    collections::BTreeSet,
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

        let mut detected_subjects = BTreeSet::new();
        let mut detected_exam_years = BTreeSet::new();
        let mut question_like_file_count = 0i64;
        let mut requires_review = false;

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
            let sample_text = if exists && is_text_like(&file.file_kind) {
                fs::read_to_string(&path).ok()
            } else {
                None
            };

            let mut years = detect_exam_years(&file.file_name);
            if let Some(text) = &sample_text {
                years.extend(detect_exam_years(text));
            }
            for year in years {
                detected_exam_years.insert(year);
            }

            let mut subjects = detect_subject_hints(&file.file_name);
            if let Some(text) = &sample_text {
                subjects.extend(detect_subject_hints(text));
            }
            for subject in subjects {
                detected_subjects.insert(subject);
            }

            let question_like = is_question_like(&file.file_name, sample_text.as_deref());
            if question_like {
                question_like_file_count += 1;
            }

            if !exists || file.file_kind == "unknown" {
                requires_review = true;
            }

            let payload = json!({
                "file_id": file.id,
                "file_name": file.file_name,
                "file_path": file.file_path,
                "file_kind": file.file_kind,
                "mime_type": inferred_mime,
                "exists": exists,
                "question_like": question_like,
                "detected_subjects": detect_subject_hints(&file.file_name),
                "detected_exam_years": detect_exam_years(&file.file_name),
                "line_count": sample_text.as_ref().map(|text| text.lines().count()).unwrap_or(0),
                "character_count": sample_text.as_ref().map(|text| text.chars().count()).unwrap_or(0),
            });
            self.insert_insight(bundle_id, "file_reconstruction", &payload)?;
        }

        let final_status = if requires_review {
            "review_required"
        } else {
            "completed"
        };
        let overview_payload = json!({
            "file_count": files.len(),
            "question_like_file_count": question_like_file_count,
            "detected_subjects": detected_subjects,
            "detected_exam_years": detected_exam_years,
            "requires_review": requires_review,
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
        for insight in &insights {
            if insight.insight_type == "bundle_overview" {
                if let Some(subjects) = insight
                    .payload
                    .get("detected_subjects")
                    .and_then(Value::as_array)
                {
                    for subject in subjects.iter().filter_map(Value::as_str) {
                        detected_subjects.insert(subject.to_string());
                    }
                }
                if let Some(years) = insight
                    .payload
                    .get("detected_exam_years")
                    .and_then(Value::as_array)
                {
                    for year in years.iter().filter_map(Value::as_i64) {
                        detected_exam_years.insert(year);
                    }
                }
                if let Some(count) = insight
                    .payload
                    .get("question_like_file_count")
                    .and_then(Value::as_i64)
                {
                    question_like_file_count = count;
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
