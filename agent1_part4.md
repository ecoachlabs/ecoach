# eCoach — Backend Implementation Plan: Parts 6, 7, and 8
## Agent 1 Output | Tauri Commands, Phased Delivery, Master Reference

---

# PART 6: ALL TAURI COMMANDS — FULL API SURFACE

## Shared Error Type

All command modules return `Result<T, CommandError>`. The error type is defined once in `ecoach-commands/src/errors.rs` and re-exported by every command module.

```rust
// ecoach-commands/src/errors.rs

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    NotFound(String),
    Unauthorized(String),
    ValidationError(String),
    DatabaseError(String),
    BusinessRuleViolation(String),
    InternalError(String),
}

impl From<IdentityError> for CommandError { ... }
impl From<StudentModelError> for CommandError { ... }
impl From<CoachBrainError> for CommandError { ... }
// one From impl per domain error type
```

## Command Pattern (Standard)

Every command follows this shape:

```rust
#[tauri::command]
pub async fn command_name(
    state: tauri::State<'_, AppState>,
    input: InputDto,
) -> Result<OutputDto, CommandError> {
    let service = state.service_name();
    let result = service.method(input.into()).await
        .map_err(CommandError::from)?;
    Ok(result.into())
}
```

`AppState` holds `Arc`-wrapped service handles. All services are injected at startup in `src-tauri/src/main.rs`.

---

## 6.1 identity_commands.rs

**File:** `ecoach-commands/src/identity_commands.rs`
**Owns:** Account lifecycle, PIN auth, parent-student linking

### Input / Output DTOs

```rust
#[derive(Deserialize)]
pub struct CreateAccountInput {
    pub account_type: String,       // "student" | "parent" | "admin"
    pub display_name: String,
    pub pin: String,                // raw PIN, hashed in service
    pub avatar_path: Option<String>,
    pub entitlement_tier: Option<String>,
}

#[derive(Serialize)]
pub struct AccountDto {
    pub id: i64,
    pub account_type: String,
    pub display_name: String,
    pub avatar_path: Option<String>,
    pub entitlement_tier: String,
    pub status: String,
    pub first_run: bool,
    pub created_at: String,
    pub last_active_at: Option<String>,
}

#[derive(Serialize)]
pub struct SessionDto {
    pub account_id: i64,
    pub display_name: String,
    pub account_type: String,
    pub entitlement_tier: String,
    pub session_token: String,      // short-lived in-memory token
    pub authenticated_at: String,
}

#[derive(Serialize)]
pub struct AccountSummaryDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub avatar_path: Option<String>,
    pub entitlement_tier: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub avatar_path: Option<String>,
    pub grade_level: Option<String>,
    pub exam_target: Option<String>,
    pub exam_target_date: Option<String>,
    pub daily_study_budget_minutes: Option<i32>,
    pub study_days_per_week: Option<i32>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    input: CreateAccountInput,
) -> Result<AccountDto, CommandError>

#[tauri::command]
pub async fn login_with_pin(
    state: State<'_, AppState>,
    account_id: i64,
    pin: String,
) -> Result<SessionDto, CommandError>

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummaryDto>, CommandError>

#[tauri::command]
pub async fn switch_account(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDto, CommandError>

#[tauri::command]
pub async fn link_parent_student(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn reset_student_pin(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
    new_pin: String,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_linked_students(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<Vec<AccountSummaryDto>, CommandError>

#[tauri::command]
pub async fn update_account_profile(
    state: State<'_, AppState>,
    account_id: i64,
    input: UpdateProfileInput,
) -> Result<AccountDto, CommandError>
```

**Service delegation:** `IdentityService` in `ecoach-identity`

---

## 6.2 curriculum_commands.rs

**File:** `ecoach-commands/src/curriculum_commands.rs`
**Owns:** Read-only curriculum graph queries; no writes (curriculum is pack-installed)

### DTOs

```rust
#[derive(Serialize)]
pub struct SubjectDto {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub curriculum_version_id: i64,
    pub topic_count: i32,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct TopicDto {
    pub id: i64,
    pub subject_id: i64,
    pub name: String,
    pub node_id: String,        // e.g., "B7/JHS1.1.1"
    pub depth_level: i32,
    pub parent_topic_id: Option<i64>,
    pub exam_weight: i32,       // basis points
    pub prerequisite_ids: Vec<i64>,
    pub order_index: i32,
}

#[derive(Serialize)]
pub struct TopicTreeDto {
    pub subject_id: i64,
    pub nodes: Vec<TopicTreeNode>,
}

#[derive(Serialize)]
pub struct TopicTreeNode {
    pub id: i64,
    pub name: String,
    pub node_id: String,
    pub depth_level: i32,
    pub children: Vec<TopicTreeNode>,
    pub skill_atom_count: i32,
}

#[derive(Serialize)]
pub struct SkillAtomDto {
    pub id: i64,
    pub topic_id: i64,
    pub name: String,
    pub atom_type: String,
    pub cognitive_verb: String,
    pub difficulty_band: String,
    pub prerequisite_atom_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CurriculumVersionDto {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub exam_board: String,
    pub effective_year: String,
    pub is_active: bool,
    pub subject_count: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_subjects(
    state: State<'_, AppState>,
    curriculum_version_id: i64,
) -> Result<Vec<SubjectDto>, CommandError>

#[tauri::command]
pub async fn get_topics(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Vec<TopicDto>, CommandError>

#[tauri::command]
pub async fn get_topic_tree(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<TopicTreeDto, CommandError>

#[tauri::command]
pub async fn get_skill_atoms(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Vec<SkillAtomDto>, CommandError>

#[tauri::command]
pub async fn get_curriculum_versions(
    state: State<'_, AppState>,
) -> Result<Vec<CurriculumVersionDto>, CommandError>
```

**Service delegation:** `CurriculumService` in `ecoach-curriculum`

---

## 6.3 session_commands.rs

**File:** `ecoach-commands/src/session_commands.rs`
**Owns:** Session lifecycle, attempt submission, session history

### DTOs

```rust
#[derive(Deserialize)]
pub struct StartSessionInput {
    pub account_id: i64,
    pub session_type: String,       // "practice"|"diagnostic"|"mock"|"gap_repair"|"memory_review"|"coach_mission"|"elite"|"game"|"traps"
    pub subject_id: Option<i64>,
    pub topic_ids: Option<Vec<i64>>,
    pub question_count: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub is_timed: Option<bool>,
    pub difficulty_preference: Option<String>,
}

#[derive(Serialize)]
pub struct SessionDto {
    pub id: i64,
    pub account_id: i64,
    pub session_type: String,
    pub status: String,
    pub question_count: i32,
    pub first_question: Option<QuestionDto>,
    pub started_at: String,
    pub duration_minutes: Option<i32>,
    pub is_timed: bool,
}

#[derive(Deserialize)]
pub struct SubmitAttemptInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub open_response_text: Option<String>,
    pub response_time_ms: i64,
    pub confidence_level: Option<String>,   // "sure"|"not_sure"|"guessed"
    pub hint_count: i32,
}

#[derive(Serialize)]
pub struct AttemptResultDto {
    pub attempt_id: i64,
    pub is_correct: bool,
    pub correct_option_id: Option<i64>,
    pub correct_option_text: String,
    pub explanation: Option<String>,
    pub error_type: Option<String>,
    pub misconception_info: Option<String>,
    pub updated_mastery_bp: i32,
    pub updated_gap_bp: i32,
    pub next_question: Option<QuestionDto>,
    pub session_complete: bool,
}

#[derive(Serialize)]
pub struct SessionSummaryDto {
    pub id: i64,
    pub session_type: String,
    pub total_questions: i32,
    pub correct_questions: i32,
    pub accuracy_bp: i32,
    pub avg_response_time_ms: i64,
    pub duration_seconds: i64,
    pub completed_at: String,
    pub topic_results: Vec<TopicResultDto>,
    pub coach_message: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionEventInput {
    pub event_type: String,     // "paused"|"resumed"|"hint_requested"|"question_skipped"
    pub payload_json: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_session(
    state: State<'_, AppState>,
    input: StartSessionInput,
) -> Result<SessionDto, CommandError>

#[tauri::command]
pub async fn submit_attempt(
    state: State<'_, AppState>,
    session_id: i64,
    input: SubmitAttemptInput,
) -> Result<AttemptResultDto, CommandError>

#[tauri::command]
pub async fn end_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<SessionSummaryDto, CommandError>

#[tauri::command]
pub async fn get_session_history(
    state: State<'_, AppState>,
    account_id: i64,
    limit: u32,
) -> Result<Vec<SessionSummaryDto>, CommandError>

#[tauri::command]
pub async fn record_session_event(
    state: State<'_, AppState>,
    session_id: i64,
    input: SessionEventInput,
) -> Result<(), CommandError>
```

**Service delegation:** `SessionOrchestrator` in `ecoach-sessions`; calls `StudentModelService.process_answer()` on each attempt

---

## 6.4 student_commands.rs

**File:** `ecoach-commands/src/student_commands.rs`
**Owns:** Student state reads — mastery, readiness, momentum

### DTOs

```rust
#[derive(Serialize)]
pub struct StudentStateDto {
    pub account_id: i64,
    pub overall_readiness_bp: i32,
    pub overall_mastery_bp: i32,
    pub streak_days: i32,
    pub total_sessions: i32,
    pub total_study_minutes: i64,
    pub exam_target: Option<String>,
    pub exam_date: Option<String>,
    pub days_to_exam: Option<i32>,
    pub coach_lifecycle_state: String,
    pub momentum_direction: String,
}

#[derive(Serialize)]
pub struct MasteryMapDto {
    pub account_id: i64,
    pub subjects: Vec<SubjectMasteryDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct SubjectMasteryDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub mastery_bp: i32,
    pub gap_bp: i32,
    pub coverage_bp: i32,
    pub topics: Vec<TopicMasteryDto>,
}

#[derive(Serialize)]
pub struct TopicMasteryDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_state: String,
    pub mastery_bp: i32,
    pub gap_bp: i32,
    pub priority_bp: i32,
    pub accuracy_bp: i32,
    pub speed_bp: i32,
    pub retention_bp: i32,
    pub trend_direction: String,
    pub misconception_count: i32,
    pub last_seen_at: Option<String>,
}

#[derive(Serialize)]
pub struct ReadinessReportDto {
    pub account_id: i64,
    pub overall_readiness_bp: i32,
    pub predicted_exam_score_bp: i32,
    pub readiness_band: String,         // "not_ready"|"approaching"|"ready"|"strong"
    pub mastery_component_bp: i32,
    pub timed_performance_bp: i32,
    pub coverage_bp: i32,
    pub consistency_bp: i32,
    pub trend_bp: i32,
    pub critical_gaps: Vec<GapSummaryDto>,
    pub strongest_topics: Vec<String>,
    pub weakest_topics: Vec<String>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct MomentumDto {
    pub account_id: i64,
    pub momentum_score_bp: i32,
    pub direction: String,
    pub volume_bp: i32,
    pub accuracy_bp: i32,
    pub pace_bp: i32,
    pub streak_days: i32,
    pub sessions_this_week: i32,
    pub last_active_at: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_student_state(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<StudentStateDto, CommandError>

#[tauri::command]
pub async fn get_mastery_map(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MasteryMapDto, CommandError>

#[tauri::command]
pub async fn get_topic_mastery(
    state: State<'_, AppState>,
    account_id: i64,
    topic_id: i64,
) -> Result<TopicMasteryDto, CommandError>

#[tauri::command]
pub async fn get_readiness_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ReadinessReportDto, CommandError>

#[tauri::command]
pub async fn get_momentum(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MomentumDto, CommandError>
```

**Service delegation:** `StudentModelService` in `ecoach-student-model`

---

## 6.5 coach_commands.rs

**File:** `ecoach-commands/src/coach_commands.rs`
**Owns:** Coach brain state, next action, missions, interventions

