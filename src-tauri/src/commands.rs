use ecoach_commands::{
    coach_commands, content_commands, curriculum_commands, identity_commands, session_commands,
    student_commands,
    dtos::*,
    coach_commands::{
        CoachNextActionDto, CoachStateDto, ContentReadinessDto, StudentDashboardDto, TopicCaseDto,
    },
    curriculum_commands::{SubjectDto, TopicDto},
    student_commands::LearnerTruthDto,
    AppState, CommandError,
};
use ecoach_identity::CreateAccountInput;
use ecoach_sessions::{CustomTestStartInput, MockBlueprintInput, PracticeSessionStartInput};
use tauri::State;

// ── Identity ─────────────────────────────────────────────

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

// ── Coach ────────────────────────────────────────────────

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

// ── Curriculum ───────────────────────────────────────────

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

// ── Student Model ────────────────────────────────────────

#[tauri::command]
pub fn get_learner_truth(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<LearnerTruthDto, CommandError> {
    student_commands::get_learner_truth(&state, student_id)
}

// ── Content / Packs ──────────────────────────────────────

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

// ── Sessions ─────────────────────────────────────────────

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
