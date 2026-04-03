use ecoach_curriculum::{
    CurriculumAdminNodeDetail, CurriculumCohortPin, CurriculumCohortPinInput,
    CurriculumCoverageStats, CurriculumFamily, CurriculumFamilyInput, CurriculumImpactAnalysis,
    CurriculumIngestionWorkspace, CurriculumLevel, CurriculumLevelInput, CurriculumLinkedResource,
    CurriculumNodeBundle, CurriculumNodeBundleInput, CurriculumNodeCitation,
    CurriculumNodeCitationInput, CurriculumNodeComment, CurriculumNodeCommentInput,
    CurriculumNodeExemplar, CurriculumNodeExemplarInput, CurriculumNodeIntelligence,
    CurriculumNodeIntelligenceInput, CurriculumParentSummary, CurriculumPrerequisiteStep,
    CurriculumPublicSubjectOverview, CurriculumPublicTopicDetail, CurriculumPublishResult,
    CurriculumRecommendation, CurriculumRegenerationJob, CurriculumRegistryEntry,
    CurriculumRemediationMap, CurriculumReviewQueueItem, CurriculumSearchResult, CurriculumService,
    CurriculumSourceReport, CurriculumSourceUpload, CurriculumStudentHomeSnapshot,
    CurriculumStudentSubjectMap, CurriculumSubjectTrack, CurriculumSubjectTrackInput,
    CurriculumTermPeriod, CurriculumTermPeriodInput, CurriculumTopicContext, CurriculumTreeNode,
    CurriculumVersion, CurriculumVersionDiffReport, CurriculumVersionInput,
    StudentCurriculumAssignment, StudentCurriculumAssignmentInput, Subject, TopicSummary,
};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectDto {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub code: String,
    pub name: String,
    pub display_order: i64,
}