### DTOs

```rust
#[derive(Serialize)]
pub struct CoachStateDto {
    pub account_id: i64,
    pub lifecycle_state: String,            // CoachLifecycleState variant
    pub content_readiness_status: String,
    pub next_action: String,                // NextCoachAction variant
    pub active_mission_id: Option<i64>,
    pub intervention_count: i32,
    pub plan_exists: bool,
    pub plan_adherence_bp: i32,
    pub last_updated_at: String,
}

#[derive(Serialize)]
pub struct NextActionDto {
    pub action_type: String,
    pub title: String,
    pub description: String,
    pub cta_label: String,
    pub urgency: String,
    pub payload_json: Option<String>,
}

#[derive(Serialize)]
pub struct MissionDto {
    pub id: i64,
    pub account_id: i64,
    pub mission_type: String,
    pub title: String,
    pub description: String,
    pub subject_id: Option<i64>,
    pub topic_ids: Vec<i64>,
    pub estimated_minutes: i32,
    pub status: String,
    pub items: Vec<MissionItemDto>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct MissionItemDto {
    pub id: i64,
    pub item_type: String,
    pub description: String,
    pub is_completed: bool,
    pub order_index: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_coach_state(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<CoachStateDto, CommandError>

#[tauri::command]
pub async fn get_next_action(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<NextActionDto, CommandError>

#[tauri::command]
pub async fn get_current_mission(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MissionDto, CommandError>

#[tauri::command]
pub async fn dismiss_intervention(
    state: State<'_, AppState>,
    intervention_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn acknowledge_coach_event(
    state: State<'_, AppState>,
    event_id: i64,
) -> Result<(), CommandError>
```

**Service delegation:** `CoachBrainService` in `ecoach-coach-brain`

---

## 6.6 diagnostic_commands.rs

**File:** `ecoach-commands/src/diagnostic_commands.rs`
**Owns:** Academic DNA test, problem cards, diagnostic report

### DTOs

```rust
#[derive(Deserialize)]
pub struct DiagnosticInput {
    pub mode: String,                   // "light"|"standard"|"deep"
    pub subject_ids: Vec<i64>,
    pub include_timed_layer: bool,
    pub include_transfer_layer: bool,
}

#[derive(Serialize)]
pub struct DiagnosticSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub mode: String,
    pub total_estimated_items: i32,
    pub estimated_minutes: i32,
    pub current_stage: String,
    pub first_question: QuestionDto,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct DiagnosticResponseInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: String,       // "sure"|"not_sure"|"guessed"
    pub answer_changed: bool,
    pub change_count: i32,
    pub hint_used: bool,
}

#[derive(Serialize)]
pub struct DiagnosticResultDto {
    pub is_correct: bool,
    pub next_question: Option<QuestionDto>,
    pub stage_transition: Option<String>,
    pub diagnostic_complete: bool,
    pub items_remaining: i32,
}

#[derive(Serialize)]
pub struct DiagnosticReportDto {
    pub account_id: i64,
    pub generated_at: String,
    pub overall_profile: String,
    pub dimensions: DiagnosticDimensionsDto,
    pub subject_reports: Vec<SubjectDiagnosticDto>,
    pub problem_cards: Vec<ProblemCardDto>,
    pub recommended_focus_topics: Vec<i64>,
    pub archetype: String,
}

#[derive(Serialize)]
pub struct DiagnosticDimensionsDto {
    pub coverage_bp: i32,
    pub accuracy_bp: i32,
    pub recall_strength_bp: i32,
    pub reasoning_depth_bp: i32,
    pub misconception_density_bp: i32,
    pub speed_bp: i32,
    pub pressure_response_bp: i32,
    pub transfer_ability_bp: i32,
    pub stability_bp: i32,
    pub confidence_calibration_bp: i32,
    pub fatigue_pattern_bp: i32,
}

#[derive(Serialize)]
pub struct ProblemCardDto {
    pub id: i64,
    pub account_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub problem_type: String,
    pub severity: String,
    pub description: String,
    pub evidence_summary: String,
    pub recommended_intervention: String,
    pub created_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_diagnostic(
    state: State<'_, AppState>,
    account_id: i64,
    input: DiagnosticInput,
) -> Result<DiagnosticSessionDto, CommandError>

#[tauri::command]
pub async fn submit_diagnostic_response(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    input: DiagnosticResponseInput,
) -> Result<DiagnosticResultDto, CommandError>

#[tauri::command]
pub async fn get_diagnostic_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<DiagnosticReportDto, CommandError>

#[tauri::command]
pub async fn get_problem_cards(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<ProblemCardDto>, CommandError>
```

**Service delegation:** `DiagnosticBatteryService` in `ecoach-diagnostics`

---

## 6.7 mock_commands.rs

**File:** `ecoach-commands/src/mock_commands.rs`
**Owns:** Mock compilation, runtime, post-mock analysis, forecast

### DTOs

```rust
#[derive(Deserialize)]
pub struct CompileMockInput {
    pub mock_type: String,      // "forecast"|"diagnostic"|"remediation"|"final_exam"|"shock"|"wisdom"
    pub subject_ids: Vec<i64>,
    pub question_count_per_subject: i32,
    pub duration_minutes: i32,
    pub use_forecast_weights: bool,
}

#[derive(Serialize)]
pub struct MockInstanceDto {
    pub id: i64,
    pub account_id: i64,
    pub mock_type: String,
    pub total_questions: i32,
    pub duration_minutes: i32,
    pub subjects_included: Vec<String>,
    pub forecast_coverage_bp: i32,
    pub compiled_at: String,
}

#[derive(Serialize)]
pub struct MockSessionDto {
    pub id: i64,
    pub mock_instance_id: i64,
    pub status: String,
    pub first_question: QuestionDto,
    pub total_questions: i32,
    pub time_limit_seconds: i64,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct MockAttemptInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: Option<String>,
}

#[derive(Serialize)]
pub struct MockAttemptResultDto {
    pub next_question: Option<QuestionDto>,
    pub mock_complete: bool,
    pub questions_remaining: i32,
    pub time_remaining_seconds: i64,
}

#[derive(Serialize)]
pub struct MockAnalysisDto {
    pub mock_instance_id: i64,
    pub overall_score_bp: i32,
    pub predicted_bece_band: String,
    pub readiness_movement_bp: i32,
    // Section 1 — Overall summary
    pub confidence_level: String,
    pub timing_assessment: String,
    // Section 2 — Subject/topic performance
    pub subject_results: Vec<MockSubjectResultDto>,
    // Section 3 — Link-level diagnosis
    pub broken_concept_links: Vec<String>,
    pub prerequisite_failures: Vec<String>,
    // Section 4 — Misconception diagnosis
    pub misconceptions_detected: Vec<MockMisconceptionDto>,
    // Section 5 — Representation diagnosis
    pub representation_breakdown: MockRepresentationDto,
    // Section 6 — Timing diagnosis
    pub timing_breakdown: MockTimingDto,
    // Section 7 — Confidence diagnosis
    pub confidence_breakdown: MockConfidenceDto,
    // Section 8 — Action plan
    pub repair_now: Vec<String>,
    pub drill_topics: Vec<String>,
    pub review_topics: Vec<String>,
    pub next_mock_recommended_date: Option<String>,
}

#[derive(Serialize)]
pub struct MockSummaryDto {
    pub mock_instance_id: i64,
    pub mock_type: String,
    pub score_bp: i32,
    pub completed_at: String,
    pub question_count: i32,
    pub accuracy_bp: i32,
}

#[derive(Serialize)]
pub struct ForecastReportDto {
    pub account_id: i64,
    pub high_probability_topics: Vec<ForecastTopicDto>,
    pub medium_probability_topics: Vec<ForecastTopicDto>,
    pub surprise_risk_topics: Vec<ForecastTopicDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct ForecastTopicDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub forecast_score_bp: i32,
    pub probability_band: String,
    pub frequency_score_bp: i32,
    pub recency_score_bp: i32,
    pub student_mastery_bp: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn compile_mock(
    state: State<'_, AppState>,
    account_id: i64,
    input: CompileMockInput,
) -> Result<MockInstanceDto, CommandError>

#[tauri::command]
pub async fn start_mock(
    state: State<'_, AppState>,
    mock_instance_id: i64,
) -> Result<MockSessionDto, CommandError>

#[tauri::command]
pub async fn submit_mock_attempt(
    state: State<'_, AppState>,
    mock_session_id: i64,
    input: MockAttemptInput,
) -> Result<MockAttemptResultDto, CommandError>

#[tauri::command]
pub async fn end_mock(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockAnalysisDto, CommandError>

#[tauri::command]
pub async fn get_mock_history(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MockSummaryDto>, CommandError>

#[tauri::command]
pub async fn get_mock_analysis(
    state: State<'_, AppState>,
    mock_instance_id: i64,
) -> Result<MockAnalysisDto, CommandError>

#[tauri::command]
pub async fn get_forecast_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ForecastReportDto, CommandError>
```

**Service delegation:** `MockCentreService` in `ecoach-mock-centre`

---

## 6.8 gap_commands.rs

**File:** `ecoach-commands/src/gap_commands.rs`
**Owns:** Gap map, gap priority, gap repair sessions

### DTOs

```rust
#[derive(Serialize)]
pub struct GapMapDto {
    pub account_id: i64,
    pub coverage_bp: i32,
    pub total_gap_bp: i32,
    pub critical_gap_count: i32,
    pub hidden_gap_count: i32,
    pub fixed_this_month: i32,
    pub subjects: Vec<SubjectGapDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct SubjectGapDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub gap_bp: i32,
    pub critical_gaps: Vec<GapDto>,
}

#[derive(Serialize)]
pub struct GapDto {
    pub id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub gap_type: String,       // KnowledgeGapType variant
    pub severity_bp: i32,
    pub rank: i32,
    pub error_types: Vec<String>,
    pub misconception_tags: Vec<String>,
    pub last_seen_at: Option<String>,
    pub attempts: i32,
    pub correct_rate_bp: i32,
}

#[derive(Serialize)]
pub struct GapRepairSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub gap_id: i64,
    pub topic_name: String,
    pub gap_type: String,
    pub repair_strategy: String,
    pub first_question: QuestionDto,
    pub total_planned_items: i32,
    pub started_at: String,
}

#[derive(Serialize)]
pub struct RepairResultDto {
    pub is_correct: bool,
    pub explanation: String,
    pub repair_stage: String,
    pub next_question: Option<QuestionDto>,
    pub session_complete: bool,
    pub mastery_delta_bp: i32,
}

#[derive(Serialize)]
pub struct GapRepairSummaryDto {
    pub repair_session_id: i64,
    pub gap_id: i64,
    pub items_completed: i32,
    pub accuracy_bp: i32,
    pub mastery_change_bp: i32,
    pub gap_status_after: String,
    pub next_repair_recommended_at: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_gap_map(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<GapMapDto, CommandError>

#[tauri::command]
pub async fn get_gap_priority_list(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<GapDto>, CommandError>

#[tauri::command]
pub async fn start_gap_repair(
    state: State<'_, AppState>,
    account_id: i64,
    gap_id: i64,
) -> Result<GapRepairSessionDto, CommandError>

#[tauri::command]
pub async fn submit_gap_repair_attempt(
    state: State<'_, AppState>,
    repair_session_id: i64,
    input: SubmitAttemptInput,
) -> Result<RepairResultDto, CommandError>

#[tauri::command]
pub async fn close_gap_repair(
    state: State<'_, AppState>,
    repair_session_id: i64,
) -> Result<GapRepairSummaryDto, CommandError>
```

**Service delegation:** `KnowledgeGapService` in `ecoach-knowledge-gap`

---

## 6.9 memory_commands.rs

**File:** `ecoach-commands/src/memory_commands.rs`
**Owns:** Memory shelf, rescue queue, recall drills, memory health

### DTOs

