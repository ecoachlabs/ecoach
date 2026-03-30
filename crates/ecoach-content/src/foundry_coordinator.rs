use std::collections::{BTreeMap, BTreeSet};

use ecoach_substrate::{
    BasisPoints, DomainEvent, EcoachError, EcoachResult, EngineRegistry,
    FabricOrchestrationSummary, FabricSignal, clamp_bp, now_utc,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    ContentPublishJobReport, ContentPublishService, ResourceReadinessService, pack_service::slugify,
};

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
    #[serde(default)]
    pub fabric_signals: Vec<FabricSignal>,
    #[serde(default)]
    pub orchestration: FabricOrchestrationSummary,
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
    #[serde(default)]
    pub fabric_signals: Vec<FabricSignal>,
    #[serde(default)]
    pub orchestration: FabricOrchestrationSummary,
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
    #[serde(default)]
    pub fabric_signals: Vec<FabricSignal>,
    #[serde(default)]
    pub orchestration: FabricOrchestrationSummary,
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

#[derive(Debug, Clone)]
struct TopicFoundryContext {
    subject_id: i64,
    topic_id: i64,
    topic_code: Option<String>,
    topic_name: String,
    approved_candidates: Vec<CurriculumParseCandidate>,
    approved_evidence: Vec<ApprovedAcquisitionEvidence>,
}

#[derive(Debug, Clone)]
struct ApprovedAcquisitionEvidence {
    title: Option<String>,
    snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AutoReviewTaskSpec {
    candidate_id: Option<i64>,
    task_type: String,
    severity: String,
    notes: String,
}

impl AutoReviewTaskSpec {
    fn new(candidate_id: Option<i64>, task_type: &str, severity: &str, notes: String) -> Self {
        Self {
            candidate_id,
            task_type: task_type.to_string(),
            severity: severity.to_string(),
            notes,
        }
    }
}

#[derive(Debug, Clone)]
struct KnowledgeDraft {
    title: String,
    canonical_name: String,
    short_text: String,
    full_text: String,
    technical_text: String,
    exam_text: String,
    importance_score: BasisPoints,
    difficulty_level: BasisPoints,
    aliases: Vec<String>,
}

#[derive(Debug, Clone)]
struct TopicKnowledgeEntry {
    id: i64,
    entry_type: String,
    title: String,
    importance_score: BasisPoints,
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
        let mut desired_auto_tasks = Vec::new();
        if candidates.is_empty() {
            desired_auto_tasks.push(AutoReviewTaskSpec::new(
                None,
                "publish_gate",
                "high",
                "No parse candidates were extracted. Re-run parsing before review.".to_string(),
            ));
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
                self.set_candidate_review_status(candidate.id, "pending")?;
                desired_auto_tasks.push(AutoReviewTaskSpec::new(
                    Some(candidate.id),
                    "normalization",
                    severity_for_confidence(candidate.confidence_score),
                    format!(
                        "Candidate '{}' ({}) needs normalization review at {} bp confidence.",
                        candidate.raw_label, candidate.candidate_type, candidate.confidence_score
                    ),
                ));
            } else {
                self.set_candidate_review_status(candidate.id, "approved")?;
            }
        }

        for ((candidate_type, normalized_label), count) in duplicate_counts {
            if count > 1 {
                desired_auto_tasks.push(AutoReviewTaskSpec::new(
                    None,
                    "duplicate_check",
                    "medium",
                    format!(
                        "Duplicate candidate label detected for {}:{}.",
                        candidate_type, normalized_label
                    ),
                ));
            }
        }

        if !has_subject {
            desired_auto_tasks.push(AutoReviewTaskSpec::new(
                None,
                "hierarchy_fix",
                "high",
                "No subject node was extracted from this source.".to_string(),
            ));
        }
        if !has_topic {
            desired_auto_tasks.push(AutoReviewTaskSpec::new(
                None,
                "hierarchy_fix",
                "high",
                "No topic or subtopic nodes were extracted from this source.".to_string(),
            ));
        }
        if !has_objective {
            desired_auto_tasks.push(AutoReviewTaskSpec::new(
                None,
                "publish_gate",
                "medium",
                "No objective or skill nodes were extracted from this source.".to_string(),
            ));
        }

