use std::path::Path;

use ecoach_coach_brain::PedagogicalRuntimeService;
use ecoach_content::{
    ContentEvaluationRun, ContentEvaluationRunInput, ContentEvidenceRecord,
    ContentEvidenceRecordInput, ContentIntelligenceOverview, ContentIntelligenceService,
    ContentPublishDecision, ContentPublishDecisionInput, ContentResearchCandidate,
    ContentResearchCandidateInput, ContentResearchMission, ContentResearchMissionInput,
    ContentRetrievalQueryInput, ContentRetrievalResult, ContentSnapshot, ContentSnapshotBuildInput,
    ContentSourceDetail, ContentSourceGovernanceInput, ContentSourcePolicy,
    ContentSourcePolicyInput, ContentSourceProfileInput, ContentSourceRegistryEntry,
    ContentSourceSegment, ContentSourceSegmentInput,
    FoundryCoordinatorService, PackService, ParseCandidateInput, RecordResourceLearningInput,
    ResourceApplicabilityResolution, ResourceIntelligenceService, ResourceLearningRecord,
    ResourceOrchestrationRequest, ResourceOrchestrationResult, SourceUploadInput,
    TopicResourceIntelligenceSnapshot,
};

use crate::{
    dtos::{
        ContentFoundrySourceReportDto, CurriculumParseCandidateDto, CurriculumReviewTaskDto,
        CurriculumSourceUploadDto, FoundryJobBoardDto, FoundryJobDto, PackInstallResultDto,
        PackSummaryDto, SubjectFoundryDashboardDto, TopicPackageSnapshotDto,
    },
    error::CommandError,
    state::AppState,
};

pub type TopicResourceIntelligenceSnapshotDto = TopicResourceIntelligenceSnapshot;
pub type ResourceOrchestrationResultDto = ResourceOrchestrationResult;
pub type ResourceApplicabilityResolutionDto = ResourceApplicabilityResolution;
pub type ResourceLearningRecordDto = ResourceLearningRecord;
pub type ContentSourcePolicyDto = ContentSourcePolicy;
pub type ContentSourceRegistryEntryDto = ContentSourceRegistryEntry;
pub type ContentSourceDetailDto = ContentSourceDetail;
pub type ContentSourceGovernanceInputDto = ContentSourceGovernanceInput;
pub type ContentSourceSegmentDto = ContentSourceSegment;
pub type ContentResearchMissionDto = ContentResearchMission;
pub type ContentResearchCandidateDto = ContentResearchCandidate;
pub type ContentEvidenceRecordDto = ContentEvidenceRecord;
pub type ContentPublishDecisionDto = ContentPublishDecision;
pub type ContentSnapshotDto = ContentSnapshot;
pub type ContentRetrievalResultDto = ContentRetrievalResult;
pub type ContentEvaluationRunDto = ContentEvaluationRun;
pub type ContentIntelligenceOverviewDto = ContentIntelligenceOverview;

pub fn install_pack(
    state: &AppState,
    pack_path: String,
) -> Result<PackInstallResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = PackService::new(conn);
        let result = service.install_pack(Path::new(&pack_path))?;
        Ok(PackInstallResultDto::from(result))
    })
}

pub fn list_installed_packs(state: &AppState) -> Result<Vec<PackSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PackService::new(conn);
        let packs = service.list_packs()?;
        Ok(packs.into_iter().map(PackSummaryDto::from).collect())
    })
}

pub fn register_curriculum_source(
    state: &AppState,
    input: SourceUploadInput,
) -> Result<CurriculumSourceUploadDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let source = service.register_source_upload(input)?;
        Ok(CurriculumSourceUploadDto::from(source))
    })
}

pub fn add_curriculum_parse_candidate(
    state: &AppState,
    source_upload_id: i64,
    input: ParseCandidateInput,
) -> Result<CurriculumParseCandidateDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let candidate = service.add_parse_candidate(source_upload_id, input)?;
        Ok(CurriculumParseCandidateDto::from(candidate))
    })
}