```rust
#[derive(Serialize)]
pub struct MemoryShelfDto {
    pub account_id: i64,
    pub total_tracked: i32,
    pub locked_in_count: i32,
    pub stable_count: i32,
    pub vulnerable_count: i32,
    pub fading_count: i32,
    pub critical_count: i32,
    pub items: Vec<MemoryItemDto>,
}

#[derive(Serialize)]
pub struct MemoryItemDto {
    pub memory_state_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub memory_state: String,               // MemoryState variant
    pub msi_score_bp: i32,                  // Memory Strength Index
    pub decay_risk_bp: i32,
    pub last_recall_at: Option<String>,
    pub next_review_at: Option<String>,
    pub recall_success_rate_bp: i32,
    pub hint_dependency_bp: i32,
}

#[derive(Serialize)]
pub struct MemoryRescueItemDto {
    pub memory_state_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub urgency: String,
    pub decay_severity: String,
    pub recommended_session_type: String,
    pub estimated_minutes: i32,
}

#[derive(Deserialize)]
pub struct RecallInput {
    pub response_text: Option<String>,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: String,
}

#[derive(Serialize)]
pub struct RecallResultDto {
    pub is_correct: bool,
    pub new_memory_state: String,
    pub msi_delta_bp: i32,
    pub next_review_at: String,
    pub recovery_stage: String,
    pub encouragement: Option<String>,
}

#[derive(Serialize)]
pub struct MemoryHealthDto {
    pub account_id: i64,
    pub overall_health_bp: i32,
    pub at_risk_count: i32,
    pub collapsed_count: i32,
    pub stable_or_above_count: i32,
    pub upcoming_reviews: Vec<MemoryRescueItemDto>,
    pub generated_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_memory_shelf(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MemoryShelfDto, CommandError>

#[tauri::command]
pub async fn get_rescue_queue(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MemoryRescueItemDto>, CommandError>

#[tauri::command]
pub async fn submit_recall_attempt(
    state: State<'_, AppState>,
    memory_state_id: i64,
    input: RecallInput,
) -> Result<RecallResultDto, CommandError>

#[tauri::command]
pub async fn get_memory_health(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MemoryHealthDto, CommandError>
```

**Service delegation:** `MemoryEngine` in `ecoach-memory`

---

## 6.10 goals_commands.rs

**File:** `ecoach-commands/src/goals_commands.rs`
**Owns:** Exam goals, weekly plans, daily schedules, readiness timeline

### DTOs

```rust
#[derive(Deserialize)]
pub struct ExamGoalInput {
    pub exam_target: String,            // e.g., "BECE 2026"
    pub exam_date: String,              // ISO date
    pub subject_ids: Vec<i64>,
    pub target_grade: Option<String>,
    pub daily_study_budget_minutes: i32,
    pub study_days_per_week: i32,
    pub path_intensity: String,         // "relaxed"|"balanced"|"intense"
}

#[derive(Serialize)]
pub struct ExamGoalDto {
    pub id: i64,
    pub account_id: i64,
    pub exam_target: String,
    pub exam_date: String,
    pub days_remaining: i32,
    pub subject_ids: Vec<i64>,
    pub target_grade: Option<String>,
    pub daily_budget_minutes: i32,
    pub study_days_per_week: i32,
    pub state: String,
    pub confidence_bp: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct WeeklyPlanDto {
    pub account_id: i64,
    pub week_start: String,
    pub week_end: String,
    pub total_planned_minutes: i32,
    pub days: Vec<DayPlanDto>,
    pub focus_topics: Vec<String>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct DayPlanDto {
    pub date: String,
    pub subject_focus: Option<String>,
    pub planned_minutes: i32,
    pub session_types: Vec<String>,
    pub target_outcomes: Vec<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct DailyScheduleDto {
    pub date: String,
    pub account_id: i64,
    pub available_minutes: i32,
    pub sessions: Vec<ScheduledSessionDto>,
    pub total_planned_minutes: i32,
    pub coach_note: Option<String>,
}

#[derive(Serialize)]
pub struct ScheduledSessionDto {
    pub session_type: String,
    pub subject_name: String,
    pub topic_names: Vec<String>,
    pub estimated_minutes: i32,
    pub priority: String,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct UpdateGoalInput {
    pub state: Option<String>,
    pub exam_date: Option<String>,
    pub daily_budget_minutes: Option<i32>,
    pub path_intensity: Option<String>,
}

#[derive(Serialize)]
pub struct ReadinessTimelineDto {
    pub account_id: i64,
    pub current_readiness_bp: i32,
    pub projected_exam_readiness_bp: i32,
    pub projected_date_to_ready: Option<String>,
    pub exam_date: Option<String>,
    pub milestones: Vec<ReadinessMilestoneDto>,
    pub on_track: bool,
    pub risk_level: String,
}

#[derive(Serialize)]
pub struct ReadinessMilestoneDto {
    pub label: String,
    pub target_date: String,
    pub target_readiness_bp: i32,
    pub achieved: bool,
}
```

### Commands

```rust
#[tauri::command]
pub async fn set_exam_goal(
    state: State<'_, AppState>,
    account_id: i64,
    input: ExamGoalInput,
) -> Result<ExamGoalDto, CommandError>

#[tauri::command]
pub async fn get_weekly_plan(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<WeeklyPlanDto, CommandError>

#[tauri::command]
pub async fn get_daily_schedule(
    state: State<'_, AppState>,
    account_id: i64,
    date: String,
) -> Result<DailyScheduleDto, CommandError>

#[tauri::command]
pub async fn update_goal(
    state: State<'_, AppState>,
    goal_id: i64,
    input: UpdateGoalInput,
) -> Result<ExamGoalDto, CommandError>

#[tauri::command]
pub async fn get_readiness_timeline(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ReadinessTimelineDto, CommandError>
```

**Service delegation:** `GoalsCalendarService` in `ecoach-goals-calendar`

---

## 6.11 reporting_commands.rs

**File:** `ecoach-commands/src/reporting_commands.rs`
**Owns:** Parent dashboard, child reports, alerts, digests, printable reports

### DTOs

```rust
#[derive(Serialize)]
pub struct ParentDashboardDto {
    pub parent_account_id: i64,
    pub children: Vec<ChildOverviewDto>,
    pub total_alerts: i32,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct ChildOverviewDto {
    pub student_id: i64,
    pub display_name: String,
    pub academic_status: String,
    pub recent_activity_at: Option<String>,
    pub strongest_subject: Option<String>,
    pub weakest_subject: Option<String>,
    pub overall_readiness_bp: i32,
    pub streak_days: i32,
    pub unread_alerts: i32,
    pub trend_direction: String,
}

#[derive(Serialize)]
pub struct ChildReportDto {
    pub student_id: i64,
    pub display_name: String,
    pub performance_summary: PerformanceSummaryDto,
    pub activity_history: ActivityHistoryDto,
    pub attention_needed: Vec<ParentAlertDto>,
    pub subject_breakdown: Vec<SubjectMasteryDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct PerformanceSummaryDto {
    pub avg_score_trend_bp: i32,
    pub improvement_direction: String,
    pub test_frequency_per_week: f32,
    pub mock_count: i32,
    pub readiness_level: String,
    pub sessions_this_week: i32,
    pub total_study_minutes: i64,
}

#[derive(Serialize)]
pub struct ActivityHistoryDto {
    pub study_days: Vec<String>,
    pub subjects_studied: Vec<String>,
    pub tests_taken: i32,
    pub time_spent_minutes: i64,
    pub abandoned_sessions: i32,
    pub milestones_reached: Vec<String>,
}

#[derive(Serialize)]
pub struct ParentAlertDto {
    pub id: i64,
    pub alert_type: String,
    pub student_name: String,
    pub message: String,
    pub severity: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct WeeklyDigestDto {
    pub account_id: i64,
    pub week_start: String,
    pub week_end: String,
    pub sessions_completed: i32,
    pub study_minutes: i64,
    pub accuracy_this_week_bp: i32,
    pub topics_covered: Vec<String>,
    pub mastery_gains: Vec<MasteryGainDto>,
    pub top_achievement: Option<String>,
    pub areas_to_improve: Vec<String>,
}

#[derive(Serialize)]
pub struct MasteryGainDto {
    pub topic_name: String,
    pub old_state: String,
    pub new_state: String,
}

#[derive(Serialize)]
pub struct PrintableReportDto {
    pub student_id: i64,
    pub display_name: String,
    pub report_date: String,
    pub sections_json: String,      // JSON blob for PDF rendering
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_parent_dashboard(
    state: State<'_, AppState>,
    parent_account_id: i64,
) -> Result<ParentDashboardDto, CommandError>

#[tauri::command]
pub async fn get_parent_child_report(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
) -> Result<ChildReportDto, CommandError>

#[tauri::command]
pub async fn get_parent_alerts(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<Vec<ParentAlertDto>, CommandError>

#[tauri::command]
pub async fn mark_alert_read(
    state: State<'_, AppState>,
    alert_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_weekly_digest(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<WeeklyDigestDto, CommandError>

#[tauri::command]
pub async fn generate_printable_report(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<PrintableReportDto, CommandError>
```

**Service delegation:** `ReportingService` in `ecoach-reporting`

---

## 6.12 glossary_commands.rs

**File:** `ecoach-commands/src/glossary_commands.rs`
**Owns:** Glossary search and retrieval

### DTOs

```rust
#[derive(Serialize)]
pub struct GlossaryEntryDto {
    pub id: i64,
    pub term: String,
    pub plain_explanation: String,
    pub exam_explanation: Option<String>,
    pub formal_definition: Option<String>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub examples: Vec<String>,
    pub related_term_ids: Vec<i64>,
    pub audio_path: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn search_glossary(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError>

#[tauri::command]
pub async fn get_glossary_entry(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<GlossaryEntryDto, CommandError>

#[tauri::command]
pub async fn get_related_entries(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<Vec<GlossaryEntryDto>, CommandError>

#[tauri::command]
pub async fn get_topic_glossary(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Vec<GlossaryEntryDto>, CommandError>
```

**Service delegation:** `GlossaryService` in `ecoach-glossary`

---

## 6.13 library_commands.rs

**File:** `ecoach-commands/src/library_commands.rs`
**Owns:** Personal shelves, mistake bank, revision packs

### DTOs

```rust
#[derive(Serialize)]
pub struct ShelfDto {
    pub id: i64,
    pub account_id: i64,
    pub shelf_type: String,
    pub name: String,
    pub item_count: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct LibraryItemDto {
    pub id: i64,
    pub shelf_id: i64,
    pub item_type: String,
    pub title: String,
    pub reference_id: i64,
    pub state: String,
    pub saved_at: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveItemInput {
    pub shelf_id: i64,
    pub item_type: String,
    pub reference_id: i64,
    pub title: String,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct MistakeBankEntryDto {
    pub id: i64,
    pub account_id: i64,
    pub attempt_id: i64,
    pub question_id: i64,
    pub question_text: String,
    pub error_type: String,
    pub misconception: Option<String>,
    pub saved_at: String,
    pub review_count: i32,
}

#[derive(Deserialize)]
pub struct CreateRevisionPackInput {
    pub name: String,
    pub topic_ids: Vec<i64>,
    pub include_mistakes: bool,
    pub include_weak_topics: bool,
    pub question_count: i32,
}

#[derive(Serialize)]
pub struct RevisionPackDto {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub question_count: i32,
    pub topic_names: Vec<String>,
    pub created_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_library_shelves(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<ShelfDto>, CommandError>

#[tauri::command]
pub async fn get_shelf_items(
    state: State<'_, AppState>,
    shelf_id: i64,
) -> Result<Vec<LibraryItemDto>, CommandError>

#[tauri::command]
pub async fn save_to_library(
    state: State<'_, AppState>,
    account_id: i64,
    input: SaveItemInput,
) -> Result<LibraryItemDto, CommandError>

#[tauri::command]
pub async fn add_to_mistake_bank(
    state: State<'_, AppState>,
    account_id: i64,
    attempt_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_mistake_bank(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MistakeBankEntryDto>, CommandError>

#[tauri::command]
pub async fn create_revision_pack(
    state: State<'_, AppState>,
    account_id: i64,
    input: CreateRevisionPackInput,
) -> Result<RevisionPackDto, CommandError>
```