impl From<Subject> for SubjectDto {
    fn from(v: Subject) -> Self {
        Self {
            id: v.id,
            curriculum_version_id: v.curriculum_version_id,
            code: v.code,
            name: v.name,
            display_order: v.display_order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDto {
    pub id: i64,
    pub subject_id: i64,
    pub parent_topic_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub node_type: String,
    pub display_order: i64,
}

impl From<TopicSummary> for TopicDto {
    fn from(v: TopicSummary) -> Self {
        Self {
            id: v.id,
            subject_id: v.subject_id,
            parent_topic_id: v.parent_topic_id,
            code: v.code,
            name: v.name,
            node_type: v.node_type,
            display_order: v.display_order,
        }
    }
}

pub type CurriculumFamilyDto = CurriculumFamily;
pub type CurriculumVersionDto = CurriculumVersion;
pub type CurriculumSubjectTrackDto = CurriculumSubjectTrack;
pub type CurriculumLevelDto = CurriculumLevel;
pub type CurriculumTermPeriodDto = CurriculumTermPeriod;
pub type CurriculumNodeBundleDto = CurriculumNodeBundle;
pub type CurriculumSourceUploadDto = CurriculumSourceUpload;
pub type CurriculumSourceReportDto = CurriculumSourceReport;
pub type CurriculumReviewQueueItemDto = CurriculumReviewQueueItem;
pub type CurriculumPublishResultDto = CurriculumPublishResult;
pub type CurriculumVersionDiffReportDto = CurriculumVersionDiffReport;
pub type CurriculumPublicSubjectOverviewDto = CurriculumPublicSubjectOverview;
pub type CurriculumTreeNodeDto = CurriculumTreeNode;
pub type CurriculumPublicTopicDetailDto = CurriculumPublicTopicDetail;
pub type CurriculumTopicContextDto = CurriculumTopicContext;
pub type CurriculumRecommendationDto = CurriculumRecommendation;
pub type CurriculumPrerequisiteStepDto = CurriculumPrerequisiteStep;
pub type CurriculumRemediationMapDto = CurriculumRemediationMap;
pub type CurriculumCoverageStatsDto = CurriculumCoverageStats;
pub type CurriculumSearchResultDto = CurriculumSearchResult;
pub type CurriculumLinkedResourceDto = CurriculumLinkedResource;
pub type CurriculumNodeCitationDto = CurriculumNodeCitation;
pub type CurriculumNodeExemplarDto = CurriculumNodeExemplar;
pub type CurriculumNodeCommentDto = CurriculumNodeComment;
pub type CurriculumNodeIntelligenceDto = CurriculumNodeIntelligence;
pub type CurriculumRegistryEntryDto = CurriculumRegistryEntry;
pub type CurriculumIngestionWorkspaceDto = CurriculumIngestionWorkspace;
pub type CurriculumAdminNodeDetailDto = CurriculumAdminNodeDetail;
pub type CurriculumImpactAnalysisDto = CurriculumImpactAnalysis;
pub type CurriculumRegenerationJobDto = CurriculumRegenerationJob;
pub type CurriculumCohortPinDto = CurriculumCohortPin;
pub type StudentCurriculumAssignmentDto = StudentCurriculumAssignment;
pub type CurriculumStudentHomeSnapshotDto = CurriculumStudentHomeSnapshot;
pub type CurriculumStudentSubjectMapDto = CurriculumStudentSubjectMap;
pub type CurriculumParentSummaryDto = CurriculumParentSummary;
pub type CurriculumFamilyInputDto = CurriculumFamilyInput;
pub type CurriculumVersionInputDto = CurriculumVersionInput;
pub type CurriculumSubjectTrackInputDto = CurriculumSubjectTrackInput;
pub type CurriculumLevelInputDto = CurriculumLevelInput;
pub type CurriculumTermPeriodInputDto = CurriculumTermPeriodInput;
pub type CurriculumNodeBundleInputDto = CurriculumNodeBundleInput;
pub type CurriculumNodeCitationInputDto = CurriculumNodeCitationInput;
pub type CurriculumNodeExemplarInputDto = CurriculumNodeExemplarInput;
pub type CurriculumNodeCommentInputDto = CurriculumNodeCommentInput;
pub type CurriculumNodeIntelligenceInputDto = CurriculumNodeIntelligenceInput;
pub type CurriculumCohortPinInputDto = CurriculumCohortPinInput;
pub type StudentCurriculumAssignmentInputDto = StudentCurriculumAssignmentInput;

pub fn list_subjects(
    state: &AppState,
    curriculum_version_id: i64,
) -> Result<Vec<SubjectDto>, CommandError> {
    state.with_connection(|conn| {
        let service = CurriculumService::new(conn);
        let subjects = service.get_subjects(curriculum_version_id)?;
        Ok(subjects.into_iter().map(SubjectDto::from).collect())
    })
}

pub fn list_topics(state: &AppState, subject_id: i64) -> Result<Vec<TopicDto>, CommandError> {
    state.with_connection(|conn| {
        let service = CurriculumService::new(conn);
        let topics = service.list_topics_for_subject(subject_id)?;
        Ok(topics.into_iter().map(TopicDto::from).collect())
    })
}

pub fn list_curriculum_families(
    state: &AppState,
) -> Result<Vec<CurriculumFamilyDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_curriculum_families()
            .map_err(Into::into)
    })
}

pub fn save_curriculum_family(
    state: &AppState,
    input: CurriculumFamilyInput,
) -> Result<CurriculumFamilyDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_family(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_version(
    state: &AppState,
    input: CurriculumVersionInput,
) -> Result<CurriculumVersionDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_version(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_subject_track(
    state: &AppState,
    input: CurriculumSubjectTrackInput,
) -> Result<CurriculumSubjectTrackDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_subject_track(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_level(
    state: &AppState,
    input: CurriculumLevelInput,
) -> Result<CurriculumLevelDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_level(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_term_period(
    state: &AppState,
    input: CurriculumTermPeriodInput,
) -> Result<CurriculumTermPeriodDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_term_period(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_node_bundle(
    state: &AppState,
    input: CurriculumNodeBundleInput,
) -> Result<CurriculumNodeBundleDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_node_bundle(input)
            .map_err(Into::into)
    })
}

pub fn approve_curriculum_node(
    state: &AppState,
    node_id: i64,
    reviewer_note: Option<String>,
) -> Result<CurriculumNodeBundleDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .approve_curriculum_node(node_id, reviewer_note.as_deref())
            .map_err(Into::into)
    })
}

