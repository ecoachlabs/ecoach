use serde::{Deserialize, Serialize};

pub type BasisPoints = u16;

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

    pub fn min_pin_len(self) -> usize {
        match self {
            Self::Student => 4,
            Self::Parent | Self::Admin => 6,
        }
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