**Service delegation:** `LibraryService` in `ecoach-library`

---

## 6.14 game_commands.rs

**File:** `ecoach-commands/src/game_commands.rs`
**Owns:** Game sessions for MindStack (Tetris) and Tug of War (MindPull)

### DTOs

```rust
#[derive(Deserialize)]
pub struct StartGameInput {
    pub game_type: String,          // "mindstack"|"tug_of_war"|"traps"
    pub subject_id: Option<i64>,
    pub topic_ids: Option<Vec<i64>>,
    pub difficulty: Option<String>,
}

#[derive(Serialize)]
pub struct GameSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub game_type: String,
    pub status: String,
    pub initial_state_json: String, // game-specific state (board, rope position, etc.)
    pub first_question: QuestionDto,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct GameActionInput {
    pub action_type: String,        // "answer"|"use_powerup"|"skip"
    pub question_id: Option<i64>,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: Option<i64>,
    pub powerup_type: Option<String>,
}

#[derive(Serialize)]
pub struct GameStateDto {
    pub game_session_id: i64,
    pub state_json: String,         // updated board/rope/score state
    pub is_correct: Option<bool>,
    pub next_question: Option<QuestionDto>,
    pub game_over: bool,
    pub score: i32,
    pub combo_count: i32,
}

#[derive(Serialize)]
pub struct GameResultDto {
    pub game_session_id: i64,
    pub game_type: String,
    pub final_score: i32,
    pub accuracy_bp: i32,
    pub max_combo: i32,
    pub topics_covered: Vec<String>,
    pub mastery_reinforced: Vec<String>,
    pub completed_at: String,
}

#[derive(Serialize)]
pub struct LeaderboardDto {
    pub game_type: String,
    pub entries: Vec<LeaderboardEntryDto>,
    pub player_rank: Option<i32>,
    pub player_best_score: Option<i32>,
}

#[derive(Serialize)]
pub struct LeaderboardEntryDto {
    pub rank: i32,
    pub display_name: String,
    pub score: i32,
    pub achieved_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_game(
    state: State<'_, AppState>,
    account_id: i64,
    input: StartGameInput,
) -> Result<GameSessionDto, CommandError>

#[tauri::command]
pub async fn submit_game_action(
    state: State<'_, AppState>,
    game_session_id: i64,
    input: GameActionInput,
) -> Result<GameStateDto, CommandError>

#[tauri::command]
pub async fn end_game(
    state: State<'_, AppState>,
    game_session_id: i64,
) -> Result<GameResultDto, CommandError>

#[tauri::command]
pub async fn get_game_leaderboard(
    state: State<'_, AppState>,
    game_type: String,
) -> Result<LeaderboardDto, CommandError>
```

**Service delegation:** `GameEngineService` in `ecoach-games`

---

## 6.15 intake_commands.rs

**File:** `ecoach-commands/src/intake_commands.rs`
**Owns:** Document upload portal, OCR bridge, candidate question review

### DTOs

```rust
#[derive(Deserialize)]
pub struct UploadDocumentInput {
    pub file_path: String,          // absolute local path
    pub document_type: String,      // "homework"|"class_notes"|"class_tests"|"assignments"|"teacher_handouts"|"revision_sheets"|"report_cards"|"exam_papers"|"textbook_snapshots"|"worksheets"
    pub subject_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct IntakeDocumentDto {
    pub id: i64,
    pub account_id: i64,
    pub document_type: String,
    pub file_name: String,
    pub status: String,             // "queued"|"processing"|"parsed"|"review_pending"|"completed"|"failed"
    pub created_at: String,
}

#[derive(Serialize)]
pub struct IntakeStatusDto {
    pub document_id: i64,
    pub status: String,
    pub progress_percent: i32,
    pub candidate_question_count: i32,
    pub approved_count: i32,
    pub rejected_count: i32,
    pub pending_count: i32,
    pub error_message: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn upload_document(
    state: State<'_, AppState>,
    account_id: i64,
    input: UploadDocumentInput,
) -> Result<IntakeDocumentDto, CommandError>

#[tauri::command]
pub async fn get_intake_status(
    state: State<'_, AppState>,
    document_id: i64,
) -> Result<IntakeStatusDto, CommandError>

#[tauri::command]
pub async fn approve_intake_question(
    state: State<'_, AppState>,
    candidate_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn reject_intake_question(
    state: State<'_, AppState>,
    candidate_id: i64,
    reason: String,
) -> Result<(), CommandError>
```

**Service delegation:** `IntakeService` in `ecoach-intake`

---

## 6.16 admin_commands.rs

**File:** `ecoach-commands/src/admin_commands.rs`
**Owns:** System status, migration checks, audit log, feature flags, content packs

### DTOs

```rust
#[derive(Serialize)]
pub struct SystemStatusDto {
    pub db_version: i32,
    pub db_size_bytes: i64,
    pub wal_mode: bool,
    pub migration_count: i32,
    pub content_pack_count: i32,
    pub account_count: i32,
    pub uptime_seconds: i64,
    pub build_version: String,
}

#[derive(Serialize)]
pub struct MigrationStatusDto {
    pub applied_count: i32,
    pub pending_count: i32,
    pub latest_applied: String,
    pub pending_migrations: Vec<String>,
    pub healthy: bool,
}

#[derive(Serialize)]
pub struct AuditEntryDto {
    pub id: i64,
    pub event_type: String,
    pub aggregate_id: String,
    pub account_id: Option<i64>,
    pub payload_json: String,
    pub occurred_at: String,
}

#[derive(Serialize)]
pub struct ContentPackStatusDto {
    pub id: i64,
    pub pack_name: String,
    pub version: String,
    pub subject_names: Vec<String>,
    pub question_count: i32,
    pub installed_at: String,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct ContentPackInstallDto {
    pub pack_id: i64,
    pub pack_name: String,
    pub version: String,
    pub subjects_installed: Vec<String>,
    pub question_count: i32,
    pub status: String,
    pub installed_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_system_status(
    state: State<'_, AppState>,
) -> Result<SystemStatusDto, CommandError>

#[tauri::command]
pub async fn run_migration_check(
    state: State<'_, AppState>,
) -> Result<MigrationStatusDto, CommandError>

#[tauri::command]
pub async fn get_audit_log(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<AuditEntryDto>, CommandError>

#[tauri::command]
pub async fn toggle_feature_flag(
    state: State<'_, AppState>,
    flag: String,
    enabled: bool,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_content_pack_status(
    state: State<'_, AppState>,
) -> Result<Vec<ContentPackStatusDto>, CommandError>

#[tauri::command]
pub async fn install_content_pack(
    state: State<'_, AppState>,
    pack_path: String,
) -> Result<ContentPackInstallDto, CommandError>
```

**Service delegation:** `AdminService` in `ecoach-commands` (thin admin layer using `ecoach-storage` and `ecoach-content` directly)

---

# PART 7: PHASED DELIVERY PLAN

## Reading Key

Each week entry lists: **Tasks**, **Migrations applied**, **Commands exposed**, **Test criteria**.

---

## Phase 0 — Foundations (Weeks 1–3)

### Week 1: Workspace, Database, Identity

**Tasks:**
- Initialize Cargo workspace with all 22 crate stubs
- Set up `ecoach-substrate`: `BasisPoints`, `Role`, `EntitlementTier`, `SeverityLevel`, `TrendDirection`, `DomainEvent`, `EventEnvelope`
- Set up `ecoach-storage`: SQLite pool via `sqlx`, WAL mode pragma, foreign keys pragma, embedded migration runner
- Write and apply migration 001 (`accounts`, `student_profiles`, `parent_profiles`, `parent_student_links`, `admin_profiles`)
- Implement `IdentityService`: `create_account`, `verify_pin`, `list_accounts`, `link_parent_student`, `reset_student_pin`
- Register Tauri app shell; expose first command group

**Migrations applied:** `001_identity.sql`

**Commands exposed:**
- `create_account`, `login_with_pin`, `list_accounts`, `switch_account`, `link_parent_student`, `reset_student_pin`, `get_linked_students`, `update_account_profile`

**Test criteria:**
- Migration applies without errors; WAL mode confirmed via `PRAGMA journal_mode`
- Account created, PIN verified, wrong PIN increments `failed_pin_attempts`
- Parent-student link created and queryable
- All 8 identity commands return valid DTOs from a seeded fixture database

---

### Week 2: Curriculum Graph

**Tasks:**
- Write and apply migrations 002 (`curriculum_versions`, `subjects`, `strands`, `sub_strands`, `content_standards`, `indicators`, `topics`, `skill_atoms`, `curriculum_node_relations`)
- Write and apply migration 003 (`questions`, `question_options`, `question_families`, `question_intelligence_profiles`)
- Implement `CurriculumService`: all 5 read queries
- Implement `QuestionRepository`: `get_by_topic`, `get_by_family`, `get_candidate_pool`
- Seed test data: 2 subjects (Mathematics, Integrated Science), 5 topics each, 50 questions total

**Migrations applied:** `002_curriculum.sql`, `003_questions.sql`

**Commands exposed:**
- `get_curriculum_versions`, `get_subjects`, `get_topics`, `get_topic_tree`, `get_skill_atoms`

**Test criteria:**
- Topic tree query returns correct nested structure for each subject
- Question candidate pool returns questions filtered by topic scope
- `get_topic_tree` completes under 50ms with 100 topics

---

### Week 3: Student State Schema + Coach Init

**Tasks:**
- Write and apply migration 004 (`student_topic_states`, `student_error_profiles`, `student_question_attempts`)
- Write and apply migration 005 (`sessions`, `session_events`, `session_questions`)
- Implement `CoachBrainService` init: `resolve_lifecycle_state`, `get_next_action` (Phase 0 states only: `OnboardingRequired`, `SubjectSelectionRequired`, `DiagnosticRequired`, `ContentReadinessRequired`, `PlanGenerationRequired`)
- Implement `StudentModelService` skeleton: `get_or_create_topic_state`
- Expose basic Tauri command boundary; register all command modules in `main.rs`

**Migrations applied:** `004_student_state.sql`, `005_sessions.sql`

**Commands exposed:**
- `get_coach_state`, `get_next_action`
- `get_student_state` (initial stub — returns zeroed state)

**Test criteria:**
- New student account triggers `OnboardingRequired` lifecycle state
- After subject selection, state advances to `DiagnosticRequired`
- `get_student_state` returns valid DTO without panicking on empty state

---

## Phase 1 — Core Primitives (Weeks 4–8)

### Week 4: Coach Brain State Machine

**Tasks:**
- Write and apply migration 006 (`coach_state`, `missions`, `mission_items`, `plan_days`, `interventions`, `coach_events`)
- Implement all 14 `CoachLifecycleState` transitions with full guard conditions (see section 8.2)
- Implement `PlanEngine` v1: `generate_daily_plan`, `generate_weekly_plan`, `mark_day_complete`
- Implement `MissionFactory`: `create_mission_from_state`, `get_active_mission`
- Wire `CoachBrainService.recompute_after_session()` — called at session end

**Migrations applied:** `006_coach.sql`

**Commands exposed:**
- `get_current_mission`, `dismiss_intervention`, `acknowledge_coach_event`
- `get_weekly_plan`, `get_daily_schedule`

**Test criteria:**
- All 14 state transitions exercised in unit tests with guard conditions
- Plan generation produces a valid 7-day plan given exam date + subject list
- Mission created correctly for `ReadyForTodayMission` state

---

### Week 5: Memory Engine

