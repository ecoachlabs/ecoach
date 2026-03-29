use chrono::{DateTime, Utc};
use ecoach_substrate::BasisPoints;
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
