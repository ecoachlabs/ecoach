use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

mod smart_central;

use crate::models::{
    AcademicNode, CurriculumAlias, CurriculumAliasInput, CurriculumAssessmentPattern,
    CurriculumConceptAtom, CurriculumConceptAtomInput, CurriculumCoverageStats,
    CurriculumFamily, CurriculumFamilyInput, CurriculumLevel, CurriculumLevelInput,
    CurriculumLinkedResource, CurriculumNode, CurriculumNodeBundle, CurriculumNodeBundleInput,
    CurriculumNodeInput, CurriculumObjective, CurriculumObjectiveInput, CurriculumParseCandidate,
    CurriculumPrerequisiteStep, CurriculumPublicSnapshot, CurriculumPublicSubjectOverview,
    CurriculumPublicTopicDetail, CurriculumPublishResult, CurriculumRecommendation,
    CurriculumRelationship, CurriculumRelationshipInput, CurriculumRemediationMap,
    CurriculumRemediationStep, CurriculumResourceLinkInput, CurriculumReviewQueueItem,
    CurriculumReviewTask, CurriculumSearchResult, CurriculumSourceReport, CurriculumSourceUpload,
    CurriculumSubjectTrack, CurriculumSubjectTrackInput, CurriculumTermPeriod,
    CurriculumTermPeriodInput, CurriculumTopicContext, CurriculumTreeNode, CurriculumVersion,
    CurriculumVersionDiffEntry, CurriculumVersionDiffReport, CurriculumVersionInput, Subject,
    TopicSummary,
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

        collect_rows(rows)
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
            .query_map([subject_id], map_topic)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        collect_rows(rows)
    }

    pub fn search_topics(
        &self,
        query: &str,
        subject_id: Option<i64>,
    ) -> EcoachResult<Vec<TopicSummary>> {
        let like_query = format!("%{}%", query);
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

            collect_rows(rows)
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

            collect_rows(rows)
        }
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

        collect_rows(rows)
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
                    confidence_score,
                ],
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
                    source_upload_id, candidate_id, task_type, status, severity, notes
                 ) VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
                params![source_upload_id, candidate_id, task_type, severity, notes],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.update_source_status(source_upload_id, "review_required")?;
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
        let sql = if source_status.is_some() {
            "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                    exam_board, education_level, subject_code, academic_year,
                    language_code, version_label, source_status, confidence_score, metadata_json
             FROM curriculum_source_uploads
             WHERE source_status = ?1
             ORDER BY created_at DESC, id DESC"
        } else {
            "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                    exam_board, education_level, subject_code, academic_year,
                    language_code, version_label, source_status, confidence_score, metadata_json
             FROM curriculum_source_uploads
             ORDER BY created_at DESC, id DESC"
        };

        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = if let Some(status) = source_status {
            statement.query_map([status], map_source_upload)
        } else {
            statement.query_map([], map_source_upload)
        }
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

        collect_rows(rows)
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

        let mut candidate_statement = self
            .conn
            .prepare(
                "SELECT id, source_upload_id, candidate_type, parent_candidate_id, raw_label,
                        normalized_label, payload_json, confidence_score, review_status
                 FROM curriculum_parse_candidates
                 WHERE source_upload_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let candidate_rows = candidate_statement
            .query_map([source_upload_id], map_parse_candidate)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let candidates = collect_rows(candidate_rows)?;

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
        let review_tasks = collect_rows(review_rows)?;

        Ok(Some(CurriculumSourceReport {
            source_upload,
            candidates,
            review_tasks,
        }))
    }

    pub fn list_curriculum_families(&self) -> EcoachResult<Vec<CurriculumFamily>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, slug, name, country_code, exam_board, education_stage, description, is_public
                 FROM curriculum_families
                 ORDER BY is_public DESC, name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], map_curriculum_family)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn save_curriculum_family(
        &self,
        input: CurriculumFamilyInput,
    ) -> EcoachResult<CurriculumFamily> {
        let slug = input.slug.unwrap_or_else(|| slugify(&input.name));
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_families
                     SET slug = ?1,
                         name = ?2,
                         country_code = ?3,
                         exam_board = ?4,
                         education_stage = ?5,
                         description = ?6,
                         is_public = ?7,
                         updated_at = datetime('now')
                     WHERE id = ?8",
                    params![
                        slug,
                        input.name,
                        input.country_code.unwrap_or_else(|| "GH".to_string()),
                        input.exam_board,
                        input.education_stage,
                        input.description,
                        if input.is_public { 1 } else { 0 },
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_family(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum family {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_families (
                        slug, name, country_code, exam_board, education_stage, description, is_public
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        slug,
                        input.name,
                        input.country_code.unwrap_or_else(|| "GH".to_string()),
                        input.exam_board,
                        input.education_stage,
                        input.description,
                        if input.is_public { 1 } else { 0 },
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_family(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("curriculum family insert did not persist".to_string()))
        }
    }

    pub fn save_curriculum_version(
        &self,
        input: CurriculumVersionInput,
    ) -> EcoachResult<CurriculumVersion> {
        let source_summary_json = serde_json::to_string(&input.source_summary)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let status = input.status.unwrap_or_else(|| "draft".to_string());
        if let Some(id) = input.id {
            self.ensure_version_mutable(id)?;
            self.conn
                .execute(
                    "UPDATE curriculum_versions
                     SET curriculum_family_id = ?1,
                         name = ?2,
                         country = ?3,
                         exam_board = ?4,
                         education_stage = ?5,
                         version_label = ?6,
                         status = ?7,
                         effective_from = ?8,
                         effective_to = ?9,
                         source_summary_json = ?10,
                         replaced_by_version_id = ?11,
                         updated_at = datetime('now')
                     WHERE id = ?12",
                    params![
                        input.curriculum_family_id,
                        input.name,
                        input.country.unwrap_or_else(|| "GH".to_string()),
                        input.exam_board,
                        input.education_stage,
                        input.version_label,
                        status,
                        input.effective_from,
                        input.effective_to,
                        source_summary_json,
                        input.replaced_by_version_id,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_version(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum version {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_versions (
                        curriculum_family_id, name, country, exam_board, education_stage,
                        version_label, status, effective_from, effective_to, source_summary_json,
                        replaced_by_version_id
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        input.curriculum_family_id,
                        input.name,
                        input.country.unwrap_or_else(|| "GH".to_string()),
                        input.exam_board,
                        input.education_stage,
                        input.version_label,
                        status,
                        input.effective_from,
                        input.effective_to,
                        source_summary_json,
                        input.replaced_by_version_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_version(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("curriculum version insert did not persist".to_string()))
        }
    }

    pub fn get_curriculum_family(&self, id: i64) -> EcoachResult<Option<CurriculumFamily>> {
        self.conn
            .query_row(
                "SELECT id, slug, name, country_code, exam_board, education_stage, description, is_public
                 FROM curriculum_families
                 WHERE id = ?1",
                [id],
                map_curriculum_family,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_curriculum_version(&self, id: i64) -> EcoachResult<Option<CurriculumVersion>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_family_id, name, country, exam_board, education_stage,
                        version_label, status, effective_from, effective_to,
                        source_summary_json, published_at, replaced_by_version_id
                 FROM curriculum_versions
                 WHERE id = ?1",
                [id],
                map_curriculum_version,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn save_curriculum_subject_track(
        &self,
        input: CurriculumSubjectTrackInput,
    ) -> EcoachResult<CurriculumSubjectTrack> {
        self.ensure_version_mutable(input.curriculum_version_id)?;
        let subject_slug = input
            .subject_slug
            .unwrap_or_else(|| slugify(&input.subject_name));
        let public_title = input
            .public_title
            .clone()
            .unwrap_or_else(|| input.subject_name.clone());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_subject_tracks
                     SET curriculum_version_id = ?1,
                         legacy_subject_id = ?2,
                         subject_code = ?3,
                         subject_name = ?4,
                         subject_slug = ?5,
                         public_title = ?6,
                         description = ?7,
                         display_order = ?8,
                         updated_at = datetime('now')
                     WHERE id = ?9",
                    params![
                        input.curriculum_version_id,
                        input.legacy_subject_id,
                        input.subject_code,
                        input.subject_name,
                        subject_slug,
                        public_title,
                        input.description,
                        input.display_order,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_subject_track(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("subject track {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_subject_tracks (
                        curriculum_version_id, legacy_subject_id, subject_code, subject_name,
                        subject_slug, public_title, description, display_order
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        input.curriculum_version_id,
                        input.legacy_subject_id,
                        input.subject_code,
                        input.subject_name,
                        subject_slug,
                        public_title,
                        input.description,
                        input.display_order,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_subject_track(self.conn.last_insert_rowid())?
                .ok_or_else(|| {
                    EcoachError::Storage(
                        "curriculum subject track insert did not persist".to_string(),
                    )
                })
        }
    }

    pub fn save_curriculum_level(
        &self,
        input: CurriculumLevelInput,
    ) -> EcoachResult<CurriculumLevel> {
        self.ensure_version_mutable(input.curriculum_version_id)?;
        let public_title = input
            .public_title
            .clone()
            .unwrap_or_else(|| input.level_name.clone());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_levels
                     SET curriculum_version_id = ?1,
                         level_code = ?2,
                         level_name = ?3,
                         stage_order = ?4,
                         public_title = ?5,
                         updated_at = datetime('now')
                     WHERE id = ?6",
                    params![
                        input.curriculum_version_id,
                        input.level_code,
                        input.level_name,
                        input.stage_order,
                        public_title,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_level(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("level {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_levels (
                        curriculum_version_id, level_code, level_name, stage_order, public_title
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        input.curriculum_version_id,
                        input.level_code,
                        input.level_name,
                        input.stage_order,
                        public_title,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_level(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("level insert did not persist".to_string()))
        }
    }

    pub fn save_curriculum_term_period(
        &self,
        input: CurriculumTermPeriodInput,
    ) -> EcoachResult<CurriculumTermPeriod> {
        let level = self.get_curriculum_level(input.level_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum level {} not found", input.level_id))
        })?;
        self.ensure_version_mutable(level.curriculum_version_id)?;
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_term_periods
                     SET level_id = ?1,
                         term_code = ?2,
                         term_name = ?3,
                         sequence_no = ?4,
                         public_term = ?5,
                         updated_at = datetime('now')
                     WHERE id = ?6",
                    params![
                        input.level_id,
                        input.term_code,
                        input.term_name,
                        input.sequence_no,
                        input.public_term,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_term_period(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("term period {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_term_periods (
                        level_id, term_code, term_name, sequence_no, public_term
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        input.level_id,
                        input.term_code,
                        input.term_name,
                        input.sequence_no,
                        input.public_term,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_term_period(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("term period insert did not persist".to_string()))
        }
    }

    pub fn save_curriculum_node_bundle(
        &self,
        input: CurriculumNodeBundleInput,
    ) -> EcoachResult<CurriculumNodeBundle> {
        let version_id = if let Some(node_id) = input.node.id {
            self.get_curriculum_node(node_id)?
                .ok_or_else(|| EcoachError::NotFound(format!("curriculum node {} not found", node_id)))?
                .curriculum_version_id
        } else {
            input.node.curriculum_version_id
        };
        self.ensure_version_mutable(version_id)?;
        let node = self.save_curriculum_node(input.node)?;
        self.replace_curriculum_objectives(node.id, &input.objectives)?;
        self.replace_curriculum_concepts(node.id, &input.concepts)?;
        self.replace_curriculum_aliases(node.id, "node", &input.aliases)?;
        self.replace_curriculum_relationships(node.id, &input.relationships)?;
        self.replace_curriculum_resource_links(node.id, &input.resource_links)?;
        self.get_curriculum_node_bundle(node.id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node bundle {} not found", node.id))
        })
    }

    pub fn approve_curriculum_node(
        &self,
        node_id: i64,
        _reviewer_note: Option<&str>,
    ) -> EcoachResult<CurriculumNodeBundle> {
        let node = self.get_curriculum_node(node_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node {} not found", node_id))
        })?;
        if node.status != "published" {
            self.conn
                .execute(
                    "UPDATE curriculum_nodes
                     SET status = 'approved',
                         review_status = 'approved',
                         confidence_score = CASE WHEN confidence_score < 7000 THEN 7000 ELSE confidence_score END,
                         updated_at = datetime('now')
                     WHERE id = ?1",
                    [node_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        self.conn
            .execute(
                "UPDATE curriculum_node_objectives
                 SET review_status = 'approved', updated_at = datetime('now')
                 WHERE curriculum_node_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_concept_atoms
                 SET review_status = 'approved', updated_at = datetime('now')
                 WHERE curriculum_node_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_resource_links
                 SET review_status = 'approved', updated_at = datetime('now')
                 WHERE entity_type = 'node' AND entity_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_curriculum_node_bundle(node_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node bundle {} not found", node_id))
        })
    }

    pub fn get_curriculum_subject_track(
        &self,
        id: i64,
    ) -> EcoachResult<Option<CurriculumSubjectTrack>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, legacy_subject_id, subject_code, subject_name,
                        subject_slug, public_title, description, display_order
                 FROM curriculum_subject_tracks
                 WHERE id = ?1",
                [id],
                map_curriculum_subject_track,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_curriculum_level(&self, id: i64) -> EcoachResult<Option<CurriculumLevel>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, level_code, level_name, stage_order, public_title
                 FROM curriculum_levels
                 WHERE id = ?1",
                [id],
                map_curriculum_level,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_curriculum_term_period(
        &self,
        id: i64,
    ) -> EcoachResult<Option<CurriculumTermPeriod>> {
        self.conn
            .query_row(
                "SELECT id, level_id, term_code, term_name, sequence_no, public_term
                 FROM curriculum_term_periods
                 WHERE id = ?1",
                [id],
                map_curriculum_term_period,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_curriculum_node(&self, id: i64) -> EcoachResult<Option<CurriculumNode>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, subject_track_id, level_id, term_id,
                        parent_node_id, legacy_topic_id, node_type, canonical_title,
                        public_title, slug, official_text, public_summary, sequence_no, depth,
                        estimated_weight, exam_relevance_score, difficulty_hint, status,
                        review_status, confidence_score
                 FROM curriculum_nodes
                 WHERE id = ?1",
                [id],
                map_curriculum_node,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_curriculum_node_bundle(
        &self,
        node_id: i64,
    ) -> EcoachResult<Option<CurriculumNodeBundle>> {
        let node = self.get_curriculum_node(node_id)?;
        let Some(node) = node else {
            return Ok(None);
        };
        let objectives = self.list_curriculum_objectives(node_id)?;
        let concepts = self.list_curriculum_concepts(node_id)?;
        let aliases = self.list_curriculum_aliases("node", node_id)?;
        let relationships = self.list_curriculum_relationships_from(node_id)?;
        let resource_links = self.list_curriculum_resource_links(node_id)?;

        Ok(Some(CurriculumNodeBundle {
            node,
            objectives,
            concepts,
            aliases,
            relationships,
            resource_links,
        }))
    }

    pub fn list_curriculum_review_queue(
        &self,
        limit: i64,
    ) -> EcoachResult<Vec<CurriculumReviewQueueItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT task.id, task.source_upload_id, task.candidate_id, task.task_type,
                        task.status, task.severity, task.notes,
                        upload.title, upload.source_kind,
                        candidate.candidate_type, candidate.raw_label
                 FROM curriculum_review_tasks task
                 INNER JOIN curriculum_source_uploads upload
                   ON upload.id = task.source_upload_id
                 LEFT JOIN curriculum_parse_candidates candidate
                   ON candidate.id = task.candidate_id
                 WHERE task.status IN ('open', 'pending')
                 ORDER BY CASE task.severity
                            WHEN 'high' THEN 0
                            WHEN 'medium' THEN 1
                            ELSE 2
                          END,
                          task.id ASC
                 LIMIT ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([limit.max(1)], |row| {
                Ok(CurriculumReviewQueueItem {
                    task: CurriculumReviewTask {
                        id: row.get(0)?,
                        source_upload_id: row.get(1)?,
                        candidate_id: row.get(2)?,
                        task_type: row.get(3)?,
                        status: row.get(4)?,
                        severity: row.get(5)?,
                        notes: row.get(6)?,
                    },
                    source_title: row.get(7)?,
                    source_kind: row.get(8)?,
                    candidate_type: row.get(9)?,
                    candidate_label: row.get(10)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn publish_curriculum_version(
        &self,
        curriculum_version_id: i64,
        generated_by_account_id: Option<i64>,
        notes: Option<&str>,
    ) -> EcoachResult<CurriculumPublishResult> {
        let version = self.get_curriculum_version(curriculum_version_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum version {} not found", curriculum_version_id))
        })?;
        let subject_count = self.count_subject_tracks(curriculum_version_id)?;
        let node_count = self.count_curriculum_nodes(curriculum_version_id)?;
        if subject_count == 0 || node_count == 0 {
            return Err(EcoachError::Validation(
                "curriculum version must have subjects and nodes before publish".to_string(),
            ));
        }
        let approved_node_count =
            self.count_curriculum_nodes_by_review(curriculum_version_id, "approved")?;
        if approved_node_count == 0 {
            return Err(EcoachError::Validation(
                "curriculum version must have approved nodes before publish".to_string(),
            ));
        }

        let previous_published = self.latest_published_version_in_family(
            version.curriculum_family_id,
            Some(curriculum_version_id),
        )?;
        let snapshot_payload = self.build_public_snapshot_payload(curriculum_version_id)?;
        let snapshot_json = serde_json::to_string(&snapshot_payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE curriculum_public_snapshots
                 SET status = 'superseded'
                 WHERE curriculum_version_id = ?1 AND status = 'live'",
                [curriculum_version_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO curriculum_public_snapshots (
                    curriculum_version_id, snapshot_kind, status, snapshot_json,
                    generated_by_account_id, generated_at, published_at
                 ) VALUES (?1, 'portal', 'live', ?2, ?3, datetime('now'), datetime('now'))",
                params![curriculum_version_id, snapshot_json, generated_by_account_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let snapshot_id = self.conn.last_insert_rowid();

        self.conn
            .execute(
                "INSERT INTO curriculum_publish_logs (
                    curriculum_version_id, snapshot_id, action_type, actor_account_id, notes
                 ) VALUES (?1, ?2, 'snapshot_generated', ?3, ?4)",
                params![curriculum_version_id, snapshot_id, generated_by_account_id, notes],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_versions
                 SET status = 'published',
                     published_at = datetime('now'),
                     updated_at = datetime('now')
                 WHERE id = ?1",
                [curriculum_version_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_nodes
                 SET status = 'published',
                     review_status = 'approved',
                     updated_at = datetime('now')
                 WHERE curriculum_version_id = ?1",
                [curriculum_version_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO curriculum_publish_logs (
                    curriculum_version_id, snapshot_id, action_type, actor_account_id, notes
                 ) VALUES (?1, ?2, 'published', ?3, ?4)",
                params![curriculum_version_id, snapshot_id, generated_by_account_id, notes],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let snapshot = self
            .get_curriculum_public_snapshot(snapshot_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum snapshot {} not found", snapshot_id))
            })?;
        let version = self.get_curriculum_version(curriculum_version_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "curriculum version {} not found after publish",
                curriculum_version_id
            ))
        })?;
        let diff_report = if let Some(previous) = previous_published {
            Some(self.get_curriculum_version_diff(previous.id, curriculum_version_id)?)
        } else {
            None
        };

        Ok(CurriculumPublishResult {
            version,
            snapshot,
            diff_report,
            subject_count,
            node_count,
        })
    }

    pub fn list_public_curriculum_subjects(
        &self,
        family_slug: &str,
        version_label: &str,
    ) -> EcoachResult<Vec<CurriculumSubjectTrack>> {
        let (_, version) = self.resolve_public_version(family_slug, version_label)?;
        self.list_subject_tracks_for_version(version.id)
    }

    pub fn get_public_curriculum_subject_overview(
        &self,
        family_slug: &str,
        version_label: &str,
        subject_slug: &str,
    ) -> EcoachResult<CurriculumPublicSubjectOverview> {
        let (family, version, subject) =
            self.resolve_public_subject(family_slug, version_label, subject_slug)?;
        let levels = self.list_levels_for_version(version.id)?;
        let total_node_count = self.count_nodes_for_subject(subject.id)?;
        let total_objective_count = self.count_objectives_for_subject(subject.id)?;
        let difficulty_distribution = self.get_subject_difficulty_distribution(subject.id)?;
        let latest_snapshot_id = self.latest_snapshot_id(version.id)?;

        Ok(CurriculumPublicSubjectOverview {
            family,
            version,
            subject,
            levels,
            total_node_count,
            total_objective_count,
            difficulty_distribution,
            latest_snapshot_id,
        })
    }

    pub fn get_public_curriculum_subject_tree(
        &self,
        family_slug: &str,
        version_label: &str,
        subject_slug: &str,
    ) -> EcoachResult<Vec<CurriculumTreeNode>> {
        let (_, version, subject) =
            self.resolve_public_subject(family_slug, version_label, subject_slug)?;
        let nodes = self.list_nodes_for_subject(version.id, subject.id, true)?;
        self.build_curriculum_tree(&nodes)
    }

    pub fn get_public_curriculum_topic_detail_by_slug(
        &self,
        slug: &str,
    ) -> EcoachResult<Option<CurriculumPublicTopicDetail>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT node.id
                 FROM curriculum_nodes node
                 INNER JOIN curriculum_versions version
                   ON version.id = node.curriculum_version_id
                 WHERE node.slug = ?1
                   AND version.status = 'published'
                   AND node.review_status = 'approved'
                 ORDER BY COALESCE(version.published_at, version.updated_at) DESC, node.id DESC
                 LIMIT 1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let node_id = statement
            .query_row([slug], |row| row.get::<_, i64>(0))
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(node_id) = node_id else {
            return Ok(None);
        };
        Ok(Some(self.build_public_topic_detail(node_id)?))
    }

    pub fn get_curriculum_version_diff(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
    ) -> EcoachResult<CurriculumVersionDiffReport> {
        let base_nodes = self.list_nodes_for_version(base_version_id)?;
        let compare_nodes = self.list_nodes_for_version(compare_version_id)?;
        let mut entries = Vec::new();

        let base_by_slug = map_nodes_by_slug(&base_nodes);
        let compare_by_slug = map_nodes_by_slug(&compare_nodes);

        for (slug, node) in &base_by_slug {
            if !compare_by_slug.contains_key(slug) {
                entries.push(CurriculumVersionDiffEntry {
                    diff_type: "removed_topic".to_string(),
                    entity_type: "node".to_string(),
                    entity_key: slug.clone(),
                    old_value: Some(node.public_title.clone()),
                    new_value: None,
                    summary: format!("{} was removed from the compared version.", node.public_title),
                });
            }
        }
        for (slug, node) in &compare_by_slug {
            if !base_by_slug.contains_key(slug) {
                entries.push(CurriculumVersionDiffEntry {
                    diff_type: "added_topic".to_string(),
                    entity_type: "node".to_string(),
                    entity_key: slug.clone(),
                    old_value: None,
                    new_value: Some(node.public_title.clone()),
                    summary: format!("{} was added in the compared version.", node.public_title),
                });
            }
        }
        for (slug, base_node) in &base_by_slug {
            if let Some(compare_node) = compare_by_slug.get(slug) {
                if normalize_text(&base_node.public_title)
                    != normalize_text(&compare_node.public_title)
                {
                    entries.push(CurriculumVersionDiffEntry {
                        diff_type: "renamed_topic".to_string(),
                        entity_type: "node".to_string(),
                        entity_key: slug.clone(),
                        old_value: Some(base_node.public_title.clone()),
                        new_value: Some(compare_node.public_title.clone()),
                        summary: format!(
                            "{} was renamed to {}.",
                            base_node.public_title, compare_node.public_title
                        ),
                    });
                }
                if base_node.parent_node_id != compare_node.parent_node_id {
                    entries.push(CurriculumVersionDiffEntry {
                        diff_type: "moved_topic".to_string(),
                        entity_type: "node".to_string(),
                        entity_key: slug.clone(),
                        old_value: base_node.parent_node_id.map(|id| id.to_string()),
                        new_value: compare_node.parent_node_id.map(|id| id.to_string()),
                        summary: format!("{} moved within the curriculum tree.", compare_node.public_title),
                    });
                }
                if base_node.sequence_no != compare_node.sequence_no {
                    entries.push(CurriculumVersionDiffEntry {
                        diff_type: "changed_ordering".to_string(),
                        entity_type: "node".to_string(),
                        entity_key: slug.clone(),
                        old_value: Some(base_node.sequence_no.to_string()),
                        new_value: Some(compare_node.sequence_no.to_string()),
                        summary: format!("{} changed position in the curriculum order.", compare_node.public_title),
                    });
                }
                if (base_node.exam_relevance_score - compare_node.exam_relevance_score).abs() >= 500
                {
                    entries.push(CurriculumVersionDiffEntry {
                        diff_type: "changed_assessment_emphasis".to_string(),
                        entity_type: "node".to_string(),
                        entity_key: slug.clone(),
                        old_value: Some(base_node.exam_relevance_score.to_string()),
                        new_value: Some(compare_node.exam_relevance_score.to_string()),
                        summary: format!(
                            "{} changed its assessment emphasis score.",
                            compare_node.public_title
                        ),
                    });
                }
            }
        }

        Ok(CurriculumVersionDiffReport {
            base_version_id,
            compare_version_id,
            migratable_question_links: self.count_migratable_resource_links(
                base_version_id,
                compare_version_id,
                "question",
            )?,
            migratable_glossary_links: self.count_migratable_resource_links(
                base_version_id,
                compare_version_id,
                "glossary",
            )?,
            migratable_study_plans: self.count_migratable_goals(base_version_id, compare_version_id)?,
            migratable_mastery_records: self
                .count_migratable_mastery_records(base_version_id, compare_version_id)?,
            entries,
        })
    }

    pub fn search_curriculum(
        &self,
        query: &str,
        published_only: bool,
        limit: i64,
    ) -> EcoachResult<Vec<CurriculumSearchResult>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let like = format!("%{}%", trimmed);
        let mut results = BTreeMap::<i64, CurriculumSearchResult>::new();

        let status_clause = if published_only {
            "version.status = 'published' AND node.review_status = 'approved'"
        } else {
            "node.review_status != 'rejected'"
        };

        let mut direct_stmt = self
            .conn
            .prepare(&format!(
                "SELECT node.id, node.subject_track_id, node.slug, node.public_title,
                        node.canonical_title, node.node_type, family.slug, version.version_label,
                        subject.public_title,
                        CASE
                            WHEN lower(node.public_title) = lower(?1) THEN 9600
                            WHEN lower(node.canonical_title) = lower(?1) THEN 9300
                            WHEN node.public_title LIKE ?2 THEN 8600
                            WHEN node.canonical_title LIKE ?2 THEN 8300
                            ELSE 7600
                        END AS score
                 FROM curriculum_nodes node
                 INNER JOIN curriculum_subject_tracks subject ON subject.id = node.subject_track_id
                 INNER JOIN curriculum_versions version ON version.id = node.curriculum_version_id
                 INNER JOIN curriculum_families family ON family.id = version.curriculum_family_id
                 WHERE {} AND (
                    node.public_title LIKE ?2 OR
                    node.canonical_title LIKE ?2 OR
                    COALESCE(node.public_summary, '') LIKE ?2 OR
                    COALESCE(node.official_text, '') LIKE ?2
                 )
                 ORDER BY score DESC, node.public_title ASC
                 LIMIT ?3",
                status_clause
            ))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = direct_stmt
            .query_map(params![trimmed, like, limit.max(1)], |row| {
                Ok(CurriculumSearchResult {
                    node_id: row.get(0)?,
                    subject_track_id: row.get(1)?,
                    slug: row.get(2)?,
                    public_title: row.get(3)?,
                    canonical_title: row.get(4)?,
                    node_type: row.get(5)?,
                    family_slug: row.get(6)?,
                    version_label: row.get(7)?,
                    subject_title: row.get(8)?,
                    relevance_score: row.get(9)?,
                    match_reasons: vec!["title or summary match".to_string()],
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            merge_search_result(&mut results, row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut alias_stmt = self
            .conn
            .prepare(&format!(
                "SELECT node.id, node.subject_track_id, node.slug, node.public_title,
                        node.canonical_title, node.node_type, family.slug, version.version_label,
                        subject.public_title, 8200, alias.alias_kind
                 FROM curriculum_aliases alias
                 INNER JOIN curriculum_nodes node
                   ON alias.entity_type = 'node' AND alias.entity_id = node.id
                 INNER JOIN curriculum_subject_tracks subject ON subject.id = node.subject_track_id
                 INNER JOIN curriculum_versions version ON version.id = node.curriculum_version_id
                 INNER JOIN curriculum_families family ON family.id = version.curriculum_family_id
                 WHERE {} AND alias.alias_text LIKE ?1
                 LIMIT ?2",
                status_clause
            ))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let alias_rows = alias_stmt
            .query_map(params![like, limit.max(1)], |row| {
                Ok(CurriculumSearchResult {
                    node_id: row.get(0)?,
                    subject_track_id: row.get(1)?,
                    slug: row.get(2)?,
                    public_title: row.get(3)?,
                    canonical_title: row.get(4)?,
                    node_type: row.get(5)?,
                    family_slug: row.get(6)?,
                    version_label: row.get(7)?,
                    subject_title: row.get(8)?,
                    relevance_score: row.get(9)?,
                    match_reasons: vec![format!("alias match ({})", row.get::<_, String>(10)?)],
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in alias_rows {
            merge_search_result(&mut results, row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut items: Vec<_> = results.into_values().collect();
        items.sort_by(|left, right| {
            right
                .relevance_score
                .cmp(&left.relevance_score)
                .then_with(|| left.public_title.cmp(&right.public_title))
        });
        items.truncate(limit.max(1) as usize);
        Ok(items)
    }

    pub fn get_curriculum_topic_resources(
        &self,
        node_id: i64,
    ) -> EcoachResult<Vec<CurriculumLinkedResource>> {
        let node = self.get_curriculum_node(node_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node {} not found", node_id))
        })?;
        let mut resources = self.list_curriculum_resource_links(node_id)?;
        let existing: BTreeSet<(String, i64)> = resources
            .iter()
            .map(|item| (item.resource_type.clone(), item.resource_id))
            .collect();

        if let Some(legacy_topic_id) = node.legacy_topic_id {
            let fallback_questions = self.list_fallback_question_links(node_id, legacy_topic_id)?;
            for item in fallback_questions {
                if !existing.contains(&(item.resource_type.clone(), item.resource_id)) {
                    resources.push(item);
                }
            }
            let fallback_entries = self.list_fallback_knowledge_links(node_id, legacy_topic_id)?;
            for item in fallback_entries {
                if !existing.contains(&(item.resource_type.clone(), item.resource_id)) {
                    resources.push(item);
                }
            }
        }

        resources.sort_by(|left, right| {
            right
                .link_strength
                .cmp(&left.link_strength)
                .then_with(|| left.resource_type.cmp(&right.resource_type))
                .then_with(|| left.display_label.cmp(&right.display_label))
        });
        Ok(resources)
    }

    pub fn get_curriculum_topic_context(&self, node_id: i64) -> EcoachResult<CurriculumTopicContext> {
        let detail = self.build_public_topic_detail(node_id)?;
        let formulas = detail
            .concepts
            .iter()
            .filter(|item| item.concept_type == "formula")
            .cloned()
            .collect::<Vec<_>>();
        let prerequisite_chain = self
            .get_curriculum_prerequisite_chain(node_id)?
            .into_iter()
            .filter_map(|step| self.get_curriculum_node(step.node_id).transpose())
            .collect::<EcoachResult<Vec<_>>>()?;

        Ok(CurriculumTopicContext {
            likely_question_types: detail.assessment_patterns.clone(),
            detail,
            formulas,
            prerequisite_chain,
        })
    }

    pub fn get_curriculum_next_best_topics(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: i64,
    ) -> EcoachResult<Vec<CurriculumRecommendation>> {
        let subject_track = self.latest_subject_track_for_legacy_subject(subject_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "curriculum subject track for legacy subject {} not found",
                subject_id
            ))
        })?;
        let nodes =
            self.list_nodes_for_subject(subject_track.curriculum_version_id, subject_track.id, false)?;
        let coverage = self.load_coverage_map(student_id)?;
        let package_scores = self.load_package_scores()?;
        let memory_states = self.load_memory_states(student_id)?;

        let mut items = Vec::new();
        for node in nodes
            .into_iter()
            .filter(|item| matches!(item.node_type.as_str(), "topic" | "subtopic" | "unit"))
        {
            let unmet = self.unmet_prerequisite_titles(student_id, &coverage, node.id)?;
            let legacy_status = node
                .legacy_topic_id
                .and_then(|id| coverage.get(&id).cloned())
                .unwrap_or_else(|| "not_introduced".to_string());
            let review_score = coverage_priority(&legacy_status);
            if review_score <= 1000 && unmet.is_empty() {
                continue;
            }
            let package_score = node
                .legacy_topic_id
                .and_then(|id| package_scores.get(&id).copied())
                .unwrap_or(5_500);
            let memory_penalty = node
                .legacy_topic_id
                .and_then(|id| memory_states.get(&id).copied())
                .unwrap_or(5_000);
            let blocker_penalty = (unmet.len() as i64) * 800;
            let priority_score = (node.exam_relevance_score
                + review_score
                + package_score
                + (10_000 - memory_penalty))
                .saturating_sub(blocker_penalty)
                .clamp(0, 10_000);
            let rationale = if unmet.is_empty() {
                format!(
                    "{} is ready for attention because coverage is {} and the topic carries exam weight.",
                    node.public_title, legacy_status
                )
            } else {
                format!(
                    "{} is important, but it still depends on {}.",
                    node.public_title,
                    unmet.join(", ")
                )
            };
            items.push(CurriculumRecommendation {
                node_id: node.id,
                slug: node.slug.clone(),
                public_title: node.public_title.clone(),
                rationale,
                priority_score,
                blocked_by_prerequisites: unmet,
            });
        }

        items.sort_by(|left, right| {
            right
                .priority_score
                .cmp(&left.priority_score)
                .then_with(|| left.public_title.cmp(&right.public_title))
        });
        items.truncate(limit.max(1) as usize);
        Ok(items)
    }

    pub fn get_curriculum_prerequisite_chain(
        &self,
        node_id: i64,
    ) -> EcoachResult<Vec<CurriculumPrerequisiteStep>> {
        let mut seen = BTreeSet::new();
        let mut frontier = vec![(node_id, 0_i64)];
        let mut chain = Vec::new();

        while let Some((current, depth)) = frontier.pop() {
            let prereqs = self.list_related_nodes(current, &["prerequisite", "depends_on"])?;
            for prereq in prereqs {
                if seen.insert(prereq.id) {
                    chain.push(CurriculumPrerequisiteStep {
                        node_id: prereq.id,
                        slug: prereq.slug.clone(),
                        public_title: prereq.public_title.clone(),
                        depth: depth + 1,
                    });
                    frontier.push((prereq.id, depth + 1));
                }
            }
        }

        chain.sort_by(|left, right| {
            left.depth
                .cmp(&right.depth)
                .then_with(|| left.public_title.cmp(&right.public_title))
        });
        Ok(chain)
    }

    pub fn get_curriculum_remediation_map(
        &self,
        node_id: i64,
    ) -> EcoachResult<CurriculumRemediationMap> {
        let detail = self.build_public_topic_detail(node_id)?;
        let mut steps = Vec::new();
        for (index, item) in self
            .get_curriculum_prerequisite_chain(node_id)?
            .into_iter()
            .enumerate()
        {
            let resource_count = self.count_resource_links_for_node(item.node_id)?;
            steps.push(CurriculumRemediationStep {
                sequence_no: index as i64 + 1,
                node_id: item.node_id,
                slug: item.slug,
                public_title: item.public_title,
                rationale: "Stabilize this dependency before retrying the target topic."
                    .to_string(),
                linked_resource_count: resource_count,
            });
        }
        steps.push(CurriculumRemediationStep {
            sequence_no: steps.len() as i64 + 1,
            node_id,
            slug: detail.node.slug.clone(),
            public_title: detail.node.public_title.clone(),
            rationale: "Return to the target topic after its dependencies are warmed back up."
                .to_string(),
            linked_resource_count: detail.resource_links.len() as i64,
        });

        Ok(CurriculumRemediationMap {
            target_node_id: detail.node.id,
            target_slug: detail.node.slug.clone(),
            target_title: detail.node.public_title.clone(),
            steps,
            misconceptions: detail.misconceptions.clone(),
        })
    }

    pub fn get_curriculum_coverage_stats(
        &self,
        subject_track_id: i64,
    ) -> EcoachResult<CurriculumCoverageStats> {
        let nodes = self.list_nodes_for_subject_track(subject_track_id)?;
        let total_nodes = nodes.len() as i64;
        let mut nodes_with_questions = 0_i64;
        let mut nodes_with_glossary = 0_i64;
        let mut nodes_with_notes = 0_i64;
        let mut nodes_with_games = 0_i64;
        let mut published_node_count = 0_i64;
        let mut weak_node_ids = Vec::new();

        for node in nodes {
            let resources = self.get_curriculum_topic_resources(node.id)?;
            let kinds: BTreeSet<String> =
                resources.iter().map(|item| item.resource_type.clone()).collect();
            if kinds.contains("question") {
                nodes_with_questions += 1;
            }
            if kinds.contains("glossary") {
                nodes_with_glossary += 1;
            }
            if kinds.contains("note") || kinds.contains("lesson") {
                nodes_with_notes += 1;
            }
            if kinds.contains("game") {
                nodes_with_games += 1;
            }
            if node.status == "published" {
                published_node_count += 1;
            }
            if !kinds.contains("question")
                || (!kinds.contains("glossary") && !kinds.contains("note"))
            {
                weak_node_ids.push(node.id);
            }
        }

        let numerator = (nodes_with_questions * 4)
            + (nodes_with_glossary * 2)
            + (nodes_with_notes * 2)
            + nodes_with_games
            + published_node_count;
        let denominator = total_nodes.max(1) * 10;
        let coverage_score_bp = ((numerator * 10_000) / denominator).clamp(0, 10_000);

        Ok(CurriculumCoverageStats {
            subject_track_id,
            total_nodes,
            nodes_with_questions,
            nodes_with_glossary,
            nodes_with_notes,
            nodes_with_games,
            published_node_count,
            coverage_score_bp,
            weak_node_ids,
        })
    }

    fn save_curriculum_node(&self, input: CurriculumNodeInput) -> EcoachResult<CurriculumNode> {
        let slug = input
            .slug
            .unwrap_or_else(|| slugify(&input.canonical_title));
        let public_title = input
            .public_title
            .clone()
            .unwrap_or_else(|| input.canonical_title.clone());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_nodes
                     SET curriculum_version_id = ?1,
                         subject_track_id = ?2,
                         level_id = ?3,
                         term_id = ?4,
                         parent_node_id = ?5,
                         legacy_topic_id = ?6,
                         node_type = ?7,
                         canonical_title = ?8,
                         public_title = ?9,
                         slug = ?10,
                         official_text = ?11,
                         public_summary = ?12,
                         sequence_no = ?13,
                         depth = ?14,
                         estimated_weight = ?15,
                         exam_relevance_score = ?16,
                         difficulty_hint = ?17,
                         confidence_score = ?18,
                         updated_at = datetime('now')
                     WHERE id = ?19",
                    params![
                        input.curriculum_version_id,
                        input.subject_track_id,
                        input.level_id,
                        input.term_id,
                        input.parent_node_id,
                        input.legacy_topic_id,
                        input.node_type,
                        input.canonical_title,
                        public_title,
                        slug,
                        input.official_text,
                        input.public_summary,
                        input.sequence_no,
                        input.depth,
                        input.estimated_weight.clamp(0, 10_000),
                        input.exam_relevance_score.clamp(0, 10_000),
                        input.difficulty_hint,
                        input.confidence_score.clamp(0, 10_000),
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum node {} not found after update", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_nodes (
                        curriculum_version_id, subject_track_id, level_id, term_id,
                        parent_node_id, legacy_topic_id, node_type, canonical_title, public_title,
                        slug, official_text, public_summary, sequence_no, depth,
                        estimated_weight, exam_relevance_score, difficulty_hint, confidence_score
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                    params![
                        input.curriculum_version_id,
                        input.subject_track_id,
                        input.level_id,
                        input.term_id,
                        input.parent_node_id,
                        input.legacy_topic_id,
                        input.node_type,
                        input.canonical_title,
                        public_title,
                        slug,
                        input.official_text,
                        input.public_summary,
                        input.sequence_no,
                        input.depth,
                        input.estimated_weight.clamp(0, 10_000),
                        input.exam_relevance_score.clamp(0, 10_000),
                        input.difficulty_hint,
                        input.confidence_score.clamp(0, 10_000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node(self.conn.last_insert_rowid())?
                .ok_or_else(|| {
                    EcoachError::Storage("curriculum node insert did not persist".to_string())
                })
        }
    }

    fn replace_curriculum_objectives(
        &self,
        node_id: i64,
        objectives: &[CurriculumObjectiveInput],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM curriculum_node_objectives WHERE curriculum_node_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for item in objectives {
            self.conn
                .execute(
                    "INSERT INTO curriculum_node_objectives (
                        curriculum_node_id, legacy_learning_objective_id, objective_text,
                        simplified_text, cognitive_level, objective_type, sequence_no,
                        confidence_score, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        node_id,
                        item.legacy_learning_objective_id,
                        item.objective_text,
                        item.simplified_text,
                        item.cognitive_level,
                        item.objective_type,
                        item.sequence_no,
                        item.confidence_score.clamp(0, 10_000),
                        item.review_status
                            .clone()
                            .unwrap_or_else(|| "pending_review".to_string()),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn replace_curriculum_concepts(
        &self,
        node_id: i64,
        concepts: &[CurriculumConceptAtomInput],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM curriculum_concept_atoms WHERE curriculum_node_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for item in concepts {
            self.conn
                .execute(
                    "INSERT INTO curriculum_concept_atoms (
                        curriculum_node_id, legacy_academic_node_id, concept_type,
                        canonical_term, public_term, description, alias_group_id, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        node_id,
                        item.legacy_academic_node_id,
                        item.concept_type,
                        item.canonical_term,
                        item.public_term,
                        item.description,
                        item.alias_group_id,
                        item.review_status
                            .clone()
                            .unwrap_or_else(|| "pending_review".to_string()),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn replace_curriculum_aliases(
        &self,
        node_id: i64,
        entity_type: &str,
        aliases: &[CurriculumAliasInput],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM curriculum_aliases WHERE entity_type = ?1 AND entity_id = ?2",
                params![entity_type, node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for item in aliases {
            self.conn
                .execute(
                    "INSERT INTO curriculum_aliases (
                        entity_type, entity_id, alias_text, alias_kind, locale
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        entity_type,
                        item.entity_id.unwrap_or(node_id),
                        item.alias_text,
                        item.alias_kind,
                        item.locale.clone().unwrap_or_else(|| "en".to_string()),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn replace_curriculum_relationships(
        &self,
        node_id: i64,
        relationships: &[CurriculumRelationshipInput],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM curriculum_relationships
                 WHERE from_entity_type = 'node' AND from_entity_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for item in relationships {
            self.conn
                .execute(
                    "INSERT INTO curriculum_relationships (
                        from_entity_type, from_entity_id, to_entity_type, to_entity_id,
                        relationship_type, strength_score
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        item.from_entity_type,
                        item.from_entity_id.unwrap_or(node_id),
                        item.to_entity_type,
                        item.to_entity_id,
                        item.relationship_type,
                        item.strength_score.clamp(0, 10_000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn replace_curriculum_resource_links(
        &self,
        node_id: i64,
        resources: &[CurriculumResourceLinkInput],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM curriculum_resource_links
                 WHERE entity_type = 'node' AND entity_id = ?1",
                [node_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for item in resources {
            self.conn
                .execute(
                    "INSERT INTO curriculum_resource_links (
                        entity_type, entity_id, resource_type, resource_id,
                        link_strength, source, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        item.entity_type,
                        item.entity_id.unwrap_or(node_id),
                        item.resource_type,
                        item.resource_id,
                        item.link_strength.clamp(0, 10_000),
                        item.source.clone().unwrap_or_else(|| "admin".to_string()),
                        item.review_status
                            .clone()
                            .unwrap_or_else(|| "approved".to_string()),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn list_curriculum_objectives(&self, node_id: i64) -> EcoachResult<Vec<CurriculumObjective>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_node_id, legacy_learning_objective_id, objective_text,
                        simplified_text, cognitive_level, objective_type, sequence_no,
                        confidence_score, review_status
                 FROM curriculum_node_objectives
                 WHERE curriculum_node_id = ?1
                 ORDER BY sequence_no ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], map_curriculum_objective)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_curriculum_concepts(&self, node_id: i64) -> EcoachResult<Vec<CurriculumConceptAtom>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_node_id, legacy_academic_node_id, concept_type,
                        canonical_term, public_term, description, alias_group_id, review_status
                 FROM curriculum_concept_atoms
                 WHERE curriculum_node_id = ?1
                 ORDER BY canonical_term ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], map_curriculum_concept_atom)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_curriculum_aliases(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> EcoachResult<Vec<CurriculumAlias>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, entity_type, entity_id, alias_text, alias_kind, locale
                 FROM curriculum_aliases
                 WHERE entity_type = ?1 AND entity_id = ?2
                 ORDER BY alias_kind ASC, alias_text ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![entity_type, entity_id], map_curriculum_alias)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_curriculum_relationships_from(
        &self,
        node_id: i64,
    ) -> EcoachResult<Vec<CurriculumRelationship>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, from_entity_type, from_entity_id, to_entity_type, to_entity_id,
                        relationship_type, strength_score
                 FROM curriculum_relationships
                 WHERE from_entity_type = 'node' AND from_entity_id = ?1
                 ORDER BY relationship_type ASC, strength_score DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], map_curriculum_relationship)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_curriculum_resource_links(
        &self,
        node_id: i64,
    ) -> EcoachResult<Vec<CurriculumLinkedResource>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, entity_type, entity_id, resource_type, resource_id, link_strength,
                        source, review_status
                 FROM curriculum_resource_links
                 WHERE entity_type = 'node' AND entity_id = ?1
                 ORDER BY link_strength DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            let (id, entity_type, entity_id, resource_type, resource_id, link_strength, source, review_status) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(CurriculumLinkedResource {
                id,
                entity_type,
                entity_id,
                resource_type: resource_type.clone(),
                resource_id,
                link_strength,
                source,
                review_status,
                display_label: self.resource_display_label(&resource_type, resource_id)?,
            });
        }
        Ok(items)
    }

    fn resolve_public_version(
        &self,
        family_slug: &str,
        version_label: &str,
    ) -> EcoachResult<(CurriculumFamily, CurriculumVersion)> {
        let family = self
            .get_curriculum_family_by_slug(family_slug)?
            .ok_or_else(|| EcoachError::NotFound(format!("curriculum family {} not found", family_slug)))?;
        let version = self
            .get_published_version_by_family_and_label(family.id, version_label)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "published curriculum version {} for family {} not found",
                    version_label, family_slug
                ))
            })?;
        Ok((family, version))
    }

    fn resolve_public_subject(
        &self,
        family_slug: &str,
        version_label: &str,
        subject_slug: &str,
    ) -> EcoachResult<(CurriculumFamily, CurriculumVersion, CurriculumSubjectTrack)> {
        let (family, version) = self.resolve_public_version(family_slug, version_label)?;
        let subject = self
            .get_subject_track_by_slug(version.id, subject_slug)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "subject {} not found for curriculum {}/{}",
                    subject_slug, family_slug, version_label
                ))
            })?;
        Ok((family, version, subject))
    }

    fn build_public_topic_detail(&self, node_id: i64) -> EcoachResult<CurriculumPublicTopicDetail> {
        let node = self.get_curriculum_node(node_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node {} not found", node_id))
        })?;
        let version = self.get_curriculum_version(node.curriculum_version_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "curriculum version {} not found",
                node.curriculum_version_id
            ))
        })?;
        let family_id = version.curriculum_family_id.ok_or_else(|| {
            EcoachError::Validation(format!(
                "curriculum version {} is missing its family",
                version.id
            ))
        })?;
        let family = self.get_curriculum_family(family_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum family {} not found", family_id))
        })?;
        let subject = self.get_curriculum_subject_track(node.subject_track_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("subject track {} not found", node.subject_track_id))
        })?;
        let objectives = self.list_curriculum_objectives(node.id)?;
        let concepts = self.list_curriculum_concepts(node.id)?;
        let aliases = self.list_curriculum_aliases("node", node.id)?;
        let prerequisites = self.list_related_nodes(node.id, &["prerequisite", "depends_on"])?;
        let related_nodes =
            self.list_related_nodes(node.id, &["related", "adjacent_to", "confused_with"])?;
        let resource_links = self.get_curriculum_topic_resources(node.id)?;
        let assessment_patterns = self.derive_assessment_patterns(&node, &resource_links)?;
        let misconceptions = self.list_misconceptions_for_node(&node)?;
        let latest_snapshot_id = self.latest_snapshot_id(version.id)?;

        Ok(CurriculumPublicTopicDetail {
            family,
            version,
            subject,
            node,
            objectives,
            concepts,
            aliases,
            prerequisites,
            related_nodes,
            resource_links,
            assessment_patterns,
            misconceptions,
            latest_snapshot_id,
        })
    }

    fn build_public_snapshot_payload(&self, version_id: i64) -> EcoachResult<Value> {
        let version = self.get_curriculum_version(version_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum version {} not found", version_id))
        })?;
        let family_id = version.curriculum_family_id.ok_or_else(|| {
            EcoachError::Validation(format!(
                "curriculum version {} is missing a family",
                version.id
            ))
        })?;
        let family = self.get_curriculum_family(family_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum family {} not found", family_id))
        })?;
        let subjects = self.list_subject_tracks_for_version(version_id)?;
        let mut subject_payloads = Vec::new();
        for subject in subjects {
            let tree = self.build_curriculum_tree(&self.list_nodes_for_subject(version_id, subject.id, false)?)?;
            subject_payloads.push(json!({
                "subject": subject,
                "tree": tree,
            }));
        }
        Ok(json!({
            "family": family,
            "version": version,
            "subjects": subject_payloads,
        }))
    }

    fn build_curriculum_tree(&self, nodes: &[CurriculumNode]) -> EcoachResult<Vec<CurriculumTreeNode>> {
        let mut by_parent: HashMap<Option<i64>, Vec<CurriculumNode>> = HashMap::new();
        let node_ids: Vec<i64> = nodes.iter().map(|item| item.id).collect();
        let objective_counts = self.count_grouped(
            "SELECT curriculum_node_id, COUNT(*) FROM curriculum_node_objectives WHERE curriculum_node_id IN ({}) GROUP BY curriculum_node_id",
            &node_ids,
        )?;
        let concept_counts = self.count_grouped(
            "SELECT curriculum_node_id, COUNT(*) FROM curriculum_concept_atoms WHERE curriculum_node_id IN ({}) GROUP BY curriculum_node_id",
            &node_ids,
        )?;
        let prerequisite_counts = self.count_grouped(
            "SELECT from_entity_id, COUNT(*) FROM curriculum_relationships WHERE from_entity_type = 'node' AND relationship_type IN ('prerequisite','depends_on') AND from_entity_id IN ({}) GROUP BY from_entity_id",
            &node_ids,
        )?;

        for node in nodes {
            by_parent.entry(node.parent_node_id).or_default().push(node.clone());
        }
        for children in by_parent.values_mut() {
            children.sort_by(|left, right| {
                left.sequence_no
                    .cmp(&right.sequence_no)
                    .then_with(|| left.public_title.cmp(&right.public_title))
            });
        }

        Ok(build_tree_children(
            None,
            &by_parent,
            &objective_counts,
            &concept_counts,
            &prerequisite_counts,
        ))
    }

    fn count_grouped(&self, sql_template: &str, ids: &[i64]) -> EcoachResult<HashMap<i64, usize>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let sql = sql_template.replacen("{}", &sql_placeholders(ids.len()), 1);
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(ids.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = HashMap::new();
        for row in rows {
            let (id, count) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(id, count);
        }
        Ok(map)
    }

    fn list_related_nodes(
        &self,
        node_id: i64,
        relation_types: &[&str],
    ) -> EcoachResult<Vec<CurriculumNode>> {
        if relation_types.is_empty() {
            return Ok(Vec::new());
        }
        let type_placeholder = sql_placeholders(relation_types.len());
        let mut params_vec: Vec<rusqlite::types::Value> =
            vec![rusqlite::types::Value::from(node_id)];
        for item in relation_types {
            params_vec.push(rusqlite::types::Value::from((*item).to_string()));
        }
        let mut statement = self
            .conn
            .prepare(&format!(
                "SELECT node.id, node.curriculum_version_id, node.subject_track_id, node.level_id,
                        node.term_id, node.parent_node_id, node.legacy_topic_id, node.node_type,
                        node.canonical_title, node.public_title, node.slug, node.official_text,
                        node.public_summary, node.sequence_no, node.depth, node.estimated_weight,
                        node.exam_relevance_score, node.difficulty_hint, node.status,
                        node.review_status, node.confidence_score
                 FROM curriculum_relationships relation
                 INNER JOIN curriculum_nodes node
                   ON relation.to_entity_type = 'node' AND relation.to_entity_id = node.id
                 WHERE relation.from_entity_type = 'node'
                   AND relation.from_entity_id = ?1
                   AND relation.relationship_type IN ({})
                 ORDER BY relation.strength_score DESC, node.public_title ASC",
                type_placeholder
            ))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), map_curriculum_node)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_fallback_question_links(
        &self,
        node_id: i64,
        legacy_topic_id: i64,
    ) -> EcoachResult<Vec<CurriculumLinkedResource>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, stem
                 FROM questions
                 WHERE topic_id = ?1
                 ORDER BY id ASC
                 LIMIT 50",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([legacy_topic_id], |row| {
                Ok(CurriculumLinkedResource {
                    id: 0,
                    entity_type: "node".to_string(),
                    entity_id: node_id,
                    resource_type: "question".to_string(),
                    resource_id: row.get(0)?,
                    link_strength: 7000,
                    source: "topic_fallback".to_string(),
                    review_status: "approved".to_string(),
                    display_label: row.get(1)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_fallback_knowledge_links(
        &self,
        node_id: i64,
        legacy_topic_id: i64,
    ) -> EcoachResult<Vec<CurriculumLinkedResource>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, entry_type, title
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                 ORDER BY importance_score DESC, id ASC
                 LIMIT 50",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([legacy_topic_id], |row| {
                let entry_type: String = row.get(1)?;
                let resource_type =
                    if matches!(entry_type.as_str(), "definition" | "term" | "formula") {
                        "glossary"
                    } else {
                        "note"
                    };
                Ok(CurriculumLinkedResource {
                    id: 0,
                    entity_type: "node".to_string(),
                    entity_id: node_id,
                    resource_type: resource_type.to_string(),
                    resource_id: row.get(0)?,
                    link_strength: 6800,
                    source: "knowledge_fallback".to_string(),
                    review_status: "approved".to_string(),
                    display_label: row.get(2)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn resource_display_label(&self, resource_type: &str, resource_id: i64) -> EcoachResult<String> {
        let sql = match resource_type {
            "question" => Some(("SELECT stem FROM questions WHERE id = ?1", "Question")),
            "glossary" | "note" | "lesson" => {
                Some(("SELECT title FROM knowledge_entries WHERE id = ?1", "Knowledge Entry"))
            }
            _ => None,
        };
        if let Some((statement, fallback)) = sql {
            let value = self
                .conn
                .query_row(statement, [resource_id], |row| row.get::<_, String>(0))
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(value) = value {
                return Ok(value);
            }
            return Ok(format!("{} {}", fallback, resource_id));
        }
        Ok(format!("{} {}", resource_type.replace('_', " "), resource_id))
    }

    fn latest_snapshot_id(&self, version_id: i64) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id
                 FROM curriculum_public_snapshots
                 WHERE curriculum_version_id = ?1 AND status = 'live'
                 ORDER BY generated_at DESC, id DESC
                 LIMIT 1",
                [version_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_curriculum_public_snapshot(
        &self,
        snapshot_id: i64,
    ) -> EcoachResult<Option<CurriculumPublicSnapshot>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, snapshot_kind, status, snapshot_json,
                        generated_by_account_id, generated_at, published_at
                 FROM curriculum_public_snapshots
                 WHERE id = ?1",
                [snapshot_id],
                map_curriculum_public_snapshot,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_curriculum_family_by_slug(&self, slug: &str) -> EcoachResult<Option<CurriculumFamily>> {
        self.conn
            .query_row(
                "SELECT id, slug, name, country_code, exam_board, education_stage, description, is_public
                 FROM curriculum_families
                 WHERE slug = ?1",
                [slug],
                map_curriculum_family,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_published_version_by_family_and_label(
        &self,
        family_id: i64,
        version_label: &str,
    ) -> EcoachResult<Option<CurriculumVersion>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_family_id, name, country, exam_board, education_stage,
                        version_label, status, effective_from, effective_to,
                        source_summary_json, published_at, replaced_by_version_id
                 FROM curriculum_versions
                 WHERE curriculum_family_id = ?1 AND version_label = ?2 AND status = 'published'
                 ORDER BY COALESCE(published_at, updated_at) DESC
                 LIMIT 1",
                params![family_id, version_label],
                map_curriculum_version,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_subject_track_by_slug(
        &self,
        version_id: i64,
        subject_slug: &str,
    ) -> EcoachResult<Option<CurriculumSubjectTrack>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, legacy_subject_id, subject_code, subject_name,
                        subject_slug, public_title, description, display_order
                 FROM curriculum_subject_tracks
                 WHERE curriculum_version_id = ?1 AND subject_slug = ?2",
                params![version_id, subject_slug],
                map_curriculum_subject_track,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn latest_subject_track_for_legacy_subject(
        &self,
        legacy_subject_id: i64,
    ) -> EcoachResult<Option<CurriculumSubjectTrack>> {
        self.conn
            .query_row(
                "SELECT track.id, track.curriculum_version_id, track.legacy_subject_id,
                        track.subject_code, track.subject_name, track.subject_slug,
                        track.public_title, track.description, track.display_order
                 FROM curriculum_subject_tracks track
                 INNER JOIN curriculum_versions version ON version.id = track.curriculum_version_id
                 WHERE track.legacy_subject_id = ?1
                 ORDER BY CASE WHEN version.status = 'published' THEN 0 ELSE 1 END,
                          COALESCE(version.published_at, version.updated_at) DESC,
                          track.id DESC
                 LIMIT 1",
                [legacy_subject_id],
                map_curriculum_subject_track,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_subject_tracks_for_version(
        &self,
        version_id: i64,
    ) -> EcoachResult<Vec<CurriculumSubjectTrack>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, legacy_subject_id, subject_code, subject_name,
                        subject_slug, public_title, description, display_order
                 FROM curriculum_subject_tracks
                 WHERE curriculum_version_id = ?1
                 ORDER BY display_order ASC, subject_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([version_id], map_curriculum_subject_track)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_levels_for_version(&self, version_id: i64) -> EcoachResult<Vec<CurriculumLevel>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, level_code, level_name, stage_order, public_title
                 FROM curriculum_levels
                 WHERE curriculum_version_id = ?1
                 ORDER BY stage_order ASC, level_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([version_id], map_curriculum_level)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_nodes_for_version(&self, version_id: i64) -> EcoachResult<Vec<CurriculumNode>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, subject_track_id, level_id, term_id,
                        parent_node_id, legacy_topic_id, node_type, canonical_title,
                        public_title, slug, official_text, public_summary, sequence_no, depth,
                        estimated_weight, exam_relevance_score, difficulty_hint, status,
                        review_status, confidence_score
                 FROM curriculum_nodes
                 WHERE curriculum_version_id = ?1
                 ORDER BY subject_track_id ASC, sequence_no ASC, public_title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([version_id], map_curriculum_node)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_nodes_for_subject(
        &self,
        version_id: i64,
        subject_track_id: i64,
        approved_only: bool,
    ) -> EcoachResult<Vec<CurriculumNode>> {
        let sql = if approved_only {
            "SELECT id, curriculum_version_id, subject_track_id, level_id, term_id,
                    parent_node_id, legacy_topic_id, node_type, canonical_title,
                    public_title, slug, official_text, public_summary, sequence_no, depth,
                    estimated_weight, exam_relevance_score, difficulty_hint, status,
                    review_status, confidence_score
             FROM curriculum_nodes
             WHERE curriculum_version_id = ?1
               AND subject_track_id = ?2
               AND review_status = 'approved'
             ORDER BY depth ASC, sequence_no ASC, public_title ASC"
        } else {
            "SELECT id, curriculum_version_id, subject_track_id, level_id, term_id,
                    parent_node_id, legacy_topic_id, node_type, canonical_title,
                    public_title, slug, official_text, public_summary, sequence_no, depth,
                    estimated_weight, exam_relevance_score, difficulty_hint, status,
                    review_status, confidence_score
             FROM curriculum_nodes
             WHERE curriculum_version_id = ?1
               AND subject_track_id = ?2
               AND review_status != 'rejected'
             ORDER BY depth ASC, sequence_no ASC, public_title ASC"
        };
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![version_id, subject_track_id], map_curriculum_node)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_nodes_for_subject_track(&self, subject_track_id: i64) -> EcoachResult<Vec<CurriculumNode>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, subject_track_id, level_id, term_id,
                        parent_node_id, legacy_topic_id, node_type, canonical_title,
                        public_title, slug, official_text, public_summary, sequence_no, depth,
                        estimated_weight, exam_relevance_score, difficulty_hint, status,
                        review_status, confidence_score
                 FROM curriculum_nodes
                 WHERE subject_track_id = ?1
                 ORDER BY depth ASC, sequence_no ASC, public_title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_track_id], map_curriculum_node)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn count_subject_tracks(&self, version_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_subject_tracks WHERE curriculum_version_id = ?1",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_curriculum_nodes(&self, version_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_nodes WHERE curriculum_version_id = ?1",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_curriculum_nodes_by_review(
        &self,
        version_id: i64,
        review_status: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_nodes
                 WHERE curriculum_version_id = ?1 AND review_status = ?2",
                params![version_id, review_status],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_nodes_for_subject(&self, subject_track_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_nodes WHERE subject_track_id = ?1",
                [subject_track_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_objectives_for_subject(&self, subject_track_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM curriculum_node_objectives objective
                 INNER JOIN curriculum_nodes node ON node.id = objective.curriculum_node_id
                 WHERE node.subject_track_id = ?1",
                [subject_track_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_resource_links_for_node(&self, node_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_resource_links
                 WHERE entity_type = 'node' AND entity_id = ?1",
                [node_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_subject_difficulty_distribution(&self, subject_track_id: i64) -> EcoachResult<Value> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT difficulty_hint, COUNT(*)
                 FROM curriculum_nodes
                 WHERE subject_track_id = ?1
                 GROUP BY difficulty_hint",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_track_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = serde_json::Map::new();
        for row in rows {
            let (label, count) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(label, Value::from(count));
        }
        Ok(Value::Object(map))
    }

    fn ensure_version_mutable(&self, version_id: i64) -> EcoachResult<()> {
        let status = self
            .conn
            .query_row(
                "SELECT status FROM curriculum_versions WHERE id = ?1",
                [version_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        match status.as_deref() {
            Some("published") => Err(EcoachError::Validation(
                "published curriculum versions are immutable; create a new version instead"
                    .to_string(),
            )),
            Some(_) => Ok(()),
            None => Err(EcoachError::NotFound(format!(
                "curriculum version {} not found",
                version_id
            ))),
        }
    }

    fn latest_published_version_in_family(
        &self,
        family_id: Option<i64>,
        exclude_version_id: Option<i64>,
    ) -> EcoachResult<Option<CurriculumVersion>> {
        let Some(family_id) = family_id else {
            return Ok(None);
        };
        let sql = if exclude_version_id.is_some() {
            "SELECT id, curriculum_family_id, name, country, exam_board, education_stage,
                    version_label, status, effective_from, effective_to, source_summary_json,
                    published_at, replaced_by_version_id
             FROM curriculum_versions
             WHERE curriculum_family_id = ?1
               AND status = 'published'
               AND id != ?2
             ORDER BY COALESCE(published_at, updated_at) DESC
             LIMIT 1"
        } else {
            "SELECT id, curriculum_family_id, name, country, exam_board, education_stage,
                    version_label, status, effective_from, effective_to, source_summary_json,
                    published_at, replaced_by_version_id
             FROM curriculum_versions
             WHERE curriculum_family_id = ?1
               AND status = 'published'
             ORDER BY COALESCE(published_at, updated_at) DESC
             LIMIT 1"
        };
        let result = if let Some(exclude_version_id) = exclude_version_id {
            self.conn
                .query_row(sql, params![family_id, exclude_version_id], map_curriculum_version)
        } else {
            self.conn.query_row(sql, [family_id], map_curriculum_version)
        };
        result
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_coverage_map(&self, student_id: i64) -> EcoachResult<HashMap<i64, String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, coverage_status
                 FROM curriculum_coverage_ledger
                 WHERE student_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = HashMap::new();
        for row in rows {
            let (topic_id, status) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(topic_id, status);
        }
        Ok(map)
    }

    fn load_package_scores(&self) -> EcoachResult<HashMap<i64, i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, MAX(resource_readiness_score)
                 FROM topic_package_snapshots
                 GROUP BY topic_id",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = HashMap::new();
        for row in rows {
            let (topic_id, score) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(topic_id, score);
        }
        Ok(map)
    }

    fn load_memory_states(&self, student_id: i64) -> EcoachResult<HashMap<i64, i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, memory_strength
                 FROM memory_states
                 WHERE student_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = HashMap::new();
        for row in rows {
            let (topic_id, score) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(topic_id, score);
        }
        Ok(map)
    }

    fn list_misconceptions_for_node(&self, node: &CurriculumNode) -> EcoachResult<Vec<String>> {
        let Some(legacy_topic_id) = node.legacy_topic_id else {
            return Ok(Vec::new());
        };
        let mut statement = self
            .conn
            .prepare(
                "SELECT title
                 FROM misconception_patterns
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY severity DESC, id ASC
                 LIMIT 12",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([legacy_topic_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn derive_assessment_patterns(
        &self,
        node: &CurriculumNode,
        resources: &[CurriculumLinkedResource],
    ) -> EcoachResult<Vec<CurriculumAssessmentPattern>> {
        let mut patterns = BTreeMap::<String, i64>::new();
        for resource in resources.iter().filter(|item| item.resource_type == "question") {
            let format = self
                .conn
                .query_row(
                    "SELECT question_format FROM questions WHERE id = ?1",
                    [resource.resource_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or_else(|| "question".to_string());
            *patterns.entry(format).or_default() += 1;
        }
        if patterns.is_empty() {
            for concept in self.list_curriculum_concepts(node.id)? {
                if concept.concept_type == "formula" {
                    *patterns.entry("formula_application".to_string()).or_default() += 1;
                }
            }
        }
        Ok(patterns
            .into_iter()
            .map(|(label, count)| CurriculumAssessmentPattern { label, count })
            .collect())
    }

    fn unmet_prerequisite_titles(
        &self,
        student_id: i64,
        coverage: &HashMap<i64, String>,
        node_id: i64,
    ) -> EcoachResult<Vec<String>> {
        let prerequisites = self.list_related_nodes(node_id, &["prerequisite", "depends_on"])?;
        let mut unmet = Vec::new();
        for prereq in prerequisites {
            let satisfied = prereq
                .legacy_topic_id
                .and_then(|id| coverage.get(&id))
                .map(|status| matches!(status.as_str(), "mastered" | "assessed" | "practiced"))
                .unwrap_or(false);
            if !satisfied {
                let reopened =
                    self.is_topic_fragile_in_memory(student_id, prereq.legacy_topic_id)?;
                if reopened || !satisfied {
                    unmet.push(prereq.public_title);
                }
            }
        }
        Ok(unmet)
    }

    fn is_topic_fragile_in_memory(
        &self,
        student_id: i64,
        legacy_topic_id: Option<i64>,
    ) -> EcoachResult<bool> {
        let Some(legacy_topic_id) = legacy_topic_id else {
            return Ok(false);
        };
        let state = self
            .conn
            .query_row(
                "SELECT memory_state
                 FROM memory_states
                 WHERE student_id = ?1 AND topic_id = ?2
                 ORDER BY id DESC
                 LIMIT 1",
                params![student_id, legacy_topic_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(matches!(state.as_deref(), Some("fragile") | Some("decayed")))
    }

    fn count_migratable_resource_links(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
        resource_type: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM curriculum_resource_links link
                 INNER JOIN curriculum_nodes base_node
                   ON base_node.id = link.entity_id
                 INNER JOIN curriculum_nodes compare_node
                   ON compare_node.curriculum_version_id = ?2
                  AND compare_node.slug = base_node.slug
                 WHERE link.entity_type = 'node'
                   AND link.resource_type = ?3
                   AND base_node.curriculum_version_id = ?1",
                params![base_version_id, compare_version_id, resource_type],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_migratable_goals(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
    ) -> EcoachResult<i64> {
        match self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM goals goal
                 INNER JOIN curriculum_nodes base_node
                   ON base_node.legacy_topic_id = goal.topic_id
                 INNER JOIN curriculum_nodes compare_node
                   ON compare_node.curriculum_version_id = ?2
                  AND compare_node.slug = base_node.slug
                 WHERE base_node.curriculum_version_id = ?1",
                params![base_version_id, compare_version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
        {
            Ok(value) => Ok(value),
            Err(EcoachError::Storage(_)) => Ok(0),
            Err(err) => Err(err),
        }
    }

    fn count_migratable_mastery_records(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM curriculum_coverage_ledger ledger
                 INNER JOIN curriculum_nodes base_node
                   ON base_node.legacy_topic_id = ledger.topic_id
                 INNER JOIN curriculum_nodes compare_node
                   ON compare_node.curriculum_version_id = ?2
                  AND compare_node.slug = base_node.slug
                 WHERE base_node.curriculum_version_id = ?1",
                params![base_version_id, compare_version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
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
    DateTime::from_str(&value).or_else(|_| {
        DateTime::parse_from_rfc3339(&value)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(EcoachError::Serialization(err.to_string())),
                )
            })
    })
}

fn map_source_upload(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumSourceUpload> {
    let metadata_json: String = row.get(14)?;
    let metadata = parse_json_column(14, &metadata_json)?;
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
    let payload = parse_json_column(6, &payload_json)?;
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

fn map_curriculum_family(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumFamily> {
    Ok(CurriculumFamily {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        country_code: row.get(3)?,
        exam_board: row.get(4)?,
        education_stage: row.get(5)?,
        description: row.get(6)?,
        is_public: row.get::<_, i64>(7)? == 1,
    })
}

fn map_curriculum_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumVersion> {
    let source_summary_json: String = row.get(10)?;
    Ok(CurriculumVersion {
        id: row.get(0)?,
        curriculum_family_id: row.get(1)?,
        name: row.get(2)?,
        country: row.get(3)?,
        exam_board: row.get(4)?,
        education_stage: row.get(5)?,
        version_label: row.get(6)?,
        status: row.get(7)?,
        effective_from: row.get(8)?,
        effective_to: row.get(9)?,
        source_summary: parse_json_column(10, &source_summary_json)?,
        published_at: row.get(11)?,
        replaced_by_version_id: row.get(12)?,
    })
}

fn map_curriculum_subject_track(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumSubjectTrack> {
    Ok(CurriculumSubjectTrack {
        id: row.get(0)?,
        curriculum_version_id: row.get(1)?,
        legacy_subject_id: row.get(2)?,
        subject_code: row.get(3)?,
        subject_name: row.get(4)?,
        subject_slug: row.get(5)?,
        public_title: row.get(6)?,
        description: row.get(7)?,
        display_order: row.get(8)?,
    })
}

fn map_curriculum_level(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumLevel> {
    Ok(CurriculumLevel {
        id: row.get(0)?,
        curriculum_version_id: row.get(1)?,
        level_code: row.get(2)?,
        level_name: row.get(3)?,
        stage_order: row.get(4)?,
        public_title: row.get(5)?,
    })
}

fn map_curriculum_term_period(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumTermPeriod> {
    Ok(CurriculumTermPeriod {
        id: row.get(0)?,
        level_id: row.get(1)?,
        term_code: row.get(2)?,
        term_name: row.get(3)?,
        sequence_no: row.get(4)?,
        public_term: row.get(5)?,
    })
}

fn map_curriculum_node(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumNode> {
    Ok(CurriculumNode {
        id: row.get(0)?,
        curriculum_version_id: row.get(1)?,
        subject_track_id: row.get(2)?,
        level_id: row.get(3)?,
        term_id: row.get(4)?,
        parent_node_id: row.get(5)?,
        legacy_topic_id: row.get(6)?,
        node_type: row.get(7)?,
        canonical_title: row.get(8)?,
        public_title: row.get(9)?,
        slug: row.get(10)?,
        official_text: row.get(11)?,
        public_summary: row.get(12)?,
        sequence_no: row.get(13)?,
        depth: row.get(14)?,
        estimated_weight: row.get(15)?,
        exam_relevance_score: row.get(16)?,
        difficulty_hint: row.get(17)?,
        status: row.get(18)?,
        review_status: row.get(19)?,
        confidence_score: row.get(20)?,
    })
}

fn map_curriculum_objective(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumObjective> {
    Ok(CurriculumObjective {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        legacy_learning_objective_id: row.get(2)?,
        objective_text: row.get(3)?,
        simplified_text: row.get(4)?,
        cognitive_level: row.get(5)?,
        objective_type: row.get(6)?,
        sequence_no: row.get(7)?,
        confidence_score: row.get(8)?,
        review_status: row.get(9)?,
    })
}

fn map_curriculum_concept_atom(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumConceptAtom> {
    Ok(CurriculumConceptAtom {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        legacy_academic_node_id: row.get(2)?,
        concept_type: row.get(3)?,
        canonical_term: row.get(4)?,
        public_term: row.get(5)?,
        description: row.get(6)?,
        alias_group_id: row.get(7)?,
        review_status: row.get(8)?,
    })
}

fn map_curriculum_alias(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumAlias> {
    Ok(CurriculumAlias {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        entity_id: row.get(2)?,
        alias_text: row.get(3)?,
        alias_kind: row.get(4)?,
        locale: row.get(5)?,
    })
}

fn map_curriculum_relationship(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumRelationship> {
    Ok(CurriculumRelationship {
        id: row.get(0)?,
        from_entity_type: row.get(1)?,
        from_entity_id: row.get(2)?,
        to_entity_type: row.get(3)?,
        to_entity_id: row.get(4)?,
        relationship_type: row.get(5)?,
        strength_score: row.get(6)?,
    })
}

fn map_curriculum_public_snapshot(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumPublicSnapshot> {
    let snapshot_json: String = row.get(4)?;
    Ok(CurriculumPublicSnapshot {
        id: row.get(0)?,
        curriculum_version_id: row.get(1)?,
        snapshot_kind: row.get(2)?,
        status: row.get(3)?,
        snapshot_json: parse_json_column(4, &snapshot_json)?,
        generated_by_account_id: row.get(5)?,
        generated_at: row.get(6)?,
        published_at: row.get(7)?,
    })
}

fn parse_json_column(index: usize, value: &str) -> rusqlite::Result<Value> {
    serde_json::from_str::<Value>(value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> EcoachResult<Vec<T>> {
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(items)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in value.chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            _ => None,
        };
        if let Some(mapped) = mapped {
            slug.push(mapped);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
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

fn merge_search_result(
    results: &mut BTreeMap<i64, CurriculumSearchResult>,
    incoming: CurriculumSearchResult,
) {
    results
        .entry(incoming.node_id)
        .and_modify(|existing| {
            existing.relevance_score = existing.relevance_score.max(incoming.relevance_score);
            for reason in &incoming.match_reasons {
                if !existing.match_reasons.contains(reason) {
                    existing.match_reasons.push(reason.clone());
                }
            }
        })
        .or_insert(incoming);
}

fn map_nodes_by_slug(nodes: &[CurriculumNode]) -> BTreeMap<String, CurriculumNode> {
    let mut map = BTreeMap::new();
    for node in nodes {
        map.insert(node.slug.clone(), node.clone());
    }
    map
}

fn build_tree_children(
    parent_id: Option<i64>,
    by_parent: &HashMap<Option<i64>, Vec<CurriculumNode>>,
    objective_counts: &HashMap<i64, usize>,
    concept_counts: &HashMap<i64, usize>,
    prerequisite_counts: &HashMap<i64, usize>,
) -> Vec<CurriculumTreeNode> {
    by_parent
        .get(&parent_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|node| CurriculumTreeNode {
            objective_count: objective_counts.get(&node.id).copied().unwrap_or_default(),
            concept_count: concept_counts.get(&node.id).copied().unwrap_or_default(),
            prerequisite_count: prerequisite_counts.get(&node.id).copied().unwrap_or_default(),
            children: build_tree_children(
                Some(node.id),
                by_parent,
                objective_counts,
                concept_counts,
                prerequisite_counts,
            ),
            node,
        })
        .collect()
}

fn sql_placeholders(count: usize) -> String {
    (0..count).map(|_| "?").collect::<Vec<_>>().join(", ")
}

fn coverage_priority(status: &str) -> i64 {
    match status {
        "not_introduced" => 9_000,
        "introduced" => 8_200,
        "taught" => 7_400,
        "practiced" => 6_200,
        "assessed" => 4_600,
        "unstable" => 7_800,
        "decayed" => 8_400,
        "re_opened" => 8_700,
        "mastered" => 900,
        _ => 7_000,
    }
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

    #[test]
    fn curriculum_portal_supports_public_snapshot_and_context() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_admin(&conn);

        let service = CurriculumService::new(&conn);
        let family = service
            .save_curriculum_family(CurriculumFamilyInput {
                id: None,
                slug: Some("ghana-bece".to_string()),
                name: "Ghana BECE".to_string(),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_stage: Some("JHS".to_string()),
                description: Some("National lower-secondary curriculum".to_string()),
                is_public: true,
            })
            .expect("family should save");
        let version = service
            .save_curriculum_version(CurriculumVersionInput {
                id: None,
                curriculum_family_id: family.id,
                name: "Ghana BECE 2027".to_string(),
                country: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_stage: Some("JHS".to_string()),
                version_label: "2027".to_string(),
                status: Some("draft".to_string()),
                effective_from: Some("2027-01-01".to_string()),
                effective_to: None,
                source_summary: json!({ "source": "idea26 backend test" }),
                replaced_by_version_id: None,
            })
            .expect("version should save");
        let subject = service
            .save_curriculum_subject_track(CurriculumSubjectTrackInput {
                id: None,
                curriculum_version_id: version.id,
                legacy_subject_id: None,
                subject_code: "MTH".to_string(),
                subject_name: "Mathematics".to_string(),
                subject_slug: Some("mathematics".to_string()),
                public_title: Some("Mathematics".to_string()),
                description: Some("Student facing curriculum branch".to_string()),
                display_order: 1,
            })
            .expect("subject track should save");
        let level = service
            .save_curriculum_level(CurriculumLevelInput {
                id: None,
                curriculum_version_id: version.id,
                level_code: "jhs3".to_string(),
                level_name: "JHS 3".to_string(),
                stage_order: 1,
                public_title: Some("JHS 3".to_string()),
            })
            .expect("level should save");
        let term = service
            .save_curriculum_term_period(CurriculumTermPeriodInput {
                id: None,
                level_id: level.id,
                term_code: "term-1".to_string(),
                term_name: "Term 1".to_string(),
                sequence_no: 1,
                public_term: Some("Term 1".to_string()),
            })
            .expect("term should save");
        let prerequisite = service
            .save_curriculum_node_bundle(CurriculumNodeBundleInput {
                node: CurriculumNodeInput {
                    id: None,
                    curriculum_version_id: version.id,
                    subject_track_id: subject.id,
                    level_id: Some(level.id),
                    term_id: Some(term.id),
                    parent_node_id: None,
                    legacy_topic_id: None,
                    node_type: "topic".to_string(),
                    canonical_title: "Fractions Foundations".to_string(),
                    public_title: Some("Fractions Foundations".to_string()),
                    slug: Some("fractions-foundations".to_string()),
                    official_text: Some("Fractions basics".to_string()),
                    public_summary: Some("Build the basic fraction skills first.".to_string()),
                    sequence_no: 1,
                    depth: 0,
                    estimated_weight: 6500,
                    exam_relevance_score: 7200,
                    difficulty_hint: "medium".to_string(),
                    confidence_score: 8000,
                },
                objectives: vec![CurriculumObjectiveInput {
                    id: None,
                    legacy_learning_objective_id: None,
                    objective_text: "Represent simple fractions.".to_string(),
                    simplified_text: Some("Show what a simple fraction means.".to_string()),
                    cognitive_level: Some("understanding".to_string()),
                    objective_type: "understanding".to_string(),
                    sequence_no: 1,
                    confidence_score: 7600,
                    review_status: Some("approved".to_string()),
                }],
                concepts: vec![CurriculumConceptAtomInput {
                    id: None,
                    legacy_academic_node_id: None,
                    concept_type: "concept".to_string(),
                    canonical_term: "Equivalent Fractions".to_string(),
                    public_term: Some("Equivalent Fractions".to_string()),
                    description: Some("Fractions that name the same value.".to_string()),
                    alias_group_id: None,
                    review_status: Some("approved".to_string()),
                }],
                aliases: vec![CurriculumAliasInput {
                    id: None,
                    entity_type: "node".to_string(),
                    entity_id: None,
                    alias_text: "Fraction Basics".to_string(),
                    alias_kind: "student_phrase".to_string(),
                    locale: Some("en".to_string()),
                }],
                relationships: vec![],
                resource_links: vec![],
            })
            .expect("prerequisite node should save");
        let target = service
            .save_curriculum_node_bundle(CurriculumNodeBundleInput {
                node: CurriculumNodeInput {
                    id: None,
                    curriculum_version_id: version.id,
                    subject_track_id: subject.id,
                    level_id: Some(level.id),
                    term_id: Some(term.id),
                    parent_node_id: None,
                    legacy_topic_id: None,
                    node_type: "topic".to_string(),
                    canonical_title: "Linear Equations".to_string(),
                    public_title: Some("Linear Equations".to_string()),
                    slug: Some("linear-equations".to_string()),
                    official_text: Some("Solve one-step and two-step equations".to_string()),
                    public_summary: Some("Balance and solve linear equations.".to_string()),
                    sequence_no: 2,
                    depth: 0,
                    estimated_weight: 7200,
                    exam_relevance_score: 8400,
                    difficulty_hint: "hard".to_string(),
                    confidence_score: 8200,
                },
                objectives: vec![CurriculumObjectiveInput {
                    id: None,
                    legacy_learning_objective_id: None,
                    objective_text: "Solve one-step linear equations.".to_string(),
                    simplified_text: Some("Find the missing value in simple equations.".to_string()),
                    cognitive_level: Some("application".to_string()),
                    objective_type: "application".to_string(),
                    sequence_no: 1,
                    confidence_score: 7800,
                    review_status: Some("approved".to_string()),
                }],
                concepts: vec![CurriculumConceptAtomInput {
                    id: None,
                    legacy_academic_node_id: None,
                    concept_type: "formula".to_string(),
                    canonical_term: "Inverse Operation".to_string(),
                    public_term: Some("Inverse Operation".to_string()),
                    description: Some("Undo operations to isolate the variable.".to_string()),
                    alias_group_id: None,
                    review_status: Some("approved".to_string()),
                }],
                aliases: vec![CurriculumAliasInput {
                    id: None,
                    entity_type: "node".to_string(),
                    entity_id: None,
                    alias_text: "Solve x".to_string(),
                    alias_kind: "student_phrase".to_string(),
                    locale: Some("en".to_string()),
                }],
                relationships: vec![CurriculumRelationshipInput {
                    id: None,
                    from_entity_type: "node".to_string(),
                    from_entity_id: None,
                    to_entity_type: "node".to_string(),
                    to_entity_id: prerequisite.node.id,
                    relationship_type: "prerequisite".to_string(),
                    strength_score: 9000,
                }],
                resource_links: vec![],
            })
            .expect("target node should save");
        service
            .approve_curriculum_node(prerequisite.node.id, None)
            .expect("prerequisite should approve");
        service
            .approve_curriculum_node(target.node.id, None)
            .expect("target should approve");
        let published = service
            .publish_curriculum_version(version.id, Some(1), Some("backend idea26 test publish"))
            .expect("version should publish");
        assert_eq!(published.version.status, "published");

        let subjects = service
            .list_public_curriculum_subjects("ghana-bece", "2027")
            .expect("subjects should list");
        assert_eq!(subjects.len(), 1);
        let overview = service
            .get_public_curriculum_subject_overview("ghana-bece", "2027", "mathematics")
            .expect("overview should load");
        assert!(overview.total_node_count >= 2);
        let tree = service
            .get_public_curriculum_subject_tree("ghana-bece", "2027", "mathematics")
            .expect("tree should load");
        assert_eq!(tree.len(), 2);
        let detail = service
            .get_public_curriculum_topic_detail_by_slug("linear-equations")
            .expect("detail query should succeed")
            .expect("detail should exist");
        assert_eq!(detail.node.public_title, "Linear Equations");
        assert_eq!(detail.prerequisites.len(), 1);
        let search = service
            .search_curriculum("Solve x", true, 10)
            .expect("search should work");
        assert!(search.iter().any(|item| item.slug == "linear-equations"));
        let context = service
            .get_curriculum_topic_context(target.node.id)
            .expect("topic context should load");
        assert!(context
            .prerequisite_chain
            .iter()
            .any(|node| node.slug == "fractions-foundations"));
        let remediation = service
            .get_curriculum_remediation_map(target.node.id)
            .expect("remediation map should load");
        assert!(remediation.steps.len() >= 2);
        let diff = service
            .get_curriculum_version_diff(version.id, version.id)
            .expect("diff should compute");
        assert_eq!(diff.base_version_id, version.id);
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
