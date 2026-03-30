use ecoach_commands::assessment_commands::{
    EliteSessionBlueprintDto, PastPaperComebackSignalDto, PastPaperInverseSignalDto,
    SessionEvidenceFabricDto, SessionRemediationPlanDto,
};
use ecoach_commands::attempt_commands::{
    AttemptResultDto, SessionCompletionResultDto, SubmitAttemptInput,
};
use ecoach_commands::coach_commands::{
    CoachNextActionDto, CoachStateDto, ContentReadinessDto, JourneyRouteSnapshotDto,
    StudentDashboardDto, TopicCaseDto,
};
use ecoach_commands::curriculum_commands::{SubjectDto, TopicDto};
use ecoach_commands::diagnostic_commands::{
    DiagnosticBatteryDto, DiagnosticCompletionSyncDto, DiagnosticPhaseItemDto,
    DiagnosticPhasePlanDto, DiagnosticRunDto, SubmitDiagnosticAttemptInput, TopicAnalyticsDto,
};
use ecoach_commands::game_commands::{
    GameAnswerResultDto, GameSessionDto, GameSummaryDto, LeaderboardEntryDto, MindstackStateDto,
    TugOfWarStateDto,
};
use ecoach_commands::library_commands::{
    ContinueLearningCardDto, GlossaryAudioProgramDto, GlossaryEntryDto, KnowledgeBundleDto,
    LibraryHomeSnapshotDto, LibraryShelfDto, QuestionKnowledgeLinkDto, RevisionPackItemDto,
    RevisionPackSummaryDto, TeachActionPlanDto, TopicRelationshipHintDto,
};
use ecoach_commands::memory_commands::{
    DecayBatchResultDto, MemoryDashboardDto, MemoryReturnLoopDto, MemoryReviewQueueItemDto,
    MemoryStateDto, RecheckItemDto, TopicMemorySummaryDto,
};
use ecoach_commands::mock_commands::{
    MockAnswerResultDto, MockReportDto, MockSessionDto, MockSessionSummaryDto,
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
use ecoach_commands::student_commands::LearnerTruthDto;
use ecoach_commands::{
    assessment_commands, attempt_commands, coach_commands, content_commands, curriculum_commands,
    diagnostic_commands, dtos::*, game_commands, identity_commands, intake_commands,
    library_commands, memory_commands, mock_commands, premium_commands, question_commands,
    readiness_commands, repair_commands, reporting_commands, session_commands, student_commands,
    traps_commands, AppState, CommandError,
};
use ecoach_content::{ParseCandidateInput, SourceUploadInput};
use ecoach_games::{
    StartGameInput, StartTrapsSessionInput, SubmitGameAnswerInput, SubmitTrapConfusionReasonInput,
    SubmitTrapRoundInput,
};
use ecoach_identity::CreateAccountInput;
use ecoach_memory::RecordMemoryEvidenceInput;
use ecoach_mock_centre::{CompileMockInput, SubmitMockAnswerInput};
use ecoach_premium::CreateInterventionInput;
use ecoach_questions::{QuestionGenerationRequestInput, QuestionSlotSpec};
use ecoach_sessions::{CustomTestStartInput, MockBlueprintInput, PracticeSessionStartInput};
use serde_json::Value;
use tauri::State;

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
pub fn search_glossary(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError> {
    library_commands::search_glossary(&state, query)
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
