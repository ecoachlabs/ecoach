use serde::{Deserialize, Serialize};

pub type BasisPoints = u16;
pub const ACCOUNT_PIN_LENGTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Student,
    Parent,
    Admin,
    SuperAdmin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Student,
    Parent,
    Admin,
}

impl AccountType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Student => "student",
            Self::Parent => "parent",
            Self::Admin => "admin",
        }
    }

    pub fn pin_len(self) -> usize {
        ACCOUNT_PIN_LENGTH
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementTier {
    Standard,
    Premium,
    Elite,
}

impl EntitlementTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Premium => "premium",
            Self::Elite => "elite",
        }
    }
}
