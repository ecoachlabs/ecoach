use chrono::{DateTime, Utc};
use ecoach_content::{
    ContentFoundrySourceReport, CurriculumParseCandidate, CurriculumReviewTask,
    CurriculumSourceUpload, FoundryJob, FoundryJobBoard, PackInstallResult, PackSummary,
    ParseCandidateCount, SubjectFoundryDashboard, TopicPackageSnapshot,
};
use ecoach_games::{ContrastPairSummary, TrapRoundResult, TrapSessionReview, TrapSessionSnapshot};
use ecoach_identity::{Account, AccountSummary};
use ecoach_questions::{
    DuplicateCheckResult, GeneratedQuestionDraft, QuestionFamilyChoice, QuestionFamilyHealth,
    QuestionGenerationRequest, QuestionLineageEdge, QuestionLineageGraph, QuestionLineageNode,
    RelatedQuestion,
};
use ecoach_sessions::{MockBlueprint, SessionSnapshot, SessionSummary};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub entitlement_tier: String,
    pub status: String,
    pub failed_pin_attempts: i64,
    pub is_locked: bool,
    pub needs_checkup: bool,
    pub last_active_label: String,
}

impl From<Account> for AccountDto {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            account_type: value.account_type.as_str().to_string(),
            entitlement_tier: value.entitlement_tier.as_str().to_string(),
            status: value.status,
            failed_pin_attempts: value.failed_pin_attempts,
            is_locked: value.locked_until.is_some(),
            needs_checkup: value.first_run,
            last_active_label: last_active_label(value.last_active_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummaryDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub status: String,
    pub needs_checkup: bool,
    pub last_active_label: String,
}

impl From<AccountSummary> for AccountSummaryDto {
    fn from(value: AccountSummary) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            account_type: value.account_type.as_str().to_string(),
            status: value.status,
            needs_checkup: value.first_run,
            last_active_label: last_active_label(value.last_active_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInstallResultDto {
    pub pack_id: String,
    pub pack_version: String,
    pub install_path: String,
}

impl From<PackInstallResult> for PackInstallResultDto {
    fn from(value: PackInstallResult) -> Self {
        Self {
            pack_id: value.pack_id,
            pack_version: value.pack_version,
            install_path: value.install_path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSummaryDto {
    pub pack_id: String,
    pub pack_version: String,
    pub subject_code: String,
    pub status: String,
}

impl From<PackSummary> for PackSummaryDto {
    fn from(value: PackSummary) -> Self {
        Self {
            pack_id: value.pack_id,
            pack_version: value.pack_version,
            subject_code: value.subject_code,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceUploadDto {
    pub id: i64,
    pub source_kind: String,
    pub title: String,
    pub subject_code: Option<String>,
    pub source_status: String,
    pub confidence_score: i64,
}

impl From<CurriculumSourceUpload> for CurriculumSourceUploadDto {
    fn from(value: CurriculumSourceUpload) -> Self {
        Self {
            id: value.id,
            source_kind: value.source_kind,
            title: value.title,
            subject_code: value.subject_code,
            source_status: value.source_status,
            confidence_score: value.confidence_score as i64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumParseCandidateDto {
    pub id: i64,
    pub candidate_type: String,
    pub raw_label: String,
    pub normalized_label: Option<String>,
    pub confidence_score: i64,
    pub review_status: String,
}

impl From<CurriculumParseCandidate> for CurriculumParseCandidateDto {
    fn from(value: CurriculumParseCandidate) -> Self {
        Self {
            id: value.id,
            candidate_type: value.candidate_type,
            raw_label: value.raw_label,
            normalized_label: value.normalized_label,
            confidence_score: value.confidence_score as i64,
            review_status: value.review_status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumReviewTaskDto {
    pub id: i64,
    pub candidate_id: Option<i64>,
    pub task_type: String,
    pub status: String,
    pub severity: String,
    pub notes: Option<String>,
}

impl From<CurriculumReviewTask> for CurriculumReviewTaskDto {
    fn from(value: CurriculumReviewTask) -> Self {
        Self {
            id: value.id,
            candidate_id: value.candidate_id,
            task_type: value.task_type,
            status: value.status,
            severity: value.severity,
            notes: value.notes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCandidateCountDto {
    pub candidate_type: String,
    pub count: i64,
}

impl From<ParseCandidateCount> for ParseCandidateCountDto {
    fn from(value: ParseCandidateCount) -> Self {
        Self {
            candidate_type: value.candidate_type,
            count: value.count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFoundrySourceReportDto {
    pub source_upload: CurriculumSourceUploadDto,
    pub candidate_counts: Vec<ParseCandidateCountDto>,
    pub low_confidence_candidate_count: i64,
    pub approved_candidate_count: i64,
    pub unresolved_review_count: i64,
    pub duplicate_label_count: i64,
    pub publish_readiness_score: i64,
    pub can_mark_reviewed: bool,
    pub recommended_actions: Vec<String>,
    pub parse_candidates: Vec<CurriculumParseCandidateDto>,
    pub review_tasks: Vec<CurriculumReviewTaskDto>,
}

impl From<ContentFoundrySourceReport> for ContentFoundrySourceReportDto {
    fn from(value: ContentFoundrySourceReport) -> Self {
        Self {
            source_upload: CurriculumSourceUploadDto::from(value.source_upload),
            candidate_counts: value
                .candidate_counts
                .into_iter()
                .map(ParseCandidateCountDto::from)
                .collect(),
            low_confidence_candidate_count: value.low_confidence_candidate_count,
            approved_candidate_count: value.approved_candidate_count,
            unresolved_review_count: value.unresolved_review_count,
            duplicate_label_count: value.duplicate_label_count,
            publish_readiness_score: value.publish_readiness_score as i64,
            can_mark_reviewed: value.can_mark_reviewed,
            recommended_actions: value.recommended_actions,
            parse_candidates: value
                .parse_candidates
                .into_iter()
                .map(CurriculumParseCandidateDto::from)
                .collect(),
            review_tasks: value
                .review_tasks
                .into_iter()
                .map(CurriculumReviewTaskDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackageSnapshotDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub package_state: String,
    pub live_health_state: String,
    pub completeness_score: i64,
    pub quality_score: i64,
    pub evidence_score: i64,
    pub source_support_count: i64,
    pub publishable_artifact_count: i64,
    pub published_artifact_count: i64,
    pub missing_components: Vec<String>,
    pub recommended_jobs: Vec<String>,
}

impl From<TopicPackageSnapshot> for TopicPackageSnapshotDto {
    fn from(value: TopicPackageSnapshot) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            package_state: value.package_state,
            live_health_state: value.live_health_state,
            completeness_score: value.completeness_score as i64,
            quality_score: value.quality_score as i64,
            evidence_score: value.evidence_score as i64,
            source_support_count: value.source_support_count,
            publishable_artifact_count: value.publishable_artifact_count,
            published_artifact_count: value.published_artifact_count,
            missing_components: value.missing_components,
            recommended_jobs: value.recommended_jobs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectFoundryDashboardDto {
    pub subject_id: i64,
    pub subject_code: String,
    pub subject_name: String,
    pub source_upload_count: i64,
    pub pending_review_sources: i64,
    pub ready_publish_jobs: i64,
    pub published_jobs: i64,
    pub average_package_score: i64,
    pub weak_topic_count: i64,
    pub strong_topic_count: i64,
    pub topics: Vec<TopicPackageSnapshotDto>,
}

impl From<SubjectFoundryDashboard> for SubjectFoundryDashboardDto {
    fn from(value: SubjectFoundryDashboard) -> Self {
        Self {
            subject_id: value.subject_id,
            subject_code: value.subject_code,
            subject_name: value.subject_name,
            source_upload_count: value.source_upload_count,
            pending_review_sources: value.pending_review_sources,
            ready_publish_jobs: value.ready_publish_jobs,
            published_jobs: value.published_jobs,
            average_package_score: value.average_package_score as i64,
            weak_topic_count: value.weak_topic_count,
            strong_topic_count: value.strong_topic_count,
            topics: value
                .topics
                .into_iter()
                .map(TopicPackageSnapshotDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryJobDto {
    pub id: i64,
    pub job_type: String,
    pub trigger_type: String,
    pub target_type: String,
    pub target_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub priority: i64,
    pub status: String,
    pub dependency_refs: Vec<String>,
    pub retry_count: i64,
    pub failure_reason: Option<String>,
}

impl From<FoundryJob> for FoundryJobDto {
    fn from(value: FoundryJob) -> Self {
        Self {
            id: value.id,
            job_type: value.job_type,
            trigger_type: value.trigger_type,
            target_type: value.target_type,
            target_id: value.target_id,
            subject_id: value.subject_id,
            topic_id: value.topic_id,
            priority: value.priority as i64,
            status: value.status,
            dependency_refs: value.dependency_refs,
            retry_count: value.retry_count,
            failure_reason: value.failure_reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryJobBoardDto {
    pub queued_count: i64,
    pub running_count: i64,
    pub blocked_count: i64,
    pub failed_count: i64,
    pub completed_count: i64,
    pub jobs: Vec<FoundryJobDto>,
}

impl From<FoundryJobBoard> for FoundryJobBoardDto {
    fn from(value: FoundryJobBoard) -> Self {
        Self {
            queued_count: value.queued_count,
            running_count: value.running_count,
            blocked_count: value.blocked_count,
            failed_count: value.failed_count,
            completed_count: value.completed_count,
            jobs: value.jobs.into_iter().map(FoundryJobDto::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshotDto {
    pub session_id: i64,
    pub session_type: String,
    pub status: String,
    pub active_item_index: i64,
    pub item_count: usize,
}

impl From<SessionSnapshot> for SessionSnapshotDto {
    fn from(value: SessionSnapshot) -> Self {
        Self {
            session_id: value.session.id,
            session_type: value.session.session_type,
            status: value.session.status,
            active_item_index: value.session.active_item_index,
            item_count: value.items.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryDto {
    pub session_id: i64,
    pub accuracy_score: Option<i64>,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub status: String,
}

impl From<SessionSummary> for SessionSummaryDto {
    fn from(value: SessionSummary) -> Self {
        Self {
            session_id: value.session_id,
            accuracy_score: value.accuracy_score,
            answered_questions: value.answered_questions,
            correct_questions: value.correct_questions,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockBlueprintDto {
    pub id: i64,
    pub title: String,
    pub blueprint_type: String,
    pub question_count: i64,
    pub readiness_score: i64,
    pub readiness_band: String,
    pub coverage: Value,
    pub status: String,
}

impl From<MockBlueprint> for MockBlueprintDto {
    fn from(value: MockBlueprint) -> Self {
        Self {
            id: value.id,
            title: value.title,
            blueprint_type: value.blueprint_type,
            question_count: value.question_count,
            readiness_score: value.readiness_score as i64,
            readiness_band: value.readiness_band,
            coverage: value.coverage,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastPairSummaryDto {
    pub pair_id: i64,
    pub title: String,
    pub left_label: String,
    pub right_label: String,
    pub confusion_score: i64,
    pub recommended_mode: String,
}

impl From<ContrastPairSummary> for ContrastPairSummaryDto {
    fn from(value: ContrastPairSummary) -> Self {
        Self {
            pair_id: value.pair_id,
            title: value.title,
            left_label: value.left_label,
            right_label: value.right_label,
            confusion_score: value.confusion_score as i64,
            recommended_mode: value.recommended_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSessionSnapshotDto {
    pub session_id: i64,
    pub mode: String,
    pub pair_title: String,
    pub left_label: String,
    pub right_label: String,
    pub round_count: usize,
    pub active_round_number: i64,
}

impl From<TrapSessionSnapshot> for TrapSessionSnapshotDto {
    fn from(value: TrapSessionSnapshot) -> Self {
        Self {
            session_id: value.session.id,
            mode: value.state.mode,
            pair_title: value.state.pair_title,
            left_label: value.left_label,
            right_label: value.right_label,
            round_count: value.rounds.len(),
            active_round_number: value.state.current_round_number,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundResultDto {
    pub round_id: i64,
    pub round_number: i64,
    pub is_correct: bool,
    pub score_earned: i64,
    pub correct_choice_label: String,
    pub explanation_text: String,
    pub session_complete: bool,
}

impl From<TrapRoundResult> for TrapRoundResultDto {
    fn from(value: TrapRoundResult) -> Self {
        Self {
            round_id: value.round_id,
            round_number: value.round_number,
            is_correct: value.is_correct,
            score_earned: value.score_earned,
            correct_choice_label: value.correct_choice_label,
            explanation_text: value.explanation_text,
            session_complete: value.session_complete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapReviewDto {
    pub session_id: i64,
    pub pair_title: String,
    pub mode: String,
    pub score: i64,
    pub accuracy_bp: i64,
    pub confusion_score: i64,
    pub round_count: usize,
}

impl From<TrapSessionReview> for TrapReviewDto {
    fn from(value: TrapSessionReview) -> Self {
        Self {
            session_id: value.session_id,
            pair_title: value.pair_title,
            mode: value.mode,
            score: value.score,
            accuracy_bp: value.accuracy_bp as i64,
            confusion_score: value.confusion_score as i64,
            round_count: value.rounds.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFamilyChoiceDto {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub total_instances: i64,
    pub generated_instances: i64,
    pub fit_score: i64,
}

impl From<QuestionFamilyChoice> for QuestionFamilyChoiceDto {
    fn from(value: QuestionFamilyChoice) -> Self {
        Self {
            family_id: value.family_id,
            family_code: value.family_code,
            family_name: value.family_name,
            subject_id: value.subject_id,
            topic_id: value.topic_id,
            total_instances: value.total_instances,
            generated_instances: value.generated_instances,
            fit_score: value.fit_score as i64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionGenerationRequestDto {
    pub id: i64,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub family_id: i64,
    pub source_question_id: Option<i64>,
    pub request_kind: String,
    pub variant_mode: String,
    pub requested_count: i64,
    pub status: String,
    pub rationale: Option<String>,
    pub generated_count: i64,
}

impl From<QuestionGenerationRequest> for QuestionGenerationRequestDto {
    fn from(value: QuestionGenerationRequest) -> Self {
        Self {
            id: value.id,
            subject_id: value.subject_id,
            topic_id: value.topic_id,
            family_id: value.family_id,
            source_question_id: value.source_question_id,
            request_kind: value.request_kind,
            variant_mode: value.variant_mode,
            requested_count: value.requested_count,
            status: value.status,
            rationale: value.rationale,
            generated_count: value.generated_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuestionDraftDto {
    pub request_id: i64,
    pub source_question_id: i64,
    pub question_id: i64,
    pub family_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub difficulty_level: i64,
    pub estimated_time_seconds: i64,
    pub option_count: usize,
    pub variant_mode: String,
    pub transform_summary: String,
}

impl From<GeneratedQuestionDraft> for GeneratedQuestionDraftDto {
    fn from(value: GeneratedQuestionDraft) -> Self {
        Self {
            request_id: value.request_id,
            source_question_id: value.source_question_id,
            question_id: value.question.id,
            family_id: value.question.family_id,
            stem: value.question.stem,
            question_format: value.question.question_format,
            difficulty_level: value.question.difficulty_level as i64,
            estimated_time_seconds: value.question.estimated_time_seconds,
            option_count: value.options.len(),
            variant_mode: value.variant_mode,
            transform_summary: value.transform_summary,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageNodeDto {
    pub question_id: i64,
    pub family_id: Option<i64>,
    pub lineage_key: String,
    pub node_role: String,
    pub origin_kind: String,
}

impl From<QuestionLineageNode> for QuestionLineageNodeDto {
    fn from(value: QuestionLineageNode) -> Self {
        Self {
            question_id: value.question_id,
            family_id: value.family_id,
            lineage_key: value.lineage_key,
            node_role: value.node_role,
            origin_kind: value.origin_kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageEdgeDto {
    pub from_question_id: i64,
    pub to_question_id: i64,
    pub relation_type: String,
    pub transform_mode: Option<String>,
    pub rationale: Option<String>,
}

impl From<QuestionLineageEdge> for QuestionLineageEdgeDto {
    fn from(value: QuestionLineageEdge) -> Self {
        Self {
            from_question_id: value.from_question_id,
            to_question_id: value.to_question_id,
            relation_type: value.relation_type,
            transform_mode: value.transform_mode,
            rationale: value.rationale,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionLineageGraphDto {
    pub focus_question_id: i64,
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes: Vec<QuestionLineageNodeDto>,
    pub edges: Vec<QuestionLineageEdgeDto>,
}

impl From<QuestionLineageGraph> for QuestionLineageGraphDto {
    fn from(value: QuestionLineageGraph) -> Self {
        let nodes = value
            .nodes
            .into_iter()
            .map(QuestionLineageNodeDto::from)
            .collect::<Vec<_>>();
        let edges = value
            .edges
            .into_iter()
            .map(QuestionLineageEdgeDto::from)
            .collect::<Vec<_>>();
        Self {
            focus_question_id: value.focus_question_id,
            node_count: nodes.len(),
            edge_count: edges.len(),
            nodes,
            edges,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFamilyHealthDto {
    pub family_id: i64,
    pub total_instances: i64,
    pub generated_instances: i64,
    pub active_instances: i64,
    pub recent_attempts: i64,
    pub recent_correct_attempts: i64,
    pub avg_response_time_ms: i64,
    pub misconception_hit_count: i64,
    pub freshness_score: i64,
    pub calibration_score: i64,
    pub quality_score: i64,
    pub health_status: String,
}

impl From<QuestionFamilyHealth> for QuestionFamilyHealthDto {
    fn from(value: QuestionFamilyHealth) -> Self {
        Self {
            family_id: value.family_id,
            total_instances: value.total_instances,
            generated_instances: value.generated_instances,
            active_instances: value.active_instances,
            recent_attempts: value.recent_attempts,
            recent_correct_attempts: value.recent_correct_attempts,
            avg_response_time_ms: value.avg_response_time_ms,
            misconception_hit_count: value.misconception_hit_count,
            freshness_score: value.freshness_score as i64,
            calibration_score: value.calibration_score as i64,
            quality_score: value.quality_score as i64,
            health_status: value.health_status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResultDto {
    pub matched_question_id: Option<i64>,
    pub similarity_score: i64,
    pub is_exact_duplicate: bool,
    pub is_near_duplicate: bool,
}

impl From<DuplicateCheckResult> for DuplicateCheckResultDto {
    fn from(value: DuplicateCheckResult) -> Self {
        Self {
            matched_question_id: value.matched_question_id,
            similarity_score: value.similarity_score as i64,
            is_exact_duplicate: value.is_exact_duplicate,
            is_near_duplicate: value.is_near_duplicate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedQuestionDto {
    pub question_id: i64,
    pub family_id: Option<i64>,
    pub stem: String,
    pub question_format: String,
    pub difficulty_level: i64,
    pub relation_type: String,
    pub similarity_score: i64,
    pub rationale: Option<String>,
}

impl From<RelatedQuestion> for RelatedQuestionDto {
    fn from(value: RelatedQuestion) -> Self {
        Self {
            question_id: value.question.id,
            family_id: value.question.family_id,
            stem: value.question.stem,
            question_format: value.question.question_format,
            difficulty_level: value.question.difficulty_level as i64,
            relation_type: value.edge.relation_type,
            similarity_score: value.edge.similarity_score as i64,
            rationale: value.edge.rationale,
        }
    }
}

fn last_active_label(last_active_at: Option<DateTime<Utc>>) -> String {
    let Some(last_active_at) = last_active_at else {
        return "Never active".to_string();
    };
    let delta = Utc::now() - last_active_at;
    if delta.num_hours() < 24 {
        "Active today".to_string()
    } else {
        format!("Away {} days", delta.num_days())
    }
}