pub fn save_curriculum_node_citation(
    state: &AppState,
    input: CurriculumNodeCitationInput,
) -> Result<CurriculumNodeCitationDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_node_citation(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_node_exemplar(
    state: &AppState,
    input: CurriculumNodeExemplarInput,
) -> Result<CurriculumNodeExemplarDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_node_exemplar(input)
            .map_err(Into::into)
    })
}

pub fn save_curriculum_node_comment(
    state: &AppState,
    input: CurriculumNodeCommentInput,
) -> Result<CurriculumNodeCommentDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .save_curriculum_node_comment(input)
            .map_err(Into::into)
    })
}

pub fn upsert_curriculum_node_intelligence(
    state: &AppState,
    input: CurriculumNodeIntelligenceInput,
) -> Result<CurriculumNodeIntelligenceDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .upsert_curriculum_node_intelligence(input)
            .map_err(Into::into)
    })
}

pub fn list_curriculum_source_uploads(
    state: &AppState,
    status: Option<String>,
) -> Result<Vec<CurriculumSourceUploadDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_source_uploads(status.as_deref())
            .map_err(Into::into)
    })
}

pub fn get_curriculum_source_report(
    state: &AppState,
    source_upload_id: i64,
) -> Result<Option<CurriculumSourceReportDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_source_report(source_upload_id)
            .map_err(Into::into)
    })
}

pub fn list_curriculum_review_queue(
    state: &AppState,
    limit: i64,
) -> Result<Vec<CurriculumReviewQueueItemDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_curriculum_review_queue(limit)
            .map_err(Into::into)
    })
}

pub fn publish_curriculum_version(
    state: &AppState,
    curriculum_version_id: i64,
    generated_by_account_id: Option<i64>,
    notes: Option<String>,
) -> Result<CurriculumPublishResultDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .publish_curriculum_version(
                curriculum_version_id,
                generated_by_account_id,
                notes.as_deref(),
            )
            .map_err(Into::into)
    })
}

pub fn get_curriculum_version_diff(
    state: &AppState,
    base_version_id: i64,
    compare_version_id: i64,
) -> Result<CurriculumVersionDiffReportDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_version_diff(base_version_id, compare_version_id)
            .map_err(Into::into)
    })
}

pub fn list_public_curriculum_subjects(
    state: &AppState,
    family_slug: String,
    version_label: String,
) -> Result<Vec<CurriculumSubjectTrackDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_public_curriculum_subjects(&family_slug, &version_label)
            .map_err(Into::into)
    })
}

pub fn get_public_curriculum_subject_overview(
    state: &AppState,
    family_slug: String,
    version_label: String,
    subject_slug: String,
) -> Result<CurriculumPublicSubjectOverviewDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_public_curriculum_subject_overview(&family_slug, &version_label, &subject_slug)
            .map_err(Into::into)
    })
}

pub fn get_public_curriculum_subject_tree(
    state: &AppState,
    family_slug: String,
    version_label: String,
    subject_slug: String,
) -> Result<Vec<CurriculumTreeNodeDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_public_curriculum_subject_tree(&family_slug, &version_label, &subject_slug)
            .map_err(Into::into)
    })
}

pub fn get_public_curriculum_topic_detail_by_slug(
    state: &AppState,
    slug: String,
) -> Result<Option<CurriculumPublicTopicDetailDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_public_curriculum_topic_detail_by_slug(&slug)
            .map_err(Into::into)
    })
}

