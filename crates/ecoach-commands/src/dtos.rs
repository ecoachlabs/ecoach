use chrono::{DateTime, Utc};
use ecoach_content::{
    ContentFoundrySourceReport, CurriculumParseCandidate, CurriculumReviewTask,
    CurriculumSourceUpload, FoundryJob, FoundryJobBoard, PackInstallResult, PackSummary,
    ParseCandidateCount, SubjectFoundryDashboard, TopicPackageSnapshot,
};
use ecoach_diagnostics::{
    DiagnosticCauseEvolution, DiagnosticLongitudinalSummary, DiagnosticResult,
    TopicDiagnosticLongitudinalSignal, TopicDiagnosticResult,
};
use ecoach_elite::{
    EliteBlueprintFamilyTarget, EliteBlueprintReport, EliteBlueprintTopicTarget, EliteProfile,
    EliteSessionBlueprint, EliteTopicProfile, EliteTrapBlueprintSignal,
};
use ecoach_games::{
    ContrastPairSummary, TrapChoiceOption, TrapReviewRound, TrapRoundCard, TrapRoundResult,
    TrapSessionReview, TrapSessionSnapshot,
};
use ecoach_glossary::KnowledgeBundleSequenceItem;
use ecoach_goals_calendar::{DailyReplan, FreeNowRecommendation};
use ecoach_identity::{Account, AccountSummary};
use ecoach_intake::{BundleFile, BundleProcessReport, ExtractedInsight, SubmissionBundle};
use ecoach_library::{LearningPathStep, PersonalizedLearningPath, TopicRelationshipHint};
use ecoach_past_papers::PastPaperComebackSignal;
use ecoach_questions::{
    DuplicateCheckResult, GeneratedQuestionDraft, QuestionFamilyChoice, QuestionFamilyHealth,
    QuestionGenerationRequest, QuestionLineageEdge, QuestionLineageGraph, QuestionLineageNode,
    QuestionRemediationPlan, RelatedQuestion,
};
use ecoach_sessions::{
    MockBlueprint, SessionEvidenceFabric, SessionInterpretation, SessionSnapshot, SessionSummary,
    SessionTopicInterpretation,
};
use ecoach_substrate::{
    FabricConsumerTarget, FabricEvidenceRecord, FabricOrchestrationSummary, FabricSignal,
};
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
pub struct SubmissionBundleDto {
    pub id: i64,
    pub student_id: i64,
    pub title: String,
    pub status: String,
}

