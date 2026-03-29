use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPublishJob {
    pub id: i64,
    pub source_upload_id: i64,
    pub content_pack_id: Option<i64>,
    pub requested_by_account_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub target_version_label: Option<String>,
    pub status: String,
    pub decision_trace: Value,
    pub artifact_summary: Value,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentQualityReport {
    pub id: i64,
    pub publish_job_id: i64,
    pub report_type: String,
    pub status: String,
    pub confidence_score: i64,
    pub metrics: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPublishJobReport {
    pub job: ContentPublishJob,
    pub quality_reports: Vec<ContentQualityReport>,
    pub blocking_report_count: i64,
    pub is_ready_to_publish: bool,
}

pub struct ContentPublishService<'a> {
    conn: &'a Connection,
}

impl<'a> ContentPublishService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_publish_job(
        &self,
        source_upload_id: i64,
        content_pack_id: Option<i64>,
        requested_by_account_id: Option<i64>,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        target_version_label: Option<&str>,
        decision_trace: &Value,
    ) -> EcoachResult<i64> {
        let decision_trace_json = serde_json::to_string(decision_trace)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO content_publish_jobs (
                    source_upload_id, content_pack_id, requested_by_account_id, subject_id,
                    topic_id, target_version_label, status, decision_trace_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'gating', ?7)",
                params![
                    source_upload_id,
                    content_pack_id,
                    requested_by_account_id,
                    subject_id,
                    topic_id,
                    target_version_label,
                    decision_trace_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_quality_report(
        &self,
        publish_job_id: i64,
        report_type: &str,
        status: &str,
        confidence_score: i64,
        metrics: &Value,
    ) -> EcoachResult<i64> {
        let metrics_json = serde_json::to_string(metrics)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO content_quality_reports (
                    publish_job_id, report_type, status, confidence_score, metrics_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    publish_job_id,
                    report_type,
                    status,
                    confidence_score.clamp(0, 10_000),
                    metrics_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if matches!(status, "fail" | "needs_review") {
            self.update_publish_job_status(publish_job_id, "review_required", None, None, false)?;
        }

        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_publish_job_status(
        &self,
        publish_job_id: i64,
        status: &str,
        decision_trace: Option<&Value>,
        artifact_summary: Option<&Value>,
        stamp_published_at: bool,
    ) -> EcoachResult<()> {
        let existing = self.get_publish_job(publish_job_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("publish job {} not found", publish_job_id))
        })?;
        let merged_decision_trace = decision_trace.unwrap_or(&existing.decision_trace);
        let merged_artifact_summary = artifact_summary.unwrap_or(&existing.artifact_summary);
        let decision_trace_json = serde_json::to_string(merged_decision_trace)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let artifact_summary_json = serde_json::to_string(merged_artifact_summary)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE content_publish_jobs
                 SET status = ?1,
                     decision_trace_json = ?2,
                     artifact_summary_json = ?3,
                     updated_at = datetime('now'),
                     published_at = CASE
                        WHEN ?4 = 1 THEN datetime('now')
                        ELSE published_at
                     END
                 WHERE id = ?5",
                params![
                    status,
                    decision_trace_json,
                    artifact_summary_json,
                    if stamp_published_at { 1 } else { 0 },
                    publish_job_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn mark_ready_to_publish(
        &self,
        publish_job_id: i64,
        artifact_summary: &Value,
    ) -> EcoachResult<()> {
        let report = self
            .get_publish_job_report(publish_job_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("publish job {} not found", publish_job_id))
            })?;
        if report.blocking_report_count > 0 {
            return Err(EcoachError::Validation(
                "cannot mark publish job ready while blocking quality reports remain".to_string(),
            ));
        }

        self.update_publish_job_status(
            publish_job_id,
            "ready_to_publish",
            None,
            Some(artifact_summary),
            false,
        )
    }

    pub fn mark_published(
        &self,
        publish_job_id: i64,
        artifact_summary: &Value,
    ) -> EcoachResult<()> {
        self.update_publish_job_status(
            publish_job_id,
            "published",
            None,
            Some(artifact_summary),
            true,
        )
    }

    pub fn list_publish_jobs(&self, status: Option<&str>) -> EcoachResult<Vec<ContentPublishJob>> {
        let sql = if status.is_some() {
            "SELECT id, source_upload_id, content_pack_id, requested_by_account_id, subject_id,
                    topic_id, target_version_label, status, decision_trace_json,
                    artifact_summary_json, published_at
             FROM content_publish_jobs
             WHERE status = ?1
             ORDER BY created_at DESC, id DESC"
        } else {
            "SELECT id, source_upload_id, content_pack_id, requested_by_account_id, subject_id,
                    topic_id, target_version_label, status, decision_trace_json,
                    artifact_summary_json, published_at
             FROM content_publish_jobs
             ORDER BY created_at DESC, id DESC"
        };

        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = if let Some(status) = status {
            statement.query_map([status], map_publish_job)
        } else {
            statement.query_map([], map_publish_job)
        }
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn get_publish_job(&self, publish_job_id: i64) -> EcoachResult<Option<ContentPublishJob>> {
        self.conn
            .query_row(
                "SELECT id, source_upload_id, content_pack_id, requested_by_account_id, subject_id,
                        topic_id, target_version_label, status, decision_trace_json,
                        artifact_summary_json, published_at
                 FROM content_publish_jobs
                 WHERE id = ?1",
                [publish_job_id],
                map_publish_job,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_publish_job_report(
        &self,
        publish_job_id: i64,
    ) -> EcoachResult<Option<ContentPublishJobReport>> {
        let job = self.get_publish_job(publish_job_id)?;
        let Some(job) = job else {
            return Ok(None);
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, publish_job_id, report_type, status, confidence_score, metrics_json, created_at
                 FROM content_quality_reports
                 WHERE publish_job_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([publish_job_id], map_quality_report)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut quality_reports = Vec::new();
        for row in rows {
            quality_reports.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let blocking_report_count = quality_reports
            .iter()
            .filter(|report| matches!(report.status.as_str(), "fail" | "needs_review"))
            .count() as i64;

        Ok(Some(ContentPublishJobReport {
            is_ready_to_publish: blocking_report_count == 0
                && matches!(
                    job.status.as_str(),
                    "gating" | "ready_to_publish" | "publishing" | "published"
                ),
            job,
            quality_reports,
            blocking_report_count,
        }))
    }
}

fn map_publish_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentPublishJob> {
    let decision_trace_json: String = row.get(8)?;
    let artifact_summary_json: String = row.get(9)?;
    Ok(ContentPublishJob {
        id: row.get(0)?,
        source_upload_id: row.get(1)?,
        content_pack_id: row.get(2)?,
        requested_by_account_id: row.get(3)?,
        subject_id: row.get(4)?,
        topic_id: row.get(5)?,
        target_version_label: row.get(6)?,
        status: row.get(7)?,
        decision_trace: parse_json_column(8, &decision_trace_json)?,
        artifact_summary: parse_json_column(9, &artifact_summary_json)?,
        published_at: row.get(10)?,
    })
}

fn map_quality_report(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentQualityReport> {
    let metrics_json: String = row.get(5)?;
    Ok(ContentQualityReport {
        id: row.get(0)?,
        publish_job_id: row.get(1)?,
        report_type: row.get(2)?,
        status: row.get(3)?,
        confidence_score: row.get(4)?,
        metrics: parse_json_column(5, &metrics_json)?,
        created_at: row.get(6)?,
    })
}

fn parse_json_column(column_index: usize, raw: &str) -> rusqlite::Result<Value> {
    serde_json::from_str::<Value>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;

    use ecoach_storage::run_runtime_migrations;

    use super::*;

    #[test]
    fn publish_job_enforces_quality_gate_before_ready() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_admin_and_source(&conn);

        let service = ContentPublishService::new(&conn);
        let publish_job_id = service
            .create_publish_job(
                1,
                None,
                Some(1),
                None,
                None,
                Some("v2"),
                &json!({ "source_status": "parsed" }),
            )
            .expect("publish job should insert");
        service
            .add_quality_report(
                publish_job_id,
                "coverage_gate",
                "needs_review",
                7200,
                &json!({ "missing_topics": 2 }),
            )
            .expect("quality report should insert");

        let report = service
            .get_publish_job_report(publish_job_id)
            .expect("publish report should load")
            .expect("publish report should exist");

        assert_eq!(report.job.status, "review_required");
        assert_eq!(report.blocking_report_count, 1);
        assert!(!report.is_ready_to_publish);
        assert!(
            service
                .mark_ready_to_publish(publish_job_id, &json!({ "pack_version": "v2" }))
                .is_err()
        );
    }

    #[test]
    fn publish_job_can_advance_after_passing_quality_reports() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_admin_and_source(&conn);

        let service = ContentPublishService::new(&conn);
        let publish_job_id = service
            .create_publish_job(
                1,
                None,
                Some(1),
                None,
                None,
                Some("v3"),
                &json!({ "source_status": "parsed" }),
            )
            .expect("publish job should insert");
        service
            .add_quality_report(
                publish_job_id,
                "coverage_gate",
                "pass",
                8800,
                &json!({ "coverage_ratio": 0.96 }),
            )
            .expect("coverage report should insert");
        service
            .add_quality_report(
                publish_job_id,
                "trust_gate",
                "warning",
                7600,
                &json!({ "review_notes": 1 }),
            )
            .expect("trust report should insert");
        service
            .mark_ready_to_publish(publish_job_id, &json!({ "artifact_count": 42 }))
            .expect("job should become ready");
        service
            .mark_published(
                publish_job_id,
                &json!({ "artifact_count": 42, "pack_version": "v3" }),
            )
            .expect("job should publish");

        let report = service
            .get_publish_job_report(publish_job_id)
            .expect("publish report should load")
            .expect("publish report should exist");

        assert_eq!(report.job.status, "published");
        assert!(report.job.published_at.is_some());
        assert_eq!(report.blocking_report_count, 0);
        assert!(report.is_ready_to_publish);
    }

    fn seed_admin_and_source(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'admin', 'Admin', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("admin should insert");
        conn.execute(
            "INSERT INTO curriculum_source_uploads (
                id, uploader_account_id, source_kind, title, source_path, country_code,
                exam_board, education_level, subject_code, academic_year, language_code,
                version_label, source_status, confidence_score, metadata_json
             ) VALUES (
                1, 1, 'curriculum', 'Math Curriculum', 'C:/curriculum/math.pdf', 'GH',
                'WAEC', 'JHS', 'MATH', '2026', 'en', 'v1', 'parsed', 8200, '{}'
             )",
            [],
        )
        .expect("source upload should insert");
    }
}