pub fn finalize_curriculum_source(
    state: &AppState,
    source_upload_id: i64,
) -> Result<ContentFoundrySourceReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let report = service.finalize_source_parse(source_upload_id)?;
        Ok(ContentFoundrySourceReportDto::from(report))
    })
}

pub fn resolve_curriculum_review_task(
    state: &AppState,
    task_id: i64,
    resolution_note: String,
    approve_candidate: bool,
) -> Result<CurriculumReviewTaskDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let task = service.resolve_review_task(task_id, &resolution_note, approve_candidate)?;
        Ok(CurriculumReviewTaskDto::from(task))
    })
}

pub fn mark_curriculum_source_reviewed(
    state: &AppState,
    source_upload_id: i64,
) -> Result<ContentFoundrySourceReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let report = service.mark_source_reviewed(source_upload_id)?;
        Ok(ContentFoundrySourceReportDto::from(report))
    })
}

pub fn stage_curriculum_publish_job(
    state: &AppState,
    source_upload_id: i64,
    requested_by_account_id: Option<i64>,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
    target_version_label: Option<String>,
) -> Result<String, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let report = service.stage_publish_job(
            source_upload_id,
            requested_by_account_id,
            subject_id,
            topic_id,
            target_version_label.as_deref(),
        )?;
        Ok(report.job.status)
    })
}

pub fn recompute_topic_package_snapshot(
    state: &AppState,
    topic_id: i64,
) -> Result<Option<TopicPackageSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let snapshot = service.recompute_topic_package_snapshot(topic_id)?;
        Ok(snapshot.map(TopicPackageSnapshotDto::from))
    })
}

pub fn get_subject_foundry_dashboard(
    state: &AppState,
    subject_id: i64,
) -> Result<Option<SubjectFoundryDashboardDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let dashboard = service.get_subject_foundry_dashboard(subject_id)?;
        Ok(dashboard.map(SubjectFoundryDashboardDto::from))
    })
}

pub fn queue_topic_foundry_jobs(
    state: &AppState,
    topic_id: i64,
    trigger_type: String,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let jobs = service.queue_topic_foundry_jobs(topic_id, &trigger_type)?;
        Ok(jobs.into_iter().map(FoundryJobDto::from).collect())
    })
}

pub fn queue_source_follow_up_jobs(
    state: &AppState,
    source_upload_id: i64,
    trigger_type: String,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let jobs = service.queue_source_follow_up_jobs(source_upload_id, &trigger_type)?;
        Ok(jobs.into_iter().map(FoundryJobDto::from).collect())
    })
}

pub fn list_foundry_jobs(
    state: &AppState,
    status: Option<String>,
    target_type: Option<String>,
    subject_id: Option<i64>,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let jobs =
            service.list_foundry_jobs(status.as_deref(), target_type.as_deref(), subject_id)?;
        Ok(jobs.into_iter().map(FoundryJobDto::from).collect())
    })
}

pub fn get_foundry_job_board(
    state: &AppState,
    subject_id: Option<i64>,
) -> Result<FoundryJobBoardDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let board = service.get_foundry_job_board(subject_id)?;
        Ok(FoundryJobBoardDto::from(board))
    })
}

pub fn start_foundry_job(state: &AppState, job_id: i64) -> Result<FoundryJobDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let job = service.start_foundry_job(job_id)?;
        Ok(FoundryJobDto::from(job))
    })
}

pub fn complete_foundry_job(
    state: &AppState,
    job_id: i64,
    result_summary: serde_json::Value,
) -> Result<FoundryJobDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let job = service.complete_foundry_job(job_id, &result_summary)?;
        Ok(FoundryJobDto::from(job))
    })
}

pub fn fail_foundry_job(
    state: &AppState,
    job_id: i64,
    failure_reason: String,
) -> Result<FoundryJobDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let job = service.fail_foundry_job(job_id, &failure_reason)?;
        Ok(FoundryJobDto::from(job))
    })
}