**Tasks:**
- Write and apply migration 007 (`student_memory_states`, `memory_review_queue`, `recall_attempts`)
- Implement `MemoryEngine`: `compute_msi`, `detect_decay`, `schedule_next_review`, `process_recall_attempt`
- Implement all 12 `MemoryState` transitions
- Implement spaced repetition scheduler: intervals 1d → 3d → 7d → 14d → 30d
- Implement 6 decay signal detectors
- Wire memory recompute into `submit_attempt` hot path

**Migrations applied:** `007_memory.sql`

**Commands exposed:**
- `get_memory_shelf`, `get_rescue_queue`, `submit_recall_attempt`, `get_memory_health`

**Test criteria:**
- MSI formula produces correct values for known input vectors
- Decay detection triggers correctly after simulated time gap
- Review scheduler produces correct next-review dates for all 5 interval steps
- MemoryState transitions tested: `Unformed` → `DurableMastery`, then regression to `AtRisk`

---

### Week 6: Session Runtime + Answer Pipeline

**Tasks:**
- Implement `SessionOrchestrator`: `start_session`, `end_session`, `pause_session`, `resume_session`
- Implement `StudentModelService.process_answer()` — the core evidence loop (classify error, compute evidence weight, update topic state, emit events)
- Implement `QuestionSelector` with candidate fit formula
- Implement `MasteryState` machine (8 states, full transition logic)
- Implement EMA update for accuracy, speed, retention, confidence scores

**Migrations applied:** (none; uses existing migrations)

**Commands exposed:**
- `start_session`, `submit_attempt`, `end_session`, `get_session_history`, `record_session_event`

**Test criteria:**
- Submit 10 correct answers → mastery score increases monotonically
- Submit 5 wrong answers with misconception tags → error profile updated
- Evidence weight halved when `hint_count > 0`
- Session complete event triggers coach brain recompute
- `classify_error` returns correct type for each of the 10 error type scenarios

---

### Week 7: Knowledge Gap Engine

**Tasks:**
- Write and apply migration 009 (`knowledge_gaps`, `gap_repair_sessions`, `gap_repair_attempts`)
- Implement `KnowledgeGapService`: `compute_gap_map`, `score_gap`, `rank_gaps`, `start_repair`, `process_repair_attempt`, `close_repair`
- Implement all 10 `KnowledgeGapType` classifications
- Implement 7 gap discovery methods (passive detection wired into `process_answer`)
- Implement gap scoring formula

**Migrations applied:** `009_knowledge_gap.sql`

**Commands exposed:**
- `get_gap_map`, `get_gap_priority_list`, `start_gap_repair`, `submit_gap_repair_attempt`, `close_gap_repair`

**Test criteria:**
- Gap map correctly identifies critical vs hidden gaps from fixture data
- Gap ranks correlate with severity scores
- Repair session routes to correct intervention for each gap type
- Passive gap detection fires correctly on wrong answer submission

---

### Week 8: Goals and Calendar

**Tasks:**
- Write and apply migration 010 (`goals`, `exam_calendar`, `schedule_ledger`, `availability_profiles`)
- Implement `GoalsCalendarService`: `set_exam_goal`, `generate_weekly_plan`, `get_daily_schedule`, `get_readiness_timeline`
- Implement deadline pressure engine: total days → weekly load → urgency level
- Implement 3-layer schedule structure (Macro / Rolling Active / Daily)
- Implement `ReadinessTimelineProjector`: project readiness trajectory to exam date

**Migrations applied:** `010_goals_calendar.sql`

**Commands exposed:**
- `set_exam_goal`, `get_weekly_plan`, `get_daily_schedule`, `update_goal`, `get_readiness_timeline`

**Test criteria:**
- Exam goal set → weekly plan generated with correct number of study days
- Readiness timeline projects future milestones correctly
- Daily schedule respects `daily_study_budget_minutes` constraint
- Edge case: exam date in 7 days → plan intensity escalates correctly

---

## Phase 2 — Essential Workflows (Weeks 9–14)

### Week 9: Journey Mode

**Tasks:**
- Implement 7 Journey Engines (Starting Point, Deadline Pressure, Curriculum Decomposition, Path Sequencing, Session Composer, Adaptation, Exam Readiness)
- Implement all 5 `JourneyPhase` transitions
- Implement session structure templates (standard 5-part and stronger-learner 4-part)
- Wire journey sessions into `start_session` with `session_type = "coach_mission"`
- Implement `AdaptationEngine`: post-session adjustments to plan

**Migrations applied:** (none; uses 004–010)

**Commands exposed:** (all session + coach commands fully functional for journey mode)

**Test criteria:**
- Full journey session flow: start → submit answers → end → coach recomputes → next mission generated
- Path sequencing correctly orders foundation-first vs high-yield-first based on archetype
- Adaptation engine modifies plan after session with accuracy < 40%

---

### Week 10: Rise Mode

**Tasks:**
- Implement `RiseEngine`: `compute_rise_student_scores`, `select_rise_stage`, `build_rise_session`
- Implement all 4 `RiseStage` transitions with entry criteria
- Implement 8 internal intelligence scores (foundation, recall, speed, accuracy, pressure_stability, misconception_density, momentum, transformation_readiness)
- Implement Momentum score and Strain score formulas
- Implement `PressureLevel` ladder (6 levels)
- Implement error-type → intervention mapping (10 mappings)

**Migrations applied:** (none new)

**Commands exposed:** (journey + session commands support Rise mode sessions)

**Test criteria:**
- Student starting in `Rescue` stage with foundation_score < 0.3 routes to reteach intervention
- Momentum score formula produces correct values for known vectors
- Stage transition from `Rescue` to `Stabilize` requires transformation_readiness >= threshold

---

### Week 11: Mock Centre — Forecast + Compilation

**Tasks:**
- Write and apply migration 008 (`mock_instances`, `mock_sessions`, `mock_attempts`, `mock_analysis`)
- Implement `ForecastEngine`: compute `ForecastScore` using all 7 components
- Implement `MockCompiler`: `compile_mock` for all 6 `MockType` variants
- Implement Mock Orchestration formula (7 components + anti-repeat penalty)
- Implement Mock Selection formula (8 components)

**Migrations applied:** `008_mock_centre.sql`

**Commands exposed:**
- `compile_mock`, `start_mock`, `get_forecast_report`, `get_mock_history`

**Test criteria:**
- Forecast scores computed correctly for seed topic frequency data
- Mock compiled with correct question distribution per subject
- Anti-repeat penalty prevents re-selection of recently seen questions

---

### Week 12: Mock Analysis — 8-Section Report

**Tasks:**
- Implement `MockAnalyzer`: `analyze_mock_session`, producing all 8 sections
- Section 1: overall summary (score, predicted BECE range, timing, readiness movement)
- Section 2: subject/topic performance table
- Section 3: broken concept links, prerequisite failures
- Section 4: misconception diagnosis with counts
- Section 5: representation breakdown (text/diagram/graph/table)
- Section 6: timing diagnosis (slow-correct, fast-careless, pacing collapse)
- Section 7: confidence diagnosis (correct-unsure, wrong-confident, guessing rate)
- Section 8: action plan generation

**Migrations applied:** (none; extends 008)

**Commands exposed:**
- `submit_mock_attempt`, `end_mock`, `get_mock_analysis`

**Test criteria:**
- All 8 sections populated for a completed 40-question mock
- Predicted BECE band computed from weighted mastery formula
- Action plan contains at least one "repair_now" topic for mock with accuracy < 60%

---

### Week 13: Parent Dashboard + Alerts Engine

**Tasks:**
- Write and apply migration 012 (`parent_alerts`, `report_snapshots`, `recommendations`)
- Implement `ReportingService`: `get_parent_dashboard`, `get_parent_child_report`, `generate_weekly_digest`
- Implement `ParentAlertEngine`: detect all 9 auto-alert conditions (inactivity, decline, exam near, etc.)
- Implement internal → parent translation logic (plain-language summaries)
- Implement `PrintableReportGenerator`

**Migrations applied:** `012_reporting.sql`

**Commands exposed:**
- `get_parent_dashboard`, `get_parent_child_report`, `get_parent_alerts`, `mark_alert_read`, `get_weekly_digest`, `generate_printable_report`

**Test criteria:**
- Alert generated when student has 5+ inactive days
- Alert generated when score trend is −8% over 3 assessments
- Parent dashboard `children` array correctly reflects all linked students
- Plain-language translation produces human-readable summary (fixture test)

---

### Week 14: Content Packs

**Tasks:**
- Write and apply migration 011 (`content_packs`, `content_pack_manifests`, `content_pack_items`)
- Implement `ContentPackService`: `install_content_pack` (verify signature, parse manifest, insert curriculum + questions)
- Implement pack manifest format (JSON schema with version, subjects, question count, curriculum version)
- Wire `ContentReadinessStatus` checks in coach brain
- Implement `AdminService`: `get_system_status`, `run_migration_check`, `get_content_pack_status`, `install_content_pack`

**Migrations applied:** `011_content_packs.sql`

**Commands exposed:**
- `install_content_pack`, `get_content_pack_status`, `get_system_status`, `run_migration_check`, `get_audit_log`, `toggle_feature_flag`

**Test criteria:**
- Pack install populates curriculum and question tables correctly
- Coach lifecycle advances from `ContentReadinessRequired` to `PlanGenerationRequired` after pack install
- Migration check correctly reports pending vs applied migrations

---

## Phase 3 — Exam Simulation & Intelligence (Weeks 15–20)

### Week 15: Diagnostic Battery Engine

**Tasks:**
- Write and apply migration 018 (`diagnostic_sessions`, `diagnostic_question_log`, `diagnostic_problem_cards`, `diagnostic_hypotheses`)
- Implement `DiagnosticBatteryService` with all 7 stages (`FastBaselineScan` through `MicroRecheck`)
- Implement all 12 diagnostic dimensions
- Implement guessing detection (8-signal multi-condition)
- Implement adaptive stage routing (zoom into weak areas)
- Implement `ProblemCardFactory`: generate cards from diagnostic results
- Implement 3 duration modes (light ~35min, standard ~60min, deep ~90min)

**Migrations applied:** `018_intake.sql` (repurposed; diagnostic tables created here)

**Commands exposed:**
- `start_diagnostic`, `submit_diagnostic_response`, `get_diagnostic_report`, `get_problem_cards`

**Test criteria:**
- Light mode completes in ≤24 baseline + 10 adaptive items
- Stage routing zooms into topics where accuracy < 50%
- Guessing detection fires when confidence = "guessed" and item is correct
- Problem cards generated for all detected weaknesses

---

### Week 16: Glossary System

**Tasks:**
- Write and apply migration 013 (`glossary_entries`, `glossary_entry_relations`, `glossary_topic_bundles`)
- Implement `GlossaryService` with FTS (SQLite full-text search via FTS5 virtual table)
- Implement related entry traversal
- Implement topic bundle queries

**Migrations applied:** `013_glossary.sql`

**Commands exposed:**
- `search_glossary`, `get_glossary_entry`, `get_related_entries`, `get_topic_glossary`

**Test criteria:**
- FTS search returns results for partial term matches
- Related entries traversal returns correct neighbors
- Topic glossary returns all entries for a given topic

---

### Week 17: Library System

**Tasks:**
- Write and apply migration 014 (`library_shelves`, `library_items`, `mistake_bank`, `revision_packs`, `revision_pack_items`)
- Implement `LibraryService`: all 6 commands
- Implement default shelves on account creation (Saved, Mistakes, Revision)
- Implement `RevisionPackBuilder`: aggregate weak topics + mistakes into session-ready pack

**Migrations applied:** `014_library.sql`

**Commands exposed:**
- `get_library_shelves`, `get_shelf_items`, `save_to_library`, `add_to_mistake_bank`, `get_mistake_bank`, `create_revision_pack`

**Test criteria:**
- Default shelves created on new account
- Mistake bank entry created from attempt with `is_correct = false`
- Revision pack contains correct question count from specified topics

---

### Week 18: Traps + Past Papers