pub fn search_curriculum(
    state: &AppState,
    query: String,
    published_only: bool,
    limit: i64,
) -> Result<Vec<CurriculumSearchResultDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .search_curriculum(&query, published_only, limit)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_topic_resources(
    state: &AppState,
    node_id: i64,
) -> Result<Vec<CurriculumLinkedResourceDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_topic_resources(node_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_topic_context(
    state: &AppState,
    node_id: i64,
) -> Result<CurriculumTopicContextDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_topic_context(node_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_next_best_topics(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    limit: i64,
) -> Result<Vec<CurriculumRecommendationDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_next_best_topics(student_id, subject_id, limit)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_prerequisite_chain(
    state: &AppState,
    node_id: i64,
) -> Result<Vec<CurriculumPrerequisiteStepDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_prerequisite_chain(node_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_remediation_map(
    state: &AppState,
    node_id: i64,
) -> Result<CurriculumRemediationMapDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_remediation_map(node_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_coverage_stats(
    state: &AppState,
    subject_track_id: i64,
) -> Result<CurriculumCoverageStatsDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_coverage_stats(subject_track_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_registry(
    state: &AppState,
) -> Result<Vec<CurriculumRegistryEntryDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_registry()
            .map_err(Into::into)
    })
}

pub fn get_curriculum_ingestion_workspace(
    state: &AppState,
    source_upload_id: i64,
) -> Result<Option<CurriculumIngestionWorkspaceDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_ingestion_workspace(source_upload_id)
            .map_err(Into::into)
    })
}

pub fn get_curriculum_admin_node_detail(
    state: &AppState,
    node_id: i64,
) -> Result<CurriculumAdminNodeDetailDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_curriculum_admin_node_detail(node_id)
            .map_err(Into::into)
    })
}

pub fn analyze_curriculum_version_impact(
    state: &AppState,
    base_version_id: i64,
    compare_version_id: i64,
) -> Result<CurriculumImpactAnalysisDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .analyze_curriculum_version_impact(base_version_id, compare_version_id)
            .map_err(Into::into)
    })
}

pub fn stage_curriculum_regeneration_jobs(
    state: &AppState,
    base_version_id: i64,
    compare_version_id: i64,
    triggered_by_account_id: Option<i64>,
    max_jobs: i64,
) -> Result<Vec<CurriculumRegenerationJobDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .stage_curriculum_regeneration_jobs(
                base_version_id,
                compare_version_id,
                triggered_by_account_id,
                max_jobs,
            )
            .map_err(Into::into)
    })
}

pub fn list_curriculum_regeneration_jobs(
    state: &AppState,
    compare_version_id: i64,
    status: Option<String>,
    limit: i64,
) -> Result<Vec<CurriculumRegenerationJobDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_curriculum_regeneration_jobs(compare_version_id, status.as_deref(), limit)
            .map_err(Into::into)
    })
}

pub fn pin_curriculum_version_to_cohort(
    state: &AppState,
    input: CurriculumCohortPinInput,
) -> Result<CurriculumCohortPinDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .pin_curriculum_version_to_cohort(input)
            .map_err(Into::into)
    })
}

pub fn list_curriculum_version_cohort_pins(
    state: &AppState,
    curriculum_version_id: i64,
) -> Result<Vec<CurriculumCohortPinDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .list_curriculum_version_cohort_pins(curriculum_version_id)
            .map_err(Into::into)
    })
}

pub fn assign_student_curriculum_version(
    state: &AppState,
    input: StudentCurriculumAssignmentInput,
) -> Result<StudentCurriculumAssignmentDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .assign_student_curriculum_version(input)
            .map_err(Into::into)
    })
}

pub fn get_active_student_curriculum_assignment(
    state: &AppState,
    student_id: i64,
) -> Result<Option<StudentCurriculumAssignmentDto>, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_active_student_curriculum_assignment(student_id)
            .map_err(Into::into)
    })
}

pub fn get_student_curriculum_home(
    state: &AppState,
    student_id: i64,
    curriculum_version_id: Option<i64>,
) -> Result<CurriculumStudentHomeSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_student_curriculum_home(student_id, curriculum_version_id)
            .map_err(Into::into)
    })
}

pub fn get_student_subject_curriculum_map(
    state: &AppState,
    student_id: i64,
    subject_track_id: i64,
) -> Result<CurriculumStudentSubjectMapDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_student_subject_curriculum_map(student_id, subject_track_id)
            .map_err(Into::into)
    })
}

pub fn get_parent_curriculum_summary(
    state: &AppState,
    parent_id: i64,
    learner_id: i64,
    curriculum_version_id: Option<i64>,
) -> Result<CurriculumParentSummaryDto, CommandError> {
    state.with_connection(|conn| {
        CurriculumService::new(conn)
            .get_parent_curriculum_summary(parent_id, learner_id, curriculum_version_id)
            .map_err(Into::into)
    })
}
