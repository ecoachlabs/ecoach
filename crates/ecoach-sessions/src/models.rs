use chrono::{DateTime, Utc};
use ecoach_questions::QuestionRemediationPlan;
use ecoach_substrate::{
    BasisPoints, FabricEvidenceRecord, FabricOrchestrationSummary, FabricSignal,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub student_id: i64,
    pub session_type: String,
    pub subject_id: Option<i64>,
    pub status: String,
    pub active_item_index: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeSessionStartInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub question_count: usize,
    pub is_timed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTestStartInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub question_count: usize,
    pub duration_minutes: Option<i64>,
    pub is_timed: bool,
    pub target_difficulty: Option<BasisPoints>,
    pub weakness_bias: bool,
}

// ── Custom test deep models (idea14) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomTestType {
    ClassTest,
    Midterm,
    MockExam,
    TerminalExam,
    RevisionQuiz,
    TeacherSurprise,
    BeceWassce,
    QuickQuiz,
    EndOfTerm,
    Custom,
}

impl CustomTestType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClassTest => "class_test",
            Self::Midterm => "midterm",
            Self::MockExam => "mock_exam",
            Self::TerminalExam => "terminal_exam",
            Self::RevisionQuiz => "revision_quiz",
            Self::TeacherSurprise => "teacher_surprise",
            Self::BeceWassce => "bece_wassce",
            Self::QuickQuiz => "quick_quiz",
            Self::EndOfTerm => "end_of_term",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomTestMode {
    LikelyQuestions,
    RealisticSimulation,
    PressureMode,
    FixWeakAreas,
    ConfidenceBuild,
    TeachThroughTest,
    LastMinuteRescue,
    TeacherStylePrep,
}

impl CustomTestMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LikelyQuestions => "likely_questions",
            Self::RealisticSimulation => "realistic_simulation",
            Self::PressureMode => "pressure_mode",
            Self::FixWeakAreas => "fix_weak_areas",
            Self::ConfidenceBuild => "confidence_build",
            Self::TeachThroughTest => "teach_through_test",
            Self::LastMinuteRescue => "last_minute_rescue",
            Self::TeacherStylePrep => "teacher_style_prep",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTestBlueprintInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub test_type: CustomTestType,
    pub mode: CustomTestMode,
    pub topic_ids: Vec<i64>,
    pub days_until_test: Option<i64>,
    pub target_score_bp: Option<BasisPoints>,
    pub question_count: Option<usize>,
    pub duration_minutes: Option<i64>,
    pub difficulty_preference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTestBlueprint {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub test_type: String,
    pub mode: String,
    pub session_archetype: String,
    pub question_count: i64,
    pub duration_minutes: Option<i64>,
    pub feedback_policy: String,
    pub hint_policy: String,
    pub pressure_profile: String,
    pub ordering_pattern: String,
    pub adaptation_policy: String,
    pub urgency_band: Option<String>,
    pub status: String,
    pub session_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTestResult {
    pub id: i64,
    pub blueprint_id: i64,
    pub session_id: i64,
    pub raw_score_bp: BasisPoints,
    pub adjusted_readiness_bp: BasisPoints,
    pub readiness_band: Option<String>,
    pub careless_error_count: i64,
    pub endurance_drop_bp: BasisPoints,
    pub speed_band: Option<String>,
    pub next_recommended_action: Option<String>,
    pub next_mode_recommendation: Option<String>,
    pub interpretation_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAdaptationState {
    pub session_id: i64,
    pub current_streak: i64,
    pub best_streak: i64,
    pub error_streak: i64,
    pub same_misconception_streak: i64,
    pub avg_response_time_ms: i64,
    pub confidence_drop_count: i64,
    pub stabilizer_questions_inserted: i64,
    pub difficulty_adjustments: i64,
    pub fatigue_indicator_bp: BasisPoints,
    pub pressure_response_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentTestReadiness {
    pub student_id: i64,
    pub subject_id: i64,
    pub test_type: String,
    pub readiness_bp: BasisPoints,
    pub readiness_band: String,
    pub attempt_count: i64,
    pub trend: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockBlueprintInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub question_count: usize,
    pub duration_minutes: Option<i64>,
    pub is_timed: bool,
    pub target_difficulty: Option<BasisPoints>,
    pub weakness_bias: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockBlueprint {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub title: String,
    pub blueprint_type: String,
    pub duration_minutes: Option<i64>,
    pub question_count: i64,
    pub readiness_score: BasisPoints,
    pub readiness_band: String,
    pub coverage: Value,
    pub quotas: Value,
    pub compiled_question_ids: Vec<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: i64,
    pub accuracy_score: Option<i64>,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTopicInterpretation {
    pub topic_id: i64,
    pub topic_name: String,
    pub attempts: i64,
    pub correct_attempts: i64,
    pub accuracy_score: BasisPoints,
    pub avg_response_time_ms: Option<i64>,
    pub dominant_error_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInterpretation {
    pub session_id: i64,
    pub student_id: i64,
    pub session_type: String,
    pub status: String,
    pub observed_at: DateTime<Utc>,
    pub is_timed: bool,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub incorrect_questions: i64,
    pub unanswered_questions: i64,
    pub accuracy_score: Option<BasisPoints>,
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
    pub topic_summaries: Vec<SessionTopicInterpretation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvidenceFabric {
    pub session_id: i64,
    pub student_id: i64,
    pub session_type: String,
    pub status: String,
    pub interpretation: SessionInterpretation,
    pub remediation_plans: Vec<QuestionRemediationPlan>,
    pub signals: Vec<FabricSignal>,
    pub evidence_records: Vec<FabricEvidenceRecord>,
    pub orchestration: FabricOrchestrationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionItem {
    pub id: i64,
    pub session_id: i64,
    pub question_id: i64,
    pub display_order: i64,
    pub source_family_id: Option<i64>,
    pub source_topic_id: Option<i64>,
    pub status: String,
    pub selected_option_id: Option<i64>,
    pub flagged: bool,
    pub response_time_ms: Option<i64>,
    pub is_correct: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session: Session,
    pub items: Vec<SessionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnswerInput {
    pub item_id: i64,
    pub selected_option_id: i64,
    pub response_time_ms: Option<i64>,
}