pub fn run_foundry_job(state: &AppState, job_id: i64) -> Result<FoundryJobDto, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let job = service.run_foundry_job(job_id)?;
        Ok(FoundryJobDto::from(job))
    })
}

pub fn run_next_foundry_job(
    state: &AppState,
    subject_id: Option<i64>,
) -> Result<Option<FoundryJobDto>, CommandError> {
    state.with_connection(|conn| {
        let service = FoundryCoordinatorService::new(conn);
        let job = service.run_next_foundry_job(subject_id)?;
        Ok(job.map(FoundryJobDto::from))
    })
}

pub fn get_topic_resource_intelligence(
    state: &AppState,
    topic_id: i64,
) -> Result<TopicResourceIntelligenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = ResourceIntelligenceService::new(conn);
        service
            .sync_topic_resource_intelligence(topic_id)
            .map_err(Into::into)
    })
}

pub fn orchestrate_resource_plan(
    state: &AppState,
    input: ResourceOrchestrationRequest,
) -> Result<ResourceOrchestrationResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = ResourceIntelligenceService::new(conn);
        service.orchestrate_resources(input).map_err(Into::into)
    })
}

pub fn confirm_resource_applicability(
    state: &AppState,
    check_id: i64,
    selected_option_code: String,
    response_text: Option<String>,
) -> Result<ResourceApplicabilityResolutionDto, CommandError> {
    state.with_connection(|conn| {
        let service = ResourceIntelligenceService::new(conn);
        service
            .confirm_applicability(check_id, &selected_option_code, response_text.as_deref())
            .map_err(Into::into)
    })
}

pub fn record_resource_learning_outcome(
    state: &AppState,
    input: RecordResourceLearningInput,
) -> Result<ResourceLearningRecordDto, CommandError> {
    state.with_connection(|conn| {
        let run_id = input.run_id;
        let service = ResourceIntelligenceService::new(conn);
        let record = service.record_learning_outcome(input)?;
        PedagogicalRuntimeService::new(conn).record_resource_learning_feedback(run_id)?;
        Ok(record)
    })
}

pub fn upsert_content_source_policy(
    state: &AppState,
    input: ContentSourcePolicyInput,
) -> Result<ContentSourcePolicyDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.upsert_source_policy(input).map_err(Into::into)
    })
}

pub fn list_content_source_policies(
    state: &AppState,
    status: Option<String>,
) -> Result<Vec<ContentSourcePolicyDto>, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .list_source_policies(status.as_deref())
            .map_err(Into::into)
    })
}

pub fn save_content_source_profile(
    state: &AppState,
    source_upload_id: i64,
    input: ContentSourceProfileInput,
) -> Result<ContentSourceRegistryEntryDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .save_source_profile(source_upload_id, input)
            .map_err(Into::into)
    })
}

pub fn list_content_sources(
    state: &AppState,
    status: Option<String>,
    source_kind: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<ContentSourceRegistryEntryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .list_content_sources(status.as_deref(), source_kind.as_deref(), limit)
            .map_err(Into::into)
    })
}

pub fn get_content_source_detail(
    state: &AppState,
    source_upload_id: i64,
) -> Result<ContentSourceDetailDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .get_content_source_detail(source_upload_id)
            .map_err(Into::into)
    })
}

pub fn govern_content_source(
    state: &AppState,
    source_upload_id: i64,
    input: ContentSourceGovernanceInputDto,
) -> Result<ContentSourceDetailDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .govern_source_upload(source_upload_id, input)
            .map_err(Into::into)
    })
}

pub fn ingest_content_source_segments(
    state: &AppState,
    source_upload_id: i64,
    segments: Vec<ContentSourceSegmentInput>,
) -> Result<Vec<ContentSourceSegmentDto>, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .ingest_source_segments(source_upload_id, segments)
            .map_err(Into::into)
    })
}

