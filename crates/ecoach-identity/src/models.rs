use chrono::{DateTime, Utc};
use ecoach_substrate::{AccountType, EntitlementTier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountInput {
    pub account_type: AccountType,
    pub display_name: String,
    pub pin: String,
    pub entitlement_tier: EntitlementTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub account_type: AccountType,
    pub display_name: String,
    pub entitlement_tier: EntitlementTier,
    pub failed_pin_attempts: i64,
    pub locked_until: Option<DateTime<Utc>>,
    pub status: String,
    pub first_run: bool,
    pub last_active_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: i64,
    pub display_name: String,
    pub account_type: AccountType,
    pub status: String,
    pub first_run: bool,
    pub last_active_at: Option<DateTime<Utc>>,
}