**Tasks:**
- Write and apply migration 017 (`traps_sessions`, `contrast_profiles`, `traps_cards`, `traps_attempts`)
- Write and apply migration 016 (`past_papers`, `past_paper_questions`, `past_paper_question_families`)
- Implement `TrapsEngine`: all 5 `TrapsMode` variants, 5 `TrapsTimerMode` variants, end-of-round summary
- Implement past paper question family mining and recurrence tracking

**Migrations applied:** `016_past_papers.sql`, `017_traps.sql`

**Commands exposed:** (traps sessions exposed via `start_session` with `session_type = "traps"`)

**Test criteria:**
- Traps session routes cards correctly by `contrast_profile`
- Timer mode `Pressure` auto-skips on timeout
- End-of-round summary accurately reflects confusion breakdown

---

### Week 19: Game Engines

**Tasks:**
- Write and apply migration 015 (`game_sessions`, `game_actions`, `leaderboard_entries`)
- Implement `MindStackEngine`: control ladder (4 levels), 5 variants, mercy design rules, reshape mechanics
- Implement `TugOfWarEngine`: rope zones (5), power-ups (5), momentum meter rules
- Implement leaderboard persistence (local, per-device)

**Migrations applied:** `015_games.sql`

**Commands exposed:**
- `start_game`, `submit_game_action`, `end_game`, `get_game_leaderboard`

**Test criteria:**
- MindStack control level correctly reflects answer quality (Excellent → full movement + reshape)
- Tug of War rope position updates correctly on correct/wrong answers
- Streak of 3 triggers "stronger pull" in Tug of War
- Leaderboard correctly ranks scores descending

---

### Week 20: Reporting Engine Extensions

**Tasks:**
- Extend migration 012 with `readiness_proofs`, `readiness_contracts`, `readiness_claims`
- Implement `ReadinessProofEngine`: generate readiness claim with evidence summary
- Implement `WeeklyDigestService`
- Implement printable report PDF data assembly
- Wire all reporting read models (home dashboard, coach directives, parent digest)

**Migrations applied:** (012 extensions)

**Commands exposed:** (all reporting commands fully functional)

**Test criteria:**
- Readiness proof generates correctly when mastery > 72% across all subjects
- Weekly digest contains correct session count and mastery gains for the week
- Printable report data JSON contains all 5 parent dashboard sections

---

## Phase 4 — Premium, Elite & Hardening (Weeks 21–26)

### Week 21: Premium Concierge

**Tasks:**
- Write and apply migration 019 (`premium_strategy_sessions`, `premium_alerts`, `premium_interventions`)
- Implement `PremiumService`: `get_premium_parent_command_center`, `generate_strategy_session`, `detect_premium_alerts`
- Implement 9 premium alert detection conditions (memory slippage, false confidence, plateau, etc.)
- Implement intervention catalog routing (9 intervention types by diagnosis)
- Implement 6-layer premium architecture (Diagnosis → Strategy → Execution → Oversight → HumanExcellence → ParentConfidence)

**Migrations applied:** `019_premium.sql`

**Commands exposed:** (premium commands added to `reporting_commands.rs` and `coach_commands.rs`)

**Test criteria:**
- Premium alerts fire correctly for each of the 9 detection conditions
- Strategy session generated with correct intervention type for each diagnosis
- Premium parent command center fields all populated

---

### Week 22: Elite Mode

**Tasks:**
- Write and apply migration 020 (`elite_sessions`, `elite_performance_profiles`, `elite_missions`, `elite_badges`)
- Implement `EliteEngine`: all 4 `EliteTier` progressions, all 7 `ElitePillar` scoring dimensions, all 7 `EliteSessionType` variants
- Implement `ElitePerformanceDimensions` scoring (8 dimensions)
- Implement EPS (Elite Performance Score) computation
- Implement elite entry criteria check (80%+ mastery + consistency + speed + low hint dependence)
- Implement elite badge award logic (7 badges)

**Migrations applied:** `020_elite.sql`

**Commands exposed:** (elite sessions exposed via `start_session` with `session_type = "elite"`)

**Test criteria:**
- Entry criteria correctly blocks students below threshold
- EPS computation correct for known dimension vectors
- `PerfectRun` session type collapses on first error
- Badge awarded when `PrecisionBeast` criteria met (≥95% accuracy over 20 elite questions)

---

### Week 23: Event Sourcing + Audit

**Tasks:**
- Write and apply migration 021 (`event_log`, `outbox_events`, `audit_log`)
- Implement full append-only event log
- Implement outbox pattern: events written to `outbox_events` before processing, marked processed after handler succeeds
- Implement correlation ID tracking across cascading events
- Implement `AuditTrailService`: query audit log, filter by event type / account / date range
- Implement replay/rebuild tools for state regeneration

**Migrations applied:** `021_event_sourcing.sql`

**Commands exposed:**
- `get_audit_log` (fully functional)

**Test criteria:**
- Every `submit_attempt` call writes to event log
- Outbox pattern: handler failure leaves event in `outbox_events` for retry
- State rebuild from event log produces identical result to current materialized state (round-trip test)

---

### Week 24: Document Intake / OCR Bridge

**Tasks:**
- Wire `IntakeService` to local OCR bridge (external process or WASM component)
- Implement document intelligence extracts (6 intelligence signals)
- Implement candidate question extraction from parsed document
- Implement approval/rejection workflow
- Implement goal creation from document intelligence (e.g., teacher correction areas → `ParentTeacher` goal)

**Migrations applied:** (018 fully activated)

**Commands exposed:**
- `upload_document`, `get_intake_status`, `approve_intake_question`, `reject_intake_question` (fully functional)

**Test criteria:**
- Upload triggers background parse job
- Parsed candidate questions appear in review queue
- Approved question added to student's personal question bank

---

### Week 25: Performance Benchmarks + Query Optimization

**Tasks:**
- Benchmark all hot-path queries: `submit_attempt` pipeline end-to-end < 100ms
- Add missing indexes (review all slow queries via EXPLAIN QUERY PLAN)
- Optimize `get_mastery_map` read model (pre-aggregate, cache in read model table)
- Implement background job hardening: idempotent handlers, bounded exponential backoff, dead-letter queue
- Implement stale read-model rebuild job
- Benchmark: mock compilation < 500ms, diagnostic report generation < 200ms

**Migrations applied:** (index additions only)

**Commands exposed:** (no new commands; all existing commands optimized)

**Test criteria:**
- `submit_attempt` completes in < 100ms on a database with 10,000 attempts
- `get_mastery_map` completes in < 50ms for a student with full subject coverage
- Background jobs survive handler panic without data corruption
- All benchmark targets met

---

### Week 26: Integration Testing + Final Schema Review

**Tasks:**
- Write and apply migration 022 (`system_settings`, `feature_flags`, `app_config`)
- Full integration test suite: onboarding → diagnostic → daily session → mock → parent report
- Final schema review: check all foreign keys, all indexes, all JSON column shapes
- Cross-crate integration tests: session → student model → coach brain → reporting pipeline
- Build verification: `cargo build --release` clean, no warnings
- Seed data package: complete test fixture for all 26 migrations

**Migrations applied:** `022_system_settings.sql`

**Commands exposed:** (no new commands)

**Test criteria:**
- Full onboarding-to-exam-ready flow passes in integration test
- All 22 migration files apply cleanly on fresh database
- `cargo test` passes across all crates
- `cargo clippy` zero warnings
- Release build size under 15MB (backend only)

---

# PART 8: MASTER REFERENCE

## 8.1 All 22 Crate Responsibilities

| Crate | Owns | Depends On |
|---|---|---|
| `ecoach-substrate` | `BasisPoints`, `Role`, `EntitlementTier`, `SeverityLevel`, `TrendDirection`, `DomainEvent`, `EventEnvelope`, scoring helpers, time utilities, shared error traits | (none) |
| `ecoach-storage` | SQLite connection pool, WAL config, migration runner, `Repository` trait, query builder helpers | `ecoach-substrate` |
| `ecoach-identity` | Account CRUD, PIN hash/verify, session tokens, parent-student links, entitlement checks | `ecoach-substrate`, `ecoach-storage` |
| `ecoach-curriculum` | Curriculum graph reads, topic tree, skill atoms, prerequisite resolution, node coverage | `ecoach-substrate`, `ecoach-storage` |
| `ecoach-content` | Content pack install/verify, manifest parsing, pack manifest schema, signed pack validation | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-questions` | Question CRUD, question selection (candidate fit formula), family classification, question intelligence profiles, 8-axis taxonomy | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-student-model` | `StudentTopicState`, mastery formula, priority formula, error classification, evidence weight, EMA updates, mastery state machine, error profiles | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions` |
| `ecoach-diagnostics` | Diagnostic battery engine (7 stages, 12 dimensions), guessing detection, adaptive routing, problem card generation, diagnostic report | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model` |
| `ecoach-coach-brain` | `CoachLifecycleState` (14 states), all 14 transitions, `NextCoachAction`, mission factory, plan engine, intervention catalog, coaching trigger rules, help ladder | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-curriculum` |
| `ecoach-memory` | `MemoryState` (12 states), MSI formula, decay detection (7 types, 6 signal categories), spaced repetition scheduler, recovery planner, connection rebuilder | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model` |
| `ecoach-sessions` | Session orchestrator, session lifecycle (created/active/paused/completed/abandoned), question sequencing, time tracking, session event log, answer pipeline coordination | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model`, `ecoach-memory`, `ecoach-coach-brain` |
| `ecoach-mock-centre` | Mock compilation (6 types), forecast engine, mock selection formula, mock runtime, 8-section post-mock analysis, weakness scoring, past paper ingestion | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model`, `ecoach-sessions` |
| `ecoach-knowledge-gap` | Gap map, 10 gap types, gap scoring, gap priority ranking, repair session, 7 discovery methods, solidification tracking | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-sessions` |
| `ecoach-goals-calendar` | Exam goals (7 categories, 8 states), deadline pressure engine, 3-layer schedule, daily session composer, readiness timeline projector, availability profiles | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-coach-brain` |
| `ecoach-intake` | Document upload, OCR bridge, candidate question extraction, approval/rejection workflow, document intelligence extraction | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum`, `ecoach-questions` |
| `ecoach-reporting` | Parent dashboard (5 sections), parent alert engine (9 conditions), child report, weekly digest, readiness proof, printable report data assembly | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-goals-calendar`, `ecoach-mock-centre` |
| `ecoach-library` | Library shelves, library items, mistake bank, revision packs, personal knowledge state | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model` |
| `ecoach-glossary` | Glossary entries, FTS5 search index, related entry traversal, topic bundles, audio path references | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-games` | `MindStackEngine` (5 variants), `TugOfWarEngine` (5 rope zones), `TrapsEngine` (5 modes), power-ups, leaderboard | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-sessions` |
| `ecoach-past-papers` | Past paper ingestion, question family mining, recurrence tracking, year/section metadata | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-curriculum` |
| `ecoach-premium` | Premium alert detection (9 conditions), strategy session engine, 6-layer premium architecture, intervention catalog routing, premium parent command center | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-coach-brain`, `ecoach-reporting` |
| `ecoach-elite` | Elite tier progression (4 tiers), 7 pillar scoring, 8 EPS dimensions, elite session types (7), elite entry criteria, badge award system (7 badges) | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-sessions`, `ecoach-student-model` |
| `ecoach-commands` | Tauri command boundary — all 16 command modules, `CommandError`, `AppState`, service wiring, DTO definitions | all domain crates |

---

## 8.2 Complete State Machine Reference

### CoachLifecycleState — 14 States

