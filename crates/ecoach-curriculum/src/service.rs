use std::str::FromStr;

use chrono::{DateTime, Utc};
use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::models::{
    AcademicNode, CurriculumParseCandidate, CurriculumReviewTask, CurriculumSourceReport,
    CurriculumSourceUpload, Subject, TopicSummary,
};

pub struct CurriculumService<'a> {
    conn: &'a Connection,
}

impl<'a> CurriculumService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_subjects(&self, curriculum_version_id: i64) -> EcoachResult<Vec<Subject>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, code, name, display_order
                 FROM subjects
                 WHERE curriculum_version_id = ?1 AND is_active = 1
                 ORDER BY display_order ASC, name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([curriculum_version_id], |row| {
                Ok(Subject {
                    id: row.get(0)?,
                    curriculum_version_id: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    display_order: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut subjects = Vec::new();
        for row in rows {
            subjects.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(subjects)
    }

    pub fn list_topics_for_subject(&self, subject_id: i64) -> EcoachResult<Vec<TopicSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, subject_id, parent_topic_id, code, name, node_type, display_order
                 FROM topics
                 WHERE subject_id = ?1 AND is_active = 1
                 ORDER BY display_order ASC, name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([subject_id], |row| {
                Ok(TopicSummary {
                    id: row.get(0)?,
                    subject_id: row.get(1)?,
                    parent_topic_id: row.get(2)?,
                    code: row.get(3)?,
                    name: row.get(4)?,
                    node_type: row.get(5)?,
                    display_order: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(topics)
    }

    pub fn search_topics(
        &self,
        query: &str,
        subject_id: Option<i64>,
    ) -> EcoachResult<Vec<TopicSummary>> {
        let like_query = format!("%{}%", query);
        let mut topics = Vec::new();

        if let Some(subject_id) = subject_id {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, subject_id, parent_topic_id, code, name, node_type, display_order
                     FROM topics
                     WHERE is_active = 1 AND subject_id = ?1 AND name LIKE ?2
                     ORDER BY display_order ASC, name ASC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let rows = statement
                .query_map(params![subject_id, like_query], map_topic)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            for row in rows {
                topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        } else {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, subject_id, parent_topic_id, code, name, node_type, display_order
                     FROM topics
                     WHERE is_active = 1 AND name LIKE ?1
                     ORDER BY display_order ASC, name ASC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let rows = statement
                .query_map([like_query], map_topic)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            for row in rows {
                topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }

        Ok(topics)
    }

    pub fn get_academic_nodes_for_topic(&self, topic_id: i64) -> EcoachResult<Vec<AcademicNode>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, topic_id, node_type, canonical_title, core_meaning, exam_relevance_score, created_at
                 FROM academic_nodes
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY canonical_title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([topic_id], |row| {
                Ok(AcademicNode {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    node_type: row.get(2)?,
                    canonical_title: row.get(3)?,
                    core_meaning: row.get(4)?,
                    exam_relevance_score: row.get(5)?,
                    created_at: parse_datetime(row.get::<_, String>(6)?)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(nodes)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_source_upload(
        &self,
        uploader_account_id: i64,
        source_kind: &str,
        title: &str,
        source_path: Option<&str>,
        country_code: Option<&str>,
        exam_board: Option<&str>,
        education_level: Option<&str>,
        subject_code: Option<&str>,
        academic_year: Option<&str>,
        language_code: Option<&str>,
        version_label: Option<&str>,
        metadata: &Value,
    ) -> EcoachResult<i64> {
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO curriculum_source_uploads (
                    uploader_account_id, source_kind, title, source_path, country_code,
                    exam_board, education_level, subject_code, academic_year, language_code,
                    version_label, metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    uploader_account_id,
                    source_kind,
                    title,
                    source_path,
                    country_code,
                    exam_board,
                    education_level,
                    subject_code,
                    academic_year,
                    language_code.unwrap_or("en"),
                    version_label,
                    metadata_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_parse_candidate(
        &self,
        source_upload_id: i64,
        candidate_type: &str,
        parent_candidate_id: Option<i64>,
        raw_label: &str,
        normalized_label: Option<&str>,
        payload: &Value,
        confidence_score: i64,
    ) -> EcoachResult<i64> {
        let payload_json = serde_json::to_string(payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO curriculum_parse_candidates (
                    source_upload_id, candidate_type, parent_candidate_id, raw_label,
                    normalized_label, payload_json, confidence_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    source_upload_id,
                    candidate_type,
                    parent_candidate_id,
                    raw_label,
                    normalized_label,
                    payload_json,
                    confidence_score.clamp(0, 10_000),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_source_uploads
                 SET source_status = 'parsed',
                     confidence_score = MAX(confidence_score, ?1),
                     updated_at = datetime('now')
                 WHERE id = ?2",
                params![confidence_score.clamp(0, 10_000), source_upload_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_review_task(
        &self,
        source_upload_id: i64,
        candidate_id: Option<i64>,
        task_type: &str,
        severity: &str,
        notes: Option<&str>,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO curriculum_review_tasks (
                    source_upload_id, candidate_id, task_type, severity, notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![source_upload_id, candidate_id, task_type, severity, notes],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_source_uploads
                 SET source_status = 'review_required', updated_at = datetime('now')
                 WHERE id = ?1",
                [source_upload_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_source_status(
        &self,
        source_upload_id: i64,
        source_status: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE curriculum_source_uploads
                 SET source_status = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![source_status, source_upload_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn list_source_uploads(
        &self,
        source_status: Option<&str>,
    ) -> EcoachResult<Vec<CurriculumSourceUpload>> {
        let mut uploads = Vec::new();
        if let Some(source_status) = source_status {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                            exam_board, education_level, subject_code, academic_year,
                            language_code, version_label, source_status, confidence_score, metadata_json
                     FROM curriculum_source_uploads
                     WHERE source_status = ?1
                     ORDER BY created_at DESC, id DESC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([source_status], map_source_upload)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                uploads.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        } else {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                            exam_board, education_level, subject_code, academic_year,
                            language_code, version_label, source_status, confidence_score, metadata_json
                     FROM curriculum_source_uploads
                     ORDER BY created_at DESC, id DESC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([], map_source_upload)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                uploads.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }
        Ok(uploads)
    }

    pub fn get_source_report(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Option<CurriculumSourceReport>> {
        let source_upload = self
            .conn
            .query_row(
                "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                        exam_board, education_level, subject_code, academic_year,
                        language_code, version_label, source_status, confidence_score, metadata_json
                 FROM curriculum_source_uploads
                 WHERE id = ?1",
                [source_upload_id],
                map_source_upload,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some(source_upload) = source_upload else {
            return Ok(None);
        };

        let mut candidates_statement = self
            .conn
            .prepare(
                "SELECT id, source_upload_id, candidate_type, parent_candidate_id, raw_label,
                        normalized_label, payload_json, confidence_score, review_status
                 FROM curriculum_parse_candidates
                 WHERE source_upload_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let candidate_rows = candidates_statement
            .query_map([source_upload_id], map_parse_candidate)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut candidates = Vec::new();
        for row in candidate_rows {
            candidates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut review_statement = self
            .conn
            .prepare(
                "SELECT id, source_upload_id, candidate_id, task_type, status, severity, notes
                 FROM curriculum_review_tasks
                 WHERE source_upload_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let review_rows = review_statement
            .query_map([source_upload_id], map_review_task)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut review_tasks = Vec::new();
        for row in review_rows {
            review_tasks.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(Some(CurriculumSourceReport {
            source_upload,
            candidates,
            review_tasks,
        }))
    }
}

fn map_topic(row: &rusqlite::Row<'_>) -> rusqlite::Result<TopicSummary> {
    Ok(TopicSummary {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        parent_topic_id: row.get(2)?,
        code: row.get(3)?,
        name: row.get(4)?,
        node_type: row.get(5)?,
        display_order: row.get(6)?,
    })
}

fn parse_datetime(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::<Utc>::from_str(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(EcoachError::Serialization(err.to_string())),
            )
        })
}

fn map_source_upload(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumSourceUpload> {
    let metadata_json: String = row.get(14)?;
    let metadata = serde_json::from_str::<Value>(&metadata_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            14,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })?;
    Ok(CurriculumSourceUpload {
        id: row.get(0)?,
        uploader_account_id: row.get(1)?,
        source_kind: row.get(2)?,
        title: row.get(3)?,
        source_path: row.get(4)?,
        country_code: row.get(5)?,
        exam_board: row.get(6)?,
        education_level: row.get(7)?,
        subject_code: row.get(8)?,
        academic_year: row.get(9)?,
        language_code: row.get(10)?,
        version_label: row.get(11)?,
        source_status: row.get(12)?,
        confidence_score: row.get(13)?,
        metadata,
    })
}

fn map_parse_candidate(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumParseCandidate> {
    let payload_json: String = row.get(6)?;
    let payload = serde_json::from_str::<Value>(&payload_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            6,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })?;
    Ok(CurriculumParseCandidate {
        id: row.get(0)?,
        source_upload_id: row.get(1)?,
        candidate_type: row.get(2)?,
        parent_candidate_id: row.get(3)?,
        raw_label: row.get(4)?,
        normalized_label: row.get(5)?,
        payload,
        confidence_score: row.get(7)?,
        review_status: row.get(8)?,
    })
}

fn map_review_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumReviewTask> {
    Ok(CurriculumReviewTask {
        id: row.get(0)?,
        source_upload_id: row.get(1)?,
        candidate_id: row.get(2)?,
        task_type: row.get(3)?,
        status: row.get(4)?,
        severity: row.get(5)?,
        notes: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;

    use ecoach_storage::run_runtime_migrations;

    use super::*;

    #[test]
    fn curriculum_source_pipeline_creates_report() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_admin(&conn);

        let service = CurriculumService::new(&conn);
        let source_upload_id = service
            .create_source_upload(
                1,
                "curriculum",
                "JHS Mathematics Curriculum",
                Some("C:/curriculum/math.pdf"),
                Some("GH"),
                Some("WAEC"),
                Some("JHS"),
                Some("MATH"),
                Some("2026"),
                Some("en"),
                Some("v1"),
                &json!({ "source_class": "official" }),
            )
            .expect("source upload should insert");
        let candidate_id = service
            .add_parse_candidate(
                source_upload_id,
                "topic",
                None,
                "Algebraic Expressions",
                Some("algebraic_expressions"),
                &json!({ "term": 2 }),
                7800,
            )
            .expect("parse candidate should insert");
        service
            .create_review_task(
                source_upload_id,
                Some(candidate_id),
                "publish_gate",
                "medium",
                Some("Confirm topic ordering."),
            )
            .expect("review task should insert");

        let report = service
            .get_source_report(source_upload_id)
            .expect("source report should query")
            .expect("source report should exist");

        assert_eq!(report.source_upload.source_status, "review_required");
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.review_tasks.len(), 1);
        assert_eq!(report.candidates[0].candidate_type, "topic");
    }

    fn seed_admin(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'admin', 'Admin', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("admin should insert");
    }
}
