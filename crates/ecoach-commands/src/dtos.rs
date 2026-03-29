use chrono::{DateTime, Utc};
use ecoach_content::{PackInstallResult, PackSummary};
use ecoach_games::{ContrastPairSummary, TrapRoundResult, TrapSessionReview, TrapSessionSnapshot};
use ecoach_identity::{Account, AccountSummary};
use ecoach_sessions::{MockBlueprint, SessionSnapshot, SessionSummary};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub entitlement_tier: String,
    pub status: String,
    pub failed_pin_attempts: i64,
    pub is_locked: bool,
    pub needs_checkup: bool,
    pub last_active_label: String,
}

impl From<Account> for AccountDto {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            account_type: value.account_type.as_str().to_string(),
            entitlement_tier: value.entitlement_tier.as_str().to_string(),
            status: value.status,
            failed_pin_attempts: value.failed_pin_attempts,
            is_locked: value.locked_until.is_some(),
            needs_checkup: value.first_run,
            last_active_label: last_active_label(value.last_active_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummaryDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub status: String,
    pub needs_checkup: bool,
    pub last_active_label: String,
}

impl From<AccountSummary> for AccountSummaryDto {
    fn from(value: AccountSummary) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            account_type: value.account_type.as_str().to_string(),
            status: value.status,
            needs_checkup: value.first_run,
            last_active_label: last_active_label(value.last_active_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInstallResultDto {
    pub pack_id: String,
    pub pack_version: String,
    pub install_path: String,
}

impl From<PackInstallResult> for PackInstallResultDto {
    fn from(value: PackInstallResult) -> Self {
        Self {
            pack_id: value.pack_id,
            pack_version: value.pack_version,
            install_path: value.install_path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSummaryDto {
    pub pack_id: String,
    pub pack_version: String,
    pub subject_code: String,
    pub status: String,
}

impl From<PackSummary> for PackSummaryDto {
    fn from(value: PackSummary) -> Self {
        Self {
            pack_id: value.pack_id,
            pack_version: value.pack_version,
            subject_code: value.subject_code,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshotDto {
    pub session_id: i64,
    pub session_type: String,
    pub status: String,
    pub active_item_index: i64,
    pub item_count: usize,
}

impl From<SessionSnapshot> for SessionSnapshotDto {
    fn from(value: SessionSnapshot) -> Self {
        Self {
            session_id: value.session.id,
            session_type: value.session.session_type,
            status: value.session.status,
            active_item_index: value.session.active_item_index,
            item_count: value.items.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryDto {
    pub session_id: i64,
    pub accuracy_score: Option<i64>,
    pub answered_questions: i64,
    pub correct_questions: i64,
    pub status: String,
}

impl From<SessionSummary> for SessionSummaryDto {
    fn from(value: SessionSummary) -> Self {
        Self {
            session_id: value.session_id,
            accuracy_score: value.accuracy_score,
            answered_questions: value.answered_questions,
            correct_questions: value.correct_questions,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockBlueprintDto {
    pub id: i64,
    pub title: String,
    pub blueprint_type: String,
    pub question_count: i64,
    pub readiness_score: i64,
    pub readiness_band: String,
    pub coverage: Value,
    pub status: String,
}

impl From<MockBlueprint> for MockBlueprintDto {
    fn from(value: MockBlueprint) -> Self {
        Self {
            id: value.id,
            title: value.title,
            blueprint_type: value.blueprint_type,
            question_count: value.question_count,
            readiness_score: value.readiness_score as i64,
            readiness_band: value.readiness_band,
            coverage: value.coverage,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastPairSummaryDto {
    pub pair_id: i64,
    pub title: String,
    pub left_label: String,
    pub right_label: String,
    pub confusion_score: i64,
    pub recommended_mode: String,
}

impl From<ContrastPairSummary> for ContrastPairSummaryDto {
    fn from(value: ContrastPairSummary) -> Self {
        Self {
            pair_id: value.pair_id,
            title: value.title,
            left_label: value.left_label,
            right_label: value.right_label,
            confusion_score: value.confusion_score as i64,
            recommended_mode: value.recommended_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSessionSnapshotDto {
    pub session_id: i64,
    pub mode: String,
    pub pair_title: String,
    pub left_label: String,
    pub right_label: String,
    pub round_count: usize,
    pub active_round_number: i64,
}

impl From<TrapSessionSnapshot> for TrapSessionSnapshotDto {
    fn from(value: TrapSessionSnapshot) -> Self {
        Self {
            session_id: value.session.id,
            mode: value.state.mode,
            pair_title: value.state.pair_title,
            left_label: value.left_label,
            right_label: value.right_label,
            round_count: value.rounds.len(),
            active_round_number: value.state.current_round_number,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapRoundResultDto {
    pub round_id: i64,
    pub round_number: i64,
    pub is_correct: bool,
    pub score_earned: i64,
    pub correct_choice_label: String,
    pub explanation_text: String,
    pub session_complete: bool,
}

impl From<TrapRoundResult> for TrapRoundResultDto {
    fn from(value: TrapRoundResult) -> Self {
        Self {
            round_id: value.round_id,
            round_number: value.round_number,
            is_correct: value.is_correct,
            score_earned: value.score_earned,
            correct_choice_label: value.correct_choice_label,
            explanation_text: value.explanation_text,
            session_complete: value.session_complete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapReviewDto {
    pub session_id: i64,
    pub pair_title: String,
    pub mode: String,
    pub score: i64,
    pub accuracy_bp: i64,
    pub confusion_score: i64,
    pub round_count: usize,
}

impl From<TrapSessionReview> for TrapReviewDto {
    fn from(value: TrapSessionReview) -> Self {
        Self {
            session_id: value.session_id,
            pair_title: value.pair_title,
            mode: value.mode,
            score: value.score,
            accuracy_bp: value.accuracy_bp as i64,
            confusion_score: value.confusion_score as i64,
            round_count: value.rounds.len(),
        }
    }
}

fn last_active_label(last_active_at: Option<DateTime<Utc>>) -> String {
    let Some(last_active_at) = last_active_at else {
        return "Never active".to_string();
    };
    let delta = Utc::now() - last_active_at;
    if delta.num_hours() < 24 {
        "Active today".to_string()
    } else {
        format!("Away {} days", delta.num_days())
    }
}