impl From<SubmissionBundle> for SubmissionBundleDto {
    fn from(value: SubmissionBundle) -> Self {
        Self {
            id: value.id,
            student_id: value.student_id,
            title: value.title,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFileDto {
    pub id: i64,
    pub bundle_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub file_kind: String,
}

impl From<BundleFile> for BundleFileDto {
    fn from(value: BundleFile) -> Self {
        Self {
            id: value.id,
            bundle_id: value.bundle_id,
            file_name: value.file_name,
            file_path: value.file_path,
            mime_type: value.mime_type,
            file_kind: value.file_kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedInsightDto {
    pub id: i64,
    pub bundle_id: i64,
    pub insight_type: String,
    pub payload: Value,
    pub created_at: String,
}

impl From<ExtractedInsight> for ExtractedInsightDto {
    fn from(value: ExtractedInsight) -> Self {
        Self {
            id: value.id,
            bundle_id: value.bundle_id,
            insight_type: value.insight_type,
            payload: value.payload,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleProcessReportDto {
    pub bundle: SubmissionBundleDto,
    pub files: Vec<BundleFileDto>,
    pub insights: Vec<ExtractedInsightDto>,
    pub detected_subjects: Vec<String>,
    pub detected_exam_years: Vec<i64>,
    pub detected_topics: Vec<String>,
    pub detected_dates: Vec<String>,
    pub question_like_file_count: i64,
    pub answer_like_file_count: i64,
    pub ocr_candidate_file_count: i64,
    pub ocr_recovered_file_count: i64,
    pub layout_recovered_file_count: i64,
    pub estimated_question_count: i64,
    pub estimated_answer_count: i64,
    pub reconstructed_document_count: i64,
    pub paired_assessment_document_count: i64,
    pub reconstruction_confidence_score: i64,
    pub extracted_question_block_count: i64,
    pub aligned_question_pair_count: i64,
    pub high_confidence_alignment_count: i64,
    pub medium_confidence_alignment_count: i64,
    pub low_confidence_alignment_count: i64,
    pub score_signal_count: i64,
    pub remark_signal_count: i64,
    pub needs_confirmation: bool,
    pub unresolved_alignment_count: i64,
    pub review_priority: String,
    pub bundle_kind: String,
    pub detected_document_roles: Vec<String>,
    pub weakness_signals: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub review_reasons: Vec<String>,
}

impl From<BundleProcessReport> for BundleProcessReportDto {
    fn from(value: BundleProcessReport) -> Self {
        Self {
            bundle: SubmissionBundleDto::from(value.bundle),
            files: value.files.into_iter().map(BundleFileDto::from).collect(),
            insights: value
                .insights
                .into_iter()
                .map(ExtractedInsightDto::from)
                .collect(),
            detected_subjects: value.detected_subjects,
            detected_exam_years: value.detected_exam_years,
            detected_topics: value.detected_topics,
            detected_dates: value.detected_dates,
            question_like_file_count: value.question_like_file_count,
            answer_like_file_count: value.answer_like_file_count,
            ocr_candidate_file_count: value.ocr_candidate_file_count,
            ocr_recovered_file_count: value.ocr_recovered_file_count,
            layout_recovered_file_count: value.layout_recovered_file_count,
            estimated_question_count: value.estimated_question_count,
            estimated_answer_count: value.estimated_answer_count,
            reconstructed_document_count: value.reconstructed_document_count,
            paired_assessment_document_count: value.paired_assessment_document_count,
            reconstruction_confidence_score: value.reconstruction_confidence_score,
            extracted_question_block_count: value.extracted_question_block_count,
            aligned_question_pair_count: value.aligned_question_pair_count,
            high_confidence_alignment_count: value.high_confidence_alignment_count,
            medium_confidence_alignment_count: value.medium_confidence_alignment_count,
            low_confidence_alignment_count: value.low_confidence_alignment_count,
            score_signal_count: value.score_signal_count,
            remark_signal_count: value.remark_signal_count,
            needs_confirmation: value.needs_confirmation,
            unresolved_alignment_count: value.unresolved_alignment_count,
            review_priority: value.review_priority,
            bundle_kind: value.bundle_kind,
            detected_document_roles: value.detected_document_roles,
            weakness_signals: value.weakness_signals,
            recommended_actions: value.recommended_actions,
            review_reasons: value.review_reasons,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRelationshipHintDto {
    pub relation_type: String,
    pub from_title: String,
    pub to_title: String,
    pub explanation: String,
    pub hop_count: i64,
    pub strength_score: i64,
    pub focus_topic_id: Option<i64>,
}

impl From<TopicRelationshipHint> for LibraryRelationshipHintDto {
    fn from(value: TopicRelationshipHint) -> Self {
        Self {
            relation_type: value.relation_type,
            from_title: value.from_title,
            to_title: value.to_title,
            explanation: value.explanation,
            hop_count: value.hop_count,
            strength_score: value.strength_score as i64,
            focus_topic_id: value.focus_topic_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPathStepDto {
    pub sequence_no: i64,
    pub step_type: String,
    pub title: String,
    pub detail: String,
    pub topic_id: Option<i64>,
    pub bundle_id: Option<i64>,
    pub question_id: Option<i64>,
}

impl From<LearningPathStep> for LearningPathStepDto {
    fn from(value: LearningPathStep) -> Self {
        Self {
            sequence_no: value.sequence_no,
            step_type: value.step_type,
            title: value.title,
            detail: value.detail,
            topic_id: value.topic_id,
            bundle_id: value.bundle_id,
            question_id: value.question_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedLearningPathDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub activity_type: String,
    pub priority_score: i64,
    pub reason: String,
    pub mastery_score: i64,
    pub gap_score: i64,
    pub recommended_bundle_ids: Vec<i64>,
    pub recommended_bundle_titles: Vec<String>,
    pub related_topic_names: Vec<String>,
    pub relationship_hints: Vec<LibraryRelationshipHintDto>,
    pub steps: Vec<LearningPathStepDto>,
}

impl From<PersonalizedLearningPath> for PersonalizedLearningPathDto {
    fn from(value: PersonalizedLearningPath) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            activity_type: value.activity_type,
            priority_score: value.priority_score as i64,
            reason: value.reason,
            mastery_score: value.mastery_score,
            gap_score: value.gap_score,
            recommended_bundle_ids: value.recommended_bundle_ids,
            recommended_bundle_titles: value.recommended_bundle_titles,
            related_topic_names: value.related_topic_names,
            relationship_hints: value
                .relationship_hints
                .into_iter()
                .map(LibraryRelationshipHintDto::from)
                .collect(),
            steps: value
                .steps
                .into_iter()
                .map(LearningPathStepDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeNowRecommendationDto {
    pub date: String,
    pub minute_of_day: i64,
    pub available_now: bool,
    pub window_end_minute: Option<i64>,
    pub suggested_duration_minutes: i64,
    pub session_type: String,
    pub rationale: String,
    pub focus_topic_ids: Vec<i64>,
    pub target_id: Option<i64>,
    pub carryover_attempts: i64,
    pub carryover_correct: i64,
    pub pressure_score: i64,
    pub repair_buffer_minutes: i64,
    pub recommended_comeback_topic_id: Option<i64>,
    pub recent_repair_outcome: Option<String>,
}

impl From<FreeNowRecommendation> for FreeNowRecommendationDto {
    fn from(value: FreeNowRecommendation) -> Self {
        Self {
            date: value.date,
            minute_of_day: value.minute_of_day,
            available_now: value.available_now,
            window_end_minute: value.window_end_minute,
            suggested_duration_minutes: value.suggested_duration_minutes,
            session_type: value.session_type,
            rationale: value.rationale,
            focus_topic_ids: value.focus_topic_ids,
            target_id: value.target_id,
            carryover_attempts: value.carryover_attempts,
            carryover_correct: value.carryover_correct,
            pressure_score: value.pressure_score as i64,
            repair_buffer_minutes: value.repair_buffer_minutes,
            recommended_comeback_topic_id: value.recommended_comeback_topic_id,
            recent_repair_outcome: value.recent_repair_outcome,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReplanDto {
    pub date: String,
    pub available_now: bool,
    pub remaining_capacity_minutes: i64,
    pub remaining_target_minutes: i64,
    pub recommended_session_count: i64,
    pub next_session_type: String,
    pub focus_topic_ids: Vec<i64>,
    pub target_id: Option<i64>,
    pub rationale: String,
    pub pressure_score: i64,
    pub repair_buffer_minutes: i64,
    pub recommended_comeback_topic_id: Option<i64>,
    pub recent_repair_outcome: Option<String>,
}

impl From<DailyReplan> for DailyReplanDto {
    fn from(value: DailyReplan) -> Self {
        Self {
            date: value.date,
            available_now: value.available_now,
            remaining_capacity_minutes: value.remaining_capacity_minutes,
            remaining_target_minutes: value.remaining_target_minutes,
            recommended_session_count: value.recommended_session_count,
            next_session_type: value.next_session_type,
            focus_topic_ids: value.focus_topic_ids,
            target_id: value.target_id,
            rationale: value.rationale,
            pressure_score: value.pressure_score as i64,
            repair_buffer_minutes: value.repair_buffer_minutes,
            recommended_comeback_topic_id: value.recommended_comeback_topic_id,
            recent_repair_outcome: value.recent_repair_outcome,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBundleSequenceItemDto {
    pub bundle_id: i64,
    pub title: String,
    pub bundle_type: String,
    pub sequence_order: i64,
    pub focus_reason: String,
    pub due_review_count: i64,
    pub focus_entry_ids: Vec<i64>,
    pub focus_entry_titles: Vec<String>,
}

impl From<KnowledgeBundleSequenceItem> for KnowledgeBundleSequenceItemDto {
    fn from(value: KnowledgeBundleSequenceItem) -> Self {
        Self {
            bundle_id: value.bundle_id,
            title: value.title,
            bundle_type: value.bundle_type,
            sequence_order: value.sequence_order,
            focus_reason: value.focus_reason,
            due_review_count: value.due_review_count,
            focus_entry_ids: value.focus_entry_ids,
            focus_entry_titles: value.focus_entry_titles,
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
    pub fabric_signals: Vec<FabricSignalDto>,
    pub orchestration: FabricOrchestrationSummaryDto,
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
            fabric_signals: value
                .fabric_signals
                .into_iter()
                .map(FabricSignalDto::from)
                .collect(),
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
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
    pub fabric_signals: Vec<FabricSignalDto>,
    pub orchestration: FabricOrchestrationSummaryDto,
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
            fabric_signals: value
                .fabric_signals
                .into_iter()
                .map(FabricSignalDto::from)
                .collect(),
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
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
    pub fabric_signals: Vec<FabricSignalDto>,
    pub orchestration: FabricOrchestrationSummaryDto,
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
            fabric_signals: value
                .fabric_signals
                .into_iter()
                .map(FabricSignalDto::from)
                .collect(),
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
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
pub struct SessionTopicInterpretationDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub attempts: i64,
    pub correct_attempts: i64,
    pub accuracy_score: i64,
    pub avg_response_time_ms: Option<i64>,
    pub dominant_error_type: Option<String>,
}

impl From<SessionTopicInterpretation> for SessionTopicInterpretationDto {
    fn from(value: SessionTopicInterpretation) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            attempts: value.attempts,
            correct_attempts: value.correct_attempts,
            accuracy_score: value.accuracy_score as i64,
            avg_response_time_ms: value.avg_response_time_ms,
            dominant_error_type: value.dominant_error_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInterpretationDto {
    pub session_id: i64,
    pub student_id: i64,
    pub session_type: String,
    pub status: String,
    pub observed_at: String,
    pub is_timed: bool,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub incorrect_questions: i64,
    pub unanswered_questions: i64,
    pub accuracy_score: Option<i64>,
    pub avg_response_time_ms: Option<i64>,
    pub flagged_count: i64,
    pub distinct_topic_count: i64,
    pub misconception_hit_count: i64,
    pub pressure_breakdown_count: i64,
    pub transfer_variant_count: i64,
    pub retention_check_count: i64,
    pub mixed_context_count: i64,
    pub supported_answer_count: i64,
    pub independent_answer_count: i64,
    pub dominant_error_type: Option<String>,
    pub interpretation_tags: Vec<String>,
    pub next_action_hint: String,
    pub topic_summaries: Vec<SessionTopicInterpretationDto>,
}

impl From<SessionInterpretation> for SessionInterpretationDto {
    fn from(value: SessionInterpretation) -> Self {
        Self {
            session_id: value.session_id,
            student_id: value.student_id,
            session_type: value.session_type,
            status: value.status,
            observed_at: value.observed_at.to_rfc3339(),
            is_timed: value.is_timed,
            answered_questions: value.answered_questions,
            correct_questions: value.correct_questions,
            incorrect_questions: value.incorrect_questions,
            unanswered_questions: value.unanswered_questions,
            accuracy_score: value.accuracy_score.map(|score| score as i64),
            avg_response_time_ms: value.avg_response_time_ms,
            flagged_count: value.flagged_count,
            distinct_topic_count: value.distinct_topic_count,
            misconception_hit_count: value.misconception_hit_count,
            pressure_breakdown_count: value.pressure_breakdown_count,
            transfer_variant_count: value.transfer_variant_count,
            retention_check_count: value.retention_check_count,
            mixed_context_count: value.mixed_context_count,
            supported_answer_count: value.supported_answer_count,
            independent_answer_count: value.independent_answer_count,
            dominant_error_type: value.dominant_error_type,
            interpretation_tags: value.interpretation_tags,
            next_action_hint: value.next_action_hint,
            topic_summaries: value
                .topic_summaries
                .into_iter()
                .map(SessionTopicInterpretationDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricSignalDto {
    pub engine_key: String,
    pub signal_type: String,
    pub status: Option<String>,
    pub score: Option<i64>,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub question_id: Option<i64>,
    pub observed_at: String,
    pub payload: Value,
}

impl From<FabricSignal> for FabricSignalDto {
    fn from(value: FabricSignal) -> Self {
        Self {
            engine_key: value.engine_key,
            signal_type: value.signal_type,
            status: value.status,
            score: value.score.map(|score| score as i64),
            topic_id: value.topic_id,
            node_id: value.node_id,
            question_id: value.question_id,
            observed_at: value.observed_at,
            payload: value.payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricEvidenceRecordDto {
    pub stream: String,
    pub reference_id: String,
    pub event_type: String,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub question_id: Option<i64>,
    pub occurred_at: String,
    pub payload: Value,
}

impl From<FabricEvidenceRecord> for FabricEvidenceRecordDto {
    fn from(value: FabricEvidenceRecord) -> Self {
        Self {
            stream: value.stream,
            reference_id: value.reference_id,
            event_type: value.event_type,
            topic_id: value.topic_id,
            node_id: value.node_id,
            question_id: value.question_id,
            occurred_at: value.occurred_at,
            payload: value.payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricConsumerTargetDto {
    pub engine_key: String,
    pub engine_title: String,
    pub matched_inputs: Vec<String>,
}

impl From<FabricConsumerTarget> for FabricConsumerTargetDto {
    fn from(value: FabricConsumerTarget) -> Self {
        Self {
            engine_key: value.engine_key,
            engine_title: value.engine_title,
            matched_inputs: value.matched_inputs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricOrchestrationSummaryDto {
    pub available_inputs: Vec<String>,
    pub consumer_targets: Vec<FabricConsumerTargetDto>,
}

impl From<FabricOrchestrationSummary> for FabricOrchestrationSummaryDto {
    fn from(value: FabricOrchestrationSummary) -> Self {
        Self {
            available_inputs: value.available_inputs,
            consumer_targets: value
                .consumer_targets
                .into_iter()
                .map(FabricConsumerTargetDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCauseEvolutionDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub current_hypothesis_code: Option<String>,
    pub previous_hypothesis_code: Option<String>,
    pub evolution_status: String,
    pub recurrence_count: i64,
    pub confidence_delta: Option<i64>,
    pub summary: String,
}

impl From<DiagnosticCauseEvolution> for DiagnosticCauseEvolutionDto {
    fn from(value: DiagnosticCauseEvolution) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            current_hypothesis_code: value.current_hypothesis_code,
            previous_hypothesis_code: value.previous_hypothesis_code,
            evolution_status: value.evolution_status,
            recurrence_count: value.recurrence_count,
            confidence_delta: value.confidence_delta,
            summary: value.summary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicDiagnosticLongitudinalSignalDto {
    pub previous_diagnostic_id: Option<i64>,
    pub previous_completed_at: Option<String>,
    pub previous_classification: Option<String>,
    pub previous_mastery_score: Option<i64>,
    pub mastery_delta: Option<i64>,
    pub pressure_delta: Option<i64>,
    pub flexibility_delta: Option<i64>,
    pub trend: String,
    pub cause_evolution: Option<DiagnosticCauseEvolutionDto>,
}

impl From<TopicDiagnosticLongitudinalSignal> for TopicDiagnosticLongitudinalSignalDto {
    fn from(value: TopicDiagnosticLongitudinalSignal) -> Self {
        Self {
            previous_diagnostic_id: value.previous_diagnostic_id,
            previous_completed_at: value.previous_completed_at,
            previous_classification: value.previous_classification,
            previous_mastery_score: value.previous_mastery_score.map(|score| score as i64),
            mastery_delta: value.mastery_delta,
            pressure_delta: value.pressure_delta,
            flexibility_delta: value.flexibility_delta,
            trend: value.trend,
            cause_evolution: value.cause_evolution.map(DiagnosticCauseEvolutionDto::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicDiagnosticResultDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: i64,
    pub fluency_score: i64,
    pub precision_score: i64,
    pub pressure_score: i64,
    pub flexibility_score: i64,
    pub stability_score: i64,
    pub classification: String,
    pub longitudinal_signal: Option<TopicDiagnosticLongitudinalSignalDto>,
}

impl From<TopicDiagnosticResult> for TopicDiagnosticResultDto {
    fn from(value: TopicDiagnosticResult) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            mastery_score: value.mastery_score as i64,
            fluency_score: value.fluency_score as i64,
            precision_score: value.precision_score as i64,
            pressure_score: value.pressure_score as i64,
            flexibility_score: value.flexibility_score as i64,
            stability_score: value.stability_score as i64,
            classification: value.classification,
            longitudinal_signal: value
                .longitudinal_signal
                .map(TopicDiagnosticLongitudinalSignalDto::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticLongitudinalSummaryDto {
    pub previous_diagnostic_id: Option<i64>,
    pub previous_completed_at: Option<String>,
    pub overall_readiness_delta: Option<i64>,
    pub trend: String,
    pub improved_topic_count: usize,
    pub declined_topic_count: usize,
    pub stable_topic_count: usize,
    pub persistent_cause_count: usize,
    pub shifted_cause_count: usize,
    pub new_cause_count: usize,
    pub top_regressions: Vec<String>,
    pub cause_evolution: Vec<DiagnosticCauseEvolutionDto>,
}

impl From<DiagnosticLongitudinalSummary> for DiagnosticLongitudinalSummaryDto {
    fn from(value: DiagnosticLongitudinalSummary) -> Self {
        Self {
            previous_diagnostic_id: value.previous_diagnostic_id,
            previous_completed_at: value.previous_completed_at,
            overall_readiness_delta: value.overall_readiness_delta,
            trend: value.trend,
            improved_topic_count: value.improved_topic_count,
            declined_topic_count: value.declined_topic_count,
            stable_topic_count: value.stable_topic_count,
            persistent_cause_count: value.persistent_cause_count,
            shifted_cause_count: value.shifted_cause_count,
            new_cause_count: value.new_cause_count,
            top_regressions: value.top_regressions,
            cause_evolution: value
                .cause_evolution
                .into_iter()
                .map(DiagnosticCauseEvolutionDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticResultDto {
    pub overall_readiness: i64,
    pub readiness_band: String,
    pub topic_results: Vec<TopicDiagnosticResultDto>,
    pub recommended_next_actions: Vec<String>,
    pub longitudinal_summary: Option<DiagnosticLongitudinalSummaryDto>,
}

impl From<DiagnosticResult> for DiagnosticResultDto {
    fn from(value: DiagnosticResult) -> Self {
        Self {
            overall_readiness: value.overall_readiness as i64,
            readiness_band: value.readiness_band,
            topic_results: value
                .topic_results
                .into_iter()
                .map(TopicDiagnosticResultDto::from)
                .collect(),
            recommended_next_actions: value.recommended_next_actions,
            longitudinal_summary: value
                .longitudinal_summary
                .map(DiagnosticLongitudinalSummaryDto::from),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteProfileDto {
    pub student_id: i64,
    pub subject_id: i64,
    pub eps_score: i64,
    pub tier: String,
    pub precision_score: i64,
    pub speed_score: i64,
    pub depth_score: i64,
    pub composure_score: i64,
}

impl From<EliteProfile> for EliteProfileDto {
    fn from(value: EliteProfile) -> Self {
        Self {
            student_id: value.student_id,
            subject_id: value.subject_id,
            eps_score: value.eps_score as i64,
            tier: value.tier,
            precision_score: value.precision_score as i64,
            speed_score: value.speed_score as i64,
            depth_score: value.depth_score as i64,
            composure_score: value.composure_score as i64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteTopicProfileDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub precision_score: i64,
    pub speed_score: i64,
    pub depth_score: i64,
    pub composure_score: i64,
    pub consistency_score: i64,
    pub trap_resistance_score: i64,
    pub domination_score: i64,
    pub status: String,
}

impl From<EliteTopicProfile> for EliteTopicProfileDto {
    fn from(value: EliteTopicProfile) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            precision_score: value.precision_score as i64,
            speed_score: value.speed_score as i64,
            depth_score: value.depth_score as i64,
            composure_score: value.composure_score as i64,
            consistency_score: value.consistency_score as i64,
            trap_resistance_score: value.trap_resistance_score as i64,
            domination_score: value.domination_score as i64,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteSessionBlueprintDto {
    pub student_id: i64,
    pub subject_id: i64,
    pub session_class: String,
    pub target_topic_ids: Vec<i64>,
    pub target_family_ids: Vec<i64>,
    pub authoring_modes: Vec<String>,
    pub target_question_count: i64,
    pub rationale: String,
}

impl From<EliteSessionBlueprint> for EliteSessionBlueprintDto {
    fn from(value: EliteSessionBlueprint) -> Self {
        Self {
            student_id: value.student_id,
            subject_id: value.subject_id,
            session_class: value.session_class,
            target_topic_ids: value.target_topic_ids,
            target_family_ids: value.target_family_ids,
            authoring_modes: value.authoring_modes,
            target_question_count: value.target_question_count,
            rationale: value.rationale,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintTopicTargetDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub domination_score: i64,
    pub precision_score: i64,
    pub trap_resistance_score: i64,
    pub status: String,
    pub selection_reason: String,
}

impl From<EliteBlueprintTopicTarget> for EliteBlueprintTopicTargetDto {
    fn from(value: EliteBlueprintTopicTarget) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            domination_score: value.domination_score as i64,
            precision_score: value.precision_score as i64,
            trap_resistance_score: value.trap_resistance_score as i64,
            status: value.status,
            selection_reason: value.selection_reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintFamilyTargetDto {
    pub family_id: i64,
    pub family_code: Option<String>,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub health_status: Option<String>,
    pub recurrence_score: i64,
    pub replacement_score: i64,
    pub selection_reason: String,
}

impl From<EliteBlueprintFamilyTarget> for EliteBlueprintFamilyTargetDto {
    fn from(value: EliteBlueprintFamilyTarget) -> Self {
        Self {
            family_id: value.family_id,
            family_code: value.family_code,
            family_name: value.family_name,
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            health_status: value.health_status,
            recurrence_score: value.recurrence_score as i64,
            replacement_score: value.replacement_score as i64,
            selection_reason: value.selection_reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteTrapBlueprintSignalDto {
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub confusion_score: i64,
    pub similarity_trap_bp: i64,
    pub which_is_which_bp: i64,
    pub timed_out_count: i64,
    pub force_trapsense: bool,
    pub rationale: Option<String>,
}

impl From<EliteTrapBlueprintSignal> for EliteTrapBlueprintSignalDto {
    fn from(value: EliteTrapBlueprintSignal) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            confusion_score: value.confusion_score as i64,
            similarity_trap_bp: value.similarity_trap_bp as i64,
            which_is_which_bp: value.which_is_which_bp as i64,
            timed_out_count: value.timed_out_count,
            force_trapsense: value.force_trapsense,
            rationale: value.rationale,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EliteBlueprintReportDto {
    pub blueprint: EliteSessionBlueprintDto,
    pub profile: Option<EliteProfileDto>,
    pub topic_targets: Vec<EliteBlueprintTopicTargetDto>,
    pub family_targets: Vec<EliteBlueprintFamilyTargetDto>,
    pub trap_signal: Option<EliteTrapBlueprintSignalDto>,
}

impl From<EliteBlueprintReport> for EliteBlueprintReportDto {
    fn from(value: EliteBlueprintReport) -> Self {
        Self {
            blueprint: EliteSessionBlueprintDto::from(value.blueprint),
            profile: value.profile.map(EliteProfileDto::from),
            topic_targets: value
                .topic_targets
                .into_iter()
                .map(EliteBlueprintTopicTargetDto::from)
                .collect(),
            family_targets: value
                .family_targets
                .into_iter()
                .map(EliteBlueprintFamilyTargetDto::from)
                .collect(),
            trap_signal: value.trap_signal.map(EliteTrapBlueprintSignalDto::from),
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
    pub pair_code: Option<String>,
    pub title: String,
    pub left_label: String,
    pub right_label: String,
    pub summary_text: Option<String>,
    pub trap_strength: i64,
    pub difficulty_score: i64,
    pub confusion_score: i64,
    pub last_accuracy_bp: i64,
    pub recommended_mode: String,
    pub available_modes: Vec<String>,
}

impl From<ContrastPairSummary> for ContrastPairSummaryDto {
    fn from(value: ContrastPairSummary) -> Self {
        Self {
            pair_id: value.pair_id,
            pair_code: value.pair_code,
            title: value.title,
            left_label: value.left_label,
            right_label: value.right_label,
            summary_text: value.summary_text,
            trap_strength: value.trap_strength as i64,
            difficulty_score: value.difficulty_score as i64,
            confusion_score: value.confusion_score as i64,
            last_accuracy_bp: value.last_accuracy_bp as i64,
            recommended_mode: value.recommended_mode,
            available_modes: value.available_modes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapChoiceOptionDto {
    pub code: String,
    pub label: String,
}

impl From<TrapChoiceOption> for TrapChoiceOptionDto {
    fn from(value: TrapChoiceOption) -> Self {
        Self {
            code: value.code,
            label: value.label,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundCardDto {
    pub id: i64,
    pub round_number: i64,
    pub pair_id: i64,
    pub mode: String,
    pub lane: String,
    pub prompt_text: String,
    pub prompt_payload: Value,
    pub answer_options: Vec<TrapChoiceOptionDto>,
    pub reveal_count: i64,
    pub max_reveal_count: i64,
    pub status: String,
}

impl From<TrapRoundCard> for TrapRoundCardDto {
    fn from(value: TrapRoundCard) -> Self {
        Self {
            id: value.id,
            round_number: value.round_number,
            pair_id: value.pair_id,
            mode: value.mode,
            lane: value.lane,
            prompt_text: value.prompt_text,
            prompt_payload: value.prompt_payload,
            answer_options: value
                .answer_options
                .into_iter()
                .map(TrapChoiceOptionDto::from)
                .collect(),
            reveal_count: value.reveal_count,
            max_reveal_count: value.max_reveal_count,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSessionSnapshotDto {
    pub session_id: i64,
    pub session_state: String,
    pub score: i64,
    pub mode: String,
    pub pair_id: i64,
    pub pair_title: String,
    pub left_label: String,
    pub right_label: String,
    pub summary_text: Option<String>,
    pub recommended_mode: String,
    pub correct_discriminations: i64,
    pub total_discriminations: i64,
    pub confusion_score: i64,
    pub current_round_id: Option<i64>,
    pub round_count: usize,
    pub active_round_number: i64,
    pub rounds: Vec<TrapRoundCardDto>,
}

impl From<TrapSessionSnapshot> for TrapSessionSnapshotDto {
    fn from(value: TrapSessionSnapshot) -> Self {
        Self {
            session_id: value.session.id,
            session_state: value.session.session_state,
            score: value.session.score,
            mode: value.state.mode,
            pair_id: value.state.pair_id,
            pair_title: value.state.pair_title,
            left_label: value.left_label,
            right_label: value.right_label,
            summary_text: value.summary_text,
            recommended_mode: value.recommended_mode,
            correct_discriminations: value.state.correct_discriminations,
            total_discriminations: value.state.total_discriminations,
            confusion_score: value.state.confusion_score as i64,
            current_round_id: value.state.current_round_id,
            round_count: value.rounds.len(),
            active_round_number: value.state.current_round_number,
            rounds: value
                .rounds
                .into_iter()
                .map(TrapRoundCardDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundResultDto {
    pub round_id: i64,
    pub round_number: i64,
    pub is_correct: bool,
    pub score_earned: i64,
    pub new_score: i64,
    pub streak: i64,
    pub selected_choice_code: Option<String>,
    pub selected_choice_label: Option<String>,
    pub correct_choice_code: String,
    pub correct_choice_label: String,
    pub explanation_text: String,
    pub confusion_signal: String,
    pub next_round_id: Option<i64>,
    pub session_complete: bool,
}

impl From<TrapRoundResult> for TrapRoundResultDto {
    fn from(value: TrapRoundResult) -> Self {
        Self {
            round_id: value.round_id,
            round_number: value.round_number,
            is_correct: value.is_correct,
            score_earned: value.score_earned,
            new_score: value.new_score,
            streak: value.streak,
            selected_choice_code: value.selected_choice_code,
            selected_choice_label: value.selected_choice_label,
            correct_choice_code: value.correct_choice_code,
            correct_choice_label: value.correct_choice_label,
            explanation_text: value.explanation_text,
            confusion_signal: value.confusion_signal,
            next_round_id: value.next_round_id,
            session_complete: value.session_complete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapReviewRoundDto {
    pub round_id: i64,
    pub round_number: i64,
    pub mode: String,
    pub lane: String,
    pub prompt_text: String,
    pub selected_choice_label: Option<String>,
    pub correct_choice_label: String,
    pub is_correct: bool,
    pub timed_out: bool,
    pub response_time_ms: Option<i64>,
    pub confusion_reason_code: Option<String>,
    pub confusion_reason_text: Option<String>,
    pub explanation_text: String,
}

impl From<TrapReviewRound> for TrapReviewRoundDto {
    fn from(value: TrapReviewRound) -> Self {
        Self {
            round_id: value.round_id,
            round_number: value.round_number,
            mode: value.mode,
            lane: value.lane,
            prompt_text: value.prompt_text,
            selected_choice_label: value.selected_choice_label,
            correct_choice_label: value.correct_choice_label,
            is_correct: value.is_correct,
            timed_out: value.timed_out,
            response_time_ms: value.response_time_ms,
            confusion_reason_code: value.confusion_reason_code,
            confusion_reason_text: value.confusion_reason_text,
            explanation_text: value.explanation_text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapReviewDto {
    pub session_id: i64,
    pub pair_id: i64,
    pub pair_title: String,
    pub mode: String,
    pub score: i64,
    pub accuracy_bp: i64,
    pub confusion_score: i64,
    pub weakest_lane: Option<String>,
    pub timed_out_count: i64,
    pub recommended_next_mode: String,
    pub dominant_confusion_reason: Option<String>,
    pub remediation_actions: Vec<String>,
    pub round_count: usize,
    pub rounds: Vec<TrapReviewRoundDto>,
}

impl From<TrapSessionReview> for TrapReviewDto {
    fn from(value: TrapSessionReview) -> Self {
        Self {
            session_id: value.session_id,
            pair_id: value.pair_id,
            pair_title: value.pair_title,
            mode: value.mode,
            score: value.score,
            accuracy_bp: value.accuracy_bp as i64,
            confusion_score: value.confusion_score as i64,
            weakest_lane: value.weakest_lane,
            timed_out_count: value.timed_out_count,
            recommended_next_mode: value.recommended_next_mode,
            dominant_confusion_reason: value.dominant_confusion_reason,
            remediation_actions: value.remediation_actions,
            round_count: value.rounds.len(),
            rounds: value
                .rounds
                .into_iter()
                .map(TrapReviewRoundDto::from)
                .collect(),
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
pub struct QuestionRemediationPlanDto {
    pub family_choice: QuestionFamilyChoiceDto,
    pub variant_mode: String,
    pub priority_score: i64,
    pub source_question_id: Option<i64>,
    pub request_kind: String,
    pub rationale: String,
}

impl From<QuestionRemediationPlan> for QuestionRemediationPlanDto {
    fn from(value: QuestionRemediationPlan) -> Self {
        Self {
            family_choice: QuestionFamilyChoiceDto::from(value.family_choice),
            variant_mode: value.variant_mode,
            priority_score: value.priority_score as i64,
            source_question_id: value.source_question_id,
            request_kind: value.request_kind,
            rationale: value.rationale,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvidenceFabricDto {
    pub session_id: i64,
    pub student_id: i64,
    pub session_type: String,
    pub status: String,
    pub interpretation: SessionInterpretationDto,
    pub remediation_plans: Vec<QuestionRemediationPlanDto>,
    pub signals: Vec<FabricSignalDto>,
    pub evidence_records: Vec<FabricEvidenceRecordDto>,
    pub orchestration: FabricOrchestrationSummaryDto,
}

impl From<SessionEvidenceFabric> for SessionEvidenceFabricDto {
    fn from(value: SessionEvidenceFabric) -> Self {
        Self {
            session_id: value.session_id,
            student_id: value.student_id,
            session_type: value.session_type,
            status: value.status,
            interpretation: SessionInterpretationDto::from(value.interpretation),
            remediation_plans: value
                .remediation_plans
                .into_iter()
                .map(QuestionRemediationPlanDto::from)
                .collect(),
            signals: value
                .signals
                .into_iter()
                .map(FabricSignalDto::from)
                .collect(),
            evidence_records: value
                .evidence_records
                .into_iter()
                .map(FabricEvidenceRecordDto::from)
                .collect(),
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperComebackSignalDto {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub comeback_score: i64,
    pub historical_strength_score: i64,
    pub dormant_years: i64,
    pub recurrence_score: i64,
    pub replacement_score: i64,
    pub paper_count: i64,
    pub last_seen_year: Option<i64>,
    pub rationale: String,
}

impl From<PastPaperComebackSignal> for PastPaperComebackSignalDto {
    fn from(value: PastPaperComebackSignal) -> Self {
        Self {
            family_id: value.family_id,
            family_code: value.family_code,
            family_name: value.family_name,
            topic_id: value.topic_id,
            comeback_score: value.comeback_score as i64,
            historical_strength_score: value.historical_strength_score as i64,
            dormant_years: value.dormant_years,
            recurrence_score: value.recurrence_score as i64,
            replacement_score: value.replacement_score as i64,
            paper_count: value.paper_count,
            last_seen_year: value.last_seen_year,
            rationale: value.rationale,
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

#[cfg(test)]
mod tests {
    use ecoach_games::{GameSession, TrapsState};

    use super::*;
    use serde_json::json;

    fn sample_fabric_signal(signal_type: &str) -> FabricSignal {
        FabricSignal {
            engine_key: "content_foundry".to_string(),
            signal_type: signal_type.to_string(),
            status: Some("active".to_string()),
            score: Some(7_600),
            topic_id: Some(11),
            node_id: None,
            question_id: None,
            observed_at: "2026-03-30T12:00:00Z".to_string(),
            payload: json!({ "reason": signal_type }),
        }
    }

    fn sample_orchestration() -> FabricOrchestrationSummary {
        FabricOrchestrationSummary {
            available_inputs: vec!["content_foundry".to_string(), "topic_package".to_string()],
            consumer_targets: vec![FabricConsumerTarget {
                engine_key: "reporting".to_string(),
                engine_title: "Reporting".to_string(),
                matched_inputs: vec!["content_foundry".to_string()],
            }],
        }
    }

    #[test]
    fn content_foundry_source_report_dto_preserves_fabric_metadata() {
        let dto = ContentFoundrySourceReportDto::from(ContentFoundrySourceReport {
            source_upload: CurriculumSourceUpload {
                id: 5,
                uploader_account_id: 9,
                source_kind: "pdf".to_string(),
                title: "WAEC Algebra".to_string(),
                source_path: Some("packs/algebra.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("SHS".to_string()),
                subject_code: Some("MTH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: "en".to_string(),
                version_label: Some("v1".to_string()),
                source_status: "review_required".to_string(),
                confidence_score: 6_800,
                metadata: json!({ "pages": 4 }),
            },
            candidate_counts: vec![ParseCandidateCount {
                candidate_type: "topic".to_string(),
                count: 3,
            }],
            parse_candidates: vec![CurriculumParseCandidate {
                id: 12,
                source_upload_id: 5,
                candidate_type: "topic".to_string(),
                parent_candidate_id: None,
                raw_label: "Fractions".to_string(),
                normalized_label: Some("fractions".to_string()),
                payload: json!({ "span": [1, 2] }),
                confidence_score: 7_100,
                review_status: "pending".to_string(),
            }],
            review_tasks: vec![CurriculumReviewTask {
                id: 19,
                source_upload_id: 5,
                candidate_id: Some(12),
                task_type: "validate_topic".to_string(),
                status: "open".to_string(),
                severity: "medium".to_string(),
                notes: Some("Check topic merge".to_string()),
            }],
            publish_jobs: Vec::new(),
            low_confidence_candidate_count: 1,
            approved_candidate_count: 2,
            unresolved_review_count: 1,
            duplicate_label_count: 0,
            publish_readiness_score: 7_200,
            can_mark_reviewed: true,
            recommended_actions: vec!["Resolve topic validation".to_string()],
            fabric_signals: vec![sample_fabric_signal("content_review_required")],
            orchestration: sample_orchestration(),
        });

        assert_eq!(dto.fabric_signals.len(), 1);
        assert_eq!(dto.fabric_signals[0].signal_type, "content_review_required");
        assert_eq!(
            dto.orchestration.consumer_targets[0].engine_key,
            "reporting"
        );
    }

    #[test]
    fn topic_package_and_dashboard_dtos_preserve_fabric_metadata() {
        let topic = TopicPackageSnapshot {
            topic_id: 11,
            subject_id: 2,
            topic_name: "Fractions".to_string(),
            package_state: "publishable".to_string(),
            live_health_state: "healthy".to_string(),
            resource_readiness_score: 8_400,
            completeness_score: 8_600,
            quality_score: 8_300,
            evidence_score: 7_900,
            source_support_count: 4,
            contrast_pair_count: 2,
            publishable_artifact_count: 6,
            published_artifact_count: 5,
            missing_components: vec!["worked_example".to_string()],
            recommended_jobs: vec!["publish_topic_package".to_string()],
            fabric_signals: vec![sample_fabric_signal("topic_publish_ready")],
            orchestration: sample_orchestration(),
        };

        let topic_dto = TopicPackageSnapshotDto::from(topic.clone());
        assert_eq!(
            topic_dto.fabric_signals[0].signal_type,
            "topic_publish_ready"
        );
        assert_eq!(
            topic_dto.orchestration.available_inputs,
            vec!["content_foundry".to_string(), "topic_package".to_string()]
        );

        let dashboard_dto = SubjectFoundryDashboardDto::from(SubjectFoundryDashboard {
            subject_id: 2,
            subject_code: "MTH".to_string(),
            subject_name: "Mathematics".to_string(),
            source_upload_count: 3,
            pending_review_sources: 1,
            ready_publish_jobs: 2,
            published_jobs: 4,
            average_package_score: 8_100,
            weak_topic_count: 1,
            strong_topic_count: 6,
            topics: vec![topic],
            fabric_signals: vec![sample_fabric_signal("subject_publish_ready")],
            orchestration: sample_orchestration(),
        });

        assert_eq!(dashboard_dto.fabric_signals.len(), 1);
        assert_eq!(
            dashboard_dto.fabric_signals[0].signal_type,
            "subject_publish_ready"
        );
        assert_eq!(dashboard_dto.topics[0].fabric_signals.len(), 1);
    }

    #[test]
    fn elite_blueprint_report_dto_preserves_targets_and_trap_signal() {
        let dto = EliteBlueprintReportDto::from(EliteBlueprintReport {
            blueprint: EliteSessionBlueprint {
                student_id: 7,
                subject_id: 3,
                session_class: "trapsense".to_string(),
                target_topic_ids: vec![11, 12],
                target_family_ids: vec![21],
                authoring_modes: vec![
                    "misconception_probe".to_string(),
                    "representation_shift".to_string(),
                ],
                target_question_count: 10,
                rationale: "Recent trap pressure is still active.".to_string(),
            },
            profile: Some(EliteProfile {
                student_id: 7,
                subject_id: 3,
                eps_score: 8_100,
                tier: "apex".to_string(),
                precision_score: 6_200,
                speed_score: 7_500,
                depth_score: 7_800,
                composure_score: 7_300,
            }),
            topic_targets: vec![EliteBlueprintTopicTarget {
                topic_id: 11,
                topic_name: "Fractions".to_string(),
                domination_score: 4_200,
                precision_score: 5_000,
                trap_resistance_score: 4_700,
                status: "fragile".to_string(),
                selection_reason: "Trap pressure moved this topic to the front.".to_string(),
            }],
            family_targets: vec![EliteBlueprintFamilyTarget {
                family_id: 21,
                family_code: Some("ALG_TRAP".to_string()),
                family_name: "Algebra Trap".to_string(),
                topic_id: Some(11),
                topic_name: Some("Fractions".to_string()),
                health_status: Some("fragile".to_string()),
                recurrence_score: 7_400,
                replacement_score: 8_100,
                selection_reason: "Replacement pressure is high.".to_string(),
            }],
            trap_signal: Some(EliteTrapBlueprintSignal {
                topic_id: Some(11),
                topic_name: Some("Fractions".to_string()),
                confusion_score: 8_200,
                similarity_trap_bp: 4_400,
                which_is_which_bp: 4_900,
                timed_out_count: 3,
                force_trapsense: true,
                rationale: Some("Trap evidence is still fragile.".to_string()),
            }),
        });

        assert_eq!(dto.blueprint.session_class, "trapsense");
        assert_eq!(
            dto.profile.as_ref().map(|profile| profile.tier.as_str()),
            Some("apex")
        );
        assert_eq!(
            dto.topic_targets[0].selection_reason,
            "Trap pressure moved this topic to the front."
        );
        assert_eq!(
            dto.family_targets[0].family_code.as_deref(),
            Some("ALG_TRAP")
        );
        assert_eq!(
            dto.trap_signal
                .as_ref()
                .map(|signal| signal.force_trapsense),
            Some(true)
        );
    }

    #[test]
    fn trap_dtos_preserve_rounds_and_remediation_detail() {
        let snapshot = TrapSessionSnapshotDto::from(TrapSessionSnapshot {
            session: GameSession {
                id: 41,
                student_id: 7,
                game_type: "traps".to_string(),
                subject_id: 3,
                session_state: "active".to_string(),
                score: 120,
                rounds_total: 5,
                rounds_played: 2,
                streak: 1,
                best_streak: 2,
                created_at: "2026-03-30T10:00:00Z".to_string(),
                completed_at: None,
            },
            state: TrapsState {
                pair_id: 91,
                pair_title: "Distance vs Displacement".to_string(),
                mode: "unmask".to_string(),
                correct_discriminations: 1,
                total_discriminations: 2,
                confusion_score: 6_700,
                current_round_id: Some(501),
                current_round_number: 3,
            },
            left_label: "Distance".to_string(),
            right_label: "Displacement".to_string(),
            summary_text: Some("Students swap scalar and vector meaning.".to_string()),
            recommended_mode: "which_is_which".to_string(),
            rounds: vec![TrapRoundCard {
                id: 501,
                round_number: 3,
                pair_id: 91,
                mode: "unmask".to_string(),
                lane: "which_is_which".to_string(),
                prompt_text: "Which term includes direction?".to_string(),
                prompt_payload: json!({ "clue": "vector" }),
                answer_options: vec![
                    TrapChoiceOption {
                        code: "left".to_string(),
                        label: "Distance".to_string(),
                    },
                    TrapChoiceOption {
                        code: "right".to_string(),
                        label: "Displacement".to_string(),
                    },
                ],
                reveal_count: 1,
                max_reveal_count: 2,
                status: "active".to_string(),
            }],
        });

        let review = TrapReviewDto::from(TrapSessionReview {
            session_id: 41,
            pair_id: 91,
            pair_title: "Distance vs Displacement".to_string(),
            mode: "unmask".to_string(),
            score: 120,
            accuracy_bp: 6_500,
            confusion_score: 6_700,
            weakest_lane: Some("which_is_which".to_string()),
            timed_out_count: 1,
            recommended_next_mode: "difference_drill".to_string(),
            dominant_confusion_reason: Some("near_miss_language".to_string()),
            remediation_actions: vec![
                "Replay the pair in difference drill.".to_string(),
                "Review the vector cue.".to_string(),
            ],
            rounds: vec![TrapReviewRound {
                round_id: 501,
                round_number: 3,
                mode: "unmask".to_string(),
                lane: "which_is_which".to_string(),
                prompt_text: "Which term includes direction?".to_string(),
                selected_choice_label: Some("Distance".to_string()),
                correct_choice_label: "Displacement".to_string(),
                is_correct: false,
                timed_out: false,
                response_time_ms: Some(4_200),
                confusion_reason_code: Some("near_miss_language".to_string()),
                confusion_reason_text: Some("Both sounded like movement.".to_string()),
                explanation_text: "Displacement includes direction.".to_string(),
            }],
        });

        assert_eq!(snapshot.session_state, "active");
        assert_eq!(snapshot.rounds[0].answer_options.len(), 2);
        assert_eq!(snapshot.recommended_mode, "which_is_which");
        assert_eq!(review.weakest_lane.as_deref(), Some("which_is_which"));
        assert_eq!(review.remediation_actions.len(), 2);
        assert_eq!(
            review.rounds[0].confusion_reason_code.as_deref(),
            Some("near_miss_language")
        );
    }
}
