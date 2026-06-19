use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrustLevel {
    Unverified,
    Verified,
    Blocked,
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Unverified => write!(f, "UNVERIFIED"),
            TrustLevel::Verified => write!(f, "VERIFIED"),
            TrustLevel::Blocked => write!(f, "BLOCKED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub username: String,
    pub public_key: String,
    pub fingerprint: String,
    pub trust_level: TrustLevel,
    pub account_status: String,
    pub added_at: DateTime<Utc>,
    pub notes: String,
}
