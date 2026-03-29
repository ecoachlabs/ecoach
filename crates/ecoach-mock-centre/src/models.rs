use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockStatus {
    Created,
    Active,
    Paused,
    TimeUp,
    Completed,
    Abandoned,
}

impl MockStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::TimeUp => "time_up",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockGrade {
    A1,
    B2,
    B3,
    C4,
    C5,
    C6,
    D7,
    E8,
    F9,
}

impl MockGrade {
    pub fn from_percentage(pct: f64) -> Self {
        match pct {
            p if p >= 75.0 => Self::A1,
            p if p >= 70.0 => Self::B2,
            p if p >= 65.0 => Self::B3,
            p if p >= 60.0 => Self::C4,
            p if p >= 55.0 => Self::C5,
            p if p >= 50.0 => Self::C6,
            p if p >= 45.0 => Self::D7,
            p if p >= 40.0 => Self::E8,
            _ => Self::F9,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::A1 => "A1",
            Self::B2 => "B2",
            Self::B3 => "B3",
            Self::C4 => "C4",
            Self::C5 => "C5",
            Self::C6 => "C6",
            Self::D7 => "D7",
            Self::E8 => "E8",
            Self::F9 => "F9",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileMockInput {
    pub student_id: i64,
    pub subject_id: i64,
    /// Total duration of mock exam in minutes
    pub duration_minutes: i64,
    /// Total number of questions
    pub question_count: usize,
    /// Optional: restrict to specific topics (empty = all topics in subject)
    pub topic_ids: Vec<i64>,
    /// Paper year for past paper simulation (optional)
    pub paper_year: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitMockAnswerInput {
    pub mock_session_id: i64,
    pub question_id: i64,
    pub selected_option_id: i64,
    pub confidence_level: Option<String>,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSession {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub session_id: i64,
    pub status: String,
    pub duration_minutes: i64,
    pub time_remaining_seconds: Option<i64>,
    pub question_count: i64,
    pub answered_count: i64,
    pub paper_year: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAnswerResult {
    pub question_id: i64,
    pub was_correct: bool,
    pub answered_count: i64,
    pub remaining_count: i64,
    pub time_remaining_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockReport {
    pub mock_session_id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub grade: String,
    pub total_score: i64,
    pub max_score: i64,
    pub percentage: f64,
    pub accuracy_bp: BasisPoints,
    pub time_used_seconds: i64,
    pub total_time_seconds: i64,
    pub questions_answered: i64,
    pub questions_correct: i64,
    pub questions_unanswered: i64,
    pub topic_breakdown: Vec<MockTopicScore>,
    pub improvement_vs_last: Option<ImprovementDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTopicScore {
    pub topic_id: i64,
    pub topic_name: String,
    pub correct: i64,
    pub total: i64,
    pub accuracy_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementDelta {
    pub previous_grade: String,
    pub previous_percentage: f64,
    pub delta_percentage: f64,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionSummary {
    pub id: i64,
    pub subject_id: i64,
    pub grade: Option<String>,
    pub percentage: Option<f64>,
    pub status: String,
    pub paper_year: Option<String>,
    pub created_at: String,
}