| State | Description | Entry Condition | Exit Transitions |
|---|---|---|---|
| `OnboardingRequired` | Account just created, no profile | `first_run = 1` | → `SubjectSelectionRequired` when profile completed |
| `SubjectSelectionRequired` | Profile done, no subjects selected | No subjects enrolled | → `ContentReadinessRequired` when subjects selected |
| `ContentReadinessRequired` | Subjects selected, no content pack | No packs for selected subjects | → `DiagnosticRequired` when packs installed |
| `DiagnosticRequired` | Content ready, no baseline | No diagnostic completed | → `PlanGenerationRequired` when diagnostic done |
| `PlanGenerationRequired` | Diagnostic done, no plan | No active plan | → `ReadyForTodayMission` when plan generated |
| `ReadyForTodayMission` | Plan active, today's mission pending | Plan exists, no active mission | → `MissionInProgress` when session started |
| `MissionInProgress` | Session actively running | Session status = `active` | → `MissionReviewRequired` when session ends |
| `MissionReviewRequired` | Session ended, results not reviewed | Session completed, coach not updated | → `ReadyForTodayMission` after recompute |
| `RepairRequired` | Topic blocked; repair needed | accuracy < 40% after 2 sessions | → `ReadyForTodayMission` after repair session |
| `BlockedOnTopic` | No unblocked topics available | All current topics blocked | → `PlanAdjustmentRequired` |
| `PlanAdjustmentRequired` | Plan drift detected | Missed sessions / low adherence | → `ReadyForTodayMission` after plan updated |
| `ReviewDay` | Scheduled review milestone | Checkpoint day in plan | → `ReadyForTodayMission` after review |
| `ExamMode` | Final exam phase | ≤14 days to exam | → terminal (exam date passed) |
| `StalledNoContent` | Content gap — topic has no questions | Question coverage = Red | → `ContentReadinessRequired` after pack update |

### MemoryState — 12 States

| State | Level | MSI Range | Description |
|---|---|---|---|
| `Unformed` | 0 | 0 | Never encountered |
| `Exposed` | 1 | 1–15 | Seen once; no retention |
| `Familiar` | 2 | 16–30 | Recognizes with context |
| `Recognizable` | 3 | 31–45 | MCQ recognition reliable |
| `SupportedRecall` | 4 | 46–55 | Needs cues to recall |
| `FreeRecall` | 5 | 56–65 | Recalls without prompts |
| `AppliedRecall` | 6 | 66–74 | Uses in problem contexts |
| `TransferRecall` | 7 | 75–83 | Recalls in new wordings |
| `PressureStable` | 8 | 84–89 | Holds under timed conditions |
| `DurableMastery` | 9 | 90–100 | Locked in; stable over time |
| `AtRisk` | 10 | — | Was higher; decay detected |
| `Collapsed` | 11 | 0–25 | Regressed significantly |

**Transition rules:**
- Forward: MSI crosses threshold boundary + minimum evidence count
- Regression: MSI drops below lower boundary of current state
- `AtRisk`: triggered when MSI drops > 15bp from peak without time gap
- `Collapsed`: triggered when MSI < 25 after prior `FreeRecall` or higher

### MasteryState — 8 States

| State | Mastery Range | Evidence Required | Fragility Limit |
|---|---|---|---|
| `Unseen` | 0 | 0 | — |
| `Exposed` | 0–24% | ≥1 | — |
| `Emerging` | 25–44% | ≥3 | — |
| `Partial` | 45–59% | ≥8 | — |
| `Fragile` | 60–71% | ≥15 | < 30% |
| `Stable` | 72–81% | ≥25 | < 20% |
| `Robust` | 82–89% | — | < 15%, transfer ≥ 65%, retention ≥ 70% |
| `ExamReady` | ≥ 90% | — | < 15%, retention ≥ 80%, pressure_collapse < 15% |

Regression is possible at all states. `ExamReady` → `Robust` if mastery drops below 80%.

### JourneyPhase — 5 Phases

| Phase | Focus | Entry Trigger | Exit Trigger |
|---|---|---|---|
| `StabilizeFoundation` | Fix major weaknesses; core concepts | Journey start | Critical gaps closed; foundation_score ≥ 60% |
| `BuildCore` | Cover main syllabus; deepen understanding | Foundation stable | ≥ 70% syllabus coverage |
| `StrengthenWeakLinks` | Attack weak topics; recurring mistakes | Core built | Top-ranked gaps addressed |
| `ExamConditioning` | Timed practice; pressure; mixed-topic | ≤ 6 weeks to exam | Readiness ≥ 65% |
| `FinalReadiness` | Mocks; revision bursts; confidence; strategy | ≤ 3 weeks to exam | Exam date |

### RiseStage — 4 Stages

| Stage | Goal | Entry Criteria | Exit Criteria |
|---|---|---|---|
| `Rescue` | Stop the bleeding; first wins; shame removal | foundation_score < 40% | foundation_score ≥ 40%, first_win achieved |
| `Stabilize` | Repeatable correct thinking; scaffolded | foundation_score ≥ 40% | accuracy ≥ 55% consistently over 3 sessions |
| `Accelerate` | Speed + independence; timed drills; pressure | accuracy ≥ 55%, stable | transformation_readiness ≥ 70% |
| `Dominate` | Outperform top students; elite variants | transformation_readiness ≥ 70% | mastery ≥ 85%, speed ≥ 75% |

### MockType — 6 Types

| Type | Purpose | Selection Strategy | Conditions |
|---|---|---|---|
| `Forecast` | Maximum realism; mirrors likely exam | Weighted by ForecastScore; blueprint-aligned | Full duration; timed |
| `Diagnostic` | Maximum insight; reveals gaps | Maximize information value; probe weaknesses | Full duration; may have confidence capture |
| `Remediation` | Close known weak areas | Weight by gap severity and misconception density | Focused; can be shorter |
| `FinalExam` | Full readiness proof | Cover all blueprint targets; balanced difficulty | Full exam duration; strict timing |
| `Shock` | Resilience training; hard unexpected items | Surprise risk topics; unfamiliar formats | Short; brutal |
| `Wisdom` | Mastery proof; elite standards | High mastery requirement; elite question types | Strict accuracy targets |

---

## 8.3 All 60 Question Types

### Family A — Memory Questions (6)
1. Pure Recall — retrieve fact without any cues
2. Recognition — identify correct item from options
3. Memory Reconstruction — rebuild concept from fragments
4. Retrieval Under Pressure — recall under time constraint
5. Retention Check — recall after deliberate time gap
6. Recovery — recall after prior failure/correction

### Family B — Understanding Questions (6)
7. Concept Understanding — demonstrate grasp of core idea
8. Explanation — articulate why something is true
9. Example Generation — produce valid example of concept
10. Non-Example — identify what does NOT fit concept
11. Compare-and-Contrast — distinguish between two related concepts
12. Classification — assign item to correct category

### Family C — Reasoning Questions (6)
13. Reasoning — follow a logical chain to conclusion
14. Logical Deduction — derive conclusion from given premises
15. Inference — draw implied conclusion from evidence
16. Justification — provide valid reasoning for a claim
17. Claim Evaluation — assess whether a stated claim is correct
18. Counterexample — disprove a generalization

### Family D — Problem-Solving Questions (8)
19. Application — use concept in a new context
20. Transfer — apply knowledge to unfamiliar scenario
21. Multi-Step Problem Solving — coordinate multiple operations
22. Strategy Selection — choose most appropriate method
23. First-Step — identify the correct starting action
24. Next-Step — given partial work, choose next correct move
25. Decision-Making — select best option given constraints
26. Prioritization — rank items by given criterion

### Family E — Accuracy Questions (5)
27. Error Detection — find the mistake in given work
28. Correction — fix identified error
29. Misconception Exposure — reveal the false belief causing an error
30. Precision — answer requiring exact specification
31. Attention Control — question designed to catch careless reading

### Family F — Pattern and Structure Questions (8)
32. Pattern Recognition — identify recurring structure
33. Rule Discovery — infer the governing rule from examples
34. Sequence/Order — arrange items in correct order
35. Cause-and-Effect — identify causal relationship
36. Prediction — forecast outcome given conditions
37. Abstraction — generalize from specific to principle
38. Estimation — produce reasonable approximate answer
39. Representation Conversion — translate between formats (table → graph, etc.)

### Family G — Expression and Interpretation Questions (7)
40. Interpretation — extract meaning from data/text/diagram
41. Visualization — mentally picture described scenario
42. Mental Manipulation — rotate/transform represented object mentally
43. Synthesis — combine multiple concepts into one answer
44. Connection-Making — identify link between two ideas
45. Judgment — evaluate quality/correctness of an argument
46. Open-Ended Reasoning — construct extended explanation

### Family H — Growth-Control Questions (14)
47. Diagnostic — probe for specific weakness
48. Mastery Check — verify mastery level achieved
49. Threshold — gate question determining next difficulty
50. Adaptive Difficulty — system-selected based on current performance
51. Rescue — highly scaffolded recovery question
52. Challenge/Stretch — beyond current mastery, builds ceiling
53. Reflection/Metacognitive — "what made this difficult for you?"
54. Confidence Calibration — pair answer with confidence rating
55. Real-World Scenario — concept applied to real-life context
56. Reverse Reasoning — given outcome, infer input conditions
57. Multiple-Path — two valid methods; choose more efficient
58. Deep Thinking — multi-hop reasoning across concepts
59. Speed Fluency — rapid-fire retrieval drill format
60. Capstone — comprehensive integrating question for topic mastery

---

## 8.4 Post-Mock Analysis — 8 Sections

### Section 1 — Overall Summary
- Final score (basis points and percentage)
- Predicted BECE grade band (1–9 or equivalent)
- Confidence index (how reliable the prediction is)
- Timing assessment (fast/on-pace/slow overall)
- Readiness movement (delta from before mock)

### Section 2 — Subject / Topic Performance
- Per-subject: score, strong topics, weak topics, unstable topics
- Topic-level accuracy table
- Topics that regressed vs improved vs held

### Section 3 — Link-Level Diagnosis
- Exact broken concept links detected
- Prerequisite failures (topic A required for topic B; A is weak)
- Bundle collapses (multiple related concepts failed together)

### Section 4 — Misconception Diagnosis
- Top misconceptions triggered (ranked by frequency)
- Count per misconception
- Misconception status: suspected / active / unresolved / cleared
- Which distractors were selected and what they signal

### Section 5 — Representation Diagnosis
- Performance breakdown by format: text / diagram / graph / table / symbolic
- Weakest representation format
- Representation-specific repair recommendation

### Section 6 — Timing Diagnosis
- Slow-but-correct items (knowledge present; speed deficit)
- Fast-but-careless items (rushed; accuracy cost)
- Performance collapse near session end (fatigue signal)
- Section pacing (early vs late timing patterns)

### Section 7 — Confidence Diagnosis
- Correct-but-unsure count (knowledge fragile; needs consolidation)
- Wrong-but-confident count (false mastery; priority repair)
- Estimated guessing rate
- Confidence calibration score

### Section 8 — Action Plan
- Repair now: topics requiring immediate intervention (critical gaps)
- Drill: topics needing speed or repetition work
- Review: stable topics that slipped
- Recommended next mock date (computed from current readiness trajectory)

---

## 8.5 Seven Journey Engines

### Engine 1 — Starting Point Engine
**Purpose:** Establish current level before planning begins
**Inputs:** Past scores, recent behavior, diagnostic results, self-assessment responses
**Outputs:** Current mastery level per topic, gap map baseline, learner archetype classification
**Crate:** `ecoach-diagnostics` + `ecoach-student-model`

### Engine 2 — Deadline Pressure Engine
**Purpose:** Convert available time into feasible study plan
**Inputs:** Total calendar days to exam, realistic study days per week, daily budget minutes, known unavailable dates
**Outputs:** Urgency level, feasible path assessment, goal realism verdict, weekly load targets
**Crate:** `ecoach-goals-calendar`

### Engine 3 — Curriculum Decomposition Engine
**Purpose:** Break subjects into prioritized topic sequences
**Inputs:** Selected subjects, curriculum version, exam weights per topic, prerequisite graph
**Outputs:** Ordered topic list per subject, dependency map, coverage targets
**Crate:** `ecoach-curriculum`

