use std::collections::{BTreeMap, BTreeSet};

use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{ContentPublishJobReport, ContentPublishService, ResourceReadinessService};

const LOW_CONFIDENCE_THRESHOLD: BasisPoints = 6_500;
const STRONG_CONFIDENCE_THRESHOLD: BasisPoints = 8_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceUploadInput {
    pub uploader_account_id: i64,
    pub source_kind: String,
    pub title: String,
    pub source_path: Option<String>,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_level: Option<String>,
    pub subject_code: Option<String>,
    pub academic_year: Option<String>,
    pub language_code: Option<String>,
    pub version_label: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCandidateInput {
    pub candidate_type: String,
    pub parent_candidate_id: Option<i64>,
    pub raw_label: String,
    pub normalized_label: Option<String>,
    pub payload: Value,
    pub confidence_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceUpload {
    pub id: i64,
    pub uploader_account_id: i64,
    pub source_kind: String,
    pub title: String,
    pub source_path: Option<String>,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_level: Option<String>,
    pub subject_code: Option<String>,
    pub academic_year: Option<String>,
    pub language_code: String,
    pub version_label: Option<String>,
    pub source_status: String,
    pub confidence_score: BasisPoints,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumParseCandidate {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_type: String,
    pub parent_candidate_id: Option<i64>,
    pub raw_label: String,
    pub normalized_label: Option<String>,
    pub payload: Value,
    pub confidence_score: BasisPoints,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumReviewTask {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_id: Option<i64>,
    pub task_type: String,
    pub status: String,
    pub severity: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCandidateCount {
    pub candidate_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackageSnapshot {
    pub topic_id: i64,
    pub subject_id: i64,
    pub topic_name: String,
    pub package_state: String,
    pub live_health_state: String,
    pub resource_readiness_score: BasisPoints,
    pub completeness_score: BasisPoints,
    pub quality_score: BasisPoints,
    pub evidence_score: BasisPoints,
    pub source_support_count: i64,
    pub contrast_pair_count: i64,
    pub publishable_artifact_count: i64,
    pub published_artifact_count: i64,
    pub missing_components: Vec<String>,
    pub recommended_jobs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectFoundryDashboard {
    pub subject_id: i64,
    pub subject_code: String,
    pub subject_name: String,
    pub source_upload_count: i64,
    pub pending_review_sources: i64,
    pub ready_publish_jobs: i64,
    pub published_jobs: i64,
    pub average_package_score: BasisPoints,
    pub weak_topic_count: i64,
    pub strong_topic_count: i64,
    pub topics: Vec<TopicPackageSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFoundrySourceReport {
    pub source_upload: CurriculumSourceUpload,
    pub candidate_counts: Vec<ParseCandidateCount>,
    pub parse_candidates: Vec<CurriculumParseCandidate>,
    pub review_tasks: Vec<CurriculumReviewTask>,
    pub publish_jobs: Vec<ContentPublishJobReport>,
    pub low_confidence_candidate_count: i64,
    pub approved_candidate_count: i64,
    pub unresolved_review_count: i64,
    pub duplicate_label_count: i64,
    pub publish_readiness_score: BasisPoints,
    pub can_mark_reviewed: bool,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryJob {
    pub id: i64,
    pub job_type: String,
    pub trigger_type: String,
    pub target_type: String,
    pub target_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub priority: BasisPoints,
    pub status: String,
    pub dependency_refs: Vec<String>,
    pub payload: Value,
    pub result_summary: Value,
    pub retry_count: i64,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryJobBoard {
    pub queued_count: i64,
    pub running_count: i64,
    pub blocked_count: i64,
    pub failed_count: i64,
    pub completed_count: i64,
    pub jobs: Vec<FoundryJob>,
}

pub struct FoundryCoordinatorService<'a> {
    conn: &'a Connection,
}

impl<'a> FoundryCoordinatorService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn register_source_upload(
        &self,
        input: SourceUploadInput,
    ) -> EcoachResult<CurriculumSourceUpload> {
        let metadata_json = serialize_json(&input.metadata)?;
        self.conn
            .execute(
                "INSERT INTO curriculum_source_uploads (
                    uploader_account_id, source_kind, title, source_path, country_code,
                    exam_board, education_level, subject_code, academic_year,
                    language_code, version_label, metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    input.uploader_account_id,
                    input.source_kind,
                    input.title,
                    input.source_path,
                    input.country_code,
                    input.exam_board,
                    input.education_level,
                    input.subject_code,
                    input.academic_year,
                    input.language_code.unwrap_or_else(|| "en".to_string()),
                    input.version_label,
                    metadata_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let source_upload_id = self.conn.last_insert_rowid();
        let source_upload = self.get_source_upload(source_upload_id)?.ok_or_else(|| {
            EcoachError::Storage("source upload insert did not persist".to_string())
        })?;
        self.append_runtime_event(
            "curriculum_source.registered",
            "curriculum_source",
            source_upload_id.to_string(),
            json!({
                "source_kind": source_upload.source_kind,
                "title": source_upload.title,
                "subject_code": source_upload.subject_code,
                "source_status": source_upload.source_status,
            }),
        )?;
        Ok(source_upload)
    }

    pub fn add_parse_candidate(
        &self,
        source_upload_id: i64,
        input: ParseCandidateInput,
    ) -> EcoachResult<CurriculumParseCandidate> {
        let payload_json = serialize_json(&input.payload)?;
        self.conn
            .execute(
                "INSERT INTO curriculum_parse_candidates (
                    source_upload_id, candidate_type, parent_candidate_id, raw_label,
                    normalized_label, payload_json, confidence_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    source_upload_id,
                    input.candidate_type,
                    input.parent_candidate_id,
                    input.raw_label,
                    input.normalized_label,
                    payload_json,
                    input.confidence_score as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.refresh_source_confidence(source_upload_id)?;
        self.get_parse_candidate(self.conn.last_insert_rowid())?
            .ok_or_else(|| {
                EcoachError::Storage("parse candidate insert did not persist".to_string())
            })
    }

    pub fn finalize_source_parse(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<ContentFoundrySourceReport> {
        let candidates = self.list_parse_candidates(source_upload_id)?;
        if candidates.is_empty() {
            self.ensure_review_task(
                source_upload_id,
                None,
                "publish_gate",
                "high",
                "No parse candidates were extracted. Re-run parsing before review.",
            )?;
        }

        let mut duplicate_counts = BTreeMap::<(String, String), i64>::new();
        let mut has_subject = false;
        let mut has_topic = false;
        let mut has_objective = false;

        for candidate in &candidates {
            match candidate.candidate_type.as_str() {
                "subject" => has_subject = true,
                "topic" | "subtopic" => has_topic = true,
                "objective" | "skill" => has_objective = true,
                _ => {}
            }

            let normalized = candidate
                .normalized_label
                .clone()
                .unwrap_or_else(|| candidate.raw_label.clone());
            let key = (
                candidate.candidate_type.clone(),
                normalize_phrase(&normalized),
            );
            *duplicate_counts.entry(key).or_insert(0) += 1;

            if candidate.confidence_score < LOW_CONFIDENCE_THRESHOLD
                || candidate.normalized_label.is_none()
            {
                self.ensure_review_task(
                    source_upload_id,
                    Some(candidate.id),
                    "normalization",
                    severity_for_confidence(candidate.confidence_score),
                    &format!(
                        "Candidate '{}' ({}) needs normalization review at {} bp confidence.",
                        candidate.raw_label, candidate.candidate_type, candidate.confidence_score
                    ),
                )?;
            } else {
                self.set_candidate_review_status(candidate.id, "approved")?;
            }
        }

        for ((candidate_type, normalized_label), count) in duplicate_counts {
            if count > 1 {
                self.ensure_review_task(
                    source_upload_id,
                    None,
                    "duplicate_check",
                    "medium",
                    &format!(
                        "Duplicate candidate label detected for {}:{}.",
                        candidate_type, normalized_label
                    ),
                )?;
            }
        }

        if !has_subject {
            self.ensure_review_task(
                source_upload_id,
                None,
                "hierarchy_fix",
                "high",
                "No subject node was extracted from this source.",
            )?;
        }
        if !has_topic {
            self.ensure_review_task(
                source_upload_id,
                None,
                "hierarchy_fix",
                "high",
                "No topic or subtopic nodes were extracted from this source.",
            )?;
        }
        if !has_objective {
            self.ensure_review_task(
                source_upload_id,
                None,
                "publish_gate",
                "medium",
                "No objective or skill nodes were extracted from this source.",
            )?;
        }

        let unresolved_review_count = self
            .list_review_tasks(source_upload_id)?
            .into_iter()
            .filter(|task| task.status != "resolved")
            .count() as i64;
        self.update_source_status(
            source_upload_id,
            if unresolved_review_count > 0 {
                "review_required"
            } else {
                "parsed"
            },
        )?;
        self.refresh_source_confidence(source_upload_id)?;

        let report = self.get_source_report(source_upload_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("source upload {} not found", source_upload_id))
        })?;
        self.append_runtime_event(
            "curriculum_source.parsed",
            "curriculum_source",
            source_upload_id.to_string(),
            json!({
                "source_status": report.source_upload.source_status,
                "candidate_count": report.parse_candidates.len(),
                "unresolved_review_count": report.unresolved_review_count,
                "publish_readiness_score": report.publish_readiness_score,
            }),
        )?;
        Ok(report)
    }

    pub fn resolve_review_task(
        &self,
        task_id: i64,
        resolution_note: &str,
        approve_candidate: bool,
    ) -> EcoachResult<CurriculumReviewTask> {
        let task = self
            .get_review_task(task_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("review task {} not found", task_id)))?;
        let notes = match task.notes.as_deref() {
            Some(existing) if !existing.is_empty() => {
                format!("{existing}\nResolution: {resolution_note}")
            }
            _ => format!("Resolution: {resolution_note}"),
        };
        self.conn
            .execute(
                "UPDATE curriculum_review_tasks
                 SET status = 'resolved', notes = ?1, resolved_at = datetime('now')
                 WHERE id = ?2",
                params![notes, task_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(candidate_id) = task.candidate_id {
            self.set_candidate_review_status(
                candidate_id,
                if approve_candidate {
                    "approved"
                } else {
                    "rejected"
                },
            )?;
        }
        if self
            .list_review_tasks(task.source_upload_id)?
            .into_iter()
            .all(|item| item.status == "resolved")
        {
            self.update_source_status(task.source_upload_id, "parsed")?;
        }
        self.get_review_task(task_id)?
            .ok_or_else(|| EcoachError::Storage("resolved review task vanished".to_string()))
    }

    pub fn mark_source_reviewed(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<ContentFoundrySourceReport> {
        let report = self.get_source_report(source_upload_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("source upload {} not found", source_upload_id))
        })?;
        if !report.can_mark_reviewed {
            return Err(EcoachError::Validation(
                "source still has unresolved review work or insufficient approved candidates"
                    .to_string(),
            ));
        }
        self.update_source_status(source_upload_id, "reviewed")?;
        self.get_source_report(source_upload_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("source upload {} not found", source_upload_id))
        })
    }

    pub fn get_source_report(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Option<ContentFoundrySourceReport>> {
        let Some(source_upload) = self.get_source_upload(source_upload_id)? else {
            return Ok(None);
        };
        let parse_candidates = self.list_parse_candidates(source_upload_id)?;
        let review_tasks = self.list_review_tasks(source_upload_id)?;
        let publish_jobs = self.list_publish_job_reports_for_source(source_upload_id)?;
        let low_confidence_candidate_count = parse_candidates
            .iter()
            .filter(|candidate| {
                candidate.confidence_score < LOW_CONFIDENCE_THRESHOLD
                    && candidate.review_status != "approved"
            })
            .count() as i64;
        let approved_candidate_count = parse_candidates
            .iter()
            .filter(|candidate| candidate.review_status == "approved")
            .count() as i64;
        let unresolved_review_count = review_tasks
            .iter()
            .filter(|task| task.status != "resolved")
            .count() as i64;
        let duplicate_label_count = count_duplicate_labels(&parse_candidates) as i64;
        let publish_readiness_score = compute_publish_readiness_score(
            &source_upload,
            approved_candidate_count,
            unresolved_review_count,
            duplicate_label_count,
            publish_jobs
                .iter()
                .any(|job| job.job.status == "ready_to_publish"),
            publish_jobs.iter().any(|job| job.job.status == "published"),
        );
        let can_mark_reviewed = unresolved_review_count == 0 && approved_candidate_count > 0;

        Ok(Some(ContentFoundrySourceReport {
            source_upload,
            candidate_counts: count_candidates_by_type(&parse_candidates),
            parse_candidates: parse_candidates.clone(),
            review_tasks,
            publish_jobs: publish_jobs.clone(),
            low_confidence_candidate_count,
            approved_candidate_count,
            unresolved_review_count,
            duplicate_label_count,
            publish_readiness_score,
            can_mark_reviewed,
            recommended_actions: build_source_actions(
                &parse_candidates,
                unresolved_review_count,
                duplicate_label_count,
                &publish_jobs,
                can_mark_reviewed,
            ),
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn stage_publish_job(
        &self,
        source_upload_id: i64,
        requested_by_account_id: Option<i64>,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        target_version_label: Option<&str>,
    ) -> EcoachResult<ContentPublishJobReport> {
        let report = self.get_source_report(source_upload_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("source upload {} not found", source_upload_id))
        })?;
        if report.can_mark_reviewed && report.source_upload.source_status != "reviewed" {
            self.update_source_status(source_upload_id, "reviewed")?;
        }

        let topic_snapshot = if let Some(topic_id) = topic_id {
            self.recompute_topic_package_snapshot(topic_id)?
        } else {
            None
        };
        let publish_service = ContentPublishService::new(self.conn);
        let publish_job_id = publish_service.create_publish_job(
            source_upload_id,
            None,
            requested_by_account_id,
            subject_id,
            topic_id,
            target_version_label,
            &json!({
                "source_status": report.source_upload.source_status,
                "approved_candidate_count": report.approved_candidate_count,
                "unresolved_review_count": report.unresolved_review_count,
                "duplicate_label_count": report.duplicate_label_count,
            }),
        )?;

        let parse_gate_status = if report.unresolved_review_count > 0
            || !matches!(
                report.source_upload.source_status.as_str(),
                "parsed" | "reviewed" | "published"
            ) {
            "needs_review"
        } else {
            "pass"
        };
        publish_service.add_quality_report(
            publish_job_id,
            "parse_gate",
            parse_gate_status,
            report.publish_readiness_score as i64,
            &json!({
                "approved_candidate_count": report.approved_candidate_count,
                "unresolved_review_count": report.unresolved_review_count,
                "duplicate_label_count": report.duplicate_label_count,
            }),
        )?;

        let trust_gate_status =
            if report.source_upload.confidence_score >= STRONG_CONFIDENCE_THRESHOLD {
                "pass"
            } else if report.source_upload.confidence_score >= LOW_CONFIDENCE_THRESHOLD {
                "warning"
            } else {
                "needs_review"
            };
        publish_service.add_quality_report(
            publish_job_id,
            "trust_gate",
            trust_gate_status,
            report.source_upload.confidence_score as i64,
            &json!({
                "source_confidence_score": report.source_upload.confidence_score,
                "source_status": report.source_upload.source_status,
            }),
        )?;

        let artifact_summary = if let Some(snapshot) = &topic_snapshot {
            let package_gate_status = if snapshot.completeness_score >= 6_500 {
                "pass"
            } else if snapshot.completeness_score >= 4_500
                || snapshot.resource_readiness_score >= 5_500
                || snapshot.source_support_count > 0
            {
                "warning"
            } else {
                "needs_review"
            };
            publish_service.add_quality_report(
                publish_job_id,
                "package_gate",
                package_gate_status,
                snapshot.quality_score as i64,
                &json!({
                    "topic_id": snapshot.topic_id,
                    "package_state": snapshot.package_state,
                    "live_health_state": snapshot.live_health_state,
                    "completeness_score": snapshot.completeness_score,
                    "missing_components": snapshot.missing_components,
                }),
            )?;
            json!({
                "topic_id": snapshot.topic_id,
                "package_state": snapshot.package_state,
                "live_health_state": snapshot.live_health_state,
                "completeness_score": snapshot.completeness_score,
                "quality_score": snapshot.quality_score,
                "recommended_jobs": snapshot.recommended_jobs,
            })
        } else {
            json!({
                "source_upload_id": source_upload_id,
                "publish_readiness_score": report.publish_readiness_score,
            })
        };

        if let Some(staged) = publish_service.get_publish_job_report(publish_job_id)? {
            if staged.blocking_report_count == 0 {
                publish_service.mark_ready_to_publish(publish_job_id, &artifact_summary)?;
            }
        }
        publish_service
            .get_publish_job_report(publish_job_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("publish job {} not found", publish_job_id))
            })
    }

    pub fn recompute_topic_package_snapshot(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Option<TopicPackageSnapshot>> {
        let readiness_service = ResourceReadinessService::new(self.conn);
        let Some(readiness) = readiness_service.get_topic_readiness(topic_id)? else {
            return Ok(None);
        };
        let (subject_code, topic_name) = self.topic_identity(topic_id)?;
        let reviewed_source_count = self.count_reviewed_sources_for_subject(&subject_code)?;
        let approved_evidence_count = self.count_approved_acquisition_evidence(topic_id)?;
        let contrast_pair_count = self.count_contrast_pairs(topic_id)?;
        let publishable_artifact_count = self.count_publish_jobs_for_topic(
            topic_id,
            &["ready_to_publish", "publishing", "published"],
        )?;
        let published_artifact_count =
            self.count_publish_jobs_for_topic(topic_id, &["published"])?;

        let evidence_score = compute_evidence_score(reviewed_source_count, approved_evidence_count);
        let completeness_score = compute_completeness_score(
            readiness.readiness_score,
            evidence_score,
            contrast_pair_count,
            publishable_artifact_count,
            published_artifact_count,
        );
        let quality_score = self.compute_topic_quality_score(
            readiness.readiness_score,
            topic_id,
            published_artifact_count,
        )?;

        let mut missing_components = readiness.missing_resources.clone();
        if reviewed_source_count + approved_evidence_count == 0 {
            missing_components.push("source_support_missing".to_string());
        }
        if contrast_pair_count == 0 {
            missing_components.push("contrast_pack_missing".to_string());
        }
        if publishable_artifact_count == 0 {
            missing_components.push("publish_candidate_missing".to_string());
        }
        if published_artifact_count == 0 {
            missing_components.push("live_artifact_missing".to_string());
        }
        if quality_score < 6_500 {
            missing_components.push("quality_gate_not_met".to_string());
        }
        dedupe_and_sort(&mut missing_components);

        let recommended_jobs = build_recommended_jobs(&missing_components);
        let package_state = derive_package_state(
            readiness.readiness_score,
            completeness_score,
            quality_score,
            publishable_artifact_count,
            published_artifact_count,
        );
        let live_health_state =
            derive_live_health_state(&package_state, completeness_score, published_artifact_count);

        self.upsert_topic_package_snapshot(
            readiness.subject_id,
            topic_id,
            &package_state,
            &live_health_state,
            readiness.readiness_score,
            completeness_score,
            quality_score,
            evidence_score,
            reviewed_source_count + approved_evidence_count,
            contrast_pair_count,
            publishable_artifact_count,
            published_artifact_count,
            &missing_components,
            &recommended_jobs,
        )?;

        Ok(Some(TopicPackageSnapshot {
            topic_id,
            subject_id: readiness.subject_id,
            topic_name,
            package_state,
            live_health_state,
            resource_readiness_score: readiness.readiness_score,
            completeness_score,
            quality_score,
            evidence_score,
            source_support_count: reviewed_source_count + approved_evidence_count,
            contrast_pair_count,
            publishable_artifact_count,
            published_artifact_count,
            missing_components,
            recommended_jobs,
        }))
    }

    pub fn get_subject_foundry_dashboard(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Option<SubjectFoundryDashboard>> {
        let subject = self
            .conn
            .query_row(
                "SELECT id, code, name FROM subjects WHERE id = ?1 AND is_active = 1",
                [subject_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((subject_id, subject_code, subject_name)) = subject else {
            return Ok(None);
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id FROM topics
                 WHERE subject_id = ?1 AND is_active = 1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(snapshot) = self.recompute_topic_package_snapshot(topic_id)? {
                topics.push(snapshot);
            }
        }

        let average_package_score = if topics.is_empty() {
            0
        } else {
            clamp_bp(
                topics
                    .iter()
                    .map(|topic| topic.completeness_score as i64)
                    .sum::<i64>()
                    / topics.len() as i64,
            )
        };

        Ok(Some(SubjectFoundryDashboard {
            subject_id,
            subject_code: subject_code.clone(),
            subject_name,
            source_upload_count: self.count_sources_for_subject(&subject_code)?,
            pending_review_sources: self
                .count_sources_by_status(&subject_code, "review_required")?,
            ready_publish_jobs: self
                .count_publish_jobs_for_subject(subject_id, "ready_to_publish")?,
            published_jobs: self.count_publish_jobs_for_subject(subject_id, "published")?,
            average_package_score,
            weak_topic_count: topics
                .iter()
                .filter(|topic| topic.completeness_score < 5_500)
                .count() as i64,
            strong_topic_count: topics
                .iter()
                .filter(|topic| topic.live_health_state == "live_strong")
                .count() as i64,
            topics,
        }))
    }

    pub fn queue_topic_foundry_jobs(
        &self,
        topic_id: i64,
        trigger_type: &str,
    ) -> EcoachResult<Vec<FoundryJob>> {
        let Some(snapshot) = self.recompute_topic_package_snapshot(topic_id)? else {
            return Ok(Vec::new());
        };
        let mut queued_jobs = Vec::new();
        for job_type in &snapshot.recommended_jobs {
            let job = self.queue_foundry_job_if_missing(
                job_type,
                trigger_type,
                "topic_package",
                topic_id,
                Some(snapshot.subject_id),
                Some(topic_id),
                topic_priority(
                    snapshot.completeness_score,
                    snapshot.quality_score,
                    job_type,
                ),
                &dependencies_for_topic_job(job_type, &snapshot),
                &json!({
                    "topic_id": snapshot.topic_id,
                    "topic_name": snapshot.topic_name,
                    "package_state": snapshot.package_state,
                    "live_health_state": snapshot.live_health_state,
                    "missing_components": snapshot.missing_components,
                }),
            )?;
            queued_jobs.push(job);
        }
        self.append_runtime_event(
            "content_foundry.topic_jobs_queued",
            "topic_package",
            topic_id.to_string(),
            json!({
                "topic_id": topic_id,
                "job_count": queued_jobs.len(),
                "job_types": queued_jobs.iter().map(|job| job.job_type.clone()).collect::<Vec<_>>(),
            }),
        )?;
        Ok(queued_jobs)
    }

    pub fn queue_source_follow_up_jobs(
        &self,
        source_upload_id: i64,
        trigger_type: &str,
    ) -> EcoachResult<Vec<FoundryJob>> {
        let report = self.get_source_report(source_upload_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("source upload {} not found", source_upload_id))
        })?;
        let subject_id = if let Some(subject_code) = report.source_upload.subject_code.as_deref() {
            self.subject_id_for_code(subject_code)?
        } else {
            None
        };
        let mut queued_jobs = Vec::new();
        for action in &report.recommended_actions {
            if let Some(job_type) = map_source_action_to_job_type(action) {
                let job = self.queue_foundry_job_if_missing(
                    job_type,
                    trigger_type,
                    "source_upload",
                    source_upload_id,
                    subject_id,
                    None,
                    source_priority(report.publish_readiness_score, action),
                    &dependencies_for_source_job(action, &report),
                    &json!({
                        "source_upload_id": source_upload_id,
                        "title": report.source_upload.title,
                        "source_status": report.source_upload.source_status,
                        "recommended_action": action,
                    }),
                )?;
                queued_jobs.push(job);
            }
        }
        self.append_runtime_event(
            "content_foundry.source_jobs_queued",
            "curriculum_source",
            source_upload_id.to_string(),
            json!({
                "source_upload_id": source_upload_id,
                "job_count": queued_jobs.len(),
                "job_types": queued_jobs.iter().map(|job| job.job_type.clone()).collect::<Vec<_>>(),
            }),
        )?;
        Ok(queued_jobs)
    }

    pub fn list_foundry_jobs(
        &self,
        status: Option<&str>,
        target_type: Option<&str>,
        subject_id: Option<i64>,
    ) -> EcoachResult<Vec<FoundryJob>> {
        let sql = match (
            status.is_some(),
            target_type.is_some(),
            subject_id.is_some(),
        ) {
            (true, true, true) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE status = ?1 AND target_type = ?2 AND subject_id = ?3
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (true, true, false) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE status = ?1 AND target_type = ?2
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (true, false, true) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE status = ?1 AND subject_id = ?2
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (true, false, false) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE status = ?1
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (false, true, true) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE target_type = ?1 AND subject_id = ?2
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (false, true, false) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE target_type = ?1
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (false, false, true) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE subject_id = ?1
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
            (false, false, false) => {
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 ORDER BY priority DESC, created_at ASC, id ASC"
            }
        };

        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = match (status, target_type, subject_id) {
            (Some(status), Some(target_type), Some(subject_id)) => {
                statement.query_map(params![status, target_type, subject_id], map_foundry_job)
            }
            (Some(status), Some(target_type), None) => {
                statement.query_map(params![status, target_type], map_foundry_job)
            }
            (Some(status), None, Some(subject_id)) => {
                statement.query_map(params![status, subject_id], map_foundry_job)
            }
            (Some(status), None, None) => statement.query_map(params![status], map_foundry_job),
            (None, Some(target_type), Some(subject_id)) => {
                statement.query_map(params![target_type, subject_id], map_foundry_job)
            }
            (None, Some(target_type), None) => {
                statement.query_map(params![target_type], map_foundry_job)
            }
            (None, None, Some(subject_id)) => {
                statement.query_map(params![subject_id], map_foundry_job)
            }
            (None, None, None) => statement.query_map([], map_foundry_job),
        }
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn get_foundry_job_board(&self, subject_id: Option<i64>) -> EcoachResult<FoundryJobBoard> {
        let jobs = self.list_foundry_jobs(None, None, subject_id)?;
        Ok(FoundryJobBoard {
            queued_count: jobs.iter().filter(|job| job.status == "queued").count() as i64,
            running_count: jobs.iter().filter(|job| job.status == "running").count() as i64,
            blocked_count: jobs.iter().filter(|job| job.status == "blocked").count() as i64,
            failed_count: jobs.iter().filter(|job| job.status == "failed").count() as i64,
            completed_count: jobs.iter().filter(|job| job.status == "completed").count() as i64,
            jobs,
        })
    }

    pub fn start_foundry_job(&self, job_id: i64) -> EcoachResult<FoundryJob> {
        self.update_foundry_job_status(job_id, "running", None, None, false, true)?;
        self.get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job_id)))
    }

    pub fn complete_foundry_job(
        &self,
        job_id: i64,
        result_summary: &Value,
    ) -> EcoachResult<FoundryJob> {
        self.update_foundry_job_status(
            job_id,
            "completed",
            Some(result_summary),
            None,
            true,
            false,
        )?;
        self.get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job_id)))
    }

    pub fn fail_foundry_job(&self, job_id: i64, failure_reason: &str) -> EcoachResult<FoundryJob> {
        self.update_foundry_job_status(job_id, "failed", None, Some(failure_reason), true, false)?;
        self.get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job_id)))
    }

    fn get_source_upload(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Option<CurriculumSourceUpload>> {
        self.conn
            .query_row(
                "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                        exam_board, education_level, subject_code, academic_year, language_code,
                        version_label, source_status, confidence_score, metadata_json
                 FROM curriculum_source_uploads
                 WHERE id = ?1",
                [source_upload_id],
                map_source_upload,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_parse_candidate(
        &self,
        candidate_id: i64,
    ) -> EcoachResult<Option<CurriculumParseCandidate>> {
        self.conn
            .query_row(
                "SELECT id, source_upload_id, candidate_type, parent_candidate_id, raw_label,
                        normalized_label, payload_json, confidence_score, review_status
                 FROM curriculum_parse_candidates
                 WHERE id = ?1",
                [candidate_id],
                map_parse_candidate,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_parse_candidates(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Vec<CurriculumParseCandidate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, source_upload_id, candidate_type, parent_candidate_id, raw_label,
                        normalized_label, payload_json, confidence_score, review_status
                 FROM curriculum_parse_candidates
                 WHERE source_upload_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([source_upload_id], map_parse_candidate)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn get_review_task(&self, task_id: i64) -> EcoachResult<Option<CurriculumReviewTask>> {
        self.conn
            .query_row(
                "SELECT id, source_upload_id, candidate_id, task_type, status, severity, notes
                 FROM curriculum_review_tasks
                 WHERE id = ?1",
                [task_id],
                map_review_task,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_review_tasks(&self, source_upload_id: i64) -> EcoachResult<Vec<CurriculumReviewTask>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, source_upload_id, candidate_id, task_type, status, severity, notes
                 FROM curriculum_review_tasks
                 WHERE source_upload_id = ?1
                 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([source_upload_id], map_review_task)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_publish_job_reports_for_source(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Vec<ContentPublishJobReport>> {
        let publish_service = ContentPublishService::new(self.conn);
        let mut statement = self
            .conn
            .prepare(
                "SELECT id
                 FROM content_publish_jobs
                 WHERE source_upload_id = ?1
                 ORDER BY created_at DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([source_upload_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut reports = Vec::new();
        for row in rows {
            let publish_job_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(report) = publish_service.get_publish_job_report(publish_job_id)? {
                reports.push(report);
            }
        }
        Ok(reports)
    }

    fn ensure_review_task(
        &self,
        source_upload_id: i64,
        candidate_id: Option<i64>,
        task_type: &str,
        severity: &str,
        notes: &str,
    ) -> EcoachResult<()> {
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM curriculum_review_tasks
                    WHERE source_upload_id = ?1
                      AND ((candidate_id = ?2) OR (candidate_id IS NULL AND ?2 IS NULL))
                      AND task_type = ?3
                      AND notes = ?4
                      AND status != 'resolved'
                 )",
                params![source_upload_id, candidate_id, task_type, notes],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO curriculum_review_tasks (
                        source_upload_id, candidate_id, task_type, status, severity, notes
                     ) VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
                    params![source_upload_id, candidate_id, task_type, severity, notes],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn refresh_source_confidence(&self, source_upload_id: i64) -> EcoachResult<()> {
        let average_confidence = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(confidence_score), 0)
                 FROM curriculum_parse_candidates
                 WHERE source_upload_id = ?1",
                [source_upload_id],
                |row| row.get::<_, f64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE curriculum_source_uploads
                 SET confidence_score = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![average_confidence.round() as i64, source_upload_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn set_candidate_review_status(
        &self,
        candidate_id: i64,
        review_status: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE curriculum_parse_candidates
                 SET review_status = ?1, updated_at = datetime('now')
                 WHERE id = ?2",
                params![review_status, candidate_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn update_source_status(&self, source_upload_id: i64, source_status: &str) -> EcoachResult<()> {
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

    fn topic_identity(&self, topic_id: i64) -> EcoachResult<(String, String)> {
        self.conn
            .query_row(
                "SELECT s.code, t.name
                 FROM topics t
                 JOIN subjects s ON s.id = t.subject_id
                 WHERE t.id = ?1",
                [topic_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_reviewed_sources_for_subject(&self, subject_code: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_source_uploads
                 WHERE subject_code = ?1 AND source_status IN ('reviewed', 'published')",
                [subject_code],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_sources_for_subject(&self, subject_code: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_source_uploads WHERE subject_code = ?1",
                [subject_code],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_sources_by_status(&self, subject_code: &str, status: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_source_uploads
                 WHERE subject_code = ?1 AND source_status = ?2",
                params![subject_code, status],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_approved_acquisition_evidence(&self, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM acquisition_evidence_candidates candidates
                 JOIN content_acquisition_jobs jobs ON jobs.id = candidates.job_id
                 WHERE jobs.topic_id = ?1 AND candidates.review_status = 'approved'",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_contrast_pairs(&self, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(DISTINCT pairs.id)
                 FROM contrast_pairs pairs
                 JOIN knowledge_entries left_entry ON left_entry.id = pairs.left_entry_id
                 JOIN knowledge_entries right_entry ON right_entry.id = pairs.right_entry_id
                 WHERE left_entry.topic_id = ?1 OR right_entry.topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_publish_jobs_for_topic(&self, topic_id: i64, statuses: &[&str]) -> EcoachResult<i64> {
        let quoted_statuses = statuses
            .iter()
            .map(|status| format!("'{}'", status))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT COUNT(*) FROM content_publish_jobs
             WHERE topic_id = ?1 AND status IN ({quoted_statuses})"
        );
        self.conn
            .query_row(&sql, [topic_id], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_publish_jobs_for_subject(&self, subject_id: i64, status: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM content_publish_jobs
                 WHERE subject_id = ?1 AND status = ?2",
                params![subject_id, status],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn compute_topic_quality_score(
        &self,
        readiness_score: BasisPoints,
        topic_id: i64,
        published_artifact_count: i64,
    ) -> EcoachResult<BasisPoints> {
        let avg_quality_score = self
            .conn
            .query_row(
                "SELECT AVG(reports.confidence_score)
                 FROM content_quality_reports reports
                 JOIN content_publish_jobs jobs ON jobs.id = reports.publish_job_id
                 WHERE jobs.topic_id = ?1
                   AND jobs.status IN ('ready_to_publish', 'publishing', 'published')",
                [topic_id],
                |row| row.get::<_, Option<f64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut quality_score = avg_quality_score
            .map(|score| score.round() as i64)
            .unwrap_or((readiness_score as i64 * 7) / 10);
        if published_artifact_count > 0 {
            quality_score += 600;
        }
        Ok(clamp_bp(quality_score))
    }

    #[allow(clippy::too_many_arguments)]
    fn upsert_topic_package_snapshot(
        &self,
        subject_id: i64,
        topic_id: i64,
        package_state: &str,
        live_health_state: &str,
        resource_readiness_score: BasisPoints,
        completeness_score: BasisPoints,
        quality_score: BasisPoints,
        evidence_score: BasisPoints,
        source_support_count: i64,
        contrast_pair_count: i64,
        publishable_artifact_count: i64,
        published_artifact_count: i64,
        missing_components: &[String],
        recommended_jobs: &[String],
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO topic_package_snapshots (
                    subject_id, topic_id, package_state, live_health_state,
                    resource_readiness_score, completeness_score, quality_score,
                    evidence_score, source_support_count, contrast_pair_count,
                    publishable_artifact_count, published_artifact_count,
                    missing_components_json, recommended_jobs_json, computed_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, datetime('now')
                 )
                 ON CONFLICT(subject_id, topic_id) DO UPDATE SET
                    package_state = excluded.package_state,
                    live_health_state = excluded.live_health_state,
                    resource_readiness_score = excluded.resource_readiness_score,
                    completeness_score = excluded.completeness_score,
                    quality_score = excluded.quality_score,
                    evidence_score = excluded.evidence_score,
                    source_support_count = excluded.source_support_count,
                    contrast_pair_count = excluded.contrast_pair_count,
                    publishable_artifact_count = excluded.publishable_artifact_count,
                    published_artifact_count = excluded.published_artifact_count,
                    missing_components_json = excluded.missing_components_json,
                    recommended_jobs_json = excluded.recommended_jobs_json,
                    computed_at = excluded.computed_at",
                params![
                    subject_id,
                    topic_id,
                    package_state,
                    live_health_state,
                    resource_readiness_score as i64,
                    completeness_score as i64,
                    quality_score as i64,
                    evidence_score as i64,
                    source_support_count,
                    contrast_pair_count,
                    publishable_artifact_count,
                    published_artifact_count,
                    serialize_json(&missing_components)?,
                    serialize_json(&recommended_jobs)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn queue_foundry_job_if_missing(
        &self,
        job_type: &str,
        trigger_type: &str,
        target_type: &str,
        target_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        priority: BasisPoints,
        dependency_refs: &[String],
        payload: &Value,
    ) -> EcoachResult<FoundryJob> {
        if let Some(existing) =
            self.find_active_foundry_job(job_type, target_type, target_id, subject_id, topic_id)?
        {
            return Ok(existing);
        }

        self.conn
            .execute(
                "INSERT INTO foundry_jobs (
                    job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                    priority, status, dependency_refs_json, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8, ?9)",
                params![
                    job_type,
                    trigger_type,
                    target_type,
                    target_id,
                    subject_id,
                    topic_id,
                    priority as i64,
                    serialize_json(&dependency_refs)?,
                    serialize_json(payload)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let job_id = self.conn.last_insert_rowid();
        self.append_runtime_event(
            "content_foundry.job_queued",
            "foundry_job",
            job_id.to_string(),
            json!({
                "job_type": job_type,
                "target_type": target_type,
                "target_id": target_id,
                "priority": priority,
            }),
        )?;
        self.get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::Storage("queued foundry job vanished".to_string()))
    }

    fn find_active_foundry_job(
        &self,
        job_type: &str,
        target_type: &str,
        target_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<Option<FoundryJob>> {
        self.conn
            .query_row(
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE job_type = ?1
                   AND target_type = ?2
                   AND target_id = ?3
                   AND ((subject_id = ?4) OR (subject_id IS NULL AND ?4 IS NULL))
                   AND ((topic_id = ?5) OR (topic_id IS NULL AND ?5 IS NULL))
                   AND status IN ('queued', 'running', 'blocked')
                 ORDER BY priority DESC, id DESC
                 LIMIT 1",
                params![job_type, target_type, target_id, subject_id, topic_id],
                map_foundry_job,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_foundry_job(&self, job_id: i64) -> EcoachResult<Option<FoundryJob>> {
        self.conn
            .query_row(
                "SELECT id, job_type, trigger_type, target_type, target_id, subject_id, topic_id,
                        priority, status, dependency_refs_json, payload_json, result_summary_json,
                        retry_count, failure_reason
                 FROM foundry_jobs
                 WHERE id = ?1",
                [job_id],
                map_foundry_job,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn update_foundry_job_status(
        &self,
        job_id: i64,
        status: &str,
        result_summary: Option<&Value>,
        failure_reason: Option<&str>,
        stamp_completed_at: bool,
        stamp_started_at: bool,
    ) -> EcoachResult<()> {
        let existing = self
            .get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job_id)))?;
        let result_summary_json =
            serialize_json(result_summary.unwrap_or(&existing.result_summary))?;
        self.conn
            .execute(
                "UPDATE foundry_jobs
                 SET status = ?1,
                     result_summary_json = ?2,
                     failure_reason = ?3,
                     updated_at = datetime('now'),
                     started_at = CASE WHEN ?4 = 1 THEN COALESCE(started_at, datetime('now')) ELSE started_at END,
                     completed_at = CASE WHEN ?5 = 1 THEN datetime('now') ELSE completed_at END,
                     retry_count = CASE WHEN ?1 = 'failed' THEN retry_count + 1 ELSE retry_count END
                 WHERE id = ?6",
                params![
                    status,
                    result_summary_json,
                    failure_reason,
                    if stamp_started_at { 1 } else { 0 },
                    if stamp_completed_at { 1 } else { 0 },
                    job_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(
            format!("content_foundry.job_{status}").as_str(),
            "foundry_job",
            job_id.to_string(),
            json!({
                "job_id": job_id,
                "status": status,
                "failure_reason": failure_reason,
            }),
        )?;
        Ok(())
    }

    fn subject_id_for_code(&self, subject_code: &str) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id FROM subjects WHERE code = ?1 LIMIT 1",
                [subject_code],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn append_runtime_event(
        &self,
        event_type: &str,
        aggregate_kind: &str,
        aggregate_id: String,
        payload: Value,
    ) -> EcoachResult<()> {
        let event = DomainEvent::new(event_type, aggregate_id.clone(), payload);
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.event_id,
                    event.event_type,
                    aggregate_kind,
                    aggregate_id,
                    event.trace_id,
                    serialize_json(&event.payload)?,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn map_source_upload(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumSourceUpload> {
    let metadata_json: String = row.get(14)?;
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
        confidence_score: row.get::<_, i64>(13)? as BasisPoints,
        metadata: parse_json_column(14, &metadata_json)?,
    })
}

fn map_parse_candidate(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumParseCandidate> {
    let payload_json: String = row.get(6)?;
    Ok(CurriculumParseCandidate {
        id: row.get(0)?,
        source_upload_id: row.get(1)?,
        candidate_type: row.get(2)?,
        parent_candidate_id: row.get(3)?,
        raw_label: row.get(4)?,
        normalized_label: row.get(5)?,
        payload: parse_json_column(6, &payload_json)?,
        confidence_score: row.get::<_, i64>(7)? as BasisPoints,
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

fn map_foundry_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<FoundryJob> {
    let dependency_refs_json: String = row.get(9)?;
    let payload_json: String = row.get(10)?;
    let result_summary_json: String = row.get(11)?;
    Ok(FoundryJob {
        id: row.get(0)?,
        job_type: row.get(1)?,
        trigger_type: row.get(2)?,
        target_type: row.get(3)?,
        target_id: row.get(4)?,
        subject_id: row.get(5)?,
        topic_id: row.get(6)?,
        priority: row.get::<_, i64>(7)? as BasisPoints,
        status: row.get(8)?,
        dependency_refs: serde_json::from_str::<Vec<String>>(&dependency_refs_json).map_err(
            |err| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            },
        )?,
        payload: parse_json_column(10, &payload_json)?,
        result_summary: parse_json_column(11, &result_summary_json)?,
        retry_count: row.get(12)?,
        failure_reason: row.get(13)?,
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

fn serialize_json(value: &impl Serialize) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
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

fn normalize_phrase(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn severity_for_confidence(confidence_score: BasisPoints) -> &'static str {
    if confidence_score < 4_500 {
        "high"
    } else {
        "medium"
    }
}

fn count_candidates_by_type(candidates: &[CurriculumParseCandidate]) -> Vec<ParseCandidateCount> {
    let mut counts = BTreeMap::<String, i64>::new();
    for candidate in candidates {
        *counts.entry(candidate.candidate_type.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(candidate_type, count)| ParseCandidateCount {
            candidate_type,
            count,
        })
        .collect()
}

fn count_duplicate_labels(candidates: &[CurriculumParseCandidate]) -> usize {
    let mut counts = BTreeMap::<(String, String), i64>::new();
    for candidate in candidates {
        let normalized = candidate
            .normalized_label
            .clone()
            .unwrap_or_else(|| candidate.raw_label.clone());
        let key = (
            candidate.candidate_type.clone(),
            normalize_phrase(&normalized),
        );
        *counts.entry(key).or_insert(0) += 1;
    }
    counts.values().filter(|count| **count > 1).count()
}

fn compute_publish_readiness_score(
    source_upload: &CurriculumSourceUpload,
    approved_candidate_count: i64,
    unresolved_review_count: i64,
    duplicate_label_count: i64,
    has_ready_job: bool,
    has_published_job: bool,
) -> BasisPoints {
    let mut score = source_upload.confidence_score as i64 / 2;
    if matches!(
        source_upload.source_status.as_str(),
        "parsed" | "reviewed" | "published"
    ) {
        score += 2_000;
    }
    score += (approved_candidate_count.min(8) * 350).clamp(0, 2_000);
    if unresolved_review_count == 0 {
        score += 1_500;
    } else {
        score -= (unresolved_review_count * 400).min(1_600);
    }
    if duplicate_label_count == 0 {
        score += 500;
    } else {
        score -= (duplicate_label_count * 250).min(1_000);
    }
    if has_ready_job {
        score += 750;
    }
    if has_published_job {
        score += 1_000;
    }
    clamp_bp(score)
}

fn build_source_actions(
    parse_candidates: &[CurriculumParseCandidate],
    unresolved_review_count: i64,
    duplicate_label_count: i64,
    publish_jobs: &[ContentPublishJobReport],
    can_mark_reviewed: bool,
) -> Vec<String> {
    let mut actions = BTreeSet::new();
    if parse_candidates.is_empty() {
        actions.insert("rerun_structural_parse".to_string());
    }
    if parse_candidates
        .iter()
        .any(|candidate| candidate.confidence_score < LOW_CONFIDENCE_THRESHOLD)
    {
        actions.insert("review_low_confidence_nodes".to_string());
    }
    if duplicate_label_count > 0 {
        actions.insert("resolve_duplicate_candidates".to_string());
    }
    if unresolved_review_count > 0 {
        actions.insert("clear_review_queue".to_string());
    }
    if can_mark_reviewed {
        actions.insert("mark_source_reviewed".to_string());
    }
    if publish_jobs.is_empty() && can_mark_reviewed {
        actions.insert("stage_publish_job".to_string());
    }
    if publish_jobs
        .iter()
        .any(|job| job.job.status == "ready_to_publish")
    {
        actions.insert("publish_ready_artifact".to_string());
    }
    if publish_jobs.iter().any(|job| job.job.status == "published") {
        actions.insert("monitor_live_artifact".to_string());
    }
    actions.into_iter().collect()
}

fn compute_evidence_score(reviewed_source_count: i64, approved_evidence_count: i64) -> BasisPoints {
    clamp_bp((reviewed_source_count.min(3) * 1_500) + (approved_evidence_count.min(4) * 1_200))
}

fn compute_completeness_score(
    readiness_score: BasisPoints,
    evidence_score: BasisPoints,
    contrast_pair_count: i64,
    publishable_artifact_count: i64,
    published_artifact_count: i64,
) -> BasisPoints {
    let mut score = (readiness_score as i64 * 6) / 10;
    score += (evidence_score as i64 * 25) / 100;
    if contrast_pair_count > 0 {
        score += 500;
    }
    if publishable_artifact_count > 0 {
        score += 1_100;
    }
    if published_artifact_count > 0 {
        score += 1_100;
    }
    clamp_bp(score)
}

fn build_recommended_jobs(missing_components: &[String]) -> Vec<String> {
    let mut jobs = BTreeSet::new();
    for component in missing_components {
        match component.as_str() {
            "concept_atoms_missing"
            | "learning_objectives_missing"
            | "knowledge_graph_edges_missing" => {
                jobs.insert("curriculum_enrichment_job".to_string());
            }
            "misconception_map_missing" => {
                jobs.insert("misconception_build_job".to_string());
            }
            "question_family_missing" | "question_bank_missing" => {
                jobs.insert("question_generation_job".to_string());
            }
            "explanation_layer_missing" => {
                jobs.insert("note_build_job".to_string());
            }
            "formula_support_missing" => {
                jobs.insert("formula_pack_build_job".to_string());
            }
            "worked_example_missing" => {
                jobs.insert("worked_example_build_job".to_string());
            }
            "source_support_missing" => {
                jobs.insert("source_acquisition_job".to_string());
            }
            "contrast_pack_missing" => {
                jobs.insert("contrast_build_job".to_string());
            }
            "publish_candidate_missing" => {
                jobs.insert("publish_job".to_string());
            }
            "live_artifact_missing" => {
                jobs.insert("publish_activation_job".to_string());
            }
            "quality_gate_not_met" => {
                jobs.insert("quality_review_job".to_string());
            }
            _ => {}
        }
    }
    jobs.into_iter().collect()
}

fn derive_package_state(
    readiness_score: BasisPoints,
    completeness_score: BasisPoints,
    quality_score: BasisPoints,
    publishable_artifact_count: i64,
    published_artifact_count: i64,
) -> String {
    if readiness_score == 0 {
        "unseeded".to_string()
    } else if completeness_score < 3_500 {
        "foundation_seeded".to_string()
    } else if completeness_score < 5_500 {
        "partially_supported".to_string()
    } else if publishable_artifact_count == 0 && published_artifact_count == 0 {
        "content_building".to_string()
    } else if quality_score < 5_500 {
        "quality_weak".to_string()
    } else if quality_score < 7_000 {
        "quality_mixed".to_string()
    } else if published_artifact_count > 0 && completeness_score >= 8_000 {
        "live_strong".to_string()
    } else if published_artifact_count > 0 {
        "quality_stable".to_string()
    } else if publishable_artifact_count > 0 {
        "quality_strong".to_string()
    } else {
        "under_revision".to_string()
    }
}

fn derive_live_health_state(
    package_state: &str,
    completeness_score: BasisPoints,
    published_artifact_count: i64,
) -> String {
    if published_artifact_count == 0 {
        if completeness_score >= 5_500 {
            "under_revision".to_string()
        } else {
            package_state.to_string()
        }
    } else if completeness_score >= 8_000 {
        "live_strong".to_string()
    } else {
        "quality_stable".to_string()
    }
}

fn dedupe_and_sort(items: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for item in items.drain(..) {
        seen.insert(item);
    }
    items.extend(seen);
}

fn map_source_action_to_job_type(action: &str) -> Option<&'static str> {
    match action {
        "review_low_confidence_nodes" => Some("source_review_job"),
        "resolve_duplicate_candidates" => Some("duplicate_resolution_job"),
        "mark_source_reviewed" => Some("source_approval_job"),
        "stage_publish_job" => Some("publish_job"),
        "publish_ready_artifact" => Some("publish_activation_job"),
        _ => None,
    }
}

fn topic_priority(
    completeness_score: BasisPoints,
    quality_score: BasisPoints,
    job_type: &str,
) -> BasisPoints {
    let mut priority =
        clamp_bp(10_000 - (((completeness_score as i64) + (quality_score as i64)) / 2));
    if matches!(job_type, "publish_job" | "publish_activation_job") {
        priority = clamp_bp(priority as i64 + 1_000);
    }
    priority
}

fn source_priority(publish_readiness_score: BasisPoints, action: &str) -> BasisPoints {
    let mut priority = clamp_bp(10_000 - publish_readiness_score as i64);
    if matches!(action, "stage_publish_job" | "publish_ready_artifact") {
        priority = clamp_bp(priority as i64 + 1_000);
    }
    priority
}

fn dependencies_for_topic_job(job_type: &str, snapshot: &TopicPackageSnapshot) -> Vec<String> {
    let mut dependencies = Vec::new();
    if matches!(job_type, "publish_job" | "publish_activation_job") {
        if snapshot
            .missing_components
            .iter()
            .any(|item| item == "source_support_missing")
        {
            dependencies.push("source_acquisition_job".to_string());
        }
        for needed in &[
            "note_build_job",
            "question_generation_job",
            "worked_example_build_job",
            "formula_pack_build_job",
            "contrast_build_job",
        ] {
            if snapshot.recommended_jobs.iter().any(|job| job == needed) {
                dependencies.push((*needed).to_string());
            }
        }
    } else if job_type == "quality_review_job" && snapshot.publishable_artifact_count == 0 {
        dependencies.push("publish_job".to_string());
    }
    dependencies
}

fn dependencies_for_source_job(action: &str, report: &ContentFoundrySourceReport) -> Vec<String> {
    let mut dependencies = Vec::new();
    match action {
        "mark_source_reviewed" => {
            if report.unresolved_review_count > 0 {
                dependencies.push("source_review_job".to_string());
                dependencies.push("duplicate_resolution_job".to_string());
            }
        }
        "stage_publish_job" => {
            if !report.can_mark_reviewed {
                dependencies.push("source_approval_job".to_string());
            }
        }
        "publish_ready_artifact" => {
            dependencies.push("publish_job".to_string());
        }
        _ => {}
    }
    dependencies
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use crate::{ContentPublishService, PackService};

    use super::*;

    #[test]
    fn foundry_source_report_builds_review_queue() {
        let conn = open_test_database();
        seed_admin(&conn);
        let service = FoundryCoordinatorService::new(&conn);
        let source = service
            .register_source_upload(SourceUploadInput {
                uploader_account_id: 1,
                source_kind: "curriculum".to_string(),
                title: "Math Curriculum Draft".to_string(),
                source_path: Some("C:/curriculum/math-draft.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some("MATH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("v-next".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");

        for (candidate_type, raw_label, confidence) in [
            ("subject", "Mathematics", 9200),
            ("topic", "Fractions", 5900),
            ("topic", "Fractions", 8400),
        ] {
            service
                .add_parse_candidate(
                    source.id,
                    ParseCandidateInput {
                        candidate_type: candidate_type.to_string(),
                        parent_candidate_id: None,
                        raw_label: raw_label.to_string(),
                        normalized_label: Some(raw_label.to_ascii_lowercase()),
                        payload: json!({ "page": 1 }),
                        confidence_score: confidence,
                    },
                )
                .expect("parse candidate should insert");
        }

        let report = service
            .finalize_source_parse(source.id)
            .expect("parse finalization should succeed");

        assert_eq!(report.source_upload.source_status, "review_required");
        assert!(report.unresolved_review_count >= 2);
        assert!(
            report
                .review_tasks
                .iter()
                .any(|task| task.task_type == "normalization")
        );
        assert!(
            report
                .review_tasks
                .iter()
                .any(|task| task.task_type == "duplicate_check")
        );
    }

    #[test]
    fn foundry_stages_publish_jobs_and_persists_topic_health() {
        let conn = open_test_database();
        seed_admin(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = FoundryCoordinatorService::new(&conn);
        let source = service
            .register_source_upload(SourceUploadInput {
                uploader_account_id: 1,
                source_kind: "curriculum".to_string(),
                title: "Math Curriculum v2".to_string(),
                source_path: Some("C:/curriculum/math-v2.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some("MATH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("v2".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");

        for (candidate_type, raw_label) in [
            ("subject", "Mathematics"),
            ("topic", "Fractions"),
            ("objective", "Identify equivalent fractions"),
        ] {
            service
                .add_parse_candidate(
                    source.id,
                    ParseCandidateInput {
                        candidate_type: candidate_type.to_string(),
                        parent_candidate_id: None,
                        raw_label: raw_label.to_string(),
                        normalized_label: Some(raw_label.to_ascii_lowercase()),
                        payload: json!({ "page": 1 }),
                        confidence_score: 9000,
                    },
                )
                .expect("parse candidate should insert");
        }
        service
            .finalize_source_parse(source.id)
            .expect("parse should finalize");
        service
            .mark_source_reviewed(source.id)
            .expect("source should become reviewed");

        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE code = 'FRA' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("fractions topic should exist");
        conn.execute(
            "INSERT INTO content_acquisition_jobs (
                subject_id, topic_id, intent_type, query_text, source_scope, status, result_summary_json, completed_at
             ) VALUES (?1, ?2, 'gap_fill', 'fractions note evidence', 'internal', 'completed', '{}', datetime('now'))",
            params![1, topic_id],
        )
        .expect("acquisition job should insert");
        let acquisition_job_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO acquisition_evidence_candidates (
                job_id, source_label, source_url, source_kind, title, snippet,
                extracted_payload_json, quality_score, freshness_score, review_status
             ) VALUES (?1, 'Teacher Guide', NULL, 'internal', 'Fractions Guide', 'Aligned support', '{}', 8400, 7800, 'approved')",
            [acquisition_job_id],
        )
        .expect("approved evidence should insert");

        let publish_report = service
            .stage_publish_job(source.id, Some(1), Some(1), Some(topic_id), Some("v2"))
            .expect("publish job should stage");
        assert_eq!(publish_report.job.status, "ready_to_publish");

        ContentPublishService::new(&conn)
            .mark_published(
                publish_report.job.id,
                &json!({ "topic_id": topic_id, "artifact_type": "topic_package" }),
            )
            .expect("publish job should publish");

        let snapshot = service
            .recompute_topic_package_snapshot(topic_id)
            .expect("snapshot should recompute")
            .expect("snapshot should exist");
        let topic_jobs = service
            .queue_topic_foundry_jobs(topic_id, "snapshot_refresh")
            .expect("topic jobs should queue");
        let source_jobs = service
            .queue_source_follow_up_jobs(source.id, "source_review")
            .expect("source jobs should queue");
        let running_job = service
            .start_foundry_job(topic_jobs[0].id)
            .expect("foundry job should start");
        let completed_job = service
            .complete_foundry_job(running_job.id, &json!({ "artifacts_built": 1 }))
            .expect("foundry job should complete");
        let failed_job = service
            .fail_foundry_job(source_jobs[0].id, "manual reviewer blocked publish")
            .expect("foundry job should fail");
        let board = service
            .get_foundry_job_board(Some(1))
            .expect("job board should load");
        let dashboard = service
            .get_subject_foundry_dashboard(1)
            .expect("dashboard should load")
            .expect("dashboard should exist");

        assert!(snapshot.publishable_artifact_count >= 1);
        assert!(snapshot.published_artifact_count >= 1);
        assert!(snapshot.source_support_count >= 2);
        assert!(!topic_jobs.is_empty());
        assert!(!source_jobs.is_empty());
        assert_eq!(completed_job.status, "completed");
        assert_eq!(failed_job.status, "failed");
        assert!(board.failed_count >= 1);
        assert!(board.completed_count >= 1);
        assert!(
            dashboard
                .topics
                .iter()
                .any(|topic| topic.topic_id == topic_id && topic.published_artifact_count >= 1)
        );
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn seed_admin(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'admin', 'Admin', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("admin should insert");
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