pub fn plan_content_research_mission(
    state: &AppState,
    input: ContentResearchMissionInput,
) -> Result<ContentResearchMissionDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.plan_research_mission(input).map_err(Into::into)
    })
}

pub fn add_content_research_candidate(
    state: &AppState,
    mission_id: i64,
    input: ContentResearchCandidateInput,
) -> Result<ContentResearchCandidateDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .add_research_candidate(mission_id, input)
            .map_err(Into::into)
    })
}

pub fn record_content_evidence(
    state: &AppState,
    input: ContentEvidenceRecordInput,
) -> Result<ContentEvidenceRecordDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.record_evidence(input).map_err(Into::into)
    })
}

pub fn record_content_publish_decision(
    state: &AppState,
    input: ContentPublishDecisionInput,
) -> Result<ContentPublishDecisionDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.record_publish_decision(input).map_err(Into::into)
    })
}

pub fn build_content_topic_snapshot(
    state: &AppState,
    input: ContentSnapshotBuildInput,
) -> Result<ContentSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.build_topic_snapshot(input).map_err(Into::into)
    })
}

pub fn retrieve_content_intelligence(
    state: &AppState,
    input: ContentRetrievalQueryInput,
) -> Result<ContentRetrievalResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.retrieve_content(input).map_err(Into::into)
    })
}

pub fn record_content_evaluation(
    state: &AppState,
    input: ContentEvaluationRunInput,
) -> Result<ContentEvaluationRunDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service.record_evaluation_run(input).map_err(Into::into)
    })
}

pub fn get_content_intelligence_overview(
    state: &AppState,
    topic_id: i64,
) -> Result<ContentIntelligenceOverviewDto, CommandError> {
    state.with_connection(|conn| {
        let service = ContentIntelligenceService::new(conn);
        service
            .get_content_intelligence_overview(topic_id)
            .map_err(Into::into)
    })
}