        self.sync_auto_review_tasks(source_upload_id, &desired_auto_tasks)?;

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
        let recommended_actions = build_source_actions(
            &parse_candidates,
            unresolved_review_count,
            duplicate_label_count,
            &publish_jobs,
            can_mark_reviewed,
        );
        let fabric_signals = build_source_fabric_signals(
            &source_upload,
            approved_candidate_count,
            unresolved_review_count,
            publish_readiness_score,
            &publish_jobs,
            &recommended_actions,
        );
        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            foundry_available_inputs(&fabric_signals),
        );

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
            recommended_actions,
            fabric_signals,
            orchestration,
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
            &[
                "gating",
                "review_required",
                "ready_to_publish",
                "publishing",
                "published",
            ],
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
        let fabric_signals = build_topic_fabric_signals(
            topic_id,
            topic_name.as_str(),
            &package_state,
            completeness_score,
            quality_score,
            &missing_components,
            &recommended_jobs,
            publishable_artifact_count,
            published_artifact_count,
        );
        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            foundry_available_inputs(&fabric_signals),
        );

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
            fabric_signals,
            orchestration,
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
        let fabric_signals = build_dashboard_fabric_signals(subject_id, &subject_code, &topics);
        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            foundry_available_inputs(&fabric_signals),
        );

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
            fabric_signals,
            orchestration,
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

    pub fn run_foundry_job(&self, job_id: i64) -> EcoachResult<FoundryJob> {
        let job = self
            .get_foundry_job(job_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job_id)))?;
        if job.status == "completed" {
            return Ok(job);
        }
        if job.status == "running" {
            return Ok(job);
        }

        let pending_dependencies = self.pending_foundry_dependencies(&job)?;
        if !pending_dependencies.is_empty() {
            let result_summary = json!({
                "blocked_on": pending_dependencies,
                "job_type": job.job_type,
                "target_type": job.target_type,
                "target_id": job.target_id,
            });
            self.update_foundry_job_status(
                job.id,
                "blocked",
                Some(&result_summary),
                Some("waiting for dependent foundry jobs"),
                false,
                false,
            )?;
            return self
                .get_foundry_job(job.id)?
                .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job.id)));
        }

        self.update_foundry_job_status(job.id, "running", None, None, false, true)?;
        let execution = self.execute_foundry_job(&job);
        match execution {
            Ok(result_summary) => {
                self.update_foundry_job_status(
                    job.id,
                    "completed",
                    Some(&result_summary),
                    None,
                    true,
                    false,
                )?;
            }
            Err(err) => {
                let error_message = err.to_string();
                self.update_foundry_job_status(
                    job.id,
                    "failed",
                    Some(&json!({
                        "error": error_message,
                        "job_type": job.job_type,
                        "target_type": job.target_type,
                        "target_id": job.target_id,
                    })),
                    Some(&error_message),
                    true,
                    false,
                )?;
            }
        }

        self.get_foundry_job(job.id)?
            .ok_or_else(|| EcoachError::NotFound(format!("foundry job {} not found", job.id)))
    }

    pub fn run_next_foundry_job(
        &self,
        subject_id: Option<i64>,
    ) -> EcoachResult<Option<FoundryJob>> {
        let mut candidate_jobs = self.list_foundry_jobs(None, None, subject_id)?;
        candidate_jobs.retain(|job| matches!(job.status.as_str(), "queued" | "blocked"));
        candidate_jobs.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.id.cmp(&right.id))
        });

        for job in candidate_jobs {
            let pending_dependencies = self.pending_foundry_dependencies(&job)?;
            if pending_dependencies.is_empty() {
                return self.run_foundry_job(job.id).map(Some);
            }
            let result_summary = json!({
                "blocked_on": pending_dependencies,
                "job_type": job.job_type,
                "target_type": job.target_type,
                "target_id": job.target_id,
            });
            self.update_foundry_job_status(
                job.id,
                "blocked",
                Some(&result_summary),
                Some("waiting for dependent foundry jobs"),
                false,
                false,
            )?;
        }
        Ok(None)
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

    fn sync_auto_review_tasks(
        &self,
        source_upload_id: i64,
        desired_tasks: &[AutoReviewTaskSpec],
    ) -> EcoachResult<()> {
        let desired_tasks = desired_tasks.iter().cloned().collect::<BTreeSet<_>>();
        let desired_keys = desired_tasks
            .iter()
            .map(|task| auto_review_task_key(task.candidate_id, &task.task_type, &task.notes))
            .collect::<BTreeSet<_>>();

        for task in self.list_review_tasks(source_upload_id)? {
            if task.status == "resolved" || !is_auto_review_task_type(&task.task_type) {
                continue;
            }
            let key = auto_review_task_key(
                task.candidate_id,
                &task.task_type,
                task.notes.as_deref().unwrap_or(""),
            );
            if !desired_keys.contains(&key) {
                self.auto_resolve_review_task(task.id, task.notes.as_deref())?;
            }
        }

        for task in desired_tasks {
            self.ensure_review_task(
                source_upload_id,
                task.candidate_id,
                &task.task_type,
                &task.severity,
                &task.notes,
            )?;
        }

        Ok(())
    }

    fn auto_resolve_review_task(
        &self,
        task_id: i64,
        existing_notes: Option<&str>,
    ) -> EcoachResult<()> {
        let notes = append_review_task_note(
            existing_notes,
            "Auto-resolution: latest parse refresh no longer requires this task.",
        );
        self.conn
            .execute(
                "UPDATE curriculum_review_tasks
                 SET status = 'resolved', notes = ?1, resolved_at = datetime('now')
                 WHERE id = ?2",
                params![notes, task_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
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

    fn load_topic_foundry_context(&self, topic_id: i64) -> EcoachResult<TopicFoundryContext> {
        let (subject_id, subject_code, topic_code, topic_name) = self
            .conn
            .query_row(
                "SELECT t.subject_id, s.code, t.code, t.name
                 FROM topics t
                 JOIN subjects s ON s.id = t.subject_id
                 WHERE t.id = ?1
                 LIMIT 1",
                [topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let approved_candidates = self.list_approved_parse_candidates_for_topic(
            &subject_code,
            topic_id,
            topic_code.as_deref(),
            &topic_name,
        )?;
        let approved_evidence = self.list_approved_evidence_for_topic(topic_id)?;

        Ok(TopicFoundryContext {
            subject_id,
            topic_id,
            topic_code,
            topic_name,
            approved_candidates,
            approved_evidence,
        })
    }

    fn list_approved_parse_candidates_for_topic(
        &self,
        subject_code: &str,
        topic_id: i64,
        topic_code: Option<&str>,
        topic_name: &str,
    ) -> EcoachResult<Vec<CurriculumParseCandidate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT candidates.id, candidates.source_upload_id, candidates.candidate_type,
                        candidates.parent_candidate_id, candidates.raw_label,
                        candidates.normalized_label, candidates.payload_json,
                        candidates.confidence_score, candidates.review_status
                 FROM curriculum_parse_candidates candidates
                 JOIN curriculum_source_uploads uploads
                   ON uploads.id = candidates.source_upload_id
                 WHERE uploads.subject_code = ?1
                   AND uploads.source_status IN ('reviewed', 'published')
                   AND candidates.review_status = 'approved'
                 ORDER BY candidates.confidence_score DESC, candidates.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_code], map_parse_candidate)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut candidates = collect_rows(rows)?;
        candidates.retain(|candidate| {
            candidate_matches_topic(candidate, topic_id, topic_code, topic_name)
        });
        candidates.sort_by(|left, right| {
            right
                .confidence_score
                .cmp(&left.confidence_score)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(candidates)
    }

    fn list_approved_evidence_for_topic(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Vec<ApprovedAcquisitionEvidence>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT candidates.title, candidates.snippet
                 FROM acquisition_evidence_candidates candidates
                 JOIN content_acquisition_jobs jobs ON jobs.id = candidates.job_id
                 WHERE jobs.topic_id = ?1
                   AND candidates.review_status = 'approved'
                 ORDER BY candidates.quality_score DESC,
                          candidates.freshness_score DESC,
                          candidates.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| {
                Ok(ApprovedAcquisitionEvidence {
                    title: row.get(0)?,
                    snippet: row.get(1)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
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
                   AND jobs.status IN (
                        'gating',
                        'review_required',
                        'ready_to_publish',
                        'publishing',
                        'published'
                   )",
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

    fn execute_foundry_job(&self, job: &FoundryJob) -> EcoachResult<Value> {
        match job.job_type.as_str() {
            "source_review_job" | "duplicate_resolution_job" => {
                let report = self.finalize_source_parse(job.target_id)?;
                if report.unresolved_review_count > 0 {
                    return Ok(json!({
                        "execution_state": "manual_review_required",
                        "source_upload_id": job.target_id,
                        "unresolved_review_count": report.unresolved_review_count,
                        "recommended_actions": report.recommended_actions,
                    }));
                }
                let queued_follow_up_jobs =
                    self.queue_source_follow_up_jobs(job.target_id, "job_execution")?;
                Ok(json!({
                    "execution_state": "source_parsed",
                    "source_upload_id": job.target_id,
                    "source_status": report.source_upload.source_status,
                    "approved_candidate_count": report.approved_candidate_count,
                    "queued_follow_up_jobs": queued_follow_up_jobs.iter().map(|item| item.job_type.clone()).collect::<Vec<_>>(),
                }))
            }
            "source_approval_job" => {
                let report = self.get_source_report(job.target_id)?.ok_or_else(|| {
                    EcoachError::NotFound(format!("source upload {} not found", job.target_id))
                })?;
                if !report.can_mark_reviewed {
                    return Err(EcoachError::Validation(format!(
                        "source upload {} still has unresolved review work",
                        job.target_id
                    )));
                }
                let reviewed_report = self.mark_source_reviewed(job.target_id)?;
                let queued_jobs =
                    self.queue_source_follow_up_jobs(job.target_id, "job_execution")?;
                Ok(json!({
                    "execution_state": "source_reviewed",
                    "source_upload_id": job.target_id,
                    "source_status": reviewed_report.source_upload.source_status,
                    "queued_follow_up_jobs": queued_jobs.iter().map(|item| item.job_type.clone()).collect::<Vec<_>>(),
                }))
            }
            "source_acquisition_job" => {
                let topic_id = job.topic_id.ok_or_else(|| {
                    EcoachError::Validation(
                        "source acquisition job requires a topic target".to_string(),
                    )
                })?;
                let subject_id = job.subject_id.ok_or_else(|| {
                    EcoachError::Validation(
                        "source acquisition job requires a subject target".to_string(),
                    )
                })?;
                let acquisition_job_id =
                    self.seed_acquisition_support(subject_id, topic_id, &job.payload)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                let queued_follow_up_jobs =
                    self.queue_topic_foundry_jobs(topic_id, "job_execution")?;
                Ok(json!({
                    "execution_state": "support_seeded",
                    "topic_id": topic_id,
                    "acquisition_job_id": acquisition_job_id,
                    "snapshot": snapshot,
                    "queued_follow_up_jobs": queued_follow_up_jobs.iter().map(|item| item.job_type.clone()).collect::<Vec<_>>(),
                }))
            }
            "curriculum_enrichment_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "curriculum enrichment job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_curriculum_support(topic_id)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "curriculum_support_seeded",
                    "topic_id": topic_id,
                    "seeded_node_count": seeded["node_count"],
                    "seeded_objective_count": seeded["objective_count"],
                    "seeded_edge_count": seeded["edge_count"],
                    "snapshot": snapshot,
                }))
            }
            "misconception_build_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "misconception build job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_misconception_support(topic_id)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "misconception_support_seeded",
                    "topic_id": topic_id,
                    "misconception_id": seeded,
                    "snapshot": snapshot,
                }))
            }
            "note_build_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "note build job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_knowledge_support(topic_id, "explanation")?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "note_support_seeded",
                    "topic_id": topic_id,
                    "entry_id": seeded,
                    "snapshot": snapshot,
                }))
            }
            "formula_pack_build_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "formula build job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_knowledge_support(topic_id, "formula")?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "formula_support_seeded",
                    "topic_id": topic_id,
                    "entry_id": seeded,
                    "snapshot": snapshot,
                }))
            }
            "worked_example_build_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "worked example build job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_knowledge_support(topic_id, "worked_example")?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "worked_example_seeded",
                    "topic_id": topic_id,
                    "entry_id": seeded,
                    "snapshot": snapshot,
                }))
            }
            "question_generation_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "question generation job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_question_support(topic_id)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "question_support_seeded",
                    "topic_id": topic_id,
                    "family_id": seeded["family_id"],
                    "question_id": seeded["question_id"],
                    "snapshot": snapshot,
                }))
            }
            "contrast_build_job" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "contrast build job requires a topic target".to_string(),
                        )
                    })?;
                let seeded = self.seed_contrast_support(topic_id)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "contrast_support_seeded",
                    "topic_id": topic_id,
                    "pair_id": seeded,
                    "snapshot": snapshot,
                }))
            }
            "publish_job" => {
                let publish_report = self.stage_publish_for_job(job)?;
                if let Some(topic_id) = publish_report.job.topic_id {
                    let _ = self.recompute_topic_package_snapshot(topic_id)?;
                    let _ = self.queue_topic_foundry_jobs(topic_id, "job_execution");
                }
                Ok(json!({
                    "execution_state": "publish_staged",
                    "publish_job_id": publish_report.job.id,
                    "publish_status": publish_report.job.status,
                    "blocking_report_count": publish_report.blocking_report_count,
                }))
            }
            "quality_review_job" => {
                let topic_id = job.topic_id.ok_or_else(|| {
                    EcoachError::Validation(
                        "quality review job requires a topic target".to_string(),
                    )
                })?;
                let publish_report = self.run_quality_review_for_topic(topic_id)?;
                let _ = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "quality_reviewed",
                    "topic_id": topic_id,
                    "publish_job_id": publish_report.job.id,
                    "publish_status": publish_report.job.status,
                    "blocking_report_count": publish_report.blocking_report_count,
                }))
            }
            "publish_activation_job" => {
                let topic_id = job.topic_id.ok_or_else(|| {
                    EcoachError::Validation(
                        "publish activation job requires a topic target".to_string(),
                    )
                })?;
                let publish_job = self.activate_latest_publish_job(topic_id)?;
                let snapshot = self.recompute_topic_package_snapshot(topic_id)?;
                Ok(json!({
                    "execution_state": "published",
                    "topic_id": topic_id,
                    "publish_job_id": publish_job.id,
                    "publish_status": publish_job.status,
                    "snapshot": snapshot,
                }))
            }
            other => Ok(json!({
                "execution_state": "queued_for_manual_backend_work",
                "job_type": other,
                "target_type": job.target_type,
                "target_id": job.target_id,
            })),
        }
    }

    fn pending_foundry_dependencies(&self, job: &FoundryJob) -> EcoachResult<Vec<String>> {
        let mut pending = Vec::new();
        for dependency in &job.dependency_refs {
            let status: Option<String> = self
                .conn
                .query_row(
                    "SELECT status
                     FROM foundry_jobs
                     WHERE job_type = ?1
                       AND target_type = ?2
                       AND target_id = ?3
                       AND ((subject_id = ?4) OR (subject_id IS NULL AND ?4 IS NULL))
                       AND ((topic_id = ?5) OR (topic_id IS NULL AND ?5 IS NULL))
                     ORDER BY created_at DESC, id DESC
                     LIMIT 1",
                    params![
                        dependency,
                        job.target_type,
                        job.target_id,
                        job.subject_id,
                        job.topic_id,
                    ],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !matches!(status.as_deref(), Some("completed")) {
                pending.push(dependency.clone());
            }
        }
        Ok(pending)
    }

    fn stage_publish_for_job(&self, job: &FoundryJob) -> EcoachResult<ContentPublishJobReport> {
        match job.target_type.as_str() {
            "source_upload" => self.stage_publish_job(
                job.target_id,
                None,
                job.subject_id,
                job.topic_id,
                Some("auto-job"),
            ),
            "topic_package" => {
                let topic_id = job
                    .topic_id
                    .or_else(|| job.payload["topic_id"].as_i64())
                    .ok_or_else(|| {
                        EcoachError::Validation(
                            "topic package publish job requires topic_id".to_string(),
                        )
                    })?;
                let source_upload_id = self
                    .latest_reviewed_source_upload_for_subject(job.subject_id)?
                    .ok_or_else(|| {
                        EcoachError::Validation(format!(
                            "no reviewed source upload available to stage publish for topic {}",
                            topic_id
                        ))
                    })?;
                self.stage_publish_job(
                    source_upload_id,
                    None,
                    job.subject_id,
                    Some(topic_id),
                    Some("auto-job"),
                )
            }
            other => Err(EcoachError::Validation(format!(
                "unsupported foundry publish target type: {}",
                other
            ))),
        }
    }

    fn run_quality_review_for_topic(&self, topic_id: i64) -> EcoachResult<ContentPublishJobReport> {
        let publish_job = self
            .latest_publish_job_for_topic(
                topic_id,
                &[
                    "gating",
                    "review_required",
                    "ready_to_publish",
                    "publishing",
                ],
            )?
            .ok_or_else(|| {
                EcoachError::Validation(format!(
                    "no active publish job available for topic {} quality review",
                    topic_id
                ))
            })?;
        let snapshot = self
            .recompute_topic_package_snapshot(topic_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("topic snapshot {} not found", topic_id))
            })?;
        let metrics = json!({
            "resource_readiness_score": snapshot.resource_readiness_score,
            "completeness_score": snapshot.completeness_score,
            "quality_score": snapshot.quality_score,
            "evidence_score": snapshot.evidence_score,
            "missing_components": snapshot.missing_components,
        });
        let report_type = if snapshot.quality_score >= 8_000 {
            "auto_quality_gate_strong"
        } else {
            "auto_quality_gate_review"
        };
        let status = if snapshot.quality_score >= 6_500 && snapshot.publishable_artifact_count > 0 {
            "pass"
        } else {
            "needs_review"
        };
        ContentPublishService::new(self.conn).add_quality_report(
            publish_job.id,
            report_type,
            status,
            snapshot.quality_score as i64,
            &metrics,
        )?;
        if status == "pass" {
            ContentPublishService::new(self.conn).mark_ready_to_publish(
                publish_job.id,
                &json!({
                    "topic_id": topic_id,
                    "artifact_type": "topic_package",
                    "quality_gate": report_type,
                }),
            )?;
        }
        ContentPublishService::new(self.conn)
            .get_publish_job_report(publish_job.id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("publish job {} not found", publish_job.id))
            })
    }

    fn activate_latest_publish_job(&self, topic_id: i64) -> EcoachResult<crate::ContentPublishJob> {
        let publish_job = self
            .latest_publish_job_for_topic(topic_id, &["ready_to_publish", "published"])?
            .ok_or_else(|| {
                EcoachError::Validation(format!(
                    "no ready-to-publish job available for topic {}",
                    topic_id
                ))
            })?;
        if publish_job.status != "published" {
            ContentPublishService::new(self.conn).mark_published(
                publish_job.id,
                &json!({
                    "topic_id": topic_id,
                    "artifact_type": "topic_package",
                    "publish_mode": "auto_foundry_activation",
                }),
            )?;
        }
        ContentPublishService::new(self.conn)
            .get_publish_job(publish_job.id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("publish job {} not found", publish_job.id))
            })
    }

    fn latest_publish_job_for_topic(
        &self,
        topic_id: i64,
        statuses: &[&str],
    ) -> EcoachResult<Option<crate::ContentPublishJob>> {
        let jobs = ContentPublishService::new(self.conn).list_publish_jobs(None)?;
        Ok(jobs
            .into_iter()
            .filter(|job| {
                job.topic_id == Some(topic_id)
                    && statuses.iter().any(|status| *status == job.status)
            })
            .max_by_key(|job| job.id))
    }

    fn latest_reviewed_source_upload_for_subject(
        &self,
        subject_id: Option<i64>,
    ) -> EcoachResult<Option<i64>> {
        let Some(subject_id) = subject_id else {
            return Ok(None);
        };
        let subject_code: String = self
            .conn
            .query_row(
                "SELECT code FROM subjects WHERE id = ?1 LIMIT 1",
                [subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .query_row(
                "SELECT id
                 FROM curriculum_source_uploads
                 WHERE subject_code = ?1
                   AND source_status = 'reviewed'
                 ORDER BY updated_at DESC, id DESC
                 LIMIT 1",
                [subject_code],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn seed_acquisition_support(
        &self,
        subject_id: i64,
        topic_id: i64,
        payload: &Value,
    ) -> EcoachResult<i64> {
        let topic_context = self.load_topic_foundry_context(topic_id).ok();
        let topic_name = payload["topic_name"]
            .as_str()
            .map(ToString::to_string)
            .or_else(|| {
                topic_context
                    .as_ref()
                    .map(|context| context.topic_name.clone())
            })
            .or_else(|| self.topic_identity(topic_id).ok().map(|(_, name)| name))
            .unwrap_or_else(|| "topic".to_string());
        let result_summary_json = serialize_json(&json!({
            "execution_mode": "auto_foundry_seed",
            "topic_id": topic_id,
            "topic_name": topic_name,
        }))?;
        self.conn
            .execute(
                "INSERT INTO content_acquisition_jobs (
                    subject_id, topic_id, intent_type, query_text, source_scope,
                    status, result_summary_json, completed_at
                 ) VALUES (?1, ?2, 'gap_fill', ?3, 'internal', 'completed', ?4, datetime('now'))",
                params![
                    subject_id,
                    topic_id,
                    format!("Auto foundry evidence seed for {}", topic_name),
                    result_summary_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let acquisition_job_id = self.conn.last_insert_rowid();
        let mut seeded_any = false;
        if let Some(context) = topic_context.as_ref() {
            for candidate in context
                .approved_candidates
                .iter()
                .filter(|candidate| candidate.candidate_type != "subject")
                .take(3)
            {
                let title = format!(
                    "{} evidence from {}",
                    candidate_display_label(candidate),
                    humanize_candidate_type(&candidate.candidate_type)
                );
                let snippet = payload_string(
                    &candidate.payload,
                    &[
                        "snippet",
                        "definition",
                        "description",
                        "simple_text",
                        "exam_hint",
                    ],
                )
                .unwrap_or_else(|| {
                    format!(
                        "Reviewed {} support for {} in {}.",
                        candidate.candidate_type,
                        candidate_display_label(candidate),
                        context.topic_name
                    )
                });
                self.conn
                    .execute(
                        "INSERT INTO acquisition_evidence_candidates (
                            job_id, source_label, source_url, source_kind, title, snippet,
                            extracted_payload_json, quality_score, freshness_score, review_status
                         ) VALUES (?1, ?2, NULL, 'internal', ?3, ?4, ?5, ?6, 7600, 'approved')",
                        params![
                            acquisition_job_id,
                            "Foundry Auto Support",
                            title,
                            snippet,
                            serialize_json(&json!({
                                "topic_id": topic_id,
                                "topic_name": context.topic_name,
                                "candidate_id": candidate.id,
                                "candidate_type": candidate.candidate_type,
                                "seeded_by": "foundry_job_executor",
                            }))?,
                            candidate.confidence_score as i64,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                seeded_any = true;
            }
        }

        if !seeded_any {
            self.conn
                .execute(
                    "INSERT INTO acquisition_evidence_candidates (
                        job_id, source_label, source_url, source_kind, title, snippet,
                        extracted_payload_json, quality_score, freshness_score, review_status
                     ) VALUES (?1, ?2, NULL, 'internal', ?3, ?4, ?5, 8300, 7600, 'approved')",
                    params![
                        acquisition_job_id,
                        "Foundry Auto Support",
                        format!("{} support seed", topic_name),
                        "Auto-approved internal support evidence",
                        serialize_json(&json!({
                            "topic_id": topic_id,
                            "topic_name": topic_name,
                            "seeded_by": "foundry_job_executor",
                        }))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(acquisition_job_id)
    }

    fn seed_curriculum_support(&self, topic_id: i64) -> EcoachResult<Value> {
        let context = self.load_topic_foundry_context(topic_id)?;
        let (node_id, node_inserted) = self.ensure_primary_node(topic_id, &context.topic_name)?;
        let (candidate_node_count, candidate_edge_count) =
            self.materialize_candidate_nodes(&context, node_id)?;
        let objective_inserted = self.materialize_learning_objectives(&context)?;

        let edge_inserted = if self.count_topic_edges(topic_id)? == 0 {
            self.ensure_node_edge(topic_id, "topic", node_id, "academic_node", "part_of", 7800)?
        } else {
            0
        };

        Ok(json!({
            "node_count": if node_inserted { 1 } else { 0 } + candidate_node_count,
            "objective_count": objective_inserted,
            "edge_count": edge_inserted + candidate_edge_count,
            "node_id": node_id,
            "source_candidate_count": context.approved_candidates.len(),
        }))
    }

    fn seed_misconception_support(&self, topic_id: i64) -> EcoachResult<i64> {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM misconception_patterns
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY severity DESC, id ASC
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok(existing);
        }

        let context = self.load_topic_foundry_context(topic_id)?;
        let topic_name = context.topic_name.clone();
        let (node_id, _) = self.ensure_primary_node(topic_id, &topic_name)?;
        let misconception_focus = self
            .best_candidate_label(&context, &["concept", "formula", "skill", "objective"])
            .unwrap_or_else(|| topic_name.clone());
        let misconception_hint = self
            .top_evidence_snippet(&context)
            .unwrap_or_else(|| format!("Reviewed source support exists for {}.", topic_name));
        self.conn
            .execute(
                "INSERT INTO misconception_patterns (
                    node_id, topic_id, title, misconception_statement, cause_type,
                    wrong_answer_pattern, correction_hint, severity
                 ) VALUES (?1, ?2, ?3, ?4, 'overgeneralization', ?5, ?6, 7600)",
                params![
                    node_id,
                    topic_id,
                    format!("{} surface-rule confusion", misconception_focus),
                    format!(
                        "The learner may be applying a shallow rule for {} without checking the underlying meaning in {}.",
                        misconception_focus, topic_name
                    ),
                    format!("Common wrong answer pattern around {}.", misconception_focus),
                    format!(
                        "Contrast the correct condition for {} with a near-miss example. {}",
                        misconception_focus, misconception_hint
                    ),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn seed_knowledge_support(&self, topic_id: i64, entry_type: &str) -> EcoachResult<i64> {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM knowledge_entries
                 WHERE topic_id = ?1 AND entry_type = ?2 AND status = 'active'
                 ORDER BY importance_score DESC, id ASC
                 LIMIT 1",
                params![topic_id, entry_type],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            self.ensure_generated_entry_relations(existing, topic_id, entry_type)?;
            return Ok(existing);
        }

        let context = self.load_topic_foundry_context(topic_id)?;
        let draft = self.build_knowledge_draft(&context, entry_type);
        let KnowledgeDraft {
            title,
            canonical_name,
            short_text,
            full_text,
            technical_text,
            exam_text,
            importance_score,
            difficulty_level,
            aliases,
        } = draft;
        let slug = slugify(&canonical_name);
        let simple_text = short_text.clone();

        self.conn
            .execute(
                "INSERT INTO knowledge_entries (
                    subject_id, topic_id, entry_type, title, canonical_name, slug, short_text,
                    full_text, simple_text, technical_text, exam_text, importance_score,
                    difficulty_level, grade_band, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'core', 'active')",
                params![
                    context.subject_id,
                    topic_id,
                    entry_type,
                    title,
                    canonical_name,
                    slug,
                    short_text,
                    full_text,
                    simple_text,
                    technical_text,
                    exam_text,
                    importance_score as i64,
                    difficulty_level as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_id = self.conn.last_insert_rowid();
        for alias in aliases {
            self.ensure_entry_alias(entry_id, &alias, "generated_seed")?;
        }
        self.ensure_generated_entry_relations(entry_id, topic_id, entry_type)?;
        Ok(entry_id)
    }

    fn seed_question_support(&self, topic_id: i64) -> EcoachResult<Value> {
        let context = self.load_topic_foundry_context(topic_id)?;
        let topic_name = context.topic_name.clone();
        let (node_id, _) = self.ensure_primary_node(topic_id, &topic_name)?;
        let misconception_id = self.seed_misconception_support(topic_id)?;
        let focus_label = self
            .best_candidate_label(&context, &["objective", "skill", "formula", "concept"])
            .unwrap_or_else(|| topic_name.clone());

        let family_id = if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM question_families
                 WHERE topic_id = ?1
                 ORDER BY id ASC
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            existing
        } else {
            self.conn
                .execute(
                    "INSERT INTO question_families (
                        family_code, family_name, subject_id, topic_id, family_type,
                        canonical_pattern, description
                     ) VALUES (?1, ?2, ?3, ?4, 'recurring_pattern', ?5, ?6)",
                    params![
                        format!("AUTO-{}-FAM", topic_id),
                        format!("{} coverage family", focus_label),
                        context.subject_id,
                        topic_id,
                        format!("Generated coverage pattern anchored on {}", focus_label),
                        format!(
                            "Source-aware family for {} built around {}.",
                            topic_name, focus_label
                        ),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn.last_insert_rowid()
        };

        let question_id = if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM questions
                 WHERE topic_id = ?1 AND family_id = ?2 AND is_active = 1
                 ORDER BY id ASC
                 LIMIT 1",
                params![topic_id, family_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            self.ensure_question_support_links(existing, topic_id)?;
            existing
        } else {
            let explanation_entry_id = self.seed_knowledge_support(topic_id, "explanation")?;
            let formula_entry_id = self.seed_knowledge_support(topic_id, "formula")?;
            let worked_example_entry_id =
                self.seed_knowledge_support(topic_id, "worked_example")?;
            let evidence_snippet = self.top_evidence_snippet(&context).unwrap_or_else(|| {
                format!(
                    "Reviewed content reinforces the safe application pattern for {}.",
                    topic_name
                )
            });
            self.conn
                .execute(
                    "INSERT INTO questions (
                        subject_id, topic_id, family_id, stem, question_format, explanation_text,
                        difficulty_level, estimated_time_seconds, marks, source_type,
                        primary_knowledge_role, primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function, classification_confidence, intelligence_snapshot,
                        primary_skill_id, cognitive_level, is_active, pack_id
                     ) VALUES (?1, ?2, ?3, ?4, 'mcq', ?5, 4200, 45, 1, 'generated',
                               'concept_check', 'recognition', 'single_step_identification',
                               'coverage_seed', 7200, ?6, ?7, 'understanding', 1, NULL)",
                    params![
                        context.subject_id,
                        topic_id,
                        family_id,
                        format!(
                            "Which option best shows how {} should be used inside {}?",
                            focus_label, topic_name
                        ),
                        format!(
                            "This source-aware item checks whether the learner can apply {} correctly in {}. {}",
                            focus_label, topic_name, evidence_snippet
                        ),
                        serialize_json(&json!({
                            "generated_by": "foundry_job_executor",
                            "topic_id": topic_id,
                            "family_id": family_id,
                            "focus_label": focus_label,
                            "explanation_entry_id": explanation_entry_id,
                            "formula_entry_id": formula_entry_id,
                            "worked_example_entry_id": worked_example_entry_id,
                            "candidate_count": context.approved_candidates.len(),
                            "evidence_count": context.approved_evidence.len(),
                        }))?,
                        node_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let question_id = self.conn.last_insert_rowid();
            let distractor_label = format!("Surface rule confusion in {}", focus_label);
            for (label, option_text, is_correct, misconception) in [
                (
                    "A",
                    format!(
                        "Use {} only when the quantities and meaning in {} actually match",
                        focus_label, topic_name
                    ),
                    1,
                    None,
                ),
                (
                    "B",
                    format!(
                        "Treat {} like a shortcut with no condition checks",
                        focus_label
                    ),
                    0,
                    Some(misconception_id),
                ),
                (
                    "C",
                    format!(
                        "Apply {} even when the representation changes meaning",
                        focus_label
                    ),
                    0,
                    Some(misconception_id),
                ),
                (
                    "D",
                    format!(
                        "Memorize a pattern for {} without connecting it to {}",
                        focus_label, topic_name
                    ),
                    0,
                    Some(misconception_id),
                ),
            ] {
                self.conn
                    .execute(
                        "INSERT INTO question_options (
                            question_id, option_label, option_text, is_correct, misconception_id,
                            distractor_intent, position
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            question_id,
                            label,
                            option_text,
                            is_correct,
                            misconception,
                            if is_correct == 1 {
                                None::<String>
                            } else {
                                Some(distractor_label.clone())
                            },
                            match label {
                                "A" => 1,
                                "B" => 2,
                                "C" => 3,
                                _ => 4,
                            },
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            self.conn
                .execute(
                    "INSERT INTO question_skill_links (question_id, node_id, contribution_weight, is_primary)
                     VALUES (?1, ?2, 10000, 1)",
                    params![question_id, node_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.ensure_question_support_links(question_id, topic_id)?;
            question_id
        };

        Ok(json!({
            "family_id": family_id,
            "question_id": question_id,
        }))
    }

    fn seed_contrast_support(&self, topic_id: i64) -> EcoachResult<i64> {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM contrast_pairs
                 WHERE topic_id = ?1
                 ORDER BY id ASC
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok(existing);
        }

        let context = self.load_topic_foundry_context(topic_id)?;
        let topic_name = context.topic_name.clone();
        let (left_entry_id, left_label, right_entry_id, right_label, summary_text) =
            self.select_contrast_material(&context)?;

        self.conn
            .execute(
                "INSERT INTO contrast_pairs (
                    left_entry_id, right_entry_id, title, trap_strength, created_at,
                    pair_code, subject_id, topic_id, left_label, right_label, summary_text, difficulty_score
                 ) VALUES (?1, ?2, ?3, 7200, datetime('now'), ?4, ?5, ?6, ?7, ?8, ?9, 5200)",
                params![
                    left_entry_id,
                    right_entry_id,
                    format!("{}: {} vs {}", topic_name, left_label, right_label),
                    format!("AUTO-CONTRAST-{}", topic_id),
                    context.subject_id,
                    topic_id,
                    left_label,
                    right_label,
                    summary_text,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let pair_id = self.conn.last_insert_rowid();

        for (ownership_type, atom_text, lane, explanation_text, reveal_order) in [
            (
                "left_only",
                format!(
                    "{} stays on the {} side",
                    left_label,
                    left_label.to_ascii_lowercase()
                ),
                "meaning",
                format!(
                    "{} should be recognized for its own job inside {}.",
                    left_label, topic_name
                ),
                1,
            ),
            (
                "right_only",
                format!(
                    "{} shows a different role inside {}",
                    right_label, topic_name
                ),
                "application",
                format!(
                    "{} should not be confused with {} in {}.",
                    right_label, left_label, topic_name
                ),
                2,
            ),
            (
                "both",
                format!("Both still belong to the same topic: {}", topic_name),
                "bridge",
                format!(
                    "Both sides stay inside {} but serve different learning roles.",
                    topic_name
                ),
                3,
            ),
        ] {
            self.conn
                .execute(
                    "INSERT INTO contrast_evidence_atoms (
                        pair_id, ownership_type, atom_text, created_at, lane, explanation_text,
                        difficulty_score, is_speed_ready, reveal_order
                     ) VALUES (?1, ?2, ?3, datetime('now'), ?4, ?5, 5000, 1, ?6)",
                    params![
                        pair_id,
                        ownership_type,
                        atom_text,
                        lane,
                        explanation_text,
                        reveal_order
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(pair_id)
    }

    fn materialize_candidate_nodes(
        &self,
        context: &TopicFoundryContext,
        primary_node_id: i64,
    ) -> EcoachResult<(i64, i64)> {
        let mut inserted_nodes = 0;
        let mut inserted_edges = 0;
        let mut seen_labels = BTreeSet::new();

        for candidate in context.approved_candidates.iter().filter(|candidate| {
            matches!(
                candidate.candidate_type.as_str(),
                "concept" | "formula" | "skill"
            )
        }) {
            let label = candidate_display_label(candidate);
            let normalized = normalize_phrase(&label);
            if normalized.is_empty() || !seen_labels.insert(normalized) {
                continue;
            }
            if let Some(existing_node_id) =
                self.find_academic_node_by_title(context.topic_id, &label)?
            {
                inserted_edges += self.ensure_node_edge(
                    context.topic_id,
                    "topic",
                    existing_node_id,
                    "academic_node",
                    "part_of",
                    candidate.confidence_score,
                )?;
                continue;
            }

            let node_type = node_type_for_candidate(candidate);
            let formal_description = payload_string(
                &candidate.payload,
                &[
                    "formal_description",
                    "description",
                    "definition",
                    "statement",
                ],
            )
            .unwrap_or_else(|| {
                format!(
                    "Reviewed curriculum support for {} inside {}.",
                    label, context.topic_name
                )
            });
            let simple_description = payload_string(
                &candidate.payload,
                &["simple_description", "simple_text", "learner_text"],
            )
            .unwrap_or_else(|| format!("Know what {} means in {}.", label, context.topic_name));
            let core_meaning = payload_string(
                &candidate.payload,
                &["core_meaning", "meaning", "exam_hint"],
            )
            .unwrap_or_else(|| {
                format!(
                    "{} is a reviewed curriculum anchor for {}.",
                    label, context.topic_name
                )
            });

            self.conn
                .execute(
                    "INSERT INTO academic_nodes (
                        topic_id, node_type, canonical_title, short_label, description_formal,
                        description_simple, core_meaning, difficulty_band, exam_relevance_score,
                        foundation_weight, is_active, metadata_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'medium', ?8, ?9, 1, ?10)",
                    params![
                        context.topic_id,
                        node_type,
                        label,
                        label,
                        formal_description,
                        simple_description,
                        core_meaning,
                        candidate.confidence_score as i64,
                        candidate.confidence_score as i64,
                        serialize_json(&json!({
                            "generated_by": "foundry_job_executor",
                            "candidate_id": candidate.id,
                            "candidate_type": candidate.candidate_type,
                            "source_upload_id": candidate.source_upload_id,
                        }))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let node_id = self.conn.last_insert_rowid();
            inserted_nodes += 1;
            inserted_edges += self.ensure_node_edge(
                context.topic_id,
                "topic",
                node_id,
                "academic_node",
                "part_of",
                candidate.confidence_score,
            )?;
            if node_id != primary_node_id {
                inserted_edges += self.ensure_node_edge(
                    primary_node_id,
                    "academic_node",
                    node_id,
                    "academic_node",
                    edge_type_for_candidate(candidate),
                    candidate.confidence_score,
                )?;
            }
        }

        Ok((inserted_nodes, inserted_edges))
    }

    fn materialize_learning_objectives(&self, context: &TopicFoundryContext) -> EcoachResult<i64> {
        let mut inserted = 0;
        let mut next_display_order = self.count_learning_objectives(context.topic_id)? + 1;
        let mut seen = BTreeSet::new();

        for candidate in context
            .approved_candidates
            .iter()
            .filter(|candidate| matches!(candidate.candidate_type.as_str(), "objective" | "skill"))
        {
            let objective_text = payload_string(
                &candidate.payload,
                &["objective_text", "statement", "simplified_text"],
            )
            .unwrap_or_else(|| candidate_display_label(candidate));
            let normalized = normalize_phrase(&objective_text);
            if normalized.is_empty() || !seen.insert(normalized.clone()) {
                continue;
            }
            inserted += self.ensure_learning_objective(
                context.topic_id,
                &objective_text,
                &simplified_learning_objective(&objective_text),
                cognitive_level_for_candidate(candidate),
                next_display_order,
            )?;
            if inserted > 0 {
                next_display_order += 1;
            }
            if inserted >= 3 {
                break;
            }
        }

        if inserted == 0 && self.count_learning_objectives(context.topic_id)? == 0 {
            inserted += self.ensure_learning_objective(
                context.topic_id,
                &format!("Explain the core idea behind {}.", context.topic_name),
                &format!("Understand the key idea in {}.", context.topic_name),
                "understanding",
                1,
            )?;
        }

        Ok(inserted)
    }

    fn ensure_node_edge(
        &self,
        from_node_id: i64,
        from_node_type: &str,
        to_node_id: i64,
        to_node_type: &str,
        edge_type: &str,
        strength_score: BasisPoints,
    ) -> EcoachResult<i64> {
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM node_edges
                    WHERE from_node_id = ?1
                      AND from_node_type = ?2
                      AND to_node_id = ?3
                      AND to_node_type = ?4
                      AND edge_type = ?5
                 )",
                params![
                    from_node_id,
                    from_node_type,
                    to_node_id,
                    to_node_type,
                    edge_type
                ],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 1 {
            return Ok(0);
        }
        self.conn
            .execute(
                "INSERT INTO node_edges (
                    from_node_id, from_node_type, to_node_id, to_node_type, edge_type, strength_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    from_node_id,
                    from_node_type,
                    to_node_id,
                    to_node_type,
                    edge_type,
                    strength_score as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(1)
    }

    fn ensure_learning_objective(
        &self,
        topic_id: i64,
        objective_text: &str,
        simplified_text: &str,
        cognitive_level: &str,
        display_order: i64,
    ) -> EcoachResult<i64> {
        let normalized = objective_text.trim().to_ascii_lowercase();
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM learning_objectives
                    WHERE topic_id = ?1 AND lower(objective_text) = ?2
                 )",
                params![topic_id, normalized],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 1 {
            return Ok(0);
        }
        self.conn
            .execute(
                "INSERT INTO learning_objectives (
                    topic_id, objective_text, simplified_text, cognitive_level, display_order
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    topic_id,
                    objective_text,
                    simplified_text,
                    cognitive_level,
                    display_order,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(1)
    }

    fn find_academic_node_by_title(&self, topic_id: i64, title: &str) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id
                 FROM academic_nodes
                 WHERE topic_id = ?1
                   AND lower(canonical_title) = ?2
                 LIMIT 1",
                params![topic_id, title.trim().to_ascii_lowercase()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn best_candidate_label(
        &self,
        context: &TopicFoundryContext,
        candidate_types: &[&str],
    ) -> Option<String> {
        candidate_types.iter().find_map(|candidate_type| {
            context
                .approved_candidates
                .iter()
                .find(|candidate| candidate.candidate_type == *candidate_type)
                .map(candidate_display_label)
        })
    }

    fn top_evidence_snippet(&self, context: &TopicFoundryContext) -> Option<String> {
        context
            .approved_evidence
            .iter()
            .find_map(|evidence| {
                evidence
                    .snippet
                    .as_ref()
                    .map(|value| value.trim().to_string())
            })
            .filter(|value| !value.is_empty())
    }

    fn build_knowledge_draft(
        &self,
        context: &TopicFoundryContext,
        entry_type: &str,
    ) -> KnowledgeDraft {
        let topic_name = &context.topic_name;
        let primary_label = match entry_type {
            "formula" => self
                .best_candidate_label(context, &["formula", "concept"])
                .unwrap_or_else(|| format!("{} key formula", topic_name)),
            "worked_example" => context
                .approved_evidence
                .iter()
                .find_map(|evidence| evidence.title.clone())
                .unwrap_or_else(|| format!("{} worked example", topic_name)),
            "definition" => self
                .best_candidate_label(context, &["concept", "topic"])
                .unwrap_or_else(|| format!("{} definition", topic_name)),
            _ => self
                .best_candidate_label(context, &["objective", "skill", "concept"])
                .unwrap_or_else(|| format!("{} core explanation", topic_name)),
        };

        let evidence_title = context
            .approved_evidence
            .iter()
            .find_map(|evidence| evidence.title.clone())
            .unwrap_or_else(|| format!("Reviewed support for {}", topic_name));
        let evidence_snippet = self.top_evidence_snippet(context).unwrap_or_else(|| {
            format!(
                "Reviewed content is available to reinforce {} in {}.",
                primary_label, topic_name
            )
        });
        let objective_hint = self
            .best_candidate_label(context, &["objective", "skill"])
            .unwrap_or_else(|| format!("apply {} safely", topic_name));

        let (
            title,
            short_text,
            full_text,
            technical_text,
            exam_text,
            importance_score,
            difficulty_level,
        ) = match entry_type {
            "formula" => (
                primary_label.clone(),
                format!("Formula support for {} in {}.", primary_label, topic_name),
                format!(
                    "{} is the reviewed formula anchor for {}. {}",
                    primary_label, topic_name, evidence_snippet
                ),
                format!(
                    "Track the symbolic structure of {} and the condition that makes it valid in {}.",
                    primary_label, topic_name
                ),
                format!(
                    "Exam use: decide when {} fits the demand in {} before substituting values.",
                    primary_label, topic_name
                ),
                8_200_u16,
                5_200_u16,
            ),
            "worked_example" => (
                primary_label.clone(),
                format!("Worked example support for {}.", topic_name),
                format!(
                    "{} demonstrates a safe solution route for {}. {}",
                    evidence_title, topic_name, evidence_snippet
                ),
                format!(
                    "Use the worked example as a decision path for {} and notice where {} matters.",
                    topic_name, objective_hint
                ),
                format!(
                    "Exam use: mirror this example structure when {} appears under pressure.",
                    topic_name
                ),
                7_900_u16,
                5_000_u16,
            ),
            "definition" => (
                primary_label.clone(),
                format!("Definition support for {}.", primary_label),
                format!(
                    "{} fixes the meaning boundary for {}. {}",
                    primary_label, topic_name, evidence_snippet
                ),
                format!(
                    "This definition separates {} from nearby ideas the learner may confuse in {}.",
                    primary_label, topic_name
                ),
                format!(
                    "Exam use: state {} precisely before solving any {} item.",
                    primary_label, topic_name
                ),
                7_600_u16,
                4_300_u16,
            ),
            _ => (
                primary_label.clone(),
                format!("Explanation support for {}.", topic_name),
                format!(
                    "{} anchors the main idea in {}. {}",
                    primary_label, topic_name, evidence_snippet
                ),
                format!(
                    "Link {} to the reviewed objective '{}', then explain why the method fits.",
                    topic_name, objective_hint
                ),
                format!(
                    "Exam use: justify why the chosen method is correct in {}.",
                    topic_name
                ),
                8_000_u16,
                4_600_u16,
            ),
        };

        let mut aliases = BTreeSet::new();
        aliases.insert(topic_name.clone());
        if let Some(topic_code) = &context.topic_code {
            aliases.insert(topic_code.clone());
        }
        aliases.insert(primary_label.clone());
        aliases.insert(evidence_title);
        if let Some(label) = self.best_candidate_label(context, &["objective", "skill"]) {
            aliases.insert(label);
        }

        KnowledgeDraft {
            title: title.clone(),
            canonical_name: title,
            short_text,
            full_text,
            technical_text,
            exam_text,
            importance_score,
            difficulty_level,
            aliases: aliases
                .into_iter()
                .filter(|alias| !normalize_phrase(alias).is_empty())
                .collect(),
        }
    }

    fn ensure_entry_alias(
        &self,
        entry_id: i64,
        alias_text: &str,
        alias_type: &str,
    ) -> EcoachResult<()> {
        let normalized = alias_text.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(());
        }
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM entry_aliases
                    WHERE entry_id = ?1 AND lower(alias_text) = ?2
                 )",
                params![entry_id, normalized],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO entry_aliases (entry_id, alias_text, alias_type)
                     VALUES (?1, ?2, ?3)",
                    params![entry_id, alias_text, alias_type],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn ensure_generated_entry_relations(
        &self,
        entry_id: i64,
        topic_id: i64,
        entry_type: &str,
    ) -> EcoachResult<()> {
        match entry_type {
            "formula" => {
                if let Some(explanation_id) =
                    self.find_entry_by_type(topic_id, "explanation", Some(entry_id))?
                {
                    self.ensure_knowledge_relation(
                        entry_id,
                        explanation_id,
                        "supports_explanation",
                        "Formula support should point back to the main explanation.",
                    )?;
                }
            }
            "worked_example" => {
                if let Some(formula_id) =
                    self.find_entry_by_type(topic_id, "formula", Some(entry_id))?
                {
                    self.ensure_knowledge_relation(
                        entry_id,
                        formula_id,
                        "example_for",
                        "Worked examples should reinforce the formula or method they demonstrate.",
                    )?;
                }
                if let Some(explanation_id) =
                    self.find_entry_by_type(topic_id, "explanation", Some(entry_id))?
                {
                    self.ensure_knowledge_relation(
                        entry_id,
                        explanation_id,
                        "example_for",
                        "Worked examples should reinforce the core explanation.",
                    )?;
                }
            }
            "definition" => {
                if let Some(explanation_id) =
                    self.find_entry_by_type(topic_id, "explanation", Some(entry_id))?
                {
                    self.ensure_knowledge_relation(
                        entry_id,
                        explanation_id,
                        "anchors_meaning",
                        "Definitions should anchor meaning for the linked explanation.",
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn ensure_knowledge_relation(
        &self,
        from_entry_id: i64,
        to_entry_id: i64,
        relation_type: &str,
        explanation: &str,
    ) -> EcoachResult<()> {
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM knowledge_relations
                    WHERE from_entry_id = ?1
                      AND to_entry_id = ?2
                      AND relation_type = ?3
                 )",
                params![from_entry_id, to_entry_id, relation_type],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO knowledge_relations (
                        from_entry_id, to_entry_id, relation_type, strength_score, explanation
                     ) VALUES (?1, ?2, ?3, 7600, ?4)",
                    params![from_entry_id, to_entry_id, relation_type, explanation],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn find_entry_by_type(
        &self,
        topic_id: i64,
        entry_type: &str,
        exclude_entry_id: Option<i64>,
    ) -> EcoachResult<Option<i64>> {
        let sql = if exclude_entry_id.is_some() {
            "SELECT id
             FROM knowledge_entries
             WHERE topic_id = ?1
               AND entry_type = ?2
               AND status = 'active'
               AND id != ?3
             ORDER BY importance_score DESC, id ASC
             LIMIT 1"
        } else {
            "SELECT id
             FROM knowledge_entries
             WHERE topic_id = ?1
               AND entry_type = ?2
               AND status = 'active'
             ORDER BY importance_score DESC, id ASC
             LIMIT 1"
        };
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let row = if let Some(exclude_entry_id) = exclude_entry_id {
            statement.query_row(params![topic_id, entry_type, exclude_entry_id], |row| {
                row.get(0)
            })
        } else {
            statement.query_row(params![topic_id, entry_type], |row| row.get(0))
        };
        row.optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_topic_knowledge_entries(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Vec<TopicKnowledgeEntry>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, entry_type, title, short_text, full_text, importance_score
                 FROM knowledge_entries
                 WHERE topic_id = ?1 AND status = 'active'
                 ORDER BY importance_score DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| {
                Ok(TopicKnowledgeEntry {
                    id: row.get(0)?,
                    entry_type: row.get(1)?,
                    title: row.get(2)?,
                    importance_score: row.get::<_, i64>(5)? as BasisPoints,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn ensure_question_support_links(&self, question_id: i64, topic_id: i64) -> EcoachResult<()> {
        let entries = self.list_topic_knowledge_entries(topic_id)?;
        for (index, entry) in entries.into_iter().take(3).enumerate() {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO question_glossary_links (
                        question_id, entry_id, relation_type, link_source, link_reason,
                        confidence_score, is_primary
                     ) VALUES (?1, ?2, ?3, 'planner', ?4, ?5, ?6)",
                    params![
                        question_id,
                        entry.id,
                        relation_type_for_entry_type(&entry.entry_type),
                        format!("foundry_support_{}", entry.entry_type),
                        entry.importance_score as i64,
                        if index == 0 { 1 } else { 0 },
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn select_contrast_material(
        &self,
        context: &TopicFoundryContext,
    ) -> EcoachResult<(i64, String, i64, String, String)> {
        let mut entries = self.list_topic_knowledge_entries(context.topic_id)?;
        if entries.is_empty() {
            self.seed_knowledge_support(context.topic_id, "definition")?;
            self.seed_knowledge_support(context.topic_id, "explanation")?;
            entries = self.list_topic_knowledge_entries(context.topic_id)?;
        }

        if let Some((left, right)) = select_entry_pair(&entries) {
            let summary_text = format!(
                "Use this pair to separate {} from {} inside {}.",
                left.title, right.title, context.topic_name
            );
            return Ok((
                left.id,
                humanize_entry_type(&left.entry_type),
                right.id,
                humanize_entry_type(&right.entry_type),
                summary_text,
            ));
        }

        let left_entry_id = self.seed_knowledge_support(context.topic_id, "definition")?;
        let right_entry_id = self.seed_knowledge_support(context.topic_id, "explanation")?;
        Ok((
            left_entry_id,
            "Definition".to_string(),
            right_entry_id,
            "Explanation".to_string(),
            format!(
                "Use this pair to separate the meaning of {} from an explanation of it.",
                context.topic_name
            ),
        ))
    }

    fn ensure_primary_node(&self, topic_id: i64, topic_name: &str) -> EcoachResult<(i64, bool)> {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT id
                 FROM academic_nodes
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY foundation_weight DESC, id ASC
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok((existing, false));
        }

        self.conn
            .execute(
                "INSERT INTO academic_nodes (
                    topic_id, node_type, canonical_title, short_label, description_formal,
                    description_simple, core_meaning, difficulty_band, exam_relevance_score,
                    foundation_weight, is_active, metadata_json
                 ) VALUES (?1, 'concept', ?2, ?3, ?4, ?5, ?6, 'medium', 7600, 8200, 1, ?7)",
                params![
                    topic_id,
                    format!("{} core idea", topic_name),
                    topic_name,
                    format!("Formal concept anchor for {}", topic_name),
                    format!("Simple anchor for {}", topic_name),
                    format!(
                        "The learner should recognize the central meaning of {}.",
                        topic_name
                    ),
                    serialize_json(&json!({
                        "generated_by": "foundry_job_executor",
                        "seed_kind": "curriculum_support",
                    }))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok((self.conn.last_insert_rowid(), true))
    }

    fn count_learning_objectives(&self, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM learning_objectives WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_topic_edges(&self, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM node_edges
                 WHERE (from_node_type = 'topic' AND from_node_id = ?1)
                    OR (to_node_type = 'topic' AND to_node_id = ?1)",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
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

fn normalized_tokens(value: &str) -> BTreeSet<String> {
    normalize_phrase(value)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn payload_string(payload: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        payload
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

fn candidate_display_label(candidate: &CurriculumParseCandidate) -> String {
    let raw = candidate.raw_label.trim();
    if !raw.is_empty() {
        raw.to_string()
    } else {
        candidate
            .normalized_label
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .unwrap_or_default()
    }
}

fn node_type_for_candidate(candidate: &CurriculumParseCandidate) -> &'static str {
    match candidate.candidate_type.as_str() {
        "formula" => "formula",
        "skill" => "procedure",
        _ => "concept",
    }
}

fn edge_type_for_candidate(candidate: &CurriculumParseCandidate) -> &'static str {
    match candidate.candidate_type.as_str() {
        "formula" => "uses_formula",
        "skill" => "uses_procedure",
        _ => "related",
    }
}

fn cognitive_level_for_candidate(candidate: &CurriculumParseCandidate) -> &'static str {
    match payload_string(
        &candidate.payload,
        &["cognitive_level", "level", "demand", "objective_level"],
    )
    .as_deref()
    {
        Some("knowledge") => "knowledge",
        Some("application") => "application",
        Some("reasoning") => "reasoning",
        Some("evaluation") => "evaluation",
        _ => "understanding",
    }
}

fn simplified_learning_objective(objective_text: &str) -> String {
    let trimmed = objective_text.trim();
    if trimmed.len() <= 80 {
        return trimmed.to_string();
    }
    let shortened = trimmed.chars().take(77).collect::<String>();
    format!("{}...", shortened.trim_end())
}

fn candidate_matches_topic(
    candidate: &CurriculumParseCandidate,
    topic_id: i64,
    topic_code: Option<&str>,
    topic_name: &str,
) -> bool {
    if candidate.payload.get("topic_id").and_then(Value::as_i64) == Some(topic_id)
        || candidate
            .payload
            .get("parent_topic_id")
            .and_then(Value::as_i64)
            == Some(topic_id)
    {
        return true;
    }

    let topic_phrase = normalize_phrase(topic_name);
    let topic_tokens = normalized_tokens(topic_name);
    let candidate_text = format!(
        "{} {} {}",
        candidate.raw_label,
        candidate.normalized_label.as_deref().unwrap_or_default(),
        candidate.payload
    );
    let candidate_phrase = normalize_phrase(&candidate_text);
    let candidate_tokens = normalized_tokens(&candidate_text);

    if !topic_phrase.is_empty()
        && (candidate_phrase.contains(&topic_phrase) || topic_phrase.contains(&candidate_phrase))
    {
        return true;
    }

    if !topic_tokens.is_empty() && !candidate_tokens.is_disjoint(&topic_tokens) {
        return true;
    }

    if let Some(topic_code) = topic_code {
        let topic_code_phrase = normalize_phrase(topic_code);
        if !topic_code_phrase.is_empty()
            && (candidate_phrase.contains(&topic_code_phrase)
                || payload_string(
                    &candidate.payload,
                    &["topic_code", "parent_topic_code", "code", "topic"],
                )
                .map(|value| normalize_phrase(&value) == topic_code_phrase)
                .unwrap_or(false))
        {
            return true;
        }
    }

    matches!(candidate.candidate_type.as_str(), "subject")
}

fn humanize_entry_type(entry_type: &str) -> String {
    match entry_type {
        "worked_example" => "Worked Example".to_string(),
        "definition" => "Definition".to_string(),
        "formula" => "Formula".to_string(),
        "explanation" => "Explanation".to_string(),
        other => other.replace('_', " "),
    }
}

fn humanize_candidate_type(candidate_type: &str) -> String {
    match candidate_type {
        "objective" => "Objective".to_string(),
        "skill" => "Skill".to_string(),
        "concept" => "Concept".to_string(),
        "formula" => "Formula".to_string(),
        "dependency" => "Dependency".to_string(),
        other => other.replace('_', " "),
    }
}

fn relation_type_for_entry_type(entry_type: &str) -> &'static str {
    match entry_type {
        "definition" => "definition_support",
        "worked_example" => "worked_example_support",
        "formula" => "formula_support",
        _ => "repair_support",
    }
}

fn select_entry_pair(
    entries: &[TopicKnowledgeEntry],
) -> Option<(&TopicKnowledgeEntry, &TopicKnowledgeEntry)> {
    for (left_type, right_type) in [
        ("definition", "explanation"),
        ("formula", "worked_example"),
        ("definition", "worked_example"),
        ("explanation", "formula"),
    ] {
        let left = entries.iter().find(|entry| entry.entry_type == left_type);
        let right = entries.iter().find(|entry| entry.entry_type == right_type);
        if let (Some(left), Some(right)) = (left, right) {
            return Some((left, right));
        }
    }

    if entries.len() >= 2 {
        return Some((&entries[0], &entries[1]));
    }

    None
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

fn is_auto_review_task_type(task_type: &str) -> bool {
    matches!(
        task_type,
        "normalization" | "duplicate_check" | "hierarchy_fix" | "publish_gate"
    )
}

fn auto_review_task_key(
    candidate_id: Option<i64>,
    task_type: &str,
    notes: &str,
) -> (Option<i64>, String, String) {
    (candidate_id, task_type.to_string(), notes.to_string())
}

fn append_review_task_note(existing: Option<&str>, appended: &str) -> String {
    match existing {
        Some(current) if !current.is_empty() => format!("{current}\n{appended}"),
        _ => appended.to_string(),
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
        if job_type == "publish_activation_job"
            && snapshot
                .recommended_jobs
                .iter()
                .any(|job| job == "quality_review_job")
        {
            dependencies.push("quality_review_job".to_string());
        }
    } else if job_type == "quality_review_job" && snapshot.publishable_artifact_count == 0 {
        dependencies.push("publish_job".to_string());
    }
    dedupe_and_sort(&mut dependencies);
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

fn build_source_fabric_signals(
    source_upload: &CurriculumSourceUpload,
    approved_candidate_count: i64,
    unresolved_review_count: i64,
    publish_readiness_score: BasisPoints,
    publish_jobs: &[ContentPublishJobReport],
    recommended_actions: &[String],
) -> Vec<FabricSignal> {
    let observed_at = now_utc().to_rfc3339();
    let mut signals = vec![FabricSignal {
        engine_key: "content_packs".to_string(),
        signal_type: "foundry_source_status".to_string(),
        status: Some(source_upload.source_status.clone()),
        score: Some(source_upload.confidence_score),
        topic_id: None,
        node_id: None,
        question_id: None,
        observed_at: observed_at.clone(),
        payload: json!({
            "source_kind": source_upload.source_kind,
            "subject_code": source_upload.subject_code,
            "approved_candidate_count": approved_candidate_count,
            "unresolved_review_count": unresolved_review_count,
        }),
    }];

    if publish_readiness_score > 0 {
        signals.push(FabricSignal {
            engine_key: "content_packs".to_string(),
            signal_type: "foundry_publish_readiness".to_string(),
            status: Some(
                if publish_jobs.iter().any(|job| job.job.status == "published") {
                    "published".to_string()
                } else if publish_jobs
                    .iter()
                    .any(|job| job.job.status == "ready_to_publish")
                {
                    "ready".to_string()
                } else {
                    "building".to_string()
                },
            ),
            score: Some(publish_readiness_score),
            topic_id: None,
            node_id: None,
            question_id: None,
            observed_at,
            payload: json!({
                "recommended_actions": recommended_actions,
            }),
        });
    }

    signals
}

#[allow(clippy::too_many_arguments)]
fn build_topic_fabric_signals(
    topic_id: i64,
    topic_name: &str,
    package_state: &str,
    completeness_score: BasisPoints,
    quality_score: BasisPoints,
    missing_components: &[String],
    recommended_jobs: &[String],
    publishable_artifact_count: i64,
    published_artifact_count: i64,
) -> Vec<FabricSignal> {
    let observed_at = now_utc().to_rfc3339();
    let mut signals = vec![FabricSignal {
        engine_key: "content_packs".to_string(),
        signal_type: "topic_package_health".to_string(),
        status: Some(package_state.to_string()),
        score: Some(completeness_score),
        topic_id: Some(topic_id),
        node_id: None,
        question_id: None,
        observed_at: observed_at.clone(),
        payload: json!({
            "topic_name": topic_name,
            "quality_score": quality_score,
            "missing_components": missing_components,
        }),
    }];

    if publishable_artifact_count > 0 || published_artifact_count > 0 {
        signals.push(FabricSignal {
            engine_key: "content_packs".to_string(),
            signal_type: "topic_package_publishability".to_string(),
            status: Some(if published_artifact_count > 0 {
                "live".to_string()
            } else {
                "ready".to_string()
            }),
            score: Some(quality_score),
            topic_id: Some(topic_id),
            node_id: None,
            question_id: None,
            observed_at,
            payload: json!({
                "recommended_jobs": recommended_jobs,
                "publishable_artifact_count": publishable_artifact_count,
                "published_artifact_count": published_artifact_count,
            }),
        });
    }

    signals
}

fn build_dashboard_fabric_signals(
    _subject_id: i64,
    subject_code: &str,
    topics: &[TopicPackageSnapshot],
) -> Vec<FabricSignal> {
    let observed_at = now_utc().to_rfc3339();
    let live_topics = topics
        .iter()
        .filter(|topic| topic.published_artifact_count > 0)
        .count() as i64;
    let weak_topics = topics
        .iter()
        .filter(|topic| topic.completeness_score < 5_500)
        .count() as i64;

    let mut signals = vec![FabricSignal {
        engine_key: "content_packs".to_string(),
        signal_type: "subject_foundry_health".to_string(),
        status: Some(if weak_topics > 0 {
            "repair_needed".to_string()
        } else {
            "healthy".to_string()
        }),
        score: Some(if topics.is_empty() {
            0
        } else {
            clamp_bp(
                topics
                    .iter()
                    .map(|topic| topic.completeness_score as i64)
                    .sum::<i64>()
                    / topics.len() as i64,
            )
        }),
        topic_id: None,
        node_id: None,
        question_id: None,
        observed_at,
        payload: json!({
            "subject_code": subject_code,
            "live_topics": live_topics,
            "weak_topics": weak_topics,
        }),
    }];
    for topic in topics {
        signals.extend(topic.fabric_signals.clone());
    }
    signals
}

fn foundry_available_inputs(signals: &[FabricSignal]) -> Vec<String> {
    let mut inputs = BTreeSet::new();
    for signal in signals {
        match signal.signal_type.as_str() {
            "foundry_source_status" | "topic_package_health" | "subject_foundry_health" => {
                inputs.insert("curriculum_nodes".to_string());
            }
            "topic_package_publishability" => {
                inputs.insert("question_content".to_string());
                inputs.insert("knowledge_entries".to_string());
                inputs.insert("knowledge_links".to_string());
            }
            "foundry_publish_readiness" => {
                inputs.insert("pack_manifests".to_string());
            }
            _ => {}
        }
    }
    inputs.into_iter().collect()
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
        assert!(!report.fabric_signals.is_empty());
        assert!(
            report
                .orchestration
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "content_packs")
        );
    }

    #[test]
    fn foundry_finalize_source_parse_reconciles_auto_review_tasks() {
        let conn = open_test_database();
        seed_admin(&conn);
        let service = FoundryCoordinatorService::new(&conn);
        let source = service
            .register_source_upload(SourceUploadInput {
                uploader_account_id: 1,
                source_kind: "curriculum".to_string(),
                title: "Math Curriculum Refresh".to_string(),
                source_path: Some("C:/curriculum/math-refresh.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some("MATH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("v-refresh".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");

        for (candidate_type, raw_label, confidence) in [
            ("subject", "Mathematics", 9_200),
            ("topic", "Fractions", 5_900),
            ("topic", "Fractions", 8_400),
        ] {
            service
                .add_parse_candidate(
                    source.id,
                    ParseCandidateInput {
                        candidate_type: candidate_type.to_string(),
                        parent_candidate_id: None,
                        raw_label: raw_label.to_string(),
                        normalized_label: Some(raw_label.to_ascii_lowercase()),
                        payload: json!({ "page": 2 }),
                        confidence_score: confidence,
                    },
                )
                .expect("parse candidate should insert");
        }

        let initial_report = service
            .finalize_source_parse(source.id)
            .expect("initial parse should finalize");
        assert_eq!(
            initial_report.source_upload.source_status,
            "review_required"
        );
        assert!(initial_report.unresolved_review_count >= 2);

        let low_confidence_candidate_id: i64 = conn
            .query_row(
                "SELECT id
                 FROM curriculum_parse_candidates
                 WHERE source_upload_id = ?1
                   AND confidence_score = 5900
                 LIMIT 1",
                [source.id],
                |row| row.get(0),
            )
            .expect("low confidence candidate should exist");
        conn.execute(
            "UPDATE curriculum_parse_candidates
             SET raw_label = 'Fractions Foundations',
                 normalized_label = 'fractions foundations',
                 confidence_score = 9100,
                 updated_at = datetime('now')
             WHERE id = ?1",
            [low_confidence_candidate_id],
        )
        .expect("candidate refresh should persist");
        service
            .add_parse_candidate(
                source.id,
                ParseCandidateInput {
                    candidate_type: "objective".to_string(),
                    parent_candidate_id: None,
                    raw_label: "Identify equivalent fractions".to_string(),
                    normalized_label: Some("identify equivalent fractions".to_string()),
                    payload: json!({ "page": 3 }),
                    confidence_score: 9_000,
                },
            )
            .expect("objective candidate should insert");

        let refreshed_report = service
            .finalize_source_parse(source.id)
            .expect("refresh parse should finalize");
        let follow_up_jobs = service
            .queue_source_follow_up_jobs(source.id, "parse_refresh")
            .expect("follow-up jobs should queue");

        assert_eq!(refreshed_report.source_upload.source_status, "parsed");
        assert_eq!(refreshed_report.unresolved_review_count, 0);
        assert!(
            refreshed_report
                .review_tasks
                .iter()
                .all(|task| task.status == "resolved")
        );
        assert!(
            follow_up_jobs
                .iter()
                .any(|job| job.job_type == "source_approval_job")
        );
        let refreshed_candidate_status: String = conn
            .query_row(
                "SELECT review_status
                 FROM curriculum_parse_candidates
                 WHERE id = ?1",
                [low_confidence_candidate_id],
                |row| row.get(0),
            )
            .expect("candidate status should load");
        assert_eq!(refreshed_candidate_status, "approved");
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
        assert!(
            dashboard
                .topics
                .iter()
                .any(|topic| !topic.fabric_signals.is_empty())
        );
        assert!(
            dashboard
                .orchestration
                .consumer_targets
                .iter()
                .any(|target| target.engine_key == "library")
        );
    }

    #[test]
    fn foundry_stage_publish_job_prefers_quality_follow_up_over_duplicate_publish_job() {
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
                title: "Quality Gate Topic".to_string(),
                source_path: Some("C:/curriculum/quality-gate.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some("MATH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("gate-v1".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");

        for (candidate_type, raw_label) in [
            ("subject", "Mathematics"),
            ("topic", "Ratio Review Topic"),
            ("objective", "Interpret ratio tables"),
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
                        confidence_score: 6_000,
                    },
                )
                .expect("parse candidate should insert");
        }
        let review_report = service
            .finalize_source_parse(source.id)
            .expect("parse should finalize");
        for task in review_report.review_tasks {
            service
                .resolve_review_task(
                    task.id,
                    "manual approval for trusted curriculum draft",
                    true,
                )
                .expect("review task should resolve");
        }
        service
            .mark_source_reviewed(source.id)
            .expect("source should become reviewed");

        conn.execute(
            "INSERT INTO topics (
                subject_id, code, name, description, node_type, display_order,
                exam_weight, difficulty_band, importance_weight, is_active
             ) VALUES (1, 'AUTO-GATE', 'Ratio Review Topic', 'Sparse quality gate topic', 'topic', 1200, 4500, 'medium', 6500, 1)",
            [],
        )
        .expect("gate topic should insert");
        let topic_id = conn.last_insert_rowid();

        let publish_report = service
            .stage_publish_job(source.id, Some(1), Some(1), Some(topic_id), Some("gate-v1"))
            .expect("publish job should stage");
        let snapshot = service
            .recompute_topic_package_snapshot(topic_id)
            .expect("snapshot should recompute")
            .expect("snapshot should exist");
        let queued_jobs = service
            .queue_topic_foundry_jobs(topic_id, "snapshot_refresh")
            .expect("follow-up jobs should queue");

        assert!(
            matches!(
                publish_report.job.status.as_str(),
                "review_required" | "gating"
            ),
            "sparse topic should stage into a blocked quality state first"
        );
        assert!(snapshot.publishable_artifact_count >= 1);
        assert!(
            snapshot
                .recommended_jobs
                .iter()
                .any(|job| job == "quality_review_job")
        );
        assert!(
            !snapshot
                .recommended_jobs
                .iter()
                .any(|job| job == "publish_job")
        );
        assert!(
            queued_jobs
                .iter()
                .any(|job| job.job_type == "quality_review_job")
        );
        let activation_job = queued_jobs
            .iter()
            .find(|job| job.job_type == "publish_activation_job")
            .expect("publish activation job should queue");
        assert!(
            activation_job
                .dependency_refs
                .iter()
                .any(|dependency| dependency == "quality_review_job")
        );
    }

    #[test]
    fn foundry_run_next_builds_sparse_topic_artifacts_end_to_end() {
        let conn = open_test_database();
        seed_admin(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let service = FoundryCoordinatorService::new(&conn);

        let subject_code: String = conn
            .query_row("SELECT code FROM subjects WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("subject should exist");
        let source = service
            .register_source_upload(SourceUploadInput {
                uploader_account_id: 1,
                source_kind: "curriculum".to_string(),
                title: "Sparse Topic Builder".to_string(),
                source_path: Some("C:/tmp/sparse-topic.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some(subject_code),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("draft".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");
        for (candidate_type, raw_label) in [
            ("subject", "Mathematics"),
            ("topic", "Ratio Reasoning Foundations"),
            ("objective", "Interpret ratio statements"),
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
                        confidence_score: 9_000,
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

        conn.execute(
            "INSERT INTO topics (
                subject_id, code, name, description, node_type, display_order,
                exam_weight, difficulty_band, importance_weight, is_active
             ) VALUES (1, 'AUTO-SPARSE', 'Ratio Reasoning Foundations', 'Sparse foundry build target', 'topic', 999, 4500, 'medium', 7200, 1)",
            [],
        )
        .expect("sparse topic should insert");
        let topic_id = conn.last_insert_rowid();

        let queued_jobs = service
            .queue_topic_foundry_jobs(topic_id, "snapshot_refresh")
            .expect("topic jobs should queue");
        assert!(
            queued_jobs
                .iter()
                .any(|job| job.job_type == "curriculum_enrichment_job"),
            "curriculum enrichment job should be queued for a sparse topic"
        );
        assert!(
            queued_jobs
                .iter()
                .any(|job| job.job_type == "question_generation_job"),
            "question generation job should be queued for a sparse topic"
        );
        assert!(
            queued_jobs.iter().any(|job| job.job_type == "publish_job"),
            "publish job should be queued for a sparse topic"
        );

        let mut completed_job_types = BTreeSet::new();
        for _ in 0..20 {
            let Some(job) = service
                .run_next_foundry_job(Some(1))
                .expect("running next foundry job should succeed")
            else {
                break;
            };
            if job.topic_id == Some(topic_id) && job.status == "completed" {
                completed_job_types.insert(job.job_type);
            }
        }

        let node_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM academic_nodes WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("academic nodes should count");
        let objective_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM learning_objectives WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("learning objectives should count");
        let misconception_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM misconception_patterns WHERE topic_id = ?1 AND is_active = 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("misconceptions should count");
        let question_family_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM question_families WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("question families should count");
        let question_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM questions WHERE topic_id = ?1 AND is_active = 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("questions should count");
        let explanation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'explanation' AND status = 'active'",
                [topic_id],
                |row| row.get(0),
            )
            .expect("explanations should count");
        let formula_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'formula' AND status = 'active'",
                [topic_id],
                |row| row.get(0),
            )
            .expect("formulas should count");
        let worked_example_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'worked_example' AND status = 'active'",
                [topic_id],
                |row| row.get(0),
            )
            .expect("worked examples should count");
        let contrast_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM contrast_pairs WHERE topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("contrast pairs should count");

        let latest_publish_job = ContentPublishService::new(&conn)
            .list_publish_jobs(None)
            .expect("publish jobs should list")
            .into_iter()
            .filter(|job| job.topic_id == Some(topic_id))
            .max_by_key(|job| job.id)
            .expect("topic should have a publish job");
        let snapshot = service
            .recompute_topic_package_snapshot(topic_id)
            .expect("snapshot should recompute")
            .expect("snapshot should exist");

        assert!(completed_job_types.contains("curriculum_enrichment_job"));
        assert!(completed_job_types.contains("misconception_build_job"));
        assert!(completed_job_types.contains("question_generation_job"));
        assert!(completed_job_types.contains("note_build_job"));
        assert!(completed_job_types.contains("formula_pack_build_job"));
        assert!(completed_job_types.contains("worked_example_build_job"));
        assert!(completed_job_types.contains("contrast_build_job"));
        assert!(completed_job_types.contains("publish_job"));
        assert!(node_count >= 1);
        assert!(objective_count >= 1);
        assert!(misconception_count >= 1);
        assert!(question_family_count >= 1);
        assert!(question_count >= 1);
        assert!(explanation_count >= 1);
        assert!(formula_count >= 1);
        assert!(worked_example_count >= 1);
        assert!(contrast_count >= 1);
        assert!(
            matches!(
                latest_publish_job.status.as_str(),
                "ready_to_publish" | "published"
            ),
            "sparse topic flow should stage a publish job that is at least ready to publish"
        );
        assert!(snapshot.publishable_artifact_count >= 1);
        assert!(snapshot.resource_readiness_score >= 6_000);
        assert!(!snapshot.fabric_signals.is_empty());
    }

    #[test]
    fn foundry_builders_use_reviewed_source_materials_when_available() {
        let conn = open_test_database();
        seed_admin(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let service = FoundryCoordinatorService::new(&conn);

        let subject_code: String = conn
            .query_row("SELECT code FROM subjects WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("subject should exist");
        let source = service
            .register_source_upload(SourceUploadInput {
                uploader_account_id: 1,
                source_kind: "curriculum".to_string(),
                title: "Ratio Reasoning Builder".to_string(),
                source_path: Some("C:/tmp/ratio-reasoning.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some(subject_code),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("draft".to_string()),
                metadata: json!({ "source_trust": "tier_a" }),
            })
            .expect("source should register");

        conn.execute(
            "INSERT INTO topics (
                subject_id, code, name, description, node_type, display_order,
                exam_weight, difficulty_band, importance_weight, is_active
             ) VALUES (1, 'AUTO-RATIO', 'Ratio Reasoning Foundations', 'Source-aware foundry target', 'topic', 1000, 4800, 'medium', 7600, 1)",
            [],
        )
        .expect("topic should insert");
        let topic_id = conn.last_insert_rowid();

        for (candidate_type, raw_label, payload) in [
            (
                "subject",
                "Mathematics",
                json!({
                    "topic_code": "AUTO-RATIO",
                    "topic_name": "Ratio Reasoning Foundations"
                }),
            ),
            (
                "topic",
                "Ratio Reasoning Foundations",
                json!({
                    "topic_code": "AUTO-RATIO",
                    "topic_name": "Ratio Reasoning Foundations"
                }),
            ),
            (
                "concept",
                "Ratio Table",
                json!({
                    "topic_code": "AUTO-RATIO",
                    "topic_name": "Ratio Reasoning Foundations",
                    "formal_description": "Use ratio tables to compare equivalent multiplicative relationships.",
                    "simple_text": "Use a ratio table to keep both quantities aligned.",
                    "core_meaning": "A ratio table preserves multiplicative comparison."
                }),
            ),
            (
                "objective",
                "Use a ratio table to compare part-to-whole relationships",
                json!({
                    "topic_code": "AUTO-RATIO",
                    "topic_name": "Ratio Reasoning Foundations",
                    "objective_text": "Use a ratio table to compare part-to-whole relationships",
                    "cognitive_level": "application"
                }),
            ),
            (
                "formula",
                "ratio = part:whole",
                json!({
                    "topic_code": "AUTO-RATIO",
                    "topic_name": "Ratio Reasoning Foundations",
                    "exam_hint": "Keep the order of the compared quantities fixed."
                }),
            ),
        ] {
            service
                .add_parse_candidate(
                    source.id,
                    ParseCandidateInput {
                        candidate_type: candidate_type.to_string(),
                        parent_candidate_id: None,
                        raw_label: raw_label.to_string(),
                        normalized_label: Some(raw_label.to_ascii_lowercase()),
                        payload,
                        confidence_score: 9200,
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

        conn.execute(
            "INSERT INTO content_acquisition_jobs (
                subject_id, topic_id, intent_type, query_text, source_scope, status,
                result_summary_json, completed_at
             ) VALUES (?1, ?2, 'example_hunt', 'ratio worked example', 'internal', 'completed', '{}', datetime('now'))",
            params![1, topic_id],
        )
        .expect("acquisition job should insert");
        let acquisition_job_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO acquisition_evidence_candidates (
                job_id, source_label, source_url, source_kind, title, snippet,
                extracted_payload_json, quality_score, freshness_score, review_status
             ) VALUES (?1, 'Teacher Guide', NULL, 'internal', 'Ratio table worked example',
                       'Scale both quantities by the same factor before comparing the missing term.',
                       '{}', 9100, 8200, 'approved')",
            [acquisition_job_id],
        )
        .expect("approved evidence should insert");

        service
            .seed_curriculum_support(topic_id)
            .expect("curriculum support should build");
        service
            .seed_knowledge_support(topic_id, "explanation")
            .expect("explanation should build");
        service
            .seed_knowledge_support(topic_id, "formula")
            .expect("formula should build");
        service
            .seed_knowledge_support(topic_id, "worked_example")
            .expect("worked example should build");
        service
            .seed_question_support(topic_id)
            .expect("question should build");
        service
            .seed_contrast_support(topic_id)
            .expect("contrast should build");

        let node_title: String = conn
            .query_row(
                "SELECT canonical_title
                 FROM academic_nodes
                 WHERE topic_id = ?1 AND canonical_title = 'Ratio Table'
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("source-aware node should exist");
        let objective_text: String = conn
            .query_row(
                "SELECT objective_text
                 FROM learning_objectives
                 WHERE topic_id = ?1
                 ORDER BY id ASC
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("objective should exist");
        let formula_title: String = conn
            .query_row(
                "SELECT title
                 FROM knowledge_entries
                 WHERE topic_id = ?1 AND entry_type = 'formula'
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("formula entry should exist");
        let worked_example_text: String = conn
            .query_row(
                "SELECT full_text
                 FROM knowledge_entries
                 WHERE topic_id = ?1 AND entry_type = 'worked_example'
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("worked example should exist");
        let (question_stem, question_explanation): (String, String) = conn
            .query_row(
                "SELECT stem, explanation_text
                 FROM questions
                 WHERE topic_id = ?1
                 LIMIT 1",
                [topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("generated question should exist");
        let question_link_count: i64 = conn
            .query_row(
                "SELECT COUNT(*)
                 FROM question_glossary_links links
                 JOIN questions questions ON questions.id = links.question_id
                 WHERE questions.topic_id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("question glossary links should count");
        let contrast_title: String = conn
            .query_row(
                "SELECT title
                 FROM contrast_pairs
                 WHERE topic_id = ?1
                 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("contrast pair should exist");

        assert_eq!(node_title, "Ratio Table");
        assert!(objective_text.contains("ratio table"));
        assert_eq!(formula_title, "ratio = part:whole");
        assert!(worked_example_text.contains("Scale both quantities"));
        assert!(question_stem.contains("ratio table") || question_stem.contains("Ratio Table"));
        assert!(question_explanation.contains("Scale both quantities"));
        assert!(question_link_count >= 1);
        assert!(contrast_title.contains("Formula") || contrast_title.contains("Worked Example"));
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
