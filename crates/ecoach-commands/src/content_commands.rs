use std::path::Path;

use ecoach_content::{
    FoundryCoordinatorService, PackService, ParseCandidateInput, SourceUploadInput,
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
