use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

// ── Game types (canonical: mindstack, tug_of_war, traps/difference_mastery) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameType {
    Mindstack,
    TugOfWar,
    Traps,
}

impl GameType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mindstack => "mindstack",
            Self::TugOfWar => "tug_of_war",
            Self::Traps => "traps",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mindstack" => Some(Self::Mindstack),
            "tug_of_war" => Some(Self::TugOfWar),
            "traps" => Some(Self::Traps),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameSessionStatus {
    Created,
    Active,
    Paused,
    Completed,
    Abandoned,
}

impl GameSessionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartGameInput {
    pub student_id: i64,
    pub game_type: GameType,
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub question_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitGameAnswerInput {
    pub game_session_id: i64,
    pub question_id: i64,
    pub selected_option_id: i64,
    pub response_time_ms: i64,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: i64,
    pub student_id: i64,
    pub game_type: String,
    pub subject_id: i64,
    pub session_state: String,
    pub score: i64,
    pub rounds_total: i64,
    pub rounds_played: i64,
    pub streak: i64,
    pub best_streak: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnswerResult {
    pub is_correct: bool,
    pub points_earned: i64,
    pub new_score: i64,
    pub streak: i64,
    pub effect_type: String,
    pub round_number: i64,
    pub session_complete: bool,
    pub explanation: Option<String>,
    pub misconception_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummary {
    pub session_id: i64,
    pub game_type: String,
    pub score: i64,
    pub accuracy_bp: BasisPoints,
    pub rounds_played: i64,
    pub best_streak: i64,
    pub average_response_time_ms: i64,
    pub misconception_hits: i64,
    pub performance_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLeaderboardEntry {
    pub student_id: i64,
    pub display_name: String,
    pub game_type: String,
    pub best_score: i64,
    pub games_played: i64,
}

// ── Mindstack-specific state ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindstackState {
    pub board_height: i64,
    pub cleared_rows: i64,
    pub pending_block_type: String,
}

// ── TugOfWar-specific state ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TugOfWarState {
    pub position: i64, // -10..=10, 0 is center; positive = student winning
    pub opponent_difficulty: BasisPoints,
}

// ── Traps/DifferenceMastery-specific state ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapsState {
    pub pair_id: i64,
    pub correct_discriminations: i64,
    pub total_discriminations: i64,
}