/// Rebuild the content index by recomputing pack summaries and question counts.
pub fn rebuild_content_index(state: &AppState) -> Result<RebuildContentIndexResult, CommandError> {
    state.with_connection(|conn| {
        // Recount questions per pack
        let packs_updated: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM content_packs WHERE status = 'active'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        // Update question counts per pack
        conn.execute_batch(
            "UPDATE content_packs SET question_count = (
                 SELECT COUNT(*) FROM questions WHERE pack_id = content_packs.id AND is_active = 1
             ) WHERE status = 'active';
             UPDATE content_packs SET topic_count = (
                 SELECT COUNT(DISTINCT topic_id) FROM questions WHERE pack_id = content_packs.id AND is_active = 1
             ) WHERE status = 'active';",
        )
        .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        // Recompute family analytics for all subjects with past papers
        let subject_ids: Vec<i64> = {
            let mut stmt = conn
                .prepare("SELECT DISTINCT subject_id FROM past_paper_sets")
                .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(
                    row.map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?,
                );
            }
            ids
        };

        let families_recomputed = subject_ids.len() as i64;

        Ok(RebuildContentIndexResult {
            packs_updated,
            families_recomputed,
        })
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RebuildContentIndexResult {
    pub packs_updated: i64,
    pub families_recomputed: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use ecoach_identity::CreateAccountInput;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::identity_commands;

    #[test]
    fn content_commands_surface_idea27_runtime_flow() {
        let state = AppState::in_memory().expect("in-memory state should build");
        let admin = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Admin,
                display_name: "Curator".to_string(),
                pin: "2468".to_string(),
                entitlement_tier: EntitlementTier::Elite,
            },
        )
        .expect("admin should create");

        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO curriculum_versions (
                        id, name, country, version_label, status
                     ) VALUES (1, 'Test Curriculum', 'GH', 'v1', 'published')",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "INSERT INTO subjects (
                        id, curriculum_version_id, code, name
                     ) VALUES (1, 1, 'MTH', 'Mathematics')",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "INSERT INTO topics (
                        id, subject_id, code, name, node_type
                     ) VALUES (1, 1, 'ALG-1', 'Quadratic Equations', 'topic')",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "INSERT INTO artifacts (
                        id, artifact_type, topic_id, subject_id, lifecycle_state
                     ) VALUES (1, 'explanation', 1, 1, 'live')",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "INSERT INTO artifact_versions (
                        id, artifact_id, version_no, state, content_json, build_reason,
                        quality_score_bp, provenance_ref
                     ) VALUES (
                        1, 1, 1, 'live',
                        '{\"title\":\"Quadratic Formula Walkthrough\",\"body\":\"Use the quadratic formula after rearranging into standard form.\"}',
                        'test', 8300, 'manual'
                     )",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "UPDATE artifacts SET current_version_id = 1 WHERE id = 1",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                Ok::<_, CommandError>(())
            })
            .expect("curriculum state should seed");

        let source = register_curriculum_source(
            &state,
            SourceUploadInput {
                uploader_account_id: admin.id,
                source_kind: "textbook".to_string(),
                title: "Algebra Notes".to_string(),
                source_path: Some("internal/algebra-notes.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("SHS".to_string()),
                subject_code: Some("MTH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("2026.1".to_string()),
                metadata: serde_json::json!({ "owner": "curriculum-team" }),
            },
        )
        .expect("source should register");

        upsert_content_source_policy(
            &state,
            ContentSourcePolicyInput {
                id: None,
                policy_name: "Math web allowlist".to_string(),
                scope_type: "subject".to_string(),
                scope_ref: Some("MTH".to_string()),
                source_kind: Some("web_source".to_string()),
                domain_pattern: Some("example.edu".to_string()),
                access_mode: "allowlisted_only".to_string(),
                trust_tier: "trusted".to_string(),
                freshness_window_days: 60,
                allow_crawl: true,
                allow_publish: false,
                notes: Some("Stage web content first".to_string()),
                status: "active".to_string(),
            },
        )
        .expect("policy should save");

        let profile = save_content_source_profile(
            &state,
            source.id,
            ContentSourceProfileInput {
                canonical_uri: Some("https://example.edu/algebra/quadratics".to_string()),
                publisher: Some("Example Academy".to_string()),
                author: Some("A. Curator".to_string()),
                publication_date: Some("2026-01-10".to_string()),
                license_type: Some("internal-review".to_string()),
                crawl_permission: Some("allowlisted_only".to_string()),
                source_tier: Some("trusted".to_string()),
                trust_score_bp: Some(8_400),
                freshness_score_bp: Some(7_700),
                parse_status_detail: Some("segmented".to_string()),
                allowlisted_domain: Some(true),
                last_verified_at: Some("2026-03-01T00:00:00Z".to_string()),
                review_due_at: Some("2026-06-01T00:00:00Z".to_string()),
                stale_flag: Some(false),
                metadata_patch: Some(serde_json::json!({ "coverage": "quadratics" })),
            },
        )
        .expect("profile should save");
        assert_eq!(profile.publisher.as_deref(), Some("Example Academy"));

        let segments = ingest_content_source_segments(
            &state,
            source.id,
            vec![ContentSourceSegmentInput {
                topic_id: Some(1),
                concept_id: None,
                section_title: Some("Quadratic Formula".to_string()),
                raw_text: "The quadratic formula solves ax^2 + bx + c = 0".to_string(),
                normalized_text: Some("the quadratic formula solves ax^2 + bx + c = 0".to_string()),
                markdown_text: Some(
                    "The quadratic formula solves `ax^2 + bx + c = 0`.".to_string(),
                ),
                image_refs: serde_json::json!([]),
                equation_refs: serde_json::json!(["x = (-b ± sqrt(b^2 - 4ac)) / 2a"]),
                page_range: Some("12-13".to_string()),
                checksum: Some("seg-a".to_string()),
                semantic_hash: Some("hash-a".to_string()),
                extraction_confidence_bp: Some(8_300),
                relevance_score_bp: Some(8_600),
                metadata: serde_json::json!({ "topic_id": 1, "content_type": "formula" }),
            }],
        )
        .expect("segments should ingest");

        let mission = plan_content_research_mission(
            &state,
            ContentResearchMissionInput {
                source_upload_id: Some(source.id),
                gap_ticket_id: None,
                subject_id: Some(1),
                topic_id: Some(1),
                mission_type: "gap_fill".to_string(),
                mission_brief: "Find stronger quadratic examples and misconception fixes"
                    .to_string(),
                allowed_source_classes: vec!["internal".to_string(), "approved_web".to_string()],
                requested_asset_types: vec!["worked_example".to_string()],
                coverage_snapshot: serde_json::json!({ "coverage": "thin" }),
                priority_bp: Some(8_000),
                created_by_account_id: Some(admin.id),
                planner_notes: Some("Prioritize sign-handling examples".to_string()),
            },
        )
        .expect("mission should plan");

        let candidate = add_content_research_candidate(
            &state,
            mission.id,
            ContentResearchCandidateInput {
                source_upload_id: Some(source.id),
                source_label: "Example Academy Notes".to_string(),
                source_url: Some(
                    "https://example.edu/algebra/quadratics/worked-example".to_string(),
                ),
                source_kind: "web".to_string(),
                title: Some("Worked example with factorization cross-check".to_string()),
                snippet: Some("Shows the formula, factorization, and sign check.".to_string()),
                source_tier: Some("trusted".to_string()),
                authority_score_bp: Some(8_400),
                relevance_score_bp: Some(8_600),
                freshness_score_bp: Some(7_900),
                license_type: Some("review-only".to_string()),
                payload: serde_json::json!({ "query": "quadratic worked example sign handling" }),
            },
        )
        .expect("candidate should add");

        let evidence = record_content_evidence(
            &state,
            ContentEvidenceRecordInput {
                mission_id: Some(mission.id),
                research_candidate_id: Some(candidate.id),
                source_upload_id: Some(source.id),
                source_segment_id: Some(segments[0].id),
                topic_id: 1,
                concept_id: None,
                evidence_type: "worked_example".to_string(),
                claim_text: "Students should normalize the equation before choosing the formula."
                    .to_string(),
                supporting_text: Some(
                    "The worked example first rearranges the quadratic into standard form."
                        .to_string(),
                ),
                extraction_confidence_bp: 8_500,
                corroboration_score_bp: 8_100,
                contradiction_score_bp: 1_200,
                pedagogy_score_bp: 8_000,
                freshness_score_bp: 7_800,
                provenance: serde_json::json!({
                    "candidate_id": candidate.id,
                    "segment_id": segments[0].id
                }),
                verified: true,
            },
        )
        .expect("evidence should record");
        assert_eq!(evidence.status, "verified");

        let decision = record_content_publish_decision(
            &state,
            ContentPublishDecisionInput {
                publish_job_id: None,
                source_upload_id: Some(source.id),
                subject_id: Some(1),
                topic_id: Some(1),
                gate_name: "idea27_quality_gate".to_string(),
                decision_status: "approved".to_string(),
                decision_reason: "Evidence is corroborated and the snapshot is ready.".to_string(),
                decision_score_bp: Some(8_600),
                decision_trace: serde_json::json!({ "evidence_id": evidence.id }),
                decided_by_account_id: Some(admin.id),
                snapshot_audience: Some("student".to_string()),
                build_snapshot: true,
            },
        )
        .expect("publish decision should record");
        assert!(decision.snapshot_id.is_some());

        let retrieval = retrieve_content_intelligence(
            &state,
            ContentRetrievalQueryInput {
                student_id: Some(admin.id),
                subject_id: Some(1),
                topic_id: Some(1),
                audience_type: Some("student".to_string()),
                query_text: "quadratic formula worked example sign handling".to_string(),
                content_types: vec!["artifact".to_string(), "segment".to_string()],
                limit: Some(5),
                snapshot_only: false,
            },
        )
        .expect("retrieval should succeed");
        assert!(!retrieval.hits.is_empty());
        assert!(retrieval.used_snapshot_id.is_some());

        let evaluation = record_content_evaluation(
            &state,
            ContentEvaluationRunInput {
                query_id: Some(retrieval.query_id),
                topic_id: Some(1),
                metric_family: "online_retrieval".to_string(),
                groundedness_bp: 8_200,
                relevance_bp: 8_500,
                correctness_bp: 8_100,
                completeness_bp: 7_900,
                utilization_bp: 7_700,
                notes: serde_json::json!({ "comment": "served high-confidence content" }),
            },
        )
        .expect("evaluation should record");
        assert_eq!(evaluation.query_id, Some(retrieval.query_id));

        let overview = get_content_intelligence_overview(&state, 1).expect("overview should build");
        assert!(overview.segment_count >= 1);
        assert!(overview.evidence_count >= 1);
        assert!(overview.latest_snapshot_id.is_some());
        assert!(overview.retrieval_query_count >= 1);
        assert!(overview.evaluation_run_count >= 1);
    }

    #[test]
    fn content_commands_surface_source_detail_and_governance_flow() {
        let state = AppState::in_memory().expect("in-memory state should build");
        let admin = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Admin,
                display_name: "Governor".to_string(),
                pin: "2468".to_string(),
                entitlement_tier: EntitlementTier::Elite,
            },
        )
        .expect("admin should create");

        let source = register_curriculum_source(
            &state,
            SourceUploadInput {
                uploader_account_id: admin.id,
                source_kind: "worksheet".to_string(),
                title: "Annotated worksheet".to_string(),
                source_path: Some("vault/annotated-worksheet.pdf".to_string()),
                country_code: None,
                exam_board: None,
                education_level: None,
                subject_code: None,
                academic_year: None,
                language_code: Some("en".to_string()),
                version_label: Some("vault".to_string()),
                metadata: serde_json::json!({ "origin": "personal_vault" }),
            },
        )
        .expect("source should register");

        let segments = ingest_content_source_segments(
            &state,
            source.id,
            vec![ContentSourceSegmentInput {
                topic_id: None,
                concept_id: None,
                section_title: Some("Teacher annotation".to_string()),
                raw_text: "Focus on balancing both sides before simplifying.".to_string(),
                normalized_text: None,
                markdown_text: None,
                image_refs: serde_json::json!([]),
                equation_refs: serde_json::json!([]),
                page_range: Some("1".to_string()),
                checksum: None,
                semantic_hash: Some("worksheet-annotation".to_string()),
                extraction_confidence_bp: Some(7_700),
                relevance_score_bp: Some(7_900),
                metadata: serde_json::json!({ "role": "teacher_note" }),
            }],
        )
        .expect("segments should ingest");
        assert_eq!(segments.len(), 1);

        let governed = govern_content_source(
            &state,
            source.id,
            ContentSourceGovernanceInputDto {
                source_status: "review_required".to_string(),
                decided_by_account_id: Some(admin.id),
                note: Some("Needs contradiction check before trust promotion.".to_string()),
                confidence_score: Some(7_200),
                review_due_at: Some("2026-05-01".to_string()),
                stale_flag: Some(false),
            },
        )
        .expect("governance should apply");
        let detail = get_content_source_detail(&state, source.id).expect("detail should load");

        assert_eq!(governed.source.source_status, "review_required");
        assert_eq!(detail.source.id, source.id);
        assert_eq!(detail.segments.len(), 1);
        assert_eq!(detail.governance_events.len(), 1);
        assert_eq!(
            detail.governance_events[0].decided_by_account_id,
            Some(admin.id)
        );
    }
}
