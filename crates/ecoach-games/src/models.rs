use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapsMode {
    DifferenceDrill,
    SimilarityTrap,
    KnowTheDifference,
    WhichIsWhich,
    Unmask,
}

impl TrapsMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DifferenceDrill => "difference_drill",
            Self::SimilarityTrap => "similarity_trap",
            Self::KnowTheDifference => "know_the_difference",
            Self::WhichIsWhich => "which_is_which",
            Self::Unmask => "unmask",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "difference_drill" => Some(Self::DifferenceDrill),
            "similarity_trap" => Some(Self::SimilarityTrap),
            "know_the_difference" => Some(Self::KnowTheDifference),
            "which_is_which" => Some(Self::WhichIsWhich),
            "unmask" => Some(Self::Unmask),
            _ => None,
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTrapsSessionInput {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_ids: Vec<i64>,
    pub pair_id: Option<i64>,
    pub mode: TrapsMode,
    pub round_count: usize,
    pub timer_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTrapRoundInput {
    pub game_session_id: i64,
    pub round_id: i64,
    pub selected_choice_code: Option<String>,
    pub response_time_ms: i64,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTrapConfusionReasonInput {
    pub round_id: i64,
    pub reason_code: String,
    pub reason_text: Option<String>,
}

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
    #[serde(default)]
    pub focus_signals: Vec<String>,
    pub recommended_next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLeaderboardEntry {
    pub student_id: i64,
    pub display_name: String,
    pub game_type: String,
    pub best_score: i64,
    pub games_played: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindstackState {
    pub board_height: i64,
    pub cleared_rows: i64,
    pub pending_block_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TugOfWarState {
    pub position: i64,
    pub opponent_difficulty: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapsState {
    pub pair_id: i64,
    pub pair_title: String,
    pub mode: String,
    pub correct_discriminations: i64,
    pub total_discriminations: i64,
    pub confusion_score: BasisPoints,
    pub current_round_id: Option<i64>,
    pub current_round_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapChoiceOption {
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundCard {
    pub id: i64,
    pub round_number: i64,
    pub pair_id: i64,
    pub mode: String,
    pub lane: String,
    pub prompt_text: String,
    pub prompt_payload: Value,
    pub answer_options: Vec<TrapChoiceOption>,
    pub reveal_count: i64,
    pub max_reveal_count: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSessionSnapshot {
    pub session: GameSession,
    pub state: TrapsState,
    pub left_label: String,
    pub right_label: String,
    pub summary_text: Option<String>,
    pub recommended_mode: String,
    pub rounds: Vec<TrapRoundCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundResult {
    pub round_id: i64,
    pub round_number: i64,
    pub is_correct: bool,
    pub score_earned: i64,
    pub new_score: i64,
    pub streak: i64,
    pub session_complete: bool,
    pub selected_choice_code: Option<String>,
    pub selected_choice_label: Option<String>,
    pub correct_choice_code: String,
    pub correct_choice_label: String,
    pub explanation_text: String,
    pub confusion_signal: String,
    pub next_round_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapReviewRound {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSessionReview {
    pub session_id: i64,
    pub pair_id: i64,
    pub pair_title: String,
    pub mode: String,
    pub score: i64,
    pub accuracy_bp: BasisPoints,
    pub confusion_score: BasisPoints,
    pub weakest_lane: Option<String>,
    pub timed_out_count: i64,
    pub recommended_next_mode: String,
    pub dominant_confusion_reason: Option<String>,
    #[serde(default)]
    pub remediation_actions: Vec<String>,
    pub rounds: Vec<TrapReviewRound>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastPairSummary {
    pub pair_id: i64,
    pub pair_code: Option<String>,
    pub title: String,
    pub left_label: String,
    pub right_label: String,
    pub summary_text: Option<String>,
    pub trap_strength: BasisPoints,
    pub difficulty_score: BasisPoints,
    pub confusion_score: BasisPoints,
    pub last_accuracy_bp: BasisPoints,
    pub recommended_mode: String,
    pub available_modes: Vec<String>,
}