### Engine 4 — Path Sequencing Engine
**Purpose:** Determine learning order strategy
**Inputs:** Learner archetype, current mastery per topic, time pressure, goal type
**Outputs:** Sequenced topic list using one of three strategies: foundation-first / high-yield-first / confidence-first
**Crate:** `ecoach-coach-brain`

### Engine 5 — Session Composer Engine
**Purpose:** Build each individual session from plan + current state
**Inputs:** Today's plan day, current mastery state, time available, session type, learner energy/fatigue signal
**Outputs:** Session configuration (type, topics, question count, duration, structure template)
**Crate:** `ecoach-sessions` + `ecoach-coach-brain`

### Engine 6 — Adaptation Engine
**Purpose:** Adjust plan after each session based on performance
**Inputs:** Session results, accuracy delta, mastery changes, time spent, coach events
**Outputs:** Plan day status update, next-day adjustments, topic re-ranking, intervention flags
**Crate:** `ecoach-coach-brain`

### Engine 7 — Exam Readiness Engine
**Purpose:** Continuously estimate exam performance probability
**Inputs:** Mastery per topic, timed performance, coverage, consistency, trend, retention, mock performance
**Outputs:** Overall readiness score (basis points), readiness band, predicted exam score, milestone dates
**Formula:** `Readiness = 0.25 × mastery + 0.20 × retention + 0.20 × mock_performance + 0.15 × speed + 0.10 × coverage + 0.10 × consistency` (with penalties for critical topic weakness, recurring mistakes, exam anxiety pattern)
**Crate:** `ecoach-reporting` + `ecoach-student-model`

---

## 8.6 Four BECE Subjects

| Subject | Code | Exam Weight Notes |
|---|---|---|
| English Language | ENG | Reading comprehension, grammar, vocabulary, essay writing |
| Mathematics | MATH | Number, algebra, geometry, statistics, measurement |
| Integrated Science | SCI | Biology, chemistry, physics, earth science concepts |
| Social Studies | SOC | History, geography, civics, economics — Ghana-focused |

All four subjects use the same topic tree, mastery model, memory engine, and question pipeline. Subject-specific configuration is controlled by curriculum pack data.

---

## 8.7 Diagnostic Battery Specification — Academic DNA Test

**Source:** idea34

### Overview

| Parameter | Light Mode | Standard Mode | Deep Mode |
|---|---|---|---|
| Total items | 35–38 | 46–50 | 52–56 |
| Baseline items | 18–20 | 22–24 | 22–24 |
| Adaptive zoom items | 10–12 | 14–16 | 18–20 |
| Condition layer items | 4–5 | 6–7 | 8–9 |
| Stability recheck items | 3–4 | 4–5 | 6–8 |
| Estimated duration | ~35 min | ~60 min | ~90 min |

### Seven Test Stages (Sequential, Adaptive)

1. **FastBaselineScan** — 20–24 items; broad topic coverage; identify strong / weak / uncertain zones
2. **TopicZoom** — automated zoom into weak/unstable areas identified in stage 1; 3–5 items per weak topic
3. **MisconceptionProbing** — expose why student is wrong; distractor-to-misconception mapping; confidence capture
4. **SpeedPressureLayer** — repeat subset under time conditions; compare timed vs untimed performance
5. **TransferLayer** — same concept in 5 forms: direct / word-problem / diagram / comparison / explain-why
6. **ConfidenceCapture** — 3-level rating: `sure` / `not_sure` / `guessed` captured on each item
7. **MicroRecheck** — reintroduce 4–8 items from stage 1 indirectly; test retention vs fluency vs guessing

### Twelve Dimensions Measured

| ID | Dimension | Method |
|---|---|---|
| A | Coverage | Topics touched / total syllabus topics |
| B | Accuracy | Correct rate per topic, per subject |
| C | Recall Strength | Free recall items without hints |
| D | Recognition vs Production | MCQ vs open response performance gap |
| E | Reasoning Depth | Pattern of guess / memorize / understand across items |
| F | Misconception Pattern | Distractor selection → misconception tagging |
| G | Speed | Normalized response latency per item |
| H | Pressure Response | Timed vs untimed accuracy delta |
| I | Transfer Ability | Performance on variant-form items |
| J | Stability | Consistency across parallel items |
| K | Confidence Calibration | Correct-but-unsure, wrong-but-confident ratios |
| L | Fatigue Pattern | Score decay over session duration |

### Output Format

```
DiagnosticReport {
    overall_profile: LearnerArchetype,
    dimensions: DiagnosticDimensionsDto,   // all 12 scores as basis points
    subject_reports: [SubjectDiagnosticDto per subject],
    problem_cards: [ProblemCardDto],        // one per detected weakness
    recommended_focus_topics: [topic_id],
    archetype: "WeakButConsistent" | "StrongButLazy" | "PanickingLastMinute" | "Overconfident" | "Discouraged"
}
```

---

## 8.8 Intelligence Constitution — Engine Registry

All backend engines organized by domain. Priority: P0 = build first, P1 = second wave.

### Domain A — Evidence Domain (6 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Response Evidence Ingestion | A1 | Capture raw attempt signal | Attempt submission DTO | Raw evidence record | `ecoach-sessions` | P0 |
| Content Signal Ingestion | A2 | Capture question/content quality signals | Question usage stats, feedback | Content quality event | `ecoach-questions` | P0 |
| Curriculum Ingestion | A3 | Parse and store curriculum versions | Curriculum pack JSON | Curriculum graph | `ecoach-curriculum` | P1 |
| Learner Signal Ingestion | A4 | Capture behavioral/engagement signals | Session events, timing data | Behavioral signal record | `ecoach-sessions` | P1 |
| Pack Signal Ingestion | A5 | Track content pack usage and coverage | Pack install events | Coverage ledger | `ecoach-content` | P1 |
| Evidence Normalization | A6 | Normalize raw signals into weighted evidence | Raw attempt + context | `EvidenceWeight` value | `ecoach-student-model` | P0 |

### Domain B — Knowledge Domain (10 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Topic Scope | B1 | Resolve topic boundaries and atom graph | Topic ID | Atom list, prerequisite chain | `ecoach-curriculum` | P1 |
| Concept State | B2 | Compute per-concept mastery state | Evidence records | `ConceptState` | `ecoach-student-model` | P0 |
| Topic State | B3 | Aggregate concept states to topic level | Concept states | `StudentTopicState` | `ecoach-student-model` | P0 |
| Hypothesis Competition | B4 | Maintain competing hypotheses about student knowledge | Evidence stream | Hypothesis set with probabilities | `ecoach-student-model` | P0 |
| Misconception | B5 | Track active misconceptions per student | Distractor selections, error types | Misconception profile | `ecoach-student-model` | P0 |
| Interference | B6 | Detect when similar concepts contaminate each other | Error patterns, concept neighbors | Interference pairs | `ecoach-memory` | P0 |
| Learner State | B7 | 7-dimension learner model | All student state tables | `LearnerState` aggregate | `ecoach-student-model` | P0 |
| Mastery Proof | B8 | Generate evidence-backed mastery claim | Topic state + evidence log | `MasteryProof` with evidence refs | `ecoach-student-model` | P0 |
| Coverage Gap | B9 | Compute syllabus coverage and gap map | Topic states, curriculum graph | `GapMapDto` | `ecoach-knowledge-gap` | P1 |
| Knowledge Graph | B10 | Maintain live knowledge graph for student | All mastery/gap data | Graph structure for sequencing | `ecoach-curriculum` | P1 |

### Domain C — Decision Domain (9 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Teaching Strategy | C1 | Select pedagogic approach per content type | Content type, student state | `TeachingStrategy` selection | `ecoach-coach-brain` | P0 |
| Sequencing | C2 | Order topics for maximum learning trajectory | Topic priority scores, dependencies | Ordered topic queue | `ecoach-coach-brain` | P0 |
| Timing | C3 | Determine session timing and pressure level | Student state, availability, urgency | `SessionTimingConfig` | `ecoach-goals-calendar` | P0 |
| Risk | C4 | Assess academic risk level | Mastery trends, exam countdown, gap severity | `RiskProfile` | `ecoach-coach-brain` | P0 |
| Adaptation | C5 | Adjust plan after session outcome | Session result, mastery delta | Plan adjustment commands | `ecoach-coach-brain` | P0 |
| Session Composer | C6 | Build session content from plan + state | Plan day, current state, available questions | `SessionConfig` with question list | `ecoach-sessions` | P0 |
| Content Selection | C7 | Score and select questions for a session | Session config, student state, question pool | Ordered `Vec<SelectedQuestion>` | `ecoach-questions` | P1 |
| Diagnostic Experiment | C8 | Design adaptive diagnostic probes | Weak topic hypothesis set | Diagnostic probe questions | `ecoach-diagnostics` | P1 |
| Protection Rule | C9 | Prevent harmful or counterproductive actions | Proposed action, student state | Allow / block / modify decision | `ecoach-coach-brain` | P1 |

### Domain D — Execution Design Domain (4 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Intervention Design | D1 | Select and configure repair intervention | Error type, gap type, student archetype | `InterventionConfig` | `ecoach-coach-brain` | P1 |
| Drill Generation | D2 | Build targeted drill from gap/error profile | Error fingerprint, topic scope | Drill question sequence | `ecoach-questions` | P1 |
| Mock Orchestration | D3 | Compile mock exam from forecast + gaps | Mock type, blueprint, student state | `MockInstance` with question list | `ecoach-mock-centre` | P1 |
| Assessment Construction | D4 | Build diagnostic or mastery check session | Target dimensions, topic scope | Assessment session config | `ecoach-diagnostics` | P1 |

### Domain E — Memory and Meta-Learning Domain (8 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Topic Memory | E1 | Track memory state per topic | Recall attempts, time gaps, decay signals | `MemoryState` + MSI score | `ecoach-memory` | P1 |
| Learner Memory | E2 | Aggregate memory health across all topics | All `MemoryState` records | `MemoryHealthDto` | `ecoach-memory` | P1 |
| Strategy Memory | E3 | Remember which interventions worked per student | Intervention outcomes | Strategy effectiveness profile | `ecoach-coach-brain` | P1 |
| Coach Self-Evaluation | E4 | Assess whether coach decisions are producing results | Session outcome trends, mastery velocity | Coach decision audit log | `ecoach-coach-brain` | P1 |
| Improvement Velocity | E5 | Track rate of mastery improvement over time | Mastery time series | Velocity score and trend | `ecoach-student-model` | P1 |
| Memory Strength Engine | E6 | Compute MSI from 6-component formula | Accuracy, speed, retention, variant, independence, connection scores | `BasisPoints` MSI | `ecoach-memory` | P0 |
| Decay Detection Engine | E7 | Detect memory decay before collapse | Signal categories (accuracy, time, confidence, support, stability, transfer, interference, behavioral) | `DecayAlert` with decay type | `ecoach-memory` | P0 |
| Recovery Planner | E8 | Build recovery sequence for decayed knowledge | Decay type, current memory state, intervention case (A–G) | Recovery session config | `ecoach-memory` | P1 |

### Domain F — Governance Domain (5 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Decision Arbitration | F1 | Resolve conflicts between competing engine outputs | Multiple engine recommendations | Single arbitrated action | `ecoach-coach-brain` | P1 |
| Confidence Gate | F2 | Block action when evidence is too weak | Evidence count, confidence score | Allow / defer decision | `ecoach-coach-brain` | P1 |
| Contradiction Check | F3 | Detect inconsistencies in student state | Student state snapshot | Contradiction flags | `ecoach-student-model` | P1 |
| Policy Guardrail | F4 | Enforce 8 Day-1 invariants | Proposed engine action | Compliant / non-compliant verdict | `ecoach-coach-brain` | P1 |
| Audit and Trace | F5 | Record all engine decisions with correlation IDs | All engine outputs | Event log entries | `ecoach-storage` | P0 |

---

*End of agent1_part4.md — Parts 6, 7, and 8 complete.*
