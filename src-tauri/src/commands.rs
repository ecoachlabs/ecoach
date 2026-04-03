use ecoach_coach_brain::{
    AnswerRubricInput, ConstructedAnswerEvaluationInput, ExamStrategySessionInput,
};
use ecoach_commands::assessment_commands::{
    EliteSessionBlueprintDto, PastPaperComebackSignalDto, PastPaperInverseSignalDto,
    SessionEvidenceFabricDto, SessionRemediationPlanDto,
};
use ecoach_commands::attempt_commands::{
    AttemptResultDto, SessionCompletionResultDto, SubmitAttemptInput,
};
use ecoach_commands::coach_commands::{
    AcademicCalendarEventDto, AcademicCalendarEventInputDto, AcademicCalendarSnapshotDto,
    AcademicIntentCoreSnapshotDto, AdaptationResultDto, AnswerRubricDto,
    BeatYesterdayDailySummaryDto, BeatYesterdayDailyTargetDto, BeatYesterdayDashboardDto,
    BeatYesterdayExtendedProfileDto, BeatYesterdayWeeklyReviewDto, ClimbTrendPointDto,
    CoachHubSnapshotDto, CoachIntelligenceDomeSnapshotDto, CoachJudgmentSnapshotDto,
    CoachNextActionDto, CoachOrchestrationSnapshotDto, CoachStateDto, CoachTitleHistoryEntryDto,
    ComebackFlowDto, ComposedSessionDto, CompressionActionDto, ConceptInterferenceCaseDto,
    ConsistencySnapshotDto, ConstructedAnswerEvaluationDto, ContentReadinessDto,
    DeadlinePressureDto, EngagementEventDto, EngagementEventInputDto, EngagementRiskProfileDto,
    EngineRegistryDto, EvidenceEventDto, EvidenceProbeRecommendationDto, ExamStrategyProfileDto,
    GoalFeasibilityDto, GoalRecommendationDto, JourneyRouteSnapshotDto, KnowledgeMapNodeDto,
    InterventionModeDefinitionDto, DiagnosticPrescriptionSyncDto,
    LearnerMisconceptionSnapshotDto, MasteryMapNodeDto, ParentAccessSettingsDto,
    ParentAccessSettingsInputDto, ParentAlertRecordDto, ParentFeedbackInputDto,
    ParentFeedbackRecordDto, PreparationIntensityProfileDto, ReminderScheduleDto,
    ReminderScheduleInputDto, RevengeQueueItemDto, RiseModeProfileDto, StageTransitionResultDto,
    SteppedAttemptResultDto, StrategyAdjustmentLogDto, StudentDashboardDto, StudentMomentumDto,
    SurpriseEventRecommendationDto, TeacherClimbOverviewDto, TitleDefenseBriefDto,
    TitleDefenseCompletionInputDto, TitleDefenseResultDto, TitlesHallSnapshotDto,
    TopicActionSessionDto, TopicActionSummaryDto, TopicCaseDto, TopicProofCertificationDto,
    TopicTeachingProfileDto, TopicTeachingStrategyDto, TeachingRuntimeSnapshotDto,
    UncertaintyProfileDto, VelocitySnapshotDto, InstructionalObjectEnvelopeDto,
    PersonalizationSnapshotDto,
};
use ecoach_commands::content_commands::{
    ResourceApplicabilityResolutionDto, ResourceLearningRecordDto, ResourceOrchestrationResultDto,
    TopicResourceIntelligenceSnapshotDto,
};
use ecoach_commands::curriculum_commands::{
    CurriculumAdminNodeDetailDto, CurriculumCohortPinDto, CurriculumCohortPinInputDto,
    CurriculumCoverageStatsDto, CurriculumFamilyDto, CurriculumFamilyInputDto,
    CurriculumImpactAnalysisDto, CurriculumIngestionWorkspaceDto, CurriculumLevelDto,
    CurriculumLevelInputDto, CurriculumLinkedResourceDto, CurriculumNodeBundleDto,
    CurriculumNodeBundleInputDto, CurriculumNodeCitationDto, CurriculumNodeCitationInputDto,
    CurriculumNodeCommentDto, CurriculumNodeCommentInputDto, CurriculumNodeExemplarDto,
    CurriculumNodeExemplarInputDto, CurriculumNodeIntelligenceDto,
    CurriculumNodeIntelligenceInputDto, CurriculumParentSummaryDto,
    CurriculumPrerequisiteStepDto, CurriculumPublicSubjectOverviewDto,
    CurriculumPublicTopicDetailDto, CurriculumPublishResultDto, CurriculumRecommendationDto,
    CurriculumRegenerationJobDto, CurriculumRegistryEntryDto, CurriculumRemediationMapDto,
    CurriculumReviewQueueItemDto, CurriculumSearchResultDto,
    CurriculumSourceReportDto as CurriculumPortalSourceReportDto,
    CurriculumSourceUploadDto as CurriculumPortalSourceUploadDto, CurriculumStudentHomeSnapshotDto,
    CurriculumStudentSubjectMapDto, CurriculumSubjectTrackDto, CurriculumSubjectTrackInputDto,
    CurriculumTermPeriodDto, CurriculumTermPeriodInputDto, CurriculumTopicContextDto,
    CurriculumTreeNodeDto, CurriculumVersionDiffReportDto, CurriculumVersionDto,
    CurriculumVersionInputDto, StudentCurriculumAssignmentDto,
    StudentCurriculumAssignmentInputDto, SubjectDto, TopicDto,
};
use ecoach_commands::diagnostic_commands::{
    DiagnosticBatteryDto, DiagnosticCompletionSyncDto, DiagnosticPhaseItemDto,
    DiagnosticPhasePlanDto, DiagnosticRunDto, SubmitDiagnosticAttemptInput, TopicAnalyticsDto,
};
use ecoach_commands::game_commands::{
    DuelSessionDto, GameAnswerResultDto, GameSessionDto, GameSummaryDto, LeaderboardEntryDto,
    MindstackStateDto, TugOfWarStateDto,
};
use ecoach_commands::library_commands::{
    AddLibraryNoteInputDto, AddShelfItemInputDto, BuildRevisionPackFromTemplateInputDto,
    ConceptMapViewDto, ContinueLearningCardDto, CreateCustomShelfInputDto,
    CreateGlossaryTestInputDto, CustomLibraryShelfDto, ExamHotspotDto, FormulaLabViewDto,
    GlossaryAudioProgramDto, GlossaryAudioQueueSnapshotDto, GlossaryComparisonViewDto,
    GlossaryEntryDetailDto, GlossaryEntryDto, GlossaryHomeSnapshotDto, GlossaryInteractionInputDto,
    GlossarySearchInputDto, GlossarySearchResponseDto, GlossarySearchSuggestionDto,
    GlossaryTestAttemptResultDto, GlossaryTestSessionDetailDto, KnowledgeBundleDto,
    LibraryHomeSnapshotDto, LibraryItemActionDto, LibraryItemRecordDto,
    LibraryItemStateHistoryEntryDto, LibraryNoteDto, LibrarySearchInputDto, LibrarySearchResultDto,
    LibraryShelfDto, LibraryTagDefinitionDto, OfflineLibraryItemDto, QuestionKnowledgeLinkDto,
    RecordLibraryItemActionInputDto, RevisionPackItemDto, RevisionPackSummaryDto,
    RevisionPackTemplateDto, SaveLibraryItemInputDto, StartGlossaryAudioQueueInputDto,
    SubmitGlossaryTestAttemptInputDto, TeachActionPlanDto, TeachLessonDto, TopicLibrarySnapshotDto,
    TopicRelationshipHintDto, TutorInteractionDto, TutorResponseDto,
    UpdateGlossaryAudioQueueInputDto, UpdateLibraryItemInputDto,
};
use ecoach_commands::memory_commands::{
    ActiveInterventionDto, DecayBatchResultDto, InterventionStepInputDto,
    MemoryCohortAnalyticsDto, MemoryDashboardDto, MemoryKnowledgeStateDto, MemoryReturnLoopDto,
    MemoryReviewQueueItemDto, MemoryStateDto, RecheckItemDto, ReviewScheduleItemDto,
    StudentInterferenceEdgeDto, TopicKnowledgeMapDto, TopicMemorySummaryDto,
};
use ecoach_commands::mock_commands::{
    MockAnswerResultDto, MockCentreSnapshotDto, MockDeepDiagnosisDto, MockQuestionReviewDto,
    MockReportDto, MockSessionDto, MockSessionSummaryDto,
};
use ecoach_commands::premium_commands::{
    EntitlementSnapshotDto, InterventionDto, PremiumStrategySnapshotDto, RiskDashboardDto,
    RiskFlagDto,
};
use ecoach_commands::readiness_commands::{ParentDigestDto, ReadinessReportDto};
use ecoach_commands::repair_commands::{
    GapDashboardDto, GapRepairPlanDetailDto, GapRepairPlanDto, GapScoreCardDto,
};
use ecoach_commands::reporting_commands::{
    AdminOversightSnapshotDto, HouseholdDashboardSnapshotDto, ParentDashboardSnapshotDto,
    ReportingStrategySummaryDto,
};
use ecoach_commands::session_commands::{CoachMissionSessionPlanDto, FocusModeConfigDto};
use ecoach_commands::student_commands::LearnerTruthDto;
use ecoach_commands::{
    AppState, CommandError, assessment_commands, attempt_commands, coach_commands,
    content_commands, curriculum_commands, diagnostic_commands, dtos::*, game_commands,
    identity_commands, intake_commands, library_commands, memory_commands, mock_commands,
    premium_commands, question_commands, readiness_commands, repair_commands, reporting_commands,
    session_commands, student_commands, traps_commands,
};
use ecoach_content::{
    ParseCandidateInput, RecordResourceLearningInput, ResourceOrchestrationRequest,
    SourceUploadInput,
};
use ecoach_games::{
    StartGameInput, StartTrapsSessionInput, SubmitGameAnswerInput, SubmitTrapConfusionReasonInput,
    SubmitTrapRoundInput,
};
use ecoach_identity::CreateAccountInput;
use ecoach_library::{TeachExplanationUpsertInput, TeachMicroCheckInput, TutorInteractionInput};
use ecoach_memory::RecordMemoryEvidenceInput;
use ecoach_mock_centre::{CompileMockInput, SubmitMockAnswerInput};
use ecoach_premium::CreateInterventionInput;
use ecoach_questions::{
    QuestionGenerationRequestInput, QuestionIntelligenceFilter, QuestionReviewActionInput,
    QuestionSlotSpec,
};
use ecoach_sessions::{CustomTestStartInput, MockBlueprintInput, PracticeSessionStartInput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReentryProbeResultDto {
    pub fragile_topic_ids: Vec<i64>,
    pub at_risk_topic_ids: Vec<i64>,
    pub blocking_topic_ids: Vec<i64>,
    pub needs_reactivation: bool,
    pub probe_question_count: usize,
}

// Identity

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Result<Vec<AccountSummaryDto>, CommandError> {
    identity_commands::list_accounts(&state)
}

#[tauri::command]
pub fn create_account(
    state: State<'_, AppState>,
    input: CreateAccountInput,
) -> Result<AccountDto, CommandError> {
    identity_commands::create_account(&state, input)
}

#[tauri::command]
pub fn login_with_pin(
    state: State<'_, AppState>,
    account_id: i64,
    pin: String,
) -> Result<AccountDto, CommandError> {
    identity_commands::login_with_pin(&state, account_id, pin)
}

// Coach

#[tauri::command]
pub fn get_coach_state(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<CoachStateDto, CommandError> {
    coach_commands::get_coach_state(&state, student_id)
}

#[tauri::command]
pub fn get_coach_next_action(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<CoachNextActionDto, CommandError> {
    coach_commands::get_coach_next_action(&state, student_id)
}

#[tauri::command]
pub fn get_content_readiness(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<ContentReadinessDto, CommandError> {
    coach_commands::get_content_readiness(&state, student_id)
}

#[tauri::command]
pub fn get_coach_hub_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    horizon_days: Option<usize>,
) -> Result<CoachHubSnapshotDto, CommandError> {
    coach_commands::get_coach_hub_snapshot(&state, student_id, horizon_days)
}

#[tauri::command]
pub fn get_engine_registry(state: State<'_, AppState>) -> Result<EngineRegistryDto, CommandError> {
    coach_commands::get_engine_registry(&state)
}

#[tauri::command]
pub fn get_coach_judgment_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<CoachJudgmentSnapshotDto, CommandError> {
    coach_commands::get_coach_judgment_snapshot(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn get_coach_orchestration_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<CoachOrchestrationSnapshotDto, CommandError> {
    coach_commands::get_coach_orchestration_snapshot(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn get_academic_intent_core(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<AcademicIntentCoreSnapshotDto, CommandError> {
    coach_commands::get_academic_intent_core(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn get_coach_intelligence_dome(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<CoachIntelligenceDomeSnapshotDto, CommandError> {
    coach_commands::get_coach_intelligence_dome(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn build_topic_teaching_strategy(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
) -> Result<TopicTeachingStrategyDto, CommandError> {
    coach_commands::build_topic_teaching_strategy(&state, student_id, topic_id)
}

#[tauri::command]
pub fn sync_topic_teaching_profile(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<TopicTeachingProfileDto, CommandError> {
    coach_commands::sync_topic_teaching_profile(&state, topic_id)
}

#[tauri::command]
pub fn get_topic_teaching_profile(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Option<TopicTeachingProfileDto>, CommandError> {
    coach_commands::get_topic_teaching_profile(&state, topic_id)
}

#[tauri::command]
pub fn list_instructional_objects(
    state: State<'_, AppState>,
    topic_id: i64,
    pedagogical_purpose: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<InstructionalObjectEnvelopeDto>, CommandError> {
    coach_commands::list_instructional_objects(&state, topic_id, pedagogical_purpose, limit)
}

#[tauri::command]
pub fn build_personalization_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    topic_id: Option<i64>,
) -> Result<PersonalizationSnapshotDto, CommandError> {
    coach_commands::build_personalization_snapshot(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn get_teaching_runtime_snapshot(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<TeachingRuntimeSnapshotDto, CommandError> {
    coach_commands::get_teaching_runtime_snapshot(&state, session_id)
}

#[tauri::command]
pub fn list_best_next_evidence(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<Vec<EvidenceProbeRecommendationDto>, CommandError> {
    coach_commands::list_best_next_evidence(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn evaluate_concept_interference(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<Vec<ConceptInterferenceCaseDto>, CommandError> {
    coach_commands::evaluate_concept_interference(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn list_surprise_event_recommendations(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<Vec<SurpriseEventRecommendationDto>, CommandError> {
    coach_commands::list_surprise_event_recommendations(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn list_intervention_mode_library(
    state: State<'_, AppState>,
) -> Result<Vec<InterventionModeDefinitionDto>, CommandError> {
    coach_commands::list_intervention_mode_library(&state)
}

#[tauri::command]
pub fn get_topic_intervention_prescription(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
) -> Result<DiagnosticPrescriptionSyncDto, CommandError> {
    coach_commands::get_topic_intervention_prescription(&state, student_id, topic_id)
}

#[tauri::command]
pub fn get_priority_topics(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TopicCaseDto>, CommandError> {
    coach_commands::get_priority_topics(&state, student_id, limit)
}

#[tauri::command]
pub fn get_student_dashboard(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<StudentDashboardDto, CommandError> {
    coach_commands::get_student_dashboard(&state, student_id)
}

#[tauri::command]
pub fn build_or_refresh_journey_route(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    target_exam: Option<String>,
) -> Result<JourneyRouteSnapshotDto, CommandError> {
    coach_commands::build_or_refresh_journey_route(&state, student_id, subject_id, target_exam)
}

#[tauri::command]
pub fn get_active_journey_route(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<JourneyRouteSnapshotDto>, CommandError> {
    coach_commands::get_active_journey_route(&state, student_id, subject_id)
}

#[tauri::command]
pub fn complete_journey_station(
    state: State<'_, AppState>,
    station_id: i64,
    evidence: Value,
) -> Result<JourneyRouteSnapshotDto, CommandError> {
    coach_commands::complete_journey_station(&state, station_id, evidence)
}

#[tauri::command]
pub fn generate_today_mission(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<i64, CommandError> {
    coach_commands::generate_today_mission(&state, student_id)
}

#[tauri::command]
pub fn upsert_academic_event(
    state: State<'_, AppState>,
    student_id: i64,
    event_id: Option<i64>,
    input: AcademicCalendarEventInputDto,
) -> Result<AcademicCalendarEventDto, CommandError> {
    coach_commands::upsert_academic_event(&state, student_id, event_id, input)
}

#[tauri::command]
pub fn list_academic_events(
    state: State<'_, AppState>,
    student_id: i64,
    anchor_date: Option<String>,
) -> Result<Vec<AcademicCalendarEventDto>, CommandError> {
    coach_commands::list_academic_events(&state, student_id, anchor_date)
}

#[tauri::command]
pub fn build_academic_calendar_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    anchor_date: Option<String>,
) -> Result<AcademicCalendarSnapshotDto, CommandError> {
    coach_commands::build_academic_calendar_snapshot(&state, student_id, anchor_date)
}

#[tauri::command]
pub fn get_preparation_intensity_profile(
    state: State<'_, AppState>,
    student_id: i64,
    anchor_date: Option<String>,
) -> Result<Option<PreparationIntensityProfileDto>, CommandError> {
    coach_commands::get_preparation_intensity_profile(&state, student_id, anchor_date)
}

#[tauri::command]
pub fn upsert_availability_profile(
    state: State<'_, AppState>,
    profile: ecoach_commands::coach_commands::AvailabilityProfileDto,
) -> Result<ecoach_commands::coach_commands::AvailabilityProfileDto, CommandError> {
    coach_commands::upsert_availability_profile(&state, profile)
}

#[tauri::command]
pub fn get_availability_profile(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<ecoach_commands::coach_commands::AvailabilityProfileDto>, CommandError> {
    coach_commands::get_availability_profile(&state, student_id)
}

#[tauri::command]
pub fn replace_availability_windows(
    state: State<'_, AppState>,
    student_id: i64,
    windows: Vec<ecoach_commands::coach_commands::AvailabilityWindowDto>,
) -> Result<Vec<ecoach_commands::coach_commands::AvailabilityWindowDto>, CommandError> {
    coach_commands::replace_availability_windows(&state, student_id, windows)
}

#[tauri::command]
pub fn list_availability_windows(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<ecoach_commands::coach_commands::AvailabilityWindowDto>, CommandError> {
    coach_commands::list_availability_windows(&state, student_id)
}

#[tauri::command]
pub fn add_availability_exception(
    state: State<'_, AppState>,
    student_id: i64,
    exception: ecoach_commands::coach_commands::AvailabilityExceptionDto,
) -> Result<i64, CommandError> {
    coach_commands::add_availability_exception(&state, student_id, exception)
}

#[tauri::command]
pub fn list_availability_exceptions(
    state: State<'_, AppState>,
    student_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ecoach_commands::coach_commands::AvailabilityExceptionDto>, CommandError> {
    coach_commands::list_availability_exceptions(&state, student_id, start_date, end_date)
}

#[tauri::command]
pub fn get_daily_availability(
    state: State<'_, AppState>,
    student_id: i64,
    date: String,
) -> Result<ecoach_commands::coach_commands::DailyAvailabilitySummaryDto, CommandError> {
    coach_commands::get_daily_availability(&state, student_id, &date)
}

#[tauri::command]
pub fn is_free_now(
    state: State<'_, AppState>,
    student_id: i64,
    date: String,
    minute_of_day: i64,
) -> Result<bool, CommandError> {
    coach_commands::is_free_now(&state, student_id, &date, minute_of_day)
}

#[tauri::command]
pub fn sync_exam_plan_state(
    state: State<'_, AppState>,
    input: ecoach_commands::coach_commands::ExamPlanStateInputDto,
) -> Result<ecoach_commands::coach_commands::ExamPlanStateDto, CommandError> {
    coach_commands::sync_exam_plan_state(&state, input)
}

#[tauri::command]
pub fn get_exam_plan_state(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    exam_date: String,
) -> Result<Option<ecoach_commands::coach_commands::ExamPlanStateDto>, CommandError> {
    coach_commands::get_exam_plan_state(&state, student_id, subject_id, &exam_date)
}

#[tauri::command]
pub fn get_schedule_ledger(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    date: String,
) -> Result<ecoach_commands::coach_commands::ScheduleLedgerEntryDto, CommandError> {
    coach_commands::get_schedule_ledger(&state, student_id, subject_id, &date)
}

#[tauri::command]
pub fn list_time_session_blocks(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    from_date: Option<String>,
    limit: usize,
) -> Result<Vec<ecoach_commands::coach_commands::TimeSessionBlockDto>, CommandError> {
    coach_commands::list_time_session_blocks(&state, student_id, subject_id, from_date, limit)
}

#[tauri::command]
pub fn dispatch_due_trigger_jobs(
    state: State<'_, AppState>,
    as_of: String,
    student_id: Option<i64>,
    limit: usize,
) -> Result<Vec<ecoach_commands::coach_commands::ScheduleTriggerJobDto>, CommandError> {
    coach_commands::dispatch_due_trigger_jobs(&state, &as_of, student_id, limit)
}

#[tauri::command]
pub fn build_time_orchestration_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    anchor_date: String,
    minute_of_day: i64,
    exam_date: Option<String>,
    target_effective_minutes: Option<i64>,
    plan_mode: Option<String>,
    auto_trigger_mode: Option<String>,
    horizon_days: Option<usize>,
) -> Result<ecoach_commands::coach_commands::TimeOrchestrationSnapshotDto, CommandError> {
    coach_commands::build_time_orchestration_snapshot(
        &state,
        student_id,
        subject_id,
        &anchor_date,
        minute_of_day,
        exam_date,
        target_effective_minutes,
        plan_mode,
        auto_trigger_mode,
        horizon_days,
    )
}

#[tauri::command]
pub fn recommend_free_now_session(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    date: String,
    minute_of_day: i64,
    available_minutes: i64,
) -> Result<FreeNowRecommendationDto, CommandError> {
    coach_commands::recommend_free_now_session(
        &state,
        student_id,
        subject_id,
        &date,
        minute_of_day,
        available_minutes,
    )
}

#[tauri::command]
pub fn replan_remaining_day(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    date: String,
    minute_of_day: i64,
) -> Result<DailyReplanDto, CommandError> {
    coach_commands::replan_remaining_day(&state, student_id, subject_id, &date, minute_of_day)
}

#[tauri::command]
pub fn schedule_reminder(
    state: State<'_, AppState>,
    learner_id: i64,
    input: ReminderScheduleInputDto,
) -> Result<ReminderScheduleDto, CommandError> {
    coach_commands::schedule_reminder(&state, learner_id, input)
}

#[tauri::command]
pub fn list_reminders(
    state: State<'_, AppState>,
    learner_id: i64,
    audience: Option<String>,
    status: Option<String>,
    limit: i64,
) -> Result<Vec<ReminderScheduleDto>, CommandError> {
    coach_commands::list_reminders(&state, learner_id, audience, status, limit)
}

#[tauri::command]
pub fn acknowledge_reminder(
    state: State<'_, AppState>,
    reminder_id: i64,
) -> Result<Option<ReminderScheduleDto>, CommandError> {
    coach_commands::acknowledge_reminder(&state, reminder_id)
}

#[tauri::command]
pub fn record_engagement_event(
    state: State<'_, AppState>,
    learner_id: i64,
    input: EngagementEventInputDto,
) -> Result<EngagementEventDto, CommandError> {
    coach_commands::record_engagement_event(&state, learner_id, input)
}

#[tauri::command]
pub fn get_engagement_risk_profile(
    state: State<'_, AppState>,
    learner_id: i64,
) -> Result<Option<EngagementRiskProfileDto>, CommandError> {
    coach_commands::get_engagement_risk_profile(&state, learner_id)
}

#[tauri::command]
pub fn upsert_parent_access_settings(
    state: State<'_, AppState>,
    parent_id: i64,
    learner_id: i64,
    input: ParentAccessSettingsInputDto,
) -> Result<ParentAccessSettingsDto, CommandError> {
    coach_commands::upsert_parent_access_settings(&state, parent_id, learner_id, input)
}

#[tauri::command]
pub fn get_parent_access_settings(
    state: State<'_, AppState>,
    parent_id: i64,
    learner_id: i64,
) -> Result<Option<ParentAccessSettingsDto>, CommandError> {
    coach_commands::get_parent_access_settings(&state, parent_id, learner_id)
}

#[tauri::command]
pub fn submit_parent_feedback(
    state: State<'_, AppState>,
    parent_id: i64,
    learner_id: i64,
    input: ParentFeedbackInputDto,
) -> Result<ParentFeedbackRecordDto, CommandError> {
    coach_commands::submit_parent_feedback(&state, parent_id, learner_id, input)
}

#[tauri::command]
pub fn list_parent_feedback(
    state: State<'_, AppState>,
    learner_id: i64,
    parent_id: Option<i64>,
    limit: i64,
) -> Result<Vec<ParentFeedbackRecordDto>, CommandError> {
    coach_commands::list_parent_feedback(&state, learner_id, parent_id, limit)
}

#[tauri::command]
pub fn list_parent_alerts(
    state: State<'_, AppState>,
    parent_id: i64,
    learner_id: Option<i64>,
    status: Option<String>,
    limit: i64,
) -> Result<Vec<ParentAlertRecordDto>, CommandError> {
    coach_commands::list_parent_alerts(&state, parent_id, learner_id, status, limit)
}

#[tauri::command]
pub fn acknowledge_parent_alert(
    state: State<'_, AppState>,
    alert_id: i64,
) -> Result<Option<ParentAlertRecordDto>, CommandError> {
    coach_commands::acknowledge_parent_alert(&state, alert_id)
}

#[tauri::command]
pub fn list_strategy_adjustments(
    state: State<'_, AppState>,
    learner_id: i64,
    limit: i64,
) -> Result<Vec<StrategyAdjustmentLogDto>, CommandError> {
    coach_commands::list_strategy_adjustments(&state, learner_id, limit)
}

#[tauri::command]
pub fn refresh_titles_hall(
    state: State<'_, AppState>,
    student_id: i64,
    anchor_date: Option<String>,
) -> Result<TitlesHallSnapshotDto, CommandError> {
    coach_commands::refresh_titles_hall(&state, student_id, anchor_date)
}

#[tauri::command]
pub fn get_titles_hall(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<TitlesHallSnapshotDto, CommandError> {
    coach_commands::get_titles_hall(&state, student_id)
}

#[tauri::command]
pub fn list_title_history(
    state: State<'_, AppState>,
    title_id: i64,
    limit: i64,
) -> Result<Vec<CoachTitleHistoryEntryDto>, CommandError> {
    coach_commands::list_title_history(&state, title_id, limit)
}

#[tauri::command]
pub fn begin_title_defense(
    state: State<'_, AppState>,
    student_id: i64,
    title_id: i64,
) -> Result<TitleDefenseBriefDto, CommandError> {
    coach_commands::begin_title_defense(&state, student_id, title_id)
}

#[tauri::command]
pub fn complete_title_defense(
    state: State<'_, AppState>,
    run_id: i64,
    input: TitleDefenseCompletionInputDto,
) -> Result<TitleDefenseResultDto, CommandError> {
    coach_commands::complete_title_defense(&state, run_id, input)
}

#[tauri::command]
pub fn sync_student_momentum(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<StudentMomentumDto, CommandError> {
    coach_commands::sync_student_momentum(&state, student_id)
}

#[tauri::command]
pub fn get_student_momentum(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<StudentMomentumDto>, CommandError> {
    coach_commands::get_student_momentum(&state, student_id)
}

#[tauri::command]
pub fn start_comeback_flow(
    state: State<'_, AppState>,
    student_id: i64,
    trigger_reason: String,
    days_inactive: i64,
) -> Result<ComebackFlowDto, CommandError> {
    coach_commands::start_comeback_flow(&state, student_id, &trigger_reason, days_inactive)
}

#[tauri::command]
pub fn get_active_comeback_flow(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<ComebackFlowDto>, CommandError> {
    coach_commands::get_active_comeback_flow(&state, student_id)
}

#[tauri::command]
pub fn get_revenge_queue(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<RevengeQueueItemDto>, CommandError> {
    coach_commands::get_revenge_queue(&state, student_id)
}

#[tauri::command]
pub fn get_deadline_pressure(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<DeadlinePressureDto, CommandError> {
    coach_commands::get_deadline_pressure(&state, student_id, subject_id)
}

#[tauri::command]
pub fn adapt_journey_route(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<AdaptationResultDto, CommandError> {
    coach_commands::adapt_journey_route(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_consistency_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<ConsistencySnapshotDto, CommandError> {
    coach_commands::get_consistency_snapshot(&state, student_id, subject_id)
}

#[tauri::command]
pub fn record_study_day(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    minutes: i64,
    questions: i64,
    accuracy_bp: u16,
) -> Result<(), CommandError> {
    coach_commands::record_study_day(
        &state,
        student_id,
        subject_id,
        minutes,
        questions,
        accuracy_bp,
    )
}

#[tauri::command]
pub fn get_knowledge_map(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<KnowledgeMapNodeDto>, CommandError> {
    coach_commands::get_knowledge_map(&state, student_id, subject_id)
}

#[tauri::command]
pub fn compose_session(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    station_type: String,
    route_mode: String,
    daily_budget_minutes: i64,
) -> Result<ComposedSessionDto, CommandError> {
    coach_commands::compose_session(
        &state,
        student_id,
        subject_id,
        &station_type,
        &route_mode,
        daily_budget_minutes,
    )
}

#[tauri::command]
pub fn set_exam_date(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    exam_date: String,
    daily_budget_minutes: i64,
) -> Result<(), CommandError> {
    coach_commands::set_exam_date(
        &state,
        student_id,
        subject_id,
        &exam_date,
        daily_budget_minutes,
    )
}

#[tauri::command]
pub fn interpret_attempt(
    state: State<'_, AppState>,
    attempt_id: i64,
) -> Result<EvidenceEventDto, CommandError> {
    coach_commands::interpret_attempt(&state, attempt_id)
}

#[tauri::command]
pub fn list_active_misconceptions(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<LearnerMisconceptionSnapshotDto>, CommandError> {
    coach_commands::list_active_misconceptions(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_reentry_probe(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    current_topic_id: i64,
) -> Result<ReentryProbeResultDto, CommandError> {
    let result =
        coach_commands::get_reentry_probe(&state, student_id, subject_id, current_topic_id)?;
    Ok(ReentryProbeResultDto {
        fragile_topic_ids: result.fragile_topic_ids,
        at_risk_topic_ids: result.at_risk_topic_ids,
        blocking_topic_ids: result.blocking_topic_ids,
        needs_reactivation: result.needs_reactivation,
        probe_question_count: result.probe_question_count,
    })
}

#[tauri::command]
pub fn find_reactivation_candidates(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<i64>, CommandError> {
    coach_commands::find_reactivation_candidates(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_velocity_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<VelocitySnapshotDto, CommandError> {
    coach_commands::get_velocity_snapshot(&state, student_id, subject_id)
}

#[tauri::command]
pub fn check_goal_feasibility(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<GoalFeasibilityDto, CommandError> {
    coach_commands::check_goal_feasibility(&state, student_id, subject_id)
}

#[tauri::command]
pub fn compress_route(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<CompressionActionDto>, CommandError> {
    coach_commands::compress_route(&state, student_id, subject_id)
}

#[tauri::command]
pub fn deepen_route(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<CompressionActionDto>, CommandError> {
    coach_commands::deepen_route(&state, student_id, subject_id)
}

#[tauri::command]
pub fn start_topic_action(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    topic_id: i64,
    action_mode: String,
    symptom_input: Option<String>,
) -> Result<TopicActionSessionDto, CommandError> {
    coach_commands::start_topic_action(
        &state,
        student_id,
        subject_id,
        topic_id,
        &action_mode,
        symptom_input.as_deref(),
    )
}

#[tauri::command]
pub fn complete_topic_action(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<TopicActionSummaryDto, CommandError> {
    coach_commands::complete_topic_action(&state, session_id)
}

#[tauri::command]
pub fn get_active_topic_action(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
) -> Result<Option<TopicActionSessionDto>, CommandError> {
    coach_commands::get_active_topic_action(&state, student_id, topic_id)
}

#[tauri::command]
pub fn record_topic_subtopic_completed(
    state: State<'_, AppState>,
    session_id: i64,
    step_number: i64,
) -> Result<(), CommandError> {
    coach_commands::record_topic_subtopic_completed(&state, session_id, step_number)
}

#[tauri::command]
pub fn save_learner_goal(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    goal_type: String,
    target_exam: Option<String>,
    exam_date: Option<String>,
    confidence_level: Option<String>,
) -> Result<i64, CommandError> {
    coach_commands::save_learner_goal(
        &state,
        student_id,
        subject_id,
        &goal_type,
        target_exam.as_deref(),
        exam_date.as_deref(),
        confidence_level.as_deref(),
    )
}

#[tauri::command]
pub fn get_goal_recommendation(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<GoalRecommendationDto, CommandError> {
    coach_commands::get_goal_recommendation(&state, student_id, subject_id)
}

#[tauri::command]
pub fn start_stepped_attempt(
    state: State<'_, AppState>,
    student_id: i64,
    question_id: i64,
    session_id: Option<i64>,
) -> Result<i64, CommandError> {
    coach_commands::start_stepped_attempt(&state, student_id, question_id, session_id)
}

#[tauri::command]
pub fn complete_stepped_attempt(
    state: State<'_, AppState>,
    attempt_id: i64,
) -> Result<SteppedAttemptResultDto, CommandError> {
    coach_commands::complete_stepped_attempt(&state, attempt_id)
}

#[tauri::command]
pub fn assess_topic_proof(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    topic_id: i64,
) -> Result<TopicProofCertificationDto, CommandError> {
    coach_commands::assess_topic_proof(&state, student_id, subject_id, topic_id)
}

#[tauri::command]
pub fn list_topic_proofs(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<TopicProofCertificationDto>, CommandError> {
    coach_commands::list_topic_proofs(&state, student_id, subject_id)
}

#[tauri::command]
pub fn upsert_answer_rubric(
    state: State<'_, AppState>,
    input: AnswerRubricInput,
) -> Result<AnswerRubricDto, CommandError> {
    coach_commands::upsert_answer_rubric(&state, input)
}

#[tauri::command]
pub fn get_answer_rubric(
    state: State<'_, AppState>,
    question_id: i64,
) -> Result<Option<AnswerRubricDto>, CommandError> {
    coach_commands::get_answer_rubric(&state, question_id)
}

#[tauri::command]
pub fn evaluate_constructed_answer(
    state: State<'_, AppState>,
    input: ConstructedAnswerEvaluationInput,
) -> Result<ConstructedAnswerEvaluationDto, CommandError> {
    coach_commands::evaluate_constructed_answer(&state, input)
}

#[tauri::command]
pub fn record_exam_strategy_session(
    state: State<'_, AppState>,
    input: ExamStrategySessionInput,
) -> Result<ExamStrategyProfileDto, CommandError> {
    coach_commands::record_exam_strategy_session(&state, input)
}

#[tauri::command]
pub fn get_exam_strategy_profile(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<ExamStrategyProfileDto>, CommandError> {
    coach_commands::get_exam_strategy_profile(&state, student_id, subject_id)
}

#[tauri::command]
pub fn refresh_mastery_map(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<MasteryMapNodeDto>, CommandError> {
    coach_commands::refresh_mastery_map(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_mastery_map(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<MasteryMapNodeDto>, CommandError> {
    coach_commands::get_mastery_map(&state, student_id, subject_id)
}

#[tauri::command]
pub fn enter_rise_mode(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<RiseModeProfileDto, CommandError> {
    coach_commands::enter_rise_mode(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_rise_mode_profile(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<RiseModeProfileDto>, CommandError> {
    coach_commands::get_rise_mode_profile(&state, student_id, subject_id)
}

#[tauri::command]
pub fn check_rise_mode_transition(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<StageTransitionResultDto>, CommandError> {
    coach_commands::check_rise_mode_transition(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_beat_yesterday_dashboard(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    target_date: String,
) -> Result<BeatYesterdayDashboardDto, CommandError> {
    coach_commands::get_beat_yesterday_dashboard(&state, student_id, subject_id, &target_date)
}

#[tauri::command]
pub fn generate_daily_climb_target(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    date: String,
) -> Result<BeatYesterdayDailyTargetDto, CommandError> {
    coach_commands::generate_daily_climb_target(&state, student_id, subject_id, &date)
}

#[tauri::command]
pub fn complete_daily_climb(
    state: State<'_, AppState>,
    target_id: i64,
) -> Result<BeatYesterdayDailySummaryDto, CommandError> {
    coach_commands::complete_daily_climb(&state, target_id)
}

#[tauri::command]
pub fn list_climb_trend(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    days: i64,
) -> Result<Vec<ClimbTrendPointDto>, CommandError> {
    coach_commands::list_climb_trend(&state, student_id, subject_id, days)
}

#[tauri::command]
pub fn get_beat_yesterday_extended_profile(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<BeatYesterdayExtendedProfileDto, CommandError> {
    coach_commands::get_beat_yesterday_extended_profile(&state, student_id, subject_id)
}

#[tauri::command]
pub fn update_beat_yesterday_scores(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<(), CommandError> {
    coach_commands::update_beat_yesterday_scores(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_beat_yesterday_weekly_review(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<BeatYesterdayWeeklyReviewDto, CommandError> {
    coach_commands::get_beat_yesterday_weekly_review(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_teacher_climb_overview(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Vec<TeacherClimbOverviewDto>, CommandError> {
    coach_commands::get_teacher_climb_overview(&state, subject_id)
}

// Curriculum

#[tauri::command]
pub fn list_subjects(
    state: State<'_, AppState>,
    curriculum_version_id: i64,
) -> Result<Vec<SubjectDto>, CommandError> {
    curriculum_commands::list_subjects(&state, curriculum_version_id)
}

#[tauri::command]
pub fn list_topics(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Vec<TopicDto>, CommandError> {
    curriculum_commands::list_topics(&state, subject_id)
}

#[tauri::command]
pub fn list_curriculum_families(
    state: State<'_, AppState>,
) -> Result<Vec<CurriculumFamilyDto>, CommandError> {
    curriculum_commands::list_curriculum_families(&state)
}

#[tauri::command]
pub fn save_curriculum_family(
    state: State<'_, AppState>,
    input: CurriculumFamilyInputDto,
) -> Result<CurriculumFamilyDto, CommandError> {
    curriculum_commands::save_curriculum_family(&state, input)
}

#[tauri::command]
pub fn save_curriculum_version(
    state: State<'_, AppState>,
    input: CurriculumVersionInputDto,
) -> Result<CurriculumVersionDto, CommandError> {
    curriculum_commands::save_curriculum_version(&state, input)
}

#[tauri::command]
pub fn save_curriculum_subject_track(
    state: State<'_, AppState>,
    input: CurriculumSubjectTrackInputDto,
) -> Result<CurriculumSubjectTrackDto, CommandError> {
    curriculum_commands::save_curriculum_subject_track(&state, input)
}

#[tauri::command]
pub fn save_curriculum_level(
    state: State<'_, AppState>,
    input: CurriculumLevelInputDto,
) -> Result<CurriculumLevelDto, CommandError> {
    curriculum_commands::save_curriculum_level(&state, input)
}

#[tauri::command]
pub fn save_curriculum_term_period(
    state: State<'_, AppState>,
    input: CurriculumTermPeriodInputDto,
) -> Result<CurriculumTermPeriodDto, CommandError> {
    curriculum_commands::save_curriculum_term_period(&state, input)
}

#[tauri::command]
pub fn save_curriculum_node_bundle(
    state: State<'_, AppState>,
    input: CurriculumNodeBundleInputDto,
) -> Result<CurriculumNodeBundleDto, CommandError> {
    curriculum_commands::save_curriculum_node_bundle(&state, input)
}

#[tauri::command]
pub fn approve_curriculum_node(
    state: State<'_, AppState>,
    node_id: i64,
    reviewer_note: Option<String>,
) -> Result<CurriculumNodeBundleDto, CommandError> {
    curriculum_commands::approve_curriculum_node(&state, node_id, reviewer_note)
}

#[tauri::command]
pub fn save_curriculum_node_citation(
    state: State<'_, AppState>,
    input: CurriculumNodeCitationInputDto,
) -> Result<CurriculumNodeCitationDto, CommandError> {
    curriculum_commands::save_curriculum_node_citation(&state, input)
}

#[tauri::command]
pub fn save_curriculum_node_exemplar(
    state: State<'_, AppState>,
    input: CurriculumNodeExemplarInputDto,
) -> Result<CurriculumNodeExemplarDto, CommandError> {
    curriculum_commands::save_curriculum_node_exemplar(&state, input)
}

#[tauri::command]
pub fn save_curriculum_node_comment(
    state: State<'_, AppState>,
    input: CurriculumNodeCommentInputDto,
) -> Result<CurriculumNodeCommentDto, CommandError> {
    curriculum_commands::save_curriculum_node_comment(&state, input)
}

#[tauri::command]
pub fn upsert_curriculum_node_intelligence(
    state: State<'_, AppState>,
    input: CurriculumNodeIntelligenceInputDto,
) -> Result<CurriculumNodeIntelligenceDto, CommandError> {
    curriculum_commands::upsert_curriculum_node_intelligence(&state, input)
}

#[tauri::command]
pub fn list_curriculum_source_uploads(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<CurriculumPortalSourceUploadDto>, CommandError> {
    curriculum_commands::list_curriculum_source_uploads(&state, status)
}

#[tauri::command]
pub fn get_curriculum_source_report(
    state: State<'_, AppState>,
    source_upload_id: i64,
) -> Result<Option<CurriculumPortalSourceReportDto>, CommandError> {
    curriculum_commands::get_curriculum_source_report(&state, source_upload_id)
}

#[tauri::command]
pub fn list_curriculum_review_queue(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<CurriculumReviewQueueItemDto>, CommandError> {
    curriculum_commands::list_curriculum_review_queue(&state, limit)
}

#[tauri::command]
pub fn publish_curriculum_version(
    state: State<'_, AppState>,
    curriculum_version_id: i64,
    generated_by_account_id: Option<i64>,
    notes: Option<String>,
) -> Result<CurriculumPublishResultDto, CommandError> {
    curriculum_commands::publish_curriculum_version(
        &state,
        curriculum_version_id,
        generated_by_account_id,
        notes,
    )
}

#[tauri::command]
pub fn get_curriculum_version_diff(
    state: State<'_, AppState>,
    base_version_id: i64,
    compare_version_id: i64,
) -> Result<CurriculumVersionDiffReportDto, CommandError> {
    curriculum_commands::get_curriculum_version_diff(&state, base_version_id, compare_version_id)
}

#[tauri::command]
pub fn list_public_curriculum_subjects(
    state: State<'_, AppState>,
    family_slug: String,
    version_label: String,
) -> Result<Vec<CurriculumSubjectTrackDto>, CommandError> {
    curriculum_commands::list_public_curriculum_subjects(&state, family_slug, version_label)
}

#[tauri::command]
pub fn get_public_curriculum_subject_overview(
    state: State<'_, AppState>,
    family_slug: String,
    version_label: String,
    subject_slug: String,
) -> Result<CurriculumPublicSubjectOverviewDto, CommandError> {
    curriculum_commands::get_public_curriculum_subject_overview(
        &state,
        family_slug,
        version_label,
        subject_slug,
    )
}

#[tauri::command]
pub fn get_public_curriculum_subject_tree(
    state: State<'_, AppState>,
    family_slug: String,
    version_label: String,
    subject_slug: String,
) -> Result<Vec<CurriculumTreeNodeDto>, CommandError> {
    curriculum_commands::get_public_curriculum_subject_tree(
        &state,
        family_slug,
        version_label,
        subject_slug,
    )
}

#[tauri::command]
pub fn get_public_curriculum_topic_detail_by_slug(
    state: State<'_, AppState>,
    slug: String,
) -> Result<Option<CurriculumPublicTopicDetailDto>, CommandError> {
    curriculum_commands::get_public_curriculum_topic_detail_by_slug(&state, slug)
}

#[tauri::command]
pub fn search_curriculum(
    state: State<'_, AppState>,
    query: String,
    published_only: bool,
    limit: i64,
) -> Result<Vec<CurriculumSearchResultDto>, CommandError> {
    curriculum_commands::search_curriculum(&state, query, published_only, limit)
}

#[tauri::command]
pub fn get_curriculum_topic_resources(
    state: State<'_, AppState>,
    node_id: i64,
) -> Result<Vec<CurriculumLinkedResourceDto>, CommandError> {
    curriculum_commands::get_curriculum_topic_resources(&state, node_id)
}

#[tauri::command]
pub fn get_curriculum_topic_context(
    state: State<'_, AppState>,
    node_id: i64,
) -> Result<CurriculumTopicContextDto, CommandError> {
    curriculum_commands::get_curriculum_topic_context(&state, node_id)
}

#[tauri::command]
pub fn get_curriculum_next_best_topics(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    limit: i64,
) -> Result<Vec<CurriculumRecommendationDto>, CommandError> {
    curriculum_commands::get_curriculum_next_best_topics(&state, student_id, subject_id, limit)
}

#[tauri::command]
pub fn get_curriculum_prerequisite_chain(
    state: State<'_, AppState>,
    node_id: i64,
) -> Result<Vec<CurriculumPrerequisiteStepDto>, CommandError> {
    curriculum_commands::get_curriculum_prerequisite_chain(&state, node_id)
}

#[tauri::command]
pub fn get_curriculum_remediation_map(
    state: State<'_, AppState>,
    node_id: i64,
) -> Result<CurriculumRemediationMapDto, CommandError> {
    curriculum_commands::get_curriculum_remediation_map(&state, node_id)
}

#[tauri::command]
pub fn get_curriculum_coverage_stats(
    state: State<'_, AppState>,
    subject_track_id: i64,
) -> Result<CurriculumCoverageStatsDto, CommandError> {
    curriculum_commands::get_curriculum_coverage_stats(&state, subject_track_id)
}

#[tauri::command]
pub fn get_curriculum_registry(
    state: State<'_, AppState>,
) -> Result<Vec<CurriculumRegistryEntryDto>, CommandError> {
    curriculum_commands::get_curriculum_registry(&state)
}

#[tauri::command]
pub fn get_curriculum_ingestion_workspace(
    state: State<'_, AppState>,
    source_upload_id: i64,
) -> Result<Option<CurriculumIngestionWorkspaceDto>, CommandError> {
    curriculum_commands::get_curriculum_ingestion_workspace(&state, source_upload_id)
}

#[tauri::command]
pub fn get_curriculum_admin_node_detail(
    state: State<'_, AppState>,
    node_id: i64,
) -> Result<CurriculumAdminNodeDetailDto, CommandError> {
    curriculum_commands::get_curriculum_admin_node_detail(&state, node_id)
}

#[tauri::command]
pub fn analyze_curriculum_version_impact(
    state: State<'_, AppState>,
    base_version_id: i64,
    compare_version_id: i64,
) -> Result<CurriculumImpactAnalysisDto, CommandError> {
    curriculum_commands::analyze_curriculum_version_impact(
        &state,
        base_version_id,
        compare_version_id,
    )
}

#[tauri::command]
pub fn stage_curriculum_regeneration_jobs(
    state: State<'_, AppState>,
    base_version_id: i64,
    compare_version_id: i64,
    triggered_by_account_id: Option<i64>,
    max_jobs: i64,
) -> Result<Vec<CurriculumRegenerationJobDto>, CommandError> {
    curriculum_commands::stage_curriculum_regeneration_jobs(
        &state,
        base_version_id,
        compare_version_id,
        triggered_by_account_id,
        max_jobs,
    )
}

#[tauri::command]
pub fn list_curriculum_regeneration_jobs(
    state: State<'_, AppState>,
    compare_version_id: i64,
    status: Option<String>,
    limit: i64,
) -> Result<Vec<CurriculumRegenerationJobDto>, CommandError> {
    curriculum_commands::list_curriculum_regeneration_jobs(
        &state,
        compare_version_id,
        status,
        limit,
    )
}

#[tauri::command]
pub fn pin_curriculum_version_to_cohort(
    state: State<'_, AppState>,
    input: CurriculumCohortPinInputDto,
) -> Result<CurriculumCohortPinDto, CommandError> {
    curriculum_commands::pin_curriculum_version_to_cohort(&state, input)
}

#[tauri::command]
pub fn list_curriculum_version_cohort_pins(
    state: State<'_, AppState>,
    curriculum_version_id: i64,
) -> Result<Vec<CurriculumCohortPinDto>, CommandError> {
    curriculum_commands::list_curriculum_version_cohort_pins(&state, curriculum_version_id)
}

#[tauri::command]
pub fn assign_student_curriculum_version(
    state: State<'_, AppState>,
    input: StudentCurriculumAssignmentInputDto,
) -> Result<StudentCurriculumAssignmentDto, CommandError> {
    curriculum_commands::assign_student_curriculum_version(&state, input)
}

#[tauri::command]
pub fn get_active_student_curriculum_assignment(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<StudentCurriculumAssignmentDto>, CommandError> {
    curriculum_commands::get_active_student_curriculum_assignment(&state, student_id)
}

#[tauri::command]
pub fn get_student_curriculum_home(
    state: State<'_, AppState>,
    student_id: i64,
    curriculum_version_id: Option<i64>,
) -> Result<CurriculumStudentHomeSnapshotDto, CommandError> {
    curriculum_commands::get_student_curriculum_home(&state, student_id, curriculum_version_id)
}

#[tauri::command]
pub fn get_student_subject_curriculum_map(
    state: State<'_, AppState>,
    student_id: i64,
    subject_track_id: i64,
) -> Result<CurriculumStudentSubjectMapDto, CommandError> {
    curriculum_commands::get_student_subject_curriculum_map(&state, student_id, subject_track_id)
}

#[tauri::command]
pub fn get_parent_curriculum_summary(
    state: State<'_, AppState>,
    parent_id: i64,
    learner_id: i64,
    curriculum_version_id: Option<i64>,
) -> Result<CurriculumParentSummaryDto, CommandError> {
    curriculum_commands::get_parent_curriculum_summary(
        &state,
        parent_id,
        learner_id,
        curriculum_version_id,
    )
}

// Student truth

#[tauri::command]
pub fn get_learner_truth(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<LearnerTruthDto, CommandError> {
    student_commands::get_learner_truth(&state, student_id)
}

// Content and Foundry

#[tauri::command]
pub fn list_installed_packs(
    state: State<'_, AppState>,
) -> Result<Vec<PackSummaryDto>, CommandError> {
    content_commands::list_installed_packs(&state)
}

#[tauri::command]
pub fn install_pack(
    state: State<'_, AppState>,
    path: String,
) -> Result<PackInstallResultDto, CommandError> {
    content_commands::install_pack(&state, path)
}

#[tauri::command]
pub fn register_curriculum_source(
    state: State<'_, AppState>,
    input: SourceUploadInput,
) -> Result<CurriculumSourceUploadDto, CommandError> {
    content_commands::register_curriculum_source(&state, input)
}

#[tauri::command]
pub fn add_curriculum_parse_candidate(
    state: State<'_, AppState>,
    source_upload_id: i64,
    input: ParseCandidateInput,
) -> Result<CurriculumParseCandidateDto, CommandError> {
    content_commands::add_curriculum_parse_candidate(&state, source_upload_id, input)
}

#[tauri::command]
pub fn finalize_curriculum_source(
    state: State<'_, AppState>,
    source_upload_id: i64,
) -> Result<ContentFoundrySourceReportDto, CommandError> {
    content_commands::finalize_curriculum_source(&state, source_upload_id)
}

#[tauri::command]
pub fn resolve_curriculum_review_task(
    state: State<'_, AppState>,
    task_id: i64,
    resolution_note: String,
    approve_candidate: bool,
) -> Result<CurriculumReviewTaskDto, CommandError> {
    content_commands::resolve_curriculum_review_task(
        &state,
        task_id,
        resolution_note,
        approve_candidate,
    )
}

#[tauri::command]
pub fn mark_curriculum_source_reviewed(
    state: State<'_, AppState>,
    source_upload_id: i64,
) -> Result<ContentFoundrySourceReportDto, CommandError> {
    content_commands::mark_curriculum_source_reviewed(&state, source_upload_id)
}

#[tauri::command]
pub fn stage_curriculum_publish_job(
    state: State<'_, AppState>,
    source_upload_id: i64,
    requested_by_account_id: Option<i64>,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
    target_version_label: Option<String>,
) -> Result<String, CommandError> {
    content_commands::stage_curriculum_publish_job(
        &state,
        source_upload_id,
        requested_by_account_id,
        subject_id,
        topic_id,
        target_version_label,
    )
}

#[tauri::command]
pub fn recompute_topic_package_snapshot(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Option<TopicPackageSnapshotDto>, CommandError> {
    content_commands::recompute_topic_package_snapshot(&state, topic_id)
}

#[tauri::command]
pub fn get_subject_foundry_dashboard(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Option<SubjectFoundryDashboardDto>, CommandError> {
    content_commands::get_subject_foundry_dashboard(&state, subject_id)
}

#[tauri::command]
pub fn queue_topic_foundry_jobs(
    state: State<'_, AppState>,
    topic_id: i64,
    trigger_type: String,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    content_commands::queue_topic_foundry_jobs(&state, topic_id, trigger_type)
}

#[tauri::command]
pub fn queue_source_follow_up_jobs(
    state: State<'_, AppState>,
    source_upload_id: i64,
    trigger_type: String,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    content_commands::queue_source_follow_up_jobs(&state, source_upload_id, trigger_type)
}

#[tauri::command]
pub fn list_foundry_jobs(
    state: State<'_, AppState>,
    status: Option<String>,
    target_type: Option<String>,
    subject_id: Option<i64>,
) -> Result<Vec<FoundryJobDto>, CommandError> {
    content_commands::list_foundry_jobs(&state, status, target_type, subject_id)
}

#[tauri::command]
pub fn get_foundry_job_board(
    state: State<'_, AppState>,
    subject_id: Option<i64>,
) -> Result<FoundryJobBoardDto, CommandError> {
    content_commands::get_foundry_job_board(&state, subject_id)
}

#[tauri::command]
pub fn start_foundry_job(
    state: State<'_, AppState>,
    job_id: i64,
) -> Result<FoundryJobDto, CommandError> {
    content_commands::start_foundry_job(&state, job_id)
}

#[tauri::command]
pub fn complete_foundry_job(
    state: State<'_, AppState>,
    job_id: i64,
    result_summary: Value,
) -> Result<FoundryJobDto, CommandError> {
    content_commands::complete_foundry_job(&state, job_id, result_summary)
}

#[tauri::command]
pub fn fail_foundry_job(
    state: State<'_, AppState>,
    job_id: i64,
    failure_reason: String,
) -> Result<FoundryJobDto, CommandError> {
    content_commands::fail_foundry_job(&state, job_id, failure_reason)
}

#[tauri::command]
pub fn run_foundry_job(
    state: State<'_, AppState>,
    job_id: i64,
) -> Result<FoundryJobDto, CommandError> {
    content_commands::run_foundry_job(&state, job_id)
}

#[tauri::command]
pub fn run_next_foundry_job(
    state: State<'_, AppState>,
    subject_id: Option<i64>,
) -> Result<Option<FoundryJobDto>, CommandError> {
    content_commands::run_next_foundry_job(&state, subject_id)
}

#[tauri::command]
pub fn get_topic_resource_intelligence(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<TopicResourceIntelligenceSnapshotDto, CommandError> {
    content_commands::get_topic_resource_intelligence(&state, topic_id)
}

#[tauri::command]
pub fn orchestrate_resource_plan(
    state: State<'_, AppState>,
    input: ResourceOrchestrationRequest,
) -> Result<ResourceOrchestrationResultDto, CommandError> {
    content_commands::orchestrate_resource_plan(&state, input)
}

#[tauri::command]
pub fn confirm_resource_applicability(
    state: State<'_, AppState>,
    check_id: i64,
    selected_option_code: String,
    response_text: Option<String>,
) -> Result<ResourceApplicabilityResolutionDto, CommandError> {
    content_commands::confirm_resource_applicability(
        &state,
        check_id,
        selected_option_code,
        response_text,
    )
}

#[tauri::command]
pub fn record_resource_learning_outcome(
    state: State<'_, AppState>,
    input: RecordResourceLearningInput,
) -> Result<ResourceLearningRecordDto, CommandError> {
    content_commands::record_resource_learning_outcome(&state, input)
}

// Intake

#[tauri::command]
pub fn create_submission_bundle(
    state: State<'_, AppState>,
    student_id: i64,
    title: String,
) -> Result<SubmissionBundleDto, CommandError> {
    intake_commands::create_submission_bundle(&state, student_id, title)
}

#[tauri::command]
pub fn add_submission_bundle_file(
    state: State<'_, AppState>,
    bundle_id: i64,
    file_name: String,
    file_path: String,
) -> Result<i64, CommandError> {
    intake_commands::add_submission_bundle_file(&state, bundle_id, file_name, file_path)
}

#[tauri::command]
pub fn reconstruct_submission_bundle(
    state: State<'_, AppState>,
    bundle_id: i64,
) -> Result<BundleProcessReportDto, CommandError> {
    intake_commands::reconstruct_submission_bundle(&state, bundle_id)
}

#[tauri::command]
pub fn get_submission_bundle_report(
    state: State<'_, AppState>,
    bundle_id: i64,
) -> Result<BundleProcessReportDto, CommandError> {
    intake_commands::get_submission_bundle_report(&state, bundle_id)
}

#[tauri::command]
pub fn list_submission_bundle_insights(
    state: State<'_, AppState>,
    bundle_id: i64,
) -> Result<Vec<ExtractedInsightDto>, CommandError> {
    intake_commands::list_submission_bundle_insights(&state, bundle_id)
}

// Sessions and attempt pipeline

#[tauri::command]
pub fn start_practice_session(
    state: State<'_, AppState>,
    input: PracticeSessionStartInput,
) -> Result<SessionSnapshotDto, CommandError> {
    session_commands::start_practice_session(&state, input)
}

#[tauri::command]
pub fn start_coach_mission_session(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<CoachMissionSessionPlanDto, CommandError> {
    session_commands::start_coach_mission_session(&state, student_id)
}

#[tauri::command]
pub fn compose_custom_test(
    state: State<'_, AppState>,
    input: CustomTestStartInput,
) -> Result<SessionSnapshotDto, CommandError> {
    session_commands::compose_custom_test(&state, input)
}

#[tauri::command]
pub fn complete_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<SessionSummaryDto, CommandError> {
    session_commands::complete_session(&state, session_id)
}

#[tauri::command]
pub fn generate_mock_blueprint(
    state: State<'_, AppState>,
    input: MockBlueprintInput,
) -> Result<MockBlueprintDto, CommandError> {
    session_commands::generate_mock_blueprint(&state, input)
}

#[tauri::command]
pub fn start_mock_session(
    state: State<'_, AppState>,
    blueprint_id: i64,
) -> Result<SessionSnapshotDto, CommandError> {
    session_commands::start_mock_session(&state, blueprint_id)
}

#[tauri::command]
pub fn enable_focus_mode(
    state: State<'_, AppState>,
    session_id: i64,
    focus_goal: Option<String>,
    break_schedule_json: Option<Value>,
    ambient_profile: Option<String>,
) -> Result<FocusModeConfigDto, CommandError> {
    session_commands::enable_focus_mode(
        &state,
        session_id,
        focus_goal,
        break_schedule_json,
        ambient_profile,
    )
}

#[tauri::command]
pub fn get_focus_mode_config(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<Option<FocusModeConfigDto>, CommandError> {
    session_commands::get_focus_mode_config(&state, session_id)
}

#[tauri::command]
pub fn manual_stop_session(
    state: State<'_, AppState>,
    session_id: i64,
    reason: Option<String>,
) -> Result<SessionSnapshotDto, CommandError> {
    session_commands::manual_stop_session(&state, session_id, reason)
}

#[tauri::command]
pub fn record_session_presence_event(
    state: State<'_, AppState>,
    session_id: i64,
    input: ecoach_commands::session_commands::SessionPresenceEventInputDto,
) -> Result<ecoach_commands::session_commands::SessionPresenceSnapshotDto, CommandError> {
    session_commands::record_session_presence_event(&state, session_id, input)
}

#[tauri::command]
pub fn get_session_presence_snapshot(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<Option<ecoach_commands::session_commands::SessionPresenceSnapshotDto>, CommandError> {
    session_commands::get_session_presence_snapshot(&state, session_id)
}

#[tauri::command]
pub fn list_session_presence_events(
    state: State<'_, AppState>,
    session_id: i64,
    limit: usize,
) -> Result<Vec<ecoach_commands::session_commands::SessionPresenceEventDto>, CommandError> {
    session_commands::list_session_presence_events(&state, session_id, limit)
}

#[tauri::command]
pub fn submit_attempt(
    state: State<'_, AppState>,
    input: SubmitAttemptInput,
) -> Result<AttemptResultDto, CommandError> {
    attempt_commands::submit_attempt(&state, input)
}

#[tauri::command]
pub fn complete_session_with_pipeline(
    state: State<'_, AppState>,
    student_id: i64,
    session_id: i64,
) -> Result<SessionCompletionResultDto, CommandError> {
    attempt_commands::complete_session_with_pipeline(&state, student_id, session_id)
}

// Diagnostics

#[tauri::command]
pub fn launch_diagnostic(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    mode: String,
) -> Result<DiagnosticRunDto, CommandError> {
    diagnostic_commands::launch_diagnostic(&state, student_id, subject_id, mode)
}

#[tauri::command]
pub fn get_diagnostic_battery(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<DiagnosticBatteryDto, CommandError> {
    diagnostic_commands::get_diagnostic_battery(&state, diagnostic_id)
}

#[tauri::command]
pub fn get_diagnostic_subject_blueprint(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Option<DiagnosticSubjectBlueprintDto>, CommandError> {
    diagnostic_commands::get_diagnostic_subject_blueprint(&state, subject_id)
}

#[tauri::command]
pub fn list_diagnostic_item_routing_profiles(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticItemRoutingProfileDto>, CommandError> {
    diagnostic_commands::list_diagnostic_item_routing_profiles(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_problem_cause_fix_cards(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticProblemCauseFixCardDto>, CommandError> {
    diagnostic_commands::list_diagnostic_problem_cause_fix_cards(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_intervention_prescriptions(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticInterventionPrescriptionDto>, CommandError> {
    diagnostic_commands::list_diagnostic_intervention_prescriptions(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_phase_items(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    phase_number: i64,
) -> Result<Vec<DiagnosticPhaseItemDto>, CommandError> {
    diagnostic_commands::list_diagnostic_phase_items(&state, diagnostic_id, phase_number)
}

#[tauri::command]
pub fn submit_diagnostic_attempt(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    input: SubmitDiagnosticAttemptInput,
) -> Result<(), CommandError> {
    diagnostic_commands::submit_diagnostic_attempt(&state, diagnostic_id, input)
}

#[tauri::command]
pub fn advance_diagnostic_phase(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    completed_phase_number: i64,
) -> Result<Option<DiagnosticPhasePlanDto>, CommandError> {
    diagnostic_commands::advance_diagnostic_phase(&state, diagnostic_id, completed_phase_number)
}

#[tauri::command]
pub fn get_diagnostic_report(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<TopicAnalyticsDto>, CommandError> {
    diagnostic_commands::get_diagnostic_report(&state, diagnostic_id)
}

#[tauri::command]
pub fn complete_diagnostic_and_sync(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<DiagnosticCompletionSyncDto, CommandError> {
    diagnostic_commands::complete_diagnostic_and_sync(&state, diagnostic_id)
}

#[tauri::command]
pub fn get_diagnostic_result(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Option<DiagnosticResultDto>, CommandError> {
    diagnostic_commands::get_diagnostic_result(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_skill_results(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticSkillResultDto>, CommandError> {
    diagnostic_commands::list_diagnostic_skill_results(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_recommendations(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticRecommendationDto>, CommandError> {
    diagnostic_commands::list_diagnostic_recommendations(&state, diagnostic_id)
}

#[tauri::command]
pub fn get_diagnostic_audience_report(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    audience: String,
) -> Result<Option<DiagnosticAudienceReportDto>, CommandError> {
    diagnostic_commands::get_diagnostic_audience_report(&state, diagnostic_id, audience)
}

#[tauri::command]
pub fn get_diagnostic_longitudinal_summary(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Option<DiagnosticLongitudinalSummaryDto>, CommandError> {
    diagnostic_commands::get_diagnostic_longitudinal_summary(&state, diagnostic_id)
}

#[tauri::command]
pub fn list_diagnostic_cause_evolution(
    state: State<'_, AppState>,
    diagnostic_id: i64,
) -> Result<Vec<DiagnosticCauseEvolutionDto>, CommandError> {
    diagnostic_commands::list_diagnostic_cause_evolution(&state, diagnostic_id)
}

// Question intelligence

#[tauri::command]
pub fn choose_reactor_family(
    state: State<'_, AppState>,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionFamilyChoiceDto>, CommandError> {
    question_commands::choose_reactor_family(&state, slot_spec)
}

#[tauri::command]
pub fn create_question_generation_request(
    state: State<'_, AppState>,
    input: QuestionGenerationRequestInput,
) -> Result<QuestionGenerationRequestDto, CommandError> {
    question_commands::create_question_generation_request(&state, input)
}

#[tauri::command]
pub fn process_question_generation_request(
    state: State<'_, AppState>,
    request_id: i64,
) -> Result<Vec<GeneratedQuestionDraftDto>, CommandError> {
    question_commands::process_question_generation_request(&state, request_id)
}

#[tauri::command]
pub fn get_question_lineage(
    state: State<'_, AppState>,
    question_id: i64,
) -> Result<QuestionLineageGraphDto, CommandError> {
    question_commands::get_question_lineage(&state, question_id)
}

#[tauri::command]
pub fn get_question_family_health(
    state: State<'_, AppState>,
    family_id: i64,
) -> Result<Option<QuestionFamilyHealthDto>, CommandError> {
    question_commands::get_question_family_health(&state, family_id)
}

#[tauri::command]
pub fn list_related_questions(
    state: State<'_, AppState>,
    question_id: i64,
    relation_type: Option<String>,
    limit: usize,
) -> Result<Vec<RelatedQuestionDto>, CommandError> {
    question_commands::list_related_questions(&state, question_id, relation_type, limit)
}

#[tauri::command]
pub fn detect_near_duplicate(
    state: State<'_, AppState>,
    stem: String,
    family_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<DuplicateCheckResultDto, CommandError> {
    question_commands::detect_near_duplicate(&state, stem, family_id, topic_id)
}

#[tauri::command]
pub fn recommend_question_remediation_plan(
    state: State<'_, AppState>,
    student_id: i64,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionRemediationPlanDto>, CommandError> {
    question_commands::recommend_question_remediation_plan(&state, student_id, slot_spec)
}

#[tauri::command]
pub fn get_question_intelligence(
    state: State<'_, AppState>,
    question_id: i64,
) -> Result<Option<QuestionIntelligenceSnapshotDto>, CommandError> {
    question_commands::get_question_intelligence(&state, question_id)
}

#[tauri::command]
pub fn classify_question_intelligence(
    state: State<'_, AppState>,
    question_id: i64,
    reclassify: bool,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    question_commands::classify_question_intelligence(&state, question_id, reclassify)
}

#[tauri::command]
pub fn find_questions_by_intelligence_filter(
    state: State<'_, AppState>,
    filter: QuestionIntelligenceFilter,
) -> Result<Vec<QuestionIntelligenceSnapshotDto>, CommandError> {
    question_commands::find_questions_by_intelligence_filter(&state, filter)
}

#[tauri::command]
pub fn list_question_review_queue(
    state: State<'_, AppState>,
    review_status: Option<String>,
    limit: usize,
) -> Result<Vec<QuestionReviewQueueItemDto>, CommandError> {
    question_commands::list_question_review_queue(&state, review_status, limit)
}

#[tauri::command]
pub fn review_question_intelligence(
    state: State<'_, AppState>,
    question_id: i64,
    input: QuestionReviewActionInput,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    question_commands::review_question_intelligence(&state, question_id, input)
}

#[tauri::command]
pub fn queue_question_reclassification(
    state: State<'_, AppState>,
    question_id: i64,
    trigger_reason: String,
    requested_by: Option<String>,
) -> Result<i64, CommandError> {
    question_commands::queue_question_reclassification(
        &state,
        question_id,
        trigger_reason,
        requested_by,
    )
}

#[tauri::command]
pub fn list_inverse_pressure_families(
    state: State<'_, AppState>,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperInverseSignalDto>, CommandError> {
    assessment_commands::list_inverse_pressure_families(&state, subject_id, topic_id, limit)
}

#[tauri::command]
pub fn list_comeback_candidate_families(
    state: State<'_, AppState>,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperComebackSignalDto>, CommandError> {
    assessment_commands::list_comeback_candidate_families(&state, subject_id, topic_id, limit)
}

#[tauri::command]
pub fn list_session_remediation_plans(
    state: State<'_, AppState>,
    session_id: i64,
    limit: usize,
) -> Result<Vec<SessionRemediationPlanDto>, CommandError> {
    assessment_commands::list_session_remediation_plans(&state, session_id, limit)
}

#[tauri::command]
pub fn get_session_evidence_fabric(
    state: State<'_, AppState>,
    session_id: i64,
    limit_events: usize,
) -> Result<Option<SessionEvidenceFabricDto>, CommandError> {
    assessment_commands::get_session_evidence_fabric(&state, session_id, limit_events)
}

#[tauri::command]
pub fn build_elite_session_blueprint(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteSessionBlueprintDto, CommandError> {
    assessment_commands::build_elite_session_blueprint(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_elite_profile(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<EliteProfileDto>, CommandError> {
    assessment_commands::get_elite_profile(&state, student_id, subject_id)
}

#[tauri::command]
pub fn list_elite_topic_domination(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    limit: usize,
) -> Result<Vec<EliteTopicProfileDto>, CommandError> {
    assessment_commands::list_elite_topic_domination(&state, student_id, subject_id, limit)
}

#[tauri::command]
pub fn build_elite_session_blueprint_report(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteBlueprintReportDto, CommandError> {
    assessment_commands::build_elite_session_blueprint_report(&state, student_id, subject_id)
}

// Games

#[tauri::command]
pub fn start_game(
    state: State<'_, AppState>,
    input: StartGameInput,
) -> Result<GameSessionDto, CommandError> {
    game_commands::start_game(&state, input)
}

#[tauri::command]
pub fn submit_game_answer(
    state: State<'_, AppState>,
    input: SubmitGameAnswerInput,
) -> Result<GameAnswerResultDto, CommandError> {
    game_commands::submit_game_answer(&state, input)
}

#[tauri::command]
pub fn get_game_summary(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<GameSummaryDto, CommandError> {
    game_commands::get_game_summary(&state, session_id)
}

#[tauri::command]
pub fn get_trap_session_snapshot(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<TrapSessionSnapshotDto, CommandError> {
    game_commands::get_trap_session_snapshot(&state, session_id)
}

#[tauri::command]
pub fn reveal_trap_unmask_clue(
    state: State<'_, AppState>,
    session_id: i64,
    round_id: i64,
) -> Result<TrapRoundCardDto, CommandError> {
    game_commands::reveal_trap_unmask_clue(&state, session_id, round_id)
}

#[tauri::command]
pub fn get_mindstack_state(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<MindstackStateDto, CommandError> {
    game_commands::get_mindstack_state(&state, session_id)
}

#[tauri::command]
pub fn get_tug_of_war_state(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<TugOfWarStateDto, CommandError> {
    game_commands::get_tug_of_war_state(&state, session_id)
}

#[tauri::command]
pub fn list_game_sessions(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<GameSessionDto>, CommandError> {
    game_commands::list_game_sessions(&state, student_id, limit)
}

#[tauri::command]
pub fn create_duel_session(
    state: State<'_, AppState>,
    challenger_id: i64,
    opponent_id: Option<i64>,
    subject_id: i64,
    topic_id: Option<i64>,
    duel_type: String,
    question_count: usize,
    time_limit_seconds: Option<i64>,
) -> Result<DuelSessionDto, CommandError> {
    game_commands::create_duel_session(
        &state,
        challenger_id,
        opponent_id,
        subject_id,
        topic_id,
        &duel_type,
        question_count,
        time_limit_seconds,
    )
}

#[tauri::command]
pub fn list_duel_sessions(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<DuelSessionDto>, CommandError> {
    game_commands::list_duel_sessions(&state, student_id)
}

#[tauri::command]
pub fn record_duel_outcome(
    state: State<'_, AppState>,
    duel_session_id: i64,
    challenger_score_bp: u16,
    opponent_score_bp: u16,
    winner_id: Option<i64>,
) -> Result<DuelSessionDto, CommandError> {
    game_commands::record_duel_outcome(
        &state,
        duel_session_id,
        challenger_score_bp,
        opponent_score_bp,
        winner_id,
    )
}

#[tauri::command]
pub fn get_leaderboard(
    state: State<'_, AppState>,
    game_type: String,
    limit: usize,
) -> Result<Vec<LeaderboardEntryDto>, CommandError> {
    game_commands::get_leaderboard(&state, game_type, limit)
}

#[tauri::command]
pub fn pause_game(state: State<'_, AppState>, session_id: i64) -> Result<(), CommandError> {
    game_commands::pause_game(&state, session_id)
}

#[tauri::command]
pub fn resume_game(state: State<'_, AppState>, session_id: i64) -> Result<(), CommandError> {
    game_commands::resume_game(&state, session_id)
}

#[tauri::command]
pub fn abandon_game(state: State<'_, AppState>, session_id: i64) -> Result<(), CommandError> {
    game_commands::abandon_game(&state, session_id)
}

// Traps

#[tauri::command]
pub fn list_traps_pairs(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    topic_ids: Vec<i64>,
) -> Result<Vec<ContrastPairSummaryDto>, CommandError> {
    traps_commands::list_traps_pairs(&state, student_id, subject_id, topic_ids)
}

#[tauri::command]
pub fn get_contrast_pair_profile(
    state: State<'_, AppState>,
    student_id: i64,
    pair_id: i64,
) -> Result<ContrastPairProfileDto, CommandError> {
    traps_commands::get_contrast_pair_profile(&state, student_id, pair_id)
}

#[tauri::command]
pub fn list_trap_misconception_reasons(
    state: State<'_, AppState>,
    mode: Option<String>,
) -> Result<Vec<TrapMisconceptionReasonDto>, CommandError> {
    traps_commands::list_trap_misconception_reasons(&state, mode)
}

#[tauri::command]
pub fn start_traps_session(
    state: State<'_, AppState>,
    input: StartTrapsSessionInput,
) -> Result<TrapSessionSnapshotDto, CommandError> {
    traps_commands::start_traps_session(&state, input)
}

#[tauri::command]
pub fn submit_trap_round(
    state: State<'_, AppState>,
    input: SubmitTrapRoundInput,
) -> Result<TrapRoundResultDto, CommandError> {
    traps_commands::submit_trap_round(&state, input)
}

#[tauri::command]
pub fn record_trap_confusion_reason(
    state: State<'_, AppState>,
    input: SubmitTrapConfusionReasonInput,
) -> Result<(), CommandError> {
    traps_commands::record_trap_confusion_reason(&state, input)
}

#[tauri::command]
pub fn get_trap_review(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<TrapReviewDto, CommandError> {
    traps_commands::get_trap_review(&state, session_id)
}

// Library and glossary

#[tauri::command]
pub fn get_library_home(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<LibraryShelfDto>, CommandError> {
    library_commands::get_library_home(&state, student_id)
}

#[tauri::command]
pub fn save_library_item(
    state: State<'_, AppState>,
    student_id: i64,
    item_type: String,
    reference_id: i64,
) -> Result<i64, CommandError> {
    library_commands::save_library_item(&state, student_id, item_type, reference_id)
}

#[tauri::command]
pub fn save_library_item_with_metadata(
    state: State<'_, AppState>,
    student_id: i64,
    input: SaveLibraryItemInputDto,
) -> Result<i64, CommandError> {
    library_commands::save_library_item_with_metadata(&state, student_id, input)
}

#[tauri::command]
pub fn update_library_item(
    state: State<'_, AppState>,
    library_item_id: i64,
    input: UpdateLibraryItemInputDto,
) -> Result<(), CommandError> {
    library_commands::update_library_item(&state, library_item_id, input)
}

#[tauri::command]
pub fn remove_library_item(
    state: State<'_, AppState>,
    library_item_id: i64,
) -> Result<(), CommandError> {
    library_commands::remove_library_item(&state, library_item_id)
}

#[tauri::command]
pub fn list_library_items(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<LibraryItemRecordDto>, CommandError> {
    library_commands::list_library_items(&state, student_id)
}

#[tauri::command]
pub fn list_library_item_state_history(
    state: State<'_, AppState>,
    library_item_id: i64,
    limit: usize,
) -> Result<Vec<LibraryItemStateHistoryEntryDto>, CommandError> {
    library_commands::list_library_item_state_history(&state, library_item_id, limit)
}

#[tauri::command]
pub fn record_library_item_action(
    state: State<'_, AppState>,
    student_id: i64,
    library_item_id: i64,
    input: RecordLibraryItemActionInputDto,
) -> Result<i64, CommandError> {
    library_commands::record_library_item_action(&state, student_id, library_item_id, input)
}

#[tauri::command]
pub fn list_library_item_actions(
    state: State<'_, AppState>,
    library_item_id: i64,
    limit: usize,
) -> Result<Vec<LibraryItemActionDto>, CommandError> {
    library_commands::list_library_item_actions(&state, library_item_id, limit)
}

#[tauri::command]
pub fn add_library_note(
    state: State<'_, AppState>,
    input: AddLibraryNoteInputDto,
) -> Result<i64, CommandError> {
    library_commands::add_library_note(&state, input)
}

#[tauri::command]
pub fn list_library_notes(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: Option<i64>,
    library_item_id: Option<i64>,
    limit: usize,
) -> Result<Vec<LibraryNoteDto>, CommandError> {
    library_commands::list_library_notes(&state, student_id, topic_id, library_item_id, limit)
}

#[tauri::command]
pub fn search_library(
    state: State<'_, AppState>,
    student_id: i64,
    input: LibrarySearchInputDto,
    limit: usize,
) -> Result<Vec<LibrarySearchResultDto>, CommandError> {
    library_commands::search_library(&state, student_id, input, limit)
}

#[tauri::command]
pub fn list_revision_pack_templates(
    state: State<'_, AppState>,
) -> Result<Vec<RevisionPackTemplateDto>, CommandError> {
    library_commands::list_revision_pack_templates(&state)
}

#[tauri::command]
pub fn build_revision_pack_from_template(
    state: State<'_, AppState>,
    student_id: i64,
    input: BuildRevisionPackFromTemplateInputDto,
) -> Result<RevisionPackSummaryDto, CommandError> {
    library_commands::build_revision_pack_from_template(&state, student_id, input)
}

#[tauri::command]
pub fn list_revision_packs(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<RevisionPackSummaryDto>, CommandError> {
    library_commands::list_revision_packs(&state, student_id, limit)
}

#[tauri::command]
pub fn create_custom_revision_pack(
    state: State<'_, AppState>,
    student_id: i64,
    title: String,
    question_ids: Vec<i64>,
    subject_id: Option<i64>,
) -> Result<RevisionPackSummaryDto, CommandError> {
    library_commands::create_custom_revision_pack(
        &state,
        student_id,
        title,
        question_ids,
        subject_id,
    )
}

#[tauri::command]
pub fn list_exam_hotspots(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    limit: usize,
) -> Result<Vec<ExamHotspotDto>, CommandError> {
    library_commands::list_exam_hotspots(&state, student_id, subject_id, limit)
}

#[tauri::command]
pub fn get_topic_library_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<TopicLibrarySnapshotDto, CommandError> {
    library_commands::get_topic_library_snapshot(&state, student_id, topic_id, limit)
}

#[tauri::command]
pub fn create_custom_shelf(
    state: State<'_, AppState>,
    student_id: i64,
    input: CreateCustomShelfInputDto,
) -> Result<i64, CommandError> {
    library_commands::create_custom_shelf(&state, student_id, input)
}

#[tauri::command]
pub fn add_item_to_custom_shelf(
    state: State<'_, AppState>,
    student_id: i64,
    shelf_id: i64,
    input: AddShelfItemInputDto,
) -> Result<i64, CommandError> {
    library_commands::add_item_to_custom_shelf(&state, student_id, shelf_id, input)
}

#[tauri::command]
pub fn list_custom_shelves(
    state: State<'_, AppState>,
    student_id: i64,
    include_items: bool,
    item_limit: usize,
) -> Result<Vec<CustomLibraryShelfDto>, CommandError> {
    library_commands::list_custom_shelves(&state, student_id, include_items, item_limit)
}

#[tauri::command]
pub fn list_offline_library_items(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<OfflineLibraryItemDto>, CommandError> {
    library_commands::list_offline_library_items(&state, student_id, limit)
}

#[tauri::command]
pub fn list_library_tag_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<LibraryTagDefinitionDto>, CommandError> {
    library_commands::list_library_tag_definitions(&state)
}

#[tauri::command]
pub fn search_glossary(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError> {
    library_commands::search_glossary(&state, query)
}

#[tauri::command]
pub fn search_catalog(
    state: State<'_, AppState>,
    input: GlossarySearchInputDto,
    limit: usize,
) -> Result<GlossarySearchResponseDto, CommandError> {
    library_commands::search_catalog(&state, input, limit)
}

#[tauri::command]
pub fn search_suggestions(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<GlossarySearchSuggestionDto>, CommandError> {
    library_commands::search_suggestions(&state, query, limit)
}

#[tauri::command]
pub fn search_voice(
    state: State<'_, AppState>,
    query: String,
    student_id: Option<i64>,
    limit: usize,
) -> Result<GlossarySearchResponseDto, CommandError> {
    library_commands::search_voice(&state, query, student_id, limit)
}

#[tauri::command]
pub fn get_entry_detail(
    state: State<'_, AppState>,
    student_id: Option<i64>,
    entry_id: i64,
    relation_limit: usize,
    bundle_limit: usize,
) -> Result<GlossaryEntryDetailDto, CommandError> {
    library_commands::get_entry_detail(&state, student_id, entry_id, relation_limit, bundle_limit)
}

#[tauri::command]
pub fn build_home_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: Option<i64>,
    limit: usize,
) -> Result<GlossaryHomeSnapshotDto, CommandError> {
    library_commands::build_home_snapshot(&state, student_id, subject_id, limit)
}

#[tauri::command]
pub fn build_compare_view(
    state: State<'_, AppState>,
    left_entry_id: i64,
    right_entry_id: i64,
) -> Result<GlossaryComparisonViewDto, CommandError> {
    library_commands::build_compare_view(&state, left_entry_id, right_entry_id)
}

#[tauri::command]
pub fn get_formula_lab(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<FormulaLabViewDto, CommandError> {
    library_commands::get_formula_lab(&state, entry_id)
}

#[tauri::command]
pub fn build_concept_map(
    state: State<'_, AppState>,
    entry_id: i64,
    depth: usize,
    limit: usize,
) -> Result<ConceptMapViewDto, CommandError> {
    library_commands::build_concept_map(&state, entry_id, depth, limit)
}

#[tauri::command]
pub fn record_interaction(
    state: State<'_, AppState>,
    input: GlossaryInteractionInputDto,
) -> Result<i64, CommandError> {
    library_commands::record_interaction(&state, input)
}

#[tauri::command]
pub fn start_audio_queue(
    state: State<'_, AppState>,
    student_id: i64,
    input: StartGlossaryAudioQueueInputDto,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    library_commands::start_audio_queue(&state, student_id, input)
}

#[tauri::command]
pub fn next_audio_queue(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    library_commands::next_audio_queue(&state, student_id)
}

#[tauri::command]
pub fn previous_audio_queue(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    library_commands::previous_audio_queue(&state, student_id)
}

#[tauri::command]
pub fn update_audio_queue(
    state: State<'_, AppState>,
    student_id: i64,
    input: UpdateGlossaryAudioQueueInputDto,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    library_commands::update_audio_queue(&state, student_id, input)
}

#[tauri::command]
pub fn current_audio_queue(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    library_commands::current_audio_queue(&state, student_id)
}

#[tauri::command]
pub fn create_glossary_test_session(
    state: State<'_, AppState>,
    student_id: i64,
    input: CreateGlossaryTestInputDto,
) -> Result<GlossaryTestSessionDetailDto, CommandError> {
    library_commands::create_glossary_test_session(&state, student_id, input)
}

#[tauri::command]
pub fn get_glossary_test_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<GlossaryTestSessionDetailDto, CommandError> {
    library_commands::get_glossary_test_session(&state, session_id)
}

#[tauri::command]
pub fn submit_glossary_test_attempt(
    state: State<'_, AppState>,
    student_id: i64,
    session_id: i64,
    input: SubmitGlossaryTestAttemptInputDto,
) -> Result<GlossaryTestAttemptResultDto, CommandError> {
    library_commands::submit_glossary_test_attempt(&state, student_id, session_id, input)
}

#[tauri::command]
pub fn rebuild_search_index(state: State<'_, AppState>) -> Result<usize, CommandError> {
    library_commands::rebuild_search_index(&state)
}

#[tauri::command]
pub fn get_library_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<LibraryHomeSnapshotDto, CommandError> {
    library_commands::get_library_snapshot(&state, student_id)
}

#[tauri::command]
pub fn get_continue_learning_card(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<ContinueLearningCardDto>, CommandError> {
    library_commands::get_continue_learning_card(&state, student_id)
}

#[tauri::command]
pub fn list_personalized_learning_paths(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<PersonalizedLearningPathDto>, CommandError> {
    library_commands::list_personalized_learning_paths(&state, student_id, limit)
}

#[tauri::command]
pub fn build_revision_pack(
    state: State<'_, AppState>,
    student_id: i64,
    title: String,
    question_limit: usize,
) -> Result<RevisionPackSummaryDto, CommandError> {
    library_commands::build_revision_pack(&state, student_id, title, question_limit)
}

#[tauri::command]
pub fn list_revision_pack_items(
    state: State<'_, AppState>,
    pack_id: i64,
) -> Result<Vec<RevisionPackItemDto>, CommandError> {
    library_commands::list_revision_pack_items(&state, pack_id)
}

#[tauri::command]
pub fn list_glossary_bundles_for_topic(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Vec<KnowledgeBundleDto>, CommandError> {
    library_commands::list_glossary_bundles_for_topic(&state, topic_id)
}

#[tauri::command]
pub fn list_glossary_bundle_sequence_for_topic(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<Vec<KnowledgeBundleSequenceItemDto>, CommandError> {
    library_commands::list_glossary_bundle_sequence_for_topic(&state, student_id, topic_id, limit)
}

#[tauri::command]
pub fn list_glossary_entries_for_question(
    state: State<'_, AppState>,
    question_id: i64,
) -> Result<Vec<QuestionKnowledgeLinkDto>, CommandError> {
    library_commands::list_glossary_entries_for_question(&state, question_id)
}

#[tauri::command]
pub fn build_glossary_audio_program_for_topic(
    state: State<'_, AppState>,
    topic_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    library_commands::build_glossary_audio_program_for_topic(&state, topic_id, limit)
}

#[tauri::command]
pub fn build_personalized_glossary_audio_program_for_topic(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    library_commands::build_personalized_glossary_audio_program_for_topic(
        &state, student_id, topic_id, limit,
    )
}

#[tauri::command]
pub fn build_glossary_audio_program_for_question(
    state: State<'_, AppState>,
    question_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    library_commands::build_glossary_audio_program_for_question(&state, question_id, limit)
}

#[tauri::command]
pub fn build_personalized_glossary_audio_program_for_question(
    state: State<'_, AppState>,
    student_id: i64,
    question_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    library_commands::build_personalized_glossary_audio_program_for_question(
        &state,
        student_id,
        question_id,
        limit,
    )
}

#[tauri::command]
pub fn build_teach_action_plan(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<TeachActionPlanDto, CommandError> {
    library_commands::build_teach_action_plan(&state, student_id, topic_id, limit)
}

#[tauri::command]
pub fn upsert_teach_explanation(
    state: State<'_, AppState>,
    node_id: i64,
    input: TeachExplanationUpsertInput,
) -> Result<i64, CommandError> {
    library_commands::upsert_teach_explanation(&state, node_id, input)
}

#[tauri::command]
pub fn add_teach_micro_check(
    state: State<'_, AppState>,
    explanation_id: i64,
    input: TeachMicroCheckInput,
) -> Result<i64, CommandError> {
    library_commands::add_teach_micro_check(&state, explanation_id, input)
}

#[tauri::command]
pub fn get_teach_lesson(
    state: State<'_, AppState>,
    topic_id: i64,
    explanation_level: Option<String>,
    micro_check_limit: usize,
) -> Result<TeachLessonDto, CommandError> {
    library_commands::get_teach_lesson(&state, topic_id, explanation_level, micro_check_limit)
}

#[tauri::command]
pub fn ask_tutor(
    state: State<'_, AppState>,
    input: TutorInteractionInput,
) -> Result<TutorResponseDto, CommandError> {
    library_commands::ask_tutor(&state, input)
}

#[tauri::command]
pub fn list_recent_tutor_interactions(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TutorInteractionDto>, CommandError> {
    library_commands::list_recent_tutor_interactions(&state, student_id, limit)
}

#[tauri::command]
pub fn list_topic_relationship_hints(
    state: State<'_, AppState>,
    topic_id: i64,
    limit: usize,
) -> Result<Vec<TopicRelationshipHintDto>, CommandError> {
    library_commands::list_topic_relationship_hints(&state, topic_id, limit)
}

// Memory

#[tauri::command]
pub fn get_review_queue(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<RecheckItemDto>, CommandError> {
    memory_commands::get_review_queue(&state, student_id, limit)
}

#[tauri::command]
pub fn record_retrieval_attempt(
    state: State<'_, AppState>,
    input: RecordMemoryEvidenceInput,
) -> Result<MemoryStateDto, CommandError> {
    memory_commands::record_retrieval_attempt(&state, input)
}

#[tauri::command]
pub fn get_memory_dashboard(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<MemoryDashboardDto, CommandError> {
    memory_commands::get_memory_dashboard(&state, student_id)
}

#[tauri::command]
pub fn process_decay_batch(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<DecayBatchResultDto, CommandError> {
    memory_commands::process_decay_batch(&state, limit)
}

#[tauri::command]
pub fn complete_recheck(state: State<'_, AppState>, recheck_id: i64) -> Result<(), CommandError> {
    memory_commands::complete_recheck(&state, recheck_id)
}

#[tauri::command]
pub fn build_memory_review_queue(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<MemoryReviewQueueItemDto>, CommandError> {
    memory_commands::build_review_queue(&state, student_id, limit)
}

#[tauri::command]
pub fn list_memory_topic_summaries(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TopicMemorySummaryDto>, CommandError> {
    memory_commands::list_memory_topic_summaries(&state, student_id, limit)
}

#[tauri::command]
pub fn build_memory_return_loop(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<MemoryReturnLoopDto, CommandError> {
    memory_commands::build_memory_return_loop(&state, student_id, limit)
}

#[tauri::command]
pub fn get_memory_knowledge_state(
    state: State<'_, AppState>,
    student_id: i64,
    node_id: i64,
) -> Result<MemoryKnowledgeStateDto, CommandError> {
    memory_commands::get_memory_knowledge_state(&state, student_id, node_id)
}

#[tauri::command]
pub fn list_memory_review_schedule(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ReviewScheduleItemDto>, CommandError> {
    memory_commands::list_memory_review_schedule(&state, student_id, limit)
}

#[tauri::command]
pub fn list_active_interventions(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ActiveInterventionDto>, CommandError> {
    memory_commands::list_active_interventions(&state, student_id, limit)
}

#[tauri::command]
pub fn complete_intervention_step(
    state: State<'_, AppState>,
    input: InterventionStepInputDto,
) -> Result<ActiveInterventionDto, CommandError> {
    memory_commands::complete_intervention_step(&state, input)
}

#[tauri::command]
pub fn force_recompute_knowledge_state(
    state: State<'_, AppState>,
    student_id: i64,
    knowledge_unit_id: i64,
) -> Result<MemoryKnowledgeStateDto, CommandError> {
    memory_commands::force_recompute_knowledge_state(&state, student_id, knowledge_unit_id)
}

#[tauri::command]
pub fn get_memory_cohort_analytics(
    state: State<'_, AppState>,
    topic_id: i64,
    hotspot_limit: usize,
) -> Result<MemoryCohortAnalyticsDto, CommandError> {
    memory_commands::get_memory_cohort_analytics(&state, topic_id, hotspot_limit)
}

#[tauri::command]
pub fn list_student_interference_edges(
    state: State<'_, AppState>,
    student_id: i64,
    node_id: i64,
    limit: usize,
) -> Result<Vec<StudentInterferenceEdgeDto>, CommandError> {
    memory_commands::list_student_interference_edges(&state, student_id, node_id, limit)
}

#[tauri::command]
pub fn get_topic_knowledge_map(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<TopicKnowledgeMapDto, CommandError> {
    memory_commands::get_topic_knowledge_map(&state, topic_id)
}

// Knowledge gaps and repair

#[tauri::command]
pub fn list_priority_gaps(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<GapScoreCardDto>, CommandError> {
    repair_commands::list_priority_gaps(&state, student_id, limit)
}

#[tauri::command]
pub fn generate_repair_plan(
    state: State<'_, AppState>,
    student_id: i64,
    topic_id: i64,
) -> Result<GapRepairPlanDto, CommandError> {
    repair_commands::generate_repair_plan(&state, student_id, topic_id)
}

#[tauri::command]
pub fn get_repair_plan(
    state: State<'_, AppState>,
    plan_id: i64,
) -> Result<GapRepairPlanDetailDto, CommandError> {
    repair_commands::get_repair_plan(&state, plan_id)
}

#[tauri::command]
pub fn advance_repair_item(
    state: State<'_, AppState>,
    item_id: i64,
    completed: bool,
) -> Result<GapRepairPlanDto, CommandError> {
    repair_commands::advance_repair_item(&state, item_id, completed)
}

#[tauri::command]
pub fn get_gap_dashboard(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<GapDashboardDto, CommandError> {
    repair_commands::get_gap_dashboard(&state, student_id)
}

// Readiness and parent reporting

#[tauri::command]
pub fn get_readiness_report(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<ReadinessReportDto, CommandError> {
    readiness_commands::get_readiness_report(&state, student_id)
}

#[tauri::command]
pub fn generate_parent_digest(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<ParentDigestDto, CommandError> {
    readiness_commands::generate_parent_digest(&state, parent_id)
}

#[tauri::command]
pub fn get_parent_dashboard(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<ParentDashboardSnapshotDto, CommandError> {
    reporting_commands::get_parent_dashboard(&state, parent_id)
}

#[tauri::command]
pub fn get_household_dashboard(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<HouseholdDashboardSnapshotDto, CommandError> {
    reporting_commands::get_household_dashboard(&state, parent_id)
}

#[tauri::command]
pub fn get_admin_oversight_snapshot(
    state: State<'_, AppState>,
    admin_id: i64,
) -> Result<AdminOversightSnapshotDto, CommandError> {
    reporting_commands::get_admin_oversight_snapshot(&state, admin_id)
}

#[tauri::command]
pub fn get_reporting_strategy_summary(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Option<ReportingStrategySummaryDto>, CommandError> {
    reporting_commands::get_reporting_strategy_summary(&state, student_id)
}

// Premium

#[tauri::command]
pub fn get_risk_dashboard(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<RiskDashboardDto, CommandError> {
    premium_commands::get_risk_dashboard(&state, student_id)
}

#[tauri::command]
pub fn auto_detect_risks(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<Vec<RiskFlagDto>, CommandError> {
    premium_commands::auto_detect_risks(&state, student_id)
}

#[tauri::command]
pub fn create_intervention(
    state: State<'_, AppState>,
    input: CreateInterventionInput,
) -> Result<InterventionDto, CommandError> {
    premium_commands::create_intervention(&state, input)
}

#[tauri::command]
pub fn resolve_risk_flag(
    state: State<'_, AppState>,
    flag_id: i64,
) -> Result<RiskFlagDto, CommandError> {
    premium_commands::resolve_risk_flag(&state, flag_id)
}

#[tauri::command]
pub fn resolve_intervention(
    state: State<'_, AppState>,
    intervention_id: i64,
) -> Result<InterventionDto, CommandError> {
    premium_commands::resolve_intervention(&state, intervention_id)
}

#[tauri::command]
pub fn check_entitlement(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<EntitlementSnapshotDto, CommandError> {
    premium_commands::check_entitlement(&state, student_id)
}

#[tauri::command]
pub fn is_feature_enabled(
    state: State<'_, AppState>,
    student_id: i64,
    feature_key: String,
) -> Result<bool, CommandError> {
    premium_commands::is_feature_enabled(&state, student_id, feature_key)
}

#[tauri::command]
pub fn get_premium_strategy_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<PremiumStrategySnapshotDto, CommandError> {
    premium_commands::get_strategy_snapshot(&state, student_id)
}

// Mock centre

#[tauri::command]
pub fn compile_mock(
    state: State<'_, AppState>,
    input: CompileMockInput,
) -> Result<MockSessionDto, CommandError> {
    mock_commands::compile_mock(&state, input)
}

#[tauri::command]
pub fn start_mock(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockSessionDto, CommandError> {
    mock_commands::start_mock(&state, mock_session_id)
}

#[tauri::command]
pub fn submit_mock_answer(
    state: State<'_, AppState>,
    input: SubmitMockAnswerInput,
) -> Result<MockAnswerResultDto, CommandError> {
    mock_commands::submit_mock_answer(&state, input)
}

#[tauri::command]
pub fn get_mock_report(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockReportDto, CommandError> {
    mock_commands::get_mock_report(&state, mock_session_id)
}

#[tauri::command]
pub fn pause_mock(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockSessionDto, CommandError> {
    mock_commands::pause_mock(&state, mock_session_id)
}

#[tauri::command]
pub fn resume_mock(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockSessionDto, CommandError> {
    mock_commands::resume_mock(&state, mock_session_id)
}

#[tauri::command]
pub fn list_mock_sessions(
    state: State<'_, AppState>,
    student_id: i64,
    limit: usize,
) -> Result<Vec<MockSessionSummaryDto>, CommandError> {
    mock_commands::list_mock_sessions(&state, student_id, limit)
}

#[tauri::command]
pub fn abandon_mock(state: State<'_, AppState>, mock_session_id: i64) -> Result<(), CommandError> {
    mock_commands::abandon_mock(&state, mock_session_id)
}

#[tauri::command]
pub fn start_first_mock(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<MockSessionDto, CommandError> {
    mock_commands::start_first_mock(&state, student_id, subject_id)
}

#[tauri::command]
pub fn flag_mock_question(
    state: State<'_, AppState>,
    mock_session_id: i64,
    question_id: i64,
) -> Result<(), CommandError> {
    mock_commands::flag_mock_question(&state, mock_session_id, question_id)
}

#[tauri::command]
pub fn get_mock_question_review(
    state: State<'_, AppState>,
    mock_session_id: i64,
    question_id: i64,
) -> Result<MockQuestionReviewDto, CommandError> {
    mock_commands::get_mock_question_review(&state, mock_session_id, question_id)
}

#[tauri::command]
pub fn get_mock_centre_snapshot(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
) -> Result<MockCentreSnapshotDto, CommandError> {
    mock_commands::get_mock_centre_snapshot(&state, student_id, subject_id)
}

#[tauri::command]
pub fn get_deep_diagnosis(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockDeepDiagnosisDto, CommandError> {
    mock_commands::get_deep_diagnosis(&state, mock_session_id)
}
