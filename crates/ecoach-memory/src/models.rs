use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

// ── Memory state machine ──
// seen → encoded → accessible → { anchoring | fragile }
// anchoring → confirmed → locked_in
// fragile → { fading | rebuilding }
// fading → { collapsed | recovered }
// at_risk can branch from several states

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    Seen,
    Encoded,
    Accessible,
    Fragile,
    Anchoring,
    Confirmed,
    LockedIn,
    AtRisk,
    Fading,
    Rebuilding,
    Recovered,
    Collapsed,
}

impl MemoryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Seen => "seen",
            Self::Encoded => "encoded",
            Self::Accessible => "accessible",
            Self::Fragile => "fragile",
            Self::Anchoring => "anchoring",
            Self::Confirmed => "confirmed",
            Self::LockedIn => "locked_in",
            Self::AtRisk => "at_risk",
            Self::Fading => "fading",
            Self::Rebuilding => "rebuilding",
            Self::Recovered => "recovered",
            Self::Collapsed => "collapsed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "seen" => Some(Self::Seen),
            "encoded" => Some(Self::Encoded),
            "accessible" => Some(Self::Accessible),
            "fragile" => Some(Self::Fragile),
            "anchoring" => Some(Self::Anchoring),
            "confirmed" => Some(Self::Confirmed),
            "locked_in" => Some(Self::LockedIn),
            "at_risk" => Some(Self::AtRisk),
            "fading" => Some(Self::Fading),
            "rebuilding" => Some(Self::Rebuilding),
            "recovered" => Some(Self::Recovered),
            "collapsed" => Some(Self::Collapsed),
            _ => None,
        }
    }

    pub fn is_healthy(self) -> bool {
        matches!(
            self,
            Self::Accessible | Self::Anchoring | Self::Confirmed | Self::LockedIn | Self::Recovered
        )
    }
}

// ── Recall modes ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallMode {
    FreeRecall,
    CuedRecall,
    Recognition,
    Transfer,
}

impl RecallMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreeRecall => "free_recall",
            Self::CuedRecall => "cued_recall",
            Self::Recognition => "recognition",
            Self::Transfer => "transfer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CueLevel {
    None,
    Light,
    Heavy,
}

impl CueLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Light => "light",
            Self::Heavy => "heavy",
        }
    }
}

// ── Inputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMemoryEvidenceInput {
    pub student_id: i64,
    pub node_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub recall_mode: RecallMode,
    pub cue_level: CueLevel,
    pub delay_bucket: String,
    pub interference_detected: bool,
    pub was_correct: bool,
    pub confidence_level: Option<String>,
}

// ── Outputs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateRecord {
    pub id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub memory_state: String,
    pub memory_strength: BasisPoints,
    pub recall_fluency: BasisPoints,
    pub decay_risk: BasisPoints,
    pub review_due_at: Option<String>,
    pub last_recalled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecheckItem {
    pub id: i64,
    pub student_id: i64,
    pub node_id: Option<i64>,
    pub due_at: String,
    pub schedule_type: String,
    pub status: String,
    pub memory_state: Option<String>,
    pub memory_strength: Option<BasisPoints>,
    pub decay_risk: Option<BasisPoints>,
    pub topic_name: Option<String>,
    pub node_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayBatchResult {
    pub items_processed: usize,
    pub items_decayed: usize,
    pub items_collapsed: usize,
    pub new_rechecks_scheduled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceEdge {
    pub id: i64,
    pub from_node_id: i64,
    pub to_node_id: i64,
    pub strength_score: BasisPoints,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDashboard {
    pub student_id: i64,
    pub total_items: i64,
    pub healthy_count: i64,
    pub at_risk_count: i64,
    pub fading_count: i64,
    pub collapsed_count: i64,
    pub overdue_reviews: i64,
    pub average_strength: BasisPoints,
    pub next_review_due: Option<String>,
}
